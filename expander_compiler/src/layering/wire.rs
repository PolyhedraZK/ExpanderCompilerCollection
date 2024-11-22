use std::collections::HashMap;

use crate::{
    circuit::{
        config::Config,
        input_mapping::EMPTY,
        ir::expr::VarSpec,
        layered::{Allocation, Coef, GateAdd, GateConst, GateCustom, GateMul, Input, Segment},
    },
    field::FieldArith,
    utils::pool::Pool,
};

use super::{
    compile::{CompileContext, IrContext},
    layer_layout::{subs_map, LayerLayout, LayerLayoutInner, SubLayout},
};

struct LayoutQuery {
    var_pos: HashMap<usize, usize>,
}

impl LayoutQuery {
    // given a parent layer layout, this function query the layout of a sub circuit
    fn query<F, C: Config>(
        &self,
        layer_layout_pool: &mut Pool<LayerLayout>,
        circuits: &HashMap<usize, IrContext<'_, C>>,
        vs: &[usize], // variables to query (in parent layer)
        f: F,         // f(i) = id of i-th variable in the sub circuit
        cid: usize,   // target circuit id
        lid: usize,   // target layer id
    ) -> SubLayout
    where
        F: Fn(usize) -> usize,
    {
        if vs.is_empty() {
            let subl = LayerLayout {
                circuit_id: cid,
                layer: lid,
                size: 1,
                inner: LayerLayoutInner::Dense {
                    placement: vec![EMPTY],
                },
            };
            let id = layer_layout_pool.add(&subl);
            return SubLayout {
                id,
                offset: 0,
                insn_id: EMPTY,
            };
        }
        let mut ps = vec![0; vs.len()];
        let mut l: usize = 1 << 62;
        let mut r: usize = 0;
        for (i, x) in vs.iter().enumerate() {
            ps[i] = if let Some(x) = self.var_pos.get(x) {
                *x
            } else {
                EMPTY
            };
            if ps[i] != EMPTY {
                l = l.min(ps[i]);
                r = r.max(ps[i]);
            }
        }
        let mut xor = if l <= r { l ^ r } else { 0 };
        xor |= xor >> 1;
        xor |= xor >> 2;
        xor |= xor >> 4;
        xor |= xor >> 8;
        xor |= xor >> 16;
        xor |= xor >> 32;
        xor ^= xor >> 1;
        let n = if xor == 0 { 1 } else { xor << 1 };
        let offset = if l <= r { l & !(n - 1) } else { 0 };
        let mut placement = vec![EMPTY; n];
        for i in 0..vs.len() {
            if ps[i] != EMPTY {
                placement[ps[i] - offset] = f(i);
            }
        }
        subs_map(&mut placement, circuits[&cid].lcs[lid].vars.map());
        let subl = LayerLayout {
            circuit_id: cid,
            layer: lid,
            size: n,
            inner: LayerLayoutInner::Dense { placement },
        };
        let id = layer_layout_pool.add(&subl);
        SubLayout {
            id,
            offset,
            insn_id: EMPTY,
        }
    }
}

impl<'a, C: Config> CompileContext<'a, C> {
    fn layout_query(&self, l: &LayerLayout, s: &[usize]) -> LayoutQuery {
        let mut var_pos = HashMap::new();
        match &l.inner {
            LayerLayoutInner::Dense { placement } => {
                for (i, v) in placement.iter().enumerate() {
                    if *v != EMPTY {
                        if var_pos.contains_key(&s[*v]) {
                            panic!("unexpected situation");
                        }
                        var_pos.insert(s[*v], i);
                    }
                }
            }
            LayerLayoutInner::Sparse { placement, .. } => {
                for (i, v) in placement.iter() {
                    if var_pos.contains_key(&s[*v]) {
                        panic!("unexpected situation");
                    }
                    var_pos.insert(s[*v], *i);
                }
            }
        }
        LayoutQuery { var_pos }
    }

    pub fn connect_wires(&mut self, layout_ids: &[usize]) -> Vec<usize> {
        let layouts = layout_ids
            .iter()
            .map(|x| self.layer_layout_pool.get(*x).clone())
            .collect::<Vec<_>>();
        for (a, b) in layouts.iter().zip(layouts.iter().skip(1)) {
            if a.layer + 1 != b.layer || a.circuit_id != b.circuit_id {
                panic!("unexpected situation");
            }
        }
        for (i, a) in layouts.iter().enumerate() {
            if i != a.layer {
                panic!("unexpected situation");
            }
        }
        let circuit_id = layouts[0].circuit_id;
        let ic = self.circuits.remove(&circuit_id).unwrap();
        if layouts.len() != ic.output_layer + 1 {
            panic!("unexpected situation");
        }
        let lqs = layouts
            .iter()
            .map(|x| self.layout_query(x, ic.lcs[x.layer].vars.vec()))
            .collect::<Vec<_>>();

        for (i, (lc, lq)) in ic
            .lcs
            .iter()
            .zip(lqs.iter())
            .take(ic.output_layer)
            .enumerate()
        {
            for x in lc.vars.vec() {
                if !lq.var_pos.contains_key(x) {
                    panic!("unexpected situation");
                }
            }
        }

        let mut sub_layouts_of_layer: Vec<HashMap<usize, SubLayout>> =
            vec![HashMap::new(); ic.output_layer + 1];

        // find all sub circuits
        for (i, insn_id) in ic.sub_circuit_insn_ids.iter().enumerate() {
            let insn = &ic.sub_circuit_insn_refs[i];
            let sub_id = insn.sub_circuit_id;
            let sub_c = &self.circuits[&sub_id];
            let dep = sub_c.output_layer;
            let input_layer = ic.sub_circuit_start_layer[i];
            let output_layer = input_layer + dep;

            sub_layouts_of_layer[input_layer].insert(
                *insn_id,
                lqs[input_layer].query(
                    &mut self.layer_layout_pool,
                    &self.circuits,
                    &insn.inputs,
                    |x| x + 1,
                    sub_id,
                    0,
                ),
            );
            sub_layouts_of_layer[output_layer].insert(
                *insn_id,
                lqs[output_layer].query(
                    &mut self.layer_layout_pool,
                    &self.circuits,
                    &insn.outputs,
                    |x| sub_c.circuit.outputs[x],
                    sub_id,
                    dep,
                ),
            );
        }

        // fill already known sub_layouts
        for (i, a) in layouts.iter().enumerate() {
            if let LayerLayoutInner::Sparse { sub_layout, .. } = &a.inner {
                for x in sub_layout.iter() {
                    sub_layouts_of_layer[i].insert(x.insn_id, x.clone());
                }
            }
        }

        let mut ress: Vec<Segment<C>> = Vec::new();
        for (a, b) in layouts.iter().zip(layouts.iter().skip(1)) {
            ress.push(Segment {
                num_inputs: vec![a.size],
                num_outputs: b.size,
                ..Default::default()
            });
        }

        // connect sub circuits
        for (i, insn_id) in ic.sub_circuit_insn_ids.iter().enumerate() {
            let insn = &ic.sub_circuit_insn_refs[i];
            let sub_id = insn.sub_circuit_id;
            let sub_c = &self.circuits[&sub_id];
            let dep = sub_c.output_layer;
            let input_layer = ic.sub_circuit_start_layer[i];
            let output_layer = input_layer + dep;

            let cur_sub_layout_ids = (input_layer..=output_layer)
                .map(|x| sub_layouts_of_layer[x][insn_id].id)
                .collect::<Vec<_>>();
            let segment_ids = self.connect_wires(&cur_sub_layout_ids);

            for (i, segment_id) in segment_ids.iter().enumerate() {
                let al = Allocation {
                    input_offset: vec![sub_layouts_of_layer[input_layer + i][insn_id].offset],
                    output_offset: sub_layouts_of_layer[input_layer + i + 1][insn_id].offset,
                };
                let mut found = false;
                let child_segs = &mut ress[input_layer + i].child_segs;
                for j in 0..=child_segs.len() {
                    if j == child_segs.len() {
                        child_segs.push((*segment_id, vec![al]));
                        found = true;
                        break;
                    }
                    if child_segs[j].0 == *segment_id {
                        child_segs[j].1.push(al);
                        found = true;
                        break;
                    }
                }
                if !found {
                    panic!("unexpected situation");
                }
            }
        }

        // connect self variables
        for (cur_layer, ((lc, bq), aq)) in ic
            .lcs
            .iter()
            .zip(lqs.iter())
            .skip(1)
            .zip(lqs.iter())
            .enumerate()
        {
            let next_layer = cur_layer + 1;
            let res = &mut ress[cur_layer];
            for x in lc.vars.vec().iter() {
                // only consider real variables
                if *x >= ic.num_var {
                    continue;
                }
                let pos = if let Some(p) = bq.var_pos.get(x) {
                    *p
                } else {
                    assert_eq!(cur_layer + 1, ic.output_layer);
                    //assert!(!ic.output_order.contains_key(x));
                    continue;
                };
                // if it's not the first layer, just relay it
                if ic.min_layer[*x] != next_layer {
                    res.gate_adds.push(GateAdd {
                        inputs: [Input::new(0, aq.var_pos[x])],
                        output: pos,
                        coef: Coef::Constant(C::CircuitField::one()),
                    });
                    continue;
                }
                if let Some(value) = ic.constant_like_variables.get(x) {
                    res.gate_consts.push(GateConst {
                        inputs: [],
                        output: pos,
                        coef: value.clone(),
                    });
                } else if ic.internal_variable_expr.contains_key(x) {
                    for term in ic.internal_variable_expr[x].iter() {
                        match &term.vars {
                            VarSpec::Const => {
                                res.gate_consts.push(GateConst {
                                    inputs: [],
                                    output: pos,
                                    coef: Coef::Constant(term.coef),
                                });
                            }
                            VarSpec::Linear(vid) => {
                                res.gate_adds.push(GateAdd {
                                    inputs: [Input::new(0, aq.var_pos[vid])],
                                    output: pos,
                                    coef: Coef::Constant(term.coef),
                                });
                            }
                            VarSpec::Quad(vid0, vid1) => {
                                res.gate_muls.push(GateMul {
                                    inputs: [
                                        Input::new(0, aq.var_pos[vid0]),
                                        Input::new(0, aq.var_pos[vid1]),
                                    ],
                                    output: pos,
                                    coef: Coef::Constant(term.coef),
                                });
                            }
                            VarSpec::Custom { gate_type, inputs } => {
                                res.gate_customs.push(GateCustom {
                                    gate_type: *gate_type,
                                    inputs: inputs
                                        .iter()
                                        .map(|x| Input::new(0, aq.var_pos[x]))
                                        .collect(),
                                    output: pos,
                                    coef: Coef::Constant(term.coef),
                                });
                            }
                            VarSpec::RandomLinear(vid) => {
                                res.gate_adds.push(GateAdd {
                                    inputs: [Input::new(0, aq.var_pos[vid])],
                                    output: pos,
                                    coef: Coef::Random,
                                });
                            }
                        }
                    }
                }
            }
        }

        // also combined output variables
        for (cur_layer, ((cc, bq), aq)) in ic
            .combined_constraints
            .iter()
            .zip(lqs.iter())
            .skip(1)
            .zip(lqs.iter())
            .enumerate()
        {
            let res = &mut ress[cur_layer];
            if let Some(cc) = cc {
                let pos = bq.var_pos[&cc.id];
                for v in cc.variables.iter() {
                    let coef = if *v >= ic.num_var {
                        Coef::Constant(C::CircuitField::one())
                    } else {
                        Coef::Random
                    };
                    res.gate_adds.push(GateAdd {
                        inputs: [Input::new(0, aq.var_pos[v])],
                        output: pos,
                        coef,
                    });
                }
                for i in cc.sub_circuit_ids.iter() {
                    let insn_id = ic.sub_circuit_insn_ids[*i];
                    let insn = &ic.sub_circuit_insn_refs[*i];
                    let input_layer = ic.sub_circuit_start_layer[*i];
                    let vid = self.circuits[&insn.sub_circuit_id].combined_constraints
                        [cur_layer - input_layer]
                        .as_ref()
                        .unwrap()
                        .id;
                    let vpid = self.circuits[&insn.sub_circuit_id].lcs[cur_layer - input_layer]
                        .vars
                        .get_idx(&vid);
                    let layout = self
                        .layer_layout_pool
                        .get(sub_layouts_of_layer[cur_layer][&insn_id].id);
                    let spos = match &layout.inner {
                        LayerLayoutInner::Sparse { placement, .. } => placement
                            .iter()
                            .find_map(|(i, v)| if *v == vpid { Some(*i) } else { None })
                            .unwrap(),
                        LayerLayoutInner::Dense { placement } => {
                            placement.iter().position(|x| *x == vpid).unwrap()
                        }
                    };
                    res.gate_adds.push(GateAdd {
                        inputs: [Input::new(
                            0,
                            sub_layouts_of_layer[cur_layer][&insn_id].offset + spos,
                        )],
                        output: pos,
                        coef: Coef::Constant(C::CircuitField::one()),
                    });
                }
            }
        }

        let mut ress_ids = Vec::new();

        for res in ress.iter() {
            let res_id = self.compiled_circuits.len();
            self.compiled_circuits.push(res.clone());
            ress_ids.push(res_id);
        }
        self.circuits.insert(circuit_id, ic);

        ress_ids
    }
}

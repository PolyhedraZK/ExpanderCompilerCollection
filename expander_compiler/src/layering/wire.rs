use std::collections::HashMap;

use crate::{
    circuit::{
        config::Config,
        input_mapping::EMPTY,
        ir::expr::VarSpec,
        layered::{Allocation, Coef, GateAdd, GateConst, GateMul, Segment},
    },
    field::Field,
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
    fn query<'a, F, C: Config>(
        &self,
        layer_layout_pool: &mut Pool<LayerLayout>,
        circuits: &HashMap<usize, IrContext<'a, C>>,
        vs: &Vec<usize>,
        f: F,
        cid: usize,
        lid: isize,
    ) -> SubLayout
    where
        F: Fn(usize) -> usize,
    {
        if vs.len() == 0 {
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
        while xor != 0 && (xor & (xor - 1)) != 0 {
            xor &= xor - 1;
        }
        let n = if xor == 0 { 1 } else { xor << 1 };
        let offset = if l <= r { l & !(n - 1) } else { 0 };
        let mut placement = vec![EMPTY; n];
        for i in 0..vs.len() {
            if ps[i] != EMPTY {
                placement[ps[i] - offset] = f(i);
            }
        }
        if lid >= 0 {
            subs_map(&mut placement, circuits[&cid].lcs[lid as usize].vars.map());
        } else {
            subs_map(&mut placement, circuits[&cid].lc_hint.vars.map());
        }
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
    fn layout_query(&self, l: &LayerLayout, s: &Vec<usize>) -> LayoutQuery {
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

    pub fn connect_wires(&mut self, a_: usize, b_: usize) -> usize {
        let map_id = (a_ as u128) << 64 | b_ as u128;
        if let Some(x) = self.conncected_wires.get(&map_id) {
            return *x;
        }
        let a = self.layer_layout_pool.get(a_).clone();
        let b = self.layer_layout_pool.get(b_).clone();
        if (a.layer + 1 != b.layer && (a.layer != -1 || b.layer != -1))
            || a.circuit_id != b.circuit_id
        {
            panic!("unexpected situation");
        }
        let circuit_id = a.circuit_id.clone();
        let ic = self.circuits.remove(&circuit_id).unwrap();
        let cur_layer = a.layer;
        let next_layer = b.layer;
        let (cur_lc, next_lc) = if cur_layer >= 0 {
            (&ic.lcs[cur_layer as usize], &ic.lcs[next_layer as usize])
        } else {
            (&ic.lc_hint, &ic.lc_hint)
        };
        let aq = self.layout_query(&a, cur_lc.vars.vec());
        let bq = self.layout_query(&b, next_lc.vars.vec());

        /*println!(
            "connect_wires: {} {} circuit_id={} cur_layer={} output_layer={}",
            a_, b_, a.circuit_id, cur_layer, ic.output_layer
        );
        println!("cur: {:?}", a.inner);
        println!("next: {:?}", b.inner);
        println!("cur_var: {:?}", cur_lc.vars.vec());
        println!("next_var: {:?}", next_lc.vars.vec());*/

        // check if all variables exist in the layout
        for x in cur_lc.vars.vec().iter() {
            if !aq.var_pos.contains_key(x) {
                panic!("unexpected situation");
            }
        }
        if cur_layer + 1 != ic.output_layer as isize {
            for x in next_lc.vars.vec().iter() {
                if !bq.var_pos.contains_key(x) {
                    panic!("unexpected situation");
                }
            }
        }

        let mut sub_insns: Pool<usize> = Pool::new();
        let mut sub_cur_layout: Vec<Option<SubLayout>> = Vec::new();
        let mut sub_next_layout: Vec<Option<SubLayout>> = Vec::new();
        let mut sub_cur_layout_all: HashMap<usize, SubLayout> = HashMap::new();

        // find all sub circuits
        for (i, insn_id) in ic.sub_circuit_insn_ids.iter().enumerate() {
            let insn = &ic.sub_circuit_insn_refs[i];
            let sub_id = insn.sub_circuit_id;
            let sub_c = &self.circuits[&sub_id];
            let dep = sub_c.output_layer;
            let input_layer = ic.sub_circuit_start_layer[i];
            let output_layer = input_layer + dep;
            let mut cur_layout = None;
            let mut next_layout = None;
            let outf = |x: usize| -> usize { sub_c.circuit.outputs[x] };
            let hintf = |x: usize| -> usize { *sub_c.hint_inputs.get(x) };
            if input_layer as isize <= cur_layer && output_layer as isize >= next_layer {
                // normal
                if input_layer as isize == cur_layer {
                    // for the input layer, we need to manually query the layout. (other layers are already subLayouts)
                    let mut vs = insn.inputs.clone();
                    vs.extend(ic.sub_circuit_hint_inputs[i].iter());
                    cur_layout = Some(aq.query(
                        &mut self.layer_layout_pool,
                        &self.circuits,
                        &vs,
                        |x| {
                            if x < insn.inputs.len() {
                                x + 1
                            } else {
                                *sub_c.hint_inputs.get(x - insn.inputs.len())
                            }
                        },
                        sub_id,
                        0,
                    ));
                }
                if output_layer as isize == next_layer {
                    // also for the output layer
                    next_layout = Some(bq.query(
                        &mut self.layer_layout_pool,
                        &mut self.circuits,
                        &insn.outputs,
                        outf,
                        sub_id,
                        dep as isize,
                    ));
                }
            } else if next_layer != -1
                && next_layer <= input_layer as isize
                && ic.sub_circuit_hint_inputs[i].len() != 0
            {
                // relay hint input
                if cur_layer == 0 {
                    cur_layout = Some(aq.query(
                        &mut self.layer_layout_pool,
                        &self.circuits,
                        &ic.sub_circuit_hint_inputs[i],
                        hintf,
                        sub_id,
                        -1,
                    ));
                }
                if next_layer == input_layer as isize {
                    next_layout = Some(bq.query(
                        &mut self.layer_layout_pool,
                        &self.circuits,
                        &ic.sub_circuit_hint_inputs[i],
                        hintf,
                        sub_id,
                        -1,
                    ));
                }
            } else if cur_layer == output_layer as isize {
                cur_layout = Some(aq.query(
                    &mut self.layer_layout_pool,
                    &self.circuits,
                    &insn.outputs,
                    outf,
                    sub_id,
                    dep as isize,
                ));
                sub_cur_layout_all.insert(*insn_id, cur_layout.unwrap());
                continue;
            } else {
                continue;
            }
            sub_insns.add(insn_id);
            sub_cur_layout.push(cur_layout);
            sub_next_layout.push(next_layout);
        }

        // fill already known subLayouts
        let a = self.layer_layout_pool.get(a_);
        let b = self.layer_layout_pool.get(b_);
        // fill already known sub_layouts
        if let LayerLayoutInner::Sparse { sub_layout, .. } = &a.inner {
            for x in sub_layout.iter() {
                sub_cur_layout[sub_insns.get_idx(&x.insn_id)] = Some(x.clone());
            }
        }
        if let LayerLayoutInner::Sparse { sub_layout, .. } = &b.inner {
            for x in sub_layout.iter() {
                sub_next_layout[sub_insns.get_idx(&x.insn_id)] = Some(x.clone());
            }
        }

        let mut res: Segment<C> = Segment::default();
        res.num_inputs = a.size;
        res.num_outputs = b.size;

        // connect sub circuits
        for i in 0..sub_insns.len() {
            let sub_cur_layout = sub_cur_layout[i].as_ref().unwrap();
            let sub_next_layout = sub_next_layout[i].as_ref().unwrap();
            sub_cur_layout_all.insert(*sub_insns.get(i), sub_cur_layout.clone());
            let scid = self.connect_wires(sub_cur_layout.id, sub_next_layout.id);
            let al = Allocation {
                input_offset: sub_cur_layout.offset,
                output_offset: sub_next_layout.offset,
            };
            let mut found = false;
            for j in 0..=res.child_segs.len() {
                if j == res.child_segs.len() {
                    res.child_segs.push((scid, vec![al]));
                    found = true;
                    break;
                }
                if res.child_segs[j].0 == scid {
                    res.child_segs[j].1.push(al);
                    found = true;
                    break;
                }
            }
            if !found {
                panic!("unexpected situation");
            }
        }

        // connect self variables
        for x in next_lc.vars.vec().iter() {
            // only consider real variables, except it's hint relay layer
            if *x >= ic.num_var && cur_layer != -1 {
                continue;
            }
            let pos = if let Some(p) = bq.var_pos.get(x) {
                *p
            } else {
                assert_eq!(cur_layer + 1, ic.output_layer as isize);
                //assert!(!ic.output_order.contains_key(x));
                continue;
            };
            // if it's not the first layer, just relay it
            if ic.min_layer[*x] as isize != next_layer || cur_layer == -1 {
                res.gate_adds.push(GateAdd {
                    inputs: [aq.var_pos[x]],
                    output: pos,
                    coef: Coef::Constant(C::CircuitField::one()),
                });
                continue;
            }
            if let Some(value) = ic.constant_or_random_variables.get(x) {
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
                                coef: Coef::Constant(term.coef.clone()),
                            });
                        }
                        VarSpec::Linear(vid) => {
                            res.gate_adds.push(GateAdd {
                                inputs: [aq.var_pos[vid]],
                                output: pos,
                                coef: Coef::Constant(term.coef.clone()),
                            });
                        }
                        VarSpec::Quad(vid0, vid1) => {
                            res.gate_muls.push(GateMul {
                                inputs: [aq.var_pos[vid0], aq.var_pos[vid1]],
                                output: pos,
                                coef: Coef::Constant(term.coef.clone()),
                            });
                        }
                    }
                }
            }
        }

        // also combined output variables
        let cc = if next_layer >= 0 {
            ic.combined_constraints[next_layer as usize].as_ref()
        } else {
            None
        };
        if let Some(cc) = cc {
            let pos = bq.var_pos[&cc.id];
            for v in cc.variables.iter() {
                let coef = if *v >= ic.num_var {
                    Coef::Constant(C::CircuitField::one())
                } else {
                    Coef::Random
                };
                res.gate_adds.push(GateAdd {
                    inputs: [aq.var_pos[v]],
                    output: pos,
                    coef: coef,
                });
            }
            for i in cc.sub_circuit_ids.iter() {
                let insn_id = ic.sub_circuit_insn_ids[*i];
                let insn = &ic.sub_circuit_insn_refs[*i];
                let input_layer = ic.sub_circuit_start_layer[*i];
                let vid = self.circuits[&insn.sub_circuit_id].combined_constraints
                    [cur_layer as usize - input_layer]
                    .as_ref()
                    .unwrap()
                    .id;
                let vpid = self.circuits[&insn.sub_circuit_id].lcs
                    [cur_layer as usize - input_layer]
                    .vars
                    .get_idx(&vid);
                let layout = self.layer_layout_pool.get(sub_cur_layout_all[&insn_id].id);
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
                    inputs: [sub_cur_layout_all[&insn_id].offset + spos],
                    output: pos,
                    coef: Coef::Constant(C::CircuitField::one()),
                });
            }
        }

        let res_id = self.compiled_circuits.len();
        self.compiled_circuits.push(res);
        self.conncected_wires.insert(map_id, res_id);
        self.circuits.insert(circuit_id.clone(), ic);

        res_id
    }
}

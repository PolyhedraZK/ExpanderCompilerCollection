use std::collections::HashMap;

use crate::circuit::{config::Config, input_mapping::EMPTY, ir::dest::Instruction};

use super::{compile::CompileContext, layer_layout::LayerLayoutInner};

impl<'a, C: Config> CompileContext<'a, C> {
    pub fn record_input_order(&self) -> Vec<usize> {
        let layout_id = self.layout_ids[0];
        let l = self.layer_layout_pool.get(layout_id);
        let placement = match &l.inner {
            LayerLayoutInner::Dense { placement } => placement,
            _ => {
                panic!("unexpected situation");
            }
        };
        let lc = &self.circuits[&0].lcs[0];
        let mut v = HashMap::new();
        for (i, x) in placement.iter().cloned().enumerate() {
            if x != EMPTY {
                v.insert(*lc.vars.get(x), i);
            }
        }
        let mut gi = Vec::new();
        let circuit = self.circuits[&0].circuit;
        for i in 1..=circuit.num_inputs {
            if let Some(&vi) = v.get(&i) {
                gi.push(vi);
            } else {
                gi.push(EMPTY);
            }
        }
        self.record_sub_circuit_hint_input_order(0, v, &mut gi);
        gi
    }

    fn record_sub_circuit_hint_input_order(
        &self,
        sub_id: usize,
        v: HashMap<usize, usize>,
        res: &mut Vec<usize>,
    ) {
        let ic = &self.circuits[&sub_id];
        let mut hint_input_sub_idx = ic.num_var;
        for i in ic.circuit.num_inputs + 1..=ic.circuit.num_inputs + ic.circuit.num_hint_inputs {
            if let Some(vi) = v.get(&i) {
                res.push(*vi);
            } else {
                res.push(EMPTY);
            }
        }
        for insn in ic.circuit.instructions.iter() {
            if let Instruction::SubCircuitCall { sub_circuit_id, .. } = insn {
                let subc = &self.circuits[sub_circuit_id];
                let mut sv = HashMap::new();
                for x in subc.hint_inputs.vec().iter() {
                    if let Some(vi) = v.get(&hint_input_sub_idx) {
                        sv.insert(*x, *vi);
                    }
                    hint_input_sub_idx += 1;
                }
                self.record_sub_circuit_hint_input_order(*sub_circuit_id, sv, res);
            }
        }
    }
}

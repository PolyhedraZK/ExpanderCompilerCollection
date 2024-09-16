use super::*;

impl<C: Config> Circuit<C> {
    pub fn export_to_expander<
        DestConfig: expander_rs::GKRConfig<CircuitField = C::CircuitField>,
    >(
        &self,
    ) -> expander_rs::RecursiveCircuit<DestConfig> {
        let segments = self
            .segments
            .iter()
            .map(|seg| expander_rs::Segment {
                i_var_num: seg.num_inputs.trailing_zeros() as usize,
                o_var_num: seg.num_outputs.trailing_zeros() as usize,
                gate_muls: seg
                    .gate_muls
                    .iter()
                    .map(|gate| gate.export_to_expander())
                    .collect(),
                gate_adds: seg
                    .gate_adds
                    .iter()
                    .map(|gate| gate.export_to_expander())
                    .collect(),
                gate_consts: seg
                    .gate_consts
                    .iter()
                    .map(|gate| gate.export_to_expander())
                    .collect(),
                gate_uni: seg
                    .gate_customs
                    .iter()
                    .map(|gate| {
                        let (c, r) = gate.coef.export_to_expander();
                        expander_rs::GateUni {
                            i_ids: [gate.inputs[0]],
                            o_id: gate.output,
                            coef: c,
                            is_random: r,
                            gate_type: gate.gate_type,
                        }
                    })
                    .collect(),
                child_segs: seg
                    .child_segs
                    .iter()
                    .map(|seg| {
                        (
                            seg.0,
                            seg.1
                                .iter()
                                .map(|alloc| expander_rs::Allocation {
                                    i_offset: alloc.input_offset,
                                    o_offset: alloc.output_offset,
                                })
                                .collect(),
                        )
                    })
                    .collect(),
            })
            .collect();
        expander_rs::RecursiveCircuit {
            segments,
            layers: self.layer_ids.clone(),
        }
    }
}

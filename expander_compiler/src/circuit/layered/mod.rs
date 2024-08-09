use std::fmt;

use crate::field::Field;

use super::config::Config;

#[cfg(test)]
mod tests;

pub mod serde;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Coef<C: Config> {
    Constant(C::CircuitField),
    Random,
}

impl<C: Config> Coef<C> {
    pub fn get_value_unsafe(&self) -> C::CircuitField {
        match self {
            Coef::Constant(c) => c.clone(),
            Coef::Random => C::CircuitField::random_unsafe(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Gate<C: Config, const INPUT_NUM: usize> {
    pub inputs: [usize; INPUT_NUM],
    pub output: usize,
    pub coef: Coef<C>,
}

pub type GateMul<C> = Gate<C, 2>;
pub type GateAdd<C> = Gate<C, 1>;
pub type GateConst<C> = Gate<C, 0>;

pub struct Allocation {
    pub input_offset: usize,
    pub output_offset: usize,
}

pub type ChildSpec = (usize, Vec<Allocation>);

#[derive(Default)]
pub struct Segment<C: Config> {
    pub num_inputs: usize,
    pub num_outputs: usize,
    pub child_segs: Vec<ChildSpec>,
    pub gate_muls: Vec<GateMul<C>>,
    pub gate_adds: Vec<GateAdd<C>>,
    pub gate_consts: Vec<GateConst<C>>,
}

pub struct Circuit<C: Config> {
    pub segments: Vec<Segment<C>>,
    pub layer_ids: Vec<usize>,
}

impl<C: Config> Circuit<C> {
    pub fn validate(&self) -> Result<(), String> {
        for (i, seg) in self.segments.iter().enumerate() {
            if seg.num_inputs == 0 || (seg.num_inputs & (seg.num_inputs - 1)) != 0 {
                return Err(format!(
                    "segment {} inputlen {} not power of 2",
                    i, seg.num_inputs
                ));
            }
            if seg.num_outputs == 0 || (seg.num_outputs & (seg.num_outputs - 1)) != 0 {
                return Err(format!(
                    "segment {} outputlen {} not power of 2",
                    i, seg.num_outputs
                ));
            }
            for m in seg.gate_muls.iter() {
                if m.inputs[0] >= seg.num_inputs
                    || m.inputs[1] >= seg.num_inputs
                    || m.output >= seg.num_outputs
                {
                    return Err(format!(
                        "segment {} mul gate ({}, {}, {}) out of range",
                        i, m.inputs[0], m.inputs[1], m.output
                    ));
                }
            }
            for a in seg.gate_adds.iter() {
                if a.inputs[0] >= seg.num_inputs || a.output >= seg.num_outputs {
                    return Err(format!(
                        "segment {} add gate ({}, {}) out of range",
                        i, a.inputs[0], a.output
                    ));
                }
            }
            for cs in seg.gate_consts.iter() {
                if cs.output >= seg.num_outputs {
                    return Err(format!(
                        "segment {} const gate {} out of range",
                        i, cs.output
                    ));
                }
            }
            for (sub_id, allocs) in seg.child_segs.iter() {
                if *sub_id >= i {
                    return Err(format!("segment {} subcircuit {} out of range", i, sub_id));
                }
                let subc = &self.segments[*sub_id];
                for a in allocs.iter() {
                    if a.input_offset % subc.num_inputs != 0 {
                        return Err(format!(
                            "segment {} subcircuit {} input offset {} not aligned to {}",
                            i, sub_id, a.input_offset, subc.num_inputs
                        ));
                    }
                    if a.input_offset + subc.num_inputs > seg.num_inputs {
                        return Err(format!(
                            "segment {} subcircuit {} input offset {} out of range",
                            i, sub_id, a.input_offset
                        ));
                    }
                    if a.output_offset % subc.num_outputs != 0 {
                        return Err(format!(
                            "segment {} subcircuit {} output offset {} not aligned to {}",
                            i, sub_id, a.output_offset, subc.num_outputs
                        ));
                    }
                    if a.output_offset + subc.num_outputs > seg.num_outputs {
                        return Err(format!(
                            "segment {} subcircuit {} output offset {} out of range",
                            i, sub_id, a.output_offset
                        ));
                    }
                }
            }
        }
        for x in self.layer_ids.iter() {
            if *x >= self.segments.len() {
                return Err(format!("layer id {} out of range", x));
            }
        }
        if self.layer_ids.len() == 0 {
            return Err("empty layer".to_string());
        }
        for i in 1..self.layer_ids.len() {
            let cur = &self.segments[self.layer_ids[i]];
            let prev = &self.segments[self.layer_ids[i - 1]];
            if cur.num_inputs != prev.num_outputs {
                return Err(format!(
                    "segment {} inputlen {} not equal to segment {} outputlen {}",
                    self.layer_ids[i],
                    cur.num_inputs,
                    self.layer_ids[i - 1],
                    prev.num_outputs
                ));
            }
        }
        let (input_mask, output_mask) = self.compute_masks();
        for i in 1..self.layer_ids.len() {
            for j in 0..self.segments[self.layer_ids[i]].num_inputs {
                if input_mask[self.layer_ids[i]][j] && !output_mask[self.layer_ids[i - 1]][j] {
                    return Err(format!(
                        "circuit {} input {} not initialized by circuit {} output",
                        self.layer_ids[i],
                        j,
                        self.layer_ids[i - 1]
                    ));
                }
            }
        }
        Ok(())
    }

    fn compute_masks(&self) -> (Vec<Vec<bool>>, Vec<Vec<bool>>) {
        let mut input_mask: Vec<Vec<bool>> = Vec::with_capacity(self.segments.len());
        let mut output_mask: Vec<Vec<bool>> = Vec::with_capacity(self.segments.len());
        for seg in self.segments.iter() {
            let mut input_mask_seg = vec![false; seg.num_inputs];
            let mut output_mask_seg = vec![false; seg.num_outputs];
            for m in seg.gate_muls.iter() {
                input_mask_seg[m.inputs[0]] = true;
                input_mask_seg[m.inputs[1]] = true;
                output_mask_seg[m.output] = true;
            }
            for a in seg.gate_adds.iter() {
                input_mask_seg[a.inputs[0]] = true;
                output_mask_seg[a.output] = true;
            }
            for cs in seg.gate_consts.iter() {
                output_mask_seg[cs.output] = true;
            }
            for (sub_id, allocs) in seg.child_segs.iter() {
                let subc = &self.segments[*sub_id];
                for a in allocs.iter() {
                    for j in 0..subc.num_inputs {
                        input_mask_seg[a.input_offset + j] =
                            input_mask_seg[a.input_offset + j] || input_mask[*sub_id][j];
                    }
                    for j in 0..subc.num_outputs {
                        output_mask_seg[a.output_offset + j] =
                            output_mask_seg[a.output_offset + j] || output_mask[*sub_id][j];
                    }
                }
            }
            input_mask.push(input_mask_seg);
            output_mask.push(output_mask_seg);
        }
        (input_mask, output_mask)
    }

    pub fn input_size(&self) -> usize {
        self.segments[self.layer_ids[0]].num_inputs
    }

    pub fn eval_unsafe(&self, input: Vec<C::CircuitField>) -> Vec<C::CircuitField> {
        if input.len() != self.input_size() {
            panic!("input length mismatch");
        }
        let mut cur = input;
        for &id in self.layer_ids.iter() {
            let mut next = vec![C::CircuitField::zero(); self.segments[id].num_outputs];
            self.apply_segment_unsafe(&self.segments[id], &cur, &mut next);
            cur = next;
        }
        cur
    }

    fn apply_segment_unsafe(
        &self,
        seg: &Segment<C>,
        cur: &[C::CircuitField],
        nxt: &mut [C::CircuitField],
    ) {
        for m in seg.gate_muls.iter() {
            nxt[m.output] += cur[m.inputs[0]] * cur[m.inputs[1]] * m.coef.get_value_unsafe();
        }
        for a in seg.gate_adds.iter() {
            nxt[a.output] += cur[a.inputs[0]] * a.coef.get_value_unsafe();
        }
        for cs in seg.gate_consts.iter() {
            nxt[cs.output] += cs.coef.get_value_unsafe();
        }
        for (sub_id, allocs) in seg.child_segs.iter() {
            let subc = &self.segments[*sub_id];
            for a in allocs.iter() {
                self.apply_segment_unsafe(
                    subc,
                    &cur[a.input_offset..a.input_offset + subc.num_inputs],
                    &mut nxt[a.output_offset..a.output_offset + subc.num_outputs],
                );
            }
        }
    }
}

impl<C: Config> fmt::Display for Coef<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Coef::Constant(c) => write!(f, "{}", c),
            Coef::Random => write!(f, "Random"),
        }
    }
}

impl<C: Config> fmt::Display for Segment<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "input={} output={}", self.num_inputs, self.num_outputs)?;
        for (sub_id, allocs) in self.child_segs.iter() {
            writeln!(f, "apply circuit {} at:", sub_id)?;
            for a in allocs.iter() {
                writeln!(
                    f,
                    "    input_offset={} output_offset={}",
                    a.input_offset, a.output_offset
                )?;
            }
        }
        for m in self.gate_muls.iter() {
            writeln!(
                f,
                "out{} += in{} * in{} * {}",
                m.output, m.inputs[0], m.inputs[1], m.coef
            )?;
        }
        for a in self.gate_adds.iter() {
            writeln!(f, "out{} += in{} * {}", a.output, a.inputs[0], a.coef)?;
        }
        for cs in self.gate_consts.iter() {
            writeln!(f, "out{} += {}", cs.output, cs.coef)?;
        }
        Ok(())
    }
}

impl<C: Config> fmt::Display for Circuit<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, seg) in self.segments.iter().enumerate() {
            write!(f, "Circuit {}: {}", i, seg)?;
            writeln!(f, "================================")?;
        }
        writeln!(f, "Layers: {:?}", self.layer_ids)?;
        Ok(())
    }
}

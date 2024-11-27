use std::{fmt, hash::Hash};

use arith::FieldForECC;

use crate::{field::FieldArith, hints, utils::error::Error};

use super::config::Config;

#[cfg(test)]
mod tests;

pub mod export;
pub mod opt;
pub mod serde;
pub mod stats;
pub mod witness;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Coef<C: Config> {
    Constant(C::CircuitField),
    Random,
    PublicInput(usize),
}

impl<C: Config> Coef<C> {
    pub fn get_value_unsafe(&self) -> C::CircuitField {
        match self {
            Coef::Constant(c) => *c,
            Coef::Random => C::CircuitField::random_unsafe(&mut rand::thread_rng()),
            Coef::PublicInput(id) => {
                // stub implementation
                let t = id * id % 1000000007;
                let t = t * id % 1000000007;
                C::CircuitField::from(t as u32)
            }
        }
    }

    pub fn get_value_with_public_inputs(
        &self,
        public_inputs: &[C::CircuitField],
    ) -> C::CircuitField {
        match self {
            Coef::Constant(c) => *c,
            Coef::Random => C::CircuitField::random_unsafe(&mut rand::thread_rng()),
            Coef::PublicInput(id) => {
                if *id >= public_inputs.len() {
                    panic!("public input id {} out of range", id);
                }
                public_inputs[*id]
            }
        }
    }

    pub fn validate(&self, num_public_inputs: usize) -> Result<(), Error> {
        match self {
            Coef::Constant(_) => Ok(()),
            Coef::Random => Ok(()),
            Coef::PublicInput(id) => {
                if *id >= num_public_inputs {
                    Err(Error::UserError(format!(
                        "public input id {} out of range",
                        id
                    )))
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn is_constant(&self) -> bool {
        matches!(self, Coef::Constant(_))
    }

    pub fn add_constant(&self, c: C::CircuitField) -> Self {
        match self {
            Coef::Constant(x) => Coef::Constant(*x + c),
            _ => panic!("add_constant called on non-constant"),
        }
    }

    pub fn get_constant(&self) -> Option<C::CircuitField> {
        match self {
            Coef::Constant(x) => Some(*x),
            _ => None,
        }
    }

    #[cfg(test)]
    pub fn random_no_random(mut rnd: impl rand::RngCore, num_public_inputs: usize) -> Self {
        use rand::Rng;
        if rnd.gen::<f64>() < 0.94 {
            Coef::Constant(C::CircuitField::from(rnd.next_u32()))
        } else {
            Coef::PublicInput(rnd.next_u64() as usize % num_public_inputs)
        }
    }

    pub fn export_to_expander(&self) -> (C::CircuitField, expander_circuit::CoefType) {
        match self {
            Coef::Constant(c) => (*c, expander_circuit::CoefType::Constant),
            Coef::Random => (C::CircuitField::zero(), expander_circuit::CoefType::Random),
            Coef::PublicInput(x) => (
                C::CircuitField::zero(),
                expander_circuit::CoefType::PublicInput(*x),
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Input {
    pub layer: usize,
    pub offset: usize,
}

impl Input {
    pub fn new(layer: usize, offset: usize) -> Self {
        Self { layer, offset }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Gate<C: Config, const INPUT_NUM: usize> {
    pub inputs: [Input; INPUT_NUM],
    pub output: usize,
    pub coef: Coef<C>,
}

impl<C: Config, const INPUT_NUM: usize> Gate<C, INPUT_NUM> {
    pub fn export_to_expander<
        DestConfig: expander_config::GKRConfig<CircuitField = C::CircuitField>,
    >(
        &self,
    ) -> expander_circuit::Gate<DestConfig, INPUT_NUM> {
        panic!("TODO")
    }
}

pub type GateMul<C> = Gate<C, 2>;
pub type GateAdd<C> = Gate<C, 1>;
pub type GateConst<C> = Gate<C, 0>;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GateCustom<C: Config> {
    pub gate_type: usize,
    pub inputs: Vec<Input>,
    pub output: usize,
    pub coef: Coef<C>,
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct Allocation {
    pub input_offset: Vec<usize>,
    pub output_offset: usize,
}

pub type ChildSpec = (usize, Vec<Allocation>);

#[derive(Default, Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct Segment<C: Config> {
    pub num_inputs: Vec<usize>,
    pub num_outputs: usize,
    pub child_segs: Vec<ChildSpec>,
    pub gate_muls: Vec<GateMul<C>>,
    pub gate_adds: Vec<GateAdd<C>>,
    pub gate_consts: Vec<GateConst<C>>,
    pub gate_customs: Vec<GateCustom<C>>,
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct Circuit<C: Config> {
    pub num_public_inputs: usize,
    pub num_actual_outputs: usize,
    pub expected_num_output_zeroes: usize,
    pub segments: Vec<Segment<C>>,
    pub layer_ids: Vec<usize>,
}

impl<C: Config> Circuit<C> {
    pub fn validate(&self) -> Result<(), Error> {
        for (i, seg) in self.segments.iter().enumerate() {
            for (j, x) in seg.num_inputs.iter().enumerate() {
                if *x == 0 || (*x & (*x - 1)) != 0 {
                    return Err(Error::InternalError(format!(
                        "segment {} input {} len {} not power of 2",
                        i, j, x
                    )));
                }
            }
            if seg.num_inputs.len() == 0 {
                return Err(Error::InternalError(format!("segment {} inputlen 0", i)));
            }
            if seg.num_outputs == 0 || (seg.num_outputs & (seg.num_outputs - 1)) != 0 {
                return Err(Error::InternalError(format!(
                    "segment {} outputlen {} not power of 2",
                    i, seg.num_outputs
                )));
            }
            for m in seg.gate_muls.iter() {
                if m.inputs[0].layer >= self.layer_ids.len() {
                    return Err(Error::InternalError(format!(
                        "segment {} mul gate ({:?}, {:?}, {}) input 0 layer out of range",
                        i, m.inputs[0], m.inputs[1], m.output
                    )));
                }
                if m.inputs[1].layer >= self.layer_ids.len() {
                    return Err(Error::InternalError(format!(
                        "segment {} mul gate ({:?}, {:?}, {}) input 1 layer out of range",
                        i, m.inputs[0], m.inputs[1], m.output
                    )));
                }
                if m.inputs[0].offset >= seg.num_inputs[m.inputs[0].layer] {
                    return Err(Error::InternalError(format!(
                        "segment {} mul gate ({:?}, {:?}, {}) input 0 out of range",
                        i, m.inputs[0], m.inputs[1], m.output
                    )));
                }
                if m.inputs[1].offset >= seg.num_inputs[m.inputs[1].layer] {
                    return Err(Error::InternalError(format!(
                        "segment {} mul gate ({:?}, {:?}, {}) input 1 out of range",
                        i, m.inputs[0], m.inputs[1], m.output
                    )));
                }
                if m.output >= seg.num_outputs {
                    return Err(Error::InternalError(format!(
                        "segment {} mul gate ({:?}, {:?}, {}) out of range",
                        i, m.inputs[0], m.inputs[1], m.output
                    )));
                }
            }
            for a in seg.gate_adds.iter() {
                if a.inputs[0].layer >= self.layer_ids.len() {
                    return Err(Error::InternalError(format!(
                        "segment {} add gate ({:?}, {}) input layer out of range",
                        i, a.inputs[0], a.output
                    )));
                }
                if a.inputs[0].offset >= seg.num_inputs[a.inputs[0].layer] {
                    return Err(Error::InternalError(format!(
                        "segment {} add gate ({:?}, {}) input out of range",
                        i, a.inputs[0], a.output
                    )));
                }
                if a.output >= seg.num_outputs {
                    return Err(Error::InternalError(format!(
                        "segment {} add gate ({:?}, {}) out of range",
                        i, a.inputs[0], a.output
                    )));
                }
            }
            for cs in seg.gate_consts.iter() {
                if cs.output >= seg.num_outputs {
                    return Err(Error::InternalError(format!(
                        "segment {} const gate {} out of range",
                        i, cs.output
                    )));
                }
            }
            for cu in seg.gate_customs.iter() {
                for input in cu.inputs.iter() {
                    if input.layer >= self.layer_ids.len() {
                        return Err(Error::InternalError(format!(
                            "segment {} custom gate {} input layer out of range",
                            i, cu.output
                        )));
                    }
                    if input.offset >= seg.num_inputs[input.layer] {
                        return Err(Error::InternalError(format!(
                            "segment {} custom gate {} input out of range",
                            i, cu.output
                        )));
                    }
                }
                if cu.output >= seg.num_outputs {
                    return Err(Error::InternalError(format!(
                        "segment {} custom gate {} out of range",
                        i, cu.output
                    )));
                }
            }
            for (sub_id, allocs) in seg.child_segs.iter() {
                if *sub_id >= i {
                    return Err(Error::InternalError(format!(
                        "segment {} subcircuit {} out of range",
                        i, sub_id
                    )));
                }
                let subc = &self.segments[*sub_id];
                if subc.num_inputs.len() > seg.num_inputs.len() {
                    return Err(Error::InternalError(format!(
                        "segment {} subcircuit {} input length {} larger than {}",
                        i,
                        sub_id,
                        subc.num_inputs.len(),
                        seg.num_inputs.len()
                    )));
                }
                for a in allocs.iter() {
                    if a.input_offset.len() != subc.num_inputs.len() {
                        return Err(Error::InternalError(format!(
                            "segment {} subcircuit {} input offset {:?} length not equal to {}",
                            i,
                            sub_id,
                            a.input_offset,
                            subc.num_inputs.len()
                        )));
                    }
                    for ((x, y), z) in a
                        .input_offset
                        .iter()
                        .zip(subc.num_inputs.iter())
                        .zip(seg.num_inputs.iter())
                    {
                        if *x % *y != 0 {
                            return Err(Error::InternalError(format!(
                                "segment {} subcircuit {} input offset {} not aligned to {}",
                                i, sub_id, x, y
                            )));
                        }
                        if *x + *y > *z {
                            return Err(Error::InternalError(format!(
                                "segment {} subcircuit {} input offset {} out of range",
                                i, sub_id, x
                            )));
                        }
                    }
                    if a.output_offset % subc.num_outputs != 0 {
                        return Err(Error::InternalError(format!(
                            "segment {} subcircuit {} output offset {} not aligned to {}",
                            i, sub_id, a.output_offset, subc.num_outputs
                        )));
                    }
                    if a.output_offset + subc.num_outputs > seg.num_outputs {
                        return Err(Error::InternalError(format!(
                            "segment {} subcircuit {} output offset {} out of range",
                            i, sub_id, a.output_offset
                        )));
                    }
                }
            }
        }
        for x in self.layer_ids.iter() {
            if *x >= self.segments.len() {
                return Err(Error::InternalError(format!("layer id {} out of range", x)));
            }
        }
        if self.layer_ids.is_empty() {
            return Err(Error::InternalError("empty layer".to_string()));
        }
        let mut layer_sizes = Vec::with_capacity(self.layer_ids.len() + 1);
        layer_sizes.push(self.segments[self.layer_ids[0]].num_inputs[0]);
        for l in self.layer_ids.iter() {
            layer_sizes.push(self.segments[*l].num_outputs);
        }
        for (i, l) in self.layer_ids.iter().enumerate() {
            let cur = &self.segments[*l];
            if cur.num_inputs.len() > i + 1 {
                return Err(Error::InternalError(format!(
                    "layer {} input length {} larger than {}",
                    i,
                    cur.num_inputs.len(),
                    i + 1
                )));
            }
            for (j, x) in cur.num_inputs.iter().enumerate() {
                if *x != layer_sizes[i - j] {
                    return Err(Error::InternalError(format!(
                        "layer {} input {} length {} not equal to {}",
                        i,
                        j,
                        x,
                        layer_sizes[i - j]
                    )));
                }
            }
        }
        let (input_mask, output_mask) = self.compute_masks();
        for i in 1..self.layer_ids.len() {
            for (l, len) in self.segments[self.layer_ids[i]]
                .num_inputs
                .iter()
                .enumerate()
            {
                if i == l {
                    // if this is also the global input, it's always initialized
                    continue;
                }
                for j in 0..*len {
                    if input_mask[self.layer_ids[i]][l][j]
                        && !output_mask[self.layer_ids[i - 1 - l]][j]
                    {
                        println!("{:?}", self.segments[17]);
                        println!("{:?}", self.segments[18]);
                        return Err(Error::InternalError(format!(
                            "circuit {} (layer {}) input {} not initialized by circuit {} (layer {}) output",
                            self.layer_ids[i],
                            i,
                            j,
                            self.layer_ids[i - 1 - l],
                            i - 1 - l
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    fn compute_masks(&self) -> (Vec<Vec<Vec<bool>>>, Vec<Vec<bool>>) {
        let mut input_mask: Vec<Vec<Vec<bool>>> = Vec::with_capacity(self.segments.len());
        let mut output_mask: Vec<Vec<bool>> = Vec::with_capacity(self.segments.len());
        for seg in self.segments.iter() {
            let mut input_mask_seg: Vec<Vec<bool>> =
                seg.num_inputs.iter().map(|&x| vec![false; x]).collect();
            let mut output_mask_seg = vec![false; seg.num_outputs];
            for m in seg.gate_muls.iter() {
                input_mask_seg[m.inputs[0].layer][m.inputs[0].offset] = true;
                input_mask_seg[m.inputs[1].layer][m.inputs[1].offset] = true;
                output_mask_seg[m.output] = true;
            }
            for a in seg.gate_adds.iter() {
                input_mask_seg[a.inputs[0].layer][a.inputs[0].offset] = true;
                output_mask_seg[a.output] = true;
            }
            for cs in seg.gate_consts.iter() {
                output_mask_seg[cs.output] = true;
            }
            for cu in seg.gate_customs.iter() {
                for input in cu.inputs.iter() {
                    input_mask_seg[input.layer][input.offset] = true;
                }
                output_mask_seg[cu.output] = true;
            }
            for (sub_id, allocs) in seg.child_segs.iter() {
                let subc = &self.segments[*sub_id];
                for a in allocs.iter() {
                    for (l, (off, len)) in a
                        .input_offset
                        .iter()
                        .zip(subc.num_inputs.iter())
                        .enumerate()
                    {
                        for i in 0..*len {
                            input_mask_seg[l][*off + i] =
                                input_mask_seg[l][*off + i] || input_mask[*sub_id][l][i];
                        }
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
        self.segments[self.layer_ids[0]].num_inputs[0]
    }

    pub fn eval_unsafe(&self, inputs: Vec<C::CircuitField>) -> (Vec<C::CircuitField>, bool) {
        if inputs.len() != self.input_size() {
            panic!("input length mismatch");
        }
        let mut cur = vec![inputs];
        for &id in self.layer_ids.iter() {
            let mut next = vec![C::CircuitField::zero(); self.segments[id].num_outputs];
            let mut inputs: Vec<&[C::CircuitField]> = Vec::new();
            for i in 0..self.segments[id].num_inputs.len() {
                inputs.push(&cur[cur.len() - i - 1]);
            }
            self.apply_segment_unsafe(&self.segments[id], &inputs, &mut next);
            cur.push(next);
        }
        let cur = cur.last().unwrap();
        let mut constraints_satisfied = true;
        for out in cur.iter().take(self.expected_num_output_zeroes) {
            if !out.is_zero() {
                constraints_satisfied = false;
                break;
            }
        }
        (
            cur[self.expected_num_output_zeroes..self.num_actual_outputs].to_vec(),
            constraints_satisfied,
        )
    }

    fn apply_segment_unsafe(
        &self,
        seg: &Segment<C>,
        cur: &[&[C::CircuitField]],
        nxt: &mut [C::CircuitField],
    ) {
        for m in seg.gate_muls.iter() {
            nxt[m.output] += cur[m.inputs[0].layer][m.inputs[0].offset]
                * cur[m.inputs[1].layer][m.inputs[1].offset]
                * m.coef.get_value_unsafe();
        }
        for a in seg.gate_adds.iter() {
            nxt[a.output] += cur[a.inputs[0].layer][a.inputs[0].offset] * a.coef.get_value_unsafe();
        }
        for cs in seg.gate_consts.iter() {
            nxt[cs.output] += cs.coef.get_value_unsafe();
        }
        for cu in seg.gate_customs.iter() {
            let mut inputs = Vec::with_capacity(cu.inputs.len());
            for input in cu.inputs.iter() {
                inputs.push(cur[input.layer][input.offset]);
            }
            let outputs = hints::stub_impl(cu.gate_type, &inputs, 1);
            for (i, &output) in outputs.iter().enumerate() {
                nxt[cu.output + i] += output * cu.coef.get_value_unsafe();
            }
        }
        for (sub_id, allocs) in seg.child_segs.iter() {
            let subc = &self.segments[*sub_id];
            for a in allocs.iter() {
                let inputs = a
                    .input_offset
                    .iter()
                    .zip(subc.num_inputs.iter())
                    .enumerate()
                    .map(|(l, (off, len))| &cur[l][*off..*off + *len])
                    .collect::<Vec<_>>();
                self.apply_segment_unsafe(
                    subc,
                    &inputs,
                    &mut nxt[a.output_offset..a.output_offset + subc.num_outputs],
                );
            }
        }
    }

    pub fn eval_with_public_inputs(
        &self,
        inputs: Vec<C::CircuitField>,
        public_inputs: &[C::CircuitField],
    ) -> (Vec<C::CircuitField>, bool) {
        if inputs.len() != self.input_size() {
            panic!("input length mismatch");
        }
        let mut cur = vec![inputs];
        for &id in self.layer_ids.iter() {
            let mut next = vec![C::CircuitField::zero(); self.segments[id].num_outputs];
            let mut inputs: Vec<&[C::CircuitField]> = Vec::new();
            for i in 0..self.segments[id].num_inputs.len() {
                inputs.push(&cur[cur.len() - i - 1]);
            }
            self.apply_segment_with_public_inputs(
                &self.segments[id],
                &inputs,
                &mut next,
                public_inputs,
            );
            cur.push(next);
        }
        let cur = cur.last().unwrap();
        let mut constraints_satisfied = true;
        for out in cur.iter().take(self.expected_num_output_zeroes) {
            if !out.is_zero() {
                constraints_satisfied = false;
                break;
            }
        }
        (
            cur[self.expected_num_output_zeroes..self.num_actual_outputs].to_vec(),
            constraints_satisfied,
        )
    }

    fn apply_segment_with_public_inputs(
        &self,
        seg: &Segment<C>,
        cur: &[&[C::CircuitField]],
        nxt: &mut [C::CircuitField],
        public_inputs: &[C::CircuitField],
    ) {
        for m in seg.gate_muls.iter() {
            nxt[m.output] += cur[m.inputs[0].layer][m.inputs[0].offset]
                * cur[m.inputs[1].layer][m.inputs[1].offset]
                * m.coef.get_value_with_public_inputs(public_inputs);
        }
        for a in seg.gate_adds.iter() {
            nxt[a.output] += cur[a.inputs[0].layer][a.inputs[0].offset]
                * a.coef.get_value_with_public_inputs(public_inputs);
        }
        for cs in seg.gate_consts.iter() {
            nxt[cs.output] += cs.coef.get_value_with_public_inputs(public_inputs);
        }
        for cu in seg.gate_customs.iter() {
            let mut inputs = Vec::with_capacity(cu.inputs.len());
            for input in cu.inputs.iter() {
                inputs.push(cur[input.layer][input.offset]);
            }
            let outputs = hints::stub_impl(cu.gate_type, &inputs, 1);
            for (i, &output) in outputs.iter().enumerate() {
                nxt[cu.output + i] += output * cu.coef.get_value_with_public_inputs(public_inputs);
            }
        }
        for (sub_id, allocs) in seg.child_segs.iter() {
            let subc = &self.segments[*sub_id];
            for a in allocs.iter() {
                let inputs = a
                    .input_offset
                    .iter()
                    .zip(subc.num_inputs.iter())
                    .enumerate()
                    .map(|(l, (off, len))| &cur[l][*off..*off + *len])
                    .collect::<Vec<_>>();
                self.apply_segment_with_public_inputs(
                    subc,
                    &inputs,
                    &mut nxt[a.output_offset..a.output_offset + subc.num_outputs],
                    public_inputs,
                );
            }
        }
    }

    pub fn sort_everything(&mut self) {
        for seg in self.segments.iter_mut() {
            seg.gate_muls.sort();
            seg.gate_adds.sort();
            seg.gate_consts.sort();
            seg.gate_customs.sort();
            seg.child_segs.sort();
        }
    }
}

impl<C: Config> fmt::Display for Coef<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Coef::Constant(c) => write!(f, "{}", c.to_u256()),
            Coef::Random => write!(f, "Random"),
            Coef::PublicInput(id) => write!(f, "PublicInput({})", id),
        }
    }
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(layer={}, offset={})", self.layer, self.offset)
    }
}

impl<C: Config> fmt::Display for Segment<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "input={:?} output={}", self.num_inputs, self.num_outputs)?;
        for (sub_id, allocs) in self.child_segs.iter() {
            writeln!(f, "apply circuit {} at:", sub_id)?;
            for a in allocs.iter() {
                writeln!(
                    f,
                    "    input_offset={:?} output_offset={}",
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
        for cu in self.gate_customs.iter() {
            write!(f, "out{} += custom{}(", cu.output, cu.gate_type)?;
            for (i, input) in cu.inputs.iter().enumerate() {
                write!(f, "in{}", input)?;
                if i < cu.inputs.len() - 1 {
                    write!(f, ",")?;
                }
            }
            writeln!(f, ") * {}", cu.coef)?;
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

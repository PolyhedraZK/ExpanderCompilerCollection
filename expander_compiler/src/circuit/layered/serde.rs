use std::io::{Error as IoError, Read, Write};

use crate::{field::U256, utils::serde::Serde};

use super::*;

impl<C: Config> Serde for Coef<C> {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        Ok(match self {
            Coef::Constant(c) => {
                c.serialize_into(&mut writer)?;
            }
            Coef::Random => {
                C::CircuitField::zero().serialize_into(&mut writer)?;
            }
        })
    }
    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let c = C::CircuitField::deserialize_from(&mut reader)?;
        Ok(Coef::Constant(c))
    }
}

impl<C: Config, const INPUT_NUM: usize> Serde for Gate<C, INPUT_NUM> {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        for i in 0..INPUT_NUM {
            self.inputs[i].serialize_into(&mut writer)?;
        }
        self.output.serialize_into(&mut writer)?;
        self.coef.serialize_into(&mut writer)?;
        Ok(())
    }
    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let mut inputs = [0; INPUT_NUM];
        for i in 0..INPUT_NUM {
            inputs[i] = usize::deserialize_from(&mut reader)?;
        }
        let output = usize::deserialize_from(&mut reader)?;
        let coef = Coef::deserialize_from(&mut reader)?;
        Ok(Gate {
            inputs,
            output,
            coef,
        })
    }
}

impl Serde for Allocation {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        self.input_offset.serialize_into(&mut writer)?;
        self.output_offset.serialize_into(&mut writer)?;
        Ok(())
    }
    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let input_offset = usize::deserialize_from(&mut reader)?;
        let output_offset = usize::deserialize_from(&mut reader)?;
        Ok(Allocation {
            input_offset,
            output_offset,
        })
    }
}

impl Serde for ChildSpec {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        self.0.serialize_into(&mut writer)?;
        self.1.serialize_into(&mut writer)?;
        Ok(())
    }
    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let sub_circuit_id = usize::deserialize_from(&mut reader)?;
        let allocs = Vec::<Allocation>::deserialize_from(&mut reader)?;
        Ok((sub_circuit_id, allocs))
    }
}

impl<C: Config> Serde for Segment<C> {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        self.num_inputs.serialize_into(&mut writer)?;
        self.num_outputs.serialize_into(&mut writer)?;
        self.child_segs.serialize_into(&mut writer)?;
        self.gate_muls.serialize_into(&mut writer)?;
        self.gate_adds.serialize_into(&mut writer)?;
        self.gate_consts.serialize_into(&mut writer)?;
        0usize.serialize_into(&mut writer)?;
        let mut random_coef_idx = Vec::new();
        for (i, x) in self.gate_muls.iter().enumerate() {
            if x.coef == Coef::Random {
                random_coef_idx.push(i);
            }
        }
        for (i, x) in self.gate_adds.iter().enumerate() {
            if x.coef == Coef::Random {
                random_coef_idx.push(i + self.gate_muls.len());
            }
        }
        for (i, x) in self.gate_consts.iter().enumerate() {
            if x.coef == Coef::Random {
                random_coef_idx.push(i + self.gate_muls.len() + self.gate_adds.len());
            }
        }
        random_coef_idx.serialize_into(&mut writer)?;
        Ok(())
    }
    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let num_inputs = usize::deserialize_from(&mut reader)?;
        let num_outputs = usize::deserialize_from(&mut reader)?;
        let child_segs = Vec::<ChildSpec>::deserialize_from(&mut reader)?;
        let mut gate_muls = Vec::<GateMul<C>>::deserialize_from(&mut reader)?;
        let mut gate_adds = Vec::<GateAdd<C>>::deserialize_from(&mut reader)?;
        let mut gate_consts = Vec::<GateConst<C>>::deserialize_from(&mut reader)?;
        let _ = usize::deserialize_from(&mut reader)?;
        let random_coef_idx = Vec::<usize>::deserialize_from(&mut reader)?;
        for i in random_coef_idx {
            if i < gate_muls.len() {
                gate_muls[i].coef = Coef::Random;
            } else if i < gate_muls.len() + gate_adds.len() {
                gate_adds[i - gate_muls.len()].coef = Coef::Random;
            } else {
                gate_consts[i - gate_muls.len() - gate_adds.len()].coef = Coef::Random;
            }
        }
        Ok(Segment {
            num_inputs,
            num_outputs,
            child_segs,
            gate_muls,
            gate_adds,
            gate_consts,
        })
    }
}

const MAGIC: usize = 3770719418566461763;

impl<C: Config> Serde for Circuit<C> {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        MAGIC.serialize_into(&mut writer)?;
        C::CircuitField::modulus().serialize_into(&mut writer)?;
        self.segments.serialize_into(&mut writer)?;
        self.layer_ids.serialize_into(&mut writer)?;
        Ok(())
    }
    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let magic = usize::deserialize_from(&mut reader)?;
        if magic != MAGIC {
            return Err(IoError::new(
                std::io::ErrorKind::InvalidData,
                "invalid magic number",
            ));
        }
        let modulus = U256::deserialize_from(&mut reader)?;
        if modulus != C::CircuitField::modulus() {
            return Err(IoError::new(
                std::io::ErrorKind::InvalidData,
                "invalid modulus",
            ));
        }
        let segments = Vec::<Segment<C>>::deserialize_from(&mut reader)?;
        let layer_ids = Vec::<usize>::deserialize_from(&mut reader)?;
        Ok(Circuit {
            segments,
            layer_ids,
        })
    }
}

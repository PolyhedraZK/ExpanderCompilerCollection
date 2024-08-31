use std::io::{Error as IoError, Read, Write};

use crate::{field::FieldModulus, utils::serde::Serde};

use super::*;

impl<C: Config> Serde for Coef<C> {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        Ok(match self {
            Coef::Constant(c) => {
                1u8.serialize_into(&mut writer)?;
                c.serialize_into(&mut writer)?;
            }
            Coef::Random => {
                2u8.serialize_into(&mut writer)?;
            }
            Coef::PublicInput(id) => {
                3u8.serialize_into(&mut writer)?;
                id.serialize_into(&mut writer)?;
            }
        })
    }
    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let typ = u8::deserialize_from(&mut reader)?;
        match typ {
            1 => {
                let c = C::CircuitField::deserialize_from(&mut reader)?;
                Ok(Coef::Constant(c))
            }
            2 => Ok(Coef::Random),
            3 => {
                let id = usize::deserialize_from(&mut reader)?;
                Ok(Coef::PublicInput(id))
            }
            _ => Err(IoError::new(
                std::io::ErrorKind::InvalidData,
                "invalid coef type",
            )),
        }
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

impl<C: Config> Serde for GateCustom<C> {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        self.gate_type.serialize_into(&mut writer)?;
        self.inputs.serialize_into(&mut writer)?;
        self.output.serialize_into(&mut writer)?;
        self.coef.serialize_into(&mut writer)?;
        Ok(())
    }
    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let gate_type = usize::deserialize_from(&mut reader)?;
        let inputs = Vec::<usize>::deserialize_from(&mut reader)?;
        let output = usize::deserialize_from(&mut reader)?;
        let coef = Coef::<C>::deserialize_from(&mut reader)?;
        Ok(GateCustom {
            gate_type,
            inputs,
            output,
            coef,
        })
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
        self.gate_customs.serialize_into(&mut writer)?;
        Ok(())
    }
    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let num_inputs = usize::deserialize_from(&mut reader)?;
        let num_outputs = usize::deserialize_from(&mut reader)?;
        let child_segs = Vec::<ChildSpec>::deserialize_from(&mut reader)?;
        let gate_muls = Vec::<GateMul<C>>::deserialize_from(&mut reader)?;
        let gate_adds = Vec::<GateAdd<C>>::deserialize_from(&mut reader)?;
        let gate_consts = Vec::<GateConst<C>>::deserialize_from(&mut reader)?;
        let gate_customs = Vec::<GateCustom<C>>::deserialize_from(&mut reader)?;
        Ok(Segment {
            num_inputs,
            num_outputs,
            child_segs,
            gate_muls,
            gate_adds,
            gate_consts,
            gate_customs,
        })
    }
}

const MAGIC: usize = 3770719418566461763;

impl<C: Config> Serde for Circuit<C> {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        MAGIC.serialize_into(&mut writer)?;
        C::CircuitField::modulus().serialize_into(&mut writer)?;
        self.num_public_inputs.serialize_into(&mut writer)?;
        self.num_actual_outputs.serialize_into(&mut writer)?;
        self.expected_num_output_zeroes
            .serialize_into(&mut writer)?;
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
        let modulus = ethnum::U256::deserialize_from(&mut reader)?;
        if modulus != C::CircuitField::modulus() {
            return Err(IoError::new(
                std::io::ErrorKind::InvalidData,
                "invalid modulus",
            ));
        }
        let num_public_inputs = usize::deserialize_from(&mut reader)?;
        let num_actual_outputs = usize::deserialize_from(&mut reader)?;
        let expected_num_output_zeroes = usize::deserialize_from(&mut reader)?;
        let segments = Vec::<Segment<C>>::deserialize_from(&mut reader)?;
        let layer_ids = Vec::<usize>::deserialize_from(&mut reader)?;
        Ok(Circuit {
            num_public_inputs,
            num_actual_outputs,
            expected_num_output_zeroes,
            segments,
            layer_ids,
        })
    }
}

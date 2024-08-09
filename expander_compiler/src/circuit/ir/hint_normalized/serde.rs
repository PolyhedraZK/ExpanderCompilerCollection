use std::io::{Error as IoError, Read, Write};

use crate::{
    circuit::{config::Config, ir::expr::LinComb, layered::Coef},
    utils::serde::Serde,
};

use super::Instruction;

impl<C: Config> Serde for Instruction<C> {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        match self {
            Instruction::LinComb(lin_comb) => {
                1u8.serialize_into(&mut writer)?;
                lin_comb.serialize_into(&mut writer)?;
            }
            Instruction::Mul(inputs) => {
                2u8.serialize_into(&mut writer)?;
                inputs.serialize_into(&mut writer)?;
            }
            Instruction::Hint {
                hint_id,
                inputs,
                num_outputs,
            } => {
                3u8.serialize_into(&mut writer)?;
                hint_id.serialize_into(&mut writer)?;
                inputs.serialize_into(&mut writer)?;
                num_outputs.serialize_into(&mut writer)?;
            }
            Instruction::ConstantOrRandom(coef) => {
                4u8.serialize_into(&mut writer)?;
                coef.serialize_into(&mut writer)?;
                (*coef == Coef::Random).serialize_into(&mut writer)?;
            }
            Instruction::SubCircuitCall {
                sub_circuit_id,
                inputs,
                num_outputs,
            } => {
                5u8.serialize_into(&mut writer)?;
                sub_circuit_id.serialize_into(&mut writer)?;
                inputs.serialize_into(&mut writer)?;
                num_outputs.serialize_into(&mut writer)?;
            }
        };
        Ok(())
    }
    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let instruction_type = u8::deserialize_from(&mut reader)?;
        Ok(match instruction_type {
            1 => Instruction::LinComb(LinComb::deserialize_from(&mut reader)?),
            2 => Instruction::Mul(Vec::<usize>::deserialize_from(&mut reader)?),
            3 => Instruction::Hint {
                hint_id: usize::deserialize_from(&mut reader)?,
                inputs: Vec::<usize>::deserialize_from(&mut reader)?,
                num_outputs: usize::deserialize_from(&mut reader)?,
            },
            4 => {
                let coef = Coef::<C>::deserialize_from(&mut reader)?;
                let is_random = bool::deserialize_from(&mut reader)?;
                Instruction::ConstantOrRandom(if is_random { Coef::Random } else { coef })
            }
            5 => Instruction::SubCircuitCall {
                sub_circuit_id: usize::deserialize_from(&mut reader)?,
                inputs: Vec::<usize>::deserialize_from(&mut reader)?,
                num_outputs: usize::deserialize_from(&mut reader)?,
            },
            _ => {
                return Err(IoError::new(
                    std::io::ErrorKind::InvalidData,
                    "invalid InstructionType",
                ))
            }
        })
    }
}

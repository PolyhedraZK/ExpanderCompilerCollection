use std::{
    collections::HashMap,
    io::{Error as IoError, Read, Write},
};

use crate::circuit::config::Config;
use crate::utils::serde::Serde;

use super::{Circuit, IrConfig, RootCircuit};

impl<Irc: IrConfig> Serde for Circuit<Irc>
where
    Irc::Instruction: Serde,
    Irc::Constraint: Serde,
{
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        self.instructions.serialize_into(&mut writer)?;
        self.constraints.serialize_into(&mut writer)?;
        self.outputs.serialize_into(&mut writer)?;
        self.num_inputs.serialize_into(&mut writer)?;
        if Irc::HAS_HINT_INPUT {
            self.num_hint_inputs.serialize_into(&mut writer)?;
        }
        Ok(())
    }
    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let instructions = Vec::<Irc::Instruction>::deserialize_from(&mut reader)?;
        let constraints = Vec::<Irc::Constraint>::deserialize_from(&mut reader)?;
        let outputs = Vec::<usize>::deserialize_from(&mut reader)?;
        let num_inputs = usize::deserialize_from(&mut reader)?;
        let num_hint_inputs = if Irc::HAS_HINT_INPUT {
            usize::deserialize_from(&mut reader)?
        } else {
            0
        };
        Ok(Circuit {
            instructions,
            constraints,
            outputs,
            num_inputs,
            num_hint_inputs,
        })
    }
}

impl<Irc: IrConfig> Serde for RootCircuit<Irc>
where
    Irc::Instruction: Serde,
    Irc::Constraint: Serde,
{
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        Irc::Config::CONFIG_ID.serialize_into(&mut writer)?;
        self.circuits.serialize_into(&mut writer)?;
        Ok(())
    }
    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let config_id = usize::deserialize_from(&mut reader)?;
        if config_id != Irc::Config::CONFIG_ID {
            return Err(IoError::new(
                std::io::ErrorKind::InvalidData,
                "config id mismatch",
            ));
        }
        let circuits = HashMap::<usize, Circuit<Irc>>::deserialize_from(&mut reader)?;
        Ok(RootCircuit { circuits })
    }
}
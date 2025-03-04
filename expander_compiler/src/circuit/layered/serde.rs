use std::io::{Error as IoError, Read, Write};

use crate::{field::FieldModulus, utils::serde::Serde};

use super::{
    Allocation, ChildSpec, Circuit, Coef, Config, CrossLayerInput, CrossLayerInputUsize, Gate,
    GateAdd, GateConst, GateCustom, GateMul, InputType, NormalInput, NormalInputUsize, Segment,
};

impl<C: Config> Serde for Coef<C> {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        match self {
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
        };
        Ok(())
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

impl Serde for CrossLayerInput {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        self.layer.serialize_into(&mut writer)?;
        self.offset.serialize_into(&mut writer)?;
        Ok(())
    }
    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let layer = usize::deserialize_from(&mut reader)?;
        let offset = usize::deserialize_from(&mut reader)?;
        Ok(CrossLayerInput { layer, offset })
    }
}

impl Serde for NormalInput {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        self.offset.serialize_into(&mut writer)?;
        Ok(())
    }
    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let offset = usize::deserialize_from(&mut reader)?;
        Ok(NormalInput { offset })
    }
}

impl Serde for CrossLayerInputUsize {
    fn serialize_into<W: Write>(&self, writer: W) -> Result<(), IoError> {
        self.v.serialize_into(writer)
    }
    fn deserialize_from<R: Read>(reader: R) -> Result<Self, IoError> {
        Ok(CrossLayerInputUsize {
            v: Vec::<usize>::deserialize_from(reader)?,
        })
    }
}

impl Serde for NormalInputUsize {
    fn serialize_into<W: Write>(&self, writer: W) -> Result<(), IoError> {
        self.v.serialize_into(writer)
    }
    fn deserialize_from<R: Read>(reader: R) -> Result<Self, IoError> {
        Ok(NormalInputUsize {
            v: usize::deserialize_from(reader)?,
        })
    }
}

impl<C: Config, I: InputType, const INPUT_NUM: usize> Serde for Gate<C, I, INPUT_NUM> {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        for input in &self.inputs {
            input.serialize_into(&mut writer)?;
        }
        self.output.serialize_into(&mut writer)?;
        self.coef.serialize_into(&mut writer)?;
        Ok(())
    }
    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let mut inputs = [I::Input::default(); INPUT_NUM];
        for input in inputs.iter_mut() {
            *input = I::Input::deserialize_from(&mut reader)?;
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

impl<I: InputType> Serde for Allocation<I> {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        self.input_offset.serialize_into(&mut writer)?;
        self.output_offset.serialize_into(&mut writer)?;
        Ok(())
    }
    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let input_offset = I::InputUsize::deserialize_from(&mut reader)?;
        let output_offset = usize::deserialize_from(&mut reader)?;
        Ok(Allocation {
            input_offset,
            output_offset,
        })
    }
}

impl<I: InputType> Serde for ChildSpec<I> {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        self.0.serialize_into(&mut writer)?;
        self.1.serialize_into(&mut writer)?;
        Ok(())
    }
    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let sub_circuit_id = usize::deserialize_from(&mut reader)?;
        let allocs = Vec::<Allocation<I>>::deserialize_from(&mut reader)?;
        Ok((sub_circuit_id, allocs))
    }
}

impl<C: Config, I: InputType> Serde for GateCustom<C, I> {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        self.gate_type.serialize_into(&mut writer)?;
        self.inputs.serialize_into(&mut writer)?;
        self.output.serialize_into(&mut writer)?;
        self.coef.serialize_into(&mut writer)?;
        Ok(())
    }
    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let gate_type = usize::deserialize_from(&mut reader)?;
        let inputs = Vec::<I::Input>::deserialize_from(&mut reader)?;
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

impl<C: Config, I: InputType> Serde for Segment<C, I> {
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
        let num_inputs = I::InputUsize::deserialize_from(&mut reader)?;
        let num_outputs = usize::deserialize_from(&mut reader)?;
        let child_segs = Vec::<ChildSpec<I>>::deserialize_from(&mut reader)?;
        let gate_muls = Vec::<GateMul<C, I>>::deserialize_from(&mut reader)?;
        let gate_adds = Vec::<GateAdd<C, I>>::deserialize_from(&mut reader)?;
        let gate_consts = Vec::<GateConst<C, I>>::deserialize_from(&mut reader)?;
        let gate_customs = Vec::<GateCustom<C, I>>::deserialize_from(&mut reader)?;
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

const MAGIC: usize = 3914834606642317635;

impl<C: Config, I: InputType> Serde for Circuit<C, I> {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        MAGIC.serialize_into(&mut writer)?;
        C::CircuitField::MODULUS.serialize_into(&mut writer)?;
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
        if modulus != C::CircuitField::MODULUS {
            return Err(IoError::new(
                std::io::ErrorKind::InvalidData,
                "invalid modulus",
            ));
        }
        let num_public_inputs = usize::deserialize_from(&mut reader)?;
        let num_actual_outputs = usize::deserialize_from(&mut reader)?;
        let expected_num_output_zeroes = usize::deserialize_from(&mut reader)?;
        let segments = Vec::<Segment<C, I>>::deserialize_from(&mut reader)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::{
        config::*,
        ir::{common::rand_gen::*, dest::RootCircuit},
        layered::{CrossLayerInputType, NormalInputType},
    };

    fn test_serde_for_field<C: Config, I: InputType>() {
        let mut config = RandomCircuitConfig {
            seed: 0,
            num_circuits: RandomRange { min: 1, max: 20 },
            num_inputs: RandomRange { min: 1, max: 3 },
            num_instructions: RandomRange { min: 30, max: 50 },
            num_constraints: RandomRange { min: 0, max: 5 },
            num_outputs: RandomRange { min: 1, max: 3 },
            num_terms: RandomRange { min: 1, max: 5 },
            sub_circuit_prob: 0.05,
        };
        for i in 0..500 {
            config.seed = i + 10000;
            let root = RootCircuit::<C>::random(&config);
            assert_eq!(root.validate(), Ok(()));
            let (circuit, _) = crate::layering::compile(&root);
            assert_eq!(circuit.validate(), Ok(()));
            let mut buf = Vec::new();
            circuit.serialize_into(&mut buf).unwrap();
            let circuit2 = Circuit::<C, I>::deserialize_from(&buf[..]).unwrap();
            assert_eq!(circuit, circuit2);
        }
    }

    #[test]
    fn test_serde() {
        test_serde_for_field::<M31Config, NormalInputType>();
        test_serde_for_field::<GF2Config, NormalInputType>();
        test_serde_for_field::<BN254Config, NormalInputType>();
        test_serde_for_field::<M31Config, CrossLayerInputType>();
        test_serde_for_field::<GF2Config, CrossLayerInputType>();
        test_serde_for_field::<BN254Config, CrossLayerInputType>();
    }
}

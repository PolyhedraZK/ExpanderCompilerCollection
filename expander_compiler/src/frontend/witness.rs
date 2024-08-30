use crate::{
    field::{FieldModulus, U256},
    utils::serde::Serde,
};

use super::*;

pub struct WitnessSolver<C: Config> {
    pub circuit: ir::hint_normalized::RootCircuit<C>,
}

#[derive(Debug)]
pub struct Witness<C: Config> {
    pub num_witnesses: usize,
    pub num_inputs_per_witness: usize,
    pub num_public_inputs_per_witness: usize,
    pub values: Vec<C::CircuitField>,
}

impl<C: Config> WitnessSolver<C> {
    fn solve_witness_inner<Cir: internal::DumpLoadTwoVariables<C::CircuitField>>(
        &self,
        assignment: &Cir,
    ) -> Result<(Vec<C::CircuitField>, usize), Error> {
        assert_eq!(
            assignment.num_vars(),
            (self.circuit.input_size(), self.circuit.num_public_inputs)
        );
        let mut vars = Vec::new();
        let mut public_vars = Vec::new();
        assignment.dump_into(&mut vars, &mut public_vars);
        let mut a = self.circuit.eval_with_public_inputs(vars, &public_vars)?;
        let res_len = a.len();
        a.extend(public_vars);
        Ok((a, res_len))
    }

    pub fn solve_witness<Cir: internal::DumpLoadTwoVariables<C::CircuitField>>(
        &self,
        assignment: &Cir,
    ) -> Result<Witness<C>, Error> {
        let (values, num_inputs_per_witness) = self.solve_witness_inner(assignment)?;
        Ok(Witness {
            num_witnesses: 1,
            num_inputs_per_witness,
            num_public_inputs_per_witness: self.circuit.num_public_inputs,
            values,
        })
    }

    pub fn solve_witnesses<Cir: internal::DumpLoadTwoVariables<C::CircuitField>>(
        &self,
        assignments: &[Cir],
    ) -> Result<Witness<C>, Error> {
        let mut values = Vec::new();
        let mut num_inputs_per_witness = 0;
        for assignment in assignments {
            let (a, num) = self.solve_witness_inner(assignment)?;
            values.extend(a);
            num_inputs_per_witness = num;
        }
        Ok(Witness {
            num_witnesses: assignments.len(),
            num_inputs_per_witness,
            num_public_inputs_per_witness: self.circuit.num_public_inputs,
            values,
        })
    }
}

impl<C: Config> layered::Circuit<C> {
    pub fn run(&self, witness: &Witness<C>) -> Vec<bool> {
        if witness.num_witnesses == 0 {
            panic!("expected at least 1 witness")
        }
        let mut res = Vec::new();
        let a = witness.num_inputs_per_witness;
        let b = witness.num_public_inputs_per_witness;
        for i in 0..witness.num_witnesses {
            let (_, out) = self.eval_with_public_inputs(
                witness.values[i * (a + b)..i * (a + b) + a].to_vec(),
                &witness.values[i * (a + b) + a..i * (a + b) + a + b],
            );
            res.push(out);
        }
        res
    }
}

impl<C: Config> Serde for WitnessSolver<C> {
    fn deserialize_from<R: std::io::Read>(mut reader: R) -> Result<Self, std::io::Error> {
        let circuit = ir::hint_normalized::RootCircuit::<C>::deserialize_from(&mut reader)?;
        Ok(Self { circuit })
    }
    fn serialize_into<W: std::io::Write>(&self, mut writer: W) -> Result<(), std::io::Error> {
        self.circuit.serialize_into(&mut writer)
    }
}

impl<C: Config> Serde for Witness<C> {
    fn deserialize_from<R: std::io::Read>(mut reader: R) -> Result<Self, std::io::Error> {
        let num_witnesses = usize::deserialize_from(&mut reader)?;
        let num_inputs_per_witness = usize::deserialize_from(&mut reader)?;
        let num_public_inputs_per_witness = usize::deserialize_from(&mut reader)?;
        let modulus = U256::deserialize_from(&mut reader)?;
        if modulus != C::CircuitField::modulus() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid modulus",
            ));
        }
        let mut values = Vec::with_capacity(
            num_witnesses * (num_inputs_per_witness + num_public_inputs_per_witness),
        );
        for _ in 0..num_witnesses * (num_inputs_per_witness + num_public_inputs_per_witness) {
            values.push(C::CircuitField::deserialize_from(&mut reader)?);
        }
        Ok(Self {
            num_witnesses,
            num_inputs_per_witness,
            num_public_inputs_per_witness,
            values,
        })
    }
    fn serialize_into<W: std::io::Write>(&self, mut writer: W) -> Result<(), std::io::Error> {
        self.num_witnesses.serialize_into(&mut writer)?;
        self.num_inputs_per_witness.serialize_into(&mut writer)?;
        self.num_public_inputs_per_witness
            .serialize_into(&mut writer)?;
        C::CircuitField::modulus().serialize_into(&mut writer)?;
        for v in &self.values {
            v.serialize_into(&mut writer)?;
        }
        Ok(())
    }
}

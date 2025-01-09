use std::mem;

use arith::SimdField;

use super::*;
use crate::{circuit::config::Config, field::FieldModulus, utils::serde::Serde};

#[derive(Debug)]
pub enum WitnessValues<C: Config> {
    Scalar(Vec<C::CircuitField>),
    SIMD(Vec<C::DefaultSimdField>),
}

#[derive(Debug)]
pub struct Witness<C: Config> {
    pub num_witnesses: usize,
    pub num_inputs_per_witness: usize,
    pub num_public_inputs_per_witness: usize,
    pub values: WitnessValues<C>,
}

pub struct WitnessIteratorScalar<'a, C: Config> {
    witness: &'a Witness<C>,
    index: usize,
    buf_unpacked: Vec<Vec<C::CircuitField>>,
}

impl<'a, C: Config> Iterator for WitnessIteratorScalar<'a, C> {
    type Item = (Vec<C::CircuitField>, Vec<C::CircuitField>);
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.witness.num_witnesses {
            return None;
        }
        let a = self.witness.num_inputs_per_witness;
        let b = self.witness.num_public_inputs_per_witness;
        match &self.witness.values {
            WitnessValues::Scalar(values) => {
                let res = (
                    values[self.index * (a + b)..self.index * (a + b) + a].to_vec(),
                    values[self.index * (a + b) + a..self.index * (a + b) + a + b].to_vec(),
                );
                self.index += 1;
                Some(res)
            }
            WitnessValues::SIMD(values) => {
                let pack_size = C::DefaultSimdField::PACK_SIZE;
                if self.index % pack_size == 0 {
                    self.buf_unpacked.clear();
                    for _ in 0..pack_size {
                        self.buf_unpacked.push(Vec::with_capacity(a));
                    }
                    for _ in 0..pack_size {
                        self.buf_unpacked.push(Vec::with_capacity(b));
                    }
                    let simd_index = self.index / pack_size;
                    for i in 0..a {
                        let tmp = values[simd_index * (a + b) + i].unpack();
                        for j in 0..pack_size {
                            self.buf_unpacked[j].push(tmp[j]);
                        }
                    }
                    for i in a..a + b {
                        let tmp = values[simd_index * (a + b) + i].unpack();
                        for j in 0..pack_size {
                            self.buf_unpacked[j + pack_size].push(tmp[j]);
                        }
                    }
                }
                let res = (
                    mem::take(&mut self.buf_unpacked[self.index % pack_size]),
                    mem::take(&mut self.buf_unpacked[self.index % pack_size + pack_size]),
                );
                self.index += 1;
                Some(res)
            }
        }
    }
}

impl<C: Config> Witness<C> {
    pub fn iter_scalar<'a>(&'a self) -> WitnessIteratorScalar<'a, C> {
        WitnessIteratorScalar {
            witness: self,
            index: 0,
            buf_unpacked: Vec::new(),
        }
    }
}

impl<C: Config, I: InputType> Circuit<C, I> {
    pub fn run(&self, witness: &Witness<C>) -> Vec<bool> {
        if witness.num_witnesses == 0 {
            panic!("expected at least 1 witness")
        }
        let mut res = Vec::new();
        for (inputs, public_inputs) in witness.iter_scalar() {
            let (_, out) = self.eval_with_public_inputs(inputs, &public_inputs);
            res.push(out);
        }
        res
    }
}

impl<C: Config> Witness<C> {
    pub fn to_simd<T>(&self) -> (Vec<T>, Vec<T>)
    where
        T: arith::SimdField<Scalar = C::CircuitField>,
    {
        match self.num_witnesses.cmp(&T::PACK_SIZE) {
            std::cmp::Ordering::Less => {
                println!(
                    "Warning: not enough witnesses, expect {}, got {}",
                    T::PACK_SIZE,
                    self.num_witnesses
                )
            }
            std::cmp::Ordering::Greater => {
                println!(
                    "Warning: dropping additional witnesses, expect {}, got {}",
                    T::PACK_SIZE,
                    self.num_witnesses
                )
            }
            std::cmp::Ordering::Equal => {}
        }
        let ni = self.num_inputs_per_witness;
        let np = self.num_public_inputs_per_witness;
        let mut res = Vec::with_capacity(ni);
        let mut res_public = Vec::with_capacity(np);
        for i in 0..ni + np {
            let mut values: Vec<C::CircuitField> = (0..self.num_witnesses.min(T::PACK_SIZE))
                .map(|j| self.values[j * (ni + np) + i])
                .collect();
            values.resize(T::PACK_SIZE, C::CircuitField::zero());
            let simd_value = T::pack(&values);
            if i < ni {
                res.push(simd_value);
            } else {
                res_public.push(simd_value);
            }
        }
        (res, res_public)
    }
}

impl<C: Config> Serde for Witness<C> {
    fn deserialize_from<R: std::io::Read>(mut reader: R) -> Result<Self, std::io::Error> {
        let num_witnesses = usize::deserialize_from(&mut reader)?;
        let num_inputs_per_witness = usize::deserialize_from(&mut reader)?;
        let num_public_inputs_per_witness = usize::deserialize_from(&mut reader)?;
        let modulus = ethnum::U256::deserialize_from(&mut reader)?;
        if modulus != C::CircuitField::MODULUS {
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
            values: WitnessValues::Scalar(values), // TODO: SIMD
        })
    }
    fn serialize_into<W: std::io::Write>(&self, mut writer: W) -> Result<(), std::io::Error> {
        self.num_witnesses.serialize_into(&mut writer)?;
        self.num_inputs_per_witness.serialize_into(&mut writer)?;
        self.num_public_inputs_per_witness
            .serialize_into(&mut writer)?;
        C::CircuitField::MODULUS.serialize_into(&mut writer)?;
        for v in &self.values {
            v.serialize_into(&mut writer)?;
        }
        Ok(())
    }
}

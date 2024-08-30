pub use arith::{Field as FieldArith, FieldForECC as FieldModulus, BN254, GF2, M31, U256};

use crate::utils::serde::Serde;
use arith::{FieldForECC, FieldSerde, FieldSerdeError};

pub trait Field: FieldArith + FieldForECC + FieldSerde {}

impl Field for BN254 {}
impl Field for GF2 {}
impl Field for M31 {}

impl<T: Field> Serde for T {
    fn serialize_into<W: std::io::Write>(&self, writer: W) -> Result<(), std::io::Error> {
        match <Self as FieldSerde>::serialize_into(&self, writer) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                FieldSerdeError::IOError(e) => Err(e),
                FieldSerdeError::DeserializeError => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "failed to serialize field element",
                )),
            },
        }
    }
    fn deserialize_from<R: std::io::Read>(reader: R) -> Result<Self, std::io::Error> {
        match <Self as FieldSerde>::deserialize_from(reader) {
            Ok(f) => Ok(f),
            Err(e) => match e {
                FieldSerdeError::IOError(e) => Err(e),
                FieldSerdeError::DeserializeError => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "failed to deserialize field element",
                )),
            },
        }
    }
}
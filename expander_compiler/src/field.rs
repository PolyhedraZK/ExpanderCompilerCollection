pub use arith::{Field as FieldArith, Fr};
pub use gf2::GF2;
pub use mersenne31::M31;
use serdes::ExpSerde;

// use crate::utils::serde::Serde;
// use arith::Field;
// use serdes::ArithSerde;

pub trait Field: FieldArith + ExpSerde {}

impl Field for Fr {}
impl Field for GF2 {}
impl Field for M31 {}

// // #[macro_export]
// macro_rules! impl_arith_serde {
//     ($field:ident) => {
//         impl<T: Field> ExpSerde for $field {
//             fn serialize_into<W: std::io::Write>(&self, writer: W) -> SerdeResult<()> {
//                 match <Self as FieldSerde>::serialize_into(self, writer) {
//                     Ok(_) => Ok(()),
//                     Err(e) => match e {
//                         FieldSerdeError::IOError(e) => Err(e),
//                         FieldSerdeError::DeserializeError => Err(std::io::Error::new(
//                             std::io::ErrorKind::InvalidData,
//                             "failed to serialize field element",
//                         )),
//                     },
//                 }
//             }
//             fn deserialize_from<R: std::io::Read>(reader: R) -> SerdeResult<Self> {
//                 match <Self as FieldSerde>::deserialize_from(reader) {
//                     Ok(f) => Ok(f),
//                     Err(e) => match e {
//                         FieldSerdeError::IOError(e) => Err(e),
//                         FieldSerdeError::DeserializeError => Err(std::io::Error::new(
//                             std::io::ErrorKind::InvalidData,
//                             "failed to deserialize field element",
//                         )),
//                     },
//                 }
//             }
//         }
//     };
// }

// impl_arith_serde!(Fr);
// impl_arith_serde!(GF2);
// impl_arith_serde!(M31);

// impl<T: Field> ExpSerde for T {
//     fn serialize_into<W: std::io::Write>(&self, writer: W) -> SerdeResult<()> {
//         match <Self as FieldSerde>::serialize_into(self, writer) {
//             Ok(_) => Ok(()),
//             Err(e) => match e {
//                 FieldSerdeError::IOError(e) => Err(e),
//                 FieldSerdeError::DeserializeError => Err(std::io::Error::new(
//                     std::io::ErrorKind::InvalidData,
//                     "failed to serialize field element",
//                 )),
//             },
//         }
//     }
//     fn deserialize_from<R: std::io::Read>(reader: R) -> SerdeResult<Self> {
//         match <Self as FieldSerde>::deserialize_from(reader) {
//             Ok(f) => Ok(f),
//             Err(e) => match e {
//                 FieldSerdeError::IOError(e) => Err(e),
//                 FieldSerdeError::DeserializeError => Err(std::io::Error::new(
//                     std::io::ErrorKind::InvalidData,
//                     "failed to deserialize field element",
//                 )),
//             },
//         }
//     }
// }

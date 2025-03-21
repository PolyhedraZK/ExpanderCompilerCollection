pub use arith::{Field as FieldArith, Fr as BN254};
pub use gf2::GF2;
pub use mersenne31::M31;
use serdes::ExpSerde;

pub trait Field: FieldArith + ExpSerde {}

impl Field for BN254 {}
impl Field for GF2 {}
impl Field for M31 {}

pub use arith::{Field as FieldArith, Fr as BN254Fr};
use babybear::BabyBear;
pub use gf2::GF2;
pub use goldilocks::Goldilocks;
pub use mersenne31::M31;
use serdes::ExpSerde;

pub trait Field: FieldArith + ExpSerde {}

impl Field for BN254Fr {}
impl Field for GF2 {}
impl Field for M31 {}
impl Field for Goldilocks {}
impl Field for BabyBear {}

pub use arith::{Field as FieldArith, Fr as BN254Fr};
pub use gf2::GF2;
pub use mersenne31::M31;
use serdes::ExpSerde;

pub trait Field: FieldArith + ExpSerde {}

impl Field for BN254Fr {}
impl Field for GF2 {}
impl Field for M31 {}

// This trait exist only for making Rust happy
// If we use arith::Field, Rust says upstream may add more impls
pub trait FieldRaw: FieldArith {}

impl FieldRaw for BN254Fr {}
impl FieldRaw for GF2 {}
impl FieldRaw for M31 {}
impl FieldRaw for mersenne31::M31x16 {}
impl FieldRaw for gf2::GF2x8 {}

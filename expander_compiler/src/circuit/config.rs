use std::{fmt::Debug, hash::Hash};

use crate::field::Field;

pub trait Config: Default + Clone + Ord + Debug + Hash + Copy + 'static {
    type CircuitField: Field;

    const CONFIG_ID: usize;
}

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct M31Config {}

impl Config for M31Config {
    type CircuitField = crate::field::m31::M31;

    const CONFIG_ID: usize = 1;
}

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct BN254Config {}

impl Config for BN254Config {
    type CircuitField = crate::field::bn254::BN254;

    const CONFIG_ID: usize = 2;
}

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct GF2Config {}

impl Config for GF2Config {
    type CircuitField = crate::field::gf2::GF2;

    const CONFIG_ID: usize = 3;
}

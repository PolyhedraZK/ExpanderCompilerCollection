use std::{fmt::Debug, hash::Hash};

use crate::field::Field;

pub trait Config: Default + Clone + Ord + Debug + Hash {
    type CircuitField: Field;

    const CONFIG_ID: usize;
}

#[derive(Default, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct M31Config {}

impl Config for M31Config {
    type CircuitField = crate::field::m31::M31;

    const CONFIG_ID: usize = 1;
}

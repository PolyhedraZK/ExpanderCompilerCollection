use std::{fmt::Debug, hash::Hash};

pub use gkr::{
    BN254ConfigMIMC5Raw as BN254Config, GF2ExtConfigSha2Raw as GF2Config,
    GoldilocksExtConfigSha2Raw as GoldilocksConfig, M31ExtConfigSha2RawVanilla as M31Config,
};
use gkr_engine::{FieldEngine, GKREngine};

use crate::field::Field;

pub trait Config:
    Default
    + Clone
    + Debug
    + PartialEq
    + Eq
    + Ord
    + Hash
    + Copy
    + 'static
    + GKREngine<FieldConfig: FieldEngine<CircuitField: Field>>
{
    const CONFIG_ID: usize;

    const COST_INPUT: usize = 1000;
    const COST_VARIABLE: usize = 100;
    const COST_MUL: usize = 10;
    const COST_ADD: usize = 3;
    const COST_CONST: usize = 3;

    const ENABLE_RANDOM_COMBINATION: bool = true;
}

pub type CircuitField<C: Config> = <<C as GKREngine>::FieldConfig as FieldEngine>::CircuitField;
pub type ChallengeField<C: Config> = <<C as GKREngine>::FieldConfig as FieldEngine>::ChallengeField;
pub type SIMDField<C: Config> = <<C as GKREngine>::FieldConfig as FieldEngine>::SimdCircuitField;

impl Config for M31Config {
    const CONFIG_ID: usize = 1;
}

impl Config for BN254Config {
    const CONFIG_ID: usize = 2;
}

impl Config for GF2Config {
    const CONFIG_ID: usize = 3;

    // temporary fix for Keccak_GF2
    // TODO: measure actual costs
    const COST_MUL: usize = 200;

    const ENABLE_RANDOM_COMBINATION: bool = false;
}

impl Config for GoldilocksConfig {
    const CONFIG_ID: usize = 4;
}

use std::{fmt::Debug, hash::Hash};

pub use gkr::{
    BN254ConfigMIMC5Raw, GF2ExtConfigSha2Raw, GoldilocksExtConfigSha2Raw,
    M31ExtConfigSha2RawVanilla,
};
use gkr_engine::{FieldEngine, GKREngine};

use crate::field::{Field, FieldRaw};

pub trait Config: Default
    + Clone
    + Debug
    + PartialEq
    + Eq
    + Ord
    + Hash
    + Copy
    + 'static
    + GKREngine<FieldConfig: FieldEngine<CircuitField: Field + FieldRaw, SimdCircuitField: FieldRaw>>
{
    const CONFIG_ID: usize;

    const COST_INPUT: usize = 1000;
    const COST_VARIABLE: usize = 100;
    const COST_MUL: usize = 10;
    const COST_ADD: usize = 3;
    const COST_CONST: usize = 3;

    const ENABLE_RANDOM_COMBINATION: bool = true;
}

pub type CircuitField<C> = <<C as GKREngine>::FieldConfig as FieldEngine>::CircuitField;
pub type ChallengeField<C> = <<C as GKREngine>::FieldConfig as FieldEngine>::ChallengeField;
pub type SIMDField<C> = <<C as GKREngine>::FieldConfig as FieldEngine>::SimdCircuitField;

pub type BN254Config = BN254ConfigMIMC5Raw;
pub type M31Config = M31ExtConfigSha2RawVanilla;
pub type GF2Config = GF2ExtConfigSha2Raw;
pub type GoldilocksConfig = GoldilocksExtConfigSha2Raw;

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

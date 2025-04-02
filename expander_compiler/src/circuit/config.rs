use std::{fmt::Debug, hash::Hash};

use gkr_engine::{FieldEngine, GKREngine};

use crate::field::Field;

pub trait Config: Default + Clone + Ord + Debug + Hash + Copy + 'static {
    type CircuitField: Field;

    type DefaultSimdField: arith::SimdField<Scalar = Self::CircuitField>;
    type DefaultGKRFieldConfig: FieldEngine<
        CircuitField = Self::CircuitField,
        SimdCircuitField = Self::DefaultSimdField,
    >;
    type DefaultGKRConfig: GKREngine<FieldConfig = Self::DefaultGKRFieldConfig> + Default;

    const CONFIG_ID: usize;

    const COST_INPUT: usize = 1000;
    const COST_VARIABLE: usize = 100;
    const COST_MUL: usize = 10;
    const COST_ADD: usize = 3;
    const COST_CONST: usize = 3;

    const ENABLE_RANDOM_COMBINATION: bool = true;

    fn new_expander_config() -> Self::DefaultGKRConfig {
        Self::DefaultGKRConfig::default()
    }
}

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct M31Config {}

impl Config for M31Config {
    type CircuitField = crate::field::M31;

    type DefaultSimdField = mersenne31::M31x16;
    type DefaultGKRFieldConfig = gkr_engine::M31ExtConfig;
    type DefaultGKRConfig = gkr::M31ExtConfigSha2RawVanilla; //TODO: compare with M31ExtConfigSha2Orion

    const CONFIG_ID: usize = 1;
}

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct BN254Config {}

impl Config for BN254Config {
    type CircuitField = crate::field::BN254Fr;

    type DefaultSimdField = crate::field::BN254Fr;
    type DefaultGKRFieldConfig = gkr_engine::BN254Config;
    type DefaultGKRConfig = gkr::BN254ConfigMIMC5Raw; // TODO: compare with BN254ConfigSha2Raw

    const CONFIG_ID: usize = 2;
}

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct GF2Config {}

impl Config for GF2Config {
    type CircuitField = crate::field::GF2;

    type DefaultSimdField = gf2::GF2x8;
    type DefaultGKRFieldConfig = gkr_engine::GF2ExtConfig;
    type DefaultGKRConfig = gkr::GF2ExtConfigSha2Raw; // TODO: compare with GF2ExtConfigSha2Orion

    const CONFIG_ID: usize = 3;

    // temporary fix for Keccak_GF2
    // TODO: measure actual costs
    const COST_MUL: usize = 200;

    const ENABLE_RANDOM_COMBINATION: bool = false;
}

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct GoldilocksConfig {}

impl Config for GoldilocksConfig {
    type CircuitField = crate::field::Goldilocks;

    type DefaultSimdField = goldilocks::Goldilocksx8;
    type DefaultGKRFieldConfig = gkr_engine::GoldilocksExtConfig;
    type DefaultGKRConfig = gkr::GoldilocksExtConfigSha2Raw;
    const CONFIG_ID: usize = 4;
}

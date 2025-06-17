//! Circuit configuration traits and types. Based on the `GKREngine`.

use std::{fmt::Debug, hash::Hash};

pub use gkr::{
    BN254ConfigMIMC5Raw, BabyBearx16ConfigSha2Raw, GF2ExtConfigSha2Raw, Goldilocksx8ConfigSha2Raw,
    M31x16ConfigSha2RawVanilla,
};
use gkr_engine::{FieldEngine, GKREngine};

use crate::field::{Field, FieldRaw};

/// The trait for circuit configuration.
/// It extends the `GKREngine` trait and provides additional configuration constants.
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
    /// The unique identifier for the configuration.
    /// It's used in serialization and FFI.
    const CONFIG_ID: usize;

    /// The cost of a single input variable.
    const COST_INPUT: usize = 1000;
    /// The cost of a single variable in the circuit.
    const COST_VARIABLE: usize = 100;
    /// The cost of a single multiplication gate.
    const COST_MUL: usize = 10;
    /// The cost of a single addition gate.
    const COST_ADD: usize = 3;
    /// The cost of a single constant in the circuit.
    const COST_CONST: usize = 3;

    /// Whether to enable random combination of inputs.
    /// In certain fields like GF(2), random combination is not supported.
    const ENABLE_RANDOM_COMBINATION: bool = true;
}

/// Type aliases for the circuit field of the given configuration.
pub type CircuitField<C> = <<C as GKREngine>::FieldConfig as FieldEngine>::CircuitField;
/// Type aliases for the challenge field of the given configuration.
pub type ChallengeField<C> = <<C as GKREngine>::FieldConfig as FieldEngine>::ChallengeField;
/// Type aliases for the SIMD field of the given configuration.
pub type SIMDField<C> = <<C as GKREngine>::FieldConfig as FieldEngine>::SimdCircuitField;

// The Lifetime parameter is used to ensure the mpi config is valid during the proving process.
// TODO: We should probably not include it in ECC.
/// The configuration for the BN254 curve with MIMC5 hash function.
pub type BN254Config = BN254ConfigMIMC5Raw<'static>;
/// The configuration for the M31 curve with SHA-2 hash function.
pub type M31Config = M31x16ConfigSha2RawVanilla<'static>;
/// The configuration for the GF(2) field with SHA-2 hash function.
pub type GF2Config = GF2ExtConfigSha2Raw<'static>;
/// The configuration for the Goldilocks field with SHA-2 hash function.
pub type GoldilocksConfig = Goldilocksx8ConfigSha2Raw<'static>;
/// The configuration for the BabyBear field with SHA-2 hash function.
pub type BabyBearConfig = BabyBearx16ConfigSha2Raw<'static>;

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

impl Config for BabyBearConfig {
    const CONFIG_ID: usize = 5;
}

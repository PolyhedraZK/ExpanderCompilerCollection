pub mod prove_impl;
pub mod server_fns;
pub mod setup_impl;
pub mod verify_impl;

pub mod api_pcs_defered;
pub use api_pcs_defered::*;
use arith::Fr;

use crate::frontend::BN254Config;
use expander_transcript::BytesHashTranscript;
use gkr_engine::{GKREngine, GKRScheme, MPIConfig};
use gkr_hashers::{MiMC5FiatShamirHasher, SHA256hasher};
use halo2curves::bn256::Bn256;
use poly_commit::HyperUniKZGPCS;

pub struct BN254ConfigSha2UniKZG<'a> {
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> GKREngine for BN254ConfigSha2UniKZG<'a> {
    type FieldConfig = <BN254Config as GKREngine>::FieldConfig;
    type MPIConfig = MPIConfig<'a>;
    type TranscriptConfig = BytesHashTranscript<SHA256hasher>;
    type PCSConfig = HyperUniKZGPCS<Bn256>;
    const SCHEME: GKRScheme = GKRScheme::Vanilla;
}

pub struct BN254ConfigMIMCUniKZG<'a> {
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> GKREngine for BN254ConfigMIMCUniKZG<'a> {
    type FieldConfig = <BN254Config as GKREngine>::FieldConfig;
    type MPIConfig = MPIConfig<'a>;
    type TranscriptConfig = BytesHashTranscript<MiMC5FiatShamirHasher<Fr>>;
    type PCSConfig = HyperUniKZGPCS<Bn256>;
    const SCHEME: GKRScheme = GKRScheme::Vanilla;
}

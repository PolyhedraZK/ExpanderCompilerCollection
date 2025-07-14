use gkr::{BN254ConfigSha2Hyrax, BN254ConfigSha2Raw, M31x16ConfigSha2RawVanilla};
use gkr_engine::GKREngine;

use crate::{frontend::{BN254Config, Config, M31Config}, zkcuda::proving_system::expander_pcs_defered::BN254ConfigSha2UniKZG};

pub trait ZKCudaConfig {
    type ECCConfig: Config;
    type GKRConfig: GKREngine<FieldConfig = <Self::ECCConfig as GKREngine>::FieldConfig>;

    const BATCH_PCS: bool = false;
}

pub type GetPCS<ZKCConfig> = <<ZKCConfig as ZKCudaConfig>::GKRConfig as GKREngine>::PCSConfig;
pub type GetTranscript<ZKCConfig> = <<ZKCConfig as ZKCudaConfig>::GKRConfig as GKREngine>::TranscriptConfig;
pub type GetFieldConfig<ZKCConfig> = <<ZKCConfig as ZKCudaConfig>::GKRConfig as GKREngine>::FieldConfig;

pub struct ZKCudaConfigImpl<ECC, GKR, const BATCH_PCS: bool>
where
    ECC: Config,
    GKR: GKREngine<FieldConfig = <ECC as GKREngine>::FieldConfig>,
{
    _phantom: std::marker::PhantomData<(ECC, GKR, bool)>,
}

impl<ECC, GKR, const BATCH_PCS: bool> ZKCudaConfig for ZKCudaConfigImpl<ECC, GKR, BATCH_PCS>
where
    ECC: Config,
    GKR: GKREngine<FieldConfig = <ECC as GKREngine>::FieldConfig>,
{
    type ECCConfig = ECC;
    type GKRConfig = GKR;

    const BATCH_PCS: bool = BATCH_PCS;
}

// Concrete ZKCudaConfig types for various configurations
pub type ZKCudaBN254Hyrax<'a> = ZKCudaConfigImpl<BN254Config, BN254ConfigSha2Hyrax<'a>, false>;
pub type ZKCudaBN254KZG<'a> = ZKCudaConfigImpl<BN254Config, BN254ConfigSha2UniKZG<'a>, false>;

pub type ZKCudaM31<'a> = ZKCudaConfigImpl<M31Config, M31x16ConfigSha2RawVanilla<'a>, false>;
pub type ZKCudaGF2<'a> = ZKCudaConfigImpl<BN254Config, BN254ConfigSha2Raw<'a>, false>;
pub type ZKCudaGoldilocks<'a> = ZKCudaConfigImpl<BN254Config, BN254ConfigSha2Raw<'a>, false>;
pub type ZKCudaBabyBear<'a> = ZKCudaConfigImpl<BN254Config, BN254ConfigSha2Raw<'a>, false>;

// Batch PCS types
pub type ZKCudaBN254HyraxBatchPCS<'a> = ZKCudaConfigImpl<BN254Config, BN254ConfigSha2Hyrax<'a>, true>;
pub type ZKCudaBN254KZGBatchPCS<'a> = ZKCudaConfigImpl<BN254Config, BN254ConfigSha2UniKZG<'a>, true>;

pub type ZKCudaM31BatchPCS<'a> = ZKCudaConfigImpl<M31Config, M31x16ConfigSha2RawVanilla<'a>, true>;
pub type ZKCudaGF2BatchPCS<'a> = ZKCudaConfigImpl<BN254Config, BN254ConfigSha2Raw<'a>, true>;
pub type ZKCudaGoldilocksBatchPCS<'a> = ZKCudaConfigImpl<BN254Config, BN254ConfigSha2Raw<'a>, true>;
pub type ZKCudaBabyBearBatchPCS<'a> = ZKCudaConfigImpl<BN254Config, BN254ConfigSha2Raw<'a>, true>;

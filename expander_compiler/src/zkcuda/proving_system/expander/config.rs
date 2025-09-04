use gkr::{BN254ConfigSha2Hyrax, M31x16ConfigSha2RawVanilla};
use gkr_engine::GKREngine;

use crate::{
    frontend::{BN254Config, BabyBearConfig, Config, GF2Config, GoldilocksConfig, M31Config},
    zkcuda::proving_system::expander_pcs_defered::{BN254ConfigMIMCUniKZG, BN254ConfigSha2UniKZG},
};

pub trait ZKCudaConfig {
    type ECCConfig: Config;
    type GKRConfig: GKREngine<FieldConfig = <Self::ECCConfig as GKREngine>::FieldConfig>;

    const BATCH_PCS: bool = false;
}

pub type GetPCS<ZKCConfig> = <<ZKCConfig as ZKCudaConfig>::GKRConfig as GKREngine>::PCSConfig;
pub type GetTranscript<ZKCConfig> =
    <<ZKCConfig as ZKCudaConfig>::GKRConfig as GKREngine>::TranscriptConfig;
pub type GetFieldConfig<ZKCConfig> =
    <<ZKCConfig as ZKCudaConfig>::GKRConfig as GKREngine>::FieldConfig;

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
pub type ZKCudaBN254MIMCKZG<'a> = ZKCudaConfigImpl<BN254Config, BN254ConfigMIMCUniKZG<'a>, false>;

pub type ZKCudaM31<'a> = ZKCudaConfigImpl<M31Config, M31x16ConfigSha2RawVanilla<'a>, false>;
pub type ZKCudaGF2<'a> = ZKCudaConfigImpl<GF2Config, GF2Config, false>;
pub type ZKCudaGoldilocks<'a> = ZKCudaConfigImpl<GoldilocksConfig, GoldilocksConfig, false>;
pub type ZKCudaBabyBear<'a> = ZKCudaConfigImpl<BabyBearConfig, BabyBearConfig, false>;

// Batch PCS types
pub type ZKCudaBN254HyraxBatchPCS<'a> =
    ZKCudaConfigImpl<BN254Config, BN254ConfigSha2Hyrax<'a>, true>;
pub type ZKCudaBN254KZGBatchPCS<'a> =
    ZKCudaConfigImpl<BN254Config, BN254ConfigSha2UniKZG<'a>, true>;
pub type ZKCudaBN254MIMCKZGBatchPCS<'a> =
    ZKCudaConfigImpl<BN254Config, BN254ConfigMIMCUniKZG<'a>, true>;

pub type ZKCudaM31BatchPCS<'a> = ZKCudaConfigImpl<M31Config, M31Config, true>;
pub type ZKCudaGF2BatchPCS<'a> = ZKCudaConfigImpl<GF2Config, GF2Config, true>;
pub type ZKCudaGoldilocksBatchPCS<'a> = ZKCudaConfigImpl<GoldilocksConfig, GoldilocksConfig, true>;
pub type ZKCudaBabyBearBatchPCS<'a> = ZKCudaConfigImpl<BabyBearConfig, BabyBearConfig, true>;

use expander_utils::timer::Timer;
use gkr_engine::{ExpanderPCS, GKREngine, MPIConfig, StructuredReferenceString};
use polynomials::RefMultiLinearPoly;

use super::structs::ExpanderProverSetup;
use crate::{
    frontend::{Config, SIMDField},
    zkcuda::proving_system::expander::structs::{ExpanderCommitment, ExpanderCommitmentState},
};

pub fn local_commit_impl<C, ECCConfig>(
    p_key: &<<C::PCSConfig as ExpanderPCS<C::FieldConfig>>::SRS as StructuredReferenceString>::PKey,
    vals: &[SIMDField<C>],
) -> (
    ExpanderCommitment<C::FieldConfig, C::PCSConfig>,
    ExpanderCommitmentState<C::FieldConfig, C::PCSConfig>,
)
where
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
{
    let timer = Timer::new("commit", true);

    let n_vars = vals.len().ilog2() as usize;
    let params = <C::PCSConfig as ExpanderPCS<C::FieldConfig>>::gen_params(n_vars, 1);

    let mut scratch = <C::PCSConfig as ExpanderPCS<C::FieldConfig>>::init_scratch_pad(
        &params,
        &MPIConfig::prover_new(None, None),
    );

    let commitment = <C::PCSConfig as ExpanderPCS<C::FieldConfig>>::commit(
        &params,
        &MPIConfig::prover_new(None, None),
        p_key,
        &RefMultiLinearPoly::from_ref(vals),
        &mut scratch,
    )
    .unwrap();

    timer.stop();
    (
        ExpanderCommitment {
            vals_len: vals.len(),
            commitment,
        },
        ExpanderCommitmentState { scratch },
    )
}

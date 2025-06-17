use expander_utils::timer::Timer;
use gkr_engine::{ExpanderPCS, FieldEngine, GKREngine, MPIConfig};
use polynomials::RefMultiLinearPoly;

use super::structs::ExpanderProverSetup;
use crate::{
    frontend::{Config, SIMDField},
    zkcuda::proving_system::expander::structs::{ExpanderCommitment, ExpanderCommitmentState},
};

pub fn local_commit_impl<C, ECCConfig>(
    prover_setup: &ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    vals: &[SIMDField<C>],
) -> (
    ExpanderCommitment<C::PCSField, C::FieldConfig, C::PCSConfig>,
    ExpanderCommitmentState<C::PCSField, C::FieldConfig, C::PCSConfig>,
)
where
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let timer = Timer::new("commit", true);

    let n_vars = vals.len().ilog2() as usize;
    let params = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(n_vars, 1);
    let p_key = prover_setup.p_keys.get(&vals.len()).unwrap();

    let mut scratch = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::init_scratch_pad(
        &params,
        &MPIConfig::prover_new(None, None),
    );

    let commitment = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::commit(
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

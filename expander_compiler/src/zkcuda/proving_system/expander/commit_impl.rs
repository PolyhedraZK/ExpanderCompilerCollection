use gkr_engine::{FieldEngine, GKREngine};

use crate::frontend::{Config, SIMDField};
use super::structs::ExpanderProverSetup;

pub fn local_commit_impl<C, ECCConfig>(
    prover_setup: &ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    vals: &[SIMDField<C>],
)
where 
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let timer = Timer::new("commit", true);

        let n_vars = vals.len().ilog2() as usize;
        let params =
            <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(n_vars, 1);
        let p_key = prover_setup.p_keys.get(&vals.len()).unwrap();

        let mut scratch =
            <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::init_scratch_pad(
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
            Self::Commitment {
                vals_len: vals.len(),
                commitment,
            },
            Self::CommitmentState { scratch },
        )
}
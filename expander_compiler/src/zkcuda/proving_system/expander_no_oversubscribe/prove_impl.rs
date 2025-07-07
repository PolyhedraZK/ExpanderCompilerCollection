use arith::Field;
use expander_utils::timer::Timer;
use gkr_engine::{
    ExpanderDualVarChallenge, ExpanderSingleVarChallenge, FieldEngine, GKREngine, MPIConfig,
    MPIEngine, Transcript,
};

use crate::{
    frontend::{Config, SIMDField},
    utils::misc::next_power_of_two,
    zkcuda::{
        context::ComputationGraph,
        kernel::Kernel,
        proving_system::{
            expander::{
                commit_impl::local_commit_impl,
                prove_impl::{
                    get_local_vals, pcs_local_open_impl, prepare_expander_circuit,
                    prove_gkr_with_local_vals,
                },
                structs::{ExpanderCommitmentState, ExpanderProof, ExpanderProverSetup},
            },
            expander_parallelized::{
                prove_impl::prove_kernel_gkr, server_ctrl::generate_local_mpi_config,
            },
            CombinedProof, Expander,
        },
    },
};

pub fn mpi_prove_impl<C, ECCConfig>(
    global_mpi_config: &MPIConfig<'static>,
    prover_setup: &ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    computation_graph: &ComputationGraph<ECCConfig>,
    values: &[impl AsRef<[SIMDField<C>]>],
) -> Option<CombinedProof<ECCConfig, Expander<C>>>
where
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let commit_timer = Timer::new("Commit to all input", global_mpi_config.is_root());
    let (commitments, states) = if global_mpi_config.is_root() {
        let (commitments, states) = values
            .iter()
            .map(|value| local_commit_impl::<C, ECCConfig>(prover_setup, value.as_ref()))
            .unzip::<_, _, Vec<_>, Vec<_>>();
        (Some(commitments), Some(states))
    } else {
        (None, None)
    };
    commit_timer.stop();

    let prove_timer = Timer::new("Prove all kernels", global_mpi_config.is_root());
    let proofs = computation_graph
        .proof_templates()
        .iter()
        .map(|template| {
            let commitment_values = template
                .commitment_indices()
                .iter()
                .map(|&idx| values[idx].as_ref())
                .collect::<Vec<_>>();

            let single_kernel_gkr_timer =
                Timer::new("small gkr kernel", global_mpi_config.is_root());
            let gkr_end_state = prove_kernel_gkr::<C::FieldConfig, C::TranscriptConfig, ECCConfig>(
                global_mpi_config,
                &computation_graph.kernels()[template.kernel_id()],
                &commitment_values,
                next_power_of_two(template.parallel_count()),
                template.is_broadcast(),
            );
            single_kernel_gkr_timer.stop();

            if global_mpi_config.is_root() {
                let pcs_open_timer = Timer::new("pcs open", true);
                let (mut transcript, challenge) = gkr_end_state.unwrap();
                let challenges = if let Some(challenge_y) = challenge.challenge_y() {
                    vec![challenge.challenge_x(), challenge_y]
                } else {
                    vec![challenge.challenge_x()]
                };

                challenges.iter().for_each(|c| {
                    partition_single_gkr_claim_and_open_pcs_mpi::<C>(
                        prover_setup,
                        &commitment_values,
                        &template
                            .commitment_indices()
                            .iter()
                            .map(|&idx| &states.as_ref().unwrap()[idx])
                            .collect::<Vec<_>>(),
                        c,
                        template.is_broadcast(),
                        &mut transcript,
                    );
                });

                pcs_open_timer.stop();
                Some(ExpanderProof {
                    data: vec![transcript.finalize_and_get_proof()],
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    prove_timer.stop();

    if global_mpi_config.is_root() {
        let proofs = proofs.into_iter().map(|p| p.unwrap()).collect::<Vec<_>>();
        Some(CombinedProof {
            commitments: commitments.unwrap(),
            proofs,
        })
    } else {
        None
    }
}

pub fn partition_challenge_and_location_for_pcs_mpi<F: FieldEngine>(
    gkr_challenge: &ExpanderSingleVarChallenge<F>,
    total_vals_len: usize,
    parallel_count: usize,
    is_broadcast: bool,
) -> (ExpanderSingleVarChallenge<F>, Vec<F::ChallengeField>) {
    let mut challenge = gkr_challenge.clone();
    let zero = F::ChallengeField::ZERO;
    if is_broadcast {
        let n_vals_vars = total_vals_len.ilog2() as usize;
        let component_idx_vars = challenge.rz[n_vals_vars..].to_vec();
        challenge.rz.resize(n_vals_vars, zero);
        challenge.r_mpi.clear();
        (challenge, component_idx_vars)
    } else {
        let n_vals_vars = (total_vals_len / parallel_count).ilog2() as usize;
        let component_idx_vars = challenge.rz[n_vals_vars..].to_vec();
        challenge.rz.resize(n_vals_vars, zero);

        challenge.rz.extend_from_slice(&challenge.r_mpi);
        challenge.r_mpi.clear();
        (challenge, component_idx_vars)
    }
}

#[allow(clippy::too_many_arguments)]
fn partition_single_gkr_claim_and_open_pcs_mpi<C: GKREngine>(
    p_keys: &ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    commitments_values: &[impl AsRef<[SIMDField<C>]>],
    commitments_state: &[&ExpanderCommitmentState<C::PCSField, C::FieldConfig, C::PCSConfig>],
    gkr_challenge: &ExpanderSingleVarChallenge<C::FieldConfig>,
    is_broadcast: &[bool],
    transcript: &mut C::TranscriptConfig,
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let parallel_count = 1 << gkr_challenge.r_mpi.len();
    for ((commitment_val, _state), ib) in commitments_values
        .iter()
        .zip(commitments_state)
        .zip(is_broadcast)
    {
        let val_len = commitment_val.as_ref().len();
        let (challenge_for_pcs, _) = partition_challenge_and_location_for_pcs_mpi(
            gkr_challenge,
            val_len,
            parallel_count,
            *ib,
        );

        pcs_local_open_impl::<C>(
            commitment_val.as_ref(),
            &challenge_for_pcs,
            p_keys,
            transcript,
        );
    }
}

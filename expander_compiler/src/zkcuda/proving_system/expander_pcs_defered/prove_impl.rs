use arith::Field;
use gkr_engine::{
    ExpanderPCS, ExpanderSingleVarChallenge, FieldEngine, GKREngine, MPIConfig, MPIEngine,
    Proof as BytesProof, Transcript,
};
use polynomials::MultiLinearPoly;
use serdes::ExpSerde;

use crate::{
    frontend::{Config, SIMDField},
    utils::misc::next_power_of_two,
    zkcuda::{
        proof::ComputationGraph,
        proving_system::{
            expander::{
                commit_impl::local_commit_impl,
                structs::{
                    ExpanderCommitment, ExpanderCommitmentState, ExpanderProof, ExpanderProverSetup,
                },
            },
            expander_parallelized::prove_impl::{partition_challenge_and_location_for_pcs_mpi, prove_kernel_gkr},
            CombinedProof, Expander,
        },
    },
};

pub fn pad_vals_and_commit<C, ECCConfig>(
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
    assert_eq!(prover_setup.p_keys.len(), 1);
    let len_to_commit = prover_setup.p_keys.keys().next().cloned().unwrap();

    let actual_len = vals.len();
    assert!(len_to_commit >= actual_len);

    // padding to max length and commit, this may be very inefficient
    // TODO: optimize this
    let mut vals = vals.to_vec();
    vals.resize(len_to_commit, SIMDField::<C>::ZERO);
    let (mut commitment, state) = local_commit_impl::<C, ECCConfig>(prover_setup, &vals);

    commitment.vals_len = actual_len; // Store the actual length in the commitment
    (commitment, state)
}

pub fn open_defered_pcs<C, ECCConfig>(
    prover_setup: &ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    vals: &[&[SIMDField<C>]],
    challenges: &[ExpanderSingleVarChallenge<C::FieldConfig>],
) -> ExpanderProof
where
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    // TODO: Efficiency
    let polys: Vec<_> = vals
        .iter()
        .map(|v| MultiLinearPoly::new(v.to_vec()))
        .collect();

    // TODO: Soundness
    let mut transcript = C::TranscriptConfig::new();
    let max_length = prover_setup.p_keys.keys().max().cloned().unwrap_or(0);
    let params = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(
        max_length.ilog2() as usize,
        1,
    );
    let scratch_pad = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::init_scratch_pad(
        &params,
        &MPIConfig::prover_new(None, None),
    );

    transcript.lock_proof();
    let (vals, opening) =
        <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::multi_points_batch_open(
            &params,
            &MPIConfig::prover_new(None, None),
            prover_setup.p_keys.get(&max_length).unwrap(),
            &polys,
            challenges,
            &scratch_pad,
            &mut transcript,
        );
    transcript.unlock_proof();

    let mut bytes = vec![];
    vals.serialize_into(&mut bytes).unwrap();
    opening.serialize_into(&mut bytes).unwrap();

    ExpanderProof {
        data: vec![BytesProof { bytes }],
    }
}

pub fn mpi_prove_with_pcs_defered<C, ECCConfig>(
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
    let (commitments, _states) = if global_mpi_config.is_root() {
        let (commitments, states) = values
            .iter()
            .map(|value| pad_vals_and_commit::<C, ECCConfig>(prover_setup, value.as_ref()))
            .unzip::<_, _, Vec<_>, Vec<_>>();
        (Some(commitments), Some(states))
    } else {
        (None, None)
    };

    let mut vals_ref = vec![];
    let mut challenges = vec![];

    let proofs = computation_graph
        .proof_templates
        .iter()
        .map(|template| {
            let commitment_values = template
                .commitment_indices
                .iter()
                .map(|&idx| values[idx].as_ref())
                .collect::<Vec<_>>();

            let gkr_end_state = prove_kernel_gkr::<C, ECCConfig>(
                global_mpi_config,
                &computation_graph.kernels[template.kernel_id],
                &commitment_values,
                next_power_of_two(template.parallel_count),
                &template.is_broadcast,
            );

            if global_mpi_config.is_root() {
                let (mut transcript, challenge) = gkr_end_state.unwrap();
                assert!(challenge.challenge_y().is_none());
                let challenge = challenge.challenge_x();

                let (local_vals_ref, local_challenges) = extract_pcs_claims::<C>(
                    &commitment_values,
                    &challenge,
                    &template.is_broadcast,
                    next_power_of_two(template.parallel_count),
                );

                vals_ref.extend(local_vals_ref);
                challenges.extend(local_challenges);

                Some(ExpanderProof {
                    data: vec![transcript.finalize_and_get_proof()],
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if global_mpi_config.is_root() {
        let mut proofs = proofs.into_iter().map(|p| p.unwrap()).collect::<Vec<_>>();

        let pcs_batch_opening =
            open_defered_pcs::<C, ECCConfig>(prover_setup, &vals_ref, &challenges);
        proofs.push(pcs_batch_opening);
        Some(CombinedProof {
            commitments: commitments.unwrap(),
            proofs,
        })
    } else {
        None
    }
}

pub fn extract_pcs_claims<'a, C: GKREngine>(
    commitments_values: &[&'a [SIMDField<C>]],
    gkr_challenge: &ExpanderSingleVarChallenge<C::FieldConfig>,
    is_broadcast: &[bool],
    parallel_count: usize,
) -> (
    Vec<&'a [SIMDField<C>]>,
    Vec<ExpanderSingleVarChallenge<C::FieldConfig>>,
)
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let mut commitment_values_rt = vec![];
    let mut challenges = vec![];

    for (&commitment_val, &ib) in commitments_values.iter().zip(is_broadcast) {
        let val_len = commitment_val.len();
        let (challenge_for_pcs, _) = partition_challenge_and_location_for_pcs_mpi(
            gkr_challenge,
            val_len,
            parallel_count,
            ib,
        );

        commitment_values_rt.push(commitment_val);
        challenges.push(challenge_for_pcs);
    }

    (commitment_values_rt, challenges)
}

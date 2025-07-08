use arith::{Field, Fr, SimdField};
use expander_utils::timer::Timer;
use gkr_engine::{
    BN254ConfigXN, ExpanderDualVarChallenge, ExpanderSingleVarChallenge, FieldEngine, FieldType,
    GKREngine, MPIConfig, MPIEngine, Transcript,
};

use crate::{
    frontend::{Config, SIMDField},
    utils::misc::next_power_of_two,
    zkcuda::{
        context::ComputationGraph,
        kernel::{Kernel, LayeredCircuitInputVec},
        proving_system::{
            expander::{
                commit_impl::local_commit_impl,
                prove_impl::{
                    get_local_vals, pcs_local_open_impl, prepare_expander_circuit,
                    prepare_inputs_with_local_vals,
                },
                structs::{ExpanderCommitmentState, ExpanderProof, ExpanderProverSetup},
            },
            expander_parallelized::prove_impl::partition_single_gkr_claim_and_open_pcs_mpi,
            expander_parallelized::server_ctrl::generate_local_mpi_config,
            CombinedProof, Expander,
        },
    },
};

pub fn mpi_prove_no_oversubscribe_impl<C, ECCConfig>(
    global_mpi_config: &MPIConfig<'static>,
    prover_setup: &ExpanderProverSetup<C::FieldConfig, C::PCSConfig>,
    computation_graph: &ComputationGraph<ECCConfig>,
    values: &[impl AsRef<[SIMDField<C>]>],
) -> Option<CombinedProof<ECCConfig, Expander<C>>>
where
    C: GKREngine,
    C::FieldConfig: FieldEngine<CircuitField = Fr, ChallengeField = Fr>,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
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
            let gkr_end_state = prove_kernel_gkr_no_oversubscribe::<
                C::FieldConfig,
                C::TranscriptConfig,
                ECCConfig,
            >(
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

#[allow(clippy::too_many_arguments)]
pub fn prove_kernel_gkr_no_oversubscribe<F, T, ECCConfig>(
    mpi_config: &MPIConfig<'static>,
    kernel: &Kernel<ECCConfig>,
    commitments_values: &[&[F::SimdCircuitField]],
    parallel_count: usize,
    is_broadcast: &[bool],
) -> Option<(T, ExpanderDualVarChallenge<F>)>
where
    F: FieldEngine<CircuitField = Fr, ChallengeField = Fr>,
    T: Transcript,
    ECCConfig: Config<FieldConfig = F>,
{
    let local_mpi_config = generate_local_mpi_config(mpi_config, parallel_count);

    local_mpi_config.as_ref()?;

    let local_mpi_config = local_mpi_config.unwrap();
    let local_world_size = local_mpi_config.world_size();

    let n_local_copies = parallel_count / local_world_size;
    match n_local_copies {
        1 => prove_kernel_gkr_internal::<F, F, T, ECCConfig>(
            mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
        ),
        2 => prove_kernel_gkr_internal::<F, BN254ConfigXN<2>, T, ECCConfig>(
            mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
        ),
        4 => prove_kernel_gkr_internal::<F, BN254ConfigXN<4>, T, ECCConfig>(
            mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
        ),
        8 => prove_kernel_gkr_internal::<F, BN254ConfigXN<8>, T, ECCConfig>(
            mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
        ),
        16 => prove_kernel_gkr_internal::<F, BN254ConfigXN<16>, T, ECCConfig>(
            mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
        ),
        32 => prove_kernel_gkr_internal::<F, BN254ConfigXN<32>, T, ECCConfig>(
            mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
        ),
        64 => prove_kernel_gkr_internal::<F, BN254ConfigXN<64>, T, ECCConfig>(
            mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
        ),
        128 => prove_kernel_gkr_internal::<F, BN254ConfigXN<128>, T, ECCConfig>(
            mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
        ),
        256 => prove_kernel_gkr_internal::<F, BN254ConfigXN<256>, T, ECCConfig>(
            mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
        ),
        512 => prove_kernel_gkr_internal::<F, BN254ConfigXN<512>, T, ECCConfig>(
            mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
        ),
        1024 => prove_kernel_gkr_internal::<F, BN254ConfigXN<1024>, T, ECCConfig>(
            mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
        ),
        2048 => prove_kernel_gkr_internal::<F, BN254ConfigXN<2048>, T, ECCConfig>(
            mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
        ),
        _ => {
            panic!("Unsupported parallel count: {}", parallel_count);
        }
    }
}

pub fn prove_kernel_gkr_internal<FBasic, FMulti, T, ECCConfig>(
    mpi_config: &MPIConfig<'static>,
    kernel: &Kernel<ECCConfig>,
    commitments_values: &[&[FBasic::SimdCircuitField]],
    parallel_count: usize,
    is_broadcast: &[bool],
) -> Option<(T, ExpanderDualVarChallenge<FBasic>)>
where
    FBasic: FieldEngine,
    FMulti:
        FieldEngine<CircuitField = FBasic::CircuitField, ChallengeField = FBasic::ChallengeField>,
    T: Transcript,
    ECCConfig: Config<FieldConfig = FBasic>,
{
    let world_rank = mpi_config.world_rank();
    let world_size = mpi_config.world_size();
    let n_copies = parallel_count / world_size;

    let local_commitment_values = get_local_vals_multi_copies(
        commitments_values,
        is_broadcast,
        world_rank,
        n_copies,
        parallel_count,
    );

    let (mut expander_circuit, mut prover_scratch) =
        prepare_expander_circuit::<FMulti, ECCConfig>(kernel, world_size);

    let mut transcript = T::new();
    let challenge = prove_gkr_with_local_vals_multi_copies::<FBasic, FMulti, T>(
        &mut expander_circuit,
        &mut prover_scratch,
        &local_commitment_values,
        kernel.layered_circuit_input(),
        &mut transcript,
        &mpi_config,
    );

    Some((transcript, challenge))
}

pub fn get_local_vals_multi_copies<'vals_life, F: Field>(
    global_vals: &'vals_life [impl AsRef<[F]>],
    is_broadcast: &[bool],
    local_world_rank: usize,
    n_copies: usize,
    parallel_count: usize,
) -> Vec<Vec<&'vals_life [F]>> {
    let parallel_indices = (0..n_copies)
        .map(|i| local_world_rank * n_copies + i)
        .collect::<Vec<_>>();

    parallel_indices
        .iter()
        .map(|&parallel_index| {
            get_local_vals(global_vals, is_broadcast, parallel_index, parallel_count)
        })
        .collect::<Vec<_>>()
}

pub fn prove_gkr_with_local_vals_multi_copies<FBasic, FMulti, T>(
    expander_circuit: &mut expander_circuit::Circuit<FMulti>,
    prover_scratch: &mut sumcheck::ProverScratchPad<FMulti>,
    local_commitment_values_multi_copies: &[Vec<impl AsRef<[FBasic::SimdCircuitField]>>],
    partition_info: &[LayeredCircuitInputVec],
    transcript: &mut T,
    mpi_config: &MPIConfig,
) -> ExpanderDualVarChallenge<FBasic>
where
    FBasic: FieldEngine,
    FMulti:
        FieldEngine<CircuitField = FBasic::CircuitField, ChallengeField = FBasic::ChallengeField>,
    T: Transcript,
{
    let input_vals_multi_copies = local_commitment_values_multi_copies
        .iter()
        .map(|local_commitment_values| {
            prepare_inputs_with_local_vals(
                1 << expander_circuit.log_input_size(),
                partition_info,
                local_commitment_values,
            )
        })
        .collect::<Vec<_>>();

    let mut input_vals =
        vec![FMulti::SimdCircuitField::ZERO; 1 << expander_circuit.log_input_size()];
    for (i, vals) in input_vals.iter_mut().enumerate() {
        let vals_unpacked = input_vals_multi_copies
            .iter()
            .flat_map(|v| v[i].unpack())
            .collect::<Vec<_>>();
        *vals = FMulti::SimdCircuitField::pack(&vals_unpacked);
    }
    expander_circuit.layers[0].input_vals = input_vals;

    expander_circuit.fill_rnd_coefs(transcript);
    expander_circuit.evaluate();
    let (claimed_v, challenge) =
        gkr::gkr_prove(expander_circuit, prover_scratch, transcript, mpi_config);
    assert_eq!(claimed_v, FBasic::ChallengeField::from(0));

    let n_simd_vars_basic = FBasic::SimdCircuitField::PACK_SIZE.ilog2() as usize;

    ExpanderDualVarChallenge {
        rz_0: challenge.rz_0,
        rz_1: challenge.rz_1,
        r_simd: challenge.r_simd[..n_simd_vars_basic].to_vec(),
        r_mpi: {
            let mut v = challenge.r_simd[n_simd_vars_basic..].to_vec();
            v.extend(&challenge.r_mpi);
            v
        },
    }
}

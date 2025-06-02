use expander_compiler::zkcuda::kernel::LayeredCircuitInputVec;
use expander_compiler::zkcuda::proof::ComputationGraph;
use mpi::environment::Universe;
use mpi::topology::SimpleCommunicator;
use mpi::traits::Communicator;
use std::cmp::max;
use std::collections::HashMap;

use arith::Field;
use expander_circuit::Circuit as ExpCircuit;
use expander_compiler::frontend::{Config, SIMDField};
use expander_compiler::zkcuda::proving_system::callee_utils::{
    read_broadcast_info_from_shared_memory, read_commitment_extra_info_from_shared_memory,
    read_commitment_from_shared_memory, read_commitment_values_from_shared_memory,
    read_partition_info_from_shared_memory, write_proof_to_shared_memory,
};
use expander_compiler::zkcuda::proving_system::callee_utils::{
    read_local_vals_to_commit_from_shared_memory, write_commitment_extra_info_to_shared_memory,
    write_commitment_to_shared_memory,
};
use expander_compiler::zkcuda::proving_system::{
    max_n_vars, pcs_testing_setup_fixed_seed, ExpanderGKRCommitment,
    ExpanderGKRCommitmentExtraInfo, ExpanderGKRProof, ExpanderGKRProverSetup,
    ExpanderGKRVerifierSetup,
};
use expander_utils::timer::Timer;

use gkr::gkr_prove;
use gkr_engine::{
    ExpanderPCS, ExpanderSingleVarChallenge, FieldEngine, GKREngine, MPIConfig, MPIEngine,
    Transcript,
};
use polynomials::RefMultiLinearPoly;
use serdes::ExpSerde;
use sumcheck::ProverScratchPad;

#[allow(clippy::type_complexity)]
pub fn setup<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    global_mpi_config: &MPIConfig<'static>,
    computation_graph: Option<&ComputationGraph<ECCConfig>>,
) -> (
    ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    ExpanderGKRVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
)
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let mut p_keys = HashMap::new();
    let mut v_keys = HashMap::new();

    if global_mpi_config.is_root() {
        let computation_graph = computation_graph.unwrap();
        for template in computation_graph.proof_templates.iter() {
            for (x, is_broadcast) in template
                .commitment_indices
                .iter()
                .zip(template.is_broadcast.iter())
            {
                let val_total_len = computation_graph.commitments_lens[*x];
                let val_actual_len = if *is_broadcast {
                    val_total_len
                } else {
                    val_total_len / template.parallel_count
                };
                if p_keys.contains_key(&(val_actual_len, template.parallel_count)) {
                    continue;
                }

                global_mpi_config.root_broadcast_f(&mut (val_actual_len, template.parallel_count));
                let local_mpi_config =
                    generate_local_mpi_config(global_mpi_config, template.parallel_count);
                let (_params, p_key, v_key, _scratch) = pcs_testing_setup_fixed_seed::<
                    C::FieldConfig,
                    C::TranscriptConfig,
                    C::PCSConfig,
                >(
                    val_actual_len,
                    local_mpi_config.as_ref().unwrap(),
                );
                p_keys.insert((val_actual_len, template.parallel_count), p_key);
                v_keys.insert((val_actual_len, template.parallel_count), v_key);
            }
        }
        global_mpi_config.root_broadcast_f(&mut (usize::MAX, usize::MAX)); // Signal the end of the loop
    } else {
        loop {
            let mut pair = (0usize, 0usize);
            global_mpi_config.root_broadcast_f(&mut pair);
            let (val_actual_len, parallel_count) = pair;
            if val_actual_len == usize::MAX || parallel_count == usize::MAX {
                break;
            }
            let local_mpi_config = generate_local_mpi_config(global_mpi_config, parallel_count);

            if let Some(local_mpi_config) = local_mpi_config {
                let (_params, p_key, v_key, _scratch) = pcs_testing_setup_fixed_seed::<
                    C::FieldConfig,
                    C::TranscriptConfig,
                    C::PCSConfig,
                >(
                    val_actual_len, &local_mpi_config
                );
                p_keys.insert((val_actual_len, parallel_count), p_key);
                v_keys.insert((val_actual_len, parallel_count), v_key);
            }
        }
    }

    (
        ExpanderGKRProverSetup { p_keys },
        ExpanderGKRVerifierSetup { v_keys },
    )
}

pub fn commit<C: GKREngine>(
    mpi_config: &MPIConfig,
    prover_setup: &ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let world_rank = mpi_config.world_rank();
    let world_size = mpi_config.world_size();
    if world_rank == 0 {
        println!("Expander Commit Exec Called with world size {}", world_size);
    }

    let local_vals_to_commit =
        read_local_vals_to_commit_from_shared_memory::<C::FieldConfig>(world_rank, world_size);
    let local_val_len = local_vals_to_commit.len();
    let p_key = prover_setup
        .p_keys
        .get(&(local_val_len, world_size))
        .unwrap();

    let params = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(
        local_val_len.ilog2() as usize,
        mpi_config.world_size(),
    );

    let mut scratch = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::init_scratch_pad(
        &params, mpi_config,
    );

    let commitment = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::commit(
        &params,
        mpi_config,
        p_key,
        &RefMultiLinearPoly::from_ref(&local_vals_to_commit),
        &mut scratch,
    );

    if world_rank == 0 {
        let commitment = ExpanderGKRCommitment {
            vals_len: local_val_len,
            commitment: vec![commitment.unwrap()],
        };
        let extra_info = ExpanderGKRCommitmentExtraInfo {
            scratch: vec![scratch],
        };

        write_commitment_to_shared_memory::<C::PCSField, C::FieldConfig, C::PCSConfig>(&commitment);
        write_commitment_extra_info_to_shared_memory::<C::PCSField, C::FieldConfig, C::PCSConfig>(
            &extra_info,
        );
    }
}

// Ideally, there will only one ECCConfig generics
// But we need to implement `Config` for each GKREngine, which remains to be done
// For now, the GKREngine actually controls the functionality of the prover
// The ECCConfig is only used where the `Config` trait is required
pub fn prove<C: GKREngine>(
    mpi_config: &MPIConfig,
    pcs_setup: &ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    expander_circuit: &mut ExpCircuit<C::FieldConfig>, // mut to allow filling rnd coefs and circuit inputs
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let world_rank = mpi_config.world_rank();
    let world_size = mpi_config.world_size();
    if world_rank == 0 {
        println!("Expander Prove Exec Called with world size {}", world_size);
    }

    let timer = Timer::new(
        "recurring cost: read witness&commitment",
        mpi_config.is_root(),
    );
    let _commitments = if mpi_config.is_root() {
        Some(read_commitment_from_shared_memory::<
            C::PCSField,
            C::FieldConfig,
            C::PCSConfig,
        >())
    } else {
        None
    };
    let partition_info = read_partition_info_from_shared_memory();
    let broadcast_info = read_broadcast_info_from_shared_memory();
    let commitments_extra_info =
        read_commitment_extra_info_from_shared_memory::<C::PCSField, C::FieldConfig, C::PCSConfig>(
        );
    let local_commitment_values = read_commitment_values_from_shared_memory::<C::FieldConfig>(
        &broadcast_info,
        world_rank,
        world_size,
    );
    timer.stop();

    let timer = Timer::new("gkr prove", mpi_config.is_root());
    let (max_num_input_var, max_num_output_var) = max_n_vars(expander_circuit);
    let max_num_var = max(max_num_input_var, max_num_output_var); // temp fix to a bug in Expander, remove this after Expander update.
    let mut prover_scratch =
        ProverScratchPad::<C::FieldConfig>::new(max_num_var, max_num_var, world_size);

    let mut transcript = C::TranscriptConfig::new();
    transcript.append_u8_slice(&[0u8; 32]); // TODO: Replace with the commitment, and hash an additional a few times
    expander_circuit.layers[0].input_vals = prepare_inputs(
        1usize << expander_circuit.log_input_size(),
        &partition_info,
        &local_commitment_values,
    );
    expander_circuit.fill_rnd_coefs(&mut transcript);
    expander_circuit.evaluate();
    let (claimed_v, challenge) = gkr_prove(
        expander_circuit,
        &mut prover_scratch,
        &mut transcript,
        mpi_config,
    );
    assert_eq!(
        claimed_v,
        <C::FieldConfig as FieldEngine>::ChallengeField::from(0)
    );
    timer.stop();

    let timer = Timer::new("pcs opening", mpi_config.is_root());
    prove_input_claim::<C>(
        mpi_config,
        &local_commitment_values,
        pcs_setup,
        &commitments_extra_info,
        &challenge.challenge_x(),
        &broadcast_info,
        &mut transcript,
    );
    if let Some(challenge_y) = challenge.challenge_y() {
        prove_input_claim::<C>(
            mpi_config,
            &local_commitment_values,
            pcs_setup,
            &commitments_extra_info,
            &challenge_y,
            &broadcast_info,
            &mut transcript,
        );
    }
    timer.stop();

    let proof = transcript.finalize_and_get_proof();
    if world_rank == 0 {
        write_proof_to_shared_memory(&ExpanderGKRProof { data: vec![proof] });
    }
}

#[allow(clippy::too_many_arguments)]
fn prove_input_claim<C: GKREngine>(
    mpi_config: &MPIConfig,
    local_commitments_values: &[Vec<SIMDField<C>>],
    p_keys: &ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    commitments_extra_info: &[ExpanderGKRCommitmentExtraInfo<
        C::PCSField,
        C::FieldConfig,
        C::PCSConfig,
    >],
    challenge: &ExpanderSingleVarChallenge<C::FieldConfig>,
    is_broadcast: &[bool],
    transcript: &mut C::TranscriptConfig,
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    for ((local_commitment_val, extra_info), _ib) in local_commitments_values
        .iter()
        .zip(commitments_extra_info)
        .zip(is_broadcast)
    {
        let val_len = local_commitment_val.len();
        let vals_to_open = local_commitment_val;

        let nb_challenge_vars = val_len.ilog2() as usize;
        let challenge_vars = challenge.rz[..nb_challenge_vars].to_vec();

        let params = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(
            val_len,
            mpi_config.world_size(),
        );
        let p_key = p_keys
            .p_keys
            .get(&(val_len, mpi_config.world_size()))
            .unwrap();

        let poly = RefMultiLinearPoly::from_ref(vals_to_open);
        let v = C::FieldConfig::collectively_eval_circuit_vals_at_expander_challenge(
            vals_to_open,
            &ExpanderSingleVarChallenge::<C::FieldConfig> {
                rz: challenge_vars.to_vec(),
                r_simd: challenge.r_simd.to_vec(),
                r_mpi: challenge.r_mpi.to_vec(),
            },
            &mut vec![<C::FieldConfig as FieldEngine>::Field::ZERO; val_len],
            &mut vec![
                <C::FieldConfig as FieldEngine>::ChallengeField::ZERO;
                1 << max(challenge.r_simd.len(), challenge.r_mpi.len())
            ],
            mpi_config,
        );
        transcript.append_field_element(&v);

        transcript.lock_proof();
        let opening = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::open(
            &params,
            mpi_config,
            p_key,
            &poly,
            &ExpanderSingleVarChallenge::<C::FieldConfig> {
                rz: challenge_vars.to_vec(),
                r_simd: challenge.r_simd.to_vec(),
                r_mpi: challenge.r_mpi.to_vec(),
            },
            transcript,
            &extra_info.scratch[0],
        );
        transcript.unlock_proof();

        if mpi_config.is_root() {
            let mut buffer = vec![];
            opening
                .unwrap()
                .serialize_into(&mut buffer)
                .expect("Failed to serialize opening");
            transcript.append_u8_slice(&buffer);
        }
    }
}

fn prepare_inputs<F: Field>(
    input_len: usize,
    partition_info: &[LayeredCircuitInputVec],
    local_commitment_values: &[Vec<F>],
) -> Vec<F> {
    let mut input_vals = vec![F::ZERO; input_len];
    for (partition, val) in partition_info.iter().zip(local_commitment_values.iter()) {
        assert!(partition.len == val.len());
        input_vals[partition.offset..partition.offset + partition.len].copy_from_slice(val);
    }
    input_vals
}

// pub fn verify<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
//     verifier_setup: &ExpanderGKRVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
//     expander_circuit: &mut ExpCircuit<C::FieldConfig>,
//     proof: &ExpanderGKRProof,
//     commitments: &[ExpanderGKRCommitment<C::PCSField, C::FieldConfig, C::PCSConfig>],
//     partition_info: &[LayeredCircuitInputVec],
//     parallel_count: usize,
//     is_broadcast: &[bool],
// ) -> bool
// where
//     C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
// {
//     let timer = Timer::new("verify", true);

//     let mut transcript = C::TranscriptConfig::new();
//     transcript.append_u8_slice(&[0u8; 32]);
//     expander_circuit.fill_rnd_coefs(&mut transcript);
//     let mut cursor = Cursor::new(&proof.data[0].bytes);
//     cursor.set_position(32);

//     let (mut verified, challenge, claimed_v0, claimed_v1) = gkr_verify(
//         parallel_count,
//         expander_circuit,
//         &[],
//         &<C::FieldConfig as FieldEngine>::ChallengeField::ZERO,
//         &mut transcript,
//         &mut cursor,
//     );

//     let pcs_verification_timer = Timer::new("pcs verification", true);
//     verified &= verify_input_claim::<C, ECCConfig>(
//         &mut cursor,
//         partition_info,
//         verifier_setup,
//         &challenge.challenge_x(),
//         &claimed_v0,
//         commitments,
//         is_broadcast,
//         parallel_count,
//         &mut transcript,
//     );
//     if let Some(challenge_y) = challenge.challenge_y() {
//         verified &= verify_input_claim::<C, ECCConfig>(
//             &mut cursor,
//             partition_info,
//             verifier_setup,
//             &challenge_y,
//             &claimed_v1.unwrap(),
//             commitments,
//             is_broadcast,
//             parallel_count,
//             &mut transcript,
//         );
//     }
//     pcs_verification_timer.stop();

//     timer.stop();
//     verified
// }

// #[allow(clippy::too_many_arguments)]
// fn verify_input_claim<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
//     mut proof_reader: impl Read,
//     partition_info: &[LayeredCircuitInputVec],
//     v_keys: &ExpanderGKRVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
//     challenge: &ExpanderSingleVarChallenge<C::FieldConfig>,
//     y: &<C::FieldConfig as FieldEngine>::ChallengeField,
//     commitments: &[ExpanderGKRCommitment<C::PCSField, C::FieldConfig, C::PCSConfig>],
//     is_broadcast: &[bool],
//     parallel_count: usize,
//     transcript: &mut C::TranscriptConfig,
// ) -> bool
// where
//     C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
// {
//     assert_eq!(1 << challenge.r_mpi.len(), parallel_count);
//     let mut target_y = <C::FieldConfig as FieldEngine>::ChallengeField::ZERO;
//     for ((input, commitment), ib) in partition_info.iter().zip(commitments).zip(is_broadcast) {
//         let local_vals_len =
//             <ExpanderGKRCommitment<C::PCSField, C::FieldConfig, C::PCSConfig> as Commitment<
//                 ECCConfig,
//             >>::vals_len(commitment);
//         let nb_challenge_vars = local_vals_len.ilog2() as usize;
//         let challenge_vars = challenge.rz[..nb_challenge_vars].to_vec();

//         let params = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(
//             nb_challenge_vars,
//             parallel_count,
//         );
//         let v_key = v_keys
//             .v_keys
//             .get(&(local_vals_len, parallel_count))
//             .unwrap();

//         let claim =
//             <C::FieldConfig as FieldEngine>::ChallengeField::deserialize_from(&mut proof_reader)
//                 .unwrap();
//         transcript.append_field_element(&claim);

//         let opening =
//             <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::Opening::deserialize_from(
//                 &mut proof_reader,
//             )
//             .unwrap();

//         transcript.lock_proof();
//         // individual pcs verification
//         let verified = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::verify(
//             &params,
//             v_key,
//             &commitment.commitment[0],
//             &ExpanderSingleVarChallenge::<C::FieldConfig> {
//                 rz: challenge_vars.to_vec(),
//                 r_simd: challenge.r_simd.to_vec(),
//                 r_mpi: if *ib {
//                     vec![]
//                 } else {
//                     challenge.r_mpi.to_vec()
//                 }, // In the case of broadcast, whatever x_mpi is, the opening is the same
//             },
//             claim,
//             transcript,
//             &opening,
//         );
//         transcript.unlock_proof();

//         if !verified {
//             return false;
//         }

//         let index_vars = &challenge.rz[nb_challenge_vars..];
//         let index = input.offset / input.len;
//         let index_as_bits = (0..index_vars.len())
//             .map(|i| {
//                 <C::FieldConfig as FieldEngine>::ChallengeField::from(((index >> i) & 1) as u32)
//             })
//             .collect::<Vec<_>>();
//         let v_index = EqPolynomial::<<C::FieldConfig as FieldEngine>::ChallengeField>::eq_vec(
//             index_vars,
//             &index_as_bits,
//         );

//         target_y += v_index * claim;
//     }

//     // overall claim verification
//     *y == target_y
// }

// TODO: Find a way to avoid this global state
pub static mut UNIVERSE: Option<Universe> = None;
pub static mut GLOBAL_COMMUNICATOR: Option<SimpleCommunicator> = None;
pub static mut LOCAL_COMMUNICATOR: Option<SimpleCommunicator> = None;

#[allow(static_mut_refs)]
pub fn generate_local_mpi_config(
    global_mpi_config: &MPIConfig<'static>,
    n_parties: usize,
) -> Option<MPIConfig<'static>> {
    assert!(n_parties > 0, "Number of parties must be greater than 0");

    let rank = global_mpi_config.world_rank();
    let color_v = if rank < n_parties { 0 } else { 1 };
    let color = mpi::topology::Color::with_value(color_v);
    unsafe {
        LOCAL_COMMUNICATOR = global_mpi_config
            .world
            .unwrap()
            .split_by_color_with_key(color, rank as i32);
    }
    if color_v == 0 {
        Some(MPIConfig::prover_new(global_mpi_config.universe, unsafe {
            LOCAL_COMMUNICATOR.as_ref()
        }))
    } else {
        None
    }
}

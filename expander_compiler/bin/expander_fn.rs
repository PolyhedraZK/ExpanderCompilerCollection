use expander_compiler::zkcuda::kernel::LayeredCircuitInputVec;
use mpi::ffi::MPI_Win;
use std::cmp::max;

use arith::Field;
use expander_circuit::Circuit as ExpCircuit;
use expander_compiler::frontend::{Config, SIMDField};
use expander_compiler::zkcuda::proving_system::callee_utils::{
    read_broadcast_info_from_shared_memory, read_commitment_extra_info_from_shared_memory,
    read_commitment_from_shared_memory, read_commitment_values_from_shared_memory,
    read_ecc_circuit_from_shared_memory, read_partition_info_from_shared_memory,
    write_proof_to_shared_memory,
};
use expander_compiler::zkcuda::proving_system::callee_utils::{
    read_local_vals_to_commit_from_shared_memory, read_selected_pkey_from_shared_memory,
    write_commitment_extra_info_to_shared_memory, write_commitment_to_shared_memory,
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

pub fn setup<C: GKREngine>(
    mpi_config: &MPIConfig,
    local_val_len: usize,
    p_keys: &mut ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    v_keys: &mut ExpanderGKRVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    // There might be a case local_val_len is the same but mpi size is different
    // TODO: Handle this case
    if p_keys.p_keys.contains_key(&local_val_len) {
        // If the key already exists, we can skip the setup
        return;
    }

    let (_params, p_key, v_key, _scratch) = pcs_testing_setup_fixed_seed::<
        C::FieldConfig,
        C::TranscriptConfig,
        C::PCSConfig,
    >(local_val_len, mpi_config);

    p_keys.p_keys.insert(local_val_len, p_key);
    v_keys.v_keys.insert(local_val_len, v_key);
}

pub fn register_kernel<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    mpi_config: &MPIConfig,
    kernels: &mut Vec<(ExpCircuit<C::FieldConfig>, MPI_Win)>,
) {
    let (mut expander_circuit, window) = if mpi_config.is_root() {
        let ecc_circuit = read_ecc_circuit_from_shared_memory::<ECCConfig>();
        let expander_circuit = ecc_circuit.export_to_expander().flatten::<C>();
        mpi_config.consume_obj_and_create_shared(Some(expander_circuit))
    } else {
        mpi_config.consume_obj_and_create_shared(None)
    };
    expander_circuit.pre_process_gkr::<C>();
    kernels.push((expander_circuit, window));
}

pub fn commit<C: GKREngine>(mpi_config: &MPIConfig)
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let world_rank = mpi_config.world_rank();
    let world_size = mpi_config.world_size();
    assert!(
        world_size > 1,
        "In case world_size is 1, we should not use the mpi version of the prover"
    );
    if world_rank == 0 {
        println!("Expander Commit Exec Called with world size {}", world_size);
    }

    let (local_val_len, p_key) =
        read_selected_pkey_from_shared_memory::<C::PCSField, C::FieldConfig, C::PCSConfig>();

    let local_vals_to_commit =
        read_local_vals_to_commit_from_shared_memory::<C::FieldConfig>(world_rank, world_size);

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
        &p_key,
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
pub fn prove<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    mpi_config: &MPIConfig,
    pcs_setup: &ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    expander_circuit: &mut ExpCircuit<C::FieldConfig>, // mut to allow filling rnd coefs and circuit inputs
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let world_rank = mpi_config.world_rank();
    let world_size = mpi_config.world_size();
    assert!(
        world_size > 1,
        "In case world_size is 1, we should not use the mpi version of the prover"
    );
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
    let (max_num_input_var, max_num_output_var) = max_n_vars(&expander_circuit);
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
        &expander_circuit,
        &mut prover_scratch,
        &mut transcript,
        &mpi_config,
    );
    assert_eq!(
        claimed_v,
        <C::FieldConfig as FieldEngine>::ChallengeField::from(0)
    );
    timer.stop();

    let timer = Timer::new("pcs opening", mpi_config.is_root());
    prove_input_claim::<C>(
        &mpi_config,
        &local_commitment_values,
        &pcs_setup,
        &commitments_extra_info,
        &challenge.challenge_x(),
        &broadcast_info,
        &mut transcript,
    );
    if let Some(challenge_y) = challenge.challenge_y() {
        prove_input_claim::<C>(
            &mpi_config,
            &local_commitment_values,
            &pcs_setup,
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
        let p_key = p_keys.p_keys.get(&val_len).unwrap();

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

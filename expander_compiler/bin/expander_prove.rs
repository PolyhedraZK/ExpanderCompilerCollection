mod common;
use common::ExpanderExecArgs;

use clap::Parser;
use expander_compiler::zkcuda::kernel::LayeredCircuitInputVec;
use std::cmp::max;
use std::str::FromStr;

use arith::Field;
use expander_compiler::frontend::{
    BN254Config, BabyBearConfig, Config, GF2Config, GoldilocksConfig, M31Config, SIMDField,
};
use expander_compiler::zkcuda::proving_system::callee_utils::{
    read_broadcast_info_from_shared_memory, read_commitment_extra_info_from_shared_memory,
    read_commitment_from_shared_memory, read_commitment_values_from_shared_memory,
    read_ecc_circuit_from_shared_memory, read_partition_info_from_shared_memory,
    read_pcs_setup_from_shared_memory, write_proof_to_shared_memory,
};
use expander_compiler::zkcuda::proving_system::{
    max_n_vars, ExpanderGKRCommitmentExtraInfo, ExpanderGKRProof, ExpanderGKRProverSetup,
};
use expander_utils::timer::Timer;

use gkr::{gkr_prove, BN254ConfigSha2Hyrax, BN254ConfigSha2KZG};
use gkr_engine::{
    ExpanderPCS, ExpanderSingleVarChallenge, FieldEngine, GKREngine, MPIConfig, MPIEngine,
    PolynomialCommitmentType, SharedMemory, Transcript,
};
use polynomials::RefMultiLinearPoly;
use serdes::ExpSerde;
use sumcheck::ProverScratchPad;

// Ideally, there will only one ECCConfig generics
// But we need to implement `Config` for each GKREngine, which remains to be done
// For now, the GKREngine actually controls the functionality of the prover
// The ECCConfig is only used where the `Config` trait is required
fn prove<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>()
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let mpi_config = MPIConfig::prover_new();
    let world_rank = mpi_config.world_rank();
    let world_size = mpi_config.world_size();
    assert!(
        world_size > 1,
        "In case world_size is 1, we should not use the mpi version of the prover"
    );
    if world_rank == 0 {
        println!("Expander Prove Exec Called with world size {}", world_size);
    }

    let timer = Timer::new("one time cost: read setup&circuit", mpi_config.is_root());
    let pcs_setup =
        read_pcs_setup_from_shared_memory::<C::PCSField, C::FieldConfig, C::PCSConfig>();
    let (mut expander_circuit, mut window) = if mpi_config.is_root() {
        let ecc_circuit = read_ecc_circuit_from_shared_memory::<ECCConfig>();
        let expander_circuit = ecc_circuit.export_to_expander().flatten::<C>();
        mpi_config.consume_obj_and_create_shared(Some(expander_circuit))
    } else {
        mpi_config.consume_obj_and_create_shared(None)
    };
    expander_circuit.pre_process_gkr::<C>();
    let partition_info = read_partition_info_from_shared_memory();
    let broadcast_info = read_broadcast_info_from_shared_memory();
    timer.stop();

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
    expander_circuit.discard_control_of_shared_mem();
    mpi_config.free_shared_mem(&mut window);
    MPIConfig::finalize();
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

fn main() {
    let expander_exec_args = ExpanderExecArgs::parse();
    assert_eq!(
        expander_exec_args.fiat_shamir_hash, "SHA256",
        "Only SHA256 is supported for now"
    );

    let pcs_type = PolynomialCommitmentType::from_str(&expander_exec_args.poly_commit).unwrap();

    match (expander_exec_args.field_type.as_str(), pcs_type) {
        ("M31", PolynomialCommitmentType::Raw) => {
            prove::<M31Config, M31Config>();
        }
        ("GF2", PolynomialCommitmentType::Raw) => {
            prove::<GF2Config, GF2Config>();
        }
        ("Goldilocks", PolynomialCommitmentType::Raw) => {
            prove::<GoldilocksConfig, GoldilocksConfig>();
        }
        ("BabyBear", PolynomialCommitmentType::Raw) => {
            prove::<BabyBearConfig, BabyBearConfig>();
        }
        ("BN254", PolynomialCommitmentType::Raw) => {
            prove::<BN254Config, BN254Config>();
        }
        ("BN254", PolynomialCommitmentType::Hyrax) => {
            prove::<BN254ConfigSha2Hyrax, BN254Config>();
        }
        ("BN254", PolynomialCommitmentType::KZG) => {
            prove::<BN254ConfigSha2KZG, BN254Config>();
        }
        (field_type, pcs_type) => panic!(
            "Combination of {:?} and {:?} not supported",
            field_type, pcs_type
        ),
    }
}

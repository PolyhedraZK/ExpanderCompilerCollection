mod common;
use common::ExpanderExecArgs;

use clap::Parser;
use std::cmp::max;

use arith::Field;
use expander_compiler::circuit::config::Config;
use expander_compiler::frontend::{
    BN254Config, BabyBearConfig, GF2Config, GoldilocksConfig, M31Config, SIMDField,
};
use expander_compiler::zkcuda::proving_system::callee_utils::{
    read_broadcast_info_from_shared_memory, read_commitment_extra_info_from_shared_memory,
    read_commitment_from_shared_memory, read_commitment_values_from_shared_memory,
    read_ecc_circuit_from_shared_memory, read_partition_info_from_shared_memory,
    read_pcs_setup_from_shared_memory, write_proof_to_shared_memory,
};
use expander_compiler::zkcuda::proving_system::{
    max_n_vars, prepare_inputs, ExpanderGKRCommitmentExtraInfo, ExpanderGKRProof,
    ExpanderGKRProverSetup,
};
use expander_utils::timer::Timer;

use gkr::gkr_prove;
use gkr_engine::{
    ExpanderPCS, ExpanderSingleVarChallenge, FieldEngine, MPIConfig, MPIEngine, Transcript,
};
use polynomials::MultiLinearPoly;
use serdes::ExpSerde;
use sumcheck::ProverScratchPad;

macro_rules! field {
    ($config: ident) => {
        $config::FieldConfig
    };
}

macro_rules! transcript {
    ($config: ident) => {
        $config::TranscriptConfig
    };
}

macro_rules! pcs {
    ($config: ident) => {
        $config::PCSConfig
    };
}

fn prove<C: Config>() {
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

    let timer = Timer::new("reading circuit", mpi_config.is_root());
    let pcs_setup = read_pcs_setup_from_shared_memory::<C>();
    let ecc_circuit = read_ecc_circuit_from_shared_memory::<C>();
    let partition_info = read_partition_info_from_shared_memory();

    let _commitments = read_commitment_from_shared_memory::<C>();
    let commitments_extra_info = read_commitment_extra_info_from_shared_memory::<C>();
    let commitment_values = read_commitment_values_from_shared_memory::<C>();
    let commitment_values_refs = commitment_values
        .iter()
        .map(|commitment| &commitment[..])
        .collect::<Vec<_>>();
    let broadcast_info = read_broadcast_info_from_shared_memory();
    timer.stop();

    let timer = Timer::new("gkr prove", mpi_config.is_root());
    let mut expander_circuit = ecc_circuit.export_to_expander().flatten::<C>();
    expander_circuit.pre_process_gkr::<C>();
    let (max_num_input_var, max_num_output_var) = max_n_vars(&expander_circuit);
    let max_num_var = max(max_num_input_var, max_num_output_var); // temp fix to a bug in Expander, remove this after Expander update.
    let mut prover_scratch =
        ProverScratchPad::<C::FieldConfig>::new(max_num_var, max_num_var, world_size);

    let mut transcript = C::TranscriptConfig::new();
    transcript.append_u8_slice(&[0u8; 32]); // TODO: Replace with the commitment, and hash an additional a few times
    expander_circuit.layers[0].input_vals = prepare_inputs(
        &ecc_circuit,
        &partition_info,
        &commitment_values_refs,
        &broadcast_info,
        world_rank,
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
    prove_input_claim(
        &mpi_config,
        &commitment_values_refs,
        &pcs_setup,
        &commitments_extra_info,
        &challenge.challenge_x(),
        &broadcast_info,
        &mut transcript,
    );
    if challenge.rz_1.is_some() {
        prove_input_claim(
            &mpi_config,
            &commitment_values_refs,
            &pcs_setup,
            &commitments_extra_info,
            &challenge.challenge_y(),
            &broadcast_info,
            &mut transcript,
        );
    }
    timer.stop();

    let proof = transcript.finalize_and_get_proof();
    if world_rank == 0 {
        write_proof_to_shared_memory(&ExpanderGKRProof { data: vec![proof] });
    }
    MPIConfig::finalize();
}

#[allow(clippy::too_many_arguments)]
fn prove_input_claim<C: Config>(
    mpi_config: &MPIConfig,
    commitments_values: &[&[SIMDField<C>]],
    p_keys: &ExpanderGKRProverSetup<C>,
    commitments_extra_info: &[ExpanderGKRCommitmentExtraInfo<C>],
    challenge: &ExpanderSingleVarChallenge<C::FieldConfig>,
    is_broadcast: &[bool],
    transcript: &mut transcript!(C),
) {
    let parallel_count = mpi_config.world_size();
    let parallel_index = mpi_config.world_rank();

    for ((commitment_val, extra_info), ib) in commitments_values
        .iter()
        .zip(commitments_extra_info)
        .zip(is_broadcast)
    {
        let val_len = if *ib {
            commitment_val.len()
        } else {
            commitment_val.len() / parallel_count
        };
        let vals_to_open = if *ib {
            *commitment_val
        } else {
            &commitment_val[val_len * parallel_index..val_len * (parallel_index + 1)]
        };

        let nb_challenge_vars = val_len.ilog2() as usize;
        let challenge_vars = challenge.rz[..nb_challenge_vars].to_vec();

        let params = <pcs!(C) as ExpanderPCS<field!(C)>>::gen_params(val_len);
        let p_key = p_keys.p_keys.get(&val_len).unwrap();

        let poly = MultiLinearPoly::new(vals_to_open.to_vec());
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
        let opening = <pcs!(C) as ExpanderPCS<field!(C)>>::open(
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
        )
        .unwrap();
        transcript.unlock_proof();

        let mut buffer = vec![];
        opening
            .serialize_into(&mut buffer)
            .expect("Failed to serialize opening");
        transcript.append_u8_slice(&buffer);
    }
}

fn main() {
    let expander_exec_args = ExpanderExecArgs::parse();
    match expander_exec_args.field_type.as_str() {
        "M31" => prove::<M31Config>(),
        "GF2" => prove::<GF2Config>(),
        "Goldilocks" => prove::<GoldilocksConfig>(),
        "BabyBear" => prove::<BabyBearConfig>(),
        "BN254" => prove::<BN254Config>(),
        _ => panic!("Unsupported field type"),
    }
}

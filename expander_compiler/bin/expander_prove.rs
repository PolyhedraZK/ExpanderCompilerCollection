use std::cmp;

use arith::Field;
use expander_compiler::circuit::config::Config;
use expander_compiler::frontend::M31Config;
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

use expander_config::GKRConfig;
use expander_transcript::Transcript;
use gkr::gkr_prove;
use gkr_field_config::GKRFieldConfig;
use mpi_config::MPIConfig;
use poly_commit::{ExpanderGKRChallenge, PCSForExpanderGKR};
use polynomials::{MultiLinearPoly, MultiLinearPolyExpander};
use serdes::ExpSerde;
use sumcheck::ProverScratchPad;

macro_rules! field {
    ($config: ident) => {
        $config::DefaultGKRFieldConfig
    };
}

macro_rules! transcript {
    ($config: ident) => {
        <$config::DefaultGKRConfig as GKRConfig>::Transcript
    };
}

macro_rules! pcs {
    ($config: ident) => {
        <$config::DefaultGKRConfig as GKRConfig>::PCS
    };
}

fn prove<C: Config>() {
    let mpi_config = MPIConfig::new();
    let world_rank = mpi_config.world_rank();
    let world_size = mpi_config.world_size();
    assert!(
        world_size > 1,
        "In case world_size is 1, we should not use the mpi version of the prover"
    );
    if world_rank == 0 {
        println!("Expander Prove Exec Called with world size {}", world_size);
    }

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

    let mut expander_circuit = ecc_circuit
        .export_to_expander()
        .flatten::<C::DefaultGKRConfig>();
    expander_circuit.pre_process_gkr::<C::DefaultGKRConfig>();
    let (max_num_input_var, max_num_output_var) = max_n_vars(&expander_circuit);
    let mut prover_scratch = ProverScratchPad::<C::DefaultGKRFieldConfig>::new(
        max_num_input_var,
        max_num_output_var,
        world_size,
    );

    let mut transcript = <C::DefaultGKRConfig as GKRConfig>::Transcript::new();
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
    let (claimed_v, rx, ry, rsimd, rmpi) = gkr_prove(
        &expander_circuit,
        &mut prover_scratch,
        &mut transcript,
        &mpi_config,
    );
    assert_eq!(
        claimed_v,
        <C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField::from(0)
    );

    prove_input_claim(
        &mpi_config,
        &commitment_values_refs,
        &pcs_setup,
        &commitments_extra_info,
        &rx,
        &rsimd,
        &rmpi,
        &broadcast_info,
        &mut transcript,
    );
    if let Some(ry) = ry {
        prove_input_claim(
            &mpi_config,
            &commitment_values_refs,
            &pcs_setup,
            &commitments_extra_info,
            &ry,
            &rsimd,
            &rmpi,
            &broadcast_info,
            &mut transcript,
        );
    }

    let proof = transcript.finalize_and_get_proof();
    if world_rank == 0 {
        println!("proof len {}", proof.bytes.len());
        write_proof_to_shared_memory(&ExpanderGKRProof { data: vec![proof] });
    }
    MPIConfig::finalize();
}

#[allow(clippy::too_many_arguments)]
fn prove_input_claim<C: Config>(
    mpi_config: &MPIConfig,
    commitments_values: &[&[C::DefaultSimdField]],
    p_keys: &ExpanderGKRProverSetup<C>,
    commitments_extra_info: &[ExpanderGKRCommitmentExtraInfo<C>],
    x: &[<field!(C) as GKRFieldConfig>::ChallengeField],
    x_simd: &[<field!(C) as GKRFieldConfig>::ChallengeField],
    x_mpi: &[<field!(C) as GKRFieldConfig>::ChallengeField],
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
        let challenge_vars = x[..nb_challenge_vars].to_vec();

        let params = <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::gen_params(val_len);
        let p_key = p_keys.p_keys.get(&val_len).unwrap();

        let poly = MultiLinearPoly::new(vals_to_open.to_vec());
        let v = MultiLinearPolyExpander::<field!(C)>::collectively_eval_circuit_vals_at_expander_challenge(
            vals_to_open,
            &challenge_vars,
            x_simd,
            x_mpi,
            &mut vec![<C::DefaultGKRFieldConfig as GKRFieldConfig>::Field::ZERO; val_len],
            &mut vec![<C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField::ZERO; 1 << cmp::max(x_simd.len(), x_mpi.len())],
            mpi_config,
        );
        transcript.append_field_element(&v);

        transcript.lock_proof();
        let opening = <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::open(
            &params,
            mpi_config,
            p_key,
            &poly,
            &ExpanderGKRChallenge::<C::DefaultGKRFieldConfig> {
                x: challenge_vars.to_vec(),
                x_simd: x_simd.to_vec(),
                x_mpi: x_mpi.to_vec(),
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
    // TODO: Add command line argument parsing
    prove::<M31Config>();
}

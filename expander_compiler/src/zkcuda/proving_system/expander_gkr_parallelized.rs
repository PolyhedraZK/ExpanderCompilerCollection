use std::collections::HashMap;
use std::io::{Cursor, Read};

use crate::circuit::config::Config;

use super::super::kernel::Kernel;
use super::{check_inputs, pcs_testing_setup_fixed_seed, prepare_inputs, Commitment, ExpanderGKRProvingSystem, Proof, ProvingSystem};
use super::expander_gkr::{ExpanderGKRCommitment, ExpanderGKRCommitmentExtraInfo, ExpanderGKRProof, ExpanderGKRProverSetup, ExpanderGKRVerifierSetup};
use super::shared_mem::*;

use arith::Field;
use chrono::format;
use expander_circuit::Circuit;
use expander_config::GKRConfig;
use expander_transcript::{Proof as ExpanderProof, Transcript};
use gkr::{gkr_prove, gkr_verify};
use gkr_field_config::GKRFieldConfig;
use mpi_config::MPIConfig;
use poly_commit::{
    expander_pcs_init_testing_only, ExpanderGKRChallenge, PCSForExpanderGKR,
    StructuredReferenceString,
};
use polynomials::{EqPolynomial, MultiLinearPoly, MultiLinearPolyExpander};
use serdes::ExpSerde;
use sumcheck::ProverScratchPad;
use shared_memory::{Shmem, ShmemConf};
use std::process::Command;

use rand::rngs::StdRng;
use rand::SeedableRng;

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


pub struct ParallelizedExpanderGKRProvingSystem<C: Config> {
    _config: std::marker::PhantomData<C>,
}

impl<C: Config> ProvingSystem<C> for ParallelizedExpanderGKRProvingSystem<C> {
    type ProverSetup = ExpanderGKRProverSetup<C>;
    type VerifierSetup = ExpanderGKRVerifierSetup<C>;
    type Proof = ExpanderGKRProof;
    type Commitment = ExpanderGKRCommitment<C>;
    type CommitmentExtraInfo = ExpanderGKRCommitmentExtraInfo<C>;

    fn setup(
        computation_graph: &crate::zkcuda::proof::ComputationGraph<C>,
    ) -> (Self::ProverSetup, Self::VerifierSetup) {
        // All of currently supported PCSs(Raw, Orion, Hyrax) do not require the multi-core information in the step of `setup`
        // So we can simply reuse the setup function from the non-parallelized version
        // TODO: Consider how to do this properly in supporting future mpi-info-awared PCSs
        ExpanderGKRProvingSystem::<C>::setup(computation_graph)
    }

    fn commit(
        prover_setup: &Self::ProverSetup,
        vals: &[C::DefaultSimdField],
        parallel_count: usize,
        is_broadcast: bool,
    ) -> (Self::Commitment, Self::CommitmentExtraInfo) {
        if is_broadcast || parallel_count == 1 {
            ExpanderGKRProvingSystem::<C>::commit(
                prover_setup,
                vals,
                parallel_count,
                is_broadcast,
            )
        } else {
            let actual_local_len = if is_broadcast {
                vals.len()
            } else {
                vals.len() / parallel_count
            };

            // TODO: The size here is for the raw commitment, add an function in the pcs trait to get the size of the commitment
            init_commitment_and_extra_info_shared_memory::<C>(unsafe {SHARED_MEMORY.input_vals.as_ref().unwrap().len()}, 1);
            write_pcs_setup_to_shared_memory(prover_setup, actual_local_len);
            write_commit_vals_to_shared_memory::<C>(&vals.to_vec());
            exec_pcs_commit(parallel_count);
            read_commitment_and_extra_info_from_shared_memory()
        }
    }

    fn prove(
        prover_setup: &Self::ProverSetup,
        kernel: &Kernel<C>,
        commitments: &[Self::Commitment],
        commitments_extra_info: &[Self::CommitmentExtraInfo],
        commitments_values: &[&[<C as Config>::DefaultSimdField]],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> Self::Proof {
        if parallel_count == 1 {
            ExpanderGKRProvingSystem::<C>::prove(
                prover_setup,
                kernel,
                commitments,
                commitments_extra_info,
                commitments_values,
                parallel_count,
                is_broadcast,
            )
        } else {
            let expander_circuit = kernel
                .layered_circuit
                .export_to_expander()
                .flatten::<C::DefaultGKRConfig>();
        
            write_circuit_to_shared_memory::<C>(&expander_circuit);
            let inputs = (0..parallel_count)
                .map(|i| {
                    prepare_inputs(
                        kernel,
                        commitments_values,
                        is_broadcast,
                        i,
                    )
                })
                .collect::<Vec<_>>();
            write_proving_inputs_to_shared_memory::<C>(&inputs);
            exec_gkr_prove_with_pcs(parallel_count);
            read_proof_from_shared_memory::<C>()
        }
    }

    // For verification, we don't need the mpi executor and shared memory, it's always run by a single party
    fn verify(
        verifier_setup: &Self::VerifierSetup,
        kernel: &Kernel<C>,
        proof: &Self::Proof,
        commitments: &[Self::Commitment],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> bool {
        let mut expander_circuit = kernel
            .layered_circuit
            .export_to_expander()
            .flatten::<C::DefaultGKRConfig>();
        expander_circuit.pre_process_gkr::<C::DefaultGKRConfig>();

        let mut transcript = <C::DefaultGKRConfig as GKRConfig>::Transcript::new();
        transcript.append_u8_slice(&[0u8; 32]);
        expander_circuit.fill_rnd_coefs(&mut transcript);
        let mut cursor = Cursor::new(&proof.data[0].bytes);
        cursor.set_position(32);

        let (mut verified, rz0, rz1, r_simd, r_mpi, claimed_v0, claimed_v1) = gkr_verify(
            &MPIConfig::default(),
            &expander_circuit,
            &[],
            &<C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField::ZERO,
            &mut transcript,
            &mut cursor,
        );

        verified &= verify_input_claim(
            &mut cursor,
            kernel,
            verifier_setup,
            &rz0,
            &r_simd,
            &r_mpi,
            &claimed_v0,
            commitments,
            is_broadcast,
            parallel_count,
            &mut transcript,
        );
        if let Some(rz1) = rz1 {
            verified &= verify_input_claim(
                &mut cursor,
                kernel,
                verifier_setup,
                &rz0,
                &r_simd,
                &r_mpi,
                &claimed_v1.unwrap(),
                commitments,
                is_broadcast,
                parallel_count,
                &mut transcript,
            );
        }

        true
    }
}

#[allow(clippy::too_many_arguments)]
fn verify_input_claim<C: Config>(
    mut proof_reader: impl Read,
    kernel: &Kernel<C>,
    v_keys: &ExpanderGKRVerifierSetup<C>,
    x: &[<C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField],
    x_simd: &[<C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField],
    x_mpi: &[<C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField],
    y: &<C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField,
    commitments: &[ExpanderGKRCommitment<C>],
    is_broadcast: &[bool],
    parallel_count: usize,
    transcript: &mut transcript!(C),
) -> bool {
    let mut target_y = <C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField::ZERO;
    for ((input, commitment), ib) in kernel
        .layered_circuit_input
        .iter()
        .zip(commitments)
        .zip(is_broadcast)
    {
        let commitment_len = commitment.vals_len();
        let nb_challenge_vars = commitment_len.ilog2() as usize;
        let challenge_vars = x[..nb_challenge_vars].to_vec();

        let params = <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::gen_params(
            commitment_len.ilog2() as usize,
        );
        let v_key = v_keys.v_keys.get(&commitment_len).unwrap();

        let claim =
            <field!(C) as GKRFieldConfig>::ChallengeField::deserialize_from(&mut proof_reader)
                .unwrap();
        transcript.append_field_element(&claim);

        let opening =
            <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::Opening::deserialize_from(
                &mut proof_reader,
            )
            .unwrap();

        transcript.lock_proof();
        // individual pcs verification
        let verified = <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::verify(
            &params,
            v_key,
            &commitment.commitment[0],
            &ExpanderGKRChallenge::<C::DefaultGKRFieldConfig> {
                x: challenge_vars,
                x_simd: x_simd.to_vec(),
                x_mpi: x_mpi.to_vec(),
            },
            claim,
            transcript,
            &opening,
        );
        transcript.unlock_proof();

        if !verified {
            return false;
        }

        let index_vars = &x[nb_challenge_vars..];
        let index = input.offset / input.len;
        let index_as_bits = (0..index_vars.len())
            .map(|i| {
                <C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField::from(
                    ((index >> i) & 1) as u32,
                )
            })
            .collect::<Vec<_>>();
        let v_index =
            EqPolynomial::<<C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField>::eq_vec(
                index_vars,
                &index_as_bits,
            );

        target_y += v_index * claim;
    }

    // overall claim verification
    *y == target_y
}

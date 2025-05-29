use std::io::Read;
use std::{fs, thread};

use crate::circuit::config::Config;
use crate::frontend::SIMDField;

use super::super::kernel::Kernel;
use super::callee_utils::write_proof_to_shared_memory;
use super::caller_utils::{
    init_commitment_and_extra_info_shared_memory, init_proof_shared_memory, read_commitment_and_extra_info_from_shared_memory, read_proof_from_shared_memory, start_server, write_broadcast_info_to_shared_memory, write_commit_vals_to_shared_memory, write_commitments_extra_info_to_shared_memory, write_commitments_to_shared_memory, write_commitments_values_to_shared_memory, write_ecc_circuit_to_shared_memory, write_input_partition_info_to_shared_memory, write_pcs_setup_to_shared_memory, write_selected_pkey_to_shared_memory
};
use super::client::{request_commit_input, request_prove, request_verify};
use super::expander_gkr::{
    ExpanderGKRCommitment, ExpanderGKRCommitmentExtraInfo, ExpanderGKRProof,
    ExpanderGKRProverSetup, ExpanderGKRVerifierSetup,
};
use super::server::SERVER_URL;
use super::{Commitment, ProvingSystem};
use expander_utils::timer::Timer;

use arith::Field;
use gkr_engine::{ExpanderPCS, ExpanderSingleVarChallenge, FieldEngine, GKREngine, Transcript};
use reqwest::Client;
use serdes::ExpSerde;

use polynomials::EqPolynomial;

const SINGLE_KERNEL_MAX_PROOF_SIZE: usize = 10 * 1024 * 1024; // 10MB

pub struct ParallelizedExpanderGKRProvingSystem<C: GKREngine> {
    _config: std::marker::PhantomData<C>,
}

impl<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> ProvingSystem<ECCConfig>
    for ParallelizedExpanderGKRProvingSystem<C>
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    type ProverSetup = ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>;
    type VerifierSetup = ExpanderGKRVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>;
    type Proof = ExpanderGKRProof;
    type Commitment = ExpanderGKRCommitment<C::PCSField, C::FieldConfig, C::PCSConfig>;
    type CommitmentExtraInfo =
        ExpanderGKRCommitmentExtraInfo<C::PCSField, C::FieldConfig, C::PCSConfig>;

    fn setup(
        computation_graph: &crate::zkcuda::proof::ComputationGraph<ECCConfig>,
    ) -> (Self::ProverSetup, Self::VerifierSetup) {
        let mut bytes = vec![];
        computation_graph.serialize_into(&mut bytes).unwrap();
        fs::write(
            "/tmp/computation_graph.bin",
            bytes,
        )
        .expect("Failed to write computation graph to file");

        let max_parallel_count = computation_graph.proof_templates.iter().map(|t| t.parallel_count).max().unwrap_or(1);
        start_server(max_parallel_count);
        thread::sleep(std::time::Duration::from_secs(1)); // Give the server some time to start

        // We're delegating all the computation to the service, so we don't need to do anything here
        (
            Self::ProverSetup::default(),
            Self::VerifierSetup::default(),
        )
    }

    fn commit(
        _prover_setup: &Self::ProverSetup,
        vals: &[SIMDField<C>],
        parallel_count: usize,
        is_broadcast: bool,
    ) -> (Self::Commitment, Self::CommitmentExtraInfo) {
        // TODO: The size here is for the raw commitment, add an function in the pcs trait to get the size of the commitment
        init_commitment_and_extra_info_shared_memory(SINGLE_KERNEL_MAX_PROOF_SIZE, 8);
        write_commit_vals_to_shared_memory::<ECCConfig>(vals);
        
        let client = Client::new(); 
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(request_commit_input(&client, SERVER_URL, if is_broadcast { 1 } else { parallel_count }));
        let (commitment, extra_info) = read_commitment_and_extra_info_from_shared_memory();
        
        (commitment, extra_info)
    }

    fn prove(
        _prover_setup: &Self::ProverSetup,
        kernel_id: usize,
        kernel: &Kernel<ECCConfig>,
        commitments: &[Self::Commitment],
        commitments_extra_info: &[Self::CommitmentExtraInfo],
        commitments_values: &[&[SIMDField<C>]],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> Self::Proof {
        let timer = Timer::new("prove", true);
        init_proof_shared_memory(SINGLE_KERNEL_MAX_PROOF_SIZE);
        write_input_partition_info_to_shared_memory(&kernel.layered_circuit_input);
        write_commitments_to_shared_memory(&commitments.to_vec());
        write_commitments_extra_info_to_shared_memory(&commitments_extra_info.to_vec());
        write_commitments_values_to_shared_memory::<C::FieldConfig>(commitments_values);
        write_broadcast_info_to_shared_memory(&is_broadcast.to_vec());

        let client = Client::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(request_prove(&client, SERVER_URL, parallel_count, kernel_id));

        timer.stop();
        read_proof_from_shared_memory()
    }

    // For verification, we don't need the mpi executor and shared memory, it's always run by a single party
    fn verify(
        _verifier_setup: &Self::VerifierSetup,
        kernel_id: usize,
        _kernel: &Kernel<ECCConfig>,
        proof: &Self::Proof,
        commitments: &[Self::Commitment],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> bool {
        write_proof_to_shared_memory(proof);
        write_commitments_to_shared_memory(&commitments.to_vec());
        write_broadcast_info_to_shared_memory(&is_broadcast.to_vec());

        let client = Client::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let verified = rt.block_on(request_verify(
            &client,
            SERVER_URL,
            parallel_count,
            kernel_id,
        ));
        verified
    }
}

#[allow(clippy::too_many_arguments)]
fn verify_input_claim<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    mut proof_reader: impl Read,
    kernel: &Kernel<ECCConfig>,
    v_keys: &ExpanderGKRVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    challenge: &ExpanderSingleVarChallenge<C::FieldConfig>,
    y: &<C::FieldConfig as FieldEngine>::ChallengeField,
    commitments: &[ExpanderGKRCommitment<C::PCSField, C::FieldConfig, C::PCSConfig>],
    is_broadcast: &[bool],
    parallel_count: usize,
    transcript: &mut C::TranscriptConfig,
) -> bool
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    assert_eq!(1 << challenge.r_mpi.len(), parallel_count);
    let mut target_y = <C::FieldConfig as FieldEngine>::ChallengeField::ZERO;
    for ((input, commitment), ib) in kernel
        .layered_circuit_input
        .iter()
        .zip(commitments)
        .zip(is_broadcast)
    {
        let local_vals_len =
            <ExpanderGKRCommitment<C::PCSField, C::FieldConfig, C::PCSConfig> as Commitment<
                ECCConfig,
            >>::vals_len(commitment);
        let nb_challenge_vars = local_vals_len.ilog2() as usize;
        let challenge_vars = challenge.rz[..nb_challenge_vars].to_vec();

        let params = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(
            nb_challenge_vars,
            parallel_count,
        );
        let v_key = v_keys.v_keys.get(&local_vals_len).unwrap();

        let claim =
            <C::FieldConfig as FieldEngine>::ChallengeField::deserialize_from(&mut proof_reader)
                .unwrap();
        transcript.append_field_element(&claim);

        let opening =
            <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::Opening::deserialize_from(
                &mut proof_reader,
            )
            .unwrap();

        transcript.lock_proof();
        // individual pcs verification
        let verified = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::verify(
            &params,
            v_key,
            &commitment.commitment[0],
            &ExpanderSingleVarChallenge::<C::FieldConfig> {
                rz: challenge_vars.to_vec(),
                r_simd: challenge.r_simd.to_vec(),
                r_mpi: if *ib {
                    vec![]
                } else {
                    challenge.r_mpi.to_vec()
                }, // In the case of broadcast, whatever x_mpi is, the opening is the same
            },
            claim,
            transcript,
            &opening,
        );
        transcript.unlock_proof();

        if !verified {
            return false;
        }

        let index_vars = &challenge.rz[nb_challenge_vars..];
        let index = input.offset / input.len;
        let index_as_bits = (0..index_vars.len())
            .map(|i| {
                <C::FieldConfig as FieldEngine>::ChallengeField::from(((index >> i) & 1) as u32)
            })
            .collect::<Vec<_>>();
        let v_index = EqPolynomial::<<C::FieldConfig as FieldEngine>::ChallengeField>::eq_vec(
            index_vars,
            &index_as_bits,
        );

        target_y += v_index * claim;
    }

    // overall claim verification
    *y == target_y
}

use std::io::{Cursor, Read};

use crate::circuit::config::Config;
use crate::frontend::SIMDField;

use super::super::kernel::Kernel;
use super::caller_utils::{
    exec_gkr_prove_with_pcs, exec_pcs_commit, init_commitment_and_extra_info_shared_memory,
    init_proof_shared_memory, read_commitment_and_extra_info_from_shared_memory,
    read_proof_from_shared_memory, write_broadcast_info_to_shared_memory,
    write_commit_vals_to_shared_memory, write_commitments_extra_info_to_shared_memory,
    write_commitments_to_shared_memory, write_commitments_values_to_shared_memory,
    write_ecc_circuit_to_shared_memory, write_input_partition_info_to_shared_memory,
    write_pcs_setup_to_shared_memory, write_selected_pkey_to_shared_memory,
};
use super::expander_gkr::{
    ExpanderGKRCommitment, ExpanderGKRCommitmentExtraInfo, ExpanderGKRProof,
    ExpanderGKRProverSetup, ExpanderGKRVerifierSetup,
};
use super::{Commitment, ExpanderGKRProvingSystem, ProvingSystem};
use expander_utils::timer::Timer;

use arith::Field;
use gkr::gkr_verify;
use gkr_engine::{ExpanderPCS, ExpanderSingleVarChallenge, FieldEngine, GKREngine, Transcript};
use serdes::ExpSerde;

use polynomials::EqPolynomial;

const SINGLE_KERNEL_MAX_PROOF_SIZE: usize = 10 * 1024 * 1024; // 10MB

pub struct ParallelizedExpanderGKRProvingSystem<C: GKREngine> {
    _config: std::marker::PhantomData<C>,
}

impl<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> ProvingSystem<ECCConfig>
    for ParallelizedExpanderGKRProvingSystem<C>
{
    type ProverSetup = ExpanderGKRProverSetup<C::FieldConfig, C::PCSConfig>;
    type VerifierSetup = ExpanderGKRVerifierSetup<C::FieldConfig, C::PCSConfig>;
    type Proof = ExpanderGKRProof;
    type Commitment = ExpanderGKRCommitment<C::FieldConfig, C::PCSConfig>;
    type CommitmentExtraInfo = ExpanderGKRCommitmentExtraInfo<C::FieldConfig, C::PCSConfig>;

    fn setup(
        computation_graph: &crate::zkcuda::proof::ComputationGraph<ECCConfig>,
    ) -> (Self::ProverSetup, Self::VerifierSetup) {
        // All of currently supported PCSs(Raw, Orion, Hyrax) do not require the multi-core information in the step of `setup`
        // So we can simply reuse the setup function from the non-parallelized version
        // TODO: Do this properly in supporting future mpi-info-awared PCSs
        ExpanderGKRProvingSystem::<C>::setup(computation_graph)
    }

    fn commit(
        prover_setup: &Self::ProverSetup,
        vals: &[SIMDField<C>],
        parallel_count: usize,
        is_broadcast: bool,
    ) -> (Self::Commitment, Self::CommitmentExtraInfo) {
        if is_broadcast || parallel_count == 1 {
            <ExpanderGKRProvingSystem<C> as ProvingSystem<ECCConfig>>::commit(
                prover_setup,
                vals,
                parallel_count,
                is_broadcast,
            )
        } else {
            let timer = Timer::new("commit", true);
            let actual_local_len = vals.len() / parallel_count;

            // TODO: The size here is for the raw commitment, add an function in the pcs trait to get the size of the commitment
            init_commitment_and_extra_info_shared_memory(SINGLE_KERNEL_MAX_PROOF_SIZE, 8);
            write_selected_pkey_to_shared_memory(prover_setup, actual_local_len);
            write_commit_vals_to_shared_memory::<ECCConfig>(&vals.to_vec());
            exec_pcs_commit::<C>(parallel_count);
            let (commitment, extra_info) = read_commitment_and_extra_info_from_shared_memory();
            timer.stop();
            (commitment, extra_info)
        }
    }

    fn prove(
        prover_setup: &Self::ProverSetup,
        kernel: &Kernel<ECCConfig>,
        commitments: &[Self::Commitment],
        commitments_extra_info: &[Self::CommitmentExtraInfo],
        commitments_values: &[&[SIMDField<C>]],
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
            let timer = Timer::new("prove", true);
            init_proof_shared_memory(SINGLE_KERNEL_MAX_PROOF_SIZE);
            write_pcs_setup_to_shared_memory(prover_setup);
            write_ecc_circuit_to_shared_memory(&kernel.layered_circuit);
            write_input_partition_info_to_shared_memory(&kernel.layered_circuit_input);
            write_commitments_to_shared_memory(&commitments.to_vec());
            write_commitments_extra_info_to_shared_memory(&commitments_extra_info.to_vec());
            write_commitments_values_to_shared_memory::<C::FieldConfig>(commitments_values);
            write_broadcast_info_to_shared_memory(&is_broadcast.to_vec());
            exec_gkr_prove_with_pcs::<C>(parallel_count);
            timer.stop();
            read_proof_from_shared_memory()
        }
    }

    // For verification, we don't need the mpi executor and shared memory, it's always run by a single party
    fn verify(
        verifier_setup: &Self::VerifierSetup,
        kernel: &Kernel<ECCConfig>,
        proof: &Self::Proof,
        commitments: &[Self::Commitment],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> bool {
        let timer = Timer::new("verify", true);

        let mut expander_circuit = kernel.layered_circuit.export_to_expander().flatten::<C>();
        expander_circuit.pre_process_gkr::<C>();

        let mut transcript = C::TranscriptConfig::new();
        transcript.append_u8_slice(&[0u8; 32]);
        expander_circuit.fill_rnd_coefs(&mut transcript);
        let mut cursor = Cursor::new(&proof.data[0].bytes);
        cursor.set_position(32);

        let (mut verified, challenge, claimed_v0, claimed_v1) = gkr_verify(
            parallel_count,
            &expander_circuit,
            &[],
            &<C::FieldConfig as FieldEngine>::ChallengeField::ZERO,
            &mut transcript,
            &mut cursor,
        );

        let pcs_verification_timer = Timer::new("pcs verification", true);
        verified &= verify_input_claim::<C, ECCConfig>(
            &mut cursor,
            kernel,
            verifier_setup,
            &challenge.challenge_x(),
            &claimed_v0,
            commitments,
            is_broadcast,
            parallel_count,
            &mut transcript,
        );
        if let Some(challenge_y) = challenge.challenge_y() {
            verified &= verify_input_claim::<C, ECCConfig>(
                &mut cursor,
                kernel,
                verifier_setup,
                &challenge_y,
                &claimed_v1.unwrap(),
                commitments,
                is_broadcast,
                parallel_count,
                &mut transcript,
            );
        }
        pcs_verification_timer.stop();

        timer.stop();
        verified
    }
}

#[allow(clippy::too_many_arguments)]
fn verify_input_claim<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    mut proof_reader: impl Read,
    kernel: &Kernel<ECCConfig>,
    v_keys: &ExpanderGKRVerifierSetup<C::FieldConfig, C::PCSConfig>,
    challenge: &ExpanderSingleVarChallenge<C::FieldConfig>,
    y: &<C::FieldConfig as FieldEngine>::ChallengeField,
    commitments: &[ExpanderGKRCommitment<C::FieldConfig, C::PCSConfig>],
    is_broadcast: &[bool],
    parallel_count: usize,
    transcript: &mut C::TranscriptConfig,
) -> bool {
    assert_eq!(1 << challenge.r_mpi.len(), parallel_count);
    let mut target_y = <C::FieldConfig as FieldEngine>::ChallengeField::ZERO;
    for ((input, commitment), ib) in kernel
        .layered_circuit_input
        .iter()
        .zip(commitments)
        .zip(is_broadcast)
    {
        let local_vals_len = <ExpanderGKRCommitment<C::FieldConfig, C::PCSConfig> as Commitment<
            ECCConfig,
        >>::vals_len(commitment);
        let nb_challenge_vars = local_vals_len.ilog2() as usize;
        let challenge_vars = challenge.rz[..nb_challenge_vars].to_vec();

        let params = <C::PCSConfig as ExpanderPCS<C::FieldConfig>>::gen_params(nb_challenge_vars);
        let v_key = v_keys.v_keys.get(&local_vals_len).unwrap();

        let claim =
            <C::FieldConfig as FieldEngine>::ChallengeField::deserialize_from(&mut proof_reader)
                .unwrap();
        transcript.append_field_element(&claim);

        let opening = <C::PCSConfig as ExpanderPCS<C::FieldConfig>>::Opening::deserialize_from(
            &mut proof_reader,
        )
        .unwrap();

        transcript.lock_proof();
        // individual pcs verification
        let verified = <C::PCSConfig as ExpanderPCS<C::FieldConfig>>::verify(
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

use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::collections::HashMap;
use std::io::{Cursor, Read};

use crate::circuit::config::Config;
use crate::frontend::SIMDField;
use crate::utils::misc::next_power_of_two;
use crate::zkcuda::context::DeviceMemory;
use crate::zkcuda::proof::ComputationGraph;
use crate::zkcuda::proving_system::setup_impl::local_setup_impl;
use crate::zkcuda::proving_system::{CombinedProof, KernelWiseProvingSystem, ProvingSystem, traits::Commitment, common::{check_inputs, prepare_inputs}};
use crate::zkcuda::kernel::Kernel;

use super::structs::*;
use super::utils::*;

use arith::Field;
use expander_circuit::Circuit;
use expander_utils::timer::Timer;
use gkr::{gkr_prove, gkr_verify};
use gkr_engine::{
    ExpanderPCS, ExpanderSingleVarChallenge, FieldEngine, GKREngine, MPIConfig,
    StructuredReferenceString, Transcript,
};
use poly_commit::expander_pcs_init_testing_only;
use polynomials::{EqPolynomial, RefMultiLinearPoly};
use serdes::ExpSerde;
use sumcheck::ProverScratchPad;

pub struct Expander<C: GKREngine> {
    _config: std::marker::PhantomData<C>,
}

impl<C, ECCConfig> KernelWiseProvingSystem<ECCConfig> for Expander<C>
where
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    type ProverSetup = ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>;
    type VerifierSetup = ExpanderVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>;
    type Proof = ExpanderProof;
    type Commitment = ExpanderCommitment<C::PCSField, C::FieldConfig, C::PCSConfig>;
    type CommitmentState =
        ExpanderCommitmentState<C::PCSField, C::FieldConfig, C::PCSConfig>;

    fn setup(
        computation_graph: &crate::zkcuda::proof::ComputationGraph<ECCConfig>,
    ) -> (Self::ProverSetup, Self::VerifierSetup) {
        local_setup_impl::<C, ECCConfig>(computation_graph)
    }

    fn commit(
        prover_setup: &Self::ProverSetup,
        vals: &[SIMDField<C>],
    ) -> (Self::Commitment, Self::CommitmentState) {
        let timer = Timer::new("commit", true);

        let n_vars = vals.len().ilog2() as usize;
        let params =
            <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(n_vars, 1);
        let p_key = prover_setup.p_keys.get(&vals.len()).unwrap();

        let mut scratch =
            <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::init_scratch_pad(
                &params,
                &MPIConfig::prover_new(None, None),
            );

        let commitment = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::commit(
            &params,
            &MPIConfig::prover_new(None, None),
            p_key,
            &RefMultiLinearPoly::from_ref(vals),
            &mut scratch,
        )
        .unwrap();

        timer.stop();
        (
            Self::Commitment {
                vals_len: vals.len(),
                commitment,
            },
            Self::CommitmentState { scratch },
        )
    }

    fn prove_kernel(
        prover_setup: &Self::ProverSetup,
        kernel: &Kernel<ECCConfig>,
        _commitments: &[&Self::Commitment],
        commitments_extra_info: &[&Self::CommitmentState],
        commitments_values: &[&[SIMDField<C>]],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> Self::Proof {
        let timer = Timer::new("prove", true);
        check_inputs(kernel, commitments_values, parallel_count, is_broadcast);

        let mut expander_circuit = kernel.layered_circuit.export_to_expander().flatten::<C>();
        expander_circuit.pre_process_gkr::<C>();
        let (max_num_input_var, max_num_output_var) = max_n_vars(&expander_circuit);
        let mut prover_scratch =
            ProverScratchPad::<C::FieldConfig>::new(max_num_input_var, max_num_output_var, 1);

        let mut proof = ExpanderProof { data: vec![] };

        // For each parallel index, generate the GKR proof
        for i in 0..parallel_count {
            let mut transcript = C::TranscriptConfig::new();
            transcript.append_u8_slice(&[0u8; 32]); // TODO: Replace with the commitment, and hash an additional a few times
            expander_circuit.layers[0].input_vals = prepare_inputs(
                &kernel.layered_circuit,
                &kernel.layered_circuit_input,
                commitments_values,
                is_broadcast,
                i,
            );
            expander_circuit.fill_rnd_coefs(&mut transcript);
            expander_circuit.evaluate();
            let (claimed_v, challenge) = gkr_prove(
                &expander_circuit,
                &mut prover_scratch,
                &mut transcript,
                &MPIConfig::prover_new(None, None),
            );
            assert_eq!(
                claimed_v,
                <C::FieldConfig as FieldEngine>::ChallengeField::from(0)
            );

            prove_input_claim::<C, ECCConfig>(
                kernel,
                commitments_values,
                prover_setup,
                commitments_extra_info,
                &challenge.challenge_x(),
                is_broadcast,
                i,
                parallel_count,
                &mut transcript,
            );
            if let Some(challenge_y) = challenge.challenge_y() {
                prove_input_claim::<C, ECCConfig>(
                    kernel,
                    commitments_values,
                    prover_setup,
                    commitments_extra_info,
                    &challenge_y,
                    is_broadcast,
                    i,
                    parallel_count,
                    &mut transcript,
                );
            }

            proof.data.push(transcript.finalize_and_get_proof());
        }

        timer.stop();
        proof
    }

    fn verify_kernel(
        verifier_setup: &Self::VerifierSetup,
        kernel: &Kernel<ECCConfig>,
        proof: &Self::Proof,
        commitments: &[&Self::Commitment],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> bool {
        let timer = Timer::new("verify", true);
        let mut expander_circuit = kernel.layered_circuit.export_to_expander().flatten::<C>();
        expander_circuit.pre_process_gkr::<C>();

        for i in 0..parallel_count {
            let mut transcript = C::TranscriptConfig::new();
            transcript.append_u8_slice(&[0u8; 32]);
            expander_circuit.fill_rnd_coefs(&mut transcript);

            let mut cursor = Cursor::new(&proof.data[i].bytes);
            cursor.set_position(32);
            let (mut verified, challenge, claimed_v0, claimed_v1) = gkr_verify(
                1,
                &expander_circuit,
                &[],
                &<C::FieldConfig as FieldEngine>::ChallengeField::ZERO,
                &mut transcript,
                &mut cursor,
            );

            if !verified {
                println!("Failed to verify GKR proof for parallel index {i}");
                return false;
            }

            verified &= verify_input_claim::<C, ECCConfig>(
                &mut cursor,
                kernel,
                verifier_setup,
                &challenge.challenge_x(),
                &claimed_v0,
                commitments,
                is_broadcast,
                i,
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
                    i,
                    parallel_count,
                    &mut transcript,
                );
            }

            if !verified {
                println!("Failed to verify overall pcs for parallel index {i}");
                return false;
            }
        }
        timer.stop();
        true
    }
}


/// Challenge ctructure:
/// llll pppp cccc ssss
/// Where:
///     l is the challenge for the local values
///     p is the challenge for the parallel index
///     c is the selector for the components
///     s is the challenge for the SIMD values
/// All little endian.
///
/// At the moment of commiting, we commited to the values corresponding to
///     llll pppp ssss
/// At the end of GKR, we will have the challenge
///     llll cccc ssss
/// The pppp part is not included because we're proving kernel-by-kernel.
///
/// Arguments:
/// - `challenge`: The gkr challenge: llll cccc ssss
/// - `total_vals_len`: The length of llll pppp
/// - `parallel_index`: The index of the parallel execution. pppp part.
/// - `parallel_count`: The total number of parallel executions. pppp part.
/// - `is_broadcast`: Whether the challenge is broadcasted or not.
///
/// Returns:
///     llll pppp ssss challenge
///     cccc
fn get_challenge_for_pcs<F: FieldEngine>(
    gkr_challenge: &ExpanderSingleVarChallenge<F>,
    total_vals_len: usize,
    parallel_index: usize,
    parallel_count: usize,
    is_broadcast: bool,
) -> (ExpanderSingleVarChallenge<F>, Vec<F::ChallengeField>) {
    let mut challenge = gkr_challenge.clone();
    let zero = F::ChallengeField::ZERO;
    if is_broadcast {
        let n_vals_vars = total_vals_len.ilog2() as usize;
        let component_idx_vars = challenge.rz[n_vals_vars..].to_vec();
        challenge.rz.resize(n_vals_vars, zero);
        (challenge, component_idx_vars)
    } else {
        let n_vals_vars = (total_vals_len / parallel_count).ilog2() as usize;
        let component_idx_vars = challenge.rz[n_vals_vars..].to_vec();
        challenge.rz.resize(n_vals_vars, zero);

        let n_index_vars = parallel_count.ilog2() as usize;
        let index_vars = (0..n_index_vars)
            .map(|i| F::ChallengeField::from(((parallel_index >> i) & 1) as u32))
            .collect::<Vec<_>>();

        challenge.rz.extend_from_slice(&index_vars);
        (challenge, component_idx_vars)
    }
}

#[allow(clippy::too_many_arguments)]
fn prove_input_claim<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    _kernel: &Kernel<ECCConfig>,
    commitments_values: &[&[SIMDField<C>]],
    p_keys: &ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    commitments_extra_info: &[&ExpanderCommitmentState<
        C::PCSField,
        C::FieldConfig,
        C::PCSConfig,
    >],
    challenge: &ExpanderSingleVarChallenge<C::FieldConfig>,
    is_broadcast: &[bool],
    parallel_index: usize,
    parallel_count: usize,
    transcript: &mut C::TranscriptConfig,
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    for ((commitment_val, extra_info), ib) in commitments_values
        .iter()
        .zip(commitments_extra_info)
        .zip(is_broadcast)
    {
        let val_len = commitment_val.len();
        let (challenge_for_pcs, _) =
            get_challenge_for_pcs(challenge, val_len, parallel_index, parallel_count, *ib);

        let params = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(
            val_len.ilog2() as usize,
            1,
        );
        let p_key = p_keys.p_keys.get(&val_len).unwrap();

        let poly = RefMultiLinearPoly::from_ref(commitment_val);
        let v =
            <C::FieldConfig as FieldEngine>::single_core_eval_circuit_vals_at_expander_challenge(
                commitment_val,
                &challenge_for_pcs,
            );
        transcript.append_field_element(&v);

        transcript.lock_proof();
        let opening = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::open(
            &params,
            &MPIConfig::prover_new(None, None),
            p_key,
            &poly,
            &challenge_for_pcs,
            transcript,
            &extra_info.scratch,
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

#[allow(clippy::too_many_arguments)]
fn verify_input_claim<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    mut proof_reader: impl Read,
    kernel: &Kernel<ECCConfig>,
    v_keys: &ExpanderVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    challenge: &ExpanderSingleVarChallenge<C::FieldConfig>,
    y: &<C::FieldConfig as FieldEngine>::ChallengeField,
    commitments: &[&ExpanderCommitment<C::PCSField, C::FieldConfig, C::PCSConfig>],
    is_broadcast: &[bool],
    parallel_index: usize,
    parallel_count: usize,
    transcript: &mut C::TranscriptConfig,
) -> bool
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let mut target_y = <C::FieldConfig as FieldEngine>::ChallengeField::ZERO;
    for ((input, commitment), ib) in kernel
        .layered_circuit_input
        .iter()
        .zip(commitments.iter())
        .zip(is_broadcast)
    {
        let val_len =
            <ExpanderCommitment<C::PCSField, C::FieldConfig, C::PCSConfig> as Commitment<
                ECCConfig,
            >>::vals_len(commitment);
        let (challenge_for_pcs, component_idx_vars) =
            get_challenge_for_pcs(challenge, val_len, parallel_index, parallel_count, *ib);

        let params = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(
            val_len.ilog2() as usize,
            1,
        );
        let v_key = v_keys.v_keys.get(&val_len).unwrap();

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
        let verified = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::verify(
            &params,
            v_key,
            &commitment.commitment,
            &challenge_for_pcs,
            claim,
            transcript,
            &opening,
        );
        transcript.unlock_proof();

        let mut buffer = vec![];
        opening
            .serialize_into(&mut buffer)
            .expect("Failed to serialize opening");
        transcript.append_u8_slice(&buffer);

        if !verified {
            println!("Failed to verify single pcs opening for parallel index {parallel_index}");
            return false;
        }

        let component_index = input.offset / input.len;
        let v_index = EqPolynomial::ith_eq_vec_elem(&component_idx_vars, component_index);

        target_y += v_index * claim;
    }

    *y == target_y
}

// TODO: Generate this with procedural macros
// The idea is to implement the ProvingSystem trait for KernelWiseProvingSystem
// However, we can not simply implement ProvingSystem<C> for all KernelWiseProvingSystem<C> because
// If later we want a customized implementation of ProvingSystem for some struct A
// The compiler will not allow use to do so, complaining that KernelWiseProvingSystem may be later implemented for A
// causing a potential conflict.
// In this case, generate the implementation with a procedural macro seems to be the best solution.
impl<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> ProvingSystem<ECCConfig>
    for Expander<C>
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    type ProverSetup = <Self as KernelWiseProvingSystem<ECCConfig>>::ProverSetup;
    type VerifierSetup = <Self as KernelWiseProvingSystem<ECCConfig>>::VerifierSetup;
    type Proof = CombinedProof<ECCConfig, Self>;

    fn setup(
        computation_graph: &ComputationGraph<ECCConfig>,
    ) -> (Self::ProverSetup, Self::VerifierSetup) {
        <Self as KernelWiseProvingSystem<ECCConfig>>::setup(computation_graph)
    }

    fn prove(
        prover_setup: &Self::ProverSetup,
        computation_graph: &ComputationGraph<ECCConfig>,
        device_memories: &[DeviceMemory<ECCConfig>],
    ) -> Self::Proof {
        let (commitments, extra_infos) = device_memories
            .iter()
            .map(|device_memory| {
                <Self as KernelWiseProvingSystem<ECCConfig>>::commit(
                    prover_setup,
                    &device_memory.values[..],
                )
            })
            .unzip::<_, _, Vec<_>, Vec<_>>();

        let proofs: Vec<<Self as KernelWiseProvingSystem<ECCConfig>>::Proof> = computation_graph
            .proof_templates
            .iter()
            .map(|template| {
                let (mut local_commitments, mut local_extra_info, mut local_vals) =
                    (vec![], vec![], vec![]);
                for idx in &template.commitment_indices {
                    local_commitments.push(&commitments[*idx]);
                    local_extra_info.push(&extra_infos[*idx]);
                    local_vals.push(&device_memories[*idx].values[..]);
                }

                <Self as KernelWiseProvingSystem<ECCConfig>>::prove_kernel(
                    prover_setup,
                    &computation_graph.kernels[template.kernel_id],
                    &local_commitments,
                    &local_extra_info,
                    &local_vals,
                    next_power_of_two(template.parallel_count),
                    &template.is_broadcast,
                )
            })
            .collect::<Vec<_>>();

        CombinedProof {
            commitments,
            proofs,
        }
    }

    fn verify(
        verifier_setup: &Self::VerifierSetup,
        computation_graph: &ComputationGraph<ECCConfig>,
        proof: &Self::Proof,
    ) -> bool {
        let verified = proof
            .proofs
            .par_iter()
            .zip(computation_graph.proof_templates.par_iter())
            .map(|(local_proof, template)| {
                let local_commitments = template
                    .commitment_indices
                    .iter()
                    .map(|idx| &proof.commitments[*idx])
                    .collect::<Vec<_>>();

                <Self as KernelWiseProvingSystem<ECCConfig>>::verify_kernel(
                    verifier_setup,
                    &computation_graph.kernels[template.kernel_id],
                    local_proof,
                    &local_commitments,
                    next_power_of_two(template.parallel_count),
                    &template.is_broadcast,
                )
            })
            .collect::<Vec<_>>();

        verified.iter().all(|x| *x)
    }

    fn post_process() {
        <Self as KernelWiseProvingSystem<ECCConfig>>::post_process();
    }
}

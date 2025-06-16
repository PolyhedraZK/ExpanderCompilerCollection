#![allow(clippy::type_complexity)]

use std::collections::HashMap;
use std::io::Cursor;

use crate::circuit::config::Config;
use crate::frontend::SIMDField;
use crate::utils::misc::next_power_of_two;
use crate::zkcuda::context::DeviceMemory;
use crate::zkcuda::proof::ComputationGraph;
use crate::zkcuda::proving_system::ProvingSystem;

use super::super::kernel::Kernel;
use super::{check_inputs, prepare_inputs, Commitment};

use arith::Field;
use expander_circuit::Circuit;
use expander_utils::timer::Timer;
use gkr::{gkr_prove, gkr_verify};
use gkr_engine::{
    ExpanderPCS, ExpanderSingleVarChallenge, FieldEngine, GKREngine, MPIConfig,
    Proof as ExpanderProof, StructuredReferenceString, Transcript,
};
use poly_commit::expander_pcs_init_testing_only;
use polynomials::{MultiLinearPoly, RefMultiLinearPoly};
use serdes::ExpSerde;
use sumcheck::ProverScratchPad;

#[derive(ExpSerde)]
pub struct ExpanderCommitment<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> {
    pub vals_len: usize,
    pub commitment: PCS::Commitment,
}

impl<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Clone
    for ExpanderCommitment<PCSField, F, PCS>
{
    fn clone(&self) -> Self {
        Self {
            vals_len: self.vals_len,
            commitment: self.commitment.clone(),
        }
    }
}

impl<
        F: FieldEngine,
        PCS: ExpanderPCS<F, F::SimdCircuitField>,
        ECCConfig: Config<FieldConfig = F>,
    > Commitment<ECCConfig> for ExpanderCommitment<F::SimdCircuitField, F, PCS>
{
    fn vals_len(&self) -> usize {
        self.vals_len
    }
}

#[derive(ExpSerde)]
pub struct ExpanderCommitmentState<
    PCSField: Field,
    F: FieldEngine,
    PCS: ExpanderPCS<F, PCSField>,
> {
    pub scratch: PCS::ScratchPad,
}

impl<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Clone
    for ExpanderCommitmentState<PCSField, F, PCS>
{
    fn clone(&self) -> Self {
        Self {
            scratch: self.scratch.clone(),
        }
    }
}

#[derive(ExpSerde)]
pub struct ExpanderProverSetup<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> {
    pub p_keys: HashMap<usize, <PCS::SRS as StructuredReferenceString>::PKey>,
}

impl<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Default
    for ExpanderProverSetup<PCSField, F, PCS>
{
    fn default() -> Self {
        Self {
            p_keys: HashMap::new(),
        }
    }
}

impl<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Clone
    for ExpanderProverSetup<PCSField, F, PCS>
{
    fn clone(&self) -> Self {
        Self {
            p_keys: self.p_keys.clone(),
        }
    }
}

#[derive(ExpSerde)]
pub struct ExpanderVerifierSetup<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>>
{
    pub v_keys: HashMap<usize, <PCS::SRS as StructuredReferenceString>::VKey>,
}

// implement default
impl<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Default
    for ExpanderVerifierSetup<PCSField, F, PCS>
{
    fn default() -> Self {
        Self {
            v_keys: HashMap::new(),
        }
    }
}

impl<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Clone
    for ExpanderVerifierSetup<PCSField, F, PCS>
{
    fn clone(&self) -> Self {
        Self {
            v_keys: self.v_keys.clone(),
        }
    }
}

#[derive(Clone, ExpSerde)]
pub struct ExpanderProof {
    pub data: Vec<ExpanderProof>, // each kernel may have several proofs, one for each parallel execution
}

#[derive(ExpSerde)]
pub struct CombinedProof<C: GKREngine> {
    pub commitments: Vec<ExpanderCommitment<C::PCSField, C::FieldConfig, C::PCSConfig>>,
    pub proofs: Vec<ExpanderProof>, // Multiple proofs for each kernel
    pub defered_pcs_opening: ExpanderProof,
}

impl<C: GKREngine> Clone for CombinedProof<C> {
    fn clone(&self) -> Self {
        Self {
            commitments: self.commitments.clone(),
            proofs: self.proofs.clone(),
            defered_pcs_opening: self.defered_pcs_opening.clone(),
        }
    }
}

pub struct ExpanderPCSDefered<C: GKREngine> {
    _config: std::marker::PhantomData<C>,
}

impl<C: GKREngine> ExpanderPCSDefered<C>
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    fn setup_impl<ECCConfig: Config<FieldConfig = C::FieldConfig>>(
        computation_graph: &crate::zkcuda::proof::ComputationGraph<ECCConfig>,
    ) -> (
        ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
        ExpanderVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    ) {
        let mut p_keys = HashMap::new();
        let mut v_keys = HashMap::new();
        let max_commitment_len = computation_graph
            .commitments_lens
            .iter()
            .max()
            .cloned()
            .unwrap_or(0);

        let (_params, p_key, v_key, _scratch) =
            pcs_testing_setup_fixed_seed::<C::FieldConfig, C::TranscriptConfig, C::PCSConfig>(
                max_commitment_len,
                &MPIConfig::prover_new(None, None),
            );
        p_keys.insert(max_commitment_len, p_key);
        v_keys.insert(max_commitment_len, v_key);

        (
            ExpanderProverSetup { p_keys },
            ExpanderVerifierSetup { v_keys },
        )
    }

    fn commit(
        prover_setup: &ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
        vals: &[SIMDField<C>],
    ) -> (
        ExpanderCommitment<C::PCSField, C::FieldConfig, C::PCSConfig>,
        ExpanderCommitmentState<C::PCSField, C::FieldConfig, C::PCSConfig>,
    ) {
        let timer = Timer::new("commit", true);

        assert_eq!(prover_setup.p_keys.len(), 1);
        let actual_len = vals.len();
        let len_to_commit = prover_setup.p_keys.keys().next().cloned().unwrap();
        assert!(len_to_commit >= actual_len);
        let mut vals = vals.to_vec();
        vals.resize(len_to_commit, SIMDField::<C>::ZERO);

        let p_key = prover_setup.p_keys.get(&len_to_commit).unwrap();

        let n_vars = len_to_commit.ilog2() as usize;
        let params =
            <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(n_vars, 1);

        let mut scratch =
            <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::init_scratch_pad(
                &params,
                &MPIConfig::prover_new(None, None),
            );

        let commitment = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::commit(
            &params,
            &MPIConfig::prover_new(None, None),
            p_key,
            &RefMultiLinearPoly::from_ref(&vals),
            &mut scratch,
        )
        .unwrap();

        timer.stop();
        (
            ExpanderCommitment {
                vals_len: actual_len,
                commitment,
            },
            ExpanderCommitmentState { scratch },
        )
    }

    fn prove_kernel<'a, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
        kernel: &Kernel<ECCConfig>,
        commitments_values: &[&'a [SIMDField<C>]],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> (
        ExpanderProof,
        Vec<&'a [SIMDField<C>]>,
        Vec<ExpanderSingleVarChallenge<C::FieldConfig>>,
    ) {
        let timer = Timer::new("prove", true);
        check_inputs(kernel, commitments_values, parallel_count, is_broadcast);

        let mut expander_circuit = kernel.layered_circuit.export_to_expander().flatten::<C>();
        expander_circuit.pre_process_gkr::<C>();
        let (max_num_input_var, max_num_output_var) = max_n_vars(&expander_circuit);
        let mut prover_scratch =
            ProverScratchPad::<C::FieldConfig>::new(max_num_input_var, max_num_output_var, 1);

        let mut proof = ExpanderProof { data: vec![] };
        let mut polys = vec![];
        let mut challenges = vec![];

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

            let (local_polys, local_challenges) = Self::extract_pcs_claims(
                commitments_values,
                &challenge.challenge_x(),
                is_broadcast,
                i,
                parallel_count,
            );
            polys.extend(local_polys);
            challenges.extend(local_challenges);
            assert!(challenge.challenge_y().is_none());

            proof.data.push(transcript.finalize_and_get_proof());
        }

        timer.stop();
        (proof, polys, challenges)
    }

    fn verify_kernel<'a, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
        kernel: &Kernel<ECCConfig>,
        commitments: &[&'a ExpanderCommitment<C::PCSField, C::FieldConfig, C::PCSConfig>],
        proof: &ExpanderProof,
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> (
        bool,
        Vec<&'a ExpanderCommitment<C::PCSField, C::FieldConfig, C::PCSConfig>>,
        Vec<ExpanderSingleVarChallenge<C::FieldConfig>>,
    ) {
        let timer = Timer::new("verify", true);
        let mut expander_circuit = kernel.layered_circuit.export_to_expander().flatten::<C>();
        expander_circuit.pre_process_gkr::<C>();

        let mut polys_commits = vec![];
        let mut challenges = vec![];

        for i in 0..parallel_count {
            let mut transcript = C::TranscriptConfig::new();
            transcript.append_u8_slice(&[0u8; 32]);
            expander_circuit.fill_rnd_coefs(&mut transcript);

            let mut cursor = Cursor::new(&proof.data[i].bytes);
            cursor.set_position(32);
            let (verified, challenge, _claimed_v0, _claimed_v1) = gkr_verify(
                1,
                &expander_circuit,
                &[],
                &<C::FieldConfig as FieldEngine>::ChallengeField::ZERO,
                &mut transcript,
                &mut cursor,
            );

            if !verified {
                println!("Failed to verify GKR proof for parallel index {i}");
                return (false, vec![], vec![]);
            }

            let (local_commitments, local_challenges) =
                Self::verifier_extract_pcs_claims::<ECCConfig>(
                    commitments,
                    &challenge.challenge_x(),
                    is_broadcast,
                    i,
                    parallel_count,
                );
            assert!(challenge.challenge_y().is_none());

            polys_commits.extend(local_commitments);
            challenges.extend(local_challenges);
        }
        timer.stop();
        (true, polys_commits, challenges)
    }

    #[allow(clippy::too_many_arguments)]
    fn extract_pcs_claims<'a>(
        commitments_values: &[&'a [SIMDField<C>]],
        gkr_challenge: &ExpanderSingleVarChallenge<C::FieldConfig>,
        is_broadcast: &[bool],
        parallel_index: usize,
        parallel_count: usize,
    ) -> (
        Vec<&'a [SIMDField<C>]>,
        Vec<ExpanderSingleVarChallenge<C::FieldConfig>>,
    ) {
        let mut commitment_values_rt = vec![];
        let mut challenges = vec![];

        for (&commitment_val, &ib) in commitments_values.iter().zip(is_broadcast) {
            let val_len = commitment_val.len();
            let (challenge_for_pcs, _) =
                get_challenge_for_pcs(gkr_challenge, val_len, parallel_index, parallel_count, ib);

            commitment_values_rt.push(commitment_val);
            challenges.push(challenge_for_pcs);
        }

        (commitment_values_rt, challenges)
    }

    #[allow(clippy::too_many_arguments)]
    fn verifier_extract_pcs_claims<'a, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
        commitments: &[&'a ExpanderCommitment<C::PCSField, C::FieldConfig, C::PCSConfig>],
        gkr_challenge: &ExpanderSingleVarChallenge<C::FieldConfig>,
        is_broadcast: &[bool],
        parallel_index: usize,
        parallel_count: usize,
    ) -> (
        Vec<&'a ExpanderCommitment<C::PCSField, C::FieldConfig, C::PCSConfig>>,
        Vec<ExpanderSingleVarChallenge<C::FieldConfig>>,
    ) {
        let mut commitments_rt = vec![];
        let mut challenges = vec![];

        for (&commitment, ib) in commitments.iter().zip(is_broadcast) {
            let val_len =
                <ExpanderCommitment<C::PCSField, C::FieldConfig, C::PCSConfig> as Commitment<
                    ECCConfig,
                >>::vals_len(commitment);
            let (challenge_for_pcs, _) =
                get_challenge_for_pcs(gkr_challenge, val_len, parallel_index, parallel_count, *ib);

            commitments_rt.push(commitment);
            challenges.push(challenge_for_pcs);
        }

        (commitments_rt, challenges)
    }
}

pub fn pcs_testing_setup_fixed_seed<
    'a,
    FConfig: FieldEngine,
    T: Transcript,
    PCS: ExpanderPCS<FConfig, FConfig::SimdCircuitField>,
>(
    vals_len: usize,
    mpi_config: &MPIConfig<'a>,
) -> (
    PCS::Params,
    <PCS::SRS as StructuredReferenceString>::PKey,
    <PCS::SRS as StructuredReferenceString>::VKey,
    PCS::ScratchPad,
) {
    expander_pcs_init_testing_only::<FConfig, FConfig::SimdCircuitField, PCS>(
        vals_len.ilog2() as usize,
        mpi_config,
    )
}

pub fn max_n_vars<C: FieldEngine>(circuit: &Circuit<C>) -> (usize, usize) {
    let mut max_num_input_var = 0;
    let mut max_num_output_var = 0;
    for layer in circuit.layers.iter() {
        max_num_input_var = max_num_input_var.max(layer.input_var_num);
        max_num_output_var = max_num_output_var.max(layer.output_var_num);
    }
    (max_num_input_var, max_num_output_var)
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

impl<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> ProvingSystem<ECCConfig>
    for ExpanderPCSDefered<C>
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    type ProverSetup = ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>;
    type VerifierSetup = ExpanderVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>;
    type Proof = CombinedProof<C>;

    fn setup(
        computation_graph: &ComputationGraph<ECCConfig>,
    ) -> (
        ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
        Self::VerifierSetup,
    ) {
        Self::setup_impl(computation_graph)
    }

    fn prove(
        prover_setup: &ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
        computation_graph: &ComputationGraph<ECCConfig>,
        device_memories: &[DeviceMemory<ECCConfig>],
    ) -> Self::Proof {
        let (commitments, extra_infos) = device_memories
            .iter()
            .map(|device_memory| Self::commit(prover_setup, &device_memory.values[..]))
            .unzip::<_, _, Vec<_>, Vec<_>>();

        let mut proofs = vec![];
        let mut polys = vec![];
        let mut challenges = vec![];

        for template in computation_graph.proof_templates.iter() {
            let (mut local_commitments, mut local_extra_info, mut local_vals) =
                (vec![], vec![], vec![]);
            for idx in &template.commitment_indices {
                local_commitments.push(&commitments[*idx]);
                local_extra_info.push(&extra_infos[*idx]);
                local_vals.push(&device_memories[*idx].values[..]);
            }

            let (kernel_proof, local_polys, local_challenges) = Self::prove_kernel(
                &computation_graph.kernels[template.kernel_id],
                &local_vals,
                next_power_of_two(template.parallel_count),
                &template.is_broadcast,
            );
            proofs.push(kernel_proof);
            polys.extend(local_polys);
            challenges.extend(local_challenges);
        }

        // TODO: Modify the batch opening/verification interface in ExpanderPCS to accept
        // RefMultiLinearPoly
        let polys: Vec<_> = polys
            .into_iter()
            .map(|poly_vals| MultiLinearPoly::new(poly_vals.to_vec()))
            .collect();

        let mut transcript = C::TranscriptConfig::new();
        let max_num_vars = prover_setup.p_keys.keys().max().cloned().unwrap_or(0);
        let params =
            <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(max_num_vars, 1);
        let scratch_pad =
            <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::init_scratch_pad(
                &params,
                &MPIConfig::prover_new(None, None),
            );

        transcript.lock_proof();
        let (vals, opening) =
            <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::multi_points_batch_open(
                &params,
                &MPIConfig::prover_new(None, None),
                prover_setup.p_keys.get(&max_num_vars).unwrap(),
                &polys,
                &challenges,
                &scratch_pad,
                &mut transcript,
            );
        transcript.unlock_proof();

        let mut bytes = vec![];
        vals.serialize_into(&mut bytes).unwrap();
        opening.serialize_into(&mut bytes).unwrap();
        let defered_pcs_opening = ExpanderProof { bytes };

        CombinedProof {
            commitments,
            proofs,
            defered_pcs_opening,
        }
    }

    fn verify(
        verifier_setup: &Self::VerifierSetup,
        computation_graph: &ComputationGraph<ECCConfig>,
        proof: &Self::Proof,
    ) -> bool {
        let mut commitments = vec![];
        let mut challenges = vec![];

        for i in 0..proof.proofs.len() {
            let kernel_proof = &proof.proofs[i];
            let template = &computation_graph.proof_templates[i];

            let local_commitments = template
                .commitment_indices
                .iter()
                .map(|idx| &proof.commitments[*idx])
                .collect::<Vec<_>>();

            let (verified, commitments_for_pcs, challenges_for_pcs) = Self::verify_kernel(
                &computation_graph.kernels[template.kernel_id],
                &local_commitments,
                kernel_proof,
                next_power_of_two(template.parallel_count),
                &template.is_broadcast,
            );

            if !verified {
                return false;
            }
            commitments.extend(commitments_for_pcs);
            challenges.extend(challenges_for_pcs);
        }

        let commitments: Vec<_> = commitments
            .into_iter()
            .map(|commitment| commitment.commitment.clone())
            .collect();

        let mut transcript = C::TranscriptConfig::new();
        let max_num_vars = verifier_setup.v_keys.keys().max().cloned().unwrap_or(0);
        let params =
            <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(max_num_vars, 1);

        let mut defered_proof_bytes = proof.defered_pcs_opening.bytes.clone();
        let mut cursor = Cursor::new(&mut defered_proof_bytes);

        let vals =
            Vec::<<C::FieldConfig as FieldEngine>::ChallengeField>::deserialize_from(&mut cursor)
                .unwrap();
        let opening = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::BatchOpening::deserialize_from(&mut cursor).unwrap();

        transcript.lock_proof();
        let pcs_verified =
            <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::multi_points_batch_verify(
                &params,
                verifier_setup.v_keys.get(&max_num_vars).unwrap(),
                &commitments,
                &challenges,
                &vals,
                &opening,
                &mut transcript,
            );
        transcript.unlock_proof();

        pcs_verified
    }
}

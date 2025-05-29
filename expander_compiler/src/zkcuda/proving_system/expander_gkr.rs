use std::cmp::max;
use std::collections::HashMap;
use std::io::{Cursor, Read};

use crate::circuit::config::Config;
use crate::frontend::SIMDField;

use super::super::kernel::Kernel;
use super::{check_inputs, prepare_inputs, Commitment, Proof, ProvingSystem};

use arith::Field;
use expander_circuit::Circuit;
use expander_utils::timer::Timer;
use gkr::{gkr_prove, gkr_verify};
use gkr_engine::{
    ExpanderPCS, ExpanderSingleVarChallenge, FieldEngine, GKREngine, MPIConfig,
    Proof as ExpanderProof, StructuredReferenceString, Transcript,
};
use poly_commit::expander_pcs_init_testing_only;
use polynomials::{EqPolynomial, MultiLinearPoly, RefMultiLinearPoly};
use serdes::ExpSerde;
use sumcheck::ProverScratchPad;

#[allow(clippy::type_complexity)]
#[derive(ExpSerde)]
pub struct ExpanderGKRCommitment<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> {
    pub vals_len: usize,
    pub commitment: Vec<PCS::Commitment>,
}

impl<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Clone
    for ExpanderGKRCommitment<PCSField, F, PCS>
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
    > Commitment<ECCConfig> for ExpanderGKRCommitment<F::SimdCircuitField, F, PCS>
{
    fn vals_len(&self) -> usize {
        self.vals_len
    }
}

#[allow(clippy::type_complexity)]
#[derive(ExpSerde)]
pub struct ExpanderGKRCommitmentExtraInfo<
    PCSField: Field,
    F: FieldEngine,
    PCS: ExpanderPCS<F, PCSField>,
> {
    pub scratch: Vec<PCS::ScratchPad>,
}

impl<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Clone
    for ExpanderGKRCommitmentExtraInfo<PCSField, F, PCS>
{
    fn clone(&self) -> Self {
        Self {
            scratch: self.scratch.clone(),
        }
    }
}

#[allow(clippy::type_complexity)]
#[derive(ExpSerde)]
pub struct ExpanderGKRProverSetup<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> {
    pub p_keys: HashMap<usize, <PCS::SRS as StructuredReferenceString>::PKey>,
}

// implement default
impl<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Default
    for ExpanderGKRProverSetup<PCSField, F, PCS>
{
    fn default() -> Self {
        Self {
            p_keys: HashMap::new(),
        }
    }
}

impl<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Clone
    for ExpanderGKRProverSetup<PCSField, F, PCS>
{
    fn clone(&self) -> Self {
        Self {
            p_keys: self.p_keys.clone(),
        }
    }
}

#[allow(clippy::type_complexity)]
#[derive(ExpSerde)]
pub struct ExpanderGKRVerifierSetup<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>>
{
    pub v_keys: HashMap<usize, <PCS::SRS as StructuredReferenceString>::VKey>,
}

// implement default
impl<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Default
    for ExpanderGKRVerifierSetup<PCSField, F, PCS>
{
    fn default() -> Self {
        Self {
            v_keys: HashMap::new(),
        }
    }
}

impl<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Clone
    for ExpanderGKRVerifierSetup<PCSField, F, PCS>
{
    fn clone(&self) -> Self {
        Self {
            v_keys: self.v_keys.clone(),
        }
    }
}

#[derive(Clone, ExpSerde)]
pub struct ExpanderGKRProof {
    pub data: Vec<ExpanderProof>,
}

impl Proof for ExpanderGKRProof {}

pub struct ExpanderGKRProvingSystem<C: GKREngine> {
    _config: std::marker::PhantomData<C>,
}

impl<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> ProvingSystem<ECCConfig>
    for ExpanderGKRProvingSystem<C>
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
        let mut p_keys = HashMap::new();
        let mut v_keys = HashMap::new();
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
                if p_keys.contains_key(&val_actual_len) {
                    continue;
                }
                let (_params, p_key, v_key, _scratch) = pcs_testing_setup_fixed_seed::<
                    C::FieldConfig,
                    C::TranscriptConfig,
                    C::PCSConfig,
                >(
                    val_actual_len,
                    &MPIConfig::prover_new(None, None),
                );
                p_keys.insert(val_actual_len, p_key);
                v_keys.insert(val_actual_len, v_key);
            }
        }

        (
            ExpanderGKRProverSetup { p_keys },
            ExpanderGKRVerifierSetup { v_keys },
        )
    }

    fn commit(
        prover_setup: &Self::ProverSetup,
        vals: &[SIMDField<C>],
        parallel_count: usize,
        is_broadcast: bool,
    ) -> (Self::Commitment, Self::CommitmentExtraInfo) {
        let timer = Timer::new("commit", true);

        let vals_to_commit = if is_broadcast {
            vec![vals]
        } else {
            vals.chunks(vals.len() / parallel_count).collect::<Vec<_>>()
        };

        let n_vars = vals_to_commit[0].len().ilog2() as usize;
        let params =
            <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(n_vars, 1);
        let p_key = prover_setup.p_keys.get(&(1 << n_vars)).unwrap();

        let (commitment, scratch) = vals_to_commit
            .into_iter()
            .map(|vals| {
                let mut scratch =
                    <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::init_scratch_pad(
                        &params,
                        &MPIConfig::default(),
                    );

                let commitment =
                    <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::commit(
                        &params,
                        &MPIConfig::default(),
                        p_key,
                        &MultiLinearPoly::new(vals.to_vec()),
                        &mut scratch,
                    )
                    .unwrap();
                (commitment, scratch)
            })
            .unzip::<_, _, Vec<_>, Vec<_>>();

        timer.stop();
        (
            Self::Commitment {
                vals_len: 1 << n_vars,
                commitment,
            },
            Self::CommitmentExtraInfo { scratch },
        )
    }

    fn prove(
        prover_setup: &Self::ProverSetup,
        _kernel_id: usize,
        kernel: &Kernel<ECCConfig>,
        _commitments: &[Self::Commitment],
        commitments_extra_info: &[Self::CommitmentExtraInfo],
        commitments_values: &[&[SIMDField<C>]],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> Self::Proof {
        let timer = Timer::new("prove", true);
        check_inputs(kernel, commitments_values, parallel_count, is_broadcast);

        let mut expander_circuit = kernel.layered_circuit.export_to_expander().flatten::<C>();
        expander_circuit.pre_process_gkr::<C>();
        let (max_num_input_var, max_num_output_var) = max_n_vars(&expander_circuit);
        let max_num_var = max(max_num_input_var, max_num_output_var);
        let mut prover_scratch =
            ProverScratchPad::<C::FieldConfig>::new(max_num_var, max_num_var, 1);

        let mut proof = ExpanderGKRProof { data: vec![] };

        // For each parallel index, prove the GKR proof
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
                &MPIConfig::default(),
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

    fn verify(
        verifier_setup: &Self::VerifierSetup,
        _kernel_id: usize,
        kernel: &Kernel<ECCConfig>,
        proof: &Self::Proof,
        commitments: &[Self::Commitment],
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
                println!("Failed to verify GKR proof for parallel index {}", i);
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
                println!("Failed to verify overall pcs for parallel index {}", i);
                return false;
            }
        }
        timer.stop();
        true
    }
}

#[allow(clippy::type_complexity)]
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

#[allow(clippy::too_many_arguments)]
fn prove_input_claim<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    _kernel: &Kernel<ECCConfig>,
    commitments_values: &[&[SIMDField<C>]],
    p_keys: &ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    commitments_extra_info: &[ExpanderGKRCommitmentExtraInfo<
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

        let params =
            <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(val_len, 1);
        let p_key = p_keys.p_keys.get(&val_len).unwrap();

        let poly = RefMultiLinearPoly::from_ref(vals_to_open);
        let v =
            <C::FieldConfig as FieldEngine>::single_core_eval_circuit_vals_at_expander_challenge(
                vals_to_open,
                &ExpanderSingleVarChallenge {
                    rz: challenge_vars.to_vec(),
                    r_simd: challenge.r_simd.to_vec(),
                    r_mpi: vec![],
                },
            );
        transcript.append_field_element(&v);

        transcript.lock_proof();
        let scratch_for_parallel_index = if *ib {
            &extra_info.scratch[0]
        } else {
            &extra_info.scratch[parallel_index]
        };
        let opening = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::open(
            &params,
            &MPIConfig::default(),
            p_key,
            &poly,
            &ExpanderSingleVarChallenge {
                rz: challenge_vars.to_vec(),
                r_simd: challenge.r_simd.to_vec(),
                r_mpi: vec![],
            },
            transcript,
            scratch_for_parallel_index,
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
    v_keys: &ExpanderGKRVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    challenge: &ExpanderSingleVarChallenge<C::FieldConfig>,
    y: &<C::FieldConfig as FieldEngine>::ChallengeField,
    commitments: &[ExpanderGKRCommitment<C::PCSField, C::FieldConfig, C::PCSConfig>],
    is_broadcast: &[bool],
    parallel_index: usize,
    _parallel_count: usize,
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
        let commitment_len =
            <ExpanderGKRCommitment<C::PCSField, C::FieldConfig, C::PCSConfig> as Commitment<
                ECCConfig,
            >>::vals_len(commitment);
        let nb_challenge_vars = commitment_len.ilog2() as usize;
        let challenge_vars = challenge.rz[..nb_challenge_vars].to_vec();

        let params = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(
            commitment_len.ilog2() as usize,
            1,
        );
        let v_key = v_keys.v_keys.get(&commitment_len).unwrap();

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
        let commitment_for_parallel_index = if *ib {
            &commitment.commitment[0]
        } else {
            &commitment.commitment[parallel_index]
        };
        let verified = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::verify(
            &params,
            v_key,
            commitment_for_parallel_index,
            &ExpanderSingleVarChallenge {
                rz: challenge_vars,
                r_simd: challenge.r_simd.to_vec(),
                r_mpi: vec![],
            },
            claim,
            transcript,
            &opening,
        );
        transcript.unlock_proof();

        if !verified {
            println!(
                "Failed to verify single pcs opening for parallel index {}",
                parallel_index
            );
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

    *y == target_y
}

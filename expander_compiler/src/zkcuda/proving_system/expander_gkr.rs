use std::io::{Cursor, Read};
use std::collections::HashMap;

use crate::circuit::config::Config;
use crate::field;

use super::super::kernel::Kernel;
use super::{check_inputs, prepare_inputs, Commitment, ProvingSystem};

use arith::Field;
use expander_circuit::Circuit;
use expander_config::GKRConfig;
use expander_transcript::{Proof as ExpanderProof, Transcript};
use gkr::{gkr_prove, gkr_verify};
use gkr_field_config::GKRFieldConfig;
use mpi_config::MPIConfig;
use poly_commit::{
    expander_pcs_init_testing_only, raw::*, ExpanderGKRChallenge, PCSEmptyType, PCSForExpanderGKR, StructuredReferenceString,
};
use polynomials::{EqPolynomial, MultiLinearPoly, RefMultiLinearPoly};
use serdes::ExpSerde;
use sumcheck::ProverScratchPad;

use rand::rngs::StdRng;
use rand::SeedableRng;

pub struct ExpanderGKRCommitment<C, T, PCS> 
where 
    C: GKRFieldConfig,
    T: Transcript<C::ChallengeField>,
    PCS: PCSForExpanderGKR<C, T>
{
    vals_len: usize,
    commitment: PCS::Commitment,
}

impl<Cfg, C, T, PCS> Commitment<Cfg> for ExpanderGKRCommitment<C, T, PCS> 
where 
    Cfg: Config<DefaultGKRFieldConfig = C>,
    C: GKRFieldConfig,
    T: Transcript<C::ChallengeField>,
    PCS: PCSForExpanderGKR<C, T>,
{
    fn vals_len(&self) -> usize {
        self.vals_len
    }
}

// Derive does not seem to work since PCS does not implement Clone, although all the fields do
impl<C, T, PCS> Clone for ExpanderGKRCommitment<C, T, PCS> 
where 
    C: GKRFieldConfig,
    T: Transcript<C::ChallengeField>,
    PCS: PCSForExpanderGKR<C, T>,
{
    fn clone(&self) -> Self {
        Self {
            vals_len: self.vals_len,
            commitment: self.commitment.clone(),
        }
    }
}

pub struct ExpanderGKRCommitmentExtraInfo<C, T, PCS>
where 
    C: GKRFieldConfig,
    T: Transcript<C::ChallengeField>,
    PCS: PCSForExpanderGKR<C, T>,
{
    scratch: PCS::ScratchPad,
}

impl<C, T, PCS> Clone for ExpanderGKRCommitmentExtraInfo<C, T, PCS>
where
    C: GKRFieldConfig,
    T: Transcript<C::ChallengeField>,
    PCS: PCSForExpanderGKR<C, T>,
{
    fn clone(&self) -> Self {
        Self {
            scratch: self.scratch.clone(),
        }
    }
}

pub struct ExpanderGKRProverSetup<C, T, PCS> 
where 
    C: GKRFieldConfig,
    T: Transcript<C::ChallengeField>,
    PCS: PCSForExpanderGKR<C, T>,
{
    p_keys: HashMap<usize, <PCS::SRS as StructuredReferenceString>::PKey>,
}

impl<C, T, PCS> Clone for ExpanderGKRProverSetup<C, T, PCS> 
where 
    C: GKRFieldConfig,
    T: Transcript<C::ChallengeField>,
    PCS: PCSForExpanderGKR<C, T>,
{
    fn clone(&self) -> Self {
        Self {
            p_keys: self.p_keys.clone(),
        }
    }
}

pub struct ExpanderGKRVerifierSetup<C, T, PCS> 
where 
    C: GKRFieldConfig,
    T: Transcript<C::ChallengeField>,
    PCS: PCSForExpanderGKR<C, T>,
{
    v_keys: HashMap<usize, <PCS::SRS as StructuredReferenceString>::VKey>,
}

impl<C, T, PCS> Clone for ExpanderGKRVerifierSetup<C, T, PCS> 
where 
    C: GKRFieldConfig,
    T: Transcript<C::ChallengeField>,
    PCS: PCSForExpanderGKR<C, T>,
{
    fn clone(&self) -> Self {
        Self {
            v_keys: self.v_keys.clone(),
        }
    }
    
}

#[derive(Clone)]
pub struct ExpanderGKRProof {
    data: Vec<ExpanderProof>,
}

pub struct ExpanderGKRProvingSystem<C: Config> {
    _config: std::marker::PhantomData<C>,
}

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

impl<C: Config> ProvingSystem<C> for ExpanderGKRProvingSystem<C> {
    type ProverSetup = ExpanderGKRProverSetup<field!(C), transcript!(C), pcs!(C)>;
    type VerifierSetup = ExpanderGKRVerifierSetup<field!(C), transcript!(C), pcs!(C)>;
    type Proof = ExpanderGKRProof;
    type Commitment = ExpanderGKRCommitment<field!(C), transcript!(C), pcs!(C)>;
    type CommitmentExtraInfo = ExpanderGKRCommitmentExtraInfo<field!(C), transcript!(C), pcs!(C)>;

    fn setup(computation_graph: &crate::zkcuda::proof::ComputationGraph<C>) -> (Self::ProverSetup, Self::VerifierSetup) {
        let mut p_keys = HashMap::new();
        let mut v_keys = HashMap::new();
        for commitment_len in computation_graph.commitments_lens.iter() {
            if p_keys.contains_key(commitment_len) {
                continue;
            }

            let (_params, p_key, v_key, _scratch) = 
                pcs_testing_setup_fixed_seed::<field!(C), transcript!(C), pcs!(C)>(*commitment_len);
            p_keys.insert(*commitment_len, p_key);
            v_keys.insert(*commitment_len, v_key);
        }
        (ExpanderGKRProverSetup { p_keys }, ExpanderGKRVerifierSetup { v_keys })
    }

    fn commit(
            prover_setup: &Self::ProverSetup,
            vals: &Vec<<C as Config>::DefaultSimdField>,
        ) -> (Self::Commitment, Self::CommitmentExtraInfo) {
        let n_vars = vals.len().ilog2() as usize;
        let params = <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::gen_params(n_vars);
        let p_key = prover_setup.p_keys.get(&vals.len()).unwrap();
        let mut scratch = <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::init_scratch_pad(&params, &MPIConfig::default());
        
        let commitment = 
            <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::commit(
            &params,
            &MPIConfig::default(),
            p_key,
            &RefMultiLinearPoly::from_ref(vals), 
            &mut scratch,
        );
        (Self::Commitment { vals_len: vals.len(), commitment }, Self::CommitmentExtraInfo { scratch })
    }

    fn prove(
            prover_setup: &Self::ProverSetup,
            kernel: &Kernel<C>,
            _commitments: &[&Self::Commitment],
            commitments_extra_info: &[&Self::CommitmentExtraInfo],
            commitments_values: &[&[<C as Config>::DefaultSimdField]],
            parallel_count: usize,
            is_broadcast: &[bool],
        ) -> Self::Proof {
            check_inputs(kernel, commitments_values, parallel_count, is_broadcast);

            let mut expander_circuit = kernel.layered_circuit.export_to_expander().flatten();
            expander_circuit.identify_rnd_coefs();
            expander_circuit.identify_structure_info();
            let (max_num_input_var, max_num_output_var) = max_n_vars(&expander_circuit);
            let mut prover_scratch = ProverScratchPad::<C::DefaultGKRFieldConfig>::new(
                        max_num_input_var,
                        max_num_output_var,
                        1,
                    );

            let mut proof = ExpanderGKRProof { data: vec![] };

            // For each parallel index, prove the GKR proof
            for i in 0..parallel_count {
                let mut transcript = <C::DefaultGKRConfig as GKRConfig>::Transcript::new();
                transcript.append_u8_slice(&[0u8; 32]); // TODO: Replace with the commitment, and hash an additional a few times
                expander_circuit.layers[0].input_vals =
                    prepare_inputs(kernel, commitments_values, is_broadcast, i);
                expander_circuit.fill_rnd_coefs(&mut transcript);
                expander_circuit.evaluate();
                let (claimed_v, rx, ry, rsimd, _rmpi) = gkr_prove(
                    &expander_circuit,
                    &mut prover_scratch,
                    &mut transcript,
                    &MPIConfig::new(),
                );
                assert_eq!(
                    claimed_v,
                    <C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField::from(0)
                );

                prove_input_claim(
                    kernel,
                    commitments_values,
                    prover_setup,
                    commitments_extra_info,
                    &rx,
                    &rsimd,
                    is_broadcast,
                    i,
                    &mut transcript,
                );
                if let Some(ry) = ry {
                    prove_input_claim(
                        kernel,
                        commitments_values,
                        prover_setup,
                        commitments_extra_info,
                        &ry,
                        &rsimd,
                        is_broadcast,
                        i,
                        &mut transcript,
                    );
                }

                proof.data.push(transcript.finalize_and_get_proof());
            }

            proof

    }

    fn verify(
            verifier_setup: &Self::VerifierSetup,
            kernel: &Kernel<C>,
            proof: &Self::Proof,
            commitments: &[&Self::Commitment],
            parallel_count: usize,
            is_broadcast: &[bool],
        ) -> bool {
            let mut expander_circuit: Circuit<C::DefaultGKRFieldConfig> =
                kernel.layered_circuit.export_to_expander().flatten();
            expander_circuit.identify_rnd_coefs();
            expander_circuit.identify_structure_info();

            for i in 0..parallel_count {
                let mut transcript = <C::DefaultGKRConfig as GKRConfig>::Transcript::new();
                transcript.append_u8_slice(&[0u8; 32]); 
                let mut cursor = Cursor::new(&proof.data[i].bytes);
                cursor.set_position(32);
                let (mut verified, rz0, rz1, r_simd, _r_mpi, claimed_v0, claimed_v1) = gkr_verify(
                    &MPIConfig::default(),
                    &expander_circuit,
                    &[],
                    &<C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField::ZERO,
                    &mut transcript,
                    &mut cursor,
                );
    
                verified &= verify_input_claim(
                    &mut cursor,
                    &rz0,
                    &r_simd,
                    &claimed_v0,
                    kernel,
                    commitments,
                    is_broadcast,
                    i,
                    &mut transcript,
                );
                if let Some(rz1) = rz1 {
                    verified &= verify_input_claim(
                        &mut cursor,
                        &rz1,
                        &r_simd,
                        &claimed_v1.unwrap(),
                        kernel,
                        commitments,
                        is_broadcast,
                        i,
                        &mut transcript,
                    );
                }
    
                if !verified {
                    return false;
                }
            }
            true
    }

    // fn verify(
    //     kernel: &Kernel<C>,
    //     proof: &Self::Proof,
    //     commitments: &[&Self::Commitment],
    //     parallel_count: usize,
    //     is_broadcast: &[bool],
    // ) -> bool {
    //     
    // }
}

fn pcs_testing_setup_fixed_seed<
    FConfig: GKRFieldConfig, 
    T: Transcript<FConfig::ChallengeField>, 
    PCS: PCSForExpanderGKR<FConfig, T>
>(
    vals_len: usize,
) -> (
    PCS::Params,
    <PCS::SRS as StructuredReferenceString>::PKey,
    <PCS::SRS as StructuredReferenceString>::VKey,
    PCS::ScratchPad,
) {
    let mut rng = StdRng::from_seed([123; 32]);
    expander_pcs_init_testing_only::<
        FConfig,
        T,
        PCS,
    >(
        vals_len.ilog2() as usize,
        &MPIConfig::default(),
        &mut rng,
    )
}

fn max_n_vars<C: GKRFieldConfig>(circuit: &Circuit<C>) -> (usize, usize) {
    let mut max_num_input_var = 0;
    let mut max_num_output_var = 0;
    for layer in circuit.layers.iter() {
        max_num_input_var = max_num_input_var.max(layer.input_var_num);
        max_num_output_var = max_num_output_var.max(layer.output_var_num);
    }
    (max_num_input_var, max_num_output_var)
}

fn prove_input_claim<C: Config>(
    kernel: &Kernel<C>,
    commitments_values: &[&[C::DefaultSimdField]],
    p_keys: &ExpanderGKRProverSetup<field!(C), transcript!(C), pcs!(C)>,
    commitments_extra_info: &[&ExpanderGKRCommitmentExtraInfo<field!(C), transcript!(C), pcs!(C)>],
    x: &[<field!(C) as GKRFieldConfig>::ChallengeField],
    x_simd: &[<field!(C) as GKRFieldConfig>::ChallengeField],
    is_broadcast: &[bool],
    parallel_index: usize,
    transcript: &mut transcript!(C),
) {
    for (((input, commitment_val), extra_info), ib) in kernel
        .layered_circuit_input
        .iter()
        .zip(commitments_values)
        .zip(commitments_extra_info)
        .zip(is_broadcast)
    {
        let nb_challenge_vars = input.len.trailing_zeros() as usize;
        let challenge_vars = &x[..nb_challenge_vars];

        let vals = if *ib {
            commitment_val
        } else {
            &commitment_val[parallel_index * input.len..(parallel_index + 1) * input.len]
        };

        let params = <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::gen_params(
            vals.len().ilog2() as usize,
        );
        let p_key = p_keys.p_keys.get(&vals.len()).unwrap();
        
        // TODO: Remove unsafe after switching to the main branch
        let mut mutable_scratch = unsafe {
            let ptr = &extra_info.scratch as *const <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::ScratchPad as *mut <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::ScratchPad;
            &mut *ptr
        };

        // TODO: Remove unnecessary clone
        let poly = MultiLinearPoly::new(vals.to_vec());
        // Meant to use raw to evaluate the polynomial
        // TODO: Switch to `MultilinearPolyExpander` after switch to the main branch
        let v = RawExpanderGKR::<
            field!(C),
            transcript!(C),
        >::eval(vals, challenge_vars, x_simd, &[]);
        transcript.append_field_element(&v);

        transcript.lock_proof();
        let opening = <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::open(
            &params,
            &MPIConfig::default(),
            &p_key,
            &poly,
            &ExpanderGKRChallenge::<C::DefaultGKRFieldConfig> {
                x: challenge_vars.to_vec(),
                x_simd: x_simd.to_vec(),
                x_mpi: vec![],
            },
            transcript,
            &mut mutable_scratch,
        );
        transcript.unlock_proof();

        let mut buffer = vec![];
        opening
            .serialize_into(&mut buffer)
            .expect("Failed to serialize opening");
        transcript.append_u8_slice(&buffer);
    }
}

#[allow(clippy::too_many_arguments)]
fn verify_input_claim<C: Config>(
    mut proof_reader: impl Read,
    kernel: &Kernel<C>,
    x: &[<C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField],
    x_simd: &[<C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField],
    y: &<C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField,
    commitments: &[&ExpanderGKRCommitment<field!(C), transcript!(C), pcs!(C)>],
    is_broadcast: &[bool],
    parallel_index: usize,
    transcript: &mut transcript!(C),
) -> bool {
    let mut target_y = <C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField::ZERO;
    for ((input, &commitment), ib) in kernel
        .layered_circuit_input
        .iter()
        .zip(commitments)
        .zip(is_broadcast)
    {
        let claim = <C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField::deserialize_from(
            &mut proof_reader,
        )
        .unwrap();
        let nb_challenge_vars = input.len.trailing_zeros() as usize;
        let challenge_vars = &x[..nb_challenge_vars];

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

        let raw_commitment = if *ib {
            commitment.vals_ref()
        } else {
            commitment.vals_ref()[parallel_index * input.len..(parallel_index + 1) * input.len]
                .as_ref()
        };

        let (params, _p_key, v_key, _) = pcs_testing_setup_fixed_seed::<C>(raw_commitment);
        let opening = <RawExpanderGKR<
            C::DefaultGKRFieldConfig,
            <C::DefaultGKRConfig as GKRConfig>::Transcript,
        > as PCSForExpanderGKR<
            C::DefaultGKRFieldConfig,
            <C::DefaultGKRConfig as GKRConfig>::Transcript,
        >>::Opening::deserialize_from(&mut proof_reader)
        .unwrap();
        RawExpanderGKR::verify(
            &params,
            &MPIConfig::default(),
            &v_key,
            &RawCommitment {
                evals: raw_commitment.to_vec(),
            },
            &ExpanderGKRChallenge::<C::DefaultGKRFieldConfig> {
                x: challenge_vars.to_vec(),
                x_simd: x_simd.to_vec(),
                x_mpi: vec![],
            },
            claim,
            transcript,
            &opening,
        );
    }

    *y == target_y
}

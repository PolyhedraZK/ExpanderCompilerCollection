use std::io::{Cursor, Read};

use crate::circuit::config::Config;

use super::super::kernel::Kernel;
use super::{check_inputs, prepare_inputs, Commitment, Proof, ProvingSystem};

use arith::{Field, FieldSerde};
use expander_circuit::Circuit;
use expander_config::GKRConfig;
use expander_transcript::{Proof as ExpanderProof, Transcript};
use gkr::{gkr_prove, gkr_verify};
use gkr_field_config::GKRFieldConfig;
use mpi_config::MPIConfig;
use poly_commit::{
    expander_pcs_init_testing_only, raw::*, ExpanderGKRChallenge, PCSEmptyType, PCSForExpanderGKR,
};
use polynomials::{EqPolynomial, MultiLinearPoly, RefMultiLinearPoly};
use sumcheck::ProverScratchPad;

use rand::rngs::StdRng;
use rand::SeedableRng;

// TODO: Move these to a config struct
const MPI_SIZE: usize = 2;
const GKR_PROVER_BINARY: &str = "/tmp/prove_gkr.bin";
const GKR_VERIFIER_BINARY: &str = "/tmp/verify_gkr.bin";
const PCS_COMMIT_BINARY: &str = "/tmp/commit_pcs.bin";
const PCS_OPEN_BINARY: &str = "/tmp/open_pcs.bin";
const PCS_VERIFY_BINARY: &str = "/tmp/verify_pcs.bin";

#[allow(dead_code)]
#[derive(Clone)]
pub struct ExpanderGKRCommitment<C: Config> {
    vals: Vec<C::DefaultSimdField>,
    commitment: RawCommitment<C::DefaultSimdField>,
    scratch: RawExpanderGKRScratchPad,
}

impl<C: Config> Commitment<C> for ExpanderGKRCommitment<C> {
    fn vals_ref(&self) -> &[C::DefaultSimdField] {
        &self.vals
    }
}

#[derive(Clone)]
pub struct ExpanderGKRProof {
    data: Vec<ExpanderProof>,
}

impl Proof for ExpanderGKRProof {}

pub struct ExpanderGKRProvingSystem<C: Config> {
    _config: std::marker::PhantomData<C>,
}

impl<C: Config> ProvingSystem<C> for ExpanderGKRProvingSystem<C> {
    type Proof = ExpanderGKRProof;
    type Commitment = ExpanderGKRCommitment<C>;

    fn commit(vals: &[C::DefaultSimdField]) -> Self::Commitment {
        assert!(vals.len() & (vals.len() - 1) == 0);

        let (params, p_key, _v_key, mut scratch) = pcs_testing_setup_fixed_seed::<C>(vals);

        let vals = vals.to_vec();
        let poly_ref = RefMultiLinearPoly::from_ref(&vals);
        let raw_commitment = RawExpanderGKR::<
            C::DefaultGKRFieldConfig,
            <C::DefaultGKRConfig as GKRConfig>::Transcript,
        >::commit(
            &params,
            &MPIConfig::default(),
            &p_key,
            &poly_ref,
            &mut scratch,
        );

        ExpanderGKRCommitment {
            vals,
            commitment: raw_commitment,
            scratch,
        }
    }

    fn prove(
        kernel: &Kernel<C>,
        commitments: &[&Self::Commitment],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> ExpanderGKRProof {
        check_inputs(kernel, commitments, parallel_count, is_broadcast);

        let mut expander_circuit = kernel.layered_circuit.export_to_expander().flatten();
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
                prepare_inputs(kernel, commitments, is_broadcast, i);
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
                &rx,
                &rsimd,
                kernel,
                commitments,
                is_broadcast,
                i,
                &mut transcript,
            );
            if let Some(ry) = ry {
                prove_input_claim(
                    &ry,
                    &rsimd,
                    kernel,
                    commitments,
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
        kernel: &Kernel<C>,
        proof: &Self::Proof,
        commitments: &[&Self::Commitment],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> bool {
        check_inputs(kernel, commitments, parallel_count, is_broadcast);

        let expander_circuit: Circuit<C::DefaultGKRFieldConfig> =
            kernel.layered_circuit.export_to_expander().flatten();

        for i in 0..parallel_count {
            let mut transcript = <C::DefaultGKRConfig as GKRConfig>::Transcript::new();
            transcript.append_u8_slice(&[0u8; 32]); // TODO: Replace with the commitment, and hash an additional a few times
            let mut cursor = Cursor::new(&proof.data[i].bytes);
            cursor.set_position(32);
            let (mut verified, rz0, rz1, r_simd, _r_mpi, claimed_v0, claimed_v1) = gkr_verify(
                &MPIConfig::new(),
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
}

fn pcs_testing_setup_fixed_seed<C: Config>(
    vals: &[C::DefaultSimdField],
) -> (
    RawExpanderGKRParams,
    PCSEmptyType,
    PCSEmptyType,
    RawExpanderGKRScratchPad,
) {
    // We don't have an interface for the potential pcs setup
    // So we're just going to use the testing setup with fixed seed
    let mut rng = StdRng::from_seed([0; 32]);
    expander_pcs_init_testing_only::<
        C::DefaultGKRFieldConfig,
        <C::DefaultGKRConfig as GKRConfig>::Transcript,
        RawExpanderGKR<_, _>,
    >(
        vals.len().trailing_zeros() as usize,
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
    x: &[<C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField],
    x_simd: &[<C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField],
    kernel: &Kernel<C>,
    commitments: &[&ExpanderGKRCommitment<C>],
    is_broadcast: &[bool],
    parallel_index: usize,
    transcript: &mut <C::DefaultGKRConfig as GKRConfig>::Transcript,
) {
    for ((input, commitment), ib) in kernel
        .layered_circuit_input
        .iter()
        .zip(commitments.iter())
        .zip(is_broadcast)
    {
        let nb_challenge_vars = input.len.trailing_zeros() as usize;
        let challenge_vars = &x[..nb_challenge_vars];

        let vals = if *ib {
            commitment.vals_ref()
        } else {
            &commitment.vals_ref()[parallel_index * input.len..(parallel_index + 1) * input.len]
        };

        let (params, p_key, _v_key, _) = pcs_testing_setup_fixed_seed::<C>(vals);

        let poly = MultiLinearPoly::new(vals.to_vec());
        let v = RawExpanderGKR::<
            C::DefaultGKRFieldConfig,
            <C::DefaultGKRConfig as GKRConfig>::Transcript,
        >::eval(vals, challenge_vars, x_simd, &[]);
        transcript.append_field_element(&v);

        let mut scratch_pad = <RawExpanderGKR<
            C::DefaultGKRFieldConfig,
            <C::DefaultGKRConfig as GKRConfig>::Transcript,
        > as PCSForExpanderGKR<_, _>>::ScratchPad::default();
        transcript.lock_proof();
        let opening = RawExpanderGKR::open(
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
            &mut scratch_pad,
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
    x: &[<C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField],
    x_simd: &[<C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField],
    y: &<C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField,
    kernel: &Kernel<C>,
    commitments: &[&ExpanderGKRCommitment<C>],
    is_broadcast: &[bool],
    parallel_index: usize,
    transcript: &mut <C::DefaultGKRConfig as GKRConfig>::Transcript,
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

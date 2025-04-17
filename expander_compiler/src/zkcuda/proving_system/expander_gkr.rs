/*
use std::collections::HashMap;
use std::io::{Cursor, Read};

use crate::circuit::config::Config;

use super::super::kernel::Kernel;
use super::{check_inputs, prepare_inputs, Commitment, Proof, ProvingSystem};

use arith::Field;
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

#[allow(clippy::type_complexity)]
pub struct ExpanderGKRCommitment<C: Config> {
    vals_len: usize,
    commitment: Vec<<pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::Commitment>,
}

impl<C: Config> Clone for ExpanderGKRCommitment<C> {
    fn clone(&self) -> Self {
        Self {
            vals_len: self.vals_len,
            commitment: self.commitment.clone(),
        }
    }
}

impl<C: Config> ExpSerde for ExpanderGKRCommitment<C> {
    const SERIALIZED_SIZE: usize = unimplemented!();
    fn serialize_into<W: std::io::Write>(&self, mut writer: W) -> serdes::SerdeResult<()> {
        self.vals_len.serialize_into(&mut writer)?;
        self.commitment.serialize_into(&mut writer)
    }
    fn deserialize_from<R: std::io::Read>(mut reader: R) -> serdes::SerdeResult<Self> {
        let vals_len = usize::deserialize_from(&mut reader)?;
        let commitment = Vec::<
            <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::Commitment,
        >::deserialize_from(&mut reader)?;
        Ok(ExpanderGKRCommitment {
            vals_len,
            commitment,
        })
    }
}

impl<C: Config> Commitment<C> for ExpanderGKRCommitment<C> {
    fn vals_len(&self) -> usize {
        self.vals_len
    }
}

#[allow(clippy::type_complexity)]
pub struct ExpanderGKRCommitmentExtraInfo<C: Config> {
    scratch: Vec<<pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::ScratchPad>,
}

impl<C: Config> Clone for ExpanderGKRCommitmentExtraInfo<C> {
    fn clone(&self) -> Self {
        Self {
            scratch: self.scratch.clone(),
        }
    }
}

#[allow(clippy::type_complexity)]
pub struct ExpanderGKRProverSetup<C: Config>
{
    p_keys: HashMap<usize, <<pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::SRS as StructuredReferenceString>::PKey>,
}

impl<C: Config> Clone for ExpanderGKRProverSetup<C> {
    fn clone(&self) -> Self {
        Self {
            p_keys: self.p_keys.clone(),
        }
    }
}

#[allow(clippy::type_complexity)]
pub struct ExpanderGKRVerifierSetup<C: Config>
{
    v_keys: HashMap<usize, <<pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::SRS as StructuredReferenceString>::VKey>,
}

impl<C: Config> Clone for ExpanderGKRVerifierSetup<C> {
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

impl ExpSerde for ExpanderGKRProof {
    const SERIALIZED_SIZE: usize = unimplemented!();
    fn serialize_into<W: std::io::Write>(&self, mut writer: W) -> serdes::SerdeResult<()> {
        self.data.serialize_into(&mut writer)
    }
    fn deserialize_from<R: std::io::Read>(mut reader: R) -> serdes::SerdeResult<Self> {
        let data = Vec::<ExpanderProof>::deserialize_from(&mut reader)?;
        Ok(ExpanderGKRProof { data })
    }
}

impl Proof for ExpanderGKRProof {}

pub struct ExpanderGKRProvingSystem<C: Config> {
    _config: std::marker::PhantomData<C>,
}

impl<C: Config> ProvingSystem<C> for ExpanderGKRProvingSystem<C> {
    type ProverSetup = ExpanderGKRProverSetup<C>;
    type VerifierSetup = ExpanderGKRVerifierSetup<C>;
    type Proof = ExpanderGKRProof;
    type Commitment = ExpanderGKRCommitment<C>;
    type CommitmentExtraInfo = ExpanderGKRCommitmentExtraInfo<C>;

    fn setup(
        computation_graph: &crate::zkcuda::proof::ComputationGraph<C>,
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
                let (_params, p_key, v_key, _scratch) =
                    pcs_testing_setup_fixed_seed::<field!(C), transcript!(C), pcs!(C)>(
                        val_actual_len,
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
        vals: &[C::DefaultSimdField],
        parallel_count: usize,
        is_broadcast: bool,
    ) -> (Self::Commitment, Self::CommitmentExtraInfo) {
        let vals_to_commit = if is_broadcast {
            vec![vals]
        } else {
            vals.chunks(vals.len() / parallel_count).collect::<Vec<_>>()
        };

        let n_vars = vals_to_commit[0].len().ilog2() as usize;
        let params = <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::gen_params(n_vars);
        let p_key = prover_setup.p_keys.get(&(1 << n_vars)).unwrap();

        let (commitment, scratch) = vals_to_commit
            .into_iter()
            .map(|vals| {
                let mut scratch =
                    <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::init_scratch_pad(
                        &params,
                        &MPIConfig::default(),
                    );

                let commitment = <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::commit(
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
        kernel: &Kernel<C>,
        _commitments: &[Self::Commitment],
        commitments_extra_info: &[Self::CommitmentExtraInfo],
        commitments_values: &[&[<C as Config>::DefaultSimdField]],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> Self::Proof {
        check_inputs(kernel, commitments_values, parallel_count, is_broadcast);

        let mut expander_circuit = kernel
            .layered_circuit
            .export_to_expander()
            .flatten::<C::DefaultGKRConfig>();
        expander_circuit.pre_process_gkr::<C::DefaultGKRConfig>();
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
                &MPIConfig::default(),
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
                parallel_count,
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
                    parallel_count,
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
        commitments: &[Self::Commitment],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> bool {
        let mut expander_circuit = kernel
            .layered_circuit
            .export_to_expander()
            .flatten::<C::DefaultGKRConfig>();
        expander_circuit.pre_process_gkr::<C::DefaultGKRConfig>();

        for i in 0..parallel_count {
            let mut transcript = <C::DefaultGKRConfig as GKRConfig>::Transcript::new();
            transcript.append_u8_slice(&[0u8; 32]);
            expander_circuit.fill_rnd_coefs(&mut transcript);

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

            if !verified {
                println!("Failed to verify GKR proof for parallel index {}", i);
                return false;
            }

            verified &= verify_input_claim(
                &mut cursor,
                kernel,
                verifier_setup,
                &rz0,
                &r_simd,
                &claimed_v0,
                commitments,
                is_broadcast,
                i,
                parallel_count,
                &mut transcript,
            );
            if let Some(rz1) = rz1 {
                verified &= verify_input_claim(
                    &mut cursor,
                    kernel,
                    verifier_setup,
                    &rz1,
                    &r_simd,
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
        true
    }
}

#[allow(clippy::type_complexity)]
fn pcs_testing_setup_fixed_seed<
    FConfig: GKRFieldConfig,
    T: Transcript<FConfig::ChallengeField>,
    PCS: PCSForExpanderGKR<FConfig, T>,
>(
    vals_len: usize,
) -> (
    PCS::Params,
    <PCS::SRS as StructuredReferenceString>::PKey,
    <PCS::SRS as StructuredReferenceString>::VKey,
    PCS::ScratchPad,
) {
    let mut rng = StdRng::from_seed([123; 32]);
    expander_pcs_init_testing_only::<FConfig, T, PCS>(
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

#[allow(clippy::too_many_arguments)]
fn prove_input_claim<C: Config>(
    _kernel: &Kernel<C>,
    commitments_values: &[&[C::DefaultSimdField]],
    p_keys: &ExpanderGKRProverSetup<C>,
    commitments_extra_info: &[ExpanderGKRCommitmentExtraInfo<C>],
    x: &[<field!(C) as GKRFieldConfig>::ChallengeField],
    x_simd: &[<field!(C) as GKRFieldConfig>::ChallengeField],
    is_broadcast: &[bool],
    parallel_index: usize,
    parallel_count: usize,
    transcript: &mut transcript!(C),
) {
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

        // TODO: Remove unnecessary `to_vec` clone
        let poly = MultiLinearPoly::new(vals_to_open.to_vec());
        let v = MultiLinearPolyExpander::<field!(C)>::single_core_eval_circuit_vals_at_expander_challenge(
            vals_to_open,
            &challenge_vars,
            x_simd,
            &[],
        );
        transcript.append_field_element(&v);

        transcript.lock_proof();
        let scratch_for_parallel_index = if *ib {
            &extra_info.scratch[0]
        } else {
            &extra_info.scratch[parallel_index]
        };
        let opening = <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::open(
            &params,
            &MPIConfig::default(),
            p_key,
            &poly,
            &ExpanderGKRChallenge::<C::DefaultGKRFieldConfig> {
                x: challenge_vars.to_vec(),
                x_simd: x_simd.to_vec(),
                x_mpi: vec![],
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
fn verify_input_claim<C: Config>(
    mut proof_reader: impl Read,
    kernel: &Kernel<C>,
    v_keys: &ExpanderGKRVerifierSetup<C>,
    x: &[<C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField],
    x_simd: &[<C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField],
    y: &<C::DefaultGKRFieldConfig as GKRFieldConfig>::ChallengeField,
    commitments: &[ExpanderGKRCommitment<C>],
    is_broadcast: &[bool],
    parallel_index: usize,
    _parallel_count: usize,
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
        let commitment_for_parallel_index = if *ib {
            &commitment.commitment[0]
        } else {
            &commitment.commitment[parallel_index]
        };
        let verified = <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::verify(
            &params,
            v_key,
            commitment_for_parallel_index,
            &ExpanderGKRChallenge::<C::DefaultGKRFieldConfig> {
                x: challenge_vars,
                x_simd: x_simd.to_vec(),
                x_mpi: vec![],
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

    *y == target_y
}
*/

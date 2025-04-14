use std::collections::HashMap;
use std::io::{Cursor, Read};

use crate::circuit::config::Config;

use super::super::kernel::Kernel;
use super::{check_inputs, pcs_testing_setup_fixed_seed, prepare_inputs, Commitment, ExpanderGKRProvingSystem, Proof, ProvingSystem};
use super::expander_gkr::{ExpanderGKRCommitment, ExpanderGKRCommitmentExtraInfo, ExpanderGKRProof, ExpanderGKRProverSetup, ExpanderGKRVerifierSetup};

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

#[derive(Default)]
pub struct SharedMemory {
    pub pcs_setup: Option<Shmem>,
    pub input_vals: Option<Shmem>,
    pub commitment: Option<Shmem>,
    pub extra_info: Option<Shmem>,
}

static mut SHARED_MEMORY: SharedMemory = SharedMemory {
    pcs_setup: None,
    input_vals: None,
    commitment: None,
    extra_info: None,
};

fn init_commitment_and_extra_info_shared_memory<C: Config>(commitment_size: usize, extra_info_size: usize) {
    if unsafe { SHARED_MEMORY.commitment.is_some() } {
        return;
    }

    unsafe {
        SHARED_MEMORY.commitment = Some(
            ShmemConf::new()
                .size(commitment_size)
                .flink("commitment")
                .create()
                .unwrap(),
        );
        SHARED_MEMORY.extra_info = Some(
            ShmemConf::new()
                .size(extra_info_size)
                .flink("extra_info")
                .create()
                .unwrap(),
        );
    }
}

fn write_object_to_shared_memory<T: ExpSerde>(object: &T, shared_memory_ref: &mut Option<Shmem>, name: &str) {
    let mut buffer = vec![];
    object
        .serialize_into(&mut buffer)
        .expect("Failed to serialize object");

    unsafe {
        *shared_memory_ref = Some(
            ShmemConf::new()
                .size(buffer.len())
                .flink(name)
                .create()
                .unwrap(),
        );

        let object_ptr = shared_memory_ref.as_mut().unwrap().as_ptr() as *mut u8;
        std::ptr::copy_nonoverlapping(buffer.as_ptr(), object_ptr, buffer.len());
    }
}

fn write_pcs_setup_to_shared_memory<C: Config>(
    pcs_setup: &ExpanderGKRProverSetup<C>,
    actual_local_len: usize,
) {
    let setup = pcs_setup.p_keys.get(&actual_local_len).unwrap();
    write_object_to_shared_memory(setup, unsafe {&mut SHARED_MEMORY.pcs_setup}, "pcs_setup");
}

fn write_vals_to_shared_memory<C: Config>(vals: &Vec<C::DefaultSimdField>) {
    write_object_to_shared_memory(vals, unsafe {&mut SHARED_MEMORY.input_vals}, "input_vals");
}

// TODO: Is this a little dangerous to allow arbitrary cmd strings?
fn exec_command(cmd: &str) {
    let mut parts = cmd.split_whitespace();
    let command = parts.next().unwrap();
    let args: Vec<&str> = parts.collect();
    let mut child = Command::new(command)
        .args(args)
        .spawn()
        .expect("Failed to start child process");
    let _ = child.wait();
}


fn exec_pcs_commit(mpi_size: usize) {
    let cmd_str = format!(
        "mpiexec -n {} ./target/release/pcs_commit",
        mpi_size
    );
    exec_command(&cmd_str);
}

fn read_object_from_shared_memory<T: ExpSerde>(shared_memory_ref: &mut Option<Shmem>) -> T {
    let shmem = shared_memory_ref.take().unwrap();
    let object_ptr = shmem.as_ptr() as *const u8;
    let object_len = shmem.len();
    let mut buffer = vec![0u8; object_len];
    unsafe {
        std::ptr::copy_nonoverlapping(object_ptr, buffer.as_mut_ptr(), object_len);
    }
    T::deserialize_from(&mut Cursor::new(buffer)).unwrap()
}

fn read_commitment_and_extra_info_from_shared_memory<C: Config>() -> (ExpanderGKRCommitment<C>, ExpanderGKRCommitmentExtraInfo<C>) {
    let commitment = read_object_from_shared_memory(unsafe {&mut SHARED_MEMORY.commitment});
    let scratch = read_object_from_shared_memory(unsafe {&mut SHARED_MEMORY.extra_info});
    let extra_info = ExpanderGKRCommitmentExtraInfo {
        scratch: vec![scratch],
    };
    (commitment, extra_info)
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
            write_vals_to_shared_memory::<C>(&vals.to_vec());
            exec_pcs_commit(parallel_count);
            read_commitment_and_extra_info_from_shared_memory()
        }
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

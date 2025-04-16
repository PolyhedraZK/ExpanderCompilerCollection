use std::{io::Cursor, process::Command};

use expander_circuit::Circuit;
use serdes::ExpSerde;
use shared_memory::{Shmem, ShmemConf};

use crate::circuit::config::Config;

use super::{ExpanderGKRCommitment, ExpanderGKRCommitmentExtraInfo, ExpanderGKRProof, ExpanderGKRProverSetup};


#[derive(Default)]
pub struct SharedMemory {
    pub pcs_setup: Option<Shmem>,
    pub input_vals: Option<Shmem>,
    pub commitment: Option<Shmem>,
    pub extra_info: Option<Shmem>,
    pub proof: Option<Shmem>,
}

pub static mut SHARED_MEMORY: SharedMemory = SharedMemory {
    pcs_setup: None,
    input_vals: None,
    commitment: None,
    extra_info: None,
    proof: None,
};

pub fn init_commitment_and_extra_info_shared_memory<C: Config>(commitment_size: usize, extra_info_size: usize) {
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
        if shared_memory_ref.is_some() {
            *shared_memory_ref = None;
        }
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

pub fn write_pcs_setup_to_shared_memory<C: Config>(
    pcs_setup: &ExpanderGKRProverSetup<C>,
    actual_local_len: usize,
) {
    let setup = pcs_setup.p_keys.get(&actual_local_len).unwrap();
    write_object_to_shared_memory(&(actual_local_len, setup.clone()), unsafe {&mut SHARED_MEMORY.pcs_setup}, "pcs_setup");
}

pub fn write_commit_vals_to_shared_memory<C: Config>(vals: &Vec<C::DefaultSimdField>) {
    write_object_to_shared_memory(vals, unsafe {&mut SHARED_MEMORY.input_vals}, "input_vals");
}

// I think we have ExpSerde implemented for Circuit in the latest version of expander circuit
pub fn write_circuit_to_shared_memory<C: Config>(circuit: &Circuit<C::DefaultGKRFieldConfig>) {
    write_object_to_shared_memory(circuit, unsafe {&mut SHARED_MEMORY.pcs_setup}, "circuit");
}

pub fn write_proving_inputs_to_shared_memory<C: Config>(inputs: &Vec<Vec<C::DefaultSimdField>>) {
    write_object_to_shared_memory(inputs, unsafe {&mut SHARED_MEMORY.input_vals}, "inputs_vals");
}

// TODO: Is it a little dangerous to allow arbitrary cmd strings?
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

pub fn exec_pcs_commit(mpi_size: usize) {
    let cmd_str = format!(
        "mpiexec -n {} ./target/release/pcs_commit",
        mpi_size
    );
    exec_command(&cmd_str);
}

pub fn exec_gkr_prove_with_pcs(mpi_size: usize) {
    let cmd_str = format!(
        "mpiexec -n {} ./target/release/gkr_prove_with_pcs",
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

pub fn read_commitment_and_extra_info_from_shared_memory<C: Config>() -> (ExpanderGKRCommitment<C>, ExpanderGKRCommitmentExtraInfo<C>) {
    let commitment = read_object_from_shared_memory(unsafe {&mut SHARED_MEMORY.commitment});
    let scratch = read_object_from_shared_memory(unsafe {&mut SHARED_MEMORY.extra_info});
    let extra_info = ExpanderGKRCommitmentExtraInfo {
        scratch: vec![scratch],
    };
    (commitment, extra_info)
}

pub fn read_proof_from_shared_memory<C: Config>() -> ExpanderGKRProof {
    let proof = read_object_from_shared_memory(unsafe {&mut SHARED_MEMORY.proof});
    ExpanderGKRProof { data: vec![proof] }
}
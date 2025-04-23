use std::{io::Cursor, process::Command};

use crate::{circuit::layered::Circuit, zkcuda::kernel::LayeredCircuitInputVec};
use serdes::ExpSerde;
use shared_memory::{Shmem, ShmemConf};

use crate::circuit::{config::Config, layered::InputType};

use super::{
    ExpanderGKRCommitment, ExpanderGKRCommitmentExtraInfo, ExpanderGKRProof, ExpanderGKRProverSetup,
};

#[derive(Default)]
pub struct SharedMemory {
    pub pcs_setup: Option<Shmem>,
    pub circuit: Option<Shmem>,
    pub input_partition: Option<Shmem>,
    pub input_vals: Option<Shmem>,
    pub commitment: Option<Shmem>,
    pub extra_info: Option<Shmem>,
    pub broadcast_info: Option<Shmem>,
    pub proof: Option<Shmem>,
}

pub static mut SHARED_MEMORY: SharedMemory = SharedMemory {
    pcs_setup: None,
    circuit: None,
    input_partition: None,
    input_vals: None,
    commitment: None,
    extra_info: None,
    broadcast_info: None,
    proof: None,
};

pub fn init_commitment_and_extra_info_shared_memory<C: Config>(
    commitment_size: usize,
    extra_info_size: usize,
) {
    if unsafe { SHARED_MEMORY.commitment.is_some() && SHARED_MEMORY.extra_info.is_some() } {
        // TODO: Check if the sizes suffice
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

pub fn init_proof_shared_memory(max_proof_size: usize) {
    if unsafe { SHARED_MEMORY.proof.is_some() } {
        return;
    }

    unsafe {
        SHARED_MEMORY.proof = Some(
            ShmemConf::new()
                .size(max_proof_size)
                .flink("proof")
                .create()
                .unwrap(),
        );
    }
}

fn write_object_to_shared_memory<T: ExpSerde>(
    object: &T,
    shared_memory_ref: &mut Option<Shmem>,
    name: &str,
) {
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

pub fn write_selected_pkey_to_shared_memory<C: Config>(
    pcs_setup: &ExpanderGKRProverSetup<C>,
    actual_local_len: usize,
) {
    let setup = pcs_setup.p_keys.get(&actual_local_len).unwrap().clone();
    let pair = (actual_local_len, setup);

    write_object_to_shared_memory(&pair, unsafe { &mut SHARED_MEMORY.pcs_setup }, "pcs_setup");
}

pub fn write_commit_vals_to_shared_memory<C: Config>(vals: &Vec<C::DefaultSimdField>) {
    write_object_to_shared_memory(vals, unsafe { &mut SHARED_MEMORY.input_vals }, "input_vals");
}

pub fn write_pcs_setup_to_shared_memory<C: Config>(pcs_setup: &ExpanderGKRProverSetup<C>) {
    write_object_to_shared_memory(
        pcs_setup,
        unsafe { &mut SHARED_MEMORY.pcs_setup },
        "pcs_setup",
    );
}

// I think we have ExpSerde implemented for Circuit in the latest version of expander circuit
// pub fn write_circuit_to_shared_memory<C: Config>(circuit: &Circuit<C::DefaultGKRFieldConfig>) {
//     write_object_to_shared_memory(circuit, unsafe {&mut SHARED_MEMORY.pcs_setup}, "circuit");
// }
pub fn write_ecc_circuit_to_shared_memory<C: Config, I: InputType>(ecc_circuit: &Circuit<C, I>) {
    write_object_to_shared_memory(
        ecc_circuit,
        unsafe { &mut SHARED_MEMORY.circuit },
        "ecc_circuit",
    );
}

pub fn write_input_partition_info_to_shared_memory(input_partition: &Vec<LayeredCircuitInputVec>) {
    write_object_to_shared_memory(
        input_partition,
        unsafe { &mut SHARED_MEMORY.input_partition },
        "input_partition",
    );
}

pub fn write_commitments_to_shared_memory<C: Config>(commitments: &Vec<ExpanderGKRCommitment<C>>) {
    write_object_to_shared_memory(
        commitments,
        unsafe { &mut SHARED_MEMORY.commitment },
        "commitments",
    );
}

pub fn write_commitments_extra_info_to_shared_memory<C: Config>(
    commitments_extra_info: &Vec<ExpanderGKRCommitmentExtraInfo<C>>,
) {
    write_object_to_shared_memory(
        commitments_extra_info,
        unsafe { &mut SHARED_MEMORY.extra_info },
        "commitments_extra_info",
    );
}

pub fn write_commitments_values_to_shared_memory<C: Config>(
    commitments_values: &Vec<&[C::DefaultSimdField]>,
) {
    let commitments_values = commitments_values
        .iter()
        .map(|&commitment| commitment.to_vec())
        .collect::<Vec<_>>();
    write_object_to_shared_memory(
        &commitments_values,
        unsafe { &mut SHARED_MEMORY.input_vals },
        "commitments_values",
    );
}

pub fn write_broadcast_info_to_shared_memory(is_broadcast: &Vec<bool>) {
    write_object_to_shared_memory(
        is_broadcast,
        unsafe { &mut SHARED_MEMORY.broadcast_info },
        "is_broadcast",
    );
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
    let cmd_str = format!("mpiexec -n {} ./target/release/pcs_commit", mpi_size);
    exec_command(&cmd_str);
}

pub fn exec_gkr_prove_with_pcs(mpi_size: usize) {
    let cmd_str = format!(
        "mpiexec -n {} ./target/release/gkr_prove_with_pcs",
        mpi_size
    );
    exec_command(&cmd_str);
}

pub fn read_object_from_shared_memory<T: ExpSerde>(shared_memory_ref: &mut Option<Shmem>) -> T {
    let shmem = shared_memory_ref.take().unwrap();
    let object_ptr = shmem.as_ptr() as *const u8;
    let object_len = shmem.len();
    let mut buffer = vec![0u8; object_len];
    unsafe {
        std::ptr::copy_nonoverlapping(object_ptr, buffer.as_mut_ptr(), object_len);
    }
    T::deserialize_from(&mut Cursor::new(buffer)).unwrap()
}

pub fn read_commitment_and_extra_info_from_shared_memory<C: Config>(
) -> (ExpanderGKRCommitment<C>, ExpanderGKRCommitmentExtraInfo<C>) {
    let commitment = read_object_from_shared_memory(unsafe { &mut SHARED_MEMORY.commitment });
    let scratch = read_object_from_shared_memory(unsafe { &mut SHARED_MEMORY.extra_info });
    let extra_info = ExpanderGKRCommitmentExtraInfo {
        scratch: vec![scratch],
    };
    (commitment, extra_info)
}

pub fn read_proof_from_shared_memory<C: Config>() -> ExpanderGKRProof {
    let proof = read_object_from_shared_memory(unsafe { &mut SHARED_MEMORY.proof });
    ExpanderGKRProof { data: vec![proof] }
}

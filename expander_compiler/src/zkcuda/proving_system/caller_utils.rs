#![allow(static_mut_refs)]

use std::process::Command;

use crate::{
    circuit::layered::Circuit, frontend::SIMDField, zkcuda::kernel::LayeredCircuitInputVec,
};
use gkr_engine::{ExpanderPCS, FieldEngine, FieldType, GKREngine, PolynomialCommitmentType};
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

pub fn init_commitment_and_extra_info_shared_memory(
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
                .flink("/tmp/commitment")
                .force_create_flink()
                .create()
                .unwrap(),
        );
        SHARED_MEMORY.extra_info = Some(
            ShmemConf::new()
                .size(extra_info_size)
                .flink("/tmp/extra_info")
                .force_create_flink()
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
                .flink("/tmp/proof")
                .force_create_flink()
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
        if shared_memory_ref.is_some() && shared_memory_ref.as_ref().unwrap().len() >= buffer.len()
        {
        } else {
            *shared_memory_ref = None;
            *shared_memory_ref = Some(
                ShmemConf::new()
                    .size(buffer.len())
                    .flink(name)
                    .force_create_flink()
                    .create()
                    .unwrap(),
            );
        }
        let object_ptr = shared_memory_ref.as_mut().unwrap().as_ptr();
        std::ptr::copy_nonoverlapping(buffer.as_ptr(), object_ptr, buffer.len());
    }
}

pub fn write_selected_pkey_to_shared_memory<F: FieldEngine, PCS: ExpanderPCS<F>>(
    pcs_setup: &ExpanderGKRProverSetup<F, PCS>,
    actual_local_len: usize,
) {
    let setup = pcs_setup.p_keys.get(&actual_local_len).unwrap().clone();
    let pair = (actual_local_len, setup);

    write_object_to_shared_memory(
        &pair,
        unsafe { &mut SHARED_MEMORY.pcs_setup },
        "/tmp/pcs_setup",
    );
}

pub fn write_commit_vals_to_shared_memory<C: Config>(vals: &Vec<SIMDField<C>>) {
    write_object_to_shared_memory(
        vals,
        unsafe { &mut SHARED_MEMORY.input_vals },
        "/tmp/input_vals",
    );
}

pub fn write_pcs_setup_to_shared_memory<F: FieldEngine, PCS: ExpanderPCS<F>>(
    pcs_setup: &ExpanderGKRProverSetup<F, PCS>,
) {
    write_object_to_shared_memory(
        pcs_setup,
        unsafe { &mut SHARED_MEMORY.pcs_setup },
        "/tmp/pcs_setup",
    );
}

// I think we have ExpSerde implemented for Circuit in the latest version of expander circuit
// pub fn write_circuit_to_shared_memory<C: Config>(circuit: &Circuit<C::DefaultGKRFieldConfig>) {
//     write_object_to_shared_memory(circuit, unsafe {&mut SHARED_MEMORY.pcs_setup}, "/tmp/circuit");
// }
pub fn write_ecc_circuit_to_shared_memory<C: Config, I: InputType>(ecc_circuit: &Circuit<C, I>) {
    write_object_to_shared_memory(
        ecc_circuit,
        unsafe { &mut SHARED_MEMORY.circuit },
        "/tmp/circuit",
    );
}

pub fn write_input_partition_info_to_shared_memory(input_partition: &Vec<LayeredCircuitInputVec>) {
    write_object_to_shared_memory(
        input_partition,
        unsafe { &mut SHARED_MEMORY.input_partition },
        "/tmp/input_partition",
    );
}

pub fn write_commitments_to_shared_memory<F: FieldEngine, PCS: ExpanderPCS<F>>(
    commitments: &Vec<ExpanderGKRCommitment<F, PCS>>,
) {
    write_object_to_shared_memory(
        commitments,
        unsafe { &mut SHARED_MEMORY.commitment },
        "/tmp/commitment",
    );
}

pub fn write_commitments_extra_info_to_shared_memory<F: FieldEngine, PCS: ExpanderPCS<F>>(
    commitments_extra_info: &Vec<ExpanderGKRCommitmentExtraInfo<F, PCS>>,
) {
    write_object_to_shared_memory(
        commitments_extra_info,
        unsafe { &mut SHARED_MEMORY.extra_info },
        "/tmp/extra_info",
    );
}

pub fn write_commitments_values_to_shared_memory<F: FieldEngine>(
    commitments_values: &[&[F::SimdCircuitField]],
) {
    let commitments_values = commitments_values
        .iter()
        .map(|&commitment| commitment.to_vec())
        .collect::<Vec<_>>();
    write_object_to_shared_memory(
        &commitments_values,
        unsafe { &mut SHARED_MEMORY.input_vals },
        "/tmp/input_vals",
    );
}

pub fn write_broadcast_info_to_shared_memory(is_broadcast: &Vec<bool>) {
    write_object_to_shared_memory(
        is_broadcast,
        unsafe { &mut SHARED_MEMORY.broadcast_info },
        "/tmp/broadcast_info",
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

fn parse_config<C: GKREngine>(mpi_size: usize) -> (String, String, String) {
    let oversubscription = if mpi_size > num_cpus::get_physical() {
        println!("Warning: Not enough cores available for the requested number of processes. Using oversubscription.");
        "--oversubscribe"
    } else {
        ""
    };

    let field_name = match <C::FieldConfig as FieldEngine>::FIELD_TYPE {
        FieldType::M31x16 => "M31",
        FieldType::GF2Ext128 => "GF2",
        FieldType::Goldilocksx8 => "Goldilocks",
        FieldType::BabyBearx16 => "BabyBear",
        FieldType::BN254 => "BN254",
        _ => panic!("Unsupported field type"),
    };

    let pcs_name = match <C::PCSConfig as ExpanderPCS<C::FieldConfig>>::PCS_TYPE {
        PolynomialCommitmentType::Raw => "Raw",
        PolynomialCommitmentType::Hyrax => "Hyrax",
        _ => panic!("Unsupported PCS type"),
    };

    (
        oversubscription.to_string(),
        field_name.to_string(),
        pcs_name.to_string(),
    )
}

pub fn exec_pcs_commit<C: GKREngine>(mpi_size: usize) {
    let (oversubscription, field_name, pcs_name) = parse_config::<C>(mpi_size);

    let cmd_str = format!(
        "mpiexec -n {} {} ../target/release/expander_commit --field-type {} --poly-commit {}",
        mpi_size, oversubscription, field_name, pcs_name,
    );
    exec_command(&cmd_str);
}

pub fn exec_gkr_prove_with_pcs<C: GKREngine>(mpi_size: usize) {
    let (oversubscription, field_name, pcs_name) = parse_config::<C>(mpi_size);

    let cmd_str = format!(
        "mpiexec -n {} {} ../target/release/expander_prove --field-type {} --poly-commit {}",
        mpi_size, oversubscription, field_name, pcs_name,
    );
    exec_command(&cmd_str);
}

pub fn read_object_from_shared_memory<T: ExpSerde>(
    shared_memory_ref: &Option<Shmem>,
    offset: usize,
) -> T {
    let shmem = shared_memory_ref.as_ref().unwrap();
    let object_ptr = shmem.as_ptr() as *const u8;
    let object_len = shmem.len();
    let buffer = unsafe { std::slice::from_raw_parts(object_ptr.add(offset), object_len - offset) };
    T::deserialize_from(buffer).unwrap()
}

pub fn read_commitment_and_extra_info_from_shared_memory<F: FieldEngine, PCS: ExpanderPCS<F>>() -> (
    ExpanderGKRCommitment<F, PCS>,
    ExpanderGKRCommitmentExtraInfo<F, PCS>,
) {
    let commitment = read_object_from_shared_memory(unsafe { &SHARED_MEMORY.commitment }, 0);
    let extra_info = read_object_from_shared_memory(unsafe { &SHARED_MEMORY.extra_info }, 0);
    (commitment, extra_info)
}

pub fn read_proof_from_shared_memory() -> ExpanderGKRProof {
    read_object_from_shared_memory(unsafe { &SHARED_MEMORY.proof }, 0)
}

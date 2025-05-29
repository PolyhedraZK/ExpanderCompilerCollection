#![allow(static_mut_refs)]

use std::process::Command;

use crate::{
    circuit::layered::Circuit, frontend::SIMDField, zkcuda::kernel::LayeredCircuitInputVec,
};
use arith::Field;
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

unsafe fn allocate_shared_memory(handle: &mut Option<Shmem>, name: &str, target_size: usize) {
    if handle.is_some() && handle.as_ref().unwrap().len() >= target_size {
        return;
    }
    *handle = None;
    *handle = Some(
        ShmemConf::new()
            .size(target_size)
            .flink(name)
            .force_create_flink()
            .create()
            .unwrap(),
    );
}

pub fn init_commitment_and_extra_info_shared_memory(
    commitment_size: usize,
    extra_info_size: usize,
) {
    unsafe {
        allocate_shared_memory(
            &mut SHARED_MEMORY.commitment,
            "/tmp/commitment",
            commitment_size,
        );
        allocate_shared_memory(
            &mut SHARED_MEMORY.extra_info,
            "/tmp/extra_info",
            extra_info_size,
        );
    }
}

pub fn init_proof_shared_memory(max_proof_size: usize) {
    unsafe {
        allocate_shared_memory(&mut SHARED_MEMORY.proof, "/tmp/proof", max_proof_size);
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

pub fn write_selected_pkey_to_shared_memory<
    PCSField: Field,
    F: FieldEngine,
    PCS: ExpanderPCS<F, PCSField>,
>(
    pcs_setup: &ExpanderGKRProverSetup<PCSField, F, PCS>,
    actual_local_len: usize,
) {
    let setup = pcs_setup
        .p_keys
        .get(&(actual_local_len, 1))
        .unwrap()
        .clone();
    let pair = (actual_local_len, setup);

    write_object_to_shared_memory(
        &pair,
        unsafe { &mut SHARED_MEMORY.pcs_setup },
        "/tmp/pcs_setup",
    );
}

pub fn write_commit_vals_to_shared_memory<C: Config>(vals: &[SIMDField<C>]) {
    // Field implements Copy, so we can just copy the data
    // The first usize is the length of the vector
    let vals_size = std::mem::size_of_val(vals);
    let total_size = std::mem::size_of::<usize>() + vals_size;
    unsafe {
        allocate_shared_memory(&mut SHARED_MEMORY.input_vals, "/tmp/input_vals", total_size);

        let mut ptr = SHARED_MEMORY.input_vals.as_mut().unwrap().as_ptr();

        // Copy the length of the vector
        let len = vals.len();
        let len_ptr = &len as *const usize as *const u8;
        std::ptr::copy_nonoverlapping(len_ptr, ptr, std::mem::size_of::<usize>());

        // Copy the values
        ptr = ptr.add(std::mem::size_of::<usize>());
        std::ptr::copy_nonoverlapping(vals.as_ptr() as *const u8, ptr, vals_size);
    }
}

pub fn write_pcs_setup_to_shared_memory<
    PCSField: Field,
    F: FieldEngine,
    PCS: ExpanderPCS<F, PCSField>,
>(
    pcs_setup: &ExpanderGKRProverSetup<PCSField, F, PCS>,
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

pub fn write_commitments_to_shared_memory<
    PCSField: Field,
    F: FieldEngine,
    PCS: ExpanderPCS<F, PCSField>,
>(
    commitments: &Vec<ExpanderGKRCommitment<PCSField, F, PCS>>,
) {
    write_object_to_shared_memory(
        commitments,
        unsafe { &mut SHARED_MEMORY.commitment },
        "/tmp/commitment",
    );
}

pub fn write_commitments_extra_info_to_shared_memory<
    PCSField: Field,
    F: FieldEngine,
    PCS: ExpanderPCS<F, PCSField>,
>(
    commitments_extra_info: &Vec<ExpanderGKRCommitmentExtraInfo<PCSField, F, PCS>>,
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
    let total_size = std::mem::size_of::<usize>()
        + commitments_values
            .iter()
            .map(|v| std::mem::size_of::<usize>() + std::mem::size_of_val(*v))
            .sum::<usize>();

    unsafe {
        allocate_shared_memory(&mut SHARED_MEMORY.input_vals, "/tmp/input_vals", total_size);

        let mut ptr = SHARED_MEMORY.input_vals.as_mut().unwrap().as_ptr();

        // Copy the length of the vector
        let len = commitments_values.len();
        let len_ptr = &len as *const usize as *const u8;
        std::ptr::copy_nonoverlapping(len_ptr, ptr, std::mem::size_of::<usize>());
        ptr = ptr.add(std::mem::size_of::<usize>());

        for vals in commitments_values {
            let vals_size = std::mem::size_of_val(*vals);
            let vals_len = vals.len();
            let len_ptr = &vals_len as *const usize as *const u8;
            std::ptr::copy_nonoverlapping(len_ptr, ptr, std::mem::size_of::<usize>());
            ptr = ptr.add(std::mem::size_of::<usize>());

            std::ptr::copy_nonoverlapping(vals.as_ptr() as *const u8, ptr, vals_size);
            ptr = ptr.add(vals_size);
        }
    }
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

pub fn start_server(max_parallel_count: usize) {
    let cmd_str = format!(
        "mpiexec -n {} ../target/release/expander_serve",
        max_parallel_count
    );
    let mut parts = cmd_str.split_whitespace();
    let command = parts.next().unwrap();
    let args: Vec<&str> = parts.collect();
    let _ = Command::new(command)
        .args(args)
        .spawn()
        .expect("Failed to start child process");
}

fn parse_config<C: GKREngine>(mpi_size: usize) -> (String, String, String)
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
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

    let pcs_name = match <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::PCS_TYPE {
        PolynomialCommitmentType::Raw => "Raw",
        PolynomialCommitmentType::Hyrax => "Hyrax",
        PolynomialCommitmentType::KZG => "KZG",
        _ => panic!("Unsupported PCS type"),
    };

    (
        oversubscription.to_string(),
        field_name.to_string(),
        pcs_name.to_string(),
    )
}

pub fn exec_pcs_commit<C: GKREngine>(mpi_size: usize)
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let (oversubscription, field_name, pcs_name) = parse_config::<C>(mpi_size);

    let cmd_str = format!(
        "mpiexec -n {} {} ../target/release/expander_commit --field-type {} --poly-commit {}",
        mpi_size, oversubscription, field_name, pcs_name,
    );
    exec_command(&cmd_str);
}

pub fn exec_gkr_prove_with_pcs<C: GKREngine>(mpi_size: usize)
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
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

pub fn read_commitment_and_extra_info_from_shared_memory<
    PCSField: Field,
    F: FieldEngine,
    PCS: ExpanderPCS<F, PCSField>,
>() -> (
    ExpanderGKRCommitment<PCSField, F, PCS>,
    ExpanderGKRCommitmentExtraInfo<PCSField, F, PCS>,
) {
    let commitment = read_object_from_shared_memory(unsafe { &SHARED_MEMORY.commitment }, 0);
    let extra_info = read_object_from_shared_memory(unsafe { &SHARED_MEMORY.extra_info }, 0);
    (commitment, extra_info)
}

pub fn read_proof_from_shared_memory() -> ExpanderGKRProof {
    read_object_from_shared_memory(unsafe { &SHARED_MEMORY.proof }, 0)
}

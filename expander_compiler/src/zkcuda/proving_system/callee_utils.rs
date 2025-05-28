use arith::Field;
use gkr_engine::{ExpanderPCS, FieldEngine, StructuredReferenceString};
use serdes::ExpSerde;
use shared_memory::ShmemConf;

use crate::{
    circuit::{
        config::Config,
        layered::{Circuit, NormalInputType},
    },
    zkcuda::kernel::LayeredCircuitInputVec,
};

pub use super::caller_utils::read_object_from_shared_memory;
use super::{
    ExpanderGKRCommitment, ExpanderGKRCommitmentExtraInfo, ExpanderGKRProof, ExpanderGKRProverSetup,
};

pub fn read_object_from_shared_memory_name_string<T: ExpSerde>(
    shared_memory_ref: &str,
    offset: usize,
) -> T {
    println!("[DEBUG] ====== Starting to open shared memory ======");
    println!("[DEBUG] Attempting to open: {}", shared_memory_ref);
    println!("[DEBUG] Current directory: {:?}", std::env::current_dir());
    
    let shmem = ShmemConf::new()
        .flink(shared_memory_ref)
        .open()
        .map_err(|e| {
            println!("[ERROR] Failed to open shared memory at {}: {}", shared_memory_ref, e);
            println!("[DEBUG] Checking /dev/shm directory contents:");
            if let Ok(entries) = std::fs::read_dir("/dev/shm") {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        println!("[DEBUG] Found file: {} (size: {} bytes)", 
                            entry.path().display(),
                            metadata.len()
                        );
                    }
                }
            }
            println!("[DEBUG] ====== End of shared memory check ======");
            e
        })
        .unwrap();
    println!("[DEBUG] Successfully opened shared memory of size: {} bytes", shmem.len());
    println!("[DEBUG] ====== Finished opening shared memory ======");
    read_object_from_shared_memory(&Some(shmem), offset)
}

#[allow(clippy::type_complexity)]
pub fn read_selected_pkey_from_shared_memory<
    PCSField: Field,
    F: FieldEngine,
    PCS: ExpanderPCS<F, PCSField>,
>() -> (usize, <PCS::SRS as StructuredReferenceString>::PKey) {
    read_object_from_shared_memory_name_string("/home/dream/tmp/pcs_setup", 0)
}

pub fn read_local_vals_to_commit_from_shared_memory<F: FieldEngine>(
    world_rank: usize,
    world_size: usize,
) -> Vec<F::SimdCircuitField> {
    let shmem = ShmemConf::new().flink("/home/dream/tmp/input_vals").open().unwrap();
    let ptr = shmem.as_ptr();
    let total_len: usize =
        usize::deserialize_from(unsafe { std::slice::from_raw_parts(ptr, size_of::<usize>()) })
            .unwrap();

    let local_len = total_len / world_size;
    let offset = size_of::<usize>() + world_rank * local_len * <F::SimdCircuitField as Field>::SIZE;
    let ptr = unsafe { ptr.add(offset) };

    let mut v = Vec::with_capacity(local_len);
    unsafe {
        std::ptr::copy_nonoverlapping(ptr as *const F::SimdCircuitField, v.as_mut_ptr(), local_len);
        v.set_len(local_len);
    }
    v
}

pub fn write_object_to_shared_memory_name_string<T: ExpSerde>(object: &T, shared_memory_ref: &str) {
    let mut buffer = vec![];
    object
        .serialize_into(&mut buffer)
        .expect("Failed to serialize object");

    unsafe {
        let shmem = ShmemConf::new().flink(shared_memory_ref).open().unwrap();
        assert!(
            shmem.len() >= buffer.len(),
            "{}, {}",
            shmem.len(),
            buffer.len()
        );

        let object_ptr = shmem.as_ptr();
        std::ptr::copy_nonoverlapping(buffer.as_ptr(), object_ptr, buffer.len());
    }
}

pub fn write_commitment_to_shared_memory<
    PCSField: Field,
    F: FieldEngine,
    PCS: ExpanderPCS<F, PCSField>,
>(
    commitment: &ExpanderGKRCommitment<PCSField, F, PCS>,
) {
    write_object_to_shared_memory_name_string(commitment, "/home/dream/tmp/commitment");
}

pub fn write_commitment_extra_info_to_shared_memory<
    PCSField: Field,
    F: FieldEngine,
    PCS: ExpanderPCS<F, PCSField>,
>(
    extra_info: &ExpanderGKRCommitmentExtraInfo<PCSField, F, PCS>,
) {
    write_object_to_shared_memory_name_string(extra_info, "/home/dream/tmp/extra_info");
}

pub fn read_pcs_setup_from_shared_memory<
    PCSField: Field,
    F: FieldEngine,
    PCS: ExpanderPCS<F, PCSField>,
>() -> ExpanderGKRProverSetup<PCSField, F, PCS> {
    read_object_from_shared_memory_name_string("/home/dream/tmp/pcs_setup", 0)
}

pub fn read_ecc_circuit_from_shared_memory<C: Config>() -> Circuit<C, NormalInputType> {
    read_object_from_shared_memory_name_string("/home/dream/tmp/circuit", 0)
}

pub fn read_partition_info_from_shared_memory() -> Vec<LayeredCircuitInputVec> {
    read_object_from_shared_memory_name_string("/home/dream/tmp/input_partition", 0)
}

pub fn read_commitment_from_shared_memory<
    PCSField: Field,
    F: FieldEngine,
    PCS: ExpanderPCS<F, PCSField>,
>() -> Vec<ExpanderGKRCommitment<PCSField, F, PCS>> {
    read_object_from_shared_memory_name_string("/home/dream/tmp/commitment", 0)
}

pub fn read_commitment_extra_info_from_shared_memory<
    PCSField: Field,
    F: FieldEngine,
    PCS: ExpanderPCS<F, PCSField>,
>() -> Vec<ExpanderGKRCommitmentExtraInfo<PCSField, F, PCS>> {
    read_object_from_shared_memory_name_string("/home/dream/tmp/extra_info", 0)
}

pub fn read_commitment_values_from_shared_memory<F: FieldEngine>(
    broadcast_info: &[bool],
    world_rank: usize,
    world_size: usize,
) -> Vec<Vec<F::SimdCircuitField>> {
    let shmem = ShmemConf::new().flink("/home/dream/tmp/input_vals").open().unwrap();
    let mut ptr = shmem.as_ptr();
    let n_components: usize =
        usize::deserialize_from(unsafe { std::slice::from_raw_parts(ptr, size_of::<usize>()) })
            .unwrap();
    ptr = unsafe { ptr.add(size_of::<usize>()) };

    assert!(
        n_components == broadcast_info.len(),
        "n_components and broadcast_info length mismatch"
    );

    broadcast_info
        .iter()
        .map(|is_broadcast| {
            let total_len_i: usize = usize::deserialize_from(unsafe {
                std::slice::from_raw_parts(ptr, size_of::<usize>())
            })
            .unwrap();
            let (local_len_i, offset) = if *is_broadcast {
                (total_len_i, size_of::<usize>())
            } else {
                (
                    total_len_i / world_size,
                    size_of::<usize>()
                        + world_rank
                            * (total_len_i / world_size)
                            * <F::SimdCircuitField as Field>::SIZE,
                )
            };

            let local_ptr = unsafe { ptr.add(offset) };
            let mut vals = Vec::with_capacity(local_len_i);
            unsafe {
                std::ptr::copy_nonoverlapping(
                    local_ptr as *const F::SimdCircuitField,
                    vals.as_mut_ptr(),
                    local_len_i,
                );
                vals.set_len(local_len_i);
            }

            ptr = unsafe {
                ptr.add(size_of::<usize>() + total_len_i * <F::SimdCircuitField as Field>::SIZE)
            };
            vals
        })
        .collect()
}

pub fn read_broadcast_info_from_shared_memory() -> Vec<bool> {
    read_object_from_shared_memory_name_string("/home/dream/tmp/broadcast_info", 0)
}

pub fn write_proof_to_shared_memory(proof: &ExpanderGKRProof) {
    write_object_to_shared_memory_name_string(proof, "/home/dream/tmp/proof");
}

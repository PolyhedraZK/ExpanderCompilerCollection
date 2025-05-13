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

pub fn read_object_from_shared_memory_name_string<T: ExpSerde>(shared_memory_ref: &str, offset: usize) -> T {
    let shmem = ShmemConf::new().flink(shared_memory_ref).open().unwrap();
    read_object_from_shared_memory(&Some(shmem), offset)
}

#[allow(clippy::type_complexity)]
pub fn read_selected_pkey_from_shared_memory<F: FieldEngine, PCS: ExpanderPCS<F>>(
) -> (usize, <PCS::SRS as StructuredReferenceString>::PKey) {
    read_object_from_shared_memory_name_string("pcs_setup", 0)
}

pub fn read_local_vals_to_commit_from_shared_memory<F: FieldEngine>(world_rank: usize, world_size: usize) -> Vec<F::SimdCircuitField> {
    let shmem = ShmemConf::new().flink("input_vals").open().unwrap();
    let ptr = shmem.as_ptr();
    let total_len: usize = usize::deserialize_from(unsafe {std::slice::from_raw_parts(ptr, size_of::<usize>())}).unwrap();
    
    let local_len = total_len / world_size;
    let offset = size_of::<usize>() + world_rank * local_len * <F::SimdCircuitField as Field>::SIZE;
    let ptr = unsafe { ptr.add(offset) };
    let mut cursor = std::io::Cursor::new(unsafe { std::slice::from_raw_parts(ptr, local_len * <F::SimdCircuitField as Field>::SIZE) });

    (0..local_len)
        .map(|_| {
            F::SimdCircuitField::deserialize_from(&mut cursor).unwrap()
        })
        .collect()
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

pub fn write_commitment_to_shared_memory<F: FieldEngine, PCS: ExpanderPCS<F>>(
    commitment: &ExpanderGKRCommitment<F, PCS>,
) {
    write_object_to_shared_memory_name_string(commitment, "commitment");
}

pub fn write_commitment_extra_info_to_shared_memory<F: FieldEngine, PCS: ExpanderPCS<F>>(
    extra_info: &ExpanderGKRCommitmentExtraInfo<F, PCS>,
) {
    write_object_to_shared_memory_name_string(extra_info, "extra_info");
}

pub fn read_pcs_setup_from_shared_memory<F: FieldEngine, PCS: ExpanderPCS<F>>(
) -> ExpanderGKRProverSetup<F, PCS> {
    read_object_from_shared_memory_name_string("pcs_setup", 0)
}

pub fn read_ecc_circuit_from_shared_memory<C: Config>() -> Circuit<C, NormalInputType> {
    read_object_from_shared_memory_name_string("circuit", 0)
}

pub fn read_partition_info_from_shared_memory() -> Vec<LayeredCircuitInputVec> {
    read_object_from_shared_memory_name_string("input_partition", 0)
}

pub fn read_commitment_from_shared_memory<F: FieldEngine, PCS: ExpanderPCS<F>>(
) -> Vec<ExpanderGKRCommitment<F, PCS>> {
    read_object_from_shared_memory_name_string("commitment", 0)
}

pub fn read_commitment_extra_info_from_shared_memory<F: FieldEngine, PCS: ExpanderPCS<F>>(
) -> Vec<ExpanderGKRCommitmentExtraInfo<F, PCS>> {
    read_object_from_shared_memory_name_string("extra_info", 0)
}

pub fn read_commitment_values_from_shared_memory<F: FieldEngine>() -> Vec<Vec<F::SimdCircuitField>>
{
    read_object_from_shared_memory_name_string("input_vals", 0)
}

pub fn read_broadcast_info_from_shared_memory() -> Vec<bool> {
    read_object_from_shared_memory_name_string("broadcast_info", 0)
}

pub fn write_proof_to_shared_memory(proof: &ExpanderGKRProof) {
    write_object_to_shared_memory_name_string(proof, "proof");
}

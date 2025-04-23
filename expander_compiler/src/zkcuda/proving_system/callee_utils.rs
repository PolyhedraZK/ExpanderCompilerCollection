use expander_config::GKRConfig;
use poly_commit::{PCSForExpanderGKR, StructuredReferenceString};
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

pub fn read_object_from_shared_memory_name_string<T: ExpSerde>(shared_memory_ref: &str) -> T {
    let shmem = ShmemConf::new().flink(shared_memory_ref).open().unwrap();
    read_object_from_shared_memory(&mut Some(shmem))
}

pub fn read_selected_pkey_from_shared_memory<C: Config>() -> (usize, <<pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::SRS as StructuredReferenceString>::PKey){
    read_object_from_shared_memory_name_string("pcs_setup")
}

pub fn read_commit_vals_from_shared_memory<C: Config>() -> Vec<C::DefaultSimdField> {
    read_object_from_shared_memory_name_string("commit_vals")
}

pub fn write_object_to_shared_memory_name_string<T: ExpSerde>(object: &T, shared_memory_ref: &str) {
    let mut buffer = vec![];
    object
        .serialize_into(&mut buffer)
        .expect("Failed to serialize object");

    unsafe {
        let shmem = ShmemConf::new().flink(shared_memory_ref).open().unwrap();

        let object_ptr = shmem.as_ptr() as *mut u8;
        std::ptr::copy_nonoverlapping(buffer.as_ptr(), object_ptr, buffer.len());
    }
}

pub fn write_commitment_to_shared_memory<C: Config>(
    commitment: &<pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::Commitment,
) {
    write_object_to_shared_memory_name_string(commitment, "commitment");
}

pub fn write_commitment_extra_info_to_shared_memory<C: Config>(
    extra_info: &<pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::ScratchPad,
) {
    write_object_to_shared_memory_name_string(extra_info, "commitment_extra_info");
}

pub fn read_pcs_setup_from_shared_memory<C: Config>() -> ExpanderGKRProverSetup<C> {
    read_object_from_shared_memory_name_string("pcs_setup")
}

pub fn read_ecc_circuit_from_shared_memory<C: Config>() -> Circuit<C, NormalInputType> {
    read_object_from_shared_memory_name_string("ecc_circuit")
}

pub fn read_partition_info_from_shared_memory() -> Vec<LayeredCircuitInputVec> {
    read_object_from_shared_memory_name_string("partition_info")
}

pub fn read_commitment_from_shared_memory<C: Config>() -> Vec<ExpanderGKRCommitment<C>> {
    read_object_from_shared_memory_name_string("commitment")
}

pub fn read_commitment_extra_info_from_shared_memory<C: Config>(
) -> Vec<ExpanderGKRCommitmentExtraInfo<C>> {
    read_object_from_shared_memory_name_string("commitment_extra_info")
}

pub fn read_commitment_values_from_shared_memory<C: Config>() -> Vec<Vec<C::DefaultSimdField>> {
    read_object_from_shared_memory_name_string("commitment_values")
}

pub fn read_broadcast_info_from_shared_memory() -> Vec<bool> {
    read_object_from_shared_memory_name_string("broadcast_info")
}

pub fn write_proof_to_shared_memory(proof: &ExpanderGKRProof) {
    write_object_to_shared_memory_name_string(proof, "proof");
}

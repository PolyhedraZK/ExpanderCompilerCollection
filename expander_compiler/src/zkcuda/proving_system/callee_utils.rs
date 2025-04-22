use expander_config::GKRConfig;
use poly_commit::{StructuredReferenceString, PCSForExpanderGKR};
use serdes::ExpSerde;
use shared_memory::ShmemConf;

use crate::circuit::config::Config;

pub use super::caller_utils::read_object_from_shared_memory;
use super::Commitment;

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

pub fn read_object_from_shared_memory_name_string<T: ExpSerde>(
    shared_memory_ref: &str,
) -> T {
    let shmem = ShmemConf::new()
        .flink(shared_memory_ref)
        .open()
        .unwrap();
    read_object_from_shared_memory(&mut Some(shmem))
}

pub fn read_selected_pkey_from_shared_memory<C: Config>() -> (usize, <<pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::SRS as StructuredReferenceString>::PKey) {
    read_object_from_shared_memory_name_string("pcs_setup")
}

pub fn read_commit_vals_from_shared_memory<C: Config>() -> Vec<C::DefaultSimdField> {
    read_object_from_shared_memory_name_string("commit_vals")
}

pub fn write_object_to_shared_memory_name_string<T: ExpSerde>(
    object: &T,
    shared_memory_ref: &str,
) {
    let mut buffer = vec![];
    object
        .serialize_into(&mut buffer)
        .expect("Failed to serialize object");

    unsafe {
        let shmem = ShmemConf::new()
            .flink(shared_memory_ref)
            .open()
            .unwrap();

        let object_ptr = shmem.as_ptr() as *mut u8;
        std::ptr::copy_nonoverlapping(buffer.as_ptr(), object_ptr, buffer.len());
    }
}

pub fn write_commitment_to_shared_memory<C: Config>(
    commitment: &<pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::Commitment
) {
    write_object_to_shared_memory_name_string(
        commitment,
        "commitment",
    );
}

pub fn write_commitment_extra_info_to_shared_memory<C: Config>(
    extra_info: &<pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::ScratchPad,
) {
    write_object_to_shared_memory_name_string(
        extra_info,
        "commitment_extra_info",
    );
}
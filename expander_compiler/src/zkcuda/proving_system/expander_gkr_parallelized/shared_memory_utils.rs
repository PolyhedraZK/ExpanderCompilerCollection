#![allow(static_mut_refs)]

use std::process::Command;

use crate::{
    circuit::layered::Circuit, frontend::SIMDField, zkcuda::{kernel::LayeredCircuitInputVec, proving_system::{CombinedProof, ExpanderGKRProvingSystem, ParallelizedExpanderGKRProvingSystem, ProvingSystem}},
};
use arith::Field;
use gkr_engine::{ExpanderPCS, FieldEngine, FieldType, GKREngine, PolynomialCommitmentType};
use serdes::ExpSerde;
use shared_memory::{Shmem, ShmemConf};

use crate::circuit::{config::Config, layered::InputType};

use super::{
    ExpanderGKRCommitment, ExpanderGKRCommitmentExtraInfo, ExpanderGKRProof,
    ExpanderGKRProverSetup, ExpanderGKRVerifierSetup,
};

#[derive(Default)]
pub struct SharedMemory {
    pub pcs_setup: Option<Shmem>,
    pub witness: Option<Shmem>,
    pub proof: Option<Shmem>,
}

pub static mut SHARED_MEMORY: SharedMemory = SharedMemory {
    pcs_setup: None,
    witness: None,
    proof: None,
};

pub struct SharedMemoryEngine {}

/// This impl block contains utility functions for managing shared memory in the context of the Expander GKR proving system.
impl SharedMemoryEngine {
    /// Allocate shared memory for the given name and size if it is not already allocated or if the existing allocation is smaller than the target size.
    /// The result is stored in the provided `handle`, it's the caller's responsibility to ensure that the `handle` lives long enough for the reader to access the shared memory.
    fn allocate_shared_memory_if_necessary(handle: &mut Option<Shmem>, name: &str, target_size: usize) {
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

    /// Write an object to shared memory. If the shared memory is not allocated or is too small, it will be allocated with the size of the serialized object.
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
            Self::allocate_shared_memory_if_necessary(shared_memory_ref, name, buffer.len());
            let object_ptr = shared_memory_ref.as_mut().unwrap().as_ptr();
            std::ptr::copy_nonoverlapping(buffer.as_ptr(), object_ptr, buffer.len());
        }
    }

    /// Read an object from shared memory. If the shared memory is not allocated, it will panic.
    pub fn read_object_from_shared_memory<T: ExpSerde>(
        shared_memory_ref: &str,
        offset: usize,
    ) -> T {
        let shmem = ShmemConf::new().flink(shared_memory_ref).open().expect("Failed to open shared memory");
        let object_ptr = shmem.as_ptr() as *const u8;
        let object_len = shmem.len();
        let buffer = unsafe { std::slice::from_raw_parts(object_ptr.add(offset), object_len - offset) };
        T::deserialize_from(buffer).expect("Failed to deserialize object")
    }
}

/// This impl block contains functions for reading/writing specific objects to shared memory.
impl SharedMemoryEngine {
    pub fn write_pcs_setup_to_shared_memory<
        PCSField: Field,
        F: FieldEngine,
        PCS: ExpanderPCS<F, PCSField>,
    >(
    pcs_setup: &(
            ExpanderGKRProverSetup<PCSField, F, PCS>,
            ExpanderGKRVerifierSetup<PCSField, F, PCS>,
        ),
    ) {
        Self::write_object_to_shared_memory(
            pcs_setup,
            unsafe { &mut SHARED_MEMORY.pcs_setup },
            "pcs_setup",
        );
    }

    pub fn read_pcs_setup_from_shared_memory<
        PCSField: Field,
        F: FieldEngine,
        PCS: ExpanderPCS<F, PCSField>,
    >() -> (
        ExpanderGKRProverSetup<PCSField, F, PCS>,
        ExpanderGKRVerifierSetup<PCSField, F, PCS>,
    ) {
        Self::read_object_from_shared_memory("pcs_setup", 0)
    }

    pub fn write_proof_to_shared_memory<C: Config>(proof: &CombinedProof<C, ExpanderGKRProvingSystem<C>>)
    where 
        C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
    {
        Self::write_object_to_shared_memory(proof, unsafe { &mut SHARED_MEMORY.proof },"proof");
    }

    pub fn read_proof_from_shared_memory<C: Config>() -> CombinedProof<C, ExpanderGKRProvingSystem<C>> 
    where
        C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
    {
        Self::read_object_from_shared_memory("proof", 0)
    }
}

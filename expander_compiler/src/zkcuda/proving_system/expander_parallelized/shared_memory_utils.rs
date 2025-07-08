#![allow(static_mut_refs)]

use crate::zkcuda::proving_system::{CombinedProof, Expander};
use arith::Field;
use gkr_engine::{ExpanderPCS, FieldEngine, GKREngine};
use serdes::ExpSerde;
use shared_memory::{Shmem, ShmemConf};

use crate::circuit::config::Config;

use crate::zkcuda::proving_system::expander::structs::{
    ExpanderProverSetup, ExpanderVerifierSetup,
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
    fn allocate_shared_memory_if_necessary(
        handle: &mut Option<Shmem>,
        name: &str,
        target_size: usize,
    ) {
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
        let shmem = ShmemConf::new()
            .flink(shared_memory_ref)
            .open()
            .expect("Failed to open shared memory");
        let object_ptr = shmem.as_ptr() as *const u8;
        let object_len = shmem.len();
        let buffer =
            unsafe { std::slice::from_raw_parts(object_ptr.add(offset), object_len - offset) };
        T::deserialize_from(buffer).expect("Failed to deserialize object")
    }
}

/// This impl block contains functions for reading/writing specific objects to shared memory.
impl SharedMemoryEngine {
    pub fn write_pcs_setup_to_shared_memory<F: FieldEngine, PCS: ExpanderPCS<F>>(
        pcs_setup: &(ExpanderProverSetup<F, PCS>, ExpanderVerifierSetup<F, PCS>),
    ) {
        Self::write_object_to_shared_memory(
            pcs_setup,
            unsafe { &mut SHARED_MEMORY.pcs_setup },
            "pcs_setup",
        );
    }

    pub fn read_pcs_setup_from_shared_memory<F: FieldEngine, PCS: ExpanderPCS<F>>(
    ) -> (ExpanderProverSetup<F, PCS>, ExpanderVerifierSetup<F, PCS>) {
        Self::read_object_from_shared_memory("pcs_setup", 0)
    }

    pub fn write_witness_to_shared_memory<F: FieldEngine>(
        values: &[impl AsRef<[F::SimdCircuitField]>],
    ) {
        let total_size = std::mem::size_of::<usize>()
            + values
                .iter()
                .map(|v| std::mem::size_of::<usize>() + std::mem::size_of_val(v.as_ref()))
                .sum::<usize>();

        unsafe {
            Self::allocate_shared_memory_if_necessary(
                &mut SHARED_MEMORY.witness,
                "witness",
                total_size,
            );

            let mut ptr = SHARED_MEMORY.witness.as_mut().unwrap().as_ptr();

            // Copy the length of the vector
            let len = values.len();
            let len_ptr = &len as *const usize as *const u8;
            std::ptr::copy_nonoverlapping(len_ptr, ptr, std::mem::size_of::<usize>());
            ptr = ptr.add(std::mem::size_of::<usize>());

            for vals in values {
                let vals_len = vals.as_ref().len();
                let len_ptr = &vals_len as *const usize as *const u8;
                std::ptr::copy_nonoverlapping(len_ptr, ptr, std::mem::size_of::<usize>());
                ptr = ptr.add(std::mem::size_of::<usize>());

                let vals_size = std::mem::size_of_val(vals.as_ref());
                std::ptr::copy_nonoverlapping(vals.as_ref().as_ptr() as *const u8, ptr, vals_size);
                ptr = ptr.add(vals_size);
            }
        }
    }

    pub fn read_witness_from_shared_memory<F: FieldEngine>() -> Vec<Vec<F::SimdCircuitField>> {
        let shmem = ShmemConf::new().flink("witness").open().unwrap();
        let mut ptr = shmem.as_ptr();
        let n_components: usize =
            usize::deserialize_from(unsafe { std::slice::from_raw_parts(ptr, size_of::<usize>()) })
                .unwrap();
        ptr = unsafe { ptr.add(size_of::<usize>()) };

        (0..n_components)
            .map(|_| {
                let total_len_i: usize = usize::deserialize_from(unsafe {
                    std::slice::from_raw_parts(ptr, size_of::<usize>())
                })
                .unwrap();
                ptr = unsafe { ptr.add(size_of::<usize>()) };

                let mut vals = Vec::with_capacity(total_len_i);
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        ptr as *const F::SimdCircuitField,
                        vals.as_mut_ptr(),
                        total_len_i,
                    );
                    vals.set_len(total_len_i);
                }

                ptr = unsafe { ptr.add(total_len_i * <F::SimdCircuitField as Field>::SIZE) };
                vals
            })
            .collect()
    }

    pub fn write_proof_to_shared_memory<
        C: GKREngine,
        ECCConfig: Config<FieldConfig = C::FieldConfig>,
    >(
        proof: &CombinedProof<ECCConfig, Expander<C>>,
    ) {
        Self::write_object_to_shared_memory(proof, unsafe { &mut SHARED_MEMORY.proof }, "proof");
    }

    pub fn read_proof_from_shared_memory<
        C: GKREngine,
        ECCConfig: Config<FieldConfig = C::FieldConfig>,
    >() -> CombinedProof<ECCConfig, Expander<C>>
where {
        Self::read_object_from_shared_memory("proof", 0)
    }
}

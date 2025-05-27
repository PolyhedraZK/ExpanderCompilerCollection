use arith::SimdField;
use serdes::ExpSerde;

use crate::field::FieldArith;
use crate::hints::registry::{EmptyHintCaller, HintCaller};
use crate::utils::misc::next_power_of_two;
use crate::{
    circuit::config::{CircuitField, Config, SIMDField},
    utils::pool::Pool,
};

use super::vec_shaped::{flatten_shaped_pack_simd, unflatten_shaped_unpack_simd};
use super::{
    kernel2::{shape_prepend, Kernel, Shape},
    proof::{ComputationGraph, ProofTemplate},
    proving_system::{ExpanderGKRProvingSystem, ProvingSystem},
    vec_shaped::{flatten_shaped, unflatten_shaped, VecShaped},
};

pub use macros::call_kernel;

pub struct DeviceMemory<C: Config> {
    pub values: Vec<SIMDField<C>>,
}

#[derive(Clone)]
pub struct DeviceMemoryHandleRaw {
    pub id: usize,
    pub shape: Shape,
}

pub type DeviceMemoryHandle = Option<DeviceMemoryHandleRaw>;

pub struct Context<C: Config, H: HintCaller<CircuitField<C>> = EmptyHintCaller> {
    pub kernel_primitives: Pool<Kernel<C>>,
    pub device_memories: Vec<DeviceMemory<C>>,
    pub proof_templates: Vec<ProofTemplate>,
    pub hint_caller: H,
}

impl<C: Config> Default for Context<C> {
    fn default() -> Self {
        Self::new(EmptyHintCaller)
    }
}

fn ensure_handle(handle: DeviceMemoryHandle) -> DeviceMemoryHandleRaw {
    match handle {
        Some(handle) => handle,
        None => panic!("Empty DeviceMemoryHandle"),
    }
}

fn pack_vec<C: Config>(v: &[CircuitField<C>]) -> Vec<SIMDField<C>> {
    v.iter()
        .map(|x| {
            let mut v = Vec::with_capacity(SIMDField::<C>::PACK_SIZE);
            for _ in 0..SIMDField::<C>::PACK_SIZE {
                v.push(*x);
            }
            SIMDField::<C>::pack(&v)
        })
        .collect::<Vec<_>>()
}

fn unpack_vec<C: Config>(v: &[SIMDField<C>]) -> Vec<CircuitField<C>> {
    v.iter().map(|x| x.unpack()[0]).collect()
}

fn check_reshape_compat(shape: &[usize], new_shape: &[usize]) {
    panic!("TODO")
}

pub trait Reshape {
    fn reshape(&self, new_shape: &[usize]) -> Self;
}

impl Reshape for DeviceMemoryHandle {
    fn reshape(&self, new_shape: &[usize]) -> Self {
        let handle = ensure_handle(self.clone());
        if handle.shape.is_none() {
            panic!("Cannot reshape non-shaped memory");
        }
        check_reshape_compat(&handle.shape.unwrap(), new_shape);
        Some(DeviceMemoryHandleRaw {
            id: handle.id,
            broadcast_type: handle.broadcast_type,
            shape: Some(new_shape.to_vec()),
        })
    }
}

impl<C: Config, H: HintCaller<CircuitField<C>>> Context<C, H> {
    pub fn new(hint_caller: H) -> Self {
        Context {
            kernels: Pool::new(),
            device_memories: vec![],
            proof_templates: vec![],
            hint_caller,
        }
    }

    fn make_device_mem(&mut self, values: Vec<SIMDField<C>>, shape: Shape) -> DeviceMemoryHandle {
        self.device_memories.push(DeviceMemory { values: values });
        Some(DeviceMemoryHandleRaw {
            id: self.device_memories.len() - 1,
            shape,
        })
    }

    pub fn copy_raw_to_device(&mut self, host_memory: &[CircuitField<C>]) -> DeviceMemoryHandle {
        let simd_host_memory = pack_vec::<C>(host_memory);
        self.make_device_mem(simd_host_memory, None)
    }

    pub fn copy_raw_simd_to_device(
        &mut self,
        simd_host_memory: &[SIMDField<C>],
    ) -> DeviceMemoryHandle {
        self.make_device_mem(simd_host_memory.to_vec(), None)
    }

    pub fn copy_to_device<T: VecShaped<CircuitField<C>>>(
        &mut self,
        host_memory: &T,
    ) -> DeviceMemoryHandle {
        let (flat, shape) = flatten_shaped(host_memory);
        let shape = Some(shape);
        let simd_flat = pack_vec::<C>(&flat);
        self.make_device_mem(simd_flat, shape)
    }

    pub fn copy_to_device_and_pack_simd<T: VecShaped<CircuitField<C>>>(
        &mut self,
        host_memory: &T,
    ) -> DeviceMemoryHandle {
        let (flat, shape) = flatten_shaped_pack_simd(host_memory);
        let shape = Some(shape);
        self.make_device_mem(flat, shape)
    }

    pub fn copy_simd_to_device<T: VecShaped<SIMDField<C>>>(
        &mut self,
        host_memory: &T,
    ) -> DeviceMemoryHandle {
        let (flat, shape) = flatten_shaped(host_memory);
        let shape = Some(shape);
        self.make_device_mem(flat, shape)
    }

    pub fn copy_raw_to_host(
        &self,
        device_memory_handle: DeviceMemoryHandle,
    ) -> Vec<CircuitField<C>> {
        let device_memory_handle = ensure_handle(device_memory_handle);
        unpack_vec::<C>(&self.device_memories[device_memory_handle.id].values)
    }

    pub fn copy_raw_simd_to_host(
        &self,
        device_memory_handle: DeviceMemoryHandle,
    ) -> Vec<SIMDField<C>> {
        let device_memory_handle = ensure_handle(device_memory_handle);
        self.device_memories[device_memory_handle.id].values.clone()
    }

    pub fn copy_to_host<T: VecShaped<CircuitField<C>> + Default>(
        &self,
        device_memory_handle: DeviceMemoryHandle,
    ) -> T {
        let device_memory_handle = ensure_handle(device_memory_handle);
        unflatten_shaped(
            &unpack_vec::<C>(&self.device_memories[device_memory_handle.id].raw_values),
            &device_memory_handle.shape.unwrap(),
        )
    }

    pub fn copy_to_host_and_unpack_simd<T: VecShaped<CircuitField<C>> + Default>(
        &self,
        device_memory_handle: DeviceMemoryHandle,
    ) -> T {
        let device_memory_handle = ensure_handle(device_memory_handle);
        unflatten_shaped_unpack_simd(
            &self.device_memories[device_memory_handle.id].raw_values,
            &device_memory_handle.shape.unwrap(),
        )
    }

    pub fn copy_simd_to_host<T: VecShaped<SIMDField<C>> + Default>(
        &self,
        device_memory_handle: DeviceMemoryHandle,
    ) -> T {
        let device_memory_handle = ensure_handle(device_memory_handle);
        unflatten_shaped(
            &self.device_memories[device_memory_handle.id].raw_values,
            &device_memory_handle.shape.unwrap(),
        )
    }

    pub fn call_kernel(&mut self, kernel: &Kernel<C>, ios: &mut [DeviceMemoryHandle]) {
        if kernel.io_shapes.len() != ios.len() {
            panic!("Invalid number of inputs/outputs");
        }
        panic!("TODO");
    }

    pub fn call_kernel_raw(
        &mut self,
        kernel: &Kernel<C>,
        //ios: &mut [DeviceMemoryHandle],
        //parallel_count: usize,
        //is_broadcast: &[bool],
    ) {
        panic!("TODO");
    }

    pub fn generate_computation_graph(&self) -> ComputationGraph<C> {
        panic!("TODO");
    }
}

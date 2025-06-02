use arith::SimdField;
use serdes::ExpSerde;

use crate::{
    circuit::config::{CircuitField, Config, SIMDField},
    field::FieldArith,
    hints::registry::{EmptyHintCaller, HintCaller},
    utils::{error::Error, misc::next_power_of_two, pool::Pool},
};

use super::vec_shaped::{flatten_shaped_pack_simd, unflatten_shaped_unpack_simd};
use super::{
    kernel2::{shape_prepend, KernelPrimitive, Shape},
    proving_system::{ExpanderGKRProvingSystem, ProvingSystem},
    vec_shaped::{flatten_shaped, unflatten_shaped, VecShaped},
};

pub use macros::call_kernel;

pub struct DeviceMemory<C: Config> {
    pub values: Vec<SIMDField<C>>,
}

#[derive(Clone, Debug)]
pub struct DeviceMemoryHandleRaw {
    pub id: usize,
    pub shape: Shape,
}

pub type DeviceMemoryHandle = Option<DeviceMemoryHandleRaw>;

#[derive(Clone, ExpSerde)]
pub struct KernelCall {
    pub kernel_id: usize,
    pub num_parallel: usize,
    pub device_memory_indices: Vec<usize>,
}

pub struct Context<C: Config, H: HintCaller<CircuitField<C>> = EmptyHintCaller> {
    pub kernel_primitives: Pool<KernelPrimitive<C>>,
    pub device_memories: Vec<DeviceMemory<C>>,
    pub kernel_calls: Vec<KernelCall>,
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

// returns Option<is_broadcast>
fn check_shape_compat(
    kernel_shape: &Shape,
    io_shape: &Shape,
    parallel_count: usize,
) -> Option<bool> {
    if kernel_shape.len() == io_shape.len() {
        if *kernel_shape == *io_shape {
            Some(true)
        } else {
            None
        }
    } else if kernel_shape.len() + 1 == io_shape.len() {
        if io_shape.iter().skip(1).eq(kernel_shape.iter()) {
            if io_shape[0] == parallel_count {
                Some(false)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn check_reshape_compat(shape: &[usize], new_shape: &[usize]) {
    // TODO!!!
    //panic!("TODO")
}

pub trait Reshape {
    fn reshape(&self, new_shape: &[usize]) -> Self;
}

impl Reshape for DeviceMemoryHandle {
    fn reshape(&self, new_shape: &[usize]) -> Self {
        let handle = ensure_handle(self.clone());
        check_reshape_compat(&handle.shape, new_shape);
        Some(DeviceMemoryHandleRaw {
            id: handle.id,
            shape: new_shape.to_vec(),
        })
    }
}

impl<C: Config, H: HintCaller<CircuitField<C>>> Context<C, H> {
    pub fn new(hint_caller: H) -> Self {
        Context {
            kernel_primitives: Pool::new(),
            device_memories: vec![],
            kernel_calls: vec![],
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

    pub fn copy_to_device<T: VecShaped<CircuitField<C>>>(
        &mut self,
        host_memory: &T,
    ) -> DeviceMemoryHandle {
        let (flat, shape) = flatten_shaped(host_memory);
        let simd_flat = pack_vec::<C>(&flat);
        self.make_device_mem(simd_flat, shape)
    }

    pub fn copy_to_device_and_pack_simd<T: VecShaped<CircuitField<C>>>(
        &mut self,
        host_memory: &T,
    ) -> DeviceMemoryHandle {
        let (flat, shape) = flatten_shaped_pack_simd(host_memory);
        self.make_device_mem(flat, shape)
    }

    pub fn copy_simd_to_device<T: VecShaped<SIMDField<C>>>(
        &mut self,
        host_memory: &T,
    ) -> DeviceMemoryHandle {
        let (flat, shape) = flatten_shaped(host_memory);
        self.make_device_mem(flat, shape)
    }

    pub fn copy_to_host<T: VecShaped<CircuitField<C>> + Default>(
        &self,
        device_memory_handle: DeviceMemoryHandle,
    ) -> T {
        let device_memory_handle = ensure_handle(device_memory_handle);
        unflatten_shaped(
            &unpack_vec::<C>(&self.device_memories[device_memory_handle.id].values),
            &device_memory_handle.shape,
        )
    }

    pub fn copy_to_host_and_unpack_simd<T: VecShaped<CircuitField<C>> + Default>(
        &self,
        device_memory_handle: DeviceMemoryHandle,
    ) -> T {
        let device_memory_handle = ensure_handle(device_memory_handle);
        unflatten_shaped_unpack_simd(
            &self.device_memories[device_memory_handle.id].values,
            &device_memory_handle.shape,
        )
    }

    pub fn copy_simd_to_host<T: VecShaped<SIMDField<C>> + Default>(
        &self,
        device_memory_handle: DeviceMemoryHandle,
    ) -> T {
        let device_memory_handle = ensure_handle(device_memory_handle);
        unflatten_shaped(
            &self.device_memories[device_memory_handle.id].values,
            &device_memory_handle.shape,
        )
    }

    fn ir_copy_from_device_memory(
        &self,
        handle: DeviceMemoryHandle,
        s: &mut [SIMDField<C>],
        is_broadcast: bool,
        parallel_index: usize,
    ) {
        let handle = ensure_handle(handle);
        if is_broadcast {
            s.copy_from_slice(&self.device_memories[handle.id].values);
        } else {
            let r = &self.device_memories[handle.id].values;
            let len = r.len();
            let chunk_size = len / handle.shape[0];
            s.copy_from_slice(&r[chunk_size * parallel_index..chunk_size * (parallel_index + 1)]);
        }
    }

    pub fn call_kernel(
        &mut self,
        kernel: &KernelPrimitive<C>,
        num_parallel: usize,
        ios: &mut [DeviceMemoryHandle],
    ) -> Result<(), Error> {
        if kernel.io_shapes.len() != ios.len() {
            panic!("Invalid number of inputs/outputs");
        }
        let mut is_broadcast = Vec::with_capacity(ios.len());
        for (i, ((kernel_shape, io), spec)) in kernel
            .io_shapes
            .iter()
            .zip(ios.iter())
            .zip(kernel.io_specs.iter())
            .enumerate()
        {
            if !spec.is_input {
                is_broadcast.push(false);
                continue;
            }
            println!(
                "Checking shape compatibility for input/output {}: kernel_shape={:?}, io_shape={:?}, num_parallel={}",
                i, kernel_shape, io, num_parallel
            );
            let io_shape = if let Some(handle) = io {
                handle.shape.clone()
            } else {
                panic!("Missing input at index {}", i)
            };
            match check_shape_compat(kernel_shape, &io_shape, num_parallel) {
                Some(ib) => is_broadcast.push(ib),
                None => {
                    panic!(
                        "Incompatible shapes: want {:?}, got {:?}, num_parallel={} (Hint: if you want to broadcast, use {:?}, otherwise use {:?})",
                        kernel_shape,
                        io_shape,
                        num_parallel,
                        kernel_shape,
                        shape_prepend(&io_shape, num_parallel)
                    );
                }
            }
        }
        for (io_spec, ib) in kernel.io_specs.iter().zip(is_broadcast.iter()) {
            if io_spec.is_output && *ib {
                panic!("Output is broadcasted, but it shouldn't be");
            }
        }

        let kernel_id = self.kernel_primitives.add(kernel);

        let mut outputs_tmp = vec![Vec::new(); kernel.io_specs.len()];
        for parallel_i in 0..num_parallel {
            let mut ir_inputs = vec![SIMDField::<C>::zero(); kernel.ir.input_size()];
            for (i, ((input, input_start), input_end)) in ios
                .iter()
                .zip(kernel.ir_input_offsets.iter())
                .zip(kernel.ir_input_offsets.iter().skip(1))
                .enumerate()
            {
                if input.is_none() {
                    continue;
                }
                self.ir_copy_from_device_memory(
                    input.clone(),
                    &mut ir_inputs[*input_start..*input_end],
                    is_broadcast[i],
                    parallel_i,
                );
            }
            let ir_outputs = kernel
                .ir
                .eval_safe_simd(ir_inputs, &[], &mut self.hint_caller)?;
            for (((spec, output_start), output_end), out) in kernel
                .io_specs
                .iter()
                .zip(kernel.ir_expected_output_offsets.iter())
                .zip(kernel.ir_expected_output_offsets.iter().skip(1))
                .zip(outputs_tmp.iter_mut())
            {
                if !spec.is_output {
                    continue;
                }
                out.extend_from_slice(&ir_outputs[*output_start..*output_end]);
            }
        }

        for (((output, spec), ov), shape) in ios
            .iter_mut()
            .zip(kernel.io_specs.iter())
            .zip(outputs_tmp.into_iter())
            .zip(kernel.io_shapes.iter())
        {
            if !spec.is_output {
                *output = None;
                continue;
            }
            let handle = self.make_device_mem(ov, shape_prepend(shape, num_parallel));
            *output = handle.clone();
        }
        self.kernel_calls.push(KernelCall {
            kernel_id,
            num_parallel,
            device_memory_indices: ios.iter().map(|x| x.as_ref().map_or(0, |h| h.id)).collect(),
        });
        Ok(())
    }

    pub fn compile_computation_graph(&self) {
        panic!("TODO");
    }
}

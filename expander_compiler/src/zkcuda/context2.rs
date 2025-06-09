use arith::SimdField;
use serdes::ExpSerde;

use crate::{
    circuit::config::{CircuitField, Config, SIMDField},
    field::FieldArith,
    hints::registry::{EmptyHintCaller, HintCaller},
    utils::{error::Error, misc::next_power_of_two, pool::Pool},
    zkcuda::shape::{merge_shape_products, shape_vec_len, Reshape, Shape, ShapeHistory, Transpose},
};

use super::vec_shaped::{flatten_shaped_pack_simd, unflatten_shaped_unpack_simd};
use super::{
    kernel2::{shape_prepend, KernelPrimitive},
    proving_system::{ExpanderGKRProvingSystem, ProvingSystem},
    vec_shaped::{flatten_shaped, unflatten_shaped, VecShaped},
};

pub use macros::call_kernel;

pub struct DeviceMemory<C: Config> {
    pub values: Vec<SIMDField<C>>,
    pub required_shape_products: Vec<usize>,
}

#[derive(Clone, Debug, ExpSerde)]
pub struct DeviceMemoryHandleRaw {
    pub id: usize,
    pub shape_history: ShapeHistory,
}

pub type DeviceMemoryHandle = Option<DeviceMemoryHandleRaw>;

#[derive(Clone, ExpSerde)]
pub struct KernelCall {
    pub kernel_id: usize,
    pub num_parallel: usize,
    pub input_handles: Vec<DeviceMemoryHandle>,
    pub output_handles: Vec<DeviceMemoryHandle>,
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

impl Reshape for DeviceMemoryHandle {
    fn reshape(&self, new_shape: &[usize]) -> Self {
        let handle = ensure_handle(self.clone());
        Some(DeviceMemoryHandleRaw {
            id: handle.id,
            shape_history: handle.shape_history.reshape(new_shape),
        })
    }
}

impl Transpose for DeviceMemoryHandle {
    fn transpose(&self, axes: &[usize]) -> Self {
        let handle = ensure_handle(self.clone());
        Some(DeviceMemoryHandleRaw {
            id: handle.id,
            shape_history: handle.shape_history.transpose(axes),
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
        let t = shape_vec_len(&shape);
        let required_shape_products = if t == 1 { vec![1] } else { vec![1, t] };
        self.device_memories.push(DeviceMemory {
            values: values,
            required_shape_products,
        });
        Some(DeviceMemoryHandleRaw {
            id: self.device_memories.len() - 1,
            shape_history: ShapeHistory::new(shape),
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
        let permuted_values = device_memory_handle
            .shape_history
            .permute_vec(&self.device_memories[device_memory_handle.id].values);
        unflatten_shaped(
            &unpack_vec::<C>(&permuted_values),
            &device_memory_handle.shape_history.shape(),
        )
    }

    pub fn copy_to_host_and_unpack_simd<T: VecShaped<CircuitField<C>> + Default>(
        &self,
        device_memory_handle: DeviceMemoryHandle,
    ) -> T {
        let device_memory_handle = ensure_handle(device_memory_handle);
        let permuted_values = device_memory_handle
            .shape_history
            .permute_vec(&self.device_memories[device_memory_handle.id].values);
        unflatten_shaped_unpack_simd(
            &permuted_values,
            &device_memory_handle.shape_history.shape(),
        )
    }

    pub fn copy_simd_to_host<T: VecShaped<SIMDField<C>> + Default>(
        &self,
        device_memory_handle: DeviceMemoryHandle,
    ) -> T {
        let device_memory_handle = ensure_handle(device_memory_handle);
        let permuted_values = device_memory_handle
            .shape_history
            .permute_vec(&self.device_memories[device_memory_handle.id].values);
        unflatten_shaped(
            &permuted_values,
            &device_memory_handle.shape_history.shape(),
        )
    }

    fn ir_copy_from_device_memory(
        &self,
        values: &[SIMDField<C>],
        s: &mut [SIMDField<C>],
        is_broadcast: bool,
        parallel_index: usize,
        chunk_size: Option<usize>,
    ) {
        if is_broadcast {
            s.copy_from_slice(&values);
        } else {
            let chunk_size = chunk_size.unwrap();
            s.copy_from_slice(
                &values[chunk_size * parallel_index..chunk_size * (parallel_index + 1)],
            );
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
                handle.shape_history.shape()
            } else {
                panic!("Missing input at index {}", i)
            };
            match check_shape_compat(kernel_shape, &io_shape, num_parallel) {
                Some(ib) => {
                    let isl = io
                        .as_ref()
                        .unwrap()
                        .shape_history
                        .get_initial_split_list(!ib);
                    let t = io.as_ref().unwrap().id;
                    self.device_memories[t].required_shape_products = merge_shape_products(
                        &isl,
                        &self.device_memories[t].required_shape_products,
                    );
                    is_broadcast.push(ib)
                }
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
        let mut ir_inputs_all = Vec::new();
        let mut chunk_sizes = Vec::new();
        for (input, &ib) in ios.iter().zip(is_broadcast.iter()) {
            if input.is_none() {
                continue;
            }
            let handle = ensure_handle(input.clone());
            let values = handle
                .shape_history
                .permute_vec(&self.device_memories[handle.id].values);
            chunk_sizes.push(if ib {
                None
            } else {
                Some(values.len() / num_parallel)
            });
            ir_inputs_all.push(values);
        }
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
                    &ir_inputs_all[i],
                    &mut ir_inputs[*input_start..*input_end],
                    is_broadcast[i],
                    parallel_i,
                    chunk_sizes[i],
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
        let input_handles = ios.to_vec();
        let mut output_handles = vec![None; kernel.io_specs.len()];

        for ((((output, out2), spec), ov), shape) in ios
            .iter_mut()
            .zip(output_handles.iter_mut())
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
            *out2 = handle;
        }
        self.kernel_calls.push(KernelCall {
            kernel_id,
            num_parallel,
            input_handles,
            output_handles,
        });
        Ok(())
    }

    pub fn compile_computation_graph(&self) {
        for dm in &self.device_memories {
            println!("{:?}", dm.required_shape_products);
        }
        panic!("TODO");
    }
}

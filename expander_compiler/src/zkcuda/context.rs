use arith::SimdField;

use crate::field::FieldArith;
use crate::utils::misc::next_power_of_two;
use crate::zkcuda::vec_shaped::unflatten_shaped;
use crate::{circuit::config::Config, utils::pool::Pool};

use super::kernel::shape_prepend;
use super::vec_shaped::{flatten_shaped, VecShaped};
use super::{
    kernel::{Kernel, Shape},
    proving_system::ProvingSystem,
};

pub use macros::call_kernel;

pub struct DeviceMemory<C: Config, P: ProvingSystem<C>> {
    pub raw_values: Vec<C::DefaultSimdField>,
    pub values: Vec<C::DefaultSimdField>,
    pub commitment: P::Commitment,
}

#[derive(Clone)]
pub struct DeviceMemoryHandleRaw {
    pub id: usize,
    pub broadcast_type: BroadcastType,
    pub shape: Shape,
}

pub type DeviceMemoryHandle = Option<DeviceMemoryHandleRaw>;

#[derive(Copy, Clone)]
pub enum BroadcastType {
    BroadcastOnly,
    NonBroadcastOnly,
    Both,
}

pub struct WrappedProof<C: Config, P: ProvingSystem<C>> {
    pub proof: P::Proof,
    pub kernel_id: usize,
    pub commitment_indices: Vec<usize>,
    pub parallel_count: usize,
    pub is_broadcast: Vec<bool>,
}

pub struct Context<C: Config, P: ProvingSystem<C>> {
    pub kernels: Pool<Kernel<C>>,
    pub device_memories: Vec<DeviceMemory<C, P>>,
    pub proofs: Vec<WrappedProof<C, P>>,
}

pub struct CombinedProof<C: Config, P: ProvingSystem<C>> {
    pub kernels: Vec<Kernel<C>>,
    pub commitments: Vec<P::Commitment>,
    pub proofs: Vec<WrappedProof<C, P>>,
}

impl<C: Config, P: ProvingSystem<C>> Default for Context<C, P> {
    fn default() -> Self {
        Context {
            kernels: Pool::new(),
            device_memories: vec![],
            proofs: vec![],
        }
    }
}

fn ensure_handle(handle: DeviceMemoryHandle) -> DeviceMemoryHandleRaw {
    match handle {
        Some(handle) => handle,
        None => panic!("Empty DeviceMemoryHandle"),
    }
}

fn pack_vec<C: Config>(v: &[C::CircuitField]) -> Vec<C::DefaultSimdField> {
    v.iter()
        .map(|x| {
            let mut v = Vec::with_capacity(C::DefaultSimdField::PACK_SIZE);
            for _ in 0..C::DefaultSimdField::PACK_SIZE {
                v.push(*x);
            }
            C::DefaultSimdField::pack(&v)
        })
        .collect::<Vec<_>>()
}

fn unpack_vec<C: Config>(v: &[C::DefaultSimdField]) -> Vec<C::CircuitField> {
    v.iter().map(|x| x.unpack()[0]).collect()
}

fn pad_vec<F: arith::Field>(v: &[F], dim0_size: usize) -> Vec<F> {
    assert_eq!(v.len() % dim0_size, 0);
    let dim1_size = v.len() / dim0_size;
    let padded_dim0_size = next_power_of_two(dim0_size);
    let padded_dim1_size = next_power_of_two(dim1_size);
    let mut padded = vec![F::zero(); padded_dim0_size * padded_dim1_size];
    for i in 0..dim0_size {
        for j in 0..dim1_size {
            padded[i * padded_dim1_size + j] = v[i * dim1_size + j];
        }
    }
    padded
}

fn dim0_size(shape: &Shape, is_broadcast: bool) -> usize {
    if let Some(shape) = shape {
        if is_broadcast || shape.is_empty() {
            1
        } else {
            shape[0]
        }
    } else {
        1
    }
}

fn broadcast_type(shape: &Shape, is_broadcast: bool) -> BroadcastType {
    if let Some(shape) = shape {
        if shape.iter().all(|x| *x > 0 && (x & (x - 1) == 0)) {
            BroadcastType::Both
        } else if is_broadcast {
            BroadcastType::BroadcastOnly
        } else {
            BroadcastType::NonBroadcastOnly
        }
    } else {
        BroadcastType::Both
    }
}

// returns (compatible, is_broadcast, parallel_count)
fn check_shape_compat(
    kernel_shape: &Shape,
    io_shape: &Shape,
    broadcast_type: BroadcastType,
) -> (bool, bool, Option<usize>) {
    if let Some(kernel_shape) = kernel_shape {
        if let Some(io_shape) = io_shape {
            if kernel_shape.len() == io_shape.len() {
                if *kernel_shape == *io_shape {
                    match broadcast_type {
                        BroadcastType::BroadcastOnly | BroadcastType::Both => (true, true, None),
                        BroadcastType::NonBroadcastOnly => (false, false, None),
                    }
                } else {
                    (false, false, None)
                }
            } else if kernel_shape.len() + 1 == io_shape.len() {
                if io_shape.iter().skip(1).eq(kernel_shape.iter()) {
                    match broadcast_type {
                        BroadcastType::BroadcastOnly => (false, false, None),
                        BroadcastType::NonBroadcastOnly | BroadcastType::Both => {
                            (true, false, Some(io_shape[0]))
                        }
                    }
                } else {
                    (false, false, None)
                }
            } else {
                (false, false, None)
            }
        } else {
            panic!("IO shape is not defined, you should use copy_to_device or copy_simd_to_device instead of copy_raw_* functions");
        }
    } else {
        panic!("Kernel shape is not defined, you should define kernel using macro, or use call_kernel_raw");
    }
}

/*
A reshape is acceptable if:
support the shape is [a,b,c]
let sequence of a*b*c,b*c,c be a1,a2,a3,... and other one be b1,b2,b3,...
there must exist n such that a1=b1, a2=b2, a3=b3, ..., an=bn
and for i>n, ai=2^k and bi=2^k for some k
*/
fn check_reshape_compat(shape: &[usize], new_shape: &[usize]) {
    let total = shape.iter().product::<usize>();
    let new_total = new_shape.iter().product::<usize>();
    if total != new_total {
        panic!("Total number of elements must be the same");
    }
    let mut i = 0;
    while i < shape.len() && i < new_shape.len() && shape[i] == new_shape[i] {
        i += 1;
    }
    for x in shape.iter().skip(i) {
        if *x & (*x - 1) != 0 {
            panic!("Incompatible shapes");
        }
    }
    for x in new_shape.iter().skip(i) {
        if *x & (*x - 1) != 0 {
            panic!("Incompatible shapes");
        }
    }
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

impl<C: Config, P: ProvingSystem<C>> Context<C, P> {
    fn make_device_mem(
        &mut self,
        values: Vec<C::DefaultSimdField>,
        padded_values: Vec<C::DefaultSimdField>,
        broadcast_type: BroadcastType,
        shape: Shape,
    ) -> DeviceMemoryHandle {
        let commitment = P::commit(&padded_values);
        self.device_memories.push(DeviceMemory {
            raw_values: values,
            values: padded_values,
            commitment,
        });
        Some(DeviceMemoryHandleRaw {
            id: self.device_memories.len() - 1,
            broadcast_type,
            shape,
        })
    }

    pub fn copy_raw_to_device(&mut self, host_memory: &[C::CircuitField]) -> DeviceMemoryHandle {
        let simd_host_memory = pack_vec::<C>(host_memory);
        self.make_device_mem(
            simd_host_memory.clone(),
            simd_host_memory,
            BroadcastType::Both,
            None,
        )
    }

    pub fn copy_raw_simd_to_device(
        &mut self,
        simd_host_memory: &[C::DefaultSimdField],
    ) -> DeviceMemoryHandle {
        self.make_device_mem(
            simd_host_memory.to_vec(),
            simd_host_memory.to_vec(),
            BroadcastType::Both,
            None,
        )
    }

    pub fn copy_to_device<T: VecShaped<C::CircuitField>>(
        &mut self,
        host_memory: &T,
        is_broadcast: bool,
    ) -> DeviceMemoryHandle {
        let (flat, shape) = flatten_shaped(host_memory);
        let shape = Some(shape);
        let simd_flat = pack_vec::<C>(&flat);
        let simd_pad = pad_vec(&simd_flat, dim0_size(&shape, is_broadcast));
        self.make_device_mem(
            simd_flat,
            simd_pad,
            broadcast_type(&shape, is_broadcast),
            shape,
        )
    }

    pub fn copy_simd_to_device<T: VecShaped<C::DefaultSimdField>>(
        &mut self,
        host_memory: &T,
        is_broadcast: bool,
    ) -> DeviceMemoryHandle {
        let (flat, shape) = flatten_shaped(host_memory);
        let shape = Some(shape);
        let pad = pad_vec(&flat, dim0_size(&shape, is_broadcast));
        self.make_device_mem(flat, pad, broadcast_type(&shape, is_broadcast), shape)
    }

    pub fn copy_raw_to_host(
        &self,
        device_memory_handle: DeviceMemoryHandle,
    ) -> Vec<C::CircuitField> {
        let device_memory_handle = ensure_handle(device_memory_handle);
        unpack_vec::<C>(&self.device_memories[device_memory_handle.id].values)
    }

    pub fn copy_raw_simd_to_host(
        &self,
        device_memory_handle: DeviceMemoryHandle,
    ) -> Vec<C::DefaultSimdField> {
        let device_memory_handle = ensure_handle(device_memory_handle);
        self.device_memories[device_memory_handle.id].values.clone()
    }

    pub fn copy_to_host<T: VecShaped<C::CircuitField> + Default>(
        &self,
        device_memory_handle: DeviceMemoryHandle,
    ) -> T {
        let device_memory_handle = ensure_handle(device_memory_handle);
        unflatten_shaped(
            &unpack_vec::<C>(&self.device_memories[device_memory_handle.id].raw_values),
            &device_memory_handle.shape.unwrap(),
        )
    }

    pub fn copy_simd_to_host<T: VecShaped<C::DefaultSimdField> + Default>(
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
        let mut check_results = Vec::with_capacity(ios.len());
        for ((kernel_shape, io), ws_input) in kernel
            .io_shapes
            .iter()
            .zip(ios.iter())
            .zip(kernel.witness_solver_io.iter())
        {
            if !ws_input.input_offset.is_some() {
                check_results.push((true, false, None));
                continue;
            }
            let (io_shape, broadcast_type) = if let Some(handle) = io {
                (handle.shape.clone(), handle.broadcast_type)
            } else {
                panic!("Missing input")
            };
            let chk = check_shape_compat(kernel_shape, &io_shape, broadcast_type);
            if !chk.0 {
                panic!("Incompatible shapes: {:?} {:?}", kernel_shape, io_shape);
            }
            check_results.push(chk);
        }
        println!("{:?}", check_results);
        let mut parallel_count = None;
        for (_, _, pc) in check_results.iter() {
            if let Some(pc) = pc {
                if let Some(parallel_count) = parallel_count {
                    if parallel_count != *pc {
                        panic!(
                            "Incompatible parallel counts: {:?} {:?}",
                            parallel_count, pc
                        );
                    }
                } else {
                    parallel_count = Some(*pc);
                }
            }
        }
        if parallel_count.is_none() {
            panic!("Parallel count is not defined");
        }
        self.call_kernel_raw(
            kernel,
            ios,
            parallel_count.unwrap(),
            &check_results.iter().map(|x| x.1).collect::<Vec<_>>(),
        );
    }

    pub fn call_kernel_raw(
        &mut self,
        kernel: &Kernel<C>,
        ios: &mut [DeviceMemoryHandle],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) {
        if kernel.witness_solver_io.len() != ios.len() {
            panic!("Invalid number of inputs/outputs");
        }
        if kernel.witness_solver_io.len() != is_broadcast.len() {
            panic!("Invalid number of is_broadcast");
        }
        if parallel_count == 0 {
            panic!("parallel_count must be at least 1");
        }
        // TODO: Is this necessary?
        // If it's not needed to pad the parallel count to a power of 2, we need to change other parts of the code
        /*if parallel_count & parallel_count - 1 != 0 {
            panic!("parallel_count must be a power of 2");
        }*/
        for i in 0..ios.len() {
            if is_broadcast[i] {
                assert!(kernel.witness_solver_io[i].output_offset.is_none());
                assert_eq!(
                    self.device_memories[ios[i].as_ref().unwrap().id]
                        .values
                        .len(),
                    next_power_of_two(kernel.witness_solver_io[i].len)
                );
            } else if kernel.witness_solver_io[i].input_offset.is_some() {
                assert_eq!(
                    self.device_memories[ios[i].as_ref().unwrap().id]
                        .values
                        .len(),
                    next_power_of_two(kernel.witness_solver_io[i].len)
                        * next_power_of_two(parallel_count)
                );
            }
        }

        let kernel_id = self.kernels.add(kernel);

        let mut handles = vec![];
        let mut lc_is_broadcast = vec![];
        for ((input, ws_input), ib) in ios
            .iter()
            .zip(kernel.witness_solver_io.iter())
            .zip(is_broadcast)
        {
            assert_eq!(input.is_some(), ws_input.input_offset.is_some());
            if input.is_some() {
                handles.push(input.clone().unwrap());
                lc_is_broadcast.push(*ib);
            }
        }

        let mut output_vecs = vec![vec![]; ios.len()];
        let mut output_vecs_raw = vec![vec![]; ios.len()];
        let mut hint_output_vec = vec![];

        for parallel_i in 0..parallel_count {
            let mut ws_inputs =
                vec![C::DefaultSimdField::zero(); kernel.witness_solver.input_size()];
            for (i, (input, ws_input)) in
                ios.iter().zip(kernel.witness_solver_io.iter()).enumerate()
            {
                if input.is_none() {
                    continue;
                }
                let device_memory = &self.device_memories[input.as_ref().unwrap().id];
                let offset = ws_input.input_offset.unwrap();
                if is_broadcast[i] {
                    for (i, x) in device_memory.values.iter().enumerate() {
                        ws_inputs[offset + i] = *x;
                    }
                } else {
                    for (i, x) in device_memory
                        .values
                        .iter()
                        .skip(parallel_i * next_power_of_two(ws_input.len))
                        .take(ws_input.len)
                        .enumerate()
                    {
                        ws_inputs[offset + i] = *x;
                    }
                }
            }
            let ws_outputs = kernel
                .witness_solver
                .eval_safe_simd(
                    ws_inputs,
                    &[],
                    &mut crate::hints::registry::HintRegistry::new(), // TODO: use null hint registry or enable hints
                )
                .unwrap(); // TODO: handle error
            for (i, ws_input) in kernel.witness_solver_io.iter().enumerate() {
                if ws_input.output_offset.is_none() {
                    continue;
                }
                let offset = ws_input.output_offset.unwrap();
                let values = &ws_outputs[offset..offset + ws_input.len];
                output_vecs[i].extend_from_slice(values);
                output_vecs_raw[i].extend_from_slice(values);
                for _ in ws_input.len..next_power_of_two(ws_input.len) {
                    output_vecs[i].push(C::DefaultSimdField::zero());
                }
            }
            if let Some(hint_io) = &kernel.witness_solver_hint_input {
                let values = &ws_outputs
                    [hint_io.output_offset.unwrap()..hint_io.output_offset.unwrap() + hint_io.len];
                hint_output_vec.extend_from_slice(values);
                for _ in hint_io.len..next_power_of_two(hint_io.len) {
                    hint_output_vec.push(C::DefaultSimdField::zero());
                }
            }
        }

        for ((((output, ws_input), mut ov), ov_raw), shape) in ios
            .iter_mut()
            .zip(kernel.witness_solver_io.iter())
            .zip(output_vecs.into_iter())
            .zip(output_vecs_raw.into_iter())
            .zip(kernel.io_shapes.iter())
        {
            if ws_input.output_offset.is_none() {
                *output = None;
                continue;
            }
            ov.resize(next_power_of_two(ov.len()), C::DefaultSimdField::zero());
            let commitment = P::commit(&ov);
            let device_memory = DeviceMemory {
                raw_values: ov_raw,
                values: ov,
                commitment,
            };
            self.device_memories.push(device_memory);
            let handle = DeviceMemoryHandleRaw {
                id: self.device_memories.len() - 1,
                broadcast_type: broadcast_type(shape, false),
                shape: shape_prepend(shape, parallel_count),
            };
            handles.push(handle.clone());
            *output = Some(handle);
            lc_is_broadcast.push(false);
        }
        if kernel.witness_solver_hint_input.is_some() {
            hint_output_vec.resize(
                next_power_of_two(hint_output_vec.len()),
                C::DefaultSimdField::zero(),
            );
            let commitment = P::commit(&hint_output_vec);
            let device_memory = DeviceMemory {
                raw_values: hint_output_vec.clone(),
                values: hint_output_vec,
                commitment,
            };
            self.device_memories.push(device_memory);
            let handle = DeviceMemoryHandleRaw {
                id: self.device_memories.len() - 1,
                broadcast_type: BroadcastType::Both,
                shape: None,
            };
            handles.push(handle);
            lc_is_broadcast.push(false);
        }
        let commitment_refs: Vec<_> = handles
            .iter()
            .map(|x| &self.device_memories[x.id].commitment)
            .collect();
        let proof = P::prove(
            kernel,
            &commitment_refs,
            next_power_of_two(parallel_count),
            &lc_is_broadcast,
        );
        self.proofs.push(WrappedProof {
            proof,
            kernel_id,
            commitment_indices: handles.iter().map(|x| x.id).collect(),
            parallel_count,
            is_broadcast: lc_is_broadcast,
        });
    }

    pub fn to_proof(self) -> CombinedProof<C, P> {
        CombinedProof {
            kernels: self.kernels.vec().clone(),
            commitments: self
                .device_memories
                .into_iter()
                .map(|x| x.commitment)
                .collect(),
            proofs: self.proofs,
        }
    }
}

impl<C: Config, P: ProvingSystem<C>> CombinedProof<C, P> {
    pub fn verify(&self) -> bool {
        for proof in self.proofs.iter() {
            if !P::verify(
                &self.kernels[proof.kernel_id],
                &proof.proof,
                &proof
                    .commitment_indices
                    .iter()
                    .map(|&x| &self.commitments[x])
                    .collect::<Vec<_>>(),
                next_power_of_two(proof.parallel_count),
                &proof.is_broadcast,
            ) {
                return false;
            }
        }
        true
    }
}

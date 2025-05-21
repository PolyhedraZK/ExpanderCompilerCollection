use arith::SimdField;
use expander_utils::timer::Timer;
use rayon::prelude::*;
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
    kernel::{shape_prepend, Kernel, Shape},
    proof::{ComputationGraph, ProofTemplate},
    proving_system::{ExpanderGKRProvingSystem, ProvingSystem},
    vec_shaped::{flatten_shaped, unflatten_shaped, VecShaped},
};

pub use macros::call_kernel;

pub struct DeviceMemory<C: Config, P: ProvingSystem<C>> {
    pub raw_values: Vec<SIMDField<C>>,
    pub values: Vec<SIMDField<C>>,
    pub _proving_system: std::marker::PhantomData<P>,
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

pub struct Context<
    C: Config,
    P: ProvingSystem<C> = ExpanderGKRProvingSystem<C>,
    H: HintCaller<CircuitField<C>> = EmptyHintCaller,
> {
    pub kernels: Pool<Kernel<C>>,
    pub device_memories: Vec<DeviceMemory<C, P>>,
    pub proof_templates: Vec<ProofTemplate>,
    pub hint_caller: H,
    pub _proving_system: std::marker::PhantomData<P>,
}

#[derive(ExpSerde)]
pub struct CombinedProof<C: Config, P: ProvingSystem<C> = ExpanderGKRProvingSystem<C>> {
    pub commitments: Vec<Vec<P::Commitment>>, // a vector of commitments for each kernel
    pub proofs: Vec<P::Proof>,
}

impl<C: Config, P: ProvingSystem<C>> Default for Context<C, P> {
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

impl<C: Config, P: ProvingSystem<C>, H: HintCaller<CircuitField<C>>> Context<C, P, H> {
    pub fn new(hint_caller: H) -> Self {
        Context {
            kernels: Pool::new(),
            device_memories: vec![],
            proof_templates: vec![],
            hint_caller,
            _proving_system: std::marker::PhantomData,
        }
    }

    fn make_device_mem(
        &mut self,
        values: Vec<SIMDField<C>>,
        padded_values: Vec<SIMDField<C>>,
        broadcast_type: BroadcastType,
        shape: Shape,
    ) -> DeviceMemoryHandle {
        self.device_memories.push(DeviceMemory {
            raw_values: values,
            values: padded_values,
            _proving_system: std::marker::PhantomData,
        });
        Some(DeviceMemoryHandleRaw {
            id: self.device_memories.len() - 1,
            broadcast_type,
            shape,
        })
    }

    pub fn copy_raw_to_device(&mut self, host_memory: &[CircuitField<C>]) -> DeviceMemoryHandle {
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
        simd_host_memory: &[SIMDField<C>],
    ) -> DeviceMemoryHandle {
        self.make_device_mem(
            simd_host_memory.to_vec(),
            simd_host_memory.to_vec(),
            BroadcastType::Both,
            None,
        )
    }

    pub fn copy_to_device<T: VecShaped<CircuitField<C>>>(
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

    pub fn copy_to_device_and_pack_simd<T: VecShaped<CircuitField<C>>>(
        &mut self,
        host_memory: &T,
        is_broadcast: bool,
    ) -> DeviceMemoryHandle {
        let (flat, shape) = flatten_shaped_pack_simd(host_memory);
        let shape = Some(shape);
        let pad = pad_vec(&flat, dim0_size(&shape, is_broadcast));
        self.make_device_mem(flat, pad, broadcast_type(&shape, is_broadcast), shape)
    }

    pub fn copy_simd_to_device<T: VecShaped<SIMDField<C>>>(
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
        let mut check_results = Vec::with_capacity(ios.len());
        for ((kernel_shape, io), ws_input) in kernel
            .io_shapes
            .iter()
            .zip(ios.iter())
            .zip(kernel.witness_solver_io.iter())
        {
            if ws_input.input_offset.is_none() {
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
            let mut ws_inputs = vec![SIMDField::<C>::zero(); kernel.witness_solver.input_size()];
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
                .eval_safe_simd(ws_inputs, &[], &mut self.hint_caller)
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
                    output_vecs[i].push(SIMDField::<C>::zero());
                }
            }
            if let Some(hint_io) = &kernel.witness_solver_hint_input {
                let values = &ws_outputs
                    [hint_io.output_offset.unwrap()..hint_io.output_offset.unwrap() + hint_io.len];
                hint_output_vec.extend_from_slice(values);
                for _ in hint_io.len..next_power_of_two(hint_io.len) {
                    hint_output_vec.push(SIMDField::<C>::zero());
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
            ov.resize(next_power_of_two(ov.len()), SIMDField::<C>::zero());
            let handle = self.make_device_mem(
                ov_raw,
                ov,
                broadcast_type(shape, false),
                shape_prepend(shape, parallel_count),
            );
            *output = handle.clone();
            handles.push(handle.unwrap());
            lc_is_broadcast.push(false);
        }
        if kernel.witness_solver_hint_input.is_some() {
            hint_output_vec.resize(
                next_power_of_two(hint_output_vec.len()),
                SIMDField::<C>::zero(),
            );
            let handle = self.make_device_mem(
                hint_output_vec.clone(),
                hint_output_vec,
                BroadcastType::Both,
                None,
            );
            handles.push(handle.unwrap());
            lc_is_broadcast.push(false);
        }
        self.proof_templates.push(ProofTemplate {
            kernel_id,
            commitment_indices: handles.iter().map(|x| x.id).collect(),
            parallel_count,
            is_broadcast: lc_is_broadcast,
        });
    }

    pub fn to_computation_graph(&self) -> ComputationGraph<C> {
        ComputationGraph {
            kernels: self.kernels.vec().clone(),
            commitments_lens: self
                .device_memories
                .iter()
                .map(|x| x.values.len())
                .collect(),
            proof_templates: self.proof_templates.clone(),
        }
    }

    pub fn proving_system_setup(
        &self,
        computation_graph: &ComputationGraph<C>,
    ) -> (P::ProverSetup, P::VerifierSetup) {
        P::setup(computation_graph)
    }

    pub fn to_proof(self, prover_setup: &P::ProverSetup) -> CombinedProof<C, P> {
        let commitments = self
            .proof_templates
            .iter()
            .map(|template| {
                template
                    .commitment_indices
                    .iter()
                    .zip(template.is_broadcast.iter())
                    .map(|(x, is_broadcast)| {
                        P::commit(
                            prover_setup,
                            &self.device_memories[*x].values,
                            next_power_of_two(template.parallel_count),
                            *is_broadcast,
                        )
                    })
                    .unzip::<_, _, Vec<_>, Vec<_>>()
            })
            .collect::<Vec<_>>();

        let proofs: Vec<P::Proof> = self
            .proof_templates
            .iter()
            .zip(commitments.iter())
            .map(|(template, commitments_kernel)| {
                P::prove(
                    prover_setup,
                    self.kernels.get(template.kernel_id),
                    &commitments_kernel.0,
                    &commitments_kernel.1,
                    &template
                        .commitment_indices
                        .iter()
                        .map(|x| &self.device_memories[*x].values[..])
                        .collect::<Vec<_>>(),
                    next_power_of_two(template.parallel_count),
                    &template.is_broadcast,
                )
            })
            .collect::<Vec<_>>();

        CombinedProof {
            commitments: commitments.into_iter().map(|x| x.0).collect(),
            proofs,
        }
    }
}

impl<C: Config> ComputationGraph<C> {
    pub fn verify<P: ProvingSystem<C>>(
        &self,
        combined_proof: &CombinedProof<C, P>,
        verifier_setup: &P::VerifierSetup,
    ) -> bool {
        // TODO: Add a proper check
        // for (commitment, len) in combined_proof
        //     .commitments
        //     .iter()
        //     .zip(self.commitments_lens.iter())
        // {
        //     if commitment.vals_len() != *len {
        //         return false;
        //     }
        // }

        let timer = Timer::new("Total Verification Time", true);
        let verified = combined_proof
            .proofs
            .par_iter()
            .zip(self.proof_templates.par_iter())
            .zip(combined_proof.commitments.par_iter())
            .map(|((proof, template), commitments_kernel)| {
                P::verify(
                    verifier_setup,
                    &self.kernels[template.kernel_id],
                    proof,
                    commitments_kernel,
                    next_power_of_two(template.parallel_count),
                    &template.is_broadcast,
                )
            })
            .collect::<Vec<_>>();

        timer.stop();
        verified.iter().all(|x| *x)
    }
}

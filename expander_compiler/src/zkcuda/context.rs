use arith::SimdField;

use crate::field::FieldArith;
use crate::utils::misc::next_power_of_two;
use crate::{circuit::config::Config, utils::pool::Pool};

use super::{kernel::Kernel, proving_system::ProvingSystem};

pub struct DeviceMemory<C: Config, P: ProvingSystem<C>> {
    pub values: Vec<C::DefaultSimdField>,
    pub commitment: P::Commitment,
}

pub type DeviceMemoryHandle = usize;

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

impl<C: Config, P: ProvingSystem<C>> Context<C, P> {
    pub fn copy_to_device(&mut self, host_memory: &[C::CircuitField]) -> DeviceMemoryHandle {
        // TODO: this is a temporary implementation
        // In the real implementation, the SIMD segmentation should be done here
        let simd_host_memory = host_memory
            .iter()
            .map(|x| {
                let mut v = Vec::new();
                for _ in 0..C::DefaultSimdField::PACK_SIZE {
                    v.push(*x);
                }
                C::DefaultSimdField::pack(&v)
            })
            .collect::<Vec<_>>();
        self.device_memories.push(DeviceMemory {
            values: simd_host_memory.to_vec(),
            commitment: P::commit(&simd_host_memory),
        });
        self.device_memories.len() - 1
    }

    pub fn copy_to_host(&self, device_memory_handle: DeviceMemoryHandle) -> Vec<C::CircuitField> {
        // TODO: this is a temporary implementation
        self.device_memories[device_memory_handle]
            .values
            .iter()
            .map(|x| x.unpack()[0])
            .collect()
    }

    pub fn call_kernel(
        &mut self,
        kernel: &Kernel<C>,
        ios: &mut [Option<DeviceMemoryHandle>],
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
                    self.device_memories[ios[i].unwrap()].values.len(),
                    next_power_of_two(kernel.witness_solver_io[i].len)
                );
            } else if kernel.witness_solver_io[i].input_offset.is_some() {
                assert_eq!(
                    self.device_memories[ios[i].unwrap()].values.len(),
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
                handles.push(input.unwrap());
                lc_is_broadcast.push(*ib);
            }
        }

        let mut output_vecs = vec![vec![]; ios.len()];
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
                let device_memory = &self.device_memories[input.unwrap()];
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

        for ((output, ws_input), mut ov) in ios
            .iter_mut()
            .zip(kernel.witness_solver_io.iter())
            .zip(output_vecs.into_iter())
        {
            if ws_input.output_offset.is_none() {
                *output = None;
                continue;
            }
            ov.resize(next_power_of_two(ov.len()), C::DefaultSimdField::zero());
            let commitment = P::commit(&ov);
            let device_memory = DeviceMemory {
                values: ov,
                commitment,
            };
            self.device_memories.push(device_memory);
            handles.push(self.device_memories.len() - 1);
            *output = Some(self.device_memories.len() - 1);
            lc_is_broadcast.push(false);
        }
        if kernel.witness_solver_hint_input.is_some() {
            hint_output_vec.resize(
                next_power_of_two(hint_output_vec.len()),
                C::DefaultSimdField::zero(),
            );
            let commitment = P::commit(&hint_output_vec);
            let device_memory = DeviceMemory {
                values: hint_output_vec,
                commitment,
            };
            self.device_memories.push(device_memory);
            handles.push(self.device_memories.len() - 1);
            lc_is_broadcast.push(false);
        }
        let commitment_refs: Vec<_> = handles
            .iter()
            .map(|&x| &self.device_memories[x].commitment)
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
            commitment_indices: handles,
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

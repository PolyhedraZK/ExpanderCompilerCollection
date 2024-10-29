use crate::circuit::config::Config;
use crate::field::FieldArith;

use super::{kernel::Kernel, proving_system::ProvingSystem};

pub struct DeviceMemory<C: Config, P: ProvingSystem<C>> {
    pub values: Vec<C::CircuitField>,
    pub commitment: P::Commitment,
}

pub type DeviceMemoryHandle = usize;

pub struct Context<C: Config, P: ProvingSystem<C>> {
    pub device_memories: Vec<DeviceMemory<C, P>>,
    pub proofs: Vec<P::Proof>,
}

pub struct CombinedProof<C: Config, P: ProvingSystem<C>> {
    pub commitments: Vec<P::Commitment>,
    pub proofs: Vec<P::Proof>,
}

impl<C: Config, P: ProvingSystem<C>> Context<C, P> {
    pub fn new() -> Self {
        Self {
            device_memories: vec![],
            proofs: vec![],
        }
    }

    pub fn copy_to_device(&mut self, host_memory: &[C::CircuitField]) -> DeviceMemoryHandle {
        self.device_memories.push(DeviceMemory {
            values: host_memory.to_vec(),
            commitment: P::commit(host_memory),
        });
        self.device_memories.len() - 1
    }

    pub fn copy_to_host(&self, device_memory_handle: DeviceMemoryHandle) -> Vec<C::CircuitField> {
        self.device_memories[device_memory_handle].values.clone()
    }

    pub fn call_kernel(&mut self, kernel: &Kernel<C>, ios: &mut [Option<DeviceMemoryHandle>]) {
        let mut ws_inputs = vec![C::CircuitField::zero(); kernel.witness_solver.input_size()];
        let mut handles = vec![];
        for (input, ws_input) in ios.iter().zip(kernel.witness_solver_io.iter()) {
            assert_eq!(input.is_some(), ws_input.input_offset.is_some());
            if input.is_none() {
                continue;
            }
            handles.push(input.unwrap());
            let device_memory = &self.device_memories[input.unwrap()];
            assert_eq!(device_memory.values.len(), ws_input.len);
            let offset = ws_input.input_offset.unwrap();
            for (i, x) in device_memory.values.iter().enumerate() {
                ws_inputs[offset + i] = *x;
            }
        }
        let ws_outputs = kernel
            .witness_solver
            .eval_with_public_inputs(ws_inputs, &[])
            .unwrap(); // TODO: handle error2
        for (output, ws_input) in ios.iter_mut().zip(kernel.witness_solver_io.iter()) {
            if ws_input.output_offset.is_none() {
                *output = None;
                continue;
            }
            let offset = ws_input.output_offset.unwrap();
            let values = ws_outputs[offset..offset + ws_input.len].to_vec();
            let commitment = P::commit(&values);
            let device_memory = DeviceMemory { values, commitment };
            self.device_memories.push(device_memory);
            handles.push(self.device_memories.len() - 1);
            *output = Some(self.device_memories.len() - 1);
        }
        if let Some(hint_io) = &kernel.witness_solver_hint_input {
            let values = ws_outputs
                [hint_io.output_offset.unwrap()..hint_io.output_offset.unwrap() + hint_io.len]
                .to_vec();
            let commitment = P::commit(&values);
            let device_memory = DeviceMemory { values, commitment };
            self.device_memories.push(device_memory);
            handles.push(self.device_memories.len() - 1);
        }
        let commitment_refs: Vec<_> = handles
            .iter()
            .map(|&x| &self.device_memories[x].commitment)
            .collect();
        let proof = P::prove(kernel, &commitment_refs);
        self.proofs.push(proof);
        // TODO: encode commitment id in proof
    }

    pub fn get_proof(&self) -> CombinedProof<C, P> {
        CombinedProof {
            commitments: self
                .device_memories
                .iter()
                .map(|x| x.commitment.clone())
                .collect(),
            proofs: self.proofs.clone(),
        }
    }
}

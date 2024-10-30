use core::panic;

use crate::circuit::config::Config;
use crate::field::FieldArith;

use super::kernel::Kernel;

pub trait Commitment<C: Config>: Clone {}

pub trait Proof: Clone {}

pub trait ProvingSystem<C: Config> {
    type Proof: Proof;
    type Commitment: Commitment<C>;
    fn commit(vals: &[C::CircuitField]) -> Self::Commitment;
    fn prove(
        kernel: &Kernel<C>,
        commitments: &[&Self::Commitment],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> Self::Proof;
    fn verify(
        kernel: &Kernel<C>,
        proof: &Self::Proof,
        commitments: &[&Self::Commitment],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> bool;
}

// dummy implementation of these traits

#[derive(Clone)]
pub struct DummyCommitment<C: Config> {
    vals: Vec<C::CircuitField>,
}

impl<C: Config> Commitment<C> for DummyCommitment<C> {}

#[derive(Clone)]
pub struct DummyProof {
    cond: Vec<bool>,
}

impl Proof for DummyProof {}

pub struct DummyProvingSystem<C: Config> {
    _config: std::marker::PhantomData<C>,
}

impl<C: Config> ProvingSystem<C> for DummyProvingSystem<C> {
    type Proof = DummyProof;
    type Commitment = DummyCommitment<C>;
    fn commit(vals: &[C::CircuitField]) -> Self::Commitment {
        DummyCommitment {
            vals: vals.to_vec(),
        }
    }
    fn prove(
        kernel: &Kernel<C>,
        commitments: &[&Self::Commitment],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> DummyProof {
        dummy_check_inputs(kernel, commitments, parallel_count, is_broadcast);
        let mut res = vec![];
        for i in 0..parallel_count {
            let lc_input = dummy_prepare_inputs(kernel, commitments, is_broadcast, i);
            let (_, cond) = kernel
                .layered_circuit
                .eval_with_public_inputs(lc_input, &[]);
            res.push(cond);
        }
        DummyProof { cond: res }
    }
    fn verify(
        kernel: &Kernel<C>,
        proof: &Self::Proof,
        commitments: &[&Self::Commitment],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> bool {
        dummy_check_inputs(kernel, commitments, parallel_count, is_broadcast);
        for i in 0..parallel_count {
            let lc_input = dummy_prepare_inputs(kernel, commitments, is_broadcast, i);
            let (_, cond) = kernel
                .layered_circuit
                .eval_with_public_inputs(lc_input, &[]);
            if cond != proof.cond[i] {
                return false;
            }
        }
        true
    }
}

fn dummy_check_inputs<C: Config>(
    kernel: &Kernel<C>,
    commitments: &[&DummyCommitment<C>],
    parallel_count: usize,
    is_broadcast: &[bool],
) {
    if kernel.layered_circuit_input.len() != commitments.len() {
        panic!("Input size mismatch");
    }
    if kernel.layered_circuit_input.len() != is_broadcast.len() {
        panic!("Input size mismatch");
    }
    for i in 0..kernel.layered_circuit_input.len() {
        if is_broadcast[i] {
            if kernel.layered_circuit_input[i].len != commitments[i].vals.len() {
                panic!("Input size mismatch");
            }
        } else if kernel.layered_circuit_input[i].len * parallel_count != commitments[i].vals.len()
        {
            panic!("Input size mismatch");
        }
    }
}

fn dummy_prepare_inputs<C: Config>(
    kernel: &Kernel<C>,
    commitments: &[&DummyCommitment<C>],
    is_broadcast: &[bool],
    parallel_index: usize,
) -> Vec<C::CircuitField> {
    let mut lc_input = vec![C::CircuitField::zero(); kernel.layered_circuit.input_size()];
    for ((input, commitment), ib) in kernel
        .layered_circuit_input
        .iter()
        .zip(commitments.iter())
        .zip(is_broadcast)
    {
        if *ib {
            for (i, x) in commitment.vals.iter().enumerate() {
                lc_input[input.offset + i] = *x;
            }
        } else {
            for (i, x) in commitment
                .vals
                .iter()
                .skip(parallel_index * input.len)
                .take(input.len)
                .enumerate()
            {
                lc_input[input.offset + i] = *x;
            }
        }
    }
    lc_input
}

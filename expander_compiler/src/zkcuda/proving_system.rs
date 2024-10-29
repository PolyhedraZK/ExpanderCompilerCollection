use crate::circuit::config::Config;
use crate::field::FieldArith;

use super::kernel::Kernel;

pub trait Commitment<C: Config>: Clone {}

pub trait Proof: Clone {}

pub trait ProvingSystem<C: Config> {
    type Proof: Proof;
    type Commitment: Commitment<C>;
    fn commit(vals: &[C::CircuitField]) -> Self::Commitment;
    fn prove(kernel: &Kernel<C>, commitments: &[&Self::Commitment]) -> Self::Proof;
    fn verify(kernel: &Kernel<C>, proof: Self::Proof, commitments: &[&Self::Commitment]) -> bool;
}

// dummy implementation of these traits

#[derive(Clone)]
pub struct DummyCommitment<C: Config> {
    vals: Vec<C::CircuitField>,
}

impl<C: Config> Commitment<C> for DummyCommitment<C> {}

#[derive(Clone)]
pub struct DummyProof {}

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
    fn prove(_kernel: &Kernel<C>, _commitments: &[&Self::Commitment]) -> Self::Proof {
        DummyProof {}
    }
    fn verify(kernel: &Kernel<C>, _proof: Self::Proof, commitments: &[&Self::Commitment]) -> bool {
        if kernel.layered_circuit_input.len() != commitments.len() {
            return false;
        }
        let mut lc_input = vec![C::CircuitField::zero(); kernel.layered_circuit.input_size()];
        for (input, commitment) in kernel.layered_circuit_input.iter().zip(commitments.iter()) {
            if input.len != commitment.vals.len() {
                return false;
            }
            for (i, x) in commitment.vals.iter().enumerate() {
                lc_input[input.offset + i] = *x;
            }
        }
        let (_, cond) = kernel
            .layered_circuit
            .eval_with_public_inputs(lc_input, &[]);
        cond
    }
}

use crate::circuit::config::Config;

use super::{Commitment, ProvingSystem, Proof, check_inputs, prepare_inputs};
use super::super::kernel::Kernel;

#[derive(Clone)]
pub struct DummyCommitment<C: Config> {
    vals: Vec<C::CircuitField>,
}

impl<C: Config> Commitment<C> for DummyCommitment<C> {
    fn vals_ref(&self) -> &[C::CircuitField] {
        &self.vals
    }
}

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
        assert!(vals.len() & (vals.len() - 1) == 0);
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
        check_inputs(kernel, commitments, parallel_count, is_broadcast);
        let mut res = vec![];
        for i in 0..parallel_count {
            let lc_input = prepare_inputs(kernel, commitments, is_broadcast, i);
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
        check_inputs(kernel, commitments, parallel_count, is_broadcast);
        for i in 0..parallel_count {
            let lc_input = prepare_inputs(kernel, commitments, is_broadcast, i);
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

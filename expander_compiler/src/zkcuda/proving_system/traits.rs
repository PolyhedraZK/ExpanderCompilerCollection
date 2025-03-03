use super::super::kernel::Kernel;
use crate::circuit::config::Config;

pub trait Commitment<C: Config>: Clone {
    fn vals_ref(&self) -> &[C::DefaultSimdField];

    fn vals_len(&self) -> usize {
        self.vals_ref().len()
    }
}

pub trait Proof: Clone {}

pub trait ProvingSystem<C: Config> {
    type Proof: Proof;
    type Commitment: Commitment<C>;
    fn commit(vals: &[C::DefaultSimdField]) -> Self::Commitment;
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

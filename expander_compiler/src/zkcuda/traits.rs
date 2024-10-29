use crate::circuit::config::Config;

use super::kernel::Kernel;

pub trait Commitment<C: Config> {}

pub trait Proof {}

pub trait ProvingSystem<C: Config> {
    type Proof: Proof;
    type Commitment: Commitment<C>;
    fn commit(vals: &[C::CircuitField]) -> Self::Commitment;
    fn prove(kernel: &Kernel<C>, commitments: &[Self::Commitment]) -> Self::Proof;
    fn verify(kernel: &Kernel<C>, proof: Self::Proof, commitments: &[Self::Commitment]) -> bool;
}

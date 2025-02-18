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

    /// `kernel`: contains some basic information such as circuit structure.
    /// `commitments`: the components/values that constitute the input to the circuit, 
    ///     In case of parallel_count = 1: the sum of the lengths of the commitments must be equal to 
    ///         the total number of inputs to the circuit
    ///     In case of parallel_count > 1: the length of each component is multiplied by parallel_count
    ///         if the corresponding is_broadcast is false
    /// `parallel_count`: the number of instances of the circuit that are being run in parallel
    /// `is_broadcast`: must have the same length as `commitments`, indicating whether the corresponding 
    ///     component is the same across all the `parallel_count` instances of the circuit
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

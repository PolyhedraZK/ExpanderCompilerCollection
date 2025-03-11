use super::super::kernel::Kernel;
use crate::circuit::config::Config;

pub trait ProvingSystem<C: Config> {
    type ProverSetup: Clone;
    type VerifierSetup: Clone;
    type Proof: Clone;
    type Commitment: Clone;
    type CommitmentExtraInfo: Clone;

    // TODO: the setup function should take the problem description as an input, 
    // probably the context or all the kernels?
    fn setup() -> (Self::ProverSetup, Self::VerifierSetup); 
    
    fn commit(vals: &[C::DefaultSimdField], prover_setup: &Self::ProverSetup) -> (Self::Commitment, Self::CommitmentExtraInfo);
    fn prove(
        prover_setup: &Self::ProverSetup,
        kernel: &Kernel<C>,
        vals: &[C::DefaultSimdField],
        commitments: &[&Self::Commitment],
        commitments_extra_info: &[Self::CommitmentExtraInfo],
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

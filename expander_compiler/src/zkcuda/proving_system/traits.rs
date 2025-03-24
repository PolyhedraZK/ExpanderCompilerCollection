use super::super::{kernel::Kernel, proof::ComputationGraph};

use crate::circuit::config::Config;

pub trait Commitment<C: Config>: Clone {
    fn vals_len(&self) -> usize;
}

pub trait ProvingSystem<C: Config> {
    type ProverSetup: Clone;
    type VerifierSetup: Clone;
    type Proof: Clone;
    type Commitment: Commitment<C>;
    type CommitmentExtraInfo: Clone;

    fn setup(computation_graph: &ComputationGraph<C>) -> (Self::ProverSetup, Self::VerifierSetup);

    fn commit(
        prover_setup: &Self::ProverSetup,
        vals: &Vec<C::DefaultSimdField>,
    ) -> (Self::Commitment, Self::CommitmentExtraInfo);

    fn prove(
        prover_setup: &Self::ProverSetup,
        kernel: &Kernel<C>,
        commitments: &[&Self::Commitment],
        commitments_extra_info: &[&Self::CommitmentExtraInfo],
        commitments_values: &[&[C::DefaultSimdField]],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> Self::Proof;

    fn verify(
        verifier_setup: &Self::VerifierSetup,
        kernel: &Kernel<C>,
        proof: &Self::Proof,
        commitments: &[&Self::Commitment],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> bool;
}

use serdes::ExpSerde;

use super::super::{kernel::Kernel, proof::ComputationGraph};

use crate::circuit::config::{Config, SIMDField};

pub trait Commitment<C: Config>: Clone + ExpSerde {
    fn vals_len(&self) -> usize;
}

pub trait Proof: Clone + ExpSerde {}

pub trait ProvingSystem<C: Config> {
    type ProverSetup: Clone;
    type VerifierSetup: Clone;
    type Proof: Proof;
    type Commitment: Commitment<C>;
    type CommitmentExtraInfo: Clone;

    fn setup(computation_graph: &ComputationGraph<C>) -> (Self::ProverSetup, Self::VerifierSetup);

    fn commit(
        prover_setup: &Self::ProverSetup,
        vals: &[SIMDField<C>],
        parallel_count: usize,
        is_broadcast: bool,
    ) -> (Self::Commitment, Self::CommitmentExtraInfo);

    fn prove(
        prover_setup: &Self::ProverSetup,
        kernel: &Kernel<C>,
        commitments: &[Self::Commitment],
        commitments_extra_info: &[Self::CommitmentExtraInfo],
        commitments_values: &[&[SIMDField<C>]],
        commitments_bit_order: &[Option<Vec<usize>>],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> Self::Proof;

    fn verify(
        verifier_setup: &Self::VerifierSetup,
        kernel: &Kernel<C>,
        proof: &Self::Proof,
        commitments: &[Self::Commitment],
        commitments_bit_order: &[Option<Vec<usize>>],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> bool;
}

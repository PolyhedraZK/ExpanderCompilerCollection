use serdes::ExpSerde;

use crate::circuit::config::{Config, SIMDField};

use super::super::kernel::Kernel;
use super::{check_inputs, prepare_inputs, Commitment, Proof, ProvingSystem};

// dummy implementation of these traits

#[derive(Clone, ExpSerde)]
pub struct DummyCommitment<C: Config> {
    vals: Vec<SIMDField<C>>,
}

impl<C: Config> Commitment<C> for DummyCommitment<C> {
    fn vals_len(&self) -> usize {
        self.vals.len()
    }
}

#[derive(Clone, ExpSerde)]
pub struct DummyProof {
    cond: Vec<Vec<bool>>,
}

impl Proof for DummyProof {}

// TODO
/*#[deprecated(
    note = "DummyProvingSystem is a dummy implementation for testing purposes. Please use ExpanderGKRProvingSystem."
)]*/
pub struct DummyProvingSystem<C: Config> {
    _config: std::marker::PhantomData<C>,
}

#[allow(deprecated)]
impl<C: Config> ProvingSystem<C> for DummyProvingSystem<C> {
    type ProverSetup = ();
    type VerifierSetup = ();
    type Proof = DummyProof;
    type Commitment = DummyCommitment<C>;
    type CommitmentExtraInfo = ();

    fn setup(
        computation_graph: &crate::zkcuda::proof::ComputationGraph<C>,
    ) -> (Self::ProverSetup, Self::VerifierSetup) {
        // let _ = computation_graph;
        computation_graph
            .commitments_lens
            .iter()
            .for_each(|&x| println!("Setup length {}", x));

        ((), ())
    }

    fn commit(
        _prover_setup: &Self::ProverSetup,
        vals: &[SIMDField<C>],
        _parallel_count: usize,
        _is_broadcast: bool,
    ) -> (Self::Commitment, Self::CommitmentExtraInfo) {
        assert!(vals.len() & (vals.len() - 1) == 0);
        (
            DummyCommitment {
                vals: vals.to_vec(),
            },
            (),
        )
    }

    fn prove(
        _prover_setup: &Self::ProverSetup,
        kernel: &Kernel<C>,
        _commitments: &[Self::Commitment],
        _commitments_extra_info: &[Self::CommitmentExtraInfo],
        commitments_values: &[&[SIMDField<C>]],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> DummyProof {
        check_inputs(kernel, commitments_values, parallel_count, is_broadcast);
        let mut res = vec![];
        for i in 0..parallel_count {
            let lc_input = prepare_inputs(
                &kernel.layered_circuit,
                &kernel.layered_circuit_input,
                commitments_values,
                is_broadcast,
                i,
            );
            let (_, cond) = kernel
                .layered_circuit
                .eval_with_public_inputs_simd(lc_input, &[]);
            for x in cond.iter() {
                if !*x {
                    panic!("constraints not satisfied");
                }
            }
            res.push(cond);
        }
        DummyProof { cond: res }
    }
    fn verify(
        _verifier_setup: &Self::VerifierSetup,
        kernel: &Kernel<C>,
        proof: &Self::Proof,
        commitments: &[Self::Commitment],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> bool {
        let values = commitments.iter().map(|c| &c.vals[..]).collect::<Vec<_>>();
        check_inputs(kernel, &values, parallel_count, is_broadcast);
        for i in 0..parallel_count {
            let lc_input = prepare_inputs(
                &kernel.layered_circuit,
                &kernel.layered_circuit_input,
                &values,
                is_broadcast,
                i,
            );
            let (_, cond) = kernel
                .layered_circuit
                .eval_with_public_inputs_simd(lc_input, &[]);
            if cond != proof.cond[i] {
                return false;
            }
        }
        true
    }
}

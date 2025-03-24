use crate::circuit::config::Config;

use super::super::kernel::Kernel;
use super::{check_inputs, prepare_inputs, Commitment, ProvingSystem};

// dummy implementation of these traits

#[derive(Clone)]
pub struct DummyCommitment<C: Config> {
    vals: Vec<C::DefaultSimdField>,
}

impl<C: Config> Commitment<C> for DummyCommitment<C> {
    fn vals_len(&self) -> usize {
        self.vals.len()
    }
}

#[derive(Clone)]
pub struct DummyProof {
    cond: Vec<Vec<bool>>,
}

/*#[deprecated(
    note = "DummyProvingSystem is a dummy implementation for testing purposes. Please use ExpanderGKRProvingSystem."
)]*/
// FIXME: after Zhiyong finishes the implementation, change back
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

    fn setup(computation_graph: &crate::zkcuda::proof::ComputationGraph<C>) -> (Self::ProverSetup, Self::VerifierSetup) {
        // let _ = computation_graph;
        computation_graph.commitments_lens.iter().for_each(|&x| println!("Setup length {}", x));
        
        ((), ())
    }

    fn commit(
        _prover_setup: &Self::ProverSetup,
        vals: &Vec<<C as Config>::DefaultSimdField>,
    ) -> (Self::Commitment, Self::CommitmentExtraInfo) {
        println!("Real Commit to {} values", vals.len());
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
        _commitments: &[&Self::Commitment],
        _commitments_extra_info: &[&Self::CommitmentExtraInfo],
        commitments_values: &[&[C::DefaultSimdField]],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> DummyProof {
        check_inputs(kernel, commitments_values, parallel_count, is_broadcast);
        let mut res = vec![];
        for i in 0..parallel_count {
            let lc_input = prepare_inputs(kernel, commitments_values, is_broadcast, i);
            let (_, cond) = kernel
                .layered_circuit
                .eval_with_public_inputs_simd(lc_input, &[]);
            res.push(cond);
        }
        DummyProof { cond: res }
    }
    fn verify(
        _verifier_setup: &Self::VerifierSetup,
        kernel: &Kernel<C>,
        proof: &Self::Proof,
        commitments: &[&Self::Commitment],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> bool {
        let values = commitments.iter().map(|c| &c.vals[..]).collect::<Vec<_>>();
        check_inputs(kernel, &values, parallel_count, is_broadcast);
        for i in 0..parallel_count {
            let lc_input = prepare_inputs(kernel, &values, is_broadcast, i);
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

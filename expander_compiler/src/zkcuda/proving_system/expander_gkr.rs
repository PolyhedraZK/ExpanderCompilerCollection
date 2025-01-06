use crate::circuit::config::Config;

use super::{Commitment, ProvingSystem, Proof};
use super::super::kernel::Kernel;

use expander_config::{GKRConfig, Config as ExpanderConfig};
use gkr::Prover;
use polynomials::MultiLinearPoly;
use mpi_config::MPIConfig;
use poly_commit::{raw::*, PCSEmptyType, PolynomialCommitmentScheme};
use gkr_field_config::GKRFieldConfig;
use expander_transcript::{Transcript, Proof as ExpanderProof};

#[derive(Clone)]
pub struct ExpanderGKRCommitment<C: Config>
{
    vals: Vec<C::CircuitField>,
    commitment: RawCommitment<C::CircuitField>,
    scratch: RawMultiLinearScratchPad<C::CircuitField>,
}

impl<C: Config> Commitment<C> for ExpanderGKRCommitment<C> {
    fn vals_ref(&self) -> &[<C as Config>::CircuitField] {
        &self.vals
    }
}

#[derive(Clone)]
pub struct ExpanderGKRProof {
    proof: ExpanderProof,
}

impl Proof for ExpanderGKRProof {}

pub struct ExpanderGKRProvingSystem<C: Config>
{
    _config: std::marker::PhantomData<C>,
}

impl<C: Config> ProvingSystem<C> for ExpanderGKRProvingSystem<C>
{
    type Proof = ExpanderGKRProof;
    type Commitment = ExpanderGKRCommitment<C>;

    fn commit(vals: &[C::CircuitField]) -> Self::Commitment {
        assert!(vals.len() & (vals.len() - 1) == 0);

        let params = RawMultiLinearParams { n_vars: vals.len().trailing_zeros() as usize };
        let poly = MultiLinearPoly::new(vals.to_vec());
        let mut pcs_scratch = RawMultiLinearPCS::init_scratch_pad(&params);
        let commitment = RawMultiLinearPCS::commit(&params, &PCSEmptyType::default(), &poly, &mut pcs_scratch);
        ExpanderGKRCommitment {
            vals: vals.to_vec(),
            commitment,
            scratch: pcs_scratch,
        }
    }

    fn prove(
        kernel: &Kernel<C>,
        commitments: &[&Self::Commitment],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> ExpanderGKRProof {
        let expander_config = ExpanderConfig::<C::DefaultGKRConfig>::new(expander_config::GKRScheme::Vanilla, MPIConfig::new());
        let mut prover = Prover::new(&expander_config);
        let mut expander_circuit = kernel.layered_circuit.export_to_expander().flatten();
        prover.prepare_mem(&expander_circuit);
        let mut transcript = <C::DefaultGKRConfig as GKRConfig>::Transcript::new();

        unimplemented!()
    }
    fn verify(
        kernel: &Kernel<C>,
        proof: &Self::Proof,
        commitments: &[&Self::Commitment],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> bool {
        // dummy_check_inputs(kernel, commitments, parallel_count, is_broadcast);
        // for i in 0..parallel_count {
        //     let lc_input = dummy_prepare_inputs(kernel, commitments, is_broadcast, i);
        //     let (_, cond) = kernel
        //         .layered_circuit
        //         .eval_with_public_inputs(lc_input, &[]);
        //     if cond != proof.cond[i] {
        //         return false;
        //     }
        // }
        // true
        unimplemented!()
    }
}
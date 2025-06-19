use crate::circuit::config::Config;
use crate::utils::misc::next_power_of_two;
use crate::zkcuda::proof::ComputationGraph;
use crate::zkcuda::proving_system::expander::structs::{
    ExpanderProverSetup, ExpanderVerifierSetup,
};
use crate::zkcuda::proving_system::expander_parallelized::client_utils::{
    client_launch_server_and_setup, client_send_witness_and_prove, wait_async, ClientHttpHelper,
};
use crate::zkcuda::proving_system::expander_parallelized::verify_impl::verify_kernel;
use crate::zkcuda::proving_system::{CombinedProof, ProvingSystem};

use super::super::Expander;

use gkr_engine::{FieldEngine, GKREngine};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

pub struct ParallelizedExpander<C: GKREngine> {
    _config: std::marker::PhantomData<C>,
}

impl<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> ProvingSystem<ECCConfig>
    for ParallelizedExpander<C>
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    type ProverSetup = ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>;
    type VerifierSetup = ExpanderVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>;
    type Proof = CombinedProof<ECCConfig, Expander<C>>;

    fn setup(
        computation_graph: &crate::zkcuda::proof::ComputationGraph<ECCConfig>,
    ) -> (Self::ProverSetup, Self::VerifierSetup) {
        client_launch_server_and_setup::<C, ECCConfig>(
            "../target/release/expander_server",
            computation_graph,
        )
    }

    fn prove(
        _prover_setup: &Self::ProverSetup,
        _computation_graph: &crate::zkcuda::proof::ComputationGraph<ECCConfig>,
        device_memories: &[crate::zkcuda::context::DeviceMemory<ECCConfig>],
    ) -> Self::Proof {
        client_send_witness_and_prove(device_memories)
    }

    fn verify(
        verifier_setup: &Self::VerifierSetup,
        computation_graph: &ComputationGraph<ECCConfig>,
        proof: &Self::Proof,
    ) -> bool {
        let verified = proof
            .proofs
            .par_iter()
            .zip(computation_graph.proof_templates.par_iter())
            .map(|(local_proof, template)| {
                let local_commitments = template
                    .commitment_indices
                    .iter()
                    .map(|idx| &proof.commitments[*idx])
                    .collect::<Vec<_>>();

                verify_kernel::<C, ECCConfig>(
                    verifier_setup,
                    &computation_graph.kernels[template.kernel_id],
                    local_proof,
                    &local_commitments,
                    next_power_of_two(template.parallel_count),
                    &template.is_broadcast,
                )
            })
            .collect::<Vec<_>>();

        verified.iter().all(|x| *x)
    }

    fn post_process() {
        wait_async(ClientHttpHelper::request_exit())
    }
}

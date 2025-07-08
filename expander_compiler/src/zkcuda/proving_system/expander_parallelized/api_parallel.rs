use crate::circuit::config::Config;
use crate::frontend::SIMDField;
use crate::utils::misc::next_power_of_two;
use crate::zkcuda::context::ComputationGraph;
use crate::zkcuda::proving_system::expander::structs::{
    ExpanderProverSetup, ExpanderVerifierSetup,
};
use crate::zkcuda::proving_system::expander_parallelized::client_utils::{
    client_launch_server_and_setup, client_parse_args, client_send_witness_and_prove, wait_async,
    ClientHttpHelper,
};
use crate::zkcuda::proving_system::expander_parallelized::verify_impl::verify_kernel;
use crate::zkcuda::proving_system::{CombinedProof, ProvingSystem};

use super::super::Expander;

use expander_utils::timer::Timer;
use gkr_engine::{FieldEngine, GKREngine};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

pub struct ParallelizedExpander<C: GKREngine> {
    _config: std::marker::PhantomData<C>,
}

impl<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> ProvingSystem<ECCConfig>
    for ParallelizedExpander<C>
{
    type ProverSetup = ExpanderProverSetup<C::FieldConfig, C::PCSConfig>;
    type VerifierSetup = ExpanderVerifierSetup<C::FieldConfig, C::PCSConfig>;
    type Proof = CombinedProof<ECCConfig, Expander<C>>;

    fn setup(
        computation_graph: &crate::zkcuda::context::ComputationGraph<ECCConfig>,
    ) -> (Self::ProverSetup, Self::VerifierSetup) {
        let server_binary =
            client_parse_args().unwrap_or("../target/release/expander_server".to_owned());
        client_launch_server_and_setup::<C, ECCConfig>(&server_binary, computation_graph, true)
    }

    fn prove(
        _prover_setup: &Self::ProverSetup,
        _computation_graph: &crate::zkcuda::context::ComputationGraph<ECCConfig>,
        device_memories: &[Vec<SIMDField<ECCConfig>>],
    ) -> Self::Proof {
        client_send_witness_and_prove(device_memories)
    }

    fn verify(
        verifier_setup: &Self::VerifierSetup,
        computation_graph: &ComputationGraph<ECCConfig>,
        proof: &Self::Proof,
    ) -> bool {
        let verification_timer = Timer::new("Verify all kernels", true);
        let verified = proof
            .proofs
            .par_iter()
            .zip(computation_graph.proof_templates().par_iter())
            .map(|(local_proof, template)| {
                let local_commitments = template
                    .commitment_indices()
                    .iter()
                    .map(|idx| &proof.commitments[*idx])
                    .collect::<Vec<_>>();

                verify_kernel::<C, ECCConfig>(
                    verifier_setup,
                    &computation_graph.kernels()[template.kernel_id()],
                    local_proof,
                    &local_commitments,
                    next_power_of_two(template.parallel_count()),
                    template.is_broadcast(),
                )
            })
            .collect::<Vec<_>>();
        verification_timer.stop();

        verified.iter().all(|x| *x)
    }

    fn post_process() {
        wait_async(ClientHttpHelper::request_exit())
    }
}

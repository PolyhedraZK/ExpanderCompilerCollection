use crate::circuit::config::Config;
use crate::frontend::SIMDField;
use crate::zkcuda::context::ComputationGraph;
use crate::zkcuda::proving_system::expander::structs::{
    ExpanderProverSetup, ExpanderVerifierSetup,
};
use crate::zkcuda::proving_system::expander_parallelized::client_utils::{
    client_launch_server_and_setup, client_parse_args, client_send_witness_and_prove, wait_async,
    ClientHttpHelper,
};
use crate::zkcuda::proving_system::{CombinedProof, ParallelizedExpander, ProvingSystem};

use super::super::Expander;

use gkr_engine::{FieldEngine, GKREngine};

pub struct ExpanderNoOverSubscribe<C: GKREngine> {
    _config: std::marker::PhantomData<C>,
}

impl<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> ProvingSystem<ECCConfig>
    for ExpanderNoOverSubscribe<C>
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    type ProverSetup = ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>;
    type VerifierSetup = ExpanderVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>;
    type Proof = CombinedProof<ECCConfig, Expander<C>>;

    fn setup(
        computation_graph: &crate::zkcuda::context::ComputationGraph<ECCConfig>,
    ) -> (Self::ProverSetup, Self::VerifierSetup) {
        let server_binary = client_parse_args()
            .unwrap_or("../target/release/expander_server_no_oversubscribe".to_owned());
        client_launch_server_and_setup::<C, ECCConfig>(&server_binary, computation_graph, false)
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
        // The proof should be the same as the one returned by ParallelizedExpander::prove
        ParallelizedExpander::verify(verifier_setup, computation_graph, proof)
    }

    fn post_process() {
        wait_async(ClientHttpHelper::request_exit())
    }
}

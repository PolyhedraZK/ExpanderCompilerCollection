use gkr_engine::{FieldEngine, GKREngine};

use crate::{
    frontend::Config,
    zkcuda::proving_system::{
        expander::structs::{ExpanderProverSetup, ExpanderVerifierSetup},
        expander_parallelized::client_utils::{
            client_launch_server_and_setup, client_send_witness_and_prove,
        },
        CombinedProof, Expander, ProvingSystem,
    },
};

pub struct ExpanderPCSDefered<C: GKREngine> {
    _config: std::marker::PhantomData<C>,
}

impl<C, ECCConfig> ProvingSystem<ECCConfig> for ExpanderPCSDefered<C>
where
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    type ProverSetup = ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>;

    type VerifierSetup = ExpanderVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>;

    type Proof = CombinedProof<ECCConfig, Expander<C>>;

    fn setup(
        computation_graph: &crate::zkcuda::proof::ComputationGraph<ECCConfig>,
    ) -> (Self::ProverSetup, Self::VerifierSetup) {
        client_launch_server_and_setup::<C, ECCConfig>(
            "../target/release/expander_server_pcs_defered",
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
        computation_graph: &crate::zkcuda::proof::ComputationGraph<ECCConfig>,
        proof: &Self::Proof,
    ) -> bool {
        super::verify_impl::verify(verifier_setup, computation_graph, proof.clone())
    }
}

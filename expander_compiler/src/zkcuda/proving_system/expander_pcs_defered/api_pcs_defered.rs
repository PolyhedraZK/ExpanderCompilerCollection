use gkr_engine::{FieldEngine, GKREngine};

use crate::{frontend::Config, zkcuda::proving_system::{expander::structs::{ExpanderProverSetup, ExpanderVerifierSetup}, expander_pcs_defered::structs::KernelWiseProofPCSDefered, ProvingSystem}};


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

    type Proof = KernelWiseProofPCSDefered<C>;

    fn setup(computation_graph: &crate::zkcuda::proof::ComputationGraph<ECCConfig>) -> (Self::ProverSetup, Self::VerifierSetup) {
        todo!()
    }

    fn prove(
        prover_setup: &Self::ProverSetup,
        computation_graph: &crate::zkcuda::proof::ComputationGraph<ECCConfig>,
        device_memories: &[crate::zkcuda::context::DeviceMemory<ECCConfig>],
    ) -> Self::Proof {
        todo!()
    }

    fn verify(
        verifier_setup: &Self::VerifierSetup,
        computation_graph: &crate::zkcuda::proof::ComputationGraph<ECCConfig>,
        proof: &Self::Proof,
    ) -> bool {
        todo!()
    }
}
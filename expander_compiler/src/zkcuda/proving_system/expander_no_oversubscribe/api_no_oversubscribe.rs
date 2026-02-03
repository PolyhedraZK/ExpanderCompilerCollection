use crate::frontend::SIMDField;
use crate::zkcuda::context::ComputationGraph;
use crate::zkcuda::proving_system::expander::config::{GetFieldConfig, GetPCS, ZKCudaConfig};
use crate::zkcuda::proving_system::expander::structs::{
    ExpanderProverSetup, ExpanderVerifierSetup,
};
use crate::zkcuda::proving_system::expander_parallelized::client_utils::{
    client_launch_server_and_setup, client_parse_args, client_send_witness_and_prove, wait_async,
    ClientHttpHelper,
};
use crate::zkcuda::proving_system::{
    CombinedProof, ExpanderPCSDefered, ParallelizedExpander, ProvingSystem,
};

use super::super::Expander;

use gkr_engine::ExpanderPCS;

pub struct ExpanderNoOverSubscribe<ZC: ZKCudaConfig> {
    _config: std::marker::PhantomData<ZC>,
}

impl<ZC: ZKCudaConfig> ProvingSystem<ZC::ECCConfig> for ExpanderNoOverSubscribe<ZC>
where
    <GetPCS<ZC> as ExpanderPCS<GetFieldConfig<ZC>>>::Commitment:
        AsRef<<GetPCS<ZC> as ExpanderPCS<GetFieldConfig<ZC>>>::Commitment>,
{
    type ProverSetup = ExpanderProverSetup<GetFieldConfig<ZC>, GetPCS<ZC>>;
    type VerifierSetup = ExpanderVerifierSetup<GetFieldConfig<ZC>, GetPCS<ZC>>;
    type Proof = CombinedProof<ZC::ECCConfig, Expander<ZC::GKRConfig>>;

    fn setup(
        computation_graph: &ComputationGraph<ZC::ECCConfig>,
    ) -> (Self::ProverSetup, Self::VerifierSetup) {
        let server_binary = client_parse_args()
            .unwrap_or("../target/release/expander_server_no_oversubscribe".to_owned());
        client_launch_server_and_setup::<ZC::GKRConfig, ZC::ECCConfig>(
            &server_binary,
            computation_graph,
            false,
            ZC::BATCH_PCS,
        )
    }

    fn prove(
        _prover_setup: &Self::ProverSetup,
        _computation_graph: &ComputationGraph<ZC::ECCConfig>,
        device_memories: Vec<Vec<SIMDField<ZC::ECCConfig>>>,
    ) -> Self::Proof {
        client_send_witness_and_prove(device_memories)
    }

    fn verify(
        verifier_setup: &Self::VerifierSetup,
        computation_graph: &ComputationGraph<ZC::ECCConfig>,
        proof: &Self::Proof,
    ) -> bool {
        match ZC::BATCH_PCS {
            true => ExpanderPCSDefered::<ZC::GKRConfig>::verify(
                verifier_setup,
                computation_graph,
                proof,
            ),
            false => ParallelizedExpander::<ZC::GKRConfig>::verify(
                verifier_setup,
                computation_graph,
                proof,
            ),
        }
    }

    fn post_process() {
        wait_async(ClientHttpHelper::request_exit())
    }
}

impl<ZC: ZKCudaConfig> ExpanderNoOverSubscribe<ZC>
where
    <GetPCS<ZC> as ExpanderPCS<GetFieldConfig<ZC>>>::Commitment:
        AsRef<<GetPCS<ZC> as ExpanderPCS<GetFieldConfig<ZC>>>::Commitment>,
{
    /// Lightweight prove that doesn't require computation_graph or prover_setup.
    /// Use this after setup() to allow releasing those large data structures before proving.
    pub fn prove_lightweight(device_memories: Vec<Vec<SIMDField<ZC::ECCConfig>>>) {
        client_send_witness_and_prove::<ZC::GKRConfig, ZC::ECCConfig>(device_memories);
    }
}

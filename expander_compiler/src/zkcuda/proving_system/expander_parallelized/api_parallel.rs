use std::fs;

use crate::circuit::config::Config;
use crate::utils::misc::next_power_of_two;
use crate::zkcuda::proof::ComputationGraph;
use crate::zkcuda::proving_system::expander::structs::{
    ExpanderProverSetup, ExpanderVerifierSetup,
};
use crate::zkcuda::proving_system::expander_parallelized::client::{
    request_exit, request_prove, request_setup,
};
use crate::zkcuda::proving_system::expander_parallelized::cmd_utils::start_server;
use crate::zkcuda::proving_system::expander_parallelized::server_utils::parse_port_number;
use crate::zkcuda::proving_system::expander_parallelized::shared_memory_utils::SharedMemoryEngine;
use crate::zkcuda::proving_system::expander_parallelized::verify_impl::verify_kernel;
use crate::zkcuda::proving_system::{CombinedProof, ProvingSystem};

use super::super::Expander;
use super::server_utils::SERVER_IP;
use expander_utils::timer::Timer;

use gkr_engine::{FieldEngine, GKREngine};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use reqwest::Client;
use serdes::ExpSerde;

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
        let setup_timer = Timer::new("setup", true);

        let mut bytes = vec![];
        computation_graph.serialize_into(&mut bytes).unwrap();
        // append current timestamp to the file name to avoid conflicts
        let setup_filename = format!(
            "/tmp/computation_graph_{}.bin",
            chrono::Utc::now().timestamp_millis()
        );
        fs::write(&setup_filename, bytes).expect("Failed to write computation graph to file");

        let max_parallel_count = computation_graph
            .proof_templates
            .iter()
            .map(|t| t.parallel_count)
            .max()
            .unwrap_or(1);

        let port = parse_port_number();
        let server_url = format!("{SERVER_IP}:{port}");
        start_server::<C>(
            "../target/release/expander_server",
            next_power_of_two(max_parallel_count),
            port,
        );

        // Keep trying until the server is ready
        loop {
            match wait_async(Client::new().get(format!("http://{server_url}/")).send()) {
                Ok(_) => break,
                Err(_) => std::thread::sleep(std::time::Duration::from_secs(1)),
            }
        }

        wait_async(request_setup(&setup_filename));

        setup_timer.stop();

        SharedMemoryEngine::read_pcs_setup_from_shared_memory()
    }

    fn prove(
        _prover_setup: &Self::ProverSetup,
        _computation_graph: &crate::zkcuda::proof::ComputationGraph<ECCConfig>,
        device_memories: &[crate::zkcuda::context::DeviceMemory<ECCConfig>],
    ) -> Self::Proof {
        let timer = Timer::new("prove", true);

        SharedMemoryEngine::write_witness_to_shared_memory::<C::FieldConfig>(
            &device_memories
                .iter()
                .map(|m| &m.values[..])
                .collect::<Vec<_>>(),
        );
        wait_async(request_prove());

        timer.stop();
        SharedMemoryEngine::read_proof_from_shared_memory()
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
        wait_async(request_exit())
    }
}

/// Run an async function in a blocking context.
#[inline(always)]
fn wait_async<F, T>(f: F) -> T
where
    F: std::future::Future<Output = T>,
{
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(f)
}

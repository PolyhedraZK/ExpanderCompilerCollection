use gkr_engine::{FieldEngine, GKREngine, MPIConfig, MPIEngine};
use serdes::ExpSerde;

use crate::{
    frontend::{Config, SIMDField},
    zkcuda::{
        context::ComputationGraph,
        proving_system::{
            expander::{
                setup_impl::local_setup_impl,
                structs::{ExpanderProverSetup, ExpanderVerifierSetup},
            },
            expander_parallelized::prove_impl::mpi_prove_impl,
            CombinedProof, Expander, ParallelizedExpander,
        },
    },
};

pub trait ServerFns<C, ECCConfig>
where
    C: gkr_engine::GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
{
    fn setup_request_handler(
        global_mpi_config: &MPIConfig<'static>,
        setup_file: Option<String>,
        computation_graph: &mut ComputationGraph<ECCConfig>,
        prover_setup: &mut ExpanderProverSetup<C::FieldConfig, C::PCSConfig>,
        verifier_setup: &mut ExpanderVerifierSetup<C::FieldConfig, C::PCSConfig>,
    );

    fn prove_request_handler(
        global_mpi_config: &MPIConfig<'static>,
        prover_setup: &ExpanderProverSetup<C::FieldConfig, C::PCSConfig>,
        computation_graph: &ComputationGraph<ECCConfig>,
        values: &[impl AsRef<[SIMDField<C>]>],
    ) -> Option<CombinedProof<ECCConfig, Expander<C>>>;
}

impl<C, ECCConfig> ServerFns<C, ECCConfig> for ParallelizedExpander<C>
where
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
{
    fn setup_request_handler(
        global_mpi_config: &MPIConfig<'static>,
        setup_file: Option<String>,
        computation_graph: &mut ComputationGraph<ECCConfig>,
        prover_setup: &mut ExpanderProverSetup<C::FieldConfig, C::PCSConfig>,
        verifier_setup: &mut ExpanderVerifierSetup<C::FieldConfig, C::PCSConfig>,
    ) {
        let setup_file = if global_mpi_config.is_root() {
            let setup_file = setup_file.expect("Setup file path must be provided");
            broadcast_string(global_mpi_config, Some(setup_file))
        } else {
            // Workers will wait for the setup file to be broadcasted
            broadcast_string(global_mpi_config, None)
        };

        read_circuit::<C, ECCConfig>(global_mpi_config, setup_file, computation_graph);
        if global_mpi_config.is_root() {
            (*prover_setup, *verifier_setup) = local_setup_impl::<C, ECCConfig>(computation_graph);
        }
    }

    fn prove_request_handler(
        global_mpi_config: &MPIConfig<'static>,
        prover_setup: &ExpanderProverSetup<C::FieldConfig, C::PCSConfig>,
        computation_graph: &ComputationGraph<ECCConfig>,
        values: &[impl AsRef<[SIMDField<C>]>],
    ) -> Option<CombinedProof<ECCConfig, Expander<C>>>
    where
        C: GKREngine,
        ECCConfig: Config<FieldConfig = C::FieldConfig>,
    {
        mpi_prove_impl(global_mpi_config, prover_setup, computation_graph, values)
    }
}

pub fn broadcast_string(global_mpi_config: &MPIConfig<'static>, string: Option<String>) -> String {
    // Broadcast the setup file path to all workers
    if global_mpi_config.is_root() && string.is_none() {
        panic!("String must be provided on the root process in broadcast_string");
    }
    let mut string_length = string.as_ref().map_or(0, |s| s.len());
    global_mpi_config.root_broadcast_f(&mut string_length);
    let mut bytes = string.map_or(vec![0u8; string_length], |s| s.into_bytes());
    global_mpi_config.root_broadcast_bytes(&mut bytes);
    String::from_utf8(bytes).expect("Failed to convert broadcasted bytes to String")
}

pub fn read_circuit<C, ECCConfig>(
    _global_mpi_config: &MPIConfig<'static>,
    setup_file: String,
    computation_graph: &mut ComputationGraph<ECCConfig>,
) where
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
{
    let computation_graph_bytes =
        std::fs::read(setup_file).expect("Failed to read computation graph from file");
    *computation_graph = ComputationGraph::<ECCConfig>::deserialize_from(std::io::Cursor::new(
        computation_graph_bytes,
    ))
    .expect("Failed to deserialize computation graph");
}

use gkr_engine::{FieldEngine, GKREngine, MPIConfig, MPIEngine, MPISharedMemory};
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
            expander_parallelized::{
                prove_impl::mpi_prove_impl, server_ctrl::SharedMemoryWINWrapper,
                shared_memory_utils::SharedMemoryEngine,
            },
            CombinedProof, Expander, ParallelizedExpander,
        },
    },
};

pub trait ServerFns<C, ECCConfig>
where
    C: gkr_engine::GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    fn setup_request_handler(
        global_mpi_config: &MPIConfig<'static>,
        setup_file: Option<String>,
        computation_graph: &mut ComputationGraph<ECCConfig>,
        prover_setup: &mut ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
        verifier_setup: &mut ExpanderVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
        mpi_win: &mut Option<SharedMemoryWINWrapper>,
    );

    fn prove_request_handler(
        global_mpi_config: &MPIConfig<'static>,
        prover_setup: &ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
        computation_graph: &ComputationGraph<ECCConfig>,
        values: &[impl AsRef<[SIMDField<C>]>],
    ) -> Option<CombinedProof<ECCConfig, Expander<C>>>;

    fn setup_shared_witness(
        global_mpi_config: &MPIConfig<'static>,
        witness_target: &mut Vec<Vec<C::PCSField>>,
        mpi_shared_memory_win: &mut Option<SharedMemoryWINWrapper>,
    ) {
        // dispose of the previous shared memory if it exists
        while let Some(w) = witness_target.pop() {
            w.discard_control_of_shared_mem();
        }
        assert!(witness_target.is_empty());

        if let Some(win_wrapper) = mpi_shared_memory_win {
            global_mpi_config.free_shared_mem(&mut win_wrapper.win);
        }

        // Allocate new shared memory for the witness
        let (witness_v, wt_shared_memory_win) =
            SharedMemoryEngine::read_shared_witness_from_shared_memory::<C::FieldConfig>(
                global_mpi_config,
            );
        *witness_target = witness_v;
        *mpi_shared_memory_win = Some(wt_shared_memory_win);
    }

    fn shared_memory_clean_up(
        global_mpi_config: &MPIConfig<'static>,
        computation_graph: ComputationGraph<ECCConfig>,
        witness: Vec<Vec<C::PCSField>>,
        cg_mpi_win: &mut Option<SharedMemoryWINWrapper>,
        wt_mpi_win: &mut Option<SharedMemoryWINWrapper>,
    ) {
        computation_graph.discard_control_of_shared_mem();
        witness.into_iter().for_each(|w| {
            w.discard_control_of_shared_mem();
        });

        if let Some(win_wrapper) = cg_mpi_win {
            global_mpi_config.free_shared_mem(&mut win_wrapper.win);
        }

        if let Some(win_wrapper) = wt_mpi_win {
            global_mpi_config.free_shared_mem(&mut win_wrapper.win);
        }
    }
}

impl<C, ECCConfig> ServerFns<C, ECCConfig> for ParallelizedExpander<C>
where
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    fn setup_request_handler(
        global_mpi_config: &MPIConfig<'static>,
        setup_file: Option<String>,
        computation_graph: &mut ComputationGraph<ECCConfig>,
        prover_setup: &mut ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
        verifier_setup: &mut ExpanderVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
        mpi_win: &mut Option<SharedMemoryWINWrapper>,
    ) where
        C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
    {
        let setup_file = if global_mpi_config.is_root() {
            let setup_file = setup_file.expect("Setup file path must be provided");
            broadcast_string(global_mpi_config, Some(setup_file))
        } else {
            // Workers will wait for the setup file to be broadcasted
            broadcast_string(global_mpi_config, None)
        };

        read_circuit::<C, ECCConfig>(global_mpi_config, setup_file, computation_graph, mpi_win);
        if global_mpi_config.is_root() {
            (*prover_setup, *verifier_setup) = local_setup_impl::<C, ECCConfig>(computation_graph);
        }
    }

    fn prove_request_handler(
        global_mpi_config: &MPIConfig<'static>,
        prover_setup: &ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
        computation_graph: &ComputationGraph<ECCConfig>,
        values: &[impl AsRef<[SIMDField<C>]>],
    ) -> Option<CombinedProof<ECCConfig, Expander<C>>>
    where
        C: GKREngine,
        ECCConfig: Config<FieldConfig = C::FieldConfig>,
        C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
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
    global_mpi_config: &MPIConfig<'static>,
    setup_file: String,
    computation_graph: &mut ComputationGraph<ECCConfig>,
    mpi_win: &mut Option<SharedMemoryWINWrapper>,
) where
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let computation_graph_bytes =
        std::fs::read(setup_file).expect("Failed to read computation graph from file");

    let (cg, win) = if global_mpi_config.is_root() {
        let cg = ComputationGraph::<ECCConfig>::deserialize_from(std::io::Cursor::new(
            computation_graph_bytes,
        ))
        .expect("Failed to deserialize computation graph from file");
        global_mpi_config.consume_obj_and_create_shared(Some(cg))
    } else {
        global_mpi_config.consume_obj_and_create_shared(None)
    };

    *computation_graph = cg;
    mpi_win.replace(SharedMemoryWINWrapper { win });
}

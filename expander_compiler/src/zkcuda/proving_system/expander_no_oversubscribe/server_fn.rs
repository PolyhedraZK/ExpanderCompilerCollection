use arith::Fr;
use gkr_engine::{FieldEngine, GKREngine, MPIConfig, MPIEngine};

use crate::{
    frontend::{Config, SIMDField},
    zkcuda::{
        context::ComputationGraph,
        proving_system::{
            expander::structs::{ExpanderProverSetup, ExpanderVerifierSetup},
            expander_no_oversubscribe::prove_impl::mpi_prove_no_oversubscribe_impl,
            expander_parallelized::{
                server_ctrl::SharedMemoryWINWrapper,
                server_fns::{broadcast_string, read_circuit, ServerFns},
            },
            expander_pcs_defered::setup_impl::pcs_setup_max_length_only,
            CombinedProof, Expander, ExpanderNoOverSubscribe, ParallelizedExpander,
        },
    },
};

impl<C, ECCConfig> ServerFns<C, ECCConfig> for ExpanderNoOverSubscribe<C>
where
    C: GKREngine,
    C::FieldConfig: FieldEngine<CircuitField = Fr, ChallengeField = Fr>,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
{
    fn setup_request_handler(
        global_mpi_config: &MPIConfig<'static>,
        setup_file: Option<String>,
        computation_graph: &mut ComputationGraph<ECCConfig>,
        prover_setup: &mut ExpanderProverSetup<C::FieldConfig, C::PCSConfig>,
        verifier_setup: &mut ExpanderVerifierSetup<C::FieldConfig, C::PCSConfig>,
        mpi_win: &mut Option<SharedMemoryWINWrapper>,
    ) {
        let setup_file = if global_mpi_config.is_root() {
            let setup_file = setup_file.expect("Setup file path must be provided");
            broadcast_string(global_mpi_config, Some(setup_file))
        } else {
            // Workers will wait for the setup file to be broadcasted
            broadcast_string(global_mpi_config, None)
        };

        read_circuit::<C, ECCConfig>(global_mpi_config, setup_file, computation_graph, mpi_win);
        if global_mpi_config.is_root() {
            (*prover_setup, *verifier_setup) =
                pcs_setup_max_length_only::<C, ECCConfig>(computation_graph);
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
        mpi_prove_no_oversubscribe_impl(global_mpi_config, prover_setup, computation_graph, values)
    }
}

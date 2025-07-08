use gkr_engine::{FieldEngine, MPIEngine};

use crate::{
    frontend::Config,
    zkcuda::{
        context::ComputationGraph,
        proving_system::{
            expander::structs::{ExpanderProverSetup, ExpanderVerifierSetup},
            expander_parallelized::server_fns::{broadcast_string, read_circuit, ServerFns},
            expander_pcs_defered::{
                prove_impl::mpi_prove_with_pcs_defered, setup_impl::pcs_setup_max_length_only,
            },
            CombinedProof, Expander, ExpanderPCSDefered,
        },
    },
};

impl<C, ECCConfig> ServerFns<C, ECCConfig> for ExpanderPCSDefered<C>
where
    C: gkr_engine::GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
{
    fn setup_request_handler(
        global_mpi_config: &gkr_engine::MPIConfig<'static>,
        setup_file: Option<String>,
        computation_graph: &mut ComputationGraph<ECCConfig>,
        prover_setup: &mut ExpanderProverSetup<
            <C as gkr_engine::GKREngine>::FieldConfig,
            <C as gkr_engine::GKREngine>::PCSConfig,
        >,
        verifier_setup: &mut ExpanderVerifierSetup<
            <C as gkr_engine::GKREngine>::FieldConfig,
            <C as gkr_engine::GKREngine>::PCSConfig,
        >,
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
            (*prover_setup, *verifier_setup) =
                pcs_setup_max_length_only::<C, ECCConfig>(computation_graph);
        }
    }

    fn prove_request_handler(
        global_mpi_config: &gkr_engine::MPIConfig<'static>,
        prover_setup: &ExpanderProverSetup<
            <C as gkr_engine::GKREngine>::FieldConfig,
            <C as gkr_engine::GKREngine>::PCSConfig,
        >,
        computation_graph: &ComputationGraph<ECCConfig>,
        values: &[impl AsRef<[crate::frontend::SIMDField<C>]>],
    ) -> Option<CombinedProof<ECCConfig, Expander<C>>> {
        mpi_prove_with_pcs_defered(global_mpi_config, prover_setup, computation_graph, values)
    }
}

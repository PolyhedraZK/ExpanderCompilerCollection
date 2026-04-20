use arith::Fr;
use gkr_engine::{FieldEngine, GKREngine, MPIConfig};

use crate::{
    frontend::SIMDField,
    zkcuda::{
        context::ComputationGraph,
        proving_system::{
            expander::{
                config::{GetFieldConfig, GetPCS, ZKCudaConfig},
                structs::{ExpanderProverSetup, ExpanderVerifierSetup},
            },
            expander_no_oversubscribe::prove_impl::mpi_prove_no_oversubscribe_impl,
            expander_parallelized::{server_ctrl::SharedMemoryWINWrapper, server_fns::ServerFns},
            CombinedProof, Expander, ExpanderNoOverSubscribe, ExpanderPCSDefered,
            ParallelizedExpander,
        },
    },
};

impl<ZC: ZKCudaConfig> ServerFns<ZC::GKRConfig, ZC::ECCConfig> for ExpanderNoOverSubscribe<ZC>
where
    <ZC::GKRConfig as GKREngine>::FieldConfig: FieldEngine<CircuitField = Fr, ChallengeField = Fr>,
{
    fn setup_request_handler(
        global_mpi_config: &MPIConfig<'static>,
        setup_file: Option<String>,
        computation_graph: &mut ComputationGraph<ZC::ECCConfig>,
        prover_setup: &mut ExpanderProverSetup<GetFieldConfig<ZC>, GetPCS<ZC>>,
        verifier_setup: &mut ExpanderVerifierSetup<GetFieldConfig<ZC>, GetPCS<ZC>>,
        mpi_win: &mut Option<SharedMemoryWINWrapper>,
    ) {
        eprintln!("Entering setup_request_handler for ExpanderNoOverSubscribe");
        match ZC::BATCH_PCS {
            true => ExpanderPCSDefered::<ZC::GKRConfig>::setup_request_handler(
                global_mpi_config,
                setup_file,
                computation_graph,
                prover_setup,
                verifier_setup,
                mpi_win,
            ),
            false => ParallelizedExpander::<ZC::GKRConfig>::setup_request_handler(
                global_mpi_config,
                setup_file,
                computation_graph,
                prover_setup,
                verifier_setup,
                mpi_win,
            ),
        }
        eprintln!("Exiting setup_request_handler for ExpanderNoOverSubscribe");
    }

    fn prove_request_handler(
        global_mpi_config: &MPIConfig<'static>,
        prover_setup: &ExpanderProverSetup<GetFieldConfig<ZC>, GetPCS<ZC>>,
        computation_graph: &ComputationGraph<ZC::ECCConfig>,
        values: &[impl AsRef<[SIMDField<ZC::ECCConfig>]>],
    ) -> Option<CombinedProof<ZC::ECCConfig, Expander<ZC::GKRConfig>>> {
        mpi_prove_no_oversubscribe_impl::<ZC>(
            global_mpi_config,
            prover_setup,
            computation_graph,
            values,
        )
    }
}

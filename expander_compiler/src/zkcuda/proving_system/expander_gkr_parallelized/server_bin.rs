use std::{net::SocketAddr, str::FromStr, sync::Arc};

use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use expander_compiler::{
    frontend::{BN254Config, BabyBearConfig, Config, GF2Config, GoldilocksConfig, M31Config},
    zkcuda::{proof::ComputationGraph, proving_system::{
        expander_gkr_parallelized::server_utils::{GLOBAL_COMMUNICATOR, ServerState, UNIVERSE, root_main, worker_main}, ExpanderGKRProverSetup,
        ExpanderGKRVerifierSetup,
    }},
};
use gkr::{BN254ConfigSha2Hyrax, BN254ConfigSha2KZG};
use gkr_engine::{
    FieldEngine, GKREngine, MPIConfig, MPIEngine, PolynomialCommitmentType,
};
use tokio::sync::{oneshot, Mutex};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ExpanderExecArgs {
    /// M31, GF2, BN254, Goldilocks, BabyBear
    #[arg(short, long, default_value = "M31")]
    pub field_type: String,

    /// Fiat-Shamir Hash: SHA256, or Poseidon, or MiMC5
    #[arg(short, long, default_value = "SHA256")]
    pub fiat_shamir_hash: String,

    /// Polynomial Commitment Scheme: Raw, or Orion
    #[arg(short, long, default_value = "Raw")]
    pub poly_commit: String,
}

#[tokio::main]
async fn main() {
    let expander_exec_args = ExpanderExecArgs::parse();
    assert_eq!(
        expander_exec_args.fiat_shamir_hash, "SHA256",
        "Only SHA256 is supported for now"
    );

    let pcs_type = PolynomialCommitmentType::from_str(&expander_exec_args.poly_commit).unwrap();

    match (expander_exec_args.field_type.as_str(), pcs_type) {
        ("M31", PolynomialCommitmentType::Raw) => {
            serve::<M31Config, M31Config>().await;
        }
        ("GF2", PolynomialCommitmentType::Raw) => {
            serve::<GF2Config, GF2Config>().await;
        }
        ("Goldilocks", PolynomialCommitmentType::Raw) => {
            serve::<GoldilocksConfig, GoldilocksConfig>().await;
        }
        ("BabyBear", PolynomialCommitmentType::Raw) => {
            serve::<BabyBearConfig, BabyBearConfig>().await;
        }
        ("BN254", PolynomialCommitmentType::Raw) => {
            serve::<BN254Config, BN254Config>().await;
        }
        ("BN254", PolynomialCommitmentType::Hyrax) => {
            serve::<BN254ConfigSha2Hyrax, BN254Config>().await;
        }
        ("BN254", PolynomialCommitmentType::KZG) => {
            serve::<BN254ConfigSha2KZG, BN254Config>().await;
        }
        (field_type, pcs_type) => panic!(
            "Combination of {:?} and {:?} not supported",
            field_type, pcs_type
        ),
    }
}

#[allow(static_mut_refs)]
async fn serve<C: GKREngine + 'static, ECCConfig: Config<FieldConfig = C::FieldConfig> + 'static>()
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let global_mpi_config = unsafe {
        UNIVERSE = MPIConfig::init();
        GLOBAL_COMMUNICATOR = UNIVERSE.as_ref().map(|u| u.world());
        MPIConfig::prover_new(UNIVERSE.as_ref(), GLOBAL_COMMUNICATOR.as_ref())
    };

    let state = ServerState {
        lock: Arc::new(Mutex::new(())),
        global_mpi_config: global_mpi_config.clone(),
        local_mpi_config: None,
        prover_setup: Arc::new(Mutex::new(ExpanderGKRProverSetup::default())),
        verifier_setup: Arc::new(Mutex::new(ExpanderGKRVerifierSetup::default())),
        computation_graph: Arc::new(Mutex::new(ComputationGraph::default())),
        shutdown_tx: Arc::new(Mutex::new(None)),
    };

    if global_mpi_config.is_root() {
        let (tx, rx) = oneshot::channel::<()>();
        state.shutdown_tx.lock().await.replace(tx);

        let app = Router::new()
            .route("/", post(root_main::<C, ECCConfig>))
            .route("/", get(|| async { "Expander Server is running" }))
            .with_state(state);

        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        println!("Server running at http://{}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(async {
                rx.await.ok();
                println!("Shutting down server...");
            })
            .await
            .unwrap();
    } else {
        worker_main::<C, ECCConfig>(global_mpi_config).await;
    }
}

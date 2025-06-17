use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
};

use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use expander_compiler::{
    frontend::{BN254Config, BabyBearConfig, Config, GF2Config, GoldilocksConfig, M31Config},
    zkcuda::{
        proof::ComputationGraph,
        proving_system::{
            expander::structs::{ExpanderProverSetup, ExpanderVerifierSetup},
            expander_parallelized::{
                server_utils::{
                    root_main, worker_main, ServerState, GLOBAL_COMMUNICATOR, SERVER_IP, UNIVERSE,
                },
                structs::{BasicServerFns, ServerFns},
            },
        },
    },
};
use gkr::{BN254ConfigSha2Hyrax, BN254ConfigSha2KZG};
use gkr_engine::{FieldEngine, GKREngine, MPIConfig, MPIEngine, PolynomialCommitmentType};
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

    /// The port number for the server to listen on.
    #[arg(short, long, default_value = "Port")]
    pub port_number: String,
}

#[tokio::main]
pub async fn main() {
    let expander_exec_args = ExpanderExecArgs::parse();
    assert_eq!(
        expander_exec_args.fiat_shamir_hash, "SHA256",
        "Only SHA256 is supported for now"
    );

    let pcs_type = PolynomialCommitmentType::from_str(&expander_exec_args.poly_commit).unwrap();

    match (expander_exec_args.field_type.as_str(), pcs_type) {
        ("M31", PolynomialCommitmentType::Raw) => {
            serve::<M31Config, M31Config, BasicServerFns<_, _>>(expander_exec_args.port_number)
                .await;
        }
        ("GF2", PolynomialCommitmentType::Raw) => {
            serve::<GF2Config, GF2Config, BasicServerFns<_, _>>(expander_exec_args.port_number)
                .await;
        }
        ("Goldilocks", PolynomialCommitmentType::Raw) => {
            serve::<GoldilocksConfig, GoldilocksConfig, BasicServerFns<_, _>>(
                expander_exec_args.port_number,
            )
            .await;
        }
        ("BabyBear", PolynomialCommitmentType::Raw) => {
            serve::<BabyBearConfig, BabyBearConfig, BasicServerFns<_, _>>(
                expander_exec_args.port_number,
            )
            .await;
        }
        ("BN254", PolynomialCommitmentType::Raw) => {
            serve::<BN254Config, BN254Config, BasicServerFns<_, _>>(expander_exec_args.port_number)
                .await;
        }
        ("BN254", PolynomialCommitmentType::Hyrax) => {
            serve::<BN254ConfigSha2Hyrax, BN254Config, BasicServerFns<_, _>>(
                expander_exec_args.port_number,
            )
            .await;
        }
        ("BN254", PolynomialCommitmentType::KZG) => {
            serve::<BN254ConfigSha2KZG, BN254Config, BasicServerFns<_, _>>(
                expander_exec_args.port_number,
            )
            .await;
        }
        (field_type, pcs_type) => {
            panic!("Combination of {field_type:?} and {pcs_type:?} not supported")
        }
    }
}

#[allow(static_mut_refs)]
async fn serve<C, ECCConfig, S>(port_number: String)
where
    C: GKREngine + 'static,
    ECCConfig: Config<FieldConfig = C::FieldConfig> + 'static,
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
    S: ServerFns<C, ECCConfig> + 'static,
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
        prover_setup: Arc::new(Mutex::new(ExpanderProverSetup::default())),
        verifier_setup: Arc::new(Mutex::new(ExpanderVerifierSetup::default())),
        computation_graph: Arc::new(Mutex::new(ComputationGraph::default())),
        shutdown_tx: Arc::new(Mutex::new(None)),
    };

    if global_mpi_config.is_root() {
        let (tx, rx) = oneshot::channel::<()>();
        state.shutdown_tx.lock().await.replace(tx);

        let app = Router::new()
            .route("/", post(root_main::<C, ECCConfig, S>))
            .route("/", get(|| async { "Expander Server is running" }))
            .with_state(state);

        let ip: IpAddr = SERVER_IP.parse().expect("Invalid SERVER_IP");
        let port_val = port_number.parse::<u16>().unwrap_or_else(|e| {
            eprintln!("Error: Invalid port number '{port_number}'. {e}.");
            std::process::exit(1);
        });
        let addr = SocketAddr::new(ip, port_val);
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        println!("Server running at http://{addr}");
        axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(async {
                rx.await.ok();
                println!("Shutting down server...");
            })
            .await
            .unwrap();
    } else {
        worker_main::<C, ECCConfig, S>(global_mpi_config).await;
    }
}

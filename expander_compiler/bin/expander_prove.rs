mod common;
use common::ExpanderExecArgs;

use clap::Parser;
use mpi::environment::Universe;
use mpi::topology::SimpleCommunicator;
use std::str::FromStr;

use arith::Field;
use expander_compiler::frontend::{
    BN254Config, BabyBearConfig, Config, GF2Config, GoldilocksConfig, M31Config, SIMDField,
};
use expander_compiler::zkcuda::proving_system::{
    max_n_vars, server::RequestType, ExpanderGKRCommitmentExtraInfo, ExpanderGKRProof,
    ExpanderGKRProverSetup,
};
use expander_utils::timer::Timer;

use gkr::{gkr_prove, BN254ConfigSha2Hyrax, BN254ConfigSha2KZG};
use gkr_engine::{
    ExpanderPCS, ExpanderSingleVarChallenge, FieldEngine, GKREngine, MPIConfig, MPIEngine,
    PolynomialCommitmentType, SharedMemory, Transcript,
};
use polynomials::RefMultiLinearPoly;
use serdes::ExpSerde;
use sumcheck::ProverScratchPad;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use expander_circuit::Circuit as ExpCircuit;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{oneshot, Mutex};

static mut UNIVERSE: Option<Universe> = None;
static mut GLOBAL_COMMUNICATOR: Option<SimpleCommunicator> = None;

struct ServerState<'a, PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> {
    lock: Arc<Mutex<()>>, // For now we want to ensure that only one request is processed at a time
    global_mpi_config: MPIConfig<'a>,
    pcs_setup: ExpanderGKRProverSetup<PCSField, F, PCS>,
    kernels: Vec<ExpCircuit<F>>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl<'a, PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Clone
    for ServerState<'a, PCSField, F, PCS>
{
    fn clone(&self) -> Self {
        ServerState {
            lock: Arc::clone(&self.lock),
            global_mpi_config: self.global_mpi_config.clone(),
            pcs_setup: self.pcs_setup.clone(),
            kernels: self.kernels.clone(),
            shutdown_tx: Arc::clone(&self.shutdown_tx),
        }
    }
}

unsafe impl<'a, PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Send
    for ServerState<'a, PCSField, F, PCS>
{
}
unsafe impl<'a, PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Sync
    for ServerState<'a, PCSField, F, PCS>
{
}

async fn root_main<'a, PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>>(
    State(state): State<ServerState<'a, PCSField, F, PCS>>,
    Json(request_type): Json<RequestType>,
) -> Json<bool> {
    let _lock = state.lock.lock().await; // Ensure only one request is processed at a time
    match request_type {
        RequestType::PCSSetup(local_val_len, mpi_world_size) => {
            println!(
                "Setting up PCS with local_val_len: {}, mpi_world_size: {}",
                local_val_len, mpi_world_size
            );
            state
                .global_mpi_config
                .root_broadcast_bytes(&mut vec![0u8; 1]);
        }
        RequestType::RegisterKernel => {
            // Handle kernel registration logic here
            println!("Registering kernel");
            state
                .global_mpi_config
                .root_broadcast_bytes(&mut vec![1u8; 1]);
        }
        RequestType::CommitInput => {
            // Handle input commitment logic here
            println!("Committing input");
            state
                .global_mpi_config
                .root_broadcast_bytes(&mut vec![2u8; 1]);
        }
        RequestType::Prove => {
            // Handle proving logic here
            println!("Proving");
            state
                .global_mpi_config
                .root_broadcast_bytes(&mut vec![3u8; 1]);
        }
        RequestType::Exit => {
            // Handle exit logic here
            println!("Exiting");
            state
                .global_mpi_config
                .root_broadcast_bytes(&mut vec![255u8; 1]);
            state
                .shutdown_tx
                .lock()
                .await
                .take()
                .map(|tx| tx.send(()).ok());
        }
    }

    axum::Json(true)
}

fn worker_main<'a>(global_mpi_config: MPIConfig<'a>) {
    loop {
        // waiting for work
        let mut bytes = vec![0u8; 1];
        global_mpi_config.root_broadcast_bytes(&mut bytes);
        match bytes[0] {
            0 => {
                // Handle PCS setup
                println!("Worker received PCS setup request");
            }
            1 => {
                // Handle kernel registration
                println!("Worker received kernel registration request");
            }
            2 => {
                // Handle input commitment
                println!("Worker received input commitment request");
            }
            3 => {
                // Handle proving
                println!("Worker received proving request");
            }
            255 => {
                // Exit condition, if needed
                println!("Worker received exit signal");
                break;
            }
            _ => {
                println!("Unknown request type received by worker");
            }
        }
    }
}

#[allow(static_mut_refs)]
async fn serve<
    C: GKREngine + 'static,
    ECCConfig: Config<FieldConfig = C::FieldConfig> + 'static,
>() {
    let global_mpi_config = unsafe {
        UNIVERSE = MPIConfig::init();
        GLOBAL_COMMUNICATOR = UNIVERSE.as_ref().map(|u| u.world());
        MPIConfig::prover_new(UNIVERSE.as_ref(), GLOBAL_COMMUNICATOR.as_ref())
    };

    let (tx, rx) = oneshot::channel::<()>();
    let state = ServerState::<'static, C::PCSField, C::FieldConfig, C::PCSConfig> {
        lock: Arc::new(Mutex::new(())),
        global_mpi_config: global_mpi_config.clone(),
        pcs_setup: ExpanderGKRProverSetup {
            p_keys: HashMap::new(),
        },
        kernels: Vec::new(),
        shutdown_tx: Arc::new(Mutex::new(Some(tx))),
    };

    if global_mpi_config.is_root() {
        let app = Router::new().route("/", post(root_main)).with_state(state);

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
        worker_main(global_mpi_config);
    }
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

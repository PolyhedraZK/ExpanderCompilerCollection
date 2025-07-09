#![allow(clippy::type_complexity)]

use crate::zkcuda::context::ComputationGraph;
use crate::zkcuda::proving_system::expander::structs::{
    ExpanderProverSetup, ExpanderVerifierSetup,
};
use crate::zkcuda::proving_system::expander_parallelized::server_fns::ServerFns;
use crate::zkcuda::proving_system::expander_parallelized::shared_memory_utils::SharedMemoryEngine;

use axum::routing::{get, post};
use axum::Router;
use clap::Parser;
use expander_utils::timer::Timer;
use mpi::environment::Universe;
use mpi::ffi::MPI_Win;
use mpi::topology::SimpleCommunicator;
use mpi::traits::Communicator;

use crate::frontend::Config;

use axum::{extract::State, Json};
use gkr_engine::{GKREngine, MPIConfig, MPIEngine};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::sync::Mutex as SyncMutex;
use tokio::sync::{oneshot, Mutex};

pub static SERVER_IP: &str = "127.0.0.1";
pub static SERVER_PORT: Lazy<SyncMutex<u16>> = Lazy::new(|| SyncMutex::new(3000));

pub fn parse_port_number() -> u16 {
    let mut port = SERVER_PORT.lock().unwrap();
    *port = std::env::var("PORT_NUMBER")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(*port);
    *port
}

#[derive(Serialize, Deserialize)]
pub enum RequestType {
    Setup(String), // The path to the computation graph setup file
    Prove,
    Exit,
}

// TODO: Find a way to avoid this global state
pub static mut UNIVERSE: Option<Universe> = None;
pub static mut GLOBAL_COMMUNICATOR: Option<SimpleCommunicator> = None;
pub static mut LOCAL_COMMUNICATOR: Option<SimpleCommunicator> = None;
pub struct SharedMemoryWINWrapper {
    pub win: MPI_Win,
}
unsafe impl Send for SharedMemoryWINWrapper {}
unsafe impl Sync for SharedMemoryWINWrapper {}

pub struct ServerState<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> {
    pub lock: Arc<Mutex<()>>, // For now we want to ensure that only one request is processed at a time
    pub global_mpi_config: MPIConfig<'static>,
    pub local_mpi_config: Option<MPIConfig<'static>>,

    pub prover_setup: Arc<Mutex<ExpanderProverSetup<C::FieldConfig, C::PCSConfig>>>,
    pub verifier_setup:
        Arc<Mutex<ExpanderVerifierSetup<C::FieldConfig, C::PCSConfig>>>,

    pub computation_graph: Arc<Mutex<ComputationGraph<ECCConfig>>>,
    pub witness: Arc<Mutex<Vec<Vec<C::PCSField>>>>,

    pub cg_shared_memory_win: Arc<Mutex<Option<SharedMemoryWINWrapper>>>, // Shared memory for computation graph
    pub wt_shared_memory_win: Arc<Mutex<Option<SharedMemoryWINWrapper>>>, // Shared memory for witness

    pub shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

unsafe impl<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> Send
    for ServerState<C, ECCConfig>
{
}

unsafe impl<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> Sync
    for ServerState<C, ECCConfig>
{
}

impl<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> Clone
    for ServerState<C, ECCConfig>
{
    fn clone(&self) -> Self {
        ServerState {
            lock: Arc::clone(&self.lock),
            global_mpi_config: self.global_mpi_config.clone(),
            local_mpi_config: self.local_mpi_config.clone(),
            prover_setup: Arc::clone(&self.prover_setup),
            verifier_setup: Arc::clone(&self.verifier_setup),
            computation_graph: Arc::clone(&self.computation_graph),
            witness: Arc::clone(&self.witness),
            cg_shared_memory_win: Arc::clone(&self.cg_shared_memory_win),
            wt_shared_memory_win: Arc::clone(&self.wt_shared_memory_win),
            shutdown_tx: Arc::clone(&self.shutdown_tx),
        }
    }
}

pub async fn root_main<C, ECCConfig, S>(
    State(state): State<ServerState<C, ECCConfig>>,
    Json(request_type): Json<RequestType>,
) -> Json<bool>
where
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,

    S: ServerFns<C, ECCConfig>,
{
    let _lock = state.lock.lock().await; // Ensure only one request is processed at a time
    match request_type {
        RequestType::Setup(setup_file) => {
            println!("Received setup request with file: {setup_file}");
            let setup_timer = Timer::new("server setup", true);
            let _ = broadcast_request_type(&state.global_mpi_config, 1);

            let mut computation_graph = state.computation_graph.lock().await;
            let mut prover_setup_guard = state.prover_setup.lock().await;
            let mut verifier_setup_guard = state.verifier_setup.lock().await;
            let mut cg_shared_memory_win = state.cg_shared_memory_win.lock().await;
            S::setup_request_handler(
                &state.global_mpi_config,
                Some(setup_file),
                &mut computation_graph,
                &mut prover_setup_guard,
                &mut verifier_setup_guard,
                &mut cg_shared_memory_win,
            );

            SharedMemoryEngine::write_pcs_setup_to_shared_memory(&(
                prover_setup_guard.clone(),
                verifier_setup_guard.clone(),
            ));

            setup_timer.stop();
        }
        RequestType::Prove => {
            println!("Received prove request");
            // Handle proving logic here
            let prove_timer = Timer::new("server prove", true);
            let _ = broadcast_request_type(&state.global_mpi_config, 2);

            let mut witness = state.witness.lock().await;
            let mut witness_win = state.wt_shared_memory_win.lock().await;
            S::setup_shared_witness(&state.global_mpi_config, &mut witness, &mut witness_win);

            let prover_setup_guard = state.prover_setup.lock().await;
            let computation_graph = state.computation_graph.lock().await;

            let proof = S::prove_request_handler(
                &state.global_mpi_config,
                &*prover_setup_guard,
                &*computation_graph,
                &witness,
            );

            SharedMemoryEngine::write_proof_to_shared_memory(proof.as_ref().unwrap());
            prove_timer.stop();
        }
        RequestType::Exit => {
            println!("Received exit request, shutting down server");
            broadcast_request_type(&state.global_mpi_config, 255);

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

pub async fn worker_main<C, ECCConfig, S>(
    global_mpi_config: MPIConfig<'static>,
    state: ServerState<C, ECCConfig>,
) where
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,

    S: ServerFns<C, ECCConfig>,
{
    loop {
        // waiting for work
        let request_type = broadcast_request_type(&global_mpi_config, 128);
        match request_type {
            1 => {
                // TODO: Do not use this much locks, use a single lock for the whole setup
                let mut computation_graph = state.computation_graph.lock().await;
                let mut prover_setup_guard = state.prover_setup.lock().await;
                let mut verifier_setup_guard = state.verifier_setup.lock().await;
                let mut cg_shared_memory_win = state.cg_shared_memory_win.lock().await;

                S::setup_request_handler(
                    &state.global_mpi_config,
                    None,
                    &mut computation_graph,
                    &mut prover_setup_guard,
                    &mut verifier_setup_guard,
                    &mut cg_shared_memory_win,
                );
            }
            2 => {
                // Prove
                let mut witness = state.witness.lock().await;
                let mut witness_win = state.wt_shared_memory_win.lock().await;
                S::setup_shared_witness(&state.global_mpi_config, &mut witness, &mut witness_win);

                let prover_setup_guard = state.prover_setup.lock().await;
                let computation_graph = state.computation_graph.lock().await;
                let proof = S::prove_request_handler(
                    &state.global_mpi_config,
                    &*prover_setup_guard,
                    &*computation_graph,
                    &witness,
                );
                assert!(proof.is_none());
            }
            255 => {
                break;
            }
            _ => {
                println!("Unknown request type received by worker");
            }
        }
    }
}

pub fn broadcast_request_type(global_mpi_config: &MPIConfig<'static>, request_type: u8) -> u8 {
    // Broadcast the request type to all workers
    let mut bytes = vec![request_type];
    global_mpi_config.root_broadcast_bytes(&mut bytes);
    if bytes.len() != 1 {
        panic!("Failed to broadcast request type");
    }
    bytes[0]
}

#[allow(static_mut_refs)]
pub fn generate_local_mpi_config(
    global_mpi_config: &MPIConfig<'static>,
    n_parties: usize,
) -> Option<MPIConfig<'static>> {
    assert!(n_parties > 0, "Number of parties must be greater than 0");

    let rank = global_mpi_config.world_rank();
    let color_v = if rank < n_parties { 0 } else { 1 };
    let color = mpi::topology::Color::with_value(color_v);
    unsafe {
        LOCAL_COMMUNICATOR = global_mpi_config
            .world
            .unwrap()
            .split_by_color_with_key(color, rank as i32);
    }
    if color_v == 0 {
        Some(MPIConfig::prover_new(global_mpi_config.universe, unsafe {
            LOCAL_COMMUNICATOR.as_ref()
        }))
    } else {
        None
    }
}

#[allow(static_mut_refs)]
pub async fn serve<C, ECCConfig, S>(port_number: String)
where
    C: GKREngine + 'static,
    ECCConfig: Config<FieldConfig = C::FieldConfig> + 'static,

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
        witness: Arc::new(Mutex::new(Vec::new())),
        cg_shared_memory_win: Arc::new(Mutex::new(None)),
        wt_shared_memory_win: Arc::new(Mutex::new(None)),
        shutdown_tx: Arc::new(Mutex::new(None)),
    };

    if global_mpi_config.is_root() {
        let (tx, rx) = oneshot::channel::<()>();
        state.shutdown_tx.lock().await.replace(tx);

        let app = Router::new()
            .route("/", post(root_main::<C, ECCConfig, S>))
            .route("/", get(|| async { "Expander Server is running" }))
            .with_state(state.clone());

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

        // it might need some time for the server to properly shutdown
        loop {
            match Arc::strong_count(&state.computation_graph) {
                1 => {
                    break;
                }
                _ => {
                    println!("Waiting for server to shutdown...");
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }
        }
    } else {
        worker_main::<C, ECCConfig, S>(global_mpi_config, state.clone()).await;
    }

    match (
        Arc::try_unwrap(state.computation_graph),
        Arc::try_unwrap(state.witness),
    ) {
        (Ok(cg_mutex), Ok(witness_mutex)) => {
            let mut cg_mpi_win = state.cg_shared_memory_win.lock().await.take();
            let mut wt_mpi_win = state.wt_shared_memory_win.lock().await.take();
            S::shared_memory_clean_up(
                &state.global_mpi_config,
                cg_mutex.into_inner(), // moves the value out
                witness_mutex.into_inner(),
                &mut cg_mpi_win,
                &mut wt_mpi_win,
            );
        }
        _ => {
            panic!("Failed to unwrap Arc, multiple references exist");
        }
    }

    if state.global_mpi_config.is_root() {
        println!("Server has been shut down.");
    }

    unsafe { mpi::ffi::MPI_Finalize() };
}

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

#![allow(clippy::type_complexity)]

use crate::zkcuda::proof::ComputationGraph;
use crate::zkcuda::proving_system::expander::setup_impl::local_setup_impl;
use crate::zkcuda::proving_system::expander::structs::{
    ExpanderProverSetup, ExpanderVerifierSetup,
};
use crate::zkcuda::proving_system::expander_parallelized::prove_impl::mpi_prove_impl;
use crate::zkcuda::proving_system::expander_parallelized::shared_memory_utils::SharedMemoryEngine;
use crate::zkcuda::proving_system::{CombinedProof, Expander};

use expander_utils::timer::Timer;
use mpi::environment::Universe;
use mpi::topology::SimpleCommunicator;
use mpi::traits::Communicator;
use serdes::ExpSerde;

use crate::frontend::{Config, SIMDField};

use axum::{extract::State, Json};
use gkr_engine::Transcript;
use gkr_engine::{
    FieldEngine, GKREngine, MPIConfig, MPIEngine,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
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

pub struct ServerState<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    pub lock: Arc<Mutex<()>>, // For now we want to ensure that only one request is processed at a time
    pub global_mpi_config: MPIConfig<'static>,
    pub local_mpi_config: Option<MPIConfig<'static>>,
    pub prover_setup: Arc<Mutex<ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>>>,
    pub verifier_setup:
        Arc<Mutex<ExpanderVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>>>,
    pub computation_graph: Arc<Mutex<ComputationGraph<ECCConfig>>>,
    pub shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

unsafe impl<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> Send
    for ServerState<C, ECCConfig>
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
}

unsafe impl<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> Sync
    for ServerState<C, ECCConfig>
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
}

impl<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> Clone
    for ServerState<C, ECCConfig>
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    fn clone(&self) -> Self {
        ServerState {
            lock: Arc::clone(&self.lock),
            global_mpi_config: self.global_mpi_config.clone(),
            local_mpi_config: self.local_mpi_config.clone(),
            prover_setup: Arc::clone(&self.prover_setup),
            verifier_setup: Arc::clone(&self.verifier_setup),
            computation_graph: Arc::clone(&self.computation_graph),
            shutdown_tx: Arc::clone(&self.shutdown_tx),
        }
    }
}

pub async fn root_main<C, ECCConfig>(
    State(state): State<ServerState<C, ECCConfig>>,
    Json(request_type): Json<RequestType>,
) -> Json<bool>
where
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let _lock = state.lock.lock().await; // Ensure only one request is processed at a time
    match request_type {
        RequestType::Setup(setup_file) => {
            let setup_timer = Timer::new("server setup", true);
            let _ = broadcast_request_type(&state.global_mpi_config, 1);

            let mut computation_graph = state.computation_graph.lock().await;
            let mut prover_setup_guard = state.prover_setup.lock().await;
            let mut verifier_setup_guard = state.verifier_setup.lock().await;
            setup_request_handler::<C, ECCConfig>(
                &state.global_mpi_config,
                Some(setup_file),
                &mut computation_graph,
                &mut prover_setup_guard,
                &mut verifier_setup_guard,
            );

            SharedMemoryEngine::write_pcs_setup_to_shared_memory(&(
                prover_setup_guard.clone(),
                verifier_setup_guard.clone(),
            ));

            setup_timer.stop();
        }
        RequestType::Prove => {
            // Handle proving logic here
            let prove_timer = Timer::new("server prove", true);
            let _ = broadcast_request_type(&state.global_mpi_config, 2);

            let witness = SharedMemoryEngine::read_witness_from_shared_memory::<C::FieldConfig>();
            let prover_setup_guard = state.prover_setup.lock().await;
            let computation_graph = state.computation_graph.lock().await;

            let proof = prove_request_handler::<C, ECCConfig>(
                &state.global_mpi_config,
                &*prover_setup_guard,
                &*computation_graph,
                &witness,
            );

            SharedMemoryEngine::write_proof_to_shared_memory(proof.as_ref().unwrap());
            prove_timer.stop();
        }
        RequestType::Exit => {
            broadcast_request_type(&state.global_mpi_config, 255);

            unsafe { mpi::ffi::MPI_Finalize() };

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

pub async fn worker_main<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    global_mpi_config: MPIConfig<'static>,
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let state = ServerState::<C, ECCConfig> {
        lock: Arc::new(Mutex::new(())),
        global_mpi_config: global_mpi_config.clone(),
        local_mpi_config: None,
        prover_setup: Arc::new(Mutex::new(ExpanderProverSetup::default())),
        verifier_setup: Arc::new(Mutex::new(ExpanderVerifierSetup::default())),
        computation_graph: Arc::new(Mutex::new(ComputationGraph::default())),
        shutdown_tx: Arc::new(Mutex::new(None)),
    };

    loop {
        // waiting for work
        let request_type = broadcast_request_type(&global_mpi_config, 128);
        match request_type {
            1 => {
                // TODO: Do not use this much locks, use a single lock for the whole setup
                let mut computation_graph = state.computation_graph.lock().await;
                let mut prover_setup_guard = state.prover_setup.lock().await;
                let mut verifier_setup_guard = state.verifier_setup.lock().await;
                setup_request_handler::<C, ECCConfig>(
                    &state.global_mpi_config,
                    None,
                    &mut computation_graph,
                    &mut prover_setup_guard,
                    &mut verifier_setup_guard,
                );
            }
            2 => {
                // Prove
                let witness =
                    SharedMemoryEngine::read_witness_from_shared_memory::<C::FieldConfig>();
                let prover_setup_guard = state.prover_setup.lock().await;
                let computation_graph = state.computation_graph.lock().await;
                let proof = prove_request_handler::<C, ECCConfig>(
                    &state.global_mpi_config,
                    &*prover_setup_guard,
                    &*computation_graph,
                    &witness,
                );
                assert!(proof.is_none());
            }
            255 => {
                // Exit condition, if needed
                unsafe { mpi::ffi::MPI_Finalize() };
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

pub fn setup_request_handler<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    global_mpi_config: &MPIConfig<'static>,
    setup_file: Option<String>,
    computation_graph: &mut ComputationGraph<ECCConfig>,
    prover_setup: &mut ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    verifier_setup: &mut ExpanderVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
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

    read_circuit::<C, ECCConfig>(global_mpi_config, setup_file, computation_graph);
    if global_mpi_config.is_root() {
        (*prover_setup, *verifier_setup) = local_setup_impl::<C, ECCConfig>(computation_graph);
    }
}

pub fn read_circuit<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    _global_mpi_config: &MPIConfig<'static>,
    setup_file: String,
    computation_graph: &mut ComputationGraph<ECCConfig>,
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let computation_graph_bytes =
        std::fs::read(setup_file).expect("Failed to read computation graph from file");
    *computation_graph = ComputationGraph::<ECCConfig>::deserialize_from(std::io::Cursor::new(
        computation_graph_bytes,
    ))
    .expect("Failed to deserialize computation graph");
}

pub fn prove_request_handler<C, ECCConfig>(
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

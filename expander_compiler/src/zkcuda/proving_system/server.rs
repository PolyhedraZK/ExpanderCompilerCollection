use arith::Field;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use expander_circuit::Circuit as ExpCircuit;
use gkr_engine::{ExpanderPCS, FieldEngine, MPIConfig, MPIEngine};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{oneshot, Mutex};

use super::ExpanderGKRProverSetup;

#[derive(Serialize, Deserialize)]
enum RequestType {
    PCSSetup(usize, usize), // (local_val_len, mpi_world_size)
    RegisterKernel,
    CommitInput,
    Prove,
    Exit,
}

#[derive(Clone)]
struct ServerState<'a, PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> {
    lock: Arc<Mutex<()>>, // For now we want to ensure that only one request is processed at a time
    global_mpi_config: MPIConfig<'a>,
    pcs_setup: ExpanderGKRProverSetup<PCSField, F, PCS>,
    kernels: Vec<ExpCircuit<F>>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

unsafe impl<'a, PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Send
    for ServerState<'a, PCSField, F, PCS>
{
}
unsafe impl<'a, PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Sync
    for ServerState<'a, PCSField, F, PCS>
{
}

fn root_main<'a, PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>>(
    State(state): State<ServerState<'a, PCSField, F, PCS>>,
    Json(request_type): Json<RequestType>,
) -> Json<bool> {
    // let _lock = state.lock.lock().await; // Ensure only one request is processed at a time
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
            return axum::Json(false); // Indicate that the server should stop
        }
    }

    axum::Json(true)
}

fn worker_main<'a>(global_mpi_config: MPIConfig<'a>) {
    // waiting for work
    loop {
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

#[tokio::main]
async fn main() {
    let universe = MPIConfig::init();
    let world = universe.as_ref().map(|u| u.world());
    let global_mpi_config = MPIConfig::prover_new(universe.as_ref(), world.as_ref());

    let (tx, rx) = oneshot::channel::<()>();
    let state = ServerState {
        lock: Arc::new(Mutex::new(())),
        global_mpi_config,
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
            .await
            .unwrap();
    } else {
        worker_main(global_mpi_config);
    }
}

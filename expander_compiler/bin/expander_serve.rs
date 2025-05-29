mod common;
use common::ExpanderExecArgs;

mod expander_fn;

use clap::Parser;
use expander_compiler::zkcuda::kernel;
use expander_compiler::zkcuda::proof::ComputationGraph;
use expander_compiler::zkcuda::proving_system::ExpanderGKRVerifierSetup;
use mpi::environment::Universe;
use mpi::ffi::MPI_Win;
use mpi::topology::SimpleCommunicator;
use serdes::ExpSerde;
use std::str::FromStr;

use arith::Field;
use expander_compiler::frontend::{
    BN254Config, BabyBearConfig, Config, GF2Config, GoldilocksConfig, M31Config,
};
use expander_compiler::zkcuda::proving_system::{server::RequestType, ExpanderGKRProverSetup};

use axum::{extract::State, routing::post, Json, Router};
use expander_circuit::Circuit as ExpCircuit;
use gkr::{BN254ConfigSha2Hyrax, BN254ConfigSha2KZG};
use gkr_engine::{
    ExpanderPCS, FieldEngine, GKREngine, MPIConfig, MPIEngine, PolynomialCommitmentType,
};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{oneshot, Mutex};

use expander_fn::{UNIVERSE, GLOBAL_COMMUNICATOR};

struct ServerState<'a, PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> {
    lock: Arc<Mutex<()>>, // For now we want to ensure that only one request is processed at a time
    global_mpi_config: MPIConfig<'a>,
    prover_setup: ExpanderGKRProverSetup<PCSField, F, PCS>,
    verifier_setup: ExpanderGKRVerifierSetup<PCSField, F, PCS>,
    kernels: HashMap<usize, (ExpCircuit<F>, MPI_Win)>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl<'a, PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Clone
    for ServerState<'a, PCSField, F, PCS>
{
    fn clone(&self) -> Self {
        ServerState {
            lock: Arc::clone(&self.lock),
            global_mpi_config: self.global_mpi_config.clone(),
            prover_setup: self.prover_setup.clone(),
            verifier_setup: self.verifier_setup.clone(),
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

async fn root_main<'a, C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    State(mut state): State<ServerState<'a, C::PCSField, C::FieldConfig, C::PCSConfig>>,
    Json(request_type): Json<RequestType>,
) -> Json<bool>
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let _lock = state.lock.lock().await; // Ensure only one request is processed at a time
    match request_type {
        RequestType::PCSSetup(mut local_val_len, mpi_world_size) => {
            // TODO: We should support the case where mpi_world_size is different from the global mpi world size
            assert_eq!(mpi_world_size, state.global_mpi_config.world_size());
            println!(
                "Setting up PCS with local_val_len: {}, mpi_world_size: {}",
                local_val_len, mpi_world_size
            );

            state
                .global_mpi_config
                .root_broadcast_bytes(&mut vec![0u8; 1]);

            state.global_mpi_config.root_broadcast_f(&mut local_val_len);
            expander_fn::setup::<C>(
                &state.global_mpi_config,
                local_val_len,
                &mut state.prover_setup,
                &mut state.verifier_setup,
            )
        }
        RequestType::RegisterKernel(mut kernel_id) => {
            // Handle kernel registration logic here
            println!("Registering kernel");
            state
                .global_mpi_config
                .root_broadcast_bytes(&mut vec![1u8; 1]);

            state.global_mpi_config.root_broadcast_f(&mut kernel_id);
            expander_fn::register_kernel::<C, ECCConfig>(
                &state.global_mpi_config,
                kernel_id,
                &mut state.kernels,
            );
        }
        RequestType::CommitInput => {
            // Handle input commitment logic here
            println!("Committing input");
            state
                .global_mpi_config
                .root_broadcast_bytes(&mut vec![2u8; 1]);
        }
        RequestType::Prove(mut kernel_id) => {
            // Handle proving logic here
            println!("Proving");
            state
                .global_mpi_config
                .root_broadcast_bytes(&mut vec![3u8; 1]);

            state.global_mpi_config.root_broadcast_f(&mut kernel_id);
            expander_fn::prove::<C, ECCConfig>(
                &state.global_mpi_config,
                &state.prover_setup,
                &mut state.kernels.get_mut(&kernel_id).expect("Kernel not found").0,
            );
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

fn worker_main<'a, C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    global_mpi_config: MPIConfig<'a>,
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let mut state = ServerState {
        lock: Arc::new(Mutex::new(())),
        global_mpi_config: global_mpi_config.clone(),
        prover_setup: ExpanderGKRProverSetup {
            p_keys: HashMap::new(),
        },
        verifier_setup: ExpanderGKRVerifierSetup {
            v_keys: HashMap::new(),
        },
        kernels: HashMap::new(),
        shutdown_tx: Arc::new(Mutex::new(None)),
    };

    loop {
        // waiting for work
        let mut bytes = vec![0u8; 1];
        global_mpi_config.root_broadcast_bytes(&mut bytes);
        match bytes[0] {
            0 => {
                let mut local_val_len = 0;
                state.global_mpi_config.root_broadcast_f(&mut local_val_len);
                assert_ne!(local_val_len, 0);
                expander_fn::setup::<C>(
                    &global_mpi_config,
                    local_val_len,
                    &mut state.prover_setup,
                    &mut state.verifier_setup,
                );
            }
            1 => {
                let mut kernel_id = 0;
                state.global_mpi_config.root_broadcast_f(&mut kernel_id);
                expander_fn::register_kernel::<C, ECCConfig>(
                    &state.global_mpi_config,
                    kernel_id,
                    &mut state.kernels,
                );
            }
            2 => {
                expander_fn::commit::<C>(&state.global_mpi_config);
            }
            3 => {
                // Handle proving
                let mut kernel_id = 0;
                state.global_mpi_config.root_broadcast_f(&mut kernel_id);
                expander_fn::prove::<C, ECCConfig>(
                    &state.global_mpi_config,
                    &state.prover_setup,
                    &mut state.kernels
                        .get_mut(&kernel_id)
                        .expect("Kernel not found")
                        .0,
                );
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

fn init_server_state<'a, C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    global_mpi_config: MPIConfig<'a>,
) -> ServerState<'a, C::PCSField, C::FieldConfig, C::PCSConfig>
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let mut server_state = ServerState {
        lock: Arc::new(Mutex::new(())),
        global_mpi_config: global_mpi_config.clone(),
        prover_setup: ExpanderGKRProverSetup::default(),
        verifier_setup: ExpanderGKRVerifierSetup::default(),
        kernels: HashMap::new(),
        shutdown_tx: Arc::new(Mutex::new(None)),
    };

    let computation_graph = if global_mpi_config.is_root() {
        // Read the computation graph from file
        let computation_graph_bytes = std::fs::read("/tmp/computation_graph.bin")
            .expect("Failed to read computation graph from file");
        Some(ComputationGraph::<ECCConfig>::deserialize_from(std::io::Cursor::new(computation_graph_bytes))
            .expect("Failed to deserialize computation graph"))
    } else {
        None
    };

    // Read the computation graph from file and initialize kernels
    if global_mpi_config.is_root() {
        let computation_graph = computation_graph.expect("Computation graph not found on root");
        computation_graph.proof_templates.iter().for_each(|template| {
            global_mpi_config.root_broadcast_f(&mut template.kernel_id.clone());
            let ecc_circuit = &computation_graph.kernels[template.kernel_id].layered_circuit;
            let expander_circuit = ecc_circuit.export_to_expander::<ECCConfig::FieldConfig>().flatten::<C>();
            let (expander_circuit, window) = global_mpi_config.consume_obj_and_create_shared(Some(expander_circuit));
            server_state.kernels.insert(template.kernel_id, (expander_circuit, window));
        });
        global_mpi_config.root_broadcast_f(&mut usize::MAX.clone()); // Signal that setup is complete

    } else {
        loop {
            let mut kernel_id = 0;
            global_mpi_config.root_broadcast_f(&mut kernel_id);
            if kernel_id == usize::MAX {
                break;
            }
            let (expander_circuit, window) = global_mpi_config.consume_obj_and_create_shared(None);
            server_state.kernels.insert(kernel_id, (expander_circuit, window));
        }
    }

    (server_state.prover_setup, server_state.verifier_setup) =
        expander_fn::setup::<C>(&global_mpi_config, computation_graph.as_ref());

    server_state
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

    let (tx, rx) = oneshot::channel::<()>();
    let state = ServerState::<'static, C::PCSField, C::FieldConfig, C::PCSConfig> {
        lock: Arc::new(Mutex::new(())),
        global_mpi_config: global_mpi_config.clone(),
        prover_setup: ExpanderGKRProverSetup {
            p_keys: HashMap::new(),
        },
        verifier_setup: ExpanderGKRVerifierSetup {
            v_keys: HashMap::new(),
        },
        kernels: HashMap::new(),
        shutdown_tx: Arc::new(Mutex::new(Some(tx))),
    };

    if global_mpi_config.is_root() {
        let app = Router::new()
            .route("/", post(root_main::<C, ECCConfig>))
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
        worker_main::<C, ECCConfig>(global_mpi_config);
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

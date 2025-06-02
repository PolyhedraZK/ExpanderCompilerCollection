mod common;
use axum::routing::get;
use common::ExpanderExecArgs;

mod expander_fn;

use clap::Parser;
use expander_compiler::zkcuda::proof::ComputationGraph;
use expander_compiler::zkcuda::proving_system::caller_utils::write_pcs_setup_to_shared_memory;
use expander_compiler::zkcuda::proving_system::ExpanderGKRVerifierSetup;
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

use expander_fn::{generate_local_mpi_config, GLOBAL_COMMUNICATOR, UNIVERSE};

struct ServerState<'a, PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> {
    lock: Arc<Mutex<()>>, // For now we want to ensure that only one request is processed at a time
    global_mpi_config: MPIConfig<'a>,
    local_mpi_config: Option<MPIConfig<'a>>,
    prover_setup: Arc<Mutex<ExpanderGKRProverSetup<PCSField, F, PCS>>>,
    verifier_setup: Arc<Mutex<ExpanderGKRVerifierSetup<PCSField, F, PCS>>>,
    kernels: Arc<Mutex<HashMap<usize, ExpCircuit<F>>>>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl<'a, PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Clone
    for ServerState<'a, PCSField, F, PCS>
{
    fn clone(&self) -> Self {
        ServerState {
            lock: Arc::clone(&self.lock),
            global_mpi_config: self.global_mpi_config.clone(),
            local_mpi_config: self.local_mpi_config.clone(),
            prover_setup: Arc::clone(&self.prover_setup),
            verifier_setup: Arc::clone(&self.verifier_setup),
            kernels: Arc::clone(&self.kernels),
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

async fn root_main<C, ECCConfig>(
    State(mut state): State<ServerState<'static, C::PCSField, C::FieldConfig, C::PCSConfig>>,
    Json(request_type): Json<RequestType>,
) -> Json<bool>
where
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let _lock = state.lock.lock().await; // Ensure only one request is processed at a time
    match request_type {
        RequestType::Setup => {
            // Handle setup logic here
            println!("Setting up");
            state
                .global_mpi_config
                .root_broadcast_bytes(&mut vec![1u8; 1]);
            let mut kernels_guard = state.kernels.lock().await;
            let mut prover_setup_guard = state.prover_setup.lock().await;
            let mut verifier_setup_guard = state.verifier_setup.lock().await;
            read_circuit_and_setup::<C, ECCConfig>(
                state.global_mpi_config.clone(),
                &mut *kernels_guard,
                &mut *prover_setup_guard,
                &mut *verifier_setup_guard,
            );
            write_pcs_setup_to_shared_memory(&(
                prover_setup_guard.clone(),
                verifier_setup_guard.clone(),
            ));
        }
        RequestType::CommitInput(mut parallel_count) => {
            // Handle input commitment logic here
            println!("Committing input");
            state
                .global_mpi_config
                .root_broadcast_bytes(&mut vec![2u8; 1]);
            state
                .global_mpi_config
                .root_broadcast_f(&mut parallel_count);
            state.local_mpi_config =
                generate_local_mpi_config(&state.global_mpi_config, parallel_count);
            let prover_setup = state.prover_setup.lock().await;
            expander_fn::commit::<C>(state.local_mpi_config.as_ref().unwrap(), &*prover_setup);
        }
        RequestType::Prove(parallel_count, kernel_id) => {
            // Handle proving logic here
            println!("Proving");
            state
                .global_mpi_config
                .root_broadcast_bytes(&mut vec![3u8; 1]);

            state
                .global_mpi_config
                .root_broadcast_f(&mut (parallel_count, kernel_id));
            state.local_mpi_config =
                generate_local_mpi_config(&state.global_mpi_config, parallel_count);
            let mut kernels_guard = state.kernels.lock().await;
            let prover_setup_guard = state.prover_setup.lock().await;
            let kernel = kernels_guard.get_mut(&kernel_id).expect("Kernel not found");
            expander_fn::prove::<C>(
                state.local_mpi_config.as_ref().unwrap(),
                &*prover_setup_guard,
                kernel,
            );
        }
        RequestType::Exit => {
            // Handle exit logic here
            println!("Exiting");
            state
                .global_mpi_config
                .root_broadcast_bytes(&mut vec![255u8; 1]);

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

async fn worker_main<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    global_mpi_config: MPIConfig<'static>,
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let state = ServerState {
        lock: Arc::new(Mutex::new(())),
        global_mpi_config: global_mpi_config.clone(),
        local_mpi_config: None,
        prover_setup: Arc::new(Mutex::new(ExpanderGKRProverSetup::default())),
        verifier_setup: Arc::new(Mutex::new(ExpanderGKRVerifierSetup::default())),
        kernels: Arc::new(Mutex::new(HashMap::new())),
        shutdown_tx: Arc::new(Mutex::new(None)),
    };

    loop {
        // waiting for work
        let mut bytes = vec![0u8; 1];
        global_mpi_config.root_broadcast_bytes(&mut bytes);
        match bytes[0] {
            1 => {
                // Setup
                let mut kernels_guard = state.kernels.lock().await;
                let mut prover_setup_guard = state.prover_setup.lock().await;
                let mut verifier_setup_guard = state.verifier_setup.lock().await;

                read_circuit_and_setup::<C, ECCConfig>(
                    global_mpi_config.clone(),
                    &mut *kernels_guard,
                    &mut *prover_setup_guard,
                    &mut *verifier_setup_guard,
                );
            }
            2 => {
                // Commit input
                let mut parallel_count = 0;
                state
                    .global_mpi_config
                    .root_broadcast_f(&mut parallel_count);
                let local_mpi_config =
                    generate_local_mpi_config(&state.global_mpi_config, parallel_count);
                if let Some(local_mpi_config) = local_mpi_config {
                    let prover_setup_guard = state.prover_setup.lock().await;
                    expander_fn::commit::<C>(&local_mpi_config, &*prover_setup_guard);
                }
            }
            3 => {
                // Prove
                let mut pair = (0usize, 0usize);
                state.global_mpi_config.root_broadcast_f(&mut pair);
                let (parallel_count, kernel_id) = pair;
                let local_mpi_config =
                    generate_local_mpi_config(&state.global_mpi_config, parallel_count);
                if let Some(local_mpi_config) = local_mpi_config {
                    let prover_setup_guard = state.prover_setup.lock().await;
                    let mut kernels_guard = state.kernels.lock().await;
                    let exp_circuit = kernels_guard.get_mut(&kernel_id).expect("Kernel not found");

                    expander_fn::prove::<C>(&local_mpi_config, &*prover_setup_guard, exp_circuit);
                }
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

fn read_circuit_and_setup<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    global_mpi_config: MPIConfig<'static>,
    circuits: &mut HashMap<usize, ExpCircuit<C::FieldConfig>>,
    prover_setup: &mut ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    verifier_setup: &mut ExpanderGKRVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let computation_graph_bytes = std::fs::read("/tmp/computation_graph.bin")
        .expect("Failed to read computation graph from file");
    let computation_graph = ComputationGraph::<ECCConfig>::deserialize_from(std::io::Cursor::new(
        computation_graph_bytes,
    ))
    .expect("Failed to deserialize computation graph");

    computation_graph
        .proof_templates
        .iter()
        .for_each(|template| {
            global_mpi_config.root_broadcast_f(&mut template.kernel_id.clone());
            let ecc_circuit = &computation_graph.kernels[template.kernel_id].layered_circuit;
            let mut expander_circuit = ecc_circuit
                .export_to_expander::<ECCConfig::FieldConfig>()
                .flatten::<C>();
            expander_circuit.pre_process_gkr::<C>();
            circuits.insert(template.kernel_id, expander_circuit);
        });

    (*prover_setup, *verifier_setup) =
        expander_fn::setup::<C, ECCConfig>(&global_mpi_config, Some(&computation_graph));
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
        kernels: Arc::new(Mutex::new(HashMap::new())),
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

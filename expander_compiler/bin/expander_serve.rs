mod common;
use common::ExpanderExecArgs;

mod expander_fn;

use clap::Parser;
use expander_compiler::zkcuda::proof::ComputationGraph;
use expander_compiler::zkcuda::proving_system::caller_utils::write_pcs_setup_to_shared_memory;
use expander_compiler::zkcuda::proving_system::ExpanderGKRVerifierSetup;
use mpi::ffi::MPI_Win;
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
    SharedMemory,
};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{oneshot, Mutex};

use expander_fn::{generate_local_mpi_config, GLOBAL_COMMUNICATOR, UNIVERSE};

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

async fn root_main<C: GKREngine>(
    State(mut state): State<ServerState<'static, C::PCSField, C::FieldConfig, C::PCSConfig>>,
    Json(request_type): Json<RequestType>,
) -> Json<bool>
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let _lock = state.lock.lock().await; // Ensure only one request is processed at a time
    match request_type {
        RequestType::CommitInput(mut parallel_count) => {
            // Handle input commitment logic here
            println!("Committing input");
            state
                .global_mpi_config
                .root_broadcast_bytes(&mut vec![2u8; 1]);
            state
                .global_mpi_config
                .root_broadcast_f(&mut parallel_count);
            let local_mpi_config =
                generate_local_mpi_config(&state.global_mpi_config, parallel_count);
            expander_fn::commit::<C>(&local_mpi_config.unwrap());
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
            let local_mpi_config =
                generate_local_mpi_config(&state.global_mpi_config, parallel_count);
            expander_fn::prove::<C>(
                &local_mpi_config.unwrap(),
                &state.prover_setup,
                &mut state
                    .kernels
                    .get_mut(&kernel_id)
                    .expect("Kernel not found")
                    .0,
            );
        }
        // RequestType::Verify(parallel_count, kernel_id) => {
        //     // Handle verification logic here
        //     println!("Verifying");
        //     let expander_circuit = &mut state
        //         .kernels
        //         .get_mut(&kernel_id)
        //         .expect("Kernel not found")
        //         .0;
        //     let proof = read_proof_from_shared_memory();
        //     let partition_info = read_partition_info_from_shared_memory();
        //     let commitments = read_commitment_from_shared_memory();
        //     let broadcast_info = read_broadcast_info_from_shared_memory();

        //     let is_verified = expander_fn::verify::<C, ECCConfig>(
        //         &state.verifier_setup,
        //         expander_circuit,
        //         &proof,
        //         &commitments,
        //         &partition_info,
        //         parallel_count,
        //         &broadcast_info,
        //     );
        //     println!("Verification result: {}", is_verified);
        // }
        RequestType::Exit => {
            // Handle exit logic here
            println!("Exiting");
            state
                .global_mpi_config
                .root_broadcast_bytes(&mut vec![255u8; 1]);
            state
                .kernels
                .into_iter()
                .for_each(|(_, (circuit, mut window))| {
                    circuit.discard_control_of_shared_mem();
                    state.global_mpi_config.free_shared_mem(&mut window);
                });

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

fn worker_main<C: GKREngine>(
    global_mpi_config: MPIConfig<'static>,
    mut state: ServerState<'static, C::PCSField, C::FieldConfig, C::PCSConfig>,
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    loop {
        // waiting for work
        let mut bytes = vec![0u8; 1];
        global_mpi_config.root_broadcast_bytes(&mut bytes);
        match bytes[0] {
            2 => {
                // Commit input
                let mut parallel_count = 0;
                state
                    .global_mpi_config
                    .root_broadcast_f(&mut parallel_count);
                let local_mpi_config =
                    generate_local_mpi_config(&state.global_mpi_config, parallel_count);
                if let Some(local_mpi_config) = local_mpi_config {
                    expander_fn::commit::<C>(&local_mpi_config);
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
                    expander_fn::prove::<C>(
                        &local_mpi_config,
                        &state.prover_setup,
                        &mut state
                            .kernels
                            .get_mut(&kernel_id)
                            .expect("Kernel not found")
                            .0,
                    );
                }
            }
            255 => {
                // Exit condition, if needed
                state
                    .kernels
                    .into_iter()
                    .for_each(|(_, (circuit, mut window))| {
                        circuit.discard_control_of_shared_mem();
                        state.global_mpi_config.free_shared_mem(&mut window);
                    });
                unsafe { mpi::ffi::MPI_Finalize() };
                break;
            }
            _ => {
                println!("Unknown request type received by worker");
            }
        }
    }
}

fn init_server_state<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    global_mpi_config: MPIConfig<'static>,
) -> ServerState<'static, C::PCSField, C::FieldConfig, C::PCSConfig>
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

    // Read the computation graph from file
    let computation_graph = if global_mpi_config.is_root() {
        // Read the computation graph from file
        let computation_graph_bytes = std::fs::read("/tmp/computation_graph.bin")
            .expect("Failed to read computation graph from file");
        Some(
            ComputationGraph::<ECCConfig>::deserialize_from(std::io::Cursor::new(
                computation_graph_bytes,
            ))
            .expect("Failed to deserialize computation graph"),
        )
    } else {
        None
    };

    // Export the computation graph to the format required by Expander
    if global_mpi_config.is_root() {
        let computation_graph = computation_graph
            .as_ref()
            .expect("Computation graph not found on root");
        computation_graph
            .proof_templates
            .iter()
            .for_each(|template| {
                global_mpi_config.root_broadcast_f(&mut template.kernel_id.clone());
                let ecc_circuit = &computation_graph.kernels[template.kernel_id].layered_circuit;
                let expander_circuit = ecc_circuit
                    .export_to_expander::<ECCConfig::FieldConfig>()
                    .flatten::<C>();
                let (mut expander_circuit, window) =
                    global_mpi_config.consume_obj_and_create_shared(Some(expander_circuit));
                expander_circuit.pre_process_gkr::<C>();
                server_state
                    .kernels
                    .insert(template.kernel_id, (expander_circuit, window));
            });
        global_mpi_config.root_broadcast_f(&mut usize::MAX.clone()); // Signal that circuit read is complete
    } else {
        loop {
            let mut kernel_id = 0;
            global_mpi_config.root_broadcast_f(&mut kernel_id);
            if kernel_id == usize::MAX {
                break;
            }
            let (mut expander_circuit, window) =
                global_mpi_config.consume_obj_and_create_shared::<ExpCircuit<C::FieldConfig>>(None);
            expander_circuit.pre_process_gkr::<C>();
            server_state
                .kernels
                .insert(kernel_id, (expander_circuit, window));
        }
    }

    (server_state.prover_setup, server_state.verifier_setup) =
        expander_fn::setup::<C, ECCConfig>(&global_mpi_config, computation_graph.as_ref());

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

    let state = init_server_state::<C, ECCConfig>(global_mpi_config.clone());
    write_pcs_setup_to_shared_memory(&(state.prover_setup.clone(), state.verifier_setup.clone()));

    if global_mpi_config.is_root() {
        let (tx, rx) = oneshot::channel::<()>();
        state.shutdown_tx.lock().await.replace(tx);

        let app = Router::new()
            .route("/", post(root_main::<C>))
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
        worker_main::<C>(global_mpi_config, state);
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

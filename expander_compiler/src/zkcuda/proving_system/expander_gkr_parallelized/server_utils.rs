use crate::utils::misc::next_power_of_two;
use crate::zkcuda::proof::ComputationGraph;
use crate::zkcuda::proving_system::shared_memory_utils::SharedMemoryEngine;
use crate::zkcuda::proving_system::{pcs_testing_setup_fixed_seed, CombinedProof, ExpanderGKRProvingSystem, ExpanderGKRVerifierSetup};
use axum::http::request;
use expander_utils::timer::Timer;
use mpi::environment::Universe;
use mpi::topology::SimpleCommunicator;
use serdes::ExpSerde;

use crate::frontend::{
    BN254Config, BabyBearConfig, Config, GF2Config, GoldilocksConfig, M31Config, SIMDField,
};
use crate::zkcuda::proving_system::ExpanderGKRProverSetup;
use arith::Field;

use axum::{extract::State, routing::post, Json, Router};
use expander_circuit::Circuit as ExpCircuit;
use gkr::{BN254ConfigSha2Hyrax, BN254ConfigSha2KZG};
use gkr_engine::{
    ExpanderPCS, FieldEngine, GKREngine, MPIConfig, MPIEngine, PolynomialCommitmentType,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{oneshot, Mutex};

pub static SERVER_URL: &str = "http://127.0.0.1:3000/";

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

struct ServerState<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> 
where 
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    lock: Arc<Mutex<()>>, // For now we want to ensure that only one request is processed at a time
    global_mpi_config: MPIConfig<'static>,
    local_mpi_config: Option<MPIConfig<'static>>,
    prover_setup: Arc<Mutex<ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>>>,
    verifier_setup: Arc<Mutex<ExpanderGKRVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>>>,
    computation_graph: Arc<Mutex<ComputationGraph<ECCConfig>>>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

unsafe impl<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> Send for ServerState<C, ECCConfig>
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
}

unsafe impl<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> Sync for ServerState<C, ECCConfig>
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

async fn root_main<C, ECCConfig>(
    State(mut state): State<ServerState<C, ECCConfig>>,
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
                state.global_mpi_config,
                Some(setup_file),
                &mut computation_graph,
                &mut *prover_setup_guard,
                &mut *verifier_setup_guard,
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

            prove_handler::<C>();

            state.local_mpi_config =
                generate_local_mpi_config(&state.global_mpi_config, parallel_count);
            let mut kernels_guard = state.kernels.lock().await;
            let prover_setup_guard = state.prover_setup.lock().await;
            let kernel = kernels_guard.get_mut(&kernel_id).expect("Kernel not found");
            prove::<C>(
                state.local_mpi_config.as_ref().unwrap(),
                &*prover_setup_guard,
                kernel,
            );
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
        let request_type = broadcast_request_type(&global_mpi_config, 128);
        match request_type {
            1 => {
                // TODO: Do not use this much locks, use a single lock for the whole setup
                let mut kernels_guard = state.kernels.lock().await;
                let mut prover_setup_guard = state.prover_setup.lock().await;
                let mut verifier_setup_guard = state.verifier_setup.lock().await;
                setup_request_handler::<C, ECCConfig>(
                    state.global_mpi_config,
                    None,
                    &mut *kernels_guard,
                    &mut *prover_setup_guard,
                    &mut *verifier_setup_guard,
                );
            }
            // 2 => {
            //     // Commit input
            //     let mut parallel_count = 0;
            //     state
            //         .global_mpi_config
            //         .root_broadcast_f(&mut parallel_count);
            //     let local_mpi_config =
            //         generate_local_mpi_config(&state.global_mpi_config, parallel_count);
            //     if let Some(local_mpi_config) = local_mpi_config {
            //         let prover_setup_guard = state.prover_setup.lock().await;
            //         commit::<C>(&local_mpi_config, &*prover_setup_guard);
            //     }
            // }
            2 => {
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

                    prove::<C>(&local_mpi_config, &*prover_setup_guard, exp_circuit);
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

fn broadcast_request_type(global_mpi_config: &MPIConfig<'static>, request_type: u8) -> u8 {
    // Broadcast the request type to all workers
    let mut bytes = vec![request_type];
    global_mpi_config.root_broadcast_bytes(&mut bytes);
    if bytes.len() != 1 {
        panic!("Failed to broadcast request type");
    }
    bytes[0]
}

fn broadcast_string(global_mpi_config: &MPIConfig<'static>, string: Option<String>) -> String {
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

fn setup_request_handler<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    global_mpi_config: MPIConfig<'static>,
    setup_file: Option<String>,
    computation_graph: &mut ComputationGraph<ECCConfig>,
    prover_setup: &mut ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    verifier_setup: &mut ExpanderGKRVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let setup_file = if global_mpi_config.is_root() {
        let setup_file = setup_file.expect("Setup file path must be provided");
        broadcast_string(&global_mpi_config, Some(setup_file))
    } else {
        // Workers will wait for the setup file to be broadcasted
        broadcast_string(&global_mpi_config, None)
    };

    read_circuit::<C, ECCConfig>(&global_mpi_config, setup_file, computation_graph);
    setup::<C, ECCConfig>(
        &global_mpi_config,
        Some(computation_graph),
        prover_setup,
        verifier_setup,
    );
}

fn read_circuit<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
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

#[allow(clippy::type_complexity)]
fn setup<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    global_mpi_config: &MPIConfig<'static>,
    computation_graph: Option<&ComputationGraph<ECCConfig>>,
    prover_setup: &mut ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    verifier_setup: &mut ExpanderGKRVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let p_keys = &mut prover_setup.p_keys;
    let v_keys = &mut verifier_setup.v_keys;

    let computation_graph = computation_graph.unwrap();
    for template in computation_graph.proof_templates.iter() {
        for (x, is_broadcast) in template
            .commitment_indices
            .iter()
            .zip(template.is_broadcast.iter())
        {
            let val_total_len = computation_graph.commitments_lens[*x];
            let val_actual_len = if *is_broadcast {
                val_total_len
            } else {
                val_total_len / template.parallel_count
            };

            let parallel_count = next_power_of_two(template.parallel_count);
            if p_keys.contains_key(&(val_actual_len, parallel_count)) {
                continue;
            }

            let local_mpi_config =
                generate_local_mpi_config(global_mpi_config, parallel_count);

            if let Some(local_mpi_config) = local_mpi_config {
                let (_params, p_key, v_key, _scratch) = pcs_testing_setup_fixed_seed::<
                    C::FieldConfig,
                    C::TranscriptConfig,
                    C::PCSConfig,
                >(
                    val_actual_len, &local_mpi_config
                );
                p_keys.insert((val_actual_len, parallel_count), p_key);
                v_keys.insert((val_actual_len, parallel_count), v_key);
            }
        }
    }
}

fn prove_handler<C, ECCConfig>(
    global_mpi_config: &MPIConfig<'static>,
    prover_setup: &ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>, 
    computation_graph: &ComputationGraph<ECCConfig>,
    values: &[impl AsRef<SIMDField<C>>]
) -> CombinedProof<ECCConfig, ExpanderGKRProvingSystem<C>> 
where
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let commitments = computation_graph
        .proof_templates
        .iter()
        .map(|template| {
            template
                .commitment_indices
                .iter()
                .zip(template.is_broadcast.iter())
                .map(|(x, is_broadcast)| {
                    commit(
                        prover_setup,
                        values[*x].as_ref(),
                        next_power_of_two(template.parallel_count),
                        *is_broadcast,
                    )
                })
                .unzip::<_, _, Vec<_>, Vec<_>>()
        })
        .collect::<Vec<_>>();

    let proofs = computation_graph
        .proof_templates
        .iter()
        .zip(commitments.iter())
        .map(|(template, commitments_kernel)| {
            prove_kernel(
                prover_setup,
                template.kernel_id,
                &computation_graph.kernels[template.kernel_id],
                &commitments_kernel.0,
                &commitments_kernel.1,
                &template
                    .commitment_indices
                    .iter()
                    .map(|x| &values[..])
                    .collect::<Vec<_>>(),
                template.parallel_count,
                &template.is_broadcast,
            )
        })
        .collect::<Vec<_>>();

    CombinedProof {
        commitments: commitments.into_iter().map(|x| x.0).collect(),
        proofs,
    }
}


pub fn commit<C: GKREngine>(
    mpi_config: &MPIConfig,
    prover_setup: &ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let world_rank = mpi_config.world_rank();
    let world_size = mpi_config.world_size();
    if world_rank == 0 {
        println!("Expander Commit Exec Called with world size {}", world_size);
    }

    let local_vals_to_commit =
        read_local_vals_to_commit_from_shared_memory::<C::FieldConfig>(world_rank, world_size);
    let local_val_len = local_vals_to_commit.len();
    let p_key = prover_setup
        .p_keys
        .get(&(local_val_len, world_size))
        .unwrap();

    let params = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(
        local_val_len.ilog2() as usize,
        mpi_config.world_size(),
    );

    let mut scratch = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::init_scratch_pad(
        &params, mpi_config,
    );

    let commitment = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::commit(
        &params,
        mpi_config,
        p_key,
        &RefMultiLinearPoly::from_ref(&local_vals_to_commit),
        &mut scratch,
    );

    if world_rank == 0 {
        let commitment = ExpanderGKRCommitment {
            vals_len: local_val_len,
            commitment: vec![commitment.unwrap()],
        };
        let extra_info = ExpanderGKRCommitmentExtraInfo {
            scratch: vec![scratch],
        };

        write_commitment_to_shared_memory::<C::PCSField, C::FieldConfig, C::PCSConfig>(&commitment);
        write_commitment_extra_info_to_shared_memory::<C::PCSField, C::FieldConfig, C::PCSConfig>(
            &extra_info,
        );
    }
}

// Ideally, there will only one ECCConfig generics
// But we need to implement `Config` for each GKREngine, which remains to be done
// For now, the GKREngine actually controls the functionality of the prover
// The ECCConfig is only used where the `Config` trait is required
pub fn prove<C: GKREngine>(
    mpi_config: &MPIConfig,
    pcs_setup: &ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    expander_circuit: &mut ExpCircuit<C::FieldConfig>, // mut to allow filling rnd coefs and circuit inputs
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let world_rank = mpi_config.world_rank();
    let world_size = mpi_config.world_size();
    if world_rank == 0 {
        println!("Expander Prove Exec Called with world size {}", world_size);
    }

    let timer = Timer::new(
        "recurring cost: read witness&commitment",
        mpi_config.is_root(),
    );
    let _commitments = if mpi_config.is_root() {
        Some(read_commitment_from_shared_memory::<
            C::PCSField,
            C::FieldConfig,
            C::PCSConfig,
        >())
    } else {
        None
    };
    let partition_info = read_partition_info_from_shared_memory();
    let broadcast_info = read_broadcast_info_from_shared_memory();
    let commitments_extra_info =
        read_commitment_extra_info_from_shared_memory::<C::PCSField, C::FieldConfig, C::PCSConfig>(
        );
    let local_commitment_values = read_commitment_values_from_shared_memory::<C::FieldConfig>(
        &broadcast_info,
        world_rank,
        world_size,
    );
    timer.stop();

    let timer = Timer::new("gkr prove", mpi_config.is_root());
    let (max_num_input_var, max_num_output_var) = max_n_vars(expander_circuit);
    let max_num_var = max(max_num_input_var, max_num_output_var); // temp fix to a bug in Expander, remove this after Expander update.
    let mut prover_scratch =
        ProverScratchPad::<C::FieldConfig>::new(max_num_var, max_num_var, world_size);

    let mut transcript = C::TranscriptConfig::new();
    transcript.append_u8_slice(&[0u8; 32]); // TODO: Replace with the commitment, and hash an additional a few times
    expander_circuit.layers[0].input_vals = prepare_inputs(
        1usize << expander_circuit.log_input_size(),
        &partition_info,
        &local_commitment_values,
    );
    expander_circuit.fill_rnd_coefs(&mut transcript);
    expander_circuit.evaluate();
    let (claimed_v, challenge) = gkr_prove(
        expander_circuit,
        &mut prover_scratch,
        &mut transcript,
        mpi_config,
    );
    assert_eq!(
        claimed_v,
        <C::FieldConfig as FieldEngine>::ChallengeField::from(0)
    );
    timer.stop();

    let timer = Timer::new("pcs opening", mpi_config.is_root());
    prove_input_claim::<C>(
        mpi_config,
        &local_commitment_values,
        pcs_setup,
        &commitments_extra_info,
        &challenge.challenge_x(),
        &broadcast_info,
        &mut transcript,
    );
    if let Some(challenge_y) = challenge.challenge_y() {
        prove_input_claim::<C>(
            mpi_config,
            &local_commitment_values,
            pcs_setup,
            &commitments_extra_info,
            &challenge_y,
            &broadcast_info,
            &mut transcript,
        );
    }
    timer.stop();

    let proof = transcript.finalize_and_get_proof();
    if world_rank == 0 {
        write_proof_to_shared_memory(&ExpanderGKRProof { data: vec![proof] });
    }
}

#[allow(clippy::too_many_arguments)]
fn prove_input_claim<C: GKREngine>(
    mpi_config: &MPIConfig,
    local_commitments_values: &[Vec<SIMDField<C>>],
    p_keys: &ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    commitments_extra_info: &[ExpanderGKRCommitmentExtraInfo<
        C::PCSField,
        C::FieldConfig,
        C::PCSConfig,
    >],
    challenge: &ExpanderSingleVarChallenge<C::FieldConfig>,
    is_broadcast: &[bool],
    transcript: &mut C::TranscriptConfig,
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    for ((local_commitment_val, extra_info), _ib) in local_commitments_values
        .iter()
        .zip(commitments_extra_info)
        .zip(is_broadcast)
    {
        let val_len = local_commitment_val.len();
        let vals_to_open = local_commitment_val;

        let nb_challenge_vars = val_len.ilog2() as usize;
        let challenge_vars = challenge.rz[..nb_challenge_vars].to_vec();

        let params = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(
            val_len,
            mpi_config.world_size(),
        );
        let p_key = p_keys
            .p_keys
            .get(&(val_len, mpi_config.world_size()))
            .unwrap();

        let poly = RefMultiLinearPoly::from_ref(vals_to_open);
        let v = C::FieldConfig::collectively_eval_circuit_vals_at_expander_challenge(
            vals_to_open,
            &ExpanderSingleVarChallenge::<C::FieldConfig> {
                rz: challenge_vars.to_vec(),
                r_simd: challenge.r_simd.to_vec(),
                r_mpi: challenge.r_mpi.to_vec(),
            },
            &mut vec![<C::FieldConfig as FieldEngine>::Field::ZERO; val_len],
            &mut vec![
                <C::FieldConfig as FieldEngine>::ChallengeField::ZERO;
                1 << max(challenge.r_simd.len(), challenge.r_mpi.len())
            ],
            mpi_config,
        );
        transcript.append_field_element(&v);

        transcript.lock_proof();
        let opening = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::open(
            &params,
            mpi_config,
            p_key,
            &poly,
            &ExpanderSingleVarChallenge::<C::FieldConfig> {
                rz: challenge_vars.to_vec(),
                r_simd: challenge.r_simd.to_vec(),
                r_mpi: challenge.r_mpi.to_vec(),
            },
            transcript,
            &extra_info.scratch[0],
        );
        transcript.unlock_proof();

        if mpi_config.is_root() {
            let mut buffer = vec![];
            opening
                .unwrap()
                .serialize_into(&mut buffer)
                .expect("Failed to serialize opening");
            transcript.append_u8_slice(&buffer);
        }
    }
}

fn prepare_inputs<F: Field>(
    input_len: usize,
    partition_info: &[LayeredCircuitInputVec],
    local_commitment_values: &[Vec<F>],
) -> Vec<F> {
    let mut input_vals = vec![F::ZERO; input_len];
    for (partition, val) in partition_info.iter().zip(local_commitment_values.iter()) {
        assert!(partition.len == val.len());
        input_vals[partition.offset..partition.offset + partition.len].copy_from_slice(val);
    }
    input_vals
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

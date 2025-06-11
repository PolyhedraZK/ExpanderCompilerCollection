#![allow(clippy::type_complexity)]

use crate::utils::misc::next_power_of_two;
use crate::zkcuda::kernel::{Kernel, LayeredCircuitInputVec};
use crate::zkcuda::proof::ComputationGraph;
use crate::zkcuda::proving_system::shared_memory_utils::SharedMemoryEngine;
use crate::zkcuda::proving_system::{
    max_n_vars, pcs_testing_setup_fixed_seed, CombinedProof, ExpanderGKRCommitment,
    ExpanderGKRCommitmentExtraInfo, ExpanderGKRProof, ExpanderGKRProvingSystem,
    ExpanderGKRVerifierSetup,
};
use expander_utils::timer::Timer;
use mpi::environment::Universe;
use mpi::topology::SimpleCommunicator;
use mpi::traits::Communicator;
use polynomials::RefMultiLinearPoly;
use serdes::ExpSerde;
use sumcheck::ProverScratchPad;

use crate::frontend::{Config, SIMDField};
use crate::zkcuda::proving_system::ExpanderGKRProverSetup;
use arith::Field;

use axum::{extract::State, Json};
use gkr::gkr_prove;
use gkr_engine::Transcript;
use gkr_engine::{
    ExpanderDualVarChallenge, ExpanderPCS, ExpanderSingleVarChallenge, FieldEngine, GKREngine,
    MPIConfig, MPIEngine,
};
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};

pub static SERVER_URL: &str = "127.0.0.1:3000";

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
    pub prover_setup: Arc<Mutex<ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>>>,
    pub verifier_setup:
        Arc<Mutex<ExpanderGKRVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>>>,
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
        prover_setup: Arc::new(Mutex::new(ExpanderGKRProverSetup::default())),
        verifier_setup: Arc::new(Mutex::new(ExpanderGKRVerifierSetup::default())),
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
    global_mpi_config: &MPIConfig<'static>,
    setup_file: Option<String>,
    computation_graph: &mut ComputationGraph<ECCConfig>,
    prover_setup: &mut ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    verifier_setup: &mut ExpanderGKRVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
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
        root_setup::<C, ECCConfig>(Some(computation_graph), prover_setup, verifier_setup);
    }
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
fn root_setup<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    computation_graph: Option<&ComputationGraph<ECCConfig>>,
    prover_setup: &mut ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    verifier_setup: &mut ExpanderGKRVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let p_keys = &mut prover_setup.p_keys;
    let v_keys = &mut verifier_setup.v_keys;
    for commitment_len in computation_graph.unwrap().commitments_lens.iter() {
        if p_keys.contains_key(commitment_len) {
            continue;
        }
        let (_params, p_key, v_key, _scratch) =
            pcs_testing_setup_fixed_seed::<C::FieldConfig, C::TranscriptConfig, C::PCSConfig>(
                *commitment_len,
                &MPIConfig::prover_new(None, None),
            );
        p_keys.insert(*commitment_len, p_key);
        v_keys.insert(*commitment_len, v_key);
    }
}

fn prove_request_handler<C, ECCConfig>(
    global_mpi_config: &MPIConfig<'static>,
    prover_setup: &ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    computation_graph: &ComputationGraph<ECCConfig>,
    values: &[impl AsRef<[SIMDField<C>]>],
) -> Option<CombinedProof<ECCConfig, ExpanderGKRProvingSystem<C>>>
where
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let (commitments, extra_infos) = if global_mpi_config.is_root() {
        let (commitments, extra_infos) = values
            .iter()
            .map(|value| root_commit::<C>(prover_setup, value.as_ref()))
            .unzip::<_, _, Vec<_>, Vec<_>>();
        (Some(commitments), Some(extra_infos))
    } else {
        (None, None)
    };

    let proofs = computation_graph
        .proof_templates
        .iter()
        .map(|template| {
            let gkr_end_state = prove_kernel_gkr::<C, ECCConfig>(
                global_mpi_config,
                &computation_graph.kernels[template.kernel_id],
                &template
                    .commitment_indices
                    .iter()
                    .map(|&idx| values[idx].as_ref())
                    .collect::<Vec<_>>(),
                next_power_of_two(template.parallel_count),
                &template.is_broadcast,
            );

            if global_mpi_config.is_root() {
                let (mut transcript, challenge) = gkr_end_state.unwrap();
                let challenges = if let Some(challenge_y) = challenge.challenge_y() {
                    vec![challenge.challenge_x(), challenge_y]
                } else {
                    vec![challenge.challenge_x()]
                };

                challenges.iter().for_each(|c| {
                    root_prove_input_claim::<C>(
                        prover_setup,
                        &template
                            .commitment_indices
                            .iter()
                            .map(|&idx| values[idx].as_ref())
                            .collect::<Vec<_>>(),
                        extra_infos.as_ref().unwrap(),
                        c,
                        &template.is_broadcast,
                        &mut transcript,
                    );
                });

                Some(ExpanderGKRProof {
                    data: vec![transcript.finalize_and_get_proof()],
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if global_mpi_config.is_root() {
        let proofs = proofs.into_iter().map(|p| p.unwrap()).collect::<Vec<_>>();
        Some(CombinedProof {
            commitments: commitments.unwrap(),
            proofs,
        })
    } else {
        None
    }
}

fn root_commit<C: GKREngine>(
    prover_setup: &ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    vals: &[SIMDField<C>],
) -> (
    ExpanderGKRCommitment<C::PCSField, C::FieldConfig, C::PCSConfig>,
    ExpanderGKRCommitmentExtraInfo<C::PCSField, C::FieldConfig, C::PCSConfig>,
)
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let n_vars = vals.len().ilog2() as usize;
    let params = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(n_vars, 1);
    let p_key = prover_setup.p_keys.get(&vals.len()).unwrap();

    let mut scratch = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::init_scratch_pad(
        &params,
        &MPIConfig::prover_new(None, None),
    );

    let commitment = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::commit(
        &params,
        &MPIConfig::prover_new(None, None),
        p_key,
        &RefMultiLinearPoly::from_ref(vals),
        &mut scratch,
    )
    .unwrap();

    (
        ExpanderGKRCommitment {
            vals_len: 1 << vals.len(),
            commitment,
        },
        ExpanderGKRCommitmentExtraInfo { scratch },
    )
}

// Ideally, there will only one ECCConfig generics
// But we need to implement `Config` for each GKREngine, which remains to be done
// For now, the GKREngine actually controls the functionality of the prover
// The ECCConfig is only used where the `Config` trait is required
#[allow(clippy::too_many_arguments)]
fn prove_kernel_gkr<C, ECCConfig>(
    mpi_config: &MPIConfig<'static>,
    kernel: &Kernel<ECCConfig>,
    commitments_values: &[&[SIMDField<C>]],
    parallel_count: usize,
    is_broadcast: &[bool],
) -> Option<(
    C::TranscriptConfig,
    ExpanderDualVarChallenge<C::FieldConfig>,
)>
where
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let local_mpi_config = generate_local_mpi_config(mpi_config, parallel_count);

    local_mpi_config.as_ref()?;

    let local_mpi_config = local_mpi_config.unwrap();
    let local_world_size = local_mpi_config.world_size();
    let local_world_rank = local_mpi_config.world_rank();

    let local_commitment_values = commitments_values
        .iter()
        .zip(is_broadcast.iter())
        .map(|(vals, is_broadcast)| {
            if *is_broadcast {
                &vals[..]
            } else {
                let local_val_len = vals.len() / local_world_size;
                &vals[local_val_len * local_world_rank..local_val_len * (local_world_rank + 1)]
            }
        })
        .collect::<Vec<_>>();

    let mut expander_circuit = kernel.layered_circuit.export_to_expander().flatten::<C>();
    expander_circuit.pre_process_gkr::<C>();
    let (max_num_input_var, max_num_output_var) = max_n_vars(&expander_circuit);
    let max_num_var = max(max_num_input_var, max_num_output_var);
    let mut prover_scratch =
        ProverScratchPad::<C::FieldConfig>::new(max_num_var, max_num_var, local_world_size);

    let mut transcript = C::TranscriptConfig::new();
    transcript.append_u8_slice(&[0u8; 32]); // TODO: Replace with the commitment, and hash an additional a few times
    expander_circuit.layers[0].input_vals = prepare_inputs(
        1usize << expander_circuit.log_input_size(),
        &kernel.layered_circuit_input,
        &local_commitment_values,
    );
    expander_circuit.fill_rnd_coefs(&mut transcript);
    expander_circuit.evaluate();
    let (claimed_v, challenge) = gkr_prove(
        &expander_circuit,
        &mut prover_scratch,
        &mut transcript,
        &local_mpi_config,
    );
    assert_eq!(
        claimed_v,
        <C::FieldConfig as FieldEngine>::ChallengeField::from(0)
    );

    Some((transcript, challenge))
}

pub fn get_challenge_for_pcs_with_mpi<F: FieldEngine>(
    gkr_challenge: &ExpanderSingleVarChallenge<F>,
    total_vals_len: usize,
    parallel_count: usize,
    is_broadcast: bool,
) -> (ExpanderSingleVarChallenge<F>, Vec<F::ChallengeField>) {
    let mut challenge = gkr_challenge.clone();
    let zero = F::ChallengeField::ZERO;
    if is_broadcast {
        let n_vals_vars = total_vals_len.ilog2() as usize;
        let component_idx_vars = challenge.rz[n_vals_vars..].to_vec();
        challenge.rz.resize(n_vals_vars, zero);
        challenge.r_mpi.clear();
        (challenge, component_idx_vars)
    } else {
        let n_vals_vars = (total_vals_len / parallel_count).ilog2() as usize;
        let component_idx_vars = challenge.rz[n_vals_vars..].to_vec();
        challenge.rz.resize(n_vals_vars, zero);

        challenge.rz.extend_from_slice(&challenge.r_mpi);
        challenge.r_mpi.clear();
        (challenge, component_idx_vars)
    }
}

#[allow(clippy::too_many_arguments)]
fn root_prove_input_claim<C: GKREngine>(
    p_keys: &ExpanderGKRProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
    commitments_values: &[impl AsRef<[SIMDField<C>]>],
    commitments_extra_info: &[ExpanderGKRCommitmentExtraInfo<
        C::PCSField,
        C::FieldConfig,
        C::PCSConfig,
    >],
    gkr_challenge: &ExpanderSingleVarChallenge<C::FieldConfig>,
    is_broadcast: &[bool],
    transcript: &mut C::TranscriptConfig,
) where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let parallel_count = 1 << gkr_challenge.r_mpi.len();
    for ((commitment_val, extra_info), ib) in commitments_values
        .iter()
        .zip(commitments_extra_info)
        .zip(is_broadcast)
    {
        let val_len = commitment_val.as_ref().len();
        let (challenge_for_pcs, _) =
            get_challenge_for_pcs_with_mpi(gkr_challenge, val_len, parallel_count, *ib);

        let params =
            <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(val_len, 1);
        let p_key = p_keys.p_keys.get(&val_len).unwrap();

        let poly = RefMultiLinearPoly::from_ref(commitment_val.as_ref());
        let v =
            <C::FieldConfig as FieldEngine>::single_core_eval_circuit_vals_at_expander_challenge(
                commitment_val.as_ref(),
                &challenge_for_pcs,
            );
        transcript.append_field_element(&v);

        transcript.lock_proof();
        let opening = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::open(
            &params,
            &MPIConfig::prover_new(None, None),
            p_key,
            &poly,
            &challenge_for_pcs,
            transcript,
            &extra_info.scratch,
        )
        .unwrap();
        transcript.unlock_proof();

        let mut buffer = vec![];
        opening
            .serialize_into(&mut buffer)
            .expect("Failed to serialize opening");
        transcript.append_u8_slice(&buffer);
    }
}

fn prepare_inputs<F: Field>(
    input_len: usize,
    partition_info: &[LayeredCircuitInputVec],
    local_commitment_values: &[impl AsRef<[F]>],
) -> Vec<F> {
    let mut input_vals = vec![F::ZERO; input_len];
    for (partition, val) in partition_info.iter().zip(local_commitment_values.iter()) {
        assert!(partition.len == val.as_ref().len());
        input_vals[partition.offset..partition.offset + partition.len]
            .copy_from_slice(val.as_ref());
    }
    input_vals
}

#[allow(static_mut_refs)]
fn generate_local_mpi_config(
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

use arith::Field;
use expander_utils::timer::Timer;
use gkr_engine::{
    ExpanderDualVarChallenge, ExpanderSingleVarChallenge, FieldEngine, GKREngine, MPIConfig,
    MPIEngine, Transcript,
};
use std::collections::HashMap;
use std::fs;

use crate::{
    frontend::{Config, SIMDField},
    utils::misc::next_power_of_two,
    zkcuda::{
        context::ComputationGraph,
        kernel::Kernel,
        proving_system::{
            expander::{
                commit_impl::local_commit_impl,
                prove_impl::{
                    get_local_vals, pcs_local_open_impl, prepare_expander_circuit,
                    prove_gkr_with_local_vals,
                },
                structs::{ExpanderCommitmentState, ExpanderProof, ExpanderProverSetup},
            },
            expander_parallelized::server_ctrl::generate_local_mpi_config,
            CombinedProof, Expander,
        },
    },
};

pub fn mpi_prove_impl<C, ECCConfig>(
    global_mpi_config: &MPIConfig<'static>,
    prover_setup: &ExpanderProverSetup<C::FieldConfig, C::PCSConfig>,
    computation_graph: &ComputationGraph<ECCConfig>,
    values: &[impl AsRef<[SIMDField<C>]>],
) -> Option<CombinedProof<ECCConfig, Expander<C>>>
where
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
{
    let commit_timer = Timer::new("Commit to all input", global_mpi_config.is_root());
    let (commitments, states) = if global_mpi_config.is_root() {
        let (commitments, states) = values
            .iter()
            .map(|value| {
                local_commit_impl::<C, ECCConfig>(
                    prover_setup.p_keys.get(&value.as_ref().len()).unwrap(),
                    value.as_ref(),
                )
            })
            .unzip::<_, _, Vec<_>, Vec<_>>();
        (Some(commitments), Some(states))
    } else {
        (None, None)
    };
    commit_timer.stop();

    let prove_timer = Timer::new("Prove all kernels", global_mpi_config.is_root());
    let proofs = computation_graph
        .proof_templates()
        .iter()
        .map(|template| {
            let commitment_values = template
                .commitment_indices()
                .iter()
                .map(|&idx| values[idx].as_ref())
                .collect::<Vec<_>>();

            let single_kernel_gkr_timer =
                Timer::new("small gkr kernel", global_mpi_config.is_root());
            let gkr_end_state = prove_kernel_gkr::<C::FieldConfig, C::TranscriptConfig, ECCConfig>(
                global_mpi_config,
                &computation_graph.kernels()[template.kernel_id()],
                &commitment_values,
                next_power_of_two(template.parallel_count()),
                template.is_broadcast(),
            );
            single_kernel_gkr_timer.stop();

            if global_mpi_config.is_root() {
                let pcs_open_timer = Timer::new("pcs open", true);
                let (mut transcript, challenge) = gkr_end_state.unwrap();
                let challenges = if let Some(challenge_y) = challenge.challenge_y() {
                    vec![challenge.challenge_x(), challenge_y]
                } else {
                    vec![challenge.challenge_x()]
                };

                challenges.iter().for_each(|c| {
                    partition_single_gkr_claim_and_open_pcs_mpi::<C>(
                        prover_setup,
                        &commitment_values,
                        &template
                            .commitment_indices()
                            .iter()
                            .map(|&idx| &states.as_ref().unwrap()[idx])
                            .collect::<Vec<_>>(),
                        c,
                        template.is_broadcast(),
                        &mut transcript,
                    );
                });

                pcs_open_timer.stop();
                Some(ExpanderProof {
                    data: vec![transcript.finalize_and_get_proof()],
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    prove_timer.stop();

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

#[allow(clippy::too_many_arguments)]
pub fn prove_kernel_gkr<F, T, ECCConfig>(
    mpi_config: &MPIConfig<'static>,
    kernel: &Kernel<ECCConfig>,
    commitments_values: &[&[F::SimdCircuitField]],
    parallel_count: usize,
    is_broadcast: &[bool],
) -> Option<(T, ExpanderDualVarChallenge<F>)>
where
    F: FieldEngine,
    T: Transcript,
    ECCConfig: Config<FieldConfig = F>,
{
    let local_mpi_config = generate_local_mpi_config(mpi_config, parallel_count);

    local_mpi_config.as_ref()?;

    let local_mpi_config = local_mpi_config.unwrap();
    let local_world_size = local_mpi_config.world_size();
    let local_world_rank = local_mpi_config.world_rank();

    let local_commitment_values = get_local_vals(
        commitments_values,
        is_broadcast,
        local_world_rank,
        local_world_size,
    );

    let (mut expander_circuit, mut prover_scratch) =
        prepare_expander_circuit::<F, ECCConfig>(kernel, local_world_size);

    let mut transcript = T::new();
    let challenge = prove_gkr_with_local_vals::<F, T>(
        &mut expander_circuit,
        &mut prover_scratch,
        &local_commitment_values,
        kernel.layered_circuit_input(),
        &mut transcript,
        &local_mpi_config,
    );

    Some((transcript, challenge))
}

pub fn partition_challenge_and_location_for_pcs_mpi<F: FieldEngine>(
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
pub fn partition_single_gkr_claim_and_open_pcs_mpi<C: GKREngine>(
    p_keys: &ExpanderProverSetup<C::FieldConfig, C::PCSConfig>,
    commitments_values: &[impl AsRef<[SIMDField<C>]>],
    commitments_state: &[&ExpanderCommitmentState<C::FieldConfig, C::PCSConfig>],
    gkr_challenge: &ExpanderSingleVarChallenge<C::FieldConfig>,
    is_broadcast: &[bool],
    transcript: &mut C::TranscriptConfig,
) {
    let parallel_count = 1 << gkr_challenge.r_mpi.len();
    for ((commitment_val, _state), ib) in commitments_values
        .iter()
        .zip(commitments_state)
        .zip(is_broadcast)
    {
        let val_len = commitment_val.as_ref().len();
        let (challenge_for_pcs, _) = partition_challenge_and_location_for_pcs_mpi(
            gkr_challenge,
            val_len,
            parallel_count,
            *ib,
        );

        pcs_local_open_impl::<C>(
            commitment_val.as_ref(),
            &challenge_for_pcs,
            p_keys,
            transcript,
        );
    }
}

// ==================== SCHEDULE-BASED EXECUTION ====================

/// Schedule representation: rank -> sequence of tasks
#[derive(Debug, Clone)]
pub struct Schedule {
    /// Map from rank to list of task names
    /// e.g., rank 0 -> ["Task14", "Task1", "Task12"]
    pub rank_tasks: HashMap<usize, Vec<String>>,
}

impl Schedule {
    /// Parse schedule from text file
    /// Format: "Rank 0: Task14 -> Task1 -> Task12"
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read schedule file: {}", e))?;

        let mut rank_tasks = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Parse "Rank X: TaskA -> TaskB -> TaskC"
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid line format: {}", line));
            }

            // Handle multiple spaces: "Rank  0:" or "Rank 0:"
            let rank_part = parts[0].trim();
            if !rank_part.starts_with("Rank") {
                return Err(format!("Expected 'Rank X', got: {}", parts[0]));
            }

            let rank_str = rank_part
                .strip_prefix("Rank")
                .ok_or_else(|| format!("Expected 'Rank X', got: {}", parts[0]))?
                .trim(); // Trim to handle multiple spaces

            let rank: usize = rank_str
                .parse()
                .map_err(|e| format!("Invalid rank number '{}': {}", rank_str, e))?;

            // Extract tasks
            let tasks_str = parts[1].trim();
            let tasks: Vec<String> = tasks_str
                .split("->")
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();

            rank_tasks.insert(rank, tasks);
        }

        Ok(Schedule { rank_tasks })
    }

    /// Get tasks for a specific rank
    pub fn get_tasks(&self, rank: usize) -> Option<&Vec<String>> {
        self.rank_tasks.get(&rank)
    }

    /// Find which ranks are executing the same task at the same step
    pub fn find_peers_at_step(&self, step: usize, task_name: &str) -> Vec<usize> {
        let mut peers = Vec::new();

        for (rank, tasks) in &self.rank_tasks {
            if tasks.len() > step && tasks[step] == task_name {
                peers.push(*rank);
            }
        }

        peers.sort();
        peers
    }

    /// Get maximum number of steps across all ranks
    pub fn max_steps(&self) -> usize {
        self.rank_tasks
            .values()
            .map(|tasks| tasks.len())
            .max()
            .unwrap_or(0)
    }
}

/// Parse task mapping file
/// Format: "Task1: 0" (Task1 maps to template index 0)
pub fn parse_task_mapping(path: &str) -> Result<HashMap<String, usize>, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read task mapping file: {}", e))?;

    let mut mapping = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() != 2 {
            continue;
        }

        let task_name = parts[0].trim().to_string();
        let template_idx: usize = parts[1]
            .trim()
            .parse()
            .map_err(|e| format!("Invalid template index: {}", e))?;

        mapping.insert(task_name, template_idx);
    }

    Ok(mapping)
}

/// Create MPI subgroup for a specific task based on peers
fn create_mpi_subgroup_for_task(
    global_mpi_config: &MPIConfig<'static>,
    peers: &[usize],
    task_name: &str,
) -> Option<MPIConfig<'static>> {
    use mpi::topology::Communicator;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let my_rank = global_mpi_config.world_rank();

    // Find my position in peers
    let my_position = peers.iter().position(|&r| r == my_rank)?;

    // Use task name hash as color
    let mut hasher = DefaultHasher::new();
    task_name.hash(&mut hasher);
    let color_value = (hasher.finish() % 10000) as i32;
    let color = mpi::topology::Color::with_value(color_value);

    // Split communicator and leak it to get 'static lifetime
    let split_comm = unsafe {
        global_mpi_config
            .world
            .unwrap()
            .split_by_color_with_key(color, my_position as i32)
    };

    // Leak the communicator to get 'static lifetime
    // split_comm is Option<SimpleCommunicator>, we need to leak the inner value
    let split_comm_static: &'static Option<_> = Box::leak(Box::new(split_comm));

    Some(MPIConfig::prover_new(
        global_mpi_config.universe,
        split_comm_static.as_ref(),
    ))
}

/// Main prove function with schedule
pub fn mpi_prove_with_schedule<C, ECCConfig>(
    global_mpi_config: &MPIConfig<'static>,
    schedule_path: &str,
    task_mapping_path: Option<&str>, // Optional: if None, use template index as task name
    prover_setup: &ExpanderProverSetup<C::FieldConfig, C::PCSConfig>,
    computation_graph: &ComputationGraph<ECCConfig>,
    values: &[impl AsRef<[SIMDField<C>]>],
) -> Option<CombinedProof<ECCConfig, Expander<C>>>
where
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
{
    let my_rank = global_mpi_config.world_rank();

    // 1. Load schedule
    let schedule = match Schedule::from_file(schedule_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[RANK {}] Failed to load schedule: {}", my_rank, e);
            return None;
        }
    };

    eprintln!(
        "[RANK {}] Loaded schedule with {} ranks, max {} steps",
        my_rank,
        schedule.rank_tasks.len(),
        schedule.max_steps()
    );

    // 2. Load task mapping (or use default: TaskX -> template X)
    let task_mapping = if let Some(path) = task_mapping_path {
        match parse_task_mapping(path) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("[RANK {}] Failed to load task mapping: {}", my_rank, e);
                return None;
            }
        }
    } else {
        // Default: Task0->0, Task1->1, etc.
        let mut default_mapping = HashMap::new();
        for i in 0..computation_graph.proof_templates().len() {
            default_mapping.insert(format!("Task{}", i), i);
        }
        default_mapping
    };

    // 3. Commit phase (only root)
    let (commitments, states) = if global_mpi_config.is_root() {
        eprintln!("[RANK {}] === COMMIT PHASE ===", my_rank);
        let commit_timer = Timer::new("Commit to all input", true);
        let (commitments, states) = values
            .iter()
            .map(|value| {
                local_commit_impl::<C, ECCConfig>(
                    prover_setup.p_keys.get(&value.as_ref().len()).unwrap(),
                    value.as_ref(),
                )
            })
            .unzip::<_, _, Vec<_>, Vec<_>>();
        commit_timer.stop();
        (Some(commitments), Some(states))
    } else {
        (None, None)
    };

    // 4. Get my tasks
    let my_tasks = match schedule.get_tasks(my_rank) {
        Some(tasks) => tasks,
        None => {
            eprintln!("[RANK {}] No tasks assigned in schedule", my_rank);
            return if my_rank == 0 {
                Some(CombinedProof {
                    commitments: commitments.unwrap(),
                    proofs: vec![],
                })
            } else {
                None
            };
        }
    };

    eprintln!("[RANK {}] My tasks: {:?}", my_rank, my_tasks);

    // 5. Execute tasks step by step
    let mut all_proofs: Vec<Option<ExpanderProof>> =
        vec![None; computation_graph.proof_templates().len()];

    for (step, task_name) in my_tasks.iter().enumerate() {
        eprintln!(
            "[RANK {}] === STEP {} === Task: {}",
            my_rank, step, task_name
        );

        // Find template index
        let template_idx = match task_mapping.get(task_name) {
            Some(&idx) => idx,
            None => {
                eprintln!("[RANK {}] Unknown task: {}", my_rank, task_name);
                continue;
            }
        };

        if template_idx >= computation_graph.proof_templates().len() {
            eprintln!(
                "[RANK {}] Invalid template index: {}",
                my_rank, template_idx
            );
            continue;
        }

        // Find peers for this task at this step
        let peers = schedule.find_peers_at_step(step, task_name);
        eprintln!("[RANK {}] Task {} peers: {:?}", my_rank, task_name, peers);

        if peers.is_empty() || !peers.contains(&my_rank) {
            eprintln!("[RANK {}] Not participating in task {}", my_rank, task_name);
            continue;
        }

        let template = &computation_graph.proof_templates()[template_idx];
        let kernel = &computation_graph.kernels()[template.kernel_id()];

        let commitment_values = template
            .commitment_indices()
            .iter()
            .map(|&idx| values[idx].as_ref())
            .collect::<Vec<_>>();

        // Create MPI subgroup if multiple peers
        let local_mpi_config = if peers.len() > 1 {
            create_mpi_subgroup_for_task(global_mpi_config, &peers, task_name)
        } else {
            None
        };

        // Execute GKR
        let gkr_result = if let Some(ref local_config) = local_mpi_config {
            eprintln!(
                "[RANK {}] Executing task {} with {} peers (local_rank={})",
                my_rank,
                task_name,
                peers.len(),
                local_config.world_rank()
            );

            prove_kernel_gkr::<C::FieldConfig, C::TranscriptConfig, ECCConfig>(
                local_config,
                kernel,
                &commitment_values,
                next_power_of_two(template.parallel_count()),
                template.is_broadcast(),
            )
        } else {
            // Single rank task
            eprintln!("[RANK {}] Executing task {} solo", my_rank, task_name);
            None // Skip for now
        };

        // PCS opening (only subgroup root)
        if let Some((mut transcript, challenge)) = gkr_result {
            let is_subgroup_root = local_mpi_config
                .as_ref()
                .map(|c| c.is_root())
                .unwrap_or(true);

            if is_subgroup_root {
                eprintln!(
                    "[RANK {}] Performing PCS opening for task {}",
                    my_rank, task_name
                );

                let pcs_timer = Timer::new(&format!("PCS for {}", task_name), true);
                let challenges = if let Some(challenge_y) = challenge.challenge_y() {
                    vec![challenge.challenge_x(), challenge_y]
                } else {
                    vec![challenge.challenge_x()]
                };

                challenges.iter().for_each(|c| {
                    partition_single_gkr_claim_and_open_pcs_mpi::<C>(
                        prover_setup,
                        &commitment_values,
                        &template
                            .commitment_indices()
                            .iter()
                            .map(|&idx| &states.as_ref().unwrap()[idx])
                            .collect::<Vec<_>>(),
                        c,
                        template.is_broadcast(),
                        &mut transcript,
                    );
                });

                pcs_timer.stop();

                all_proofs[template_idx] = Some(ExpanderProof {
                    data: vec![transcript.finalize_and_get_proof()],
                });
            }
        }
    }

    // 6. Collect results (only root)
    if global_mpi_config.is_root() {
        let proofs = all_proofs.into_iter().filter_map(|p| p).collect::<Vec<_>>();
        eprintln!("[RANK {}] Collected {} proofs", my_rank, proofs.len());

        Some(CombinedProof {
            commitments: commitments.unwrap(),
            proofs,
        })
    } else {
        None
    }
}

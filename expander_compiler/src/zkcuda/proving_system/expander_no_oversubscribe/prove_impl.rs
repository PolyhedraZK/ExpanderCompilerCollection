use crate::zkcuda::cpu_monitor::CpuMonitor;
use arith::{Field, Fr, SimdField};
use expander_utils::timer::Timer;
use gkr_engine::{
    BN254ConfigXN, ExpanderDualVarChallenge, FieldEngine, GKREngine, MPIConfig, MPIEngine,
    Transcript,
};
use std::collections::HashMap;
use std::fs;

/// 获取当前进程的内存使用情况 (RSS, 单位: KB)
fn get_memory_kb() -> u64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/self/statm") {
            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(rss_pages) = parts[1].parse::<u64>() {
                    return rss_pages * 4; // 页大小 4KB
                }
            }
        }
    }
    0
}

fn log_memory(rank: usize, tag: &str) {
    let mem_kb = get_memory_kb();
    eprintln!(
        "[MEM] rank={} {} : {} KB ({:.2} MB)",
        rank,
        tag,
        mem_kb,
        mem_kb as f64 / 1024.0
    );
}

use crate::{
    frontend::{Config, SIMDField},
    utils::misc::next_power_of_two,
    zkcuda::{
        context::ComputationGraph,
        kernel::{Kernel, LayeredCircuitInputVec},
        proving_system::{
            expander::{
                commit_impl::local_commit_impl,
                config::{GetFieldConfig, GetPCS, GetTranscript, ZKCudaConfig},
                prove_impl::{
                    get_local_vals, prepare_expander_circuit, prepare_inputs_with_local_vals,
                },
                structs::{ExpanderProof, ExpanderProverSetup},
            },
            expander_no_oversubscribe::profiler::NBytesProfiler,
            expander_parallelized::{
                prove_impl::partition_single_gkr_claim_and_open_pcs_mpi,
                server_ctrl::generate_local_mpi_config,
            },
            expander_pcs_defered::prove_impl::{
                extract_pcs_claims, max_len_setup_commit_impl, open_defered_pcs,
            },
            CombinedProof, Expander,
        },
    },
};

pub fn mpi_prove_no_oversubscribe_impl<ZC: ZKCudaConfig>(
    global_mpi_config: &MPIConfig<'static>,
    prover_setup: &ExpanderProverSetup<GetFieldConfig<ZC>, GetPCS<ZC>>,
    computation_graph: &ComputationGraph<ZC::ECCConfig>,
    values: &[impl AsRef<[SIMDField<ZC::ECCConfig>]>],
    n_bytes_profiler: &mut NBytesProfiler,
) -> Option<CombinedProof<ZC::ECCConfig, Expander<ZC::GKRConfig>>>
where
    <ZC::GKRConfig as GKREngine>::FieldConfig: FieldEngine<CircuitField = Fr, ChallengeField = Fr>,
{
    // Check for schedule file and use scheduler if available
    if std::path::Path::new("schedule.txt").exists() {
        let my_rank = global_mpi_config.world_rank();
        eprintln!(
            "[RANK {}] ⚡ Schedule file detected, using scheduled execution",
            my_rank
        );
        return mpi_prove_no_oversubscribe_with_schedule::<ZC>(
            global_mpi_config,
            "schedule.txt",
            Some("task_mapping.txt"),
            prover_setup,
            computation_graph,
            values,
            n_bytes_profiler,
        );
    }

    let commit_timer = Timer::new("Commit to all input", global_mpi_config.is_root());
    let (commitments, states) = if global_mpi_config.is_root() {
        eprintln!("\n========== COMMIT PHASE START ==========");
        eprintln!(
            "[RANK {}] Starting commit on {} values",
            global_mpi_config.world_rank(),
            values.len()
        );

        // 启动CPU监控（每200ms采样一次）
        let _cpu_monitor = CpuMonitor::start("COMMIT", 200);

        let (commitments, states) = values
            .iter()
            .map(|value| match ZC::BATCH_PCS {
                true => max_len_setup_commit_impl::<ZC::GKRConfig, ZC::ECCConfig>(
                    prover_setup,
                    value.as_ref(),
                ),
                false => local_commit_impl::<ZC::GKRConfig, ZC::ECCConfig>(
                    prover_setup.p_keys.get(&value.as_ref().len()).unwrap(),
                    value.as_ref(),
                ),
            })
            .unzip::<_, _, Vec<_>, Vec<_>>();

        // _cpu_monitor在这里自动drop，停止监控
        eprintln!("========== COMMIT PHASE END ==========\n");

        (Some(commitments), Some(states))
    } else {
        eprintln!(
            "[RANK {}] Skipping commit (not root)",
            global_mpi_config.world_rank()
        );
        (None, None)
    };
    commit_timer.stop();

    let mut vals_ref = vec![];
    let mut challenges = vec![];

    let prove_timer = Timer::new("Prove all kernels", global_mpi_config.is_root());
    let proofs =
        computation_graph
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
                let gkr_end_state = prove_kernel_gkr_no_oversubscribe::<
                    GetFieldConfig<ZC>,
                    GetTranscript<ZC>,
                    ZC::ECCConfig,
                >(
                    global_mpi_config,
                    &computation_graph.kernels()[template.kernel_id()],
                    &commitment_values,
                    next_power_of_two(template.parallel_count()),
                    template.is_broadcast(),
                    n_bytes_profiler,
                );
                single_kernel_gkr_timer.stop();

                match ZC::BATCH_PCS {
                    true => {
                        if global_mpi_config.is_root() {
                            let (mut transcript, challenge) = gkr_end_state.unwrap();
                            assert!(challenge.challenge_y().is_none());
                            let challenge = challenge.challenge_x();

                            let (local_vals_ref, local_challenges) =
                                extract_pcs_claims::<ZC::GKRConfig>(
                                    &commitment_values,
                                    &challenge,
                                    template.is_broadcast(),
                                    next_power_of_two(template.parallel_count()),
                                );

                            vals_ref.extend(local_vals_ref);
                            challenges.extend(local_challenges);

                            Some(ExpanderProof {
                                data: vec![transcript.finalize_and_get_proof()],
                            })
                        } else {
                            None
                        }
                    }
                    false => {
                        if global_mpi_config.is_root() {
                            let pcs_open_timer = Timer::new("pcs open", true);
                            let (mut transcript, challenge) = gkr_end_state.unwrap();
                            let challenges = if let Some(challenge_y) = challenge.challenge_y() {
                                vec![challenge.challenge_x(), challenge_y]
                            } else {
                                vec![challenge.challenge_x()]
                            };

                            challenges.iter().for_each(|c| {
                                partition_single_gkr_claim_and_open_pcs_mpi::<ZC::GKRConfig>(
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
                    }
                }
            })
            .collect::<Vec<_>>();
    prove_timer.stop();

    match ZC::BATCH_PCS {
        true => {
            if global_mpi_config.is_root() {
                let mut proofs = proofs.into_iter().map(|p| p.unwrap()).collect::<Vec<_>>();
                eprintln!("\n========== PCS OPENING PHASE START ==========");
                eprintln!(
                    "[RANK {}] Starting batch PCS opening for {} values, {} challenges",
                    global_mpi_config.world_rank(),
                    vals_ref.len(),
                    challenges.len()
                );
                let pcs_opening_timer = Timer::new("Batch PCS Opening for all kernels", true);
                // 启动CPU监控
                let _cpu_monitor = CpuMonitor::start("PCS_OPENING", 200);
                let pcs_batch_opening = open_defered_pcs::<ZC::GKRConfig, ZC::ECCConfig>(
                    prover_setup,
                    &vals_ref,
                    &challenges,
                );
                pcs_opening_timer.stop();
                eprintln!("========== PCS OPENING PHASE END ==========\n");

                proofs.push(pcs_batch_opening);
                Some(CombinedProof {
                    commitments: commitments.unwrap(),
                    proofs,
                })
            } else {
                None
            }
        }
        false => {
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
    }
}

#[allow(clippy::too_many_arguments)]
pub fn prove_kernel_gkr_no_oversubscribe<F, T, ECCConfig>(
    mpi_config: &MPIConfig<'static>,
    kernel: &Kernel<ECCConfig>,
    commitments_values: &[&[F::SimdCircuitField]],
    parallel_count: usize,
    is_broadcast: &[bool],
    n_bytes_profiler: &mut NBytesProfiler,
) -> Option<(T, ExpanderDualVarChallenge<F>)>
where
    F: FieldEngine<CircuitField = Fr, ChallengeField = Fr>,
    T: Transcript,
    ECCConfig: Config<FieldConfig = F>,
{
    let local_mpi_config = generate_local_mpi_config(mpi_config, parallel_count);

    local_mpi_config.as_ref()?;

    let local_mpi_config = local_mpi_config.unwrap();
    let local_world_size = local_mpi_config.world_size();

    let n_local_copies = parallel_count / local_world_size;
    match n_local_copies {
        1 => prove_kernel_gkr_internal::<F, F, T, ECCConfig>(
            &local_mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
            n_bytes_profiler,
        ),
        2 => prove_kernel_gkr_internal::<F, BN254ConfigXN<2>, T, ECCConfig>(
            &local_mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
            n_bytes_profiler,
        ),
        4 => prove_kernel_gkr_internal::<F, BN254ConfigXN<4>, T, ECCConfig>(
            &local_mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
            n_bytes_profiler,
        ),
        8 => prove_kernel_gkr_internal::<F, BN254ConfigXN<8>, T, ECCConfig>(
            &local_mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
            n_bytes_profiler,
        ),
        16 => prove_kernel_gkr_internal::<F, BN254ConfigXN<16>, T, ECCConfig>(
            &local_mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
            n_bytes_profiler,
        ),
        32 => prove_kernel_gkr_internal::<F, BN254ConfigXN<32>, T, ECCConfig>(
            &local_mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
            n_bytes_profiler,
        ),
        64 => prove_kernel_gkr_internal::<F, BN254ConfigXN<64>, T, ECCConfig>(
            &local_mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
            n_bytes_profiler,
        ),
        128 => prove_kernel_gkr_internal::<F, BN254ConfigXN<128>, T, ECCConfig>(
            &local_mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
            n_bytes_profiler,
        ),
        256 => prove_kernel_gkr_internal::<F, BN254ConfigXN<256>, T, ECCConfig>(
            &local_mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
            n_bytes_profiler,
        ),
        512 => prove_kernel_gkr_internal::<F, BN254ConfigXN<512>, T, ECCConfig>(
            &local_mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
            n_bytes_profiler,
        ),
        1024 => prove_kernel_gkr_internal::<F, BN254ConfigXN<1024>, T, ECCConfig>(
            &local_mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
            n_bytes_profiler,
        ),
        2048 => prove_kernel_gkr_internal::<F, BN254ConfigXN<2048>, T, ECCConfig>(
            &local_mpi_config,
            kernel,
            commitments_values,
            parallel_count,
            is_broadcast,
            n_bytes_profiler,
        ),
        _ => {
            panic!("Unsupported parallel count: {parallel_count}");
        }
    }
}

pub fn prove_kernel_gkr_internal<FBasic, FMulti, T, ECCConfig>(
    mpi_config: &MPIConfig<'static>,
    kernel: &Kernel<ECCConfig>,
    commitments_values: &[&[FBasic::SimdCircuitField]],
    parallel_count: usize,
    is_broadcast: &[bool],
    n_bytes_profiler: &mut NBytesProfiler,
) -> Option<(T, ExpanderDualVarChallenge<FBasic>)>
where
    FBasic: FieldEngine<CircuitField = Fr, ChallengeField = Fr>,
    FMulti:
        FieldEngine<CircuitField = FBasic::CircuitField, ChallengeField = FBasic::ChallengeField>,
    T: Transcript,
    ECCConfig: Config<FieldConfig = FBasic>,
{
    let world_rank = mpi_config.world_rank();
    let world_size = mpi_config.world_size();
    let n_copies = parallel_count / world_size;

    log_memory(world_rank, "prove_kernel_gkr_internal::start");

    let local_commitment_values = get_local_vals_multi_copies(
        commitments_values,
        is_broadcast,
        world_rank,
        n_copies,
        parallel_count,
    );
    log_memory(
        world_rank,
        "prove_kernel_gkr_internal::after_get_local_vals",
    );

    let (mut expander_circuit, mut prover_scratch) =
        prepare_expander_circuit::<FMulti, ECCConfig>(kernel, world_size);
    log_memory(
        world_rank,
        "prove_kernel_gkr_internal::after_prepare_expander_circuit",
    );

    let mut transcript = T::new();
    let challenge = prove_gkr_with_local_vals_multi_copies::<FBasic, FMulti, T>(
        &mut expander_circuit,
        &mut prover_scratch,
        &local_commitment_values,
        kernel.layered_circuit_input(),
        &mut transcript,
        mpi_config,
        n_bytes_profiler,
    );
    log_memory(world_rank, "prove_kernel_gkr_internal::after_prove_gkr");

    // expander_circuit 和 prover_scratch 在这里被 drop
    drop(expander_circuit);
    drop(prover_scratch);
    log_memory(world_rank, "prove_kernel_gkr_internal::after_drop_circuit");

    Some((transcript, challenge))
}

pub fn get_local_vals_multi_copies<'vals_life, F: Field>(
    global_vals: &'vals_life [impl AsRef<[F]>],
    is_broadcast: &[bool],
    local_world_rank: usize,
    n_copies: usize,
    parallel_count: usize,
) -> Vec<Vec<&'vals_life [F]>> {
    let parallel_indices = (0..n_copies)
        .map(|i| local_world_rank * n_copies + i)
        .collect::<Vec<_>>();

    parallel_indices
        .iter()
        .map(|&parallel_index| {
            get_local_vals(global_vals, is_broadcast, parallel_index, parallel_count)
        })
        .collect::<Vec<_>>()
}

pub fn prove_gkr_with_local_vals_multi_copies<FBasic, FMulti, T>(
    expander_circuit: &mut expander_circuit::Circuit<FMulti>,
    prover_scratch: &mut sumcheck::ProverScratchPad<FMulti>,
    local_commitment_values_multi_copies: &[Vec<impl AsRef<[FBasic::SimdCircuitField]>>],
    partition_info: &[LayeredCircuitInputVec],
    transcript: &mut T,
    mpi_config: &MPIConfig,
    _n_bytes_profiler: &mut NBytesProfiler,
) -> ExpanderDualVarChallenge<FBasic>
where
    FBasic: FieldEngine<CircuitField = Fr, ChallengeField = Fr>,
    FMulti:
        FieldEngine<CircuitField = FBasic::CircuitField, ChallengeField = FBasic::ChallengeField>,
    T: Transcript,
{
    let world_rank = mpi_config.world_rank();
    log_memory(world_rank, "prove_gkr::start");

    let input_vals_multi_copies = local_commitment_values_multi_copies
        .iter()
        .map(|local_commitment_values| {
            prepare_inputs_with_local_vals(
                1 << expander_circuit.log_input_size(),
                partition_info,
                local_commitment_values,
            )
        })
        .collect::<Vec<_>>();
    log_memory(world_rank, "prove_gkr::after_prepare_inputs");

    let mut input_vals =
        vec![FMulti::SimdCircuitField::ZERO; 1 << expander_circuit.log_input_size()];

    for (i, vals) in input_vals.iter_mut().enumerate() {
        let vals_unpacked = input_vals_multi_copies
            .iter()
            .flat_map(|v| v[i].unpack())
            .collect::<Vec<_>>();
        *vals = FMulti::SimdCircuitField::pack(&vals_unpacked);
    }
    expander_circuit.layers[0].input_vals = input_vals;
    log_memory(world_rank, "prove_gkr::after_set_input_vals");

    expander_circuit.fill_rnd_coefs(transcript);
    log_memory(world_rank, "prove_gkr::after_fill_rnd_coefs");

    expander_circuit.evaluate();
    log_memory(world_rank, "prove_gkr::after_evaluate");

    #[cfg(feature = "zkcuda_profile")]
    {
        expander_circuit.layers.iter().for_each(|layer| {
            layer.input_vals.iter().for_each(|val| {
                val.unpack().iter().for_each(|fr| {
                    _n_bytes_profiler.add_fr(*fr);
                })
            });
        });
    }

    let (claimed_v, challenge) =
        gkr::gkr_prove(expander_circuit, prover_scratch, transcript, mpi_config);
    log_memory(world_rank, "prove_gkr::after_gkr_prove");

    assert_eq!(claimed_v, FBasic::ChallengeField::from(0u32));

    let n_simd_vars_basic = FBasic::SimdCircuitField::PACK_SIZE.ilog2() as usize;

    ExpanderDualVarChallenge {
        rz_0: challenge.rz_0,
        rz_1: challenge.rz_1,
        r_simd: challenge.r_simd[..n_simd_vars_basic].to_vec(),
        r_mpi: {
            let mut v = challenge.r_simd[n_simd_vars_basic..].to_vec();
            v.extend(&challenge.r_mpi);
            v
        },
    }
}

// ==================== SCHEDULE-BASED EXECUTION ====================

/// Schedule representation: rank -> sequence of tasks
#[derive(Debug, Clone)]
pub struct Schedule {
    /// Map from rank to list of task names
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

    pub fn get_tasks(&self, rank: usize) -> Option<&Vec<String>> {
        self.rank_tasks.get(&rank)
    }

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

    pub fn max_steps(&self) -> usize {
        self.rank_tasks
            .values()
            .map(|tasks| tasks.len())
            .max()
            .unwrap_or(0)
    }

    /// Get all unique task names across all ranks
    pub fn get_all_unique_tasks(&self) -> Vec<String> {
        use std::collections::HashSet;
        let mut unique_tasks = HashSet::new();

        for tasks in self.rank_tasks.values() {
            for task in tasks {
                if task != "idle" && task != "..." {
                    unique_tasks.insert(task.clone());
                }
            }
        }

        let mut tasks: Vec<_> = unique_tasks.into_iter().collect();
        tasks.sort();
        tasks
    }

    /// Find all ranks that will execute a given task (across all steps)
    pub fn find_all_peers_for_task(&self, task_name: &str) -> Vec<usize> {
        let mut peers = Vec::new();

        for (rank, tasks) in &self.rank_tasks {
            if tasks.contains(&task_name.to_string()) {
                peers.push(*rank);
            }
        }

        peers.sort();
        peers
    }
}

/// Parse task mapping file
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

/// Parse task dependencies file
/// Format: "Task22: Task9, Task23, Task8, Task17, Task18"
pub fn parse_task_dependencies(path: &str) -> Result<HashMap<String, Vec<String>>, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read dependencies file: {}", e))?;

    let mut dependencies = HashMap::new();

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
        let deps: Vec<String> = parts[1]
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        dependencies.insert(task_name, deps);
    }

    Ok(dependencies)
}

/// Mark a task as completed by creating a marker file
fn mark_task_completed(task_name: &str, my_rank: usize, peers: &[usize]) {
    // Only the minimum rank in the peer group writes the file
    if let Some(&min_rank) = peers.iter().min() {
        if my_rank == min_rank {
            let marker_path = format!(".task_sync/{}.done", task_name);
            if let Err(e) = fs::write(
                &marker_path,
                format!(
                    "{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                ),
            ) {
                eprintln!(
                    "[RANK {}] Warning: Failed to write marker for {}: {}",
                    my_rank, task_name, e
                );
            } else {
                eprintln!(
                    "[RANK {}] ✓ Marked task {} as completed",
                    my_rank, task_name
                );
            }
        }
    }
}

/// Wait for task dependencies to be satisfied
fn wait_for_dependencies(task_name: &str, dependencies: &[String], my_rank: usize) {
    if dependencies.is_empty() {
        return;
    }

    eprintln!(
        "[RANK {}] Task {} waiting for {} dependencies: {:?}",
        my_rank,
        task_name,
        dependencies.len(),
        dependencies
    );

    for dep in dependencies {
        let marker_path = format!(".task_sync/{}.done", dep);

        if std::path::Path::new(&marker_path).exists() {
            eprintln!(
                "[RANK {}]   ✓ Dependency {} already satisfied",
                my_rank, dep
            );
            continue;
        }

        eprintln!("[RANK {}]   ⏳ Waiting for dependency: {}", my_rank, dep);

        let start_time = std::time::Instant::now();
        while !std::path::Path::new(&marker_path).exists() {
            std::thread::sleep(std::time::Duration::from_millis(100));

            // Timeout check (optional, for debugging)
            if start_time.elapsed().as_secs() > 600 {
                eprintln!(
                    "[RANK {}]   ⚠️  WARNING: Waiting for {} over 10 minutes!",
                    my_rank, dep
                );
            }
        }

        eprintln!(
            "[RANK {}]   ✓ Dependency {} satisfied (waited {:.1}s)",
            my_rank,
            dep,
            start_time.elapsed().as_secs_f64()
        );
    }

    eprintln!(
        "[RANK {}] All dependencies for {} satisfied",
        my_rank, task_name
    );
}

/// Create MPI subgroup for a specific task
/// CRITICAL: This is a collective operation - ALL ranks must call this function
fn create_mpi_subgroup_for_task(
    global_mpi_config: &MPIConfig<'static>,
    peers: &[usize],
    task_name: &str,
) -> Option<MPIConfig<'static>> {
    use mpi::topology::Communicator;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let my_rank = global_mpi_config.world_rank();

    // Check if I'm in the peer list
    let my_position_opt = peers.iter().position(|&r| r == my_rank);

    // Use different colors: 0 for participants, 1 for non-participants
    let (color_value, key) = if let Some(pos) = my_position_opt {
        // I'm in this task group
        let mut hasher = DefaultHasher::new();
        task_name.hash(&mut hasher);
        let base_color = (hasher.finish() % 5000) as i32;
        (base_color, pos as i32)
    } else {
        // I'm not in this task group, use a different color
        let mut hasher = DefaultHasher::new();
        task_name.hash(&mut hasher);
        let base_color = (hasher.finish() % 5000) as i32;
        (base_color + 5000, my_rank as i32) // Different color range for non-participants
    };

    let color = mpi::topology::Color::with_value(color_value);

    // CRITICAL: All ranks must call split (collective operation)
    let split_comm = unsafe {
        global_mpi_config
            .world
            .unwrap()
            .split_by_color_with_key(color, key)
    };

    // Only participants return a valid MPIConfig
    if my_position_opt.is_some() {
        let split_comm_static: &'static Option<_> = Box::leak(Box::new(split_comm));
        Some(MPIConfig::prover_new(
            global_mpi_config.universe,
            split_comm_static.as_ref(),
        ))
    } else {
        // Non-participants: still called split but don't use the result
        None
    }
}

/// Main prove function with schedule support
pub fn mpi_prove_no_oversubscribe_with_schedule<ZC: ZKCudaConfig>(
    global_mpi_config: &MPIConfig<'static>,
    schedule_path: &str,
    task_mapping_path: Option<&str>,
    prover_setup: &ExpanderProverSetup<GetFieldConfig<ZC>, GetPCS<ZC>>,
    computation_graph: &ComputationGraph<ZC::ECCConfig>,
    values: &[impl AsRef<[SIMDField<ZC::ECCConfig>]>],
    n_bytes_profiler: &mut NBytesProfiler,
) -> Option<CombinedProof<ZC::ECCConfig, Expander<ZC::GKRConfig>>>
where
    <ZC::GKRConfig as GKREngine>::FieldConfig: FieldEngine<CircuitField = Fr, ChallengeField = Fr>,
{
    let my_rank = global_mpi_config.world_rank();

    // Load schedule
    let schedule = match Schedule::from_file(schedule_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[RANK {}] Failed to load schedule: {}", my_rank, e);
            return None;
        }
    };

    eprintln!("[RANK {}] ========== SCHEDULER MODE ==========", my_rank);
    eprintln!(
        "[RANK {}] Loaded schedule with {} ranks, max {} steps",
        my_rank,
        schedule.rank_tasks.len(),
        schedule.max_steps()
    );

    // Safety checks
    let num_templates = computation_graph.proof_templates().len();
    let num_values = values.len();
    eprintln!(
        "[RANK {}] Computation graph has {} templates",
        my_rank, num_templates
    );
    eprintln!(
        "[RANK {}] Values array has {} elements",
        my_rank, num_values
    );

    if num_templates == 0 {
        eprintln!(
            "[RANK {}] ERROR: No templates in computation graph!",
            my_rank
        );
        return if my_rank == 0 {
            Some(CombinedProof {
                commitments: vec![],
                proofs: vec![],
            })
        } else {
            None
        };
    }

    if num_values == 0 {
        eprintln!("[RANK {}] ERROR: Values array is empty!", my_rank);
        return None;
    }

    // Load task mapping
    let task_mapping = if let Some(path) = task_mapping_path {
        match parse_task_mapping(path) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("[RANK {}] Failed to load task mapping: {}", my_rank, e);
                return None;
            }
        }
    } else {
        let mut default_mapping = HashMap::new();
        for i in 0..computation_graph.proof_templates().len() {
            default_mapping.insert(format!("Task{}", i), i);
        }
        default_mapping
    };

    // Load task dependencies (optional)
    let task_dependencies = if std::path::Path::new("task_dependencies.txt").exists() {
        match parse_task_dependencies("task_dependencies.txt") {
            Ok(deps) => {
                eprintln!("[RANK {}] Loaded {} task dependencies", my_rank, deps.len());
                deps
            }
            Err(e) => {
                eprintln!(
                    "[RANK {}] Warning: Failed to load dependencies: {}",
                    my_rank, e
                );
                HashMap::new()
            }
        }
    } else {
        eprintln!(
            "[RANK {}] No task_dependencies.txt found, using file-based sync",
            my_rank
        );
        HashMap::new()
    };

    // Create task sync directory (only root)
    if global_mpi_config.is_root() {
        fs::create_dir_all(".task_sync").ok();
        // Clean old markers
        if let Ok(entries) = fs::read_dir(".task_sync") {
            for entry in entries {
                if let Ok(entry) = entry {
                    fs::remove_file(entry.path()).ok();
                }
            }
        }
        eprintln!("[RANK 0] Initialized .task_sync directory");
    }
    global_mpi_config.barrier(); // Wait for directory creation

    // ========== PRE-CREATE ALL MPI SUBGROUPS ==========
    // CRITICAL: Create all task subgroups BEFORE any task execution
    // This allows ranks to proceed asynchronously without collective deadlock
    eprintln!(
        "[RANK {}] Pre-creating MPI subgroups for all tasks...",
        my_rank
    );

    let all_unique_tasks = schedule.get_all_unique_tasks();
    let mut task_mpi_configs: HashMap<String, Option<MPIConfig<'static>>> = HashMap::new();

    for task_name in &all_unique_tasks {
        let peers = schedule.find_all_peers_for_task(task_name);

        eprintln!(
            "[RANK {}] Creating MPI subgroup for task {} (peers: {:?})",
            my_rank, task_name, peers
        );

        // All 32 ranks call this together (collective operation)
        let mpi_config = if peers.len() >= 1 {
            create_mpi_subgroup_for_task(global_mpi_config, &peers, task_name)
        } else {
            None
        };

        task_mpi_configs.insert(task_name.clone(), mpi_config);
    }

    eprintln!(
        "[RANK {}] Pre-created {} MPI subgroups",
        my_rank,
        task_mpi_configs.len()
    );
    global_mpi_config.barrier(); // Ensure all subgroups created before proceeding

    // Commit phase (only root)
    let commit_timer = Timer::new("Commit to all input", global_mpi_config.is_root());
    let (commitments, states) = if global_mpi_config.is_root() {
        eprintln!("[RANK {}] === COMMIT PHASE ===", my_rank);
        let _cpu_monitor = CpuMonitor::start("COMMIT", 200);

        let (commitments, states) = values
            .iter()
            .map(|value| match ZC::BATCH_PCS {
                true => max_len_setup_commit_impl::<ZC::GKRConfig, ZC::ECCConfig>(
                    prover_setup,
                    value.as_ref(),
                ),
                false => local_commit_impl::<ZC::GKRConfig, ZC::ECCConfig>(
                    prover_setup.p_keys.get(&value.as_ref().len()).unwrap(),
                    value.as_ref(),
                ),
            })
            .unzip::<_, _, Vec<_>, Vec<_>>();
        (Some(commitments), Some(states))
    } else {
        (None, None)
    };
    commit_timer.stop();

    // Use indexed storage to maintain template order
    let num_templates = computation_graph.proof_templates().len();

    // Store vals and challenges per template to maintain order
    let mut vals_per_template: Vec<Option<Vec<Vec<SIMDField<ZC::ECCConfig>>>>> =
        vec![None; num_templates];
    let mut challenges_per_template: Vec<
        Option<Vec<gkr_engine::ExpanderSingleVarChallenge<GetFieldConfig<ZC>>>>,
    > = vec![None; num_templates];

    let mut vals_ref: Vec<&[SIMDField<ZC::ECCConfig>]> = vec![]; // Keep reference version for non-BATCH_PCS compatibility
    let mut challenges: Vec<gkr_engine::ExpanderSingleVarChallenge<GetFieldConfig<ZC>>> = vec![]; // For non-BATCH_PCS compatibility

    // Track which ranks are subgroup roots for result collection
    let mut i_am_subgroup_root_for_tasks = vec![];

    // Get my tasks
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

    // Execute tasks step by step
    let mut all_proofs: Vec<Option<ExpanderProof>> =
        vec![None; computation_graph.proof_templates().len()];

    for (step, task_name) in my_tasks.iter().enumerate() {
        eprintln!(
            "[RANK {}] === STEP {} === Task: {}",
            my_rank, step, task_name
        );

        // Skip idle steps
        if task_name == "idle" || task_name == "..." {
            // Idle ranks still participate in MPI collective operations
            continue;
        }

        // Wait for dependencies before proceeding
        if let Some(deps) = task_dependencies.get(task_name) {
            wait_for_dependencies(task_name, deps, my_rank);
        }

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

        // Get pre-created MPI config for this task
        let local_mpi_config = task_mpi_configs.get(task_name).and_then(|c| c.clone());

        // Check if I'm a participant (have valid MPI config or solo task)
        let all_peers = schedule.find_all_peers_for_task(task_name);
        let i_am_participant = all_peers.contains(&my_rank);

        if !i_am_participant {
            eprintln!("[RANK {}] Not participating in task {}", my_rank, task_name);
            continue;
        }

        eprintln!(
            "[RANK {}] Task {} peers: {:?} (using pre-created MPI subgroup)",
            my_rank, task_name, all_peers
        );

        let template = &computation_graph.proof_templates()[template_idx];

        // Safety check: verify all commitment indices are in bounds
        let commit_indices = template.commitment_indices();
        let mut has_error = false;
        for &idx in commit_indices {
            if idx >= values.len() {
                eprintln!(
                    "[RANK {}] ERROR: Template {} requires value index {} but values.len() = {}",
                    my_rank,
                    template_idx,
                    idx,
                    values.len()
                );
                has_error = true;
            }
        }
        if has_error {
            eprintln!(
                "[RANK {}] Skipping task {} due to index out of bounds",
                my_rank, task_name
            );
            continue;
        }

        let commitment_values = template
            .commitment_indices()
            .iter()
            .map(|&idx| values[idx].as_ref())
            .collect::<Vec<_>>();

        // Execute GKR
        let single_kernel_gkr_timer = Timer::new(
            &format!("Task {} GKR", task_name),
            local_mpi_config
                .as_ref()
                .map(|c| c.is_root())
                .unwrap_or(true),
        );

        let gkr_end_state = if let Some(ref local_config) = local_mpi_config {
            eprintln!(
                "[RANK {}] Executing task {} with {} peers (local_rank={}, group_size={})",
                my_rank,
                task_name,
                all_peers.len(),
                local_config.world_rank(),
                local_config.world_size()
            );

            prove_kernel_gkr_no_oversubscribe::<GetFieldConfig<ZC>, GetTranscript<ZC>, ZC::ECCConfig>(
                local_config,
                &computation_graph.kernels()[template.kernel_id()],
                &commitment_values,
                next_power_of_two(template.parallel_count()),
                template.is_broadcast(),
                n_bytes_profiler,
            )
        } else {
            eprintln!(
                "[RANK {}] Executing task {} solo (creating singl
       e-rank MPI config)",
                my_rank, task_name
            );

            // Create a single-rank MPI config for solo tasks
            // We use the pre-created config from task_mpi_configs
            let solo_config = task_mpi_configs.get(task_name).and_then(|c| c.as_ref());

            if let Some(config) = solo_config {
                prove_kernel_gkr_no_oversubscribe::<
                    GetFieldConfig<ZC>,
                    GetTranscript<ZC>,
                    ZC::ECCConfig,
                >(
                    config,
                    &computation_graph.kernels()[template.kernel_id()],
                    &commitment_values,
                    next_power_of_two(template.parallel_count()),
                    template.is_broadcast(),
                    n_bytes_profiler,
                )
            } else {
                // Fallback: use global MPI config for solo task
                prove_kernel_gkr_no_oversubscribe::<
                    GetFieldConfig<ZC>,
                    GetTranscript<ZC>,
                    ZC::ECCConfig,
                >(
                    global_mpi_config,
                    &computation_graph.kernels()[template.kernel_id()],
                    &commitment_values,
                    next_power_of_two(template.parallel_count()),
                    template.is_broadcast(),
                    n_bytes_profiler,
                )
            }
        };

        single_kernel_gkr_timer.stop();

        // PCS opening
        if let Some((mut transcript, challenge)) = gkr_end_state {
            let is_subgroup_root = local_mpi_config
                .as_ref()
                .map(|c| c.is_root())
                .unwrap_or(true);

            if is_subgroup_root {
                eprintln!(
                    "[RANK {}] I am subgroup root for task {}",
                    my_rank, task_name
                );
                i_am_subgroup_root_for_tasks.push(template_idx);

                match ZC::BATCH_PCS {
                    true => {
                        assert!(challenge.challenge_y().is_none());
                        let challenge_x = challenge.challenge_x();

                        let (local_vals_ref, local_challenges) = extract_pcs_claims::<ZC::GKRConfig>(
                            &commitment_values,
                            &challenge_x,
                            template.is_broadcast(),
                            next_power_of_two(template.parallel_count()),
                        );

                        // Store in indexed structure to maintain template order
                        let owned_vals: Vec<Vec<_>> =
                            local_vals_ref.iter().map(|v| v.to_vec()).collect();
                        vals_per_template[template_idx] = Some(owned_vals);
                        challenges_per_template[template_idx] = Some(local_challenges);

                        all_proofs[template_idx] = Some(ExpanderProof {
                            data: vec![transcript.finalize_and_get_proof()],
                        });
                    }
                    false => {
                        let pcs_open_timer = Timer::new(&format!("Task {} PCS", task_name), true);
                        let challenge_list = if let Some(challenge_y) = challenge.challenge_y() {
                            vec![challenge.challenge_x(), challenge_y]
                        } else {
                            vec![challenge.challenge_x()]
                        };

                        challenge_list.iter().for_each(|c| {
                            partition_single_gkr_claim_and_open_pcs_mpi::<ZC::GKRConfig>(
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
                        all_proofs[template_idx] = Some(ExpanderProof {
                            data: vec![transcript.finalize_and_get_proof()],
                        });
                    }
                }
            }
        }

        // Mark task as completed (file-based synchronization)
        mark_task_completed(task_name, my_rank, &all_peers);
    }

    // ========== CRITICAL: Global barrier ==========
    // Wait for all ranks to complete all their tasks before proceeding to PCS opening
    eprintln!(
        "[RANK {}] All my tasks completed, waiting for other ranks...",
        my_rank
    );
    global_mpi_config.barrier();
    eprintln!(
        "[RANK {}] All ranks ready, proceeding to result collection",
        my_rank
    );

    // ========== MPI Result Collection (for BATCH_PCS mode) ==========
    // Collect vals_ref and challenges from all subgroup roots to global root
    if ZC::BATCH_PCS {
        use mpi::traits::*;
        use serdes::ExpSerde;

        let i_am_subgroup_root = !i_am_subgroup_root_for_tasks.is_empty();

        eprintln!(
            "[RANK {}] Am I subgroup root? {} (for {} tasks)",
            my_rank,
            i_am_subgroup_root,
            i_am_subgroup_root_for_tasks.len()
        );

        // Step 0: All ranks send a flag to root indicating if they are subgroup roots
        let my_flag = if i_am_subgroup_root { 1u8 } else { 0u8 };

        let all_flags = if global_mpi_config.is_root() {
            // Root collects flags from all ranks
            let mut flags = vec![0u8; global_mpi_config.world_size()];
            unsafe {
                global_mpi_config
                    .world
                    .unwrap()
                    .all_gather_into(&my_flag, &mut flags[..]);
            }
            Some(flags)
        } else {
            unsafe {
                let mut flags = vec![0u8; global_mpi_config.world_size()];
                global_mpi_config
                    .world
                    .unwrap()
                    .all_gather_into(&my_flag, &mut flags[..]);
            }
            None
        };

        // Step 1: Non-root subgroup roots send their results to rank 0
        if i_am_subgroup_root && my_rank != 0 {
            eprintln!(
                "[RANK {}] Sending my results to global root (indexed by template)",
                my_rank
            );

            // Serialize the indexed structures (maintains template order)
            let mut vals_bytes = Vec::new();
            vals_per_template.serialize_into(&mut vals_bytes).unwrap();

            let mut challenges_bytes = Vec::new();
            challenges_per_template
                .serialize_into(&mut challenges_bytes)
                .unwrap();

            let mut proofs_bytes = Vec::new();
            all_proofs.serialize_into(&mut proofs_bytes).unwrap();

            // Send sizes first
            let sizes = [vals_bytes.len(), challenges_bytes.len(), proofs_bytes.len()];

            unsafe {
                global_mpi_config
                    .world
                    .unwrap()
                    .process_at_rank(0)
                    .synchronous_send(&sizes[..]);

                global_mpi_config
                    .world
                    .unwrap()
                    .process_at_rank(0)
                    .synchronous_send(&vals_bytes[..]);

                global_mpi_config
                    .world
                    .unwrap()
                    .process_at_rank(0)
                    .synchronous_send(&challenges_bytes[..]);

                global_mpi_config
                    .world
                    .unwrap()
                    .process_at_rank(0)
                    .synchronous_send(&proofs_bytes[..]);
            }

            eprintln!("[RANK {}] Results sent to global root", my_rank);
        }

        // Step 2: Global root receives all results
        if global_mpi_config.is_root() {
            eprintln!("[RANK 0] Collecting results from all subgroup roots...");

            let flags = all_flags.unwrap();

            // Identify which ranks are subgroup roots (have flag=1)
            let subgroup_roots: Vec<usize> = flags
                .iter()
                .enumerate()
                .filter(|(_, &flag)| flag == 1)
                .map(|(rank, _)| rank)
                .collect();

            eprintln!("[RANK 0] Subgroup roots detected: {:?}", subgroup_roots);

            // Receive from each subgroup root (except self)
            for &sender_rank in &subgroup_roots {
                if sender_rank == 0 {
                    continue; // Skip self
                }

                eprintln!("[RANK 0] Receiving results from rank {}", sender_rank);

                // Receive sizes
                let (sizes, _status) = unsafe {
                    global_mpi_config
                        .world
                        .unwrap()
                        .process_at_rank(sender_rank as i32)
                        .receive_vec::<usize>()
                };

                // Receive vals_per_template (indexed structure)
                let (vals_bytes, _) = unsafe {
                    global_mpi_config
                        .world
                        .unwrap()
                        .process_at_rank(sender_rank as i32)
                        .receive_vec::<u8>()
                };
                let received_vals_per_template: Vec<Option<Vec<Vec<SIMDField<ZC::ECCConfig>>>>> =
                    Vec::deserialize_from(&mut vals_bytes.as_slice()).unwrap();

                // Receive challenges_per_template
                let (challenges_bytes, _) = unsafe {
                    global_mpi_config
                        .world
                        .unwrap()
                        .process_at_rank(sender_rank as i32)
                        .receive_vec::<u8>()
                };
                let received_challenges_per_template: Vec<
                    Option<Vec<gkr_engine::ExpanderSingleVarChallenge<GetFieldConfig<ZC>>>>,
                > = Vec::deserialize_from(&mut challenges_bytes.as_slice()).unwrap();

                // Receive proofs (indexed by template)
                let (proofs_bytes, _) = unsafe {
                    global_mpi_config
                        .world
                        .unwrap()
                        .process_at_rank(sender_rank as i32)
                        .receive_vec::<u8>()
                };
                let received_all_proofs: Vec<Option<ExpanderProof>> =
                    Vec::deserialize_from(&mut proofs_bytes.as_slice()).unwrap();

                // Merge indexed data (maintains template order)
                for template_idx in 0..num_templates {
                    // Merge vals
                    if received_vals_per_template[template_idx].is_some() {
                        vals_per_template[template_idx] =
                            received_vals_per_template[template_idx].clone();
                    }

                    // Merge challenges
                    if received_challenges_per_template[template_idx].is_some() {
                        challenges_per_template[template_idx] =
                            received_challenges_per_template[template_idx].clone();
                    }

                    // Merge proofs
                    if received_all_proofs[template_idx].is_some() {
                        all_proofs[template_idx] = received_all_proofs[template_idx].clone();
                    }
                }

                let received_count = received_vals_per_template
                    .iter()
                    .filter(|v| v.is_some())
                    .count();
                eprintln!(
                    "[RANK 0] Received results from rank {} ({} templates)",
                    sender_rank, received_count
                );
            }

            // Build final vals_ref and challenges in template order
            let mut vals_ref_owned: Vec<Vec<SIMDField<ZC::ECCConfig>>> = vec![];
            let mut challenges_final = vec![];

            for template_idx in 0..num_templates {
                if let Some(vals) = &vals_per_template[template_idx] {
                    vals_ref_owned.extend(vals.clone());
                }
                if let Some(chals) = &challenges_per_template[template_idx] {
                    challenges_final.extend(chals.clone());
                }
            }

            eprintln!(
                "[RANK 0] Result collection complete. Total: {} vals, {} challenges, {} proofs",
                vals_ref_owned.len(),
                challenges_final.len(),
                all_proofs.iter().filter(|p| p.is_some()).count()
            );

            eprintln!("[RANK 0] Templates coverage:");
            for (idx, val) in vals_per_template.iter().enumerate() {
                let status = if val.is_some() { "✓" } else { "✗" };
                eprintln!("  Template {}: {}", idx, status);
            }
        }
    }

    // Collect results
    match ZC::BATCH_PCS {
        true => {
            if global_mpi_config.is_root() {
                let mut proofs = all_proofs.into_iter().filter_map(|p| p).collect::<Vec<_>>();

                let pcs_opening_timer = Timer::new("Batch PCS Opening for all kernels", true);

                // Build final vals_ref and challenges in template order
                let mut vals_ref_owned: Vec<Vec<SIMDField<ZC::ECCConfig>>> = vec![];
                let mut challenges_final = vec![];

                for template_idx in 0..num_templates {
                    if let Some(vals) = &vals_per_template[template_idx] {
                        vals_ref_owned.extend(vals.clone());
                    }
                    if let Some(chals) = &challenges_per_template[template_idx] {
                        challenges_final.extend(chals.clone());
                    }
                }

                // Convert to references for open_defered_pcs
                let vals_ref_for_pcs: Vec<&[SIMDField<ZC::ECCConfig>]> =
                    vals_ref_owned.iter().map(|v| v.as_slice()).collect();

                let pcs_batch_opening = open_defered_pcs::<ZC::GKRConfig, ZC::ECCConfig>(
                    prover_setup,
                    &vals_ref_for_pcs,
                    &challenges_final,
                );
                pcs_opening_timer.stop();

                proofs.push(pcs_batch_opening);
                Some(CombinedProof {
                    commitments: commitments.unwrap(),
                    proofs,
                })
            } else {
                None
            }
        }
        false => {
            if global_mpi_config.is_root() {
                let proofs = all_proofs.into_iter().filter_map(|p| p).collect::<Vec<_>>();
                Some(CombinedProof {
                    commitments: commitments.unwrap(),
                    proofs,
                })
            } else {
                None
            }
        }
    }
}

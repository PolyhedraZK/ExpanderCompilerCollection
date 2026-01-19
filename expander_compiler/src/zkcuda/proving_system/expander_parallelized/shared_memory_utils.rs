#![allow(static_mut_refs)]

use crate::zkcuda::proving_system::expander_parallelized::server_ctrl::SharedMemoryWINWrapper;
use crate::zkcuda::proving_system::{CombinedProof, Expander};
use arith::Field;
use gkr_engine::{ExpanderPCS, FieldEngine, GKREngine, MPIConfig, MPIEngine, MPISharedMemory};
use serdes::ExpSerde;
use shared_memory::{Shmem, ShmemConf};

use crate::circuit::config::Config;

/// 获取所有 expander_server 进程的内存占用（单位：MB）
/// 返回 (VmRSS物理内存, VmSize虚拟内存)
fn get_total_expander_memory_mb() -> (usize, usize) {
    use std::fs;
    use std::io::{BufRead, BufReader};

    let mut total_rss_kb = 0usize;
    let mut total_vmsize_kb = 0usize;

    // 遍历 /proc 目录
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                // 只处理数字目录（进程PID）
                if file_name.chars().all(|c| c.is_ascii_digit()) {
                    // 读取 /proc/[pid]/comm 检查进程名
                    let comm_path = format!("/proc/{}/comm", file_name);
                    if let Ok(comm) = fs::read_to_string(&comm_path) {
                        if comm.trim() == "expander_server" {
                            // 读取 /proc/[pid]/status 获取内存信息
                            let status_path = format!("/proc/{}/status", file_name);
                            if let Ok(file) = fs::File::open(&status_path) {
                                let reader = BufReader::new(file);
                                for line in reader.lines().flatten() {
                                    if line.starts_with("VmRSS:") {
                                        // VmRSS: 12345 kB (物理内存)
                                        if let Some(rss_str) = line.split_whitespace().nth(1) {
                                            if let Ok(rss_kb) = rss_str.parse::<usize>() {
                                                total_rss_kb += rss_kb;
                                            }
                                        }
                                    } else if line.starts_with("VmSize:") {
                                        // VmSize: 12345 kB (虚拟内存)
                                        if let Some(size_str) = line.split_whitespace().nth(1) {
                                            if let Ok(size_kb) = size_str.parse::<usize>() {
                                                total_vmsize_kb += size_kb;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    (total_rss_kb / 1024, total_vmsize_kb / 1024) // 转换为MB
}

use crate::zkcuda::proving_system::expander::structs::{
    ExpanderProverSetup, ExpanderVerifierSetup,
};

#[derive(Default)]
pub struct SharedMemory {
    pub pcs_setup: Option<Shmem>,
    pub witness: Option<Shmem>,
    pub proof: Option<Shmem>,
}

pub static mut SHARED_MEMORY: SharedMemory = SharedMemory {
    pcs_setup: None,
    witness: None,
    proof: None,
};

pub struct SharedMemoryEngine {}

/// This impl block contains utility functions for managing shared memory in the context of the Expander GKR proving system.
impl SharedMemoryEngine {
    /// Allocate shared memory for the given name and size if it is not already allocated or if the existing allocation is smaller than the target size.
    /// The result is stored in the provided `handle`, it's the caller's responsibility to ensure that the `handle` lives long enough for the reader to access the shared memory.
    fn allocate_shared_memory_if_necessary(
        handle: &mut Option<Shmem>,
        name: &str,
        target_size: usize,
    ) {
        if handle.is_some() && handle.as_ref().unwrap().len() >= target_size {
            return;
        }
        *handle = None;
        *handle = Some(
            ShmemConf::new()
                .size(target_size)
                .flink(name)
                .force_create_flink()
                .create()
                .unwrap(),
        );
    }

    /// Write an object to shared memory. If the shared memory is not allocated or is too small, it will be allocated with the size of the serialized object.
    fn write_object_to_shared_memory<T: ExpSerde>(
        object: &T,
        shared_memory_ref: &mut Option<Shmem>,
        name: &str,
    ) {
        let mut buffer = vec![];
        object
            .serialize_into(&mut buffer)
            .expect("Failed to serialize object");

        println!("Object size: {}", buffer.len());

        unsafe {
            Self::allocate_shared_memory_if_necessary(shared_memory_ref, name, buffer.len());
            let object_ptr = shared_memory_ref.as_mut().unwrap().as_ptr();
            std::ptr::copy_nonoverlapping(buffer.as_ptr(), object_ptr, buffer.len());
        }
    }

    /// Read an object from shared memory. If the shared memory is not allocated, it will panic.
    pub fn read_object_from_shared_memory<T: ExpSerde>(
        shared_memory_ref: &str,
        offset: usize,
    ) -> T {
        let shmem = ShmemConf::new()
            .flink(shared_memory_ref)
            .open()
            .expect("Failed to open shared memory");
        let object_ptr = shmem.as_ptr() as *const u8;
        let object_len = shmem.len();
        let buffer =
            unsafe { std::slice::from_raw_parts(object_ptr.add(offset), object_len - offset) };
        T::deserialize_from(buffer).expect("Failed to deserialize object")
    }
}

/// This impl block contains functions for reading/writing specific objects to shared memory.
impl SharedMemoryEngine {
    pub fn write_pcs_setup_to_shared_memory<F: FieldEngine, PCS: ExpanderPCS<F>>(
        pcs_setup: &(ExpanderProverSetup<F, PCS>, ExpanderVerifierSetup<F, PCS>),
    ) {
        println!("Writing PCS setup to shared memory...");
        Self::write_object_to_shared_memory(
            pcs_setup,
            unsafe { &mut SHARED_MEMORY.pcs_setup },
            "pcs_setup",
        );
    }

    pub fn read_pcs_setup_from_shared_memory<F: FieldEngine, PCS: ExpanderPCS<F>>(
    ) -> (ExpanderProverSetup<F, PCS>, ExpanderVerifierSetup<F, PCS>) {
        Self::read_object_from_shared_memory("pcs_setup", 0)
    }

    pub fn write_witness_to_shared_memory<F: FieldEngine>(values: Vec<Vec<F::SimdCircuitField>>) {
        let total_size = std::mem::size_of::<usize>()
            + values
                .iter()
                .map(|v| std::mem::size_of::<usize>() + std::mem::size_of_val(v.as_slice()))
                .sum::<usize>();

        println!("Writing witness to shared memory, total size: {total_size}");
        unsafe {
            Self::allocate_shared_memory_if_necessary(
                &mut SHARED_MEMORY.witness,
                "witness",
                total_size,
            );

            let mut ptr = SHARED_MEMORY.witness.as_mut().unwrap().as_ptr();

            // Copy the length of the vector
            let len = values.len();
            let len_ptr = &len as *const usize as *const u8;
            std::ptr::copy_nonoverlapping(len_ptr, ptr, std::mem::size_of::<usize>());
            ptr = ptr.add(std::mem::size_of::<usize>());

            for vals in values {
                let vals_len = vals.len();
                let len_ptr = &vals_len as *const usize as *const u8;
                std::ptr::copy_nonoverlapping(len_ptr, ptr, std::mem::size_of::<usize>());
                ptr = ptr.add(std::mem::size_of::<usize>());

                let vals_size = std::mem::size_of_val(vals.as_slice());
                std::ptr::copy_nonoverlapping(vals.as_ptr() as *const u8, ptr, vals_size);
                ptr = ptr.add(vals_size);
            }
        }
    }

    pub fn read_witness_from_shared_memory<F: FieldEngine>() -> Vec<Vec<F::SimdCircuitField>> {
        let shmem = ShmemConf::new().flink("witness").open().unwrap();
        let mut ptr = shmem.as_ptr();
        let n_components: usize =
            usize::deserialize_from(unsafe { std::slice::from_raw_parts(ptr, size_of::<usize>()) })
                .unwrap();
        ptr = unsafe { ptr.add(size_of::<usize>()) };

        (0..n_components)
            .map(|_| {
                let total_len_i: usize = usize::deserialize_from(unsafe {
                    std::slice::from_raw_parts(ptr, size_of::<usize>())
                })
                .unwrap();
                ptr = unsafe { ptr.add(size_of::<usize>()) };

                let mut vals = Vec::with_capacity(total_len_i);
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        ptr as *const F::SimdCircuitField,
                        vals.as_mut_ptr(),
                        total_len_i,
                    );
                    vals.set_len(total_len_i);
                }

                ptr = unsafe { ptr.add(total_len_i * <F::SimdCircuitField as Field>::SIZE) };
                vals
            })
            .collect()
    }

    pub fn read_shared_witness_from_shared_memory<F: FieldEngine>(
        global_mpi_config: &MPIConfig<'static>,
    ) -> (Vec<Vec<F::SimdCircuitField>>, SharedMemoryWINWrapper) {
        use std::time::Instant;

        let (rss_before, vmsize_before) = get_total_expander_memory_mb();
        // 打印关键信息：进程rank和witness长度
        println!("[MPI Rank {}] read_shared_witness_from_shared_memory: MEMORY_BEFORE = {} MB (RSS), {} MB (VmSize)",
                 global_mpi_config.world_rank(), rss_before, vmsize_before);
        let (mut mpi_shared_mem_ptr, mem_win) = if global_mpi_config.is_root() {
            let witness = Self::read_witness_from_shared_memory::<F>();
            let bytes_size = std::mem::size_of::<usize>()
                + witness.iter().map(|v| v.bytes_size()).sum::<usize>();
            let (mut mpi_shared_mem_ptr, mem_win) = global_mpi_config.create_shared_mem(bytes_size);
            let mpi_shared_mem_ptr_init = mpi_shared_mem_ptr;

            witness.len().to_memory(&mut mpi_shared_mem_ptr);
            witness.iter().for_each(|vals| {
                vals.to_memory(&mut mpi_shared_mem_ptr);
            });

            (mpi_shared_mem_ptr_init, mem_win)
        } else {
            global_mpi_config.create_shared_mem(0)
        };

        global_mpi_config.barrier();

        // ⏸️ 等待检查点：等待 /tmp/continue_witness_test 文件出现才继续
        let checkpoint_file = "/tmp/continue_witness_test1";
        println!(
            "[MPI Rank {}] ⏸️  CHECKPOINT: Waiting for file '{}' to continue...",
            global_mpi_config.world_rank(),
            checkpoint_file
        );
        println!("[MPI Rank {}] ⏸️  You can now check memory usage. Create the file to continue: touch {}",
                 global_mpi_config.world_rank(), checkpoint_file);

        let mut check_count = 0;
        loop {
            if std::path::Path::new(checkpoint_file).exists() {
                println!(
                    "[MPI Rank {}] ✅ Checkpoint file detected, continuing execution",
                    global_mpi_config.world_rank()
                );
                break;
            }

            check_count += 1;
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        // ⏱️ 开始计时：测量从共享内存读取witness的耗时
        let read_start = Instant::now();

        let n_witness = usize::new_from_memory(&mut mpi_shared_mem_ptr);
        let read_n_witness_duration = read_start.elapsed();

        println!(
            "[MPI Rank {}] ⏱️  Read n_witness={} took {:.3} µs",
            global_mpi_config.world_rank(),
            n_witness,
            read_n_witness_duration.as_micros()
        );

        let witness_read_start = Instant::now();
        let witness = (0..n_witness)
            .map(|_| Vec::<F::SimdCircuitField>::new_from_memory(&mut mpi_shared_mem_ptr))
            .collect::<Vec<_>>();
        let witness_read_duration = witness_read_start.elapsed();

        println!("[MPI Rank {}] ⏱️  Read {} witness components from shared memory took {:.3} ms ({:.3} µs)",
                 global_mpi_config.world_rank(),
                 n_witness,
                 witness_read_duration.as_secs_f64() * 1000.0,
                 witness_read_duration.as_micros());

        let (rss_after, vmsize_after) = get_total_expander_memory_mb();

        // 打印每个witness component的大小
        let total_elements: usize = witness.iter().map(|v| v.len()).sum();
        let total_bytes: usize = witness
            .iter()
            .map(|v| v.len() * std::mem::size_of_val(&v[0]))
            .sum();
        let rss_increase = rss_after.saturating_sub(rss_before);
        let vmsize_increase = vmsize_after.saturating_sub(vmsize_before);
        println!("[MPI Rank {}] Copied witness to local memory: {} components, {} total elements, ~{} MB witness data",
                 global_mpi_config.world_rank(),
                 witness.len(),
                 total_elements,
                 total_bytes / 1024 / 1024);
        println!(
            "[MPI Rank {}] MEMORY_AFTER_COPY: RSS = {} MB (+{} MB), VmSize = {} MB (+{} MB)",
            global_mpi_config.world_rank(),
            rss_after,
            rss_increase,
            vmsize_after,
            vmsize_increase
        );

        // ⏸️ 等待检查点：等待 /tmp/continue_witness_test 文件出现才继续
        let checkpoint_file = "/tmp/continue_witness_test";
        println!(
            "[MPI Rank {}] ⏸️  CHECKPOINT: Waiting for file '{}' to continue...",
            global_mpi_config.world_rank(),
            checkpoint_file
        );
        println!("[MPI Rank {}] ⏸️  You can now check memory usage. Create the file to continue: touch {}",
                 global_mpi_config.world_rank(), checkpoint_file);

        let mut check_count = 0;
        loop {
            if std::path::Path::new(checkpoint_file).exists() {
                println!(
                    "[MPI Rank {}] ✅ Checkpoint file detected, continuing execution",
                    global_mpi_config.world_rank()
                );
                break;
            }

            // 每10次检查打印一次内存状态（避免日志过多）
            if check_count % 10 == 0 {
                let (rss, vmsize) = get_total_expander_memory_mb();
                println!(
                    "[MPI Rank {}] ⏳ Still waiting... (check #{}, RSS = {} MB, VmSize = {} MB)",
                    global_mpi_config.world_rank(),
                    check_count,
                    rss,
                    vmsize
                );
            }

            check_count += 1;
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        // 🔥 主动访问witness数据，强制触发物理页分配
        println!("[MPI Rank {}] 🔥 Now actively accessing witness data to trigger physical page allocation...",
                 global_mpi_config.world_rank());

        let access_start = Instant::now();

        // 遍历所有witness数据，真正读取每个元素的字节
        let mut dummy_sum = 0u64;
        for component in witness.iter() {
            // 将Vec转为字节切片，确保访问实际内存
            let bytes: &[u8] = unsafe {
                std::slice::from_raw_parts(
                    component.as_ptr() as *const u8,
                    component.len() * std::mem::size_of::<F::SimdCircuitField>(),
                )
            };

            // 每隔4KB(页面大小)读取一个字节，确保触碰所有页面
            for i in (0..bytes.len()).step_by(4096) {
                unsafe {
                    // 使用read_volatile防止编译器优化
                    dummy_sum = dummy_sum.wrapping_add(std::ptr::read_volatile(&bytes[i]) as u64);
                }
            }
        }

        let access_duration = access_start.elapsed();
        println!(
            "[MPI Rank {}] 🔥 Finished accessing witness data (dummy_sum = {}, took {:.3}s)",
            global_mpi_config.world_rank(),
            dummy_sum,
            access_duration.as_secs_f64()
        );

        // 再次测量内存，看是否因为访问而增长
        let (rss_after_access, vmsize_after_access) = get_total_expander_memory_mb();
        let rss_increase_by_access = rss_after_access.saturating_sub(rss_after);
        println!(
            "[MPI Rank {}] 📊 MEMORY_AFTER_ACCESS: RSS = {} MB (+{} MB from copy), VmSize = {} MB",
            global_mpi_config.world_rank(),
            rss_after_access,
            rss_increase_by_access,
            vmsize_after_access
        );

        (witness, SharedMemoryWINWrapper { win: mem_win })
    }

    pub fn write_proof_to_shared_memory<
        C: GKREngine,
        ECCConfig: Config<FieldConfig = C::FieldConfig>,
    >(
        proof: &CombinedProof<ECCConfig, Expander<C>>,
    ) {
        println!("Writing proof to shared memory...");
        Self::write_object_to_shared_memory(proof, unsafe { &mut SHARED_MEMORY.proof }, "proof");
    }

    pub fn read_proof_from_shared_memory<
        C: GKREngine,
        ECCConfig: Config<FieldConfig = C::FieldConfig>,
    >() -> CombinedProof<ECCConfig, Expander<C>>
where {
        Self::read_object_from_shared_memory("proof", 0)
    }
}

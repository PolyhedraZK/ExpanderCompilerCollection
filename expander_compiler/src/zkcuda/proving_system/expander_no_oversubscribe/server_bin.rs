use std::str::FromStr;

use clap::Parser;
use expander_compiler::zkcuda::proving_system::{
    expander::config::{
        ZKCudaBN254Hyrax, ZKCudaBN254HyraxBatchPCS, ZKCudaBN254KZG, ZKCudaBN254KZGBatchPCS,
        ZKCudaBN254MIMCKZG, ZKCudaBN254MIMCKZGBatchPCS,
    },
    expander_parallelized::server_ctrl::{serve, ExpanderExecArgs},
    ExpanderNoOverSubscribe,
};
use gkr_engine::{FiatShamirHashType, PolynomialCommitmentType};

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct ProfilingAllocator {
    threshold: usize,
    allocated: AtomicUsize,
    deallocated: AtomicUsize,
    peak: AtomicUsize,
    allocation_count: AtomicUsize,
}

impl ProfilingAllocator {
    const fn new(threshold: usize) -> Self {
        ProfilingAllocator {
            threshold,
            allocated: AtomicUsize::new(0),
            deallocated: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
            allocation_count: AtomicUsize::new(0),
        }
    }

    fn current_usage(&self) -> usize {
        self.allocated
            .load(Ordering::Relaxed)
            .saturating_sub(self.deallocated.load(Ordering::Relaxed))
    }

    fn print_stats(&self) {
        eprintln!("\n=== Memory Statistics ===");
        eprintln!("Current usage: {} bytes", self.current_usage());
        eprintln!("Peak usage: {} bytes", self.peak.load(Ordering::Relaxed));
        eprintln!(
            "Total allocated: {} bytes",
            self.allocated.load(Ordering::Relaxed)
        );
        eprintln!(
            "Total deallocated: {} bytes",
            self.deallocated.load(Ordering::Relaxed)
        );
        eprintln!(
            "Allocation count: {}",
            self.allocation_count.load(Ordering::Relaxed)
        );
    }
}

unsafe impl GlobalAlloc for ProfilingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let ptr = System.alloc(layout);

        if !ptr.is_null() {
            self.allocated.fetch_add(size, Ordering::Relaxed);
            self.allocation_count.fetch_add(1, Ordering::Relaxed);

            let current = self.current_usage();
            let mut peak = self.peak.load(Ordering::Relaxed);
            while current > peak {
                match self.peak.compare_exchange_weak(
                    peak,
                    current,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(x) => peak = x,
                }
            }

            if size >= self.threshold {
                eprintln!(
                    "[ALLOC] {} bytes | Total: {} bytes",
                    size,
                    self.current_usage()
                );
            }
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        self.deallocated.fetch_add(size, Ordering::Relaxed);

        if size >= self.threshold {
            eprintln!(
                "[DEALLOC] {} bytes | Total: {} bytes",
                size,
                self.current_usage()
            );
        }

        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static ALLOCATOR: ProfilingAllocator = ProfilingAllocator::new(1024 * 1024 * 16); // 16 MB threshold

// Optional: Print stats on program exit
impl Drop for ProfilingAllocator {
    fn drop(&mut self) {
        self.print_stats();
    }
}

async fn async_main() {
    let expander_exec_args = ExpanderExecArgs::parse();

    let pcs_type = PolynomialCommitmentType::from_str(&expander_exec_args.poly_commit).unwrap();

    let fiat_shamir_hash = match expander_exec_args.fiat_shamir_hash.as_str() {
        "SHA256" => FiatShamirHashType::SHA256,
        "MIMC5" => FiatShamirHashType::MIMC5,
        _ => panic!("Unsupported Fiat-Shamir hash function"),
    };

    match (
        expander_exec_args.field_type.as_str(),
        pcs_type,
        fiat_shamir_hash,
    ) {
        ("BN254", PolynomialCommitmentType::Hyrax, FiatShamirHashType::SHA256) => {
            if expander_exec_args.batch_pcs {
                serve::<_, _, ExpanderNoOverSubscribe<ZKCudaBN254HyraxBatchPCS>>(
                    expander_exec_args.port_number,
                )
                .await;
            } else {
                serve::<_, _, ExpanderNoOverSubscribe<ZKCudaBN254Hyrax>>(
                    expander_exec_args.port_number,
                )
                .await;
            }
        }
        ("BN254", PolynomialCommitmentType::KZG, FiatShamirHashType::SHA256) => {
            if expander_exec_args.batch_pcs {
                serve::<_, _, ExpanderNoOverSubscribe<ZKCudaBN254KZGBatchPCS>>(
                    expander_exec_args.port_number,
                )
                .await;
            } else {
                serve::<_, _, ExpanderNoOverSubscribe<ZKCudaBN254KZG>>(
                    expander_exec_args.port_number,
                )
                .await;
            }
        }
        ("BN254", PolynomialCommitmentType::KZG, FiatShamirHashType::MIMC5) => {
            if expander_exec_args.batch_pcs {
                serve::<_, _, ExpanderNoOverSubscribe<ZKCudaBN254MIMCKZGBatchPCS>>(
                    expander_exec_args.port_number,
                )
                .await;
            } else {
                serve::<_, _, ExpanderNoOverSubscribe<ZKCudaBN254MIMCKZG>>(
                    expander_exec_args.port_number,
                )
                .await;
            }
        }
        (field_type, pcs_type, fiat_shamir_hash) => {
            panic!("Combination of {field_type:?}, {pcs_type:?}, and {fiat_shamir_hash:?} not supported for no oversubscribe expander proving system.");
        }
    }

    ALLOCATOR.print_stats();
}

pub fn main() {
    let stack_size_mb = std::env::var("THREAD_STACK_SIZE_MB")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(64);
    let rt = tokio::runtime::Builder::new_multi_thread()
        .thread_stack_size(stack_size_mb * 1024 * 1024) // stack size in MB
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async_main());
}

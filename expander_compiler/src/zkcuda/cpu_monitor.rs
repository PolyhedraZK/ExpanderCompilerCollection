/// CPU使用率监控模块
/// 用于验证commit和PCS opening是否真正用满所有CPU核心
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

pub struct CpuMonitor {
    stop_flag: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl CpuMonitor {
    /// 开始监控CPU使用率
    /// interval_ms: 采样间隔（毫秒）
    pub fn start(tag: &str, interval_ms: u64) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();
        let tag = tag.to_string();

        let handle = thread::spawn(move || {
            let mut prev_stats = get_cpu_stats();

            while !stop_flag_clone.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(interval_ms));

                let curr_stats = get_cpu_stats();
                if let (Some(ref prev), Some(ref curr)) = (prev_stats.as_ref(), curr_stats.as_ref())
                {
                    let usage = calculate_cpu_usage(prev, curr);
                    let num_cpus = num_cpus::get();
                    eprintln!(
                        "[CPU_MONITOR] {} | Total CPUs: {} | Usage: {:.2}% | Active cores estimate: {:.1}",
                        tag, num_cpus, usage, usage / 100.0 * num_cpus as f64
                    );
                }

                prev_stats = curr_stats;
            }
        });

        Self {
            stop_flag,
            handle: Some(handle),
        }
    }

    /// 停止监控并返回
    pub fn stop(mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for CpuMonitor {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[derive(Debug, Clone)]
struct CpuStats {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
}

#[cfg(target_os = "linux")]
fn get_cpu_stats() -> Option<CpuStats> {
    use std::fs;

    let content = fs::read_to_string("/proc/stat").ok()?;
    let line = content.lines().next()?;

    // 第一行格式: cpu user nice system idle iowait irq softirq steal
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 9 || parts[0] != "cpu" {
        return None;
    }

    Some(CpuStats {
        user: parts[1].parse().ok()?,
        nice: parts[2].parse().ok()?,
        system: parts[3].parse().ok()?,
        idle: parts[4].parse().ok()?,
        iowait: parts[5].parse().ok()?,
        irq: parts[6].parse().ok()?,
        softirq: parts[7].parse().ok()?,
        steal: parts[8].parse().ok()?,
    })
}

#[cfg(not(target_os = "linux"))]
fn get_cpu_stats() -> Option<CpuStats> {
    None
}

fn calculate_cpu_usage(prev: &CpuStats, curr: &CpuStats) -> f64 {
    let prev_idle = prev.idle + prev.iowait;
    let curr_idle = curr.idle + curr.iowait;

    let prev_total = prev.user
        + prev.nice
        + prev.system
        + prev.idle
        + prev.iowait
        + prev.irq
        + prev.softirq
        + prev.steal;
    let curr_total = curr.user
        + curr.nice
        + curr.system
        + curr.idle
        + curr.iowait
        + curr.irq
        + curr.softirq
        + curr.steal;

    let total_diff = curr_total.saturating_sub(prev_total);
    let idle_diff = curr_idle.saturating_sub(prev_idle);

    if total_diff == 0 {
        return 0.0;
    }

    (total_diff.saturating_sub(idle_diff) as f64 / total_diff as f64) * 100.0
}

/// 简化版本：单次快照CPU使用情况
pub fn snapshot_cpu_usage(tag: &str) {
    let num_cpus = num_cpus::get();

    #[cfg(target_os = "linux")]
    {
        if let Some(stats) = get_cpu_stats() {
            // 等待一小段时间再采样
            thread::sleep(Duration::from_millis(100));
            if let Some(stats2) = get_cpu_stats() {
                let usage = calculate_cpu_usage(&stats, &stats2);
                eprintln!(
                    "[CPU_SNAPSHOT] {} | Total CPUs: {} | Usage: {:.2}% | Estimated active cores: {:.1}",
                    tag, num_cpus, usage, usage / 100.0 * num_cpus as f64
                );
                return;
            }
        }
    }

    eprintln!(
        "[CPU_SNAPSHOT] {} | Total CPUs: {} | (monitoring not available on this platform)",
        tag, num_cpus
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_monitor() {
        let monitor = CpuMonitor::start("test", 100);

        // 模拟一些CPU密集型工作
        let handles: Vec<_> = (0..4)
            .map(|_| {
                thread::spawn(|| {
                    let mut sum = 0u64;
                    for i in 0..100_000_000 {
                        sum = sum.wrapping_add(i);
                    }
                    sum
                })
            })
            .collect();

        thread::sleep(Duration::from_secs(2));

        for h in handles {
            let _ = h.join();
        }

        monitor.stop();
    }
}

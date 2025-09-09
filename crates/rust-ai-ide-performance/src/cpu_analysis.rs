// CPU analysis module

use super::process_parallel;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CpuCoreMetrics {
    pub core_id: usize,
    pub usage_percent: f64,
    pub frequency_mhz: u64,
    pub temperature_celsius: f64,
}

#[derive(Debug, Clone)]
pub struct CpuMetrics {
    pub overall_usage_percent: f64,
    pub cores: Vec<CpuCoreMetrics>,
    pub load_average: (f64, f64, f64), // 1min, 5min, 15min
    pub context_switches: u64,
    pub interrupts: u64,
    pub processes_created: u64,
}

pub struct CpuAnalyzer {
    previous_metrics: Option<CpuMetrics>,
    _core_history: Vec<CpuCoreMetrics>,
}

impl CpuAnalyzer {
    pub fn new() -> Self {
        Self {
            previous_metrics: None,
            _core_history: Vec::new(),
        }
    }

    pub fn collect_cpu_metrics(&mut self) -> CpuMetrics {
        // Stub implementation - would collect actual system metrics in full implementation
        let cores = (0..num_cpus::get())
            .map(|i| CpuCoreMetrics {
                core_id: i,
                usage_percent: 0.0,
                frequency_mhz: 3000,
                temperature_celsius: 60.0,
            })
            .collect::<Vec<_>>();

        let current_metrics = CpuMetrics {
            overall_usage_percent: cores.iter().map(|c| c.usage_percent).sum::<f64>()
                / cores.len() as f64,
            cores,
            load_average: (1.0, 1.2, 1.1),
            context_switches: 0,
            interrupts: 0,
            processes_created: 0,
        };

        self.previous_metrics = Some(current_metrics.clone());
        current_metrics
    }

    pub fn detect_cpu_spikes(&self, threshold_percent: f64) -> Vec<usize> {
        // Return cores with usage above threshold
        self.previous_metrics
            .as_ref()
            .map(|metrics| {
                metrics
                    .cores
                    .iter()
                    .filter(|core| core.usage_percent > threshold_percent)
                    .map(|core| core.core_id)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_average_load(&self) -> Option<(f64, f64, f64)> {
        self.previous_metrics
            .as_ref()
            .map(|metrics| metrics.load_average)
    }

    pub fn set_custom_metrics(&mut self, metrics: CpuMetrics) {
        self.previous_metrics = Some(metrics);
    }
}

// Thread profiler for analyzing thread performance
#[derive(Debug)]
pub struct ThreadProfiler {
    thread_metrics: HashMap<String, Vec<Duration>>,
    thread_start_times: HashMap<String, Instant>,
}

impl ThreadProfiler {
    pub fn new() -> Self {
        Self {
            thread_metrics: HashMap::new(),
            thread_start_times: HashMap::new(),
        }
    }

    pub fn start_thread_monitoring(&mut self, thread_name: &str) {
        self.thread_start_times
            .insert(thread_name.to_string(), Instant::now());
    }

    pub fn stop_thread_monitoring(&mut self, thread_name: &str) -> Option<Duration> {
        if let Some(start_time) = self.thread_start_times.remove(thread_name) {
            let duration = start_time.elapsed();
            self.thread_metrics
                .entry(thread_name.to_string())
                .or_insert_with(Vec::new)
                .push(duration);
            Some(duration)
        } else {
            None
        }
    }

    pub fn get_thread_stats(&self, thread_name: &str) -> Option<ThreadStats> {
        self.thread_metrics.get(thread_name).map(|durations| {
            if durations.is_empty() {
                return ThreadStats {
                    name: thread_name.to_string(),
                    total_cpu_time: Duration::new(0, 0),
                    avg_cpu_time: Duration::new(0, 0),
                    max_cpu_time: Duration::new(0, 0),
                    min_cpu_time: Duration::new(0, 0),
                    execution_count: 0,
                };
            }

            let total_cpu_time = durations.iter().sum::<Duration>();
            let count = durations.len() as u64;
            let avg_cpu_time = total_cpu_time / count as u32;
            let max_cpu_time = durations.iter().max().unwrap().clone();
            let min_cpu_time = durations.iter().min().unwrap().clone();

            ThreadStats {
                name: thread_name.to_string(),
                total_cpu_time,
                avg_cpu_time,
                max_cpu_time,
                min_cpu_time,
                execution_count: count,
            }
        })
    }

    pub fn get_all_thread_stats(&self) -> Vec<ThreadStats> {
        self.thread_metrics
            .keys()
            .filter_map(|name| self.get_thread_stats(name))
            .collect()
    }

    pub fn clear_thread_stats(&mut self) {
        self.thread_metrics.clear();
        self.thread_start_times.clear();
    }
}

#[derive(Debug, Clone)]
pub struct ThreadStats {
    pub name: String,
    pub total_cpu_time: Duration,
    pub avg_cpu_time: Duration,
    pub max_cpu_time: Duration,
    pub min_cpu_time: Duration,
    pub execution_count: u64,
}

// CPU-intensive computation profiling
pub fn profile_cpu_intensive_task<F, R>(task_name: &str, task: F) -> (Duration, R)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = task();
    let duration = start.elapsed();

    println!("Task '{}' completed in {:?}", task_name, duration);
    (duration, result)
}

// Parallel computation optimization helper
pub fn optimize_parallel_computation<T, F, R>(
    items: Vec<T>,
    compute_fn: F,
    max_threads: Option<usize>,
) -> Vec<R>
where
    T: Send + Sync,
    F: Fn(&T) -> R + Sync + Send,
    R: Send,
{
    let threads = max_threads.unwrap_or(num_cpus::get());
    process_parallel(items, compute_fn).into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cpu_analyzer() {
        let mut analyzer = CpuAnalyzer::new();
        let metrics = analyzer.collect_cpu_metrics();

        assert!(!metrics.cores.is_empty());
        assert!(metrics.overall_usage_percent >= 0.0);
        assert!(metrics.overall_usage_percent <= 100.0);
    }

    #[test]
    fn test_thread_profiler() {
        let mut profiler = ThreadProfiler::new();

        profiler.start_thread_monitoring("test_thread");
        thread::sleep(Duration::from_millis(10));
        let duration = profiler.stop_thread_monitoring("test_thread").unwrap();

        assert!(duration >= Duration::from_millis(10));

        let stats = profiler.get_thread_stats("test_thread").unwrap();
        assert_eq!(stats.name, "test_thread");
        assert_eq!(stats.execution_count, 1);
    }

    #[test]
    fn test_profile_cpu_intensive_task() {
        let (duration, result) = profile_cpu_intensive_task("test_task", || {
            thread::sleep(Duration::from_millis(15));
            42
        });

        assert!(duration >= Duration::from_millis(15));
        assert_eq!(result, 42);
    }
}

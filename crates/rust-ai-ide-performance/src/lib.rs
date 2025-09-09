// Performance monitoring and optimization crate for Rust AI IDE

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};

pub mod caching;
pub mod cpu_analysis;
pub mod collector;
pub mod memory_analysis;
pub mod monitoring;
pub mod profiling;
pub mod regression;
pub mod adaptive_memory;
pub mod alerting;
pub mod gpu_acceleration;
pub mod storage;

pub use caching::*;
pub use cpu_analysis::*;
// Re-export unified PerformanceMetrics from shared-types for backward compatibility
pub use rust_ai_ide_shared_types::PerformanceMetrics as UnifiedPerformanceMetrics;
pub use storage::*;

pub use collector::*;
// Re-export GPU acceleration for easy access
pub use adaptive_memory::*;
pub use gpu_acceleration::*;
pub use memory_analysis::{EnhancedLeakDetector, HeapAnalyzer, HeapGrowthTrend, LeakStatistics, MemoryAnalyzer, AllocationLifetime, LeakCandidate, AutomaticFixSuggestion};
pub use monitoring::*;
pub use profiling::*;
// PerformanceMetrics is now imported from rust_ai_ide_shared_types for consistency
pub use rust_ai_ide_shared_types::PerformanceMetrics;

#[derive(Debug)]
pub struct PerformanceAnalyzer {
    metrics_history: Vec<PerformanceMetrics>,
    start_time: Instant,
}

impl PerformanceAnalyzer {
    /// Create new analyzer with pre-allocated capacity for efficiency
    pub fn new() -> Self {
        Self {
            metrics_history: Vec::with_capacity(1000), // Pre-allocate reasonable capacity
            start_time: Instant::now(),
        }
    }

    /// Create new analyzer with specific capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            metrics_history: Vec::with_capacity(capacity),
            start_time: Instant::now(),
        }
    }

    pub fn record_metric(&mut self, metric: PerformanceMetrics) {
        self.metrics_history.push(metric);
    }

    pub fn get_averages(&self) -> Option<PerformanceMetrics> {
        if self.metrics_history.is_empty() {
            return None;
        }

        let len = self.metrics_history.len() as f64;
        let mut avg = PerformanceMetrics::new();

        let (mut cpu_sum, mut mem_sum, mut disk_sum, mut net_sum, mut resp_sum) = (0.0, 0.0, 0.0, 0.0, 0.0);
        let mut throughput_sum = 0u64;
        let (mut cpu_count, mut mem_count, mut disk_count, mut net_count, mut resp_count, mut throughput_count) =
            (0, 0, 0, 0, 0, 0);

        for metric in &self.metrics_history {
            if let Some(cpu) = metric.cpu_usage_percent {
                cpu_sum += cpu;
                cpu_count += 1;
            }
            if let Some(mem) = metric.memory_usage_mb {
                mem_sum += mem;
                mem_count += 1;
            }
            if let Some(disk) = metric.disk_io_mb_per_sec {
                disk_sum += disk;
                disk_count += 1;
            }
            if let Some(net) = metric.network_io_mb_per_sec {
                net_sum += net;
                net_count += 1;
            }
            if let Some(resp) = metric.response_time_ms {
                resp_sum += resp;
                resp_count += 1;
            }
            if let Some(tp) = metric.throughput_items_per_sec {
                throughput_sum += tp;
                throughput_count += 1;
            }
        }

        avg.cpu_usage_percent = if cpu_count > 0 { Some(cpu_sum / cpu_count as f64) } else { None };
        avg.memory_usage_mb = if mem_count > 0 { Some(mem_sum / mem_count as f64) } else { None };
        avg.disk_io_mb_per_sec = if disk_count > 0 { Some(disk_sum / disk_count as f64) } else { None };
        avg.network_io_mb_per_sec = if net_count > 0 { Some(net_sum / net_count as f64) } else { None };
        avg.response_time_ms = if resp_count > 0 { Some(resp_sum / resp_count as f64) } else { None };
        avg.throughput_items_per_sec = if throughput_count > 0 { Some((throughput_sum as f64 / throughput_count as f64) as u64) } else { None };

        Some(avg)
    }
}

// Benchmark utilities
pub async fn benchmark_function<F, Fut, T>(f: F, iterations: u32) -> Duration
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let mut total_duration = Duration::new(0, 0);

    for _ in 0..iterations {
        let start = Instant::now();
        f().await;
        total_duration += start.elapsed();
    }

    total_duration / iterations
}

// Parallel processing helper with proper collection bounds and memory safety
pub fn process_parallel<T, F, R>(items: Vec<T>, processor: F) -> Vec<R>
where
    T: Send + Sync,
    F: Fn(&T) -> R + Send + Sync,
    R: Send,
{
    // Pre-allocate result vector with exact capacity for memory efficiency
    let capacity = items.len();
    let mut results: Vec<R> = Vec::with_capacity(capacity);

    // Add safety check for empty input
    if items.is_empty() {
        return results;
    }

    // Process with proper chunking for large datasets
    let chunk_size = if capacity > 1000 {
        (capacity as f64 / num_cpus::get().max(1) as f64).ceil() as usize
    } else {
        1
    }
    .max(1);

    // Collect results into pre-allocated vector for memory safety
    results = items.par_iter().map(processor).collect();

    // Ensure result length matches input length for consistency
    debug_assert_eq!(
        results.len(),
        items.len(),
        "Result length should match input length"
    );

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_analyzer() {
        let mut analyzer = PerformanceAnalyzer::new();

        let mut metric = PerformanceMetrics::new();
        metric.cpu_usage_percent = Some(50.0);
        metric.memory_usage_mb = Some(100.0);
        metric.disk_io_mb_per_sec = Some(10.0);
        metric.network_io_mb_per_sec = Some(5.0);
        metric.response_time_ms = Some(100.0);
        metric.throughput_items_per_sec = Some(1000);

        analyzer.record_metric(metric);

        let avg = analyzer.get_averages().unwrap();
        assert_eq!(avg.cpu_usage_percent, Some(50.0));
        assert_eq!(avg.memory_usage_mb, Some(100.0));
    }

    #[tokio::test]
    async fn test_benchmark_function() {
        let duration = benchmark_function(
            || async {
                tokio::time::sleep(Duration::from_millis(10)).await;
            },
            3,
        )
        .await;

        assert!(duration >= Duration::from_millis(10));
    }
}

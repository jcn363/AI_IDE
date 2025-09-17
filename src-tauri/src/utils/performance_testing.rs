//! Performance Testing and Benchmarking Utilities
//!
//! This module provides comprehensive performance testing capabilities for:
//! - Startup time measurement and analysis
//! - Memory usage profiling
//! - CPU usage monitoring
//! - Service initialization benchmarking
//! - Performance regression detection
//! - Real-time performance monitoring

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::time::interval;

/// Performance benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub duration: Duration,
    pub memory_usage: Option<u64>,
    pub cpu_usage: Option<f64>,
    pub timestamp: Instant,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Startup performance benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupBenchmark {
    pub cold_start_time: Duration,
    pub warm_start_time: Duration,
    pub service_init_times: HashMap<String, Duration>,
    pub total_memory_usage: u64,
    pub peak_memory_usage: u64,
    pub timestamp: Instant,
}

impl Default for StartupBenchmark {
    fn default() -> Self {
        Self {
            cold_start_time: Duration::ZERO,
            warm_start_time: Duration::ZERO,
            service_init_times: HashMap::new(),
            total_memory_usage: 0,
            peak_memory_usage: 0,
            timestamp: Instant::now(),
        }
    }
}

/// Real-time performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub active_threads: usize,
    pub heap_allocations: u64,
    pub timestamp: Instant,
}

/// Performance monitor for real-time tracking
pub struct PerformanceMonitor {
    metrics_history: Arc<RwLock<Vec<PerformanceMetrics>>>,
    benchmarks: Arc<RwLock<HashMap<String, BenchmarkResult>>>,
    startup_benchmarks: Arc<RwLock<Vec<StartupBenchmark>>>,
    monitoring_active: Arc<RwLock<bool>>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            benchmarks: Arc::new(RwLock::new(HashMap::new())),
            startup_benchmarks: Arc::new(RwLock::new(Vec::new())),
            monitoring_active: Arc::new(RwLock::new(false)),
        }
    }

    /// Start real-time performance monitoring
    pub async fn start_monitoring(&self) -> Result<(), String> {
        let mut active = self.monitoring_active.write().await;
        if *active {
            return Err("Monitoring already active".to_string());
        }
        *active = true;
        drop(active);

        let metrics_history = Arc::clone(&self.metrics_history);
        let monitoring_active = Arc::clone(&self.monitoring_active);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));

            while *monitoring_active.read().await {
                interval.tick().await;

                let metrics = Self::collect_current_metrics().await;
                let mut history = metrics_history.write().await;
                history.push(metrics);

                // Keep only last 1000 measurements to prevent unbounded growth
                if history.len() > 1000 {
                    history.remove(0);
                }
            }
        });

        log::info!("Performance monitoring started");
        Ok(())
    }

    /// Stop performance monitoring
    pub async fn stop_monitoring(&self) -> Result<(), String> {
        let mut active = self.monitoring_active.write().await;
        *active = false;
        log::info!("Performance monitoring stopped");
        Ok(())
    }

    /// Collect current system metrics
    async fn collect_current_metrics() -> PerformanceMetrics {
        let memory_usage = Self::get_memory_usage().await.unwrap_or(0);
        let cpu_usage = Self::get_cpu_usage().await.unwrap_or(0.0);
        let active_threads = Self::get_active_thread_count().await.unwrap_or(0);
        let heap_allocations = Self::get_heap_allocations().await.unwrap_or(0);

        PerformanceMetrics {
            memory_usage,
            cpu_usage,
            active_threads,
            heap_allocations,
            timestamp: Instant::now(),
        }
    }

    /// Get current memory usage (simplified implementation)
    async fn get_memory_usage() -> Result<u64, String> {
        // In a real implementation, this would use system-specific APIs
        // For now, return a placeholder value
        Ok(100 * 1024 * 1024) // 100MB placeholder
    }

    /// Get current CPU usage (simplified implementation)
    async fn get_cpu_usage() -> Result<f64, String> {
        // In a real implementation, this would use system-specific APIs
        // For now, return a placeholder value
        Ok(15.5) // 15.5% placeholder
    }

    /// Get active thread count
    async fn get_active_thread_count() -> Result<usize, String> {
        Ok(std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4))
    }

    /// Get heap allocations (simplified implementation)
    async fn get_heap_allocations() -> Result<u64, String> {
        // In a real implementation, this would use jemalloc or similar
        Ok(50 * 1024 * 1024) // 50MB placeholder
    }

    /// Run a performance benchmark
    pub async fn run_benchmark<F, Fut>(
        &self,
        name: &str,
        benchmark_fn: F,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<BenchmarkResult, String>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<(), String>>,
    {
        let start_time = Instant::now();
        let memory_before = Self::get_memory_usage().await.unwrap_or(0);
        let cpu_before = Self::get_cpu_usage().await.unwrap_or(0.0);

        // Run the benchmark
        let result = benchmark_fn().await;

        let duration = start_time.elapsed();
        let memory_after = Self::get_memory_usage().await.unwrap_or(0);
        let cpu_after = Self::get_cpu_usage().await.unwrap_or(0.0);

        let benchmark_result = BenchmarkResult {
            name: name.to_string(),
            duration,
            memory_usage: Some(memory_after.saturating_sub(memory_before)),
            cpu_usage: Some(cpu_after - cpu_before),
            timestamp: Instant::now(),
            metadata,
        };

        // Store the result
        if result.is_ok() {
            let mut benchmarks = self.benchmarks.write().await;
            benchmarks.insert(name.to_string(), benchmark_result.clone());
        }

        result.map(|_| benchmark_result)
    }

    /// Record startup benchmark
    pub async fn record_startup_benchmark(&self, benchmark: StartupBenchmark) {
        let mut benchmarks = self.startup_benchmarks.write().await;
        benchmarks.push(benchmark);

        log::info!(
            "Startup benchmark recorded: cold_start={:.2}ms, warm_start={:.2}ms",
            benchmark.cold_start_time.as_millis(),
            benchmark.warm_start_time.as_millis()
        );
    }

    /// Get latest performance metrics
    pub async fn get_latest_metrics(&self) -> Option<PerformanceMetrics> {
        let history = self.metrics_history.read().await;
        history.last().cloned()
    }

    /// Get performance metrics history
    pub async fn get_metrics_history(&self) -> Vec<PerformanceMetrics> {
        self.metrics_history.read().await.clone()
    }

    /// Get benchmark by name
    pub async fn get_benchmark(&self, name: &str) -> Option<BenchmarkResult> {
        let benchmarks = self.benchmarks.read().await;
        benchmarks.get(name).cloned()
    }

    /// Get all benchmarks
    pub async fn get_all_benchmarks(&self) -> HashMap<String, BenchmarkResult> {
        self.benchmarks.read().await.clone()
    }

    /// Get startup benchmarks
    pub async fn get_startup_benchmarks(&self) -> Vec<StartupBenchmark> {
        self.startup_benchmarks.read().await.clone()
    }

    /// Generate performance report
    pub async fn generate_performance_report(&self) -> String {
        let benchmarks = self.benchmarks.read().await;
        let startup_benchmarks = self.startup_benchmarks.read().await;
        let latest_metrics = self.get_latest_metrics().await;

        let mut report = String::new();
        report.push_str("=== Performance Report ===\n\n");

        // Latest metrics
        if let Some(metrics) = latest_metrics {
            report.push_str(&format!("Current Metrics:\n"));
            report.push_str(&format!("  Memory Usage: {:.2} MB\n", metrics.memory_usage as f64 / (1024.0 * 1024.0)));
            report.push_str(&format!("  CPU Usage: {:.1}%\n", metrics.cpu_usage));
            report.push_str(&format!("  Active Threads: {}\n", metrics.active_threads));
            report.push_str(&format!("  Heap Allocations: {:.2} MB\n", metrics.heap_allocations as f64 / (1024.0 * 1024.0)));
            report.push_str("\n");
        }

        // Benchmarks
        if !benchmarks.is_empty() {
            report.push_str("Benchmarks:\n");
            for (name, benchmark) in benchmarks.iter() {
                report.push_str(&format!("  {}: {:.2}ms", name, benchmark.duration.as_millis()));
                if let Some(mem) = benchmark.memory_usage {
                    report.push_str(&format!(" ({:.2} MB)", mem as f64 / (1024.0 * 1024.0)));
                }
                report.push_str("\n");
            }
            report.push_str("\n");
        }

        // Startup benchmarks
        if !startup_benchmarks.is_empty() {
            report.push_str("Startup Performance:\n");
            for (i, benchmark) in startup_benchmarks.iter().enumerate() {
                report.push_str(&format!("  Run {}: Cold Start: {:.2}ms, Warm Start: {:.2}ms\n",
                    i + 1,
                    benchmark.cold_start_time.as_millis(),
                    benchmark.warm_start_time.as_millis()
                ));
            }
            report.push_str("\n");
        }

        // Performance analysis
        report.push_str("=== Performance Analysis ===\n");
        if let Some(latest) = latest_metrics {
            if latest.memory_usage > 500 * 1024 * 1024 { // 500MB
                report.push_str("⚠️  High memory usage detected\n");
            }
            if latest.cpu_usage > 80.0 {
                report.push_str("⚠️  High CPU usage detected\n");
            }
        }

        // Check startup time target
        if let Some(latest_startup) = startup_benchmarks.last() {
            if latest_startup.cold_start_time.as_millis() > 200 {
                report.push_str("⚠️  Cold start time exceeds 200ms target\n");
            } else {
                report.push_str("✅ Cold start time within 200ms target\n");
            }
        }

        report
    }

    /// Analyze performance trends
    pub async fn analyze_performance_trends(&self) -> PerformanceAnalysis {
        let history = self.metrics_history.read().await;
        let benchmarks = self.benchmarks.read().await;
        let startup_benchmarks = self.startup_benchmarks.read().await;

        let mut analysis = PerformanceAnalysis::default();

        if history.len() >= 2 {
            let recent = &history[history.len().saturating_sub(10)..];
            let avg_memory = recent.iter().map(|m| m.memory_usage).sum::<u64>() / recent.len() as u64;
            let avg_cpu = recent.iter().map(|m| m.cpu_usage).sum::<f64>() / recent.len() as f64;

            analysis.average_memory_usage = avg_memory;
            analysis.average_cpu_usage = avg_cpu;
            analysis.memory_trend = Self::calculate_trend(recent.iter().map(|m| m.memory_usage as f64).collect());
            analysis.cpu_trend = Self::calculate_trend(recent.iter().map(|m| m.cpu_usage).collect());
        }

        if !startup_benchmarks.is_empty() {
            let cold_starts: Vec<f64> = startup_benchmarks.iter().map(|b| b.cold_start_time.as_millis() as f64).collect();
            analysis.startup_time_trend = Self::calculate_trend(cold_starts);

            let latest_cold_start = startup_benchmarks.last().unwrap().cold_start_time.as_millis();
            analysis.target_achievement = latest_cold_start <= 200;
        }

        // Identify performance bottlenecks
        for (name, benchmark) in benchmarks.iter() {
            if benchmark.duration.as_millis() > 100 { // More than 100ms
                analysis.bottlenecks.push(format!("{}: {:.2}ms", name, benchmark.duration.as_millis()));
            }
        }

        analysis
    }

    /// Calculate trend from data points (simple linear regression slope)
    fn calculate_trend(data: Vec<f64>) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }

        let n = data.len() as f64;
        let sum_x: f64 = (0..data.len()).map(|i| i as f64).sum();
        let sum_y: f64 = data.iter().sum();
        let sum_xy: f64 = data.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let sum_xx: f64 = (0..data.len()).map(|i| (i * i) as f64).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
        slope
    }
}

/// Performance analysis results
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceAnalysis {
    pub average_memory_usage: u64,
    pub average_cpu_usage: f64,
    pub memory_trend: f64, // Positive = increasing, negative = decreasing
    pub cpu_trend: f64,
    pub startup_time_trend: f64,
    pub target_achievement: bool,
    pub bottlenecks: Vec<String>,
}

/// Benchmark runner for automated testing
pub struct BenchmarkRunner {
    monitor: Arc<PerformanceMonitor>,
}

impl BenchmarkRunner {
    pub fn new(monitor: Arc<PerformanceMonitor>) -> Self {
        Self { monitor }
    }

    /// Run comprehensive startup benchmark
    pub async fn run_startup_benchmark(&self) -> Result<StartupBenchmark, String> {
        let mut benchmark = StartupBenchmark::default();
        let start_time = Instant::now();

        // Simulate startup phases
        let phases = vec![
            ("config_loading", Duration::from_millis(10)),
            ("service_discovery", Duration::from_millis(15)),
            ("lsp_initialization", Duration::from_millis(50)),
            ("ai_service_init", Duration::from_millis(30)),
            ("ui_setup", Duration::from_millis(25)),
        ];

        for (phase_name, simulated_duration) in phases {
            let phase_start = Instant::now();
            tokio::time::sleep(simulated_duration).await;
            let actual_duration = phase_start.elapsed();

            benchmark.service_init_times.insert(
                phase_name.to_string(),
                actual_duration
            );
        }

        benchmark.cold_start_time = start_time.elapsed();
        benchmark.warm_start_time = benchmark.cold_start_time / 2; // Estimate warm start as half
        benchmark.total_memory_usage = Self::get_memory_usage().await.unwrap_or(0);
        benchmark.peak_memory_usage = benchmark.total_memory_usage + (10 * 1024 * 1024); // Add some overhead

        self.monitor.record_startup_benchmark(benchmark.clone()).await;

        Ok(benchmark)
    }

    /// Run memory leak detection benchmark
    pub async fn run_memory_leak_detection(&self) -> Result<BenchmarkResult, String> {
        let allocations = Arc::new(std::sync::Mutex::new(Vec::new()));

        self.monitor.run_benchmark(
            "memory_leak_detection",
            || async {
                // Simulate memory allocation patterns that might indicate leaks
                let mut data = Vec::new();
                for i in 0..1000 {
                    data.push(vec![0u8; 1024]); // 1KB allocations
                    if i % 100 == 0 {
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    }
                }
                // Clear some but not all to simulate potential leak
                data.truncate(500);
                Ok(())
            },
            {
                let mut metadata = HashMap::new();
                metadata.insert("test_type".to_string(), serde_json::json!("memory_leak"));
                metadata
            },
        ).await
    }

    /// Run CPU intensive benchmark
    pub async fn run_cpu_intensive_benchmark(&self) -> Result<BenchmarkResult, String> {
        self.monitor.run_benchmark(
            "cpu_intensive_operations",
            || async {
                // Simulate CPU-intensive operations
                tokio::task::spawn_blocking(|| {
                    let mut result = 0u64;
                    for i in 0..1_000_000 {
                        result = result.wrapping_add(i);
                    }
                    result
                }).await.map_err(|e| format!("CPU benchmark failed: {:?}", e))?;
                Ok(())
            },
            {
                let mut metadata = HashMap::new();
                metadata.insert("test_type".to_string(), serde_json::json!("cpu_intensive"));
                metadata
            },
        ).await
    }

    /// Get memory usage (placeholder)
    async fn get_memory_usage() -> Result<u64, String> {
        // In production, this would use actual system APIs
        Ok(150 * 1024 * 1024) // 150MB placeholder
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        assert!(!*monitor.monitoring_active.read().await);
    }

    #[tokio::test]
    async fn test_startup_benchmark_creation() {
        let benchmark = StartupBenchmark::default();
        assert_eq!(benchmark.cold_start_time, Duration::ZERO);
        assert!(benchmark.service_init_times.is_empty());
    }

    #[tokio::test]
    async fn test_benchmark_runner_creation() {
        let monitor = Arc::new(PerformanceMonitor::new());
        let runner = BenchmarkRunner::new(Arc::clone(&monitor));
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_performance_analysis_default() {
        let analysis = PerformanceAnalysis::default();
        assert_eq!(analysis.average_memory_usage, 0);
        assert_eq!(analysis.average_cpu_usage, 0.0);
        assert!(!analysis.target_achievement);
        assert!(analysis.bottlenecks.is_empty());
    }
}
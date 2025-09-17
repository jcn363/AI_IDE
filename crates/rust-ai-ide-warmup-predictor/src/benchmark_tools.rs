//! Performance Benchmarking Tools
//!
//! This module provides comprehensive performance benchmarking capabilities for the warmup prediction system,
//! including latency measurement, throughput analysis, memory profiling, accuracy validation, load testing,
//! statistical analysis, and automated performance monitoring.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use sysinfo::{System, SystemExt, ProcessExt, CpuExt, MemoryExt};
use std::sync::atomic::{AtomicU64, Ordering};
use rand::prelude::*;
use tokio::sync::Semaphore;
use std::fs;
use chrono::{DateTime, Utc};

use crate::error::{Result, WarmupError};
use crate::types::WarmupRequest;
use crate::ml_trainer::{MLModelTrainer, TrainingConfig, MLModelType};
use crate::ml_evaluator::{MLModelEvaluator, EvaluationConfig};

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Number of iterations for each benchmark
    pub iterations: usize,
    /// Warmup iterations before measurement
    pub warmup_iterations: usize,
    /// Maximum benchmark duration
    pub max_duration: Duration,
    /// Enable memory profiling
    pub memory_profiling: bool,
    /// Enable CPU profiling
    pub cpu_profiling: bool,
    /// Enable detailed latency analysis
    pub detailed_latency: bool,
    /// Statistical confidence level
    pub confidence_level: f64,
    /// Concurrent requests for load testing
    pub concurrent_requests: usize,
}

/// Benchmark result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Benchmark name
    pub name: String,
    /// Total duration
    pub total_duration: Duration,
    /// Average latency
    pub avg_latency: Duration,
    /// Median latency
    pub median_latency: Duration,
    /// 95th percentile latency
    pub p95_latency: Duration,
    /// 99th percentile latency
    pub p99_latency: Duration,
    /// Minimum latency
    pub min_latency: Duration,
    /// Maximum latency
    pub max_latency: Duration,
    /// Throughput (requests per second)
    pub throughput: f64,
    /// Memory usage (MB)
    pub memory_usage_mb: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Error rate
    pub error_rate: f64,
    /// Latency distribution
    pub latency_distribution: Vec<Duration>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Comparative benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparativeBenchmark {
    /// Baseline benchmark result
    pub baseline: BenchmarkResult,
    /// Current benchmark result
    pub current: BenchmarkResult,
    /// Performance improvement/degradation
    pub improvement_percent: f64,
    /// Statistical significance
    pub statistically_significant: bool,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Performance metrics with enhanced monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// CPU usage over time
    pub cpu_usage_over_time: Vec<f64>,
    /// Memory usage over time
    pub memory_usage_over_time: Vec<f64>,
    /// Latency over time
    pub latency_over_time: Vec<Duration>,
    /// Throughput over time
    pub throughput_over_time: Vec<f64>,
    /// System load average
    pub system_load: f64,
    /// Available memory
    pub available_memory_mb: f64,
    /// Total memory
    pub total_memory_mb: f64,
    /// Network I/O metrics
    pub network_io_bytes: u64,
    /// Disk I/O metrics
    pub disk_io_bytes: u64,
    /// Garbage collection metrics
    pub gc_cycles: u64,
    /// Heap allocations
    pub heap_allocations: u64,
}

/// Memory profiling data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryProfile {
    /// Current heap size
    pub heap_size_bytes: u64,
    /// Peak heap size during test
    pub peak_heap_size_bytes: u64,
    /// Total allocations
    pub total_allocations: u64,
    /// Total deallocations
    pub total_deallocations: u64,
    /// Memory leak detection
    pub potential_leaks: Vec<String>,
    /// Fragmentation ratio
    pub fragmentation_ratio: f64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Latency analysis with percentiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAnalysis {
    /// P50 latency (median)
    pub p50: Duration,
    /// P90 latency
    pub p90: Duration,
    /// P95 latency
    pub p95: Duration,
    /// P99 latency
    pub p99: Duration,
    /// P99.9 latency
    pub p999: Duration,
    /// Mean latency
    pub mean: Duration,
    /// Standard deviation
    pub std_dev: Duration,
    /// Min latency
    pub min: Duration,
    /// Max latency
    pub max: Duration,
    /// Jitter (variance in latency)
    pub jitter: Duration,
}

/// Accuracy validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyValidation {
    /// Overall prediction accuracy
    pub overall_accuracy: f64,
    /// Precision score
    pub precision: f64,
    /// Recall score
    pub recall: f64,
    /// F1 score
    pub f1_score: f64,
    /// True positives
    pub true_positives: u64,
    /// True negatives
    pub true_negatives: u64,
    /// False positives
    pub false_positives: u64,
    /// False negatives
    pub false_negatives: u64,
    /// Statistical significance (p-value)
    pub statistical_significance: f64,
    /// Confidence interval
    pub confidence_interval: (f64, f64),
}

/// Statistical analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalAnalysis {
    /// Mean value
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Variance
    pub variance: f64,
    /// Skewness
    pub skewness: f64,
    /// Kurtosis
    pub kurtosis: f64,
    /// Confidence interval (95%)
    pub confidence_interval_95: (f64, f64),
    /// Sample size
    pub sample_size: usize,
    /// Outlier count
    pub outlier_count: usize,
}

/// Load testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestConfig {
    /// Target throughput (requests per second)
    pub target_throughput: f64,
    /// Test duration
    pub duration: Duration,
    /// Ramp-up time
    pub ramp_up_time: Duration,
    /// Maximum concurrent users
    pub max_concurrent_users: usize,
    /// Request patterns
    pub request_patterns: Vec<RequestPattern>,
}

/// Request pattern for load testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestPattern {
    /// Pattern weight (probability)
    pub weight: f64,
    /// Request template
    pub request_template: WarmupRequest,
}

/// Performance benchmarker with enhanced capabilities
#[derive(Debug)]
pub struct PerformanceBenchmarker {
    /// Benchmark configuration
    config: BenchmarkConfig,
    /// System information
    system: Arc<RwLock<System>>,
    /// Historical benchmark results
    history: Arc<RwLock<Vec<BenchmarkResult>>>,
    /// Performance metrics cache
    metrics_cache: Arc<RwLock<HashMap<String, PerformanceMetrics>>>,
    /// Baseline results for regression detection
    baselines: Arc<RwLock<HashMap<String, BenchmarkResult>>>,
    /// Statistical analyzer
    statistical_analyzer: Arc<RwLock<StatisticalAnalyzer>>,
}

/// Comprehensive benchmark suite for all 7 core components
#[derive(Debug)]
pub struct ComprehensiveBenchmarkSuite {
    /// Performance benchmarker
    benchmarker: PerformanceBenchmarker,
    /// Memory profiler
    memory_profiler: Arc<RwLock<MemoryProfiler>>,
    /// Load tester
    load_tester: Arc<RwLock<LoadTester>>,
    /// Accuracy validator
    accuracy_validator: Arc<RwLock<AccuracyValidator>>,
    /// Stress tester
    stress_tester: Arc<RwLock<StressTester>>,
    /// Automated monitor
    automated_monitor: Arc<RwLock<AutomatedMonitor>>,
}

/// Memory profiler implementation
#[derive(Debug)]
pub struct MemoryProfiler {
    /// Current memory snapshots
    snapshots: Vec<MemoryProfile>,
    /// Leak detection state
    leak_detector: HashMap<String, u64>,
}

/// Load testing framework
#[derive(Debug)]
pub struct LoadTester {
    /// Active test scenarios
    scenarios: HashMap<String, LoadTestScenario>,
    /// Concurrent request semaphore
    concurrency_limiter: Arc<Semaphore>,
}

/// Accuracy validation framework
#[derive(Debug)]
pub struct AccuracyValidator {
    /// Ground truth dataset
    ground_truth: Vec<AccuracyTestCase>,
    /// Statistical significance threshold
    significance_threshold: f64,
}

/// Stress testing framework
#[derive(Debug)]
pub struct StressTester {
    /// Saturation points tracking
    saturation_points: Vec<SaturationPoint>,
    /// System limits
    system_limits: SystemLimits,
}

/// Automated performance monitor
#[derive(Debug)]
pub struct AutomatedMonitor {
    /// Alert thresholds
    alert_thresholds: HashMap<String, f64>,
    /// Historical performance data
    historical_data: Vec<PerformanceSnapshot>,
    /// Regression detector
    regression_detector: RegressionDetector,
}

/// Statistical analyzer
#[derive(Debug)]
pub struct StatisticalAnalyzer {
    /// Sample data storage
    samples: HashMap<String, Vec<f64>>,
    /// Analysis cache
    analysis_cache: HashMap<String, StatisticalAnalysis>,
}

/// Test case for accuracy validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyTestCase {
    /// Input request
    pub input: WarmupRequest,
    /// Expected prediction result
    pub expected_result: Option<ModelId>,
    /// Expected confidence threshold
    pub expected_confidence: f64,
}

/// Load test scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestScenario {
    /// Scenario name
    pub name: String,
    /// Request patterns
    pub patterns: Vec<RequestPattern>,
    /// Target throughput
    pub target_throughput: f64,
    /// Duration
    pub duration: Duration,
}

/// Saturation point in stress testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaturationPoint {
    /// Load level
    pub load_level: f64,
    /// Response time at saturation
    pub response_time: Duration,
    /// Error rate at saturation
    pub error_rate: f64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// System resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLimits {
    /// Max CPU usage
    pub max_cpu_percent: f64,
    /// Max memory usage (MB)
    pub max_memory_mb: u64,
    /// Max network bandwidth (Mbps)
    pub max_network_mbps: f64,
    /// Max concurrent requests
    pub max_concurrent_requests: usize,
}

/// Performance snapshot for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,
    /// System metrics
    pub system_metrics: SystemMetrics,
    /// Application metrics
    pub application_metrics: ApplicationMetrics,
}

/// System metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Memory usage MB
    pub memory_mb: f64,
    /// Disk I/O bytes
    pub disk_io_bytes: u64,
    /// Network I/O bytes
    pub network_io_bytes: u64,
}

/// Application metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationMetrics {
    /// Active connections
    pub active_connections: u64,
    /// Request rate
    pub request_rate: f64,
    /// Error rate
    pub error_rate: f64,
    /// Average latency
    pub avg_latency_ms: f64,
}

/// Regression detector
#[derive(Debug)]
pub struct RegressionDetector {
    /// Baseline performance metrics
    baselines: HashMap<String, PerformanceSnapshot>,
    /// Change detection thresholds
    thresholds: HashMap<String, f64>,
}

impl PerformanceBenchmarker {
    /// Create a new performance benchmarker
    pub fn new(config: BenchmarkConfig) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            config,
            system: Arc::new(RwLock::new(system)),
            history: Arc::new(RwLock::new(Vec::new())),
            metrics_cache: Arc::new(RwLock::new(HashMap::new())),
            baselines: Arc::new(RwLock::new(HashMap::new())),
            statistical_analyzer: Arc::new(RwLock::new(StatisticalAnalyzer::new())),
        }
    }

    /// Create comprehensive benchmark suite
    pub fn create_comprehensive_suite(config: BenchmarkConfig) -> Result<ComprehensiveBenchmarkSuite> {
        let benchmarker = Self::new(config);

        Ok(ComprehensiveBenchmarkSuite {
            benchmarker,
            memory_profiler: Arc::new(RwLock::new(MemoryProfiler::new())),
            load_tester: Arc::new(RwLock::new(LoadTester::new())),
            accuracy_validator: Arc::new(RwLock::new(AccuracyValidator::new())),
            stress_tester: Arc::new(RwLock::new(StressTester::new())),
            automated_monitor: Arc::new(RwLock::new(AutomatedMonitor::new())),
        })
    }

    /// Run comprehensive benchmark on warmup predictor
    pub async fn benchmark_warmup_predictor<F, Fut>(
        &self,
        benchmark_name: &str,
        predictor_function: F,
    ) -> Result<BenchmarkResult>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        let mut latencies = Vec::new();
        let mut errors = 0;
        let start_time = Instant::now();

        // Warmup phase
        for _ in 0..self.config.warmup_iterations {
            if let Err(_) = predictor_function().await {
                errors += 1;
            }
        }

        // Measurement phase
        for _ in 0..self.config.iterations {
            let iteration_start = Instant::now();

            let result = predictor_function().await;
            let iteration_duration = iteration_start.elapsed();

            latencies.push(iteration_duration);

            if result.is_err() {
                errors += 1;
            }

            // Check timeout
            if start_time.elapsed() > self.config.max_duration {
                break;
            }
        }

        let total_duration = start_time.elapsed();

        // Calculate statistics
        let (avg_latency, median_latency, p95_latency, p99_latency, min_latency, max_latency) =
            self.calculate_latency_stats(&latencies)?;

        let throughput = self.config.iterations as f64 / total_duration.as_secs_f64();
        let error_rate = errors as f64 / (self.config.iterations + self.config.warmup_iterations) as f64;

        // System metrics
        let (memory_usage, cpu_usage) = self.measure_system_resources().await?;

        let result = BenchmarkResult {
            name: benchmark_name.to_string(),
            total_duration,
            avg_latency,
            median_latency,
            p95_latency,
            p99_latency,
            min_latency,
            max_latency,
            throughput,
            memory_usage_mb: memory_usage,
            cpu_usage_percent: cpu_usage,
            error_rate,
            latency_distribution: latencies,
            timestamp: chrono::Utc::now(),
        };

        // Store in history
        let mut history = self.history.write().await;
        history.push(result.clone());

        Ok(result)
    }

    /// Run load test with multiple concurrent requests
    pub async fn run_load_test<F, Fut>(
        &self,
        test_name: &str,
        load_config: &LoadTestConfig,
        request_function: F,
    ) -> Result<BenchmarkResult>
    where
        F: Fn(WarmupRequest) -> Fut + Send + Sync + Clone,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        let start_time = Instant::now();
        let mut handles = Vec::new();
        let mut latencies = Vec::new();
        let mut errors = 0;

        // Create request pattern weights
        let total_weight: f64 = load_config.request_patterns.iter().map(|p| p.weight).sum();
        let mut cumulative_weights = Vec::new();
        let mut cumulative = 0.0;

        for pattern in &load_config.request_patterns {
            cumulative += pattern.weight / total_weight;
            cumulative_weights.push(cumulative);
        }

        // Generate requests for the test duration
        let test_end = start_time + load_config.duration;
        let mut request_count = 0;

        while Instant::now() < test_end {
            let current_concurrent = handles.len();

            // Ramp up users
            if current_concurrent < load_config.max_concurrent_users {
                let ramp_progress = start_time.elapsed().as_secs_f64() / load_config.ramp_up_time.as_secs_f64();
                let target_concurrent = (load_config.max_concurrent_users as f64 * ramp_progress.min(1.0)) as usize;

                while handles.len() < target_concurrent {
                    let request_pattern = self.select_request_pattern(&load_config.request_patterns, &cumulative_weights);
                    let request_function_clone = request_function.clone();

                    let handle = tokio::spawn(async move {
                        let request_start = Instant::now();
                        let result = request_function_clone(request_pattern).await;
                        let latency = request_start.elapsed();
                        (result, latency)
                    });

                    handles.push(handle);
                }
            }

            // Clean up completed requests
            let mut completed_indices = Vec::new();
            for (i, handle) in handles.iter_mut().enumerate() {
                if handle.is_finished() {
                    let (result, latency) = handle.await.unwrap();
                    latencies.push(latency);
                    request_count += 1;

                    if result.is_err() {
                        errors += 1;
                    }

                    completed_indices.push(i);
                }
            }

            // Remove completed handles (in reverse order to maintain indices)
            for i in completed_indices.into_iter().rev() {
                handles.remove(i);
            }

            // Small delay to prevent busy waiting
            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        // Wait for remaining requests
        for handle in handles {
            let (result, latency) = handle.await.unwrap();
            latencies.push(latency);
            request_count += 1;

            if result.is_err() {
                errors += 1;
            }
        }

        let total_duration = start_time.elapsed();
        let (avg_latency, median_latency, p95_latency, p99_latency, min_latency, max_latency) =
            self.calculate_latency_stats(&latencies)?;

        let throughput = request_count as f64 / total_duration.as_secs_f64();
        let error_rate = errors as f64 / request_count as f64;

        // System metrics
        let (memory_usage, cpu_usage) = self.measure_system_resources().await?;

        let result = BenchmarkResult {
            name: test_name.to_string(),
            total_duration,
            avg_latency,
            median_latency,
            p95_latency,
            p99_latency,
            min_latency,
            max_latency,
            throughput,
            memory_usage_mb: memory_usage,
            cpu_usage_percent: cpu_usage,
            error_rate,
            latency_distribution: latencies,
            timestamp: chrono::Utc::now(),
        };

        Ok(result)
    }

    /// Compare two benchmark results
    pub fn compare_benchmarks(&self, baseline: &BenchmarkResult, current: &BenchmarkResult) -> ComparativeBenchmark {
        let improvement_percent = ((current.throughput - baseline.throughput) / baseline.throughput) * 100.0;

        // Simple statistical significance test (t-test approximation)
        let n1 = baseline.latency_distribution.len() as f64;
        let n2 = current.latency_distribution.len() as f64;

        let mean1 = baseline.avg_latency.as_secs_f64() * 1000.0; // Convert to ms
        let mean2 = current.avg_latency.as_secs_f64() * 1000.0;
        let std1 = self.calculate_std_dev(&baseline.latency_distribution.iter().map(|d| d.as_secs_f64() * 1000.0).collect::<Vec<_>>());
        let std2 = self.calculate_std_dev(&current.latency_distribution.iter().map(|d| d.as_secs_f64() * 1000.0).collect::<Vec<_>>());

        let se = ((std1 * std1 / n1) + (std2 * std2 / n2)).sqrt();
        let t_stat = (mean2 - mean1) / se;
        let statistically_significant = t_stat.abs() > 1.96; // 95% confidence

        let mut recommendations = Vec::new();

        if improvement_percent > 10.0 {
            recommendations.push("Significant performance improvement detected".to_string());
        } else if improvement_percent < -10.0 {
            recommendations.push("Performance degradation detected - investigate bottlenecks".to_string());
        }

        if current.error_rate > baseline.error_rate + 0.05 {
            recommendations.push("Error rate has increased - check for regressions".to_string());
        }

        if current.p95_latency > baseline.p95_latency * 1.5 {
            recommendations.push("Latency tail has worsened - optimize for high percentiles".to_string());
        }

        ComparativeBenchmark {
            baseline: baseline.clone(),
            current: current.clone(),
            improvement_percent,
            statistically_significant,
            recommendations,
        }
    }

    /// Generate performance report
    pub async fn generate_performance_report(&self, results: &[BenchmarkResult]) -> String {
        let mut report = String::new();

        report.push_str("# Performance Benchmark Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        for result in results {
            report.push_str(&format!("## Benchmark: {}\n\n", result.name));
            report.push_str(&format!("- **Total Duration**: {:.2}s\n", result.total_duration.as_secs_f64()));
            report.push_str(&format!("- **Throughput**: {:.2} req/s\n", result.throughput));
            report.push_str(&format!("- **Average Latency**: {:.2}ms\n", result.avg_latency.as_millis()));
            report.push_str(&format!("- **Median Latency**: {:.2}ms\n", result.median_latency.as_millis()));
            report.push_str(&format!("- **95th Percentile**: {:.2}ms\n", result.p95_latency.as_millis()));
            report.push_str(&format!("- **99th Percentile**: {:.2}ms\n", result.p99_latency.as_millis()));
            report.push_str(&format!("- **Memory Usage**: {:.2} MB\n", result.memory_usage_mb));
            report.push_str(&format!("- **CPU Usage**: {:.2}%\n", result.cpu_usage_percent));
            report.push_str(&format!("- **Error Rate**: {:.2}%\n", result.error_rate * 100.0));
            report.push_str("\n");
        }

        if results.len() > 1 {
            report.push_str("## Comparative Analysis\n\n");
            for i in 1..results.len() {
                let baseline = &results[0];
                let current = &results[i];
                let comparison = self.compare_benchmarks(baseline, current);

                report.push_str(&format!("### {} vs {}\n", current.name, baseline.name));
                report.push_str(&format!("- **Performance Change**: {:.2}%\n", comparison.improvement_percent));
                report.push_str(&format!("- **Statistical Significance**: {}\n", comparison.statistically_significant));

                if !comparison.recommendations.is_empty() {
                    report.push_str("- **Recommendations**:\n");
                    for rec in &comparison.recommendations {
                        report.push_str(&format!("  - {}\n", rec));
                    }
                }
                report.push_str("\n");
            }
        }

        report
    }

    /// Benchmark ML model training performance
    pub async fn benchmark_ml_training(
        &self,
        benchmark_name: &str,
        trainer: &MLModelTrainer,
        dataset: &crate::ml_trainer::TrainingDataset,
        model_type: MLModelType,
        training_config: &TrainingConfig,
    ) -> Result<BenchmarkResult> {
        let predictor_function = || async {
            let _result = trainer.train_model(dataset, model_type.clone()).await?;
            Ok(())
        };

        self.benchmark_warmup_predictor(benchmark_name, predictor_function).await
    }

    /// Benchmark ML model evaluation performance
    pub async fn benchmark_ml_evaluation(
        &self,
        benchmark_name: &str,
        evaluator: &MLModelEvaluator,
        models: &[(&dyn crate::ml_trainer::MLModel, &str)],
        dataset: &crate::ml_trainer::TrainingDataset,
        training_config: &TrainingConfig,
    ) -> Result<BenchmarkResult> {
        let predictor_function = || async {
            let _result = evaluator.compare_models(models, dataset, training_config).await?;
            Ok(())
        };

        self.benchmark_warmup_predictor(benchmark_name, predictor_function).await
    }

    /// Measure current system resources
    async fn measure_system_resources(&self) -> Result<(f64, f64)> {
        let mut system = self.system.write().await;
        system.refresh_all();

        let memory_usage = system.used_memory() as f64 / 1024.0 / 1024.0; // MB
        let cpu_usage = system.global_cpu_info().cpu_usage() as f64;

        Ok((memory_usage, cpu_usage))
    }

    /// Calculate latency statistics
    fn calculate_latency_stats(&self, latencies: &[Duration]) -> Result<(Duration, Duration, Duration, Duration, Duration, Duration)> {
        if latencies.is_empty() {
            return Err(WarmupError::PredictionEngine {
                message: "No latency measurements available".to_string(),
            });
        }

        let mut sorted_latencies = latencies.to_vec();
        sorted_latencies.sort();

        let len = sorted_latencies.len();
        let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
        let median_latency = sorted_latencies[len / 2];

        let p95_index = (len as f64 * 0.95) as usize;
        let p99_index = (len as f64 * 0.99) as usize;

        let p95_latency = sorted_latencies[p95_index.min(len - 1)];
        let p99_latency = sorted_latencies[p99_index.min(len - 1)];
        let min_latency = sorted_latencies[0];
        let max_latency = sorted_latencies[len - 1];

        Ok((avg_latency, median_latency, p95_latency, p99_latency, min_latency, max_latency))
    }

    /// Calculate standard deviation
    fn calculate_std_dev(&self, values: &[f64]) -> f64 {
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        variance.sqrt()
    }

    /// Select request pattern based on weights
    fn select_request_pattern(&self, patterns: &[RequestPattern], cumulative_weights: &[f64]) -> WarmupRequest {
        let rand_val: f64 = rand::random();

        for (i, &weight) in cumulative_weights.iter().enumerate() {
            if rand_val <= weight {
                return patterns[i].request_template.clone();
            }
        }

        patterns.last().unwrap().request_template.clone()
    }

    /// Get benchmark history
    pub async fn get_history(&self) -> Vec<BenchmarkResult> {
        self.history.read().await.clone()
    }

    /// Clear benchmark history
    pub async fn clear_history(&self) {
        let mut history = self.history.write().await;
        history.clear();
    }

    /// Export benchmark results to JSON
    pub async fn export_results(&self, filename: &str) -> Result<()> {
        let history = self.get_history().await;
        let json = serde_json::to_string_pretty(&history)?;

        tokio::fs::write(filename, json).await.map_err(|e| {
            WarmupError::PredictionEngine {
                message: format!("Failed to write benchmark results: {}", e),
            }
        })?;

        Ok(())
    }

    /// Micro-benchmark individual algorithms (7 core components)
    pub async fn micro_benchmark_components<F>(
        &self,
        component_name: &str,
        algorithm_function: F,
    ) -> Result<BenchmarkResult>
    where
        F: Fn() -> Result<()> + Send + Sync,
    {
        let predictor_function = || async {
            // Wrap sync algorithm in async context
            tokio::task::spawn_blocking(move || algorithm_function())
                .await
                .map_err(|e| WarmupError::PredictionEngine {
                    message: format!("Micro-benchmark task failed: {}", e),
                })?
        };

        self.benchmark_warmup_predictor(
            &format!("micro_{}", component_name),
            predictor_function,
        ).await
    }

    /// Analyze latency with detailed percentiles
    pub fn analyze_latency_distribution(&self, latencies: &[Duration]) -> Result<LatencyAnalysis> {
        if latencies.is_empty() {
            return Err(WarmupError::PredictionEngine {
                message: "No latency data available for analysis".to_string(),
            });
        }

        let mut sorted_latencies = latencies.to_vec();
        sorted_latencies.sort();

        let len = sorted_latencies.len();
        let mean = latencies.iter().sum::<Duration>() / latencies.len() as u32;

        // Calculate percentiles
        let p50_idx = (len as f64 * 0.5) as usize;
        let p90_idx = (len as f64 * 0.9) as usize;
        let p95_idx = (len as f64 * 0.95) as usize;
        let p99_idx = (len as f64 * 0.99) as usize;
        let p999_idx = (len as f64 * 0.999) as usize;

        let p50 = sorted_latencies[p50_idx.min(len - 1)];
        let p90 = sorted_latencies[p90_idx.min(len - 1)];
        let p95 = sorted_latencies[p95_idx.min(len - 1)];
        let p99 = sorted_latencies[p99_idx.min(len - 1)];
        let p999 = sorted_latencies[p999_idx.min(len - 1)];

        // Calculate jitter (coefficient of variation)
        let variance = latencies.iter()
            .map(|&lat| {
                let diff = lat.as_secs_f64() - mean.as_secs_f64();
                diff * diff
            })
            .sum::<f64>() / latencies.len() as f64;
        let std_dev = Duration::from_secs_f64(variance.sqrt());
        let jitter = std_dev;

        Ok(LatencyAnalysis {
            p50,
            p90,
            p95,
            p99,
            p999,
            mean,
            std_dev,
            min: sorted_latencies[0],
            max: sorted_latencies[len - 1],
            jitter,
        })
    }

    /// Memory profiling with leak detection
    pub async fn memory_profile<F, Fut>(
        &self,
        test_name: &str,
        test_function: F,
    ) -> Result<MemoryProfile>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        let start_time = Instant::now();

        // Capture initial memory state
        let (initial_heap, initial_cpu) = self.measure_system_resources().await?;

        // Run test with memory monitoring
        let result = test_function().await;

        // Capture final memory state
        let (final_heap, final_cpu) = self.measure_system_resources().await?;
        let duration = start_time.elapsed();

        // Simple leak detection based on memory growth
        let memory_growth = final_heap as f64 - initial_heap as f64;
        let potential_leaks = if memory_growth > 50.0 * 1024.0 * 1024.0 { // 50MB threshold
            vec![format!("Memory growth of {:.2}MB detected", memory_growth / (1024.0 * 1024.0))]
        } else {
            vec![]
        };

        // Fragmentation estimation (simplified)
        let fragmentation_ratio = if final_heap > initial_heap {
            (final_heap as f64 - initial_heap as f64) / final_heap as f64
        } else {
            0.0
        };

        if let Err(e) = result {
            return Err(e);
        }

        Ok(MemoryProfile {
            heap_size_bytes: final_heap as u64 * 1024 * 1024, // Convert MB to bytes
            peak_heap_size_bytes: final_heap as u64 * 1024 * 1024,
            total_allocations: 0, // Would need instrumentation
            total_deallocations: 0, // Would need instrumentation
            potential_leaks,
            fragmentation_ratio,
            timestamp: Utc::now(),
        })
    }

    /// Statistical analysis with confidence intervals
    pub async fn statistical_analysis(&self, metric_name: &str, samples: &[f64]) -> Result<StatisticalAnalysis> {
        if samples.is_empty() {
            return Err(WarmupError::PredictionEngine {
                message: "No samples provided for statistical analysis".to_string(),
            });
        }

        let n = samples.len() as f64;
        let mean = samples.iter().sum::<f64>() / n;

        let variance = samples.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / (n - 1.0);
        let std_dev = variance.sqrt();

        // Calculate skewness and kurtosis
        let skewness = samples.iter()
            .map(|x| ((x - mean) / std_dev).powi(3))
            .sum::<f64>() / n;

        let kurtosis = samples.iter()
            .map(|x| ((x - mean) / std_dev).powi(4))
            .sum::<f64>() / n - 3.0;

        // 95% confidence interval (assuming normal distribution)
        let margin_of_error = 1.96 * std_dev / n.sqrt();
        let confidence_interval = (mean - margin_of_error, mean + margin_of_error);

        // Outlier detection using IQR method
        let mut sorted_samples = samples.to_vec();
        sorted_samples.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_idx = (n * 0.25) as usize;
        let q3_idx = (n * 0.75) as usize;
        let q1 = sorted_samples[q1_idx.min(samples.len() - 1)];
        let q3 = sorted_samples[q3_idx.min(samples.len() - 1)];
        let iqr = q3 - q1;

        let outlier_count = samples.iter()
            .filter(|&&x| x < q1 - 1.5 * iqr || x > q3 + 1.5 * iqr)
            .count();

        Ok(StatisticalAnalysis {
            mean,
            std_dev,
            variance,
            skewness,
            kurtosis,
            confidence_interval_95: confidence_interval,
            sample_size: samples.len(),
            outlier_count,
        })
    }

    /// Regression detection against baselines
    pub async fn detect_regression(&self, metric_name: &str, current_value: f64, threshold: f64) -> Result<bool> {
        let baselines = self.baselines.read().await;

        if let Some(baseline) = baselines.get(metric_name) {
            let baseline_throughput = baseline.throughput;
            let degradation = (baseline_throughput - current_value) / baseline_throughput;

            Ok(degradation > threshold)
        } else {
            // No baseline available, treat as no regression
            Ok(false)
        }
    }

    /// Set baseline for regression detection
    pub async fn set_baseline(&self, metric_name: &str, result: BenchmarkResult) -> Result<()> {
        let mut baselines = self.baselines.write().await;
        baselines.insert(metric_name.to_string(), result);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ModelTask, Complexity, RequestPriority, UserContext, ProjectContext, WarmupRequest};

    #[tokio::test]
    async fn test_benchmarker_creation() {
        let config = BenchmarkConfig {
            iterations: 100,
            warmup_iterations: 10,
            max_duration: Duration::from_secs(60),
            memory_profiling: true,
            cpu_profiling: true,
            detailed_latency: true,
            confidence_level: 0.95,
            concurrent_requests: 10,
        };

        let benchmarker = PerformanceBenchmarker::new(config);
        assert!(benchmarker.history.try_read().is_ok());
    }

    #[tokio::test]
    async fn test_latency_statistics() {
        let benchmarker = PerformanceBenchmarker::new(BenchmarkConfig::default());

        let latencies = vec![
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(15),
            Duration::from_millis(25),
            Duration::from_millis(12),
        ];

        let stats = benchmarker.calculate_latency_stats(&latencies);
        assert!(stats.is_ok());

        let (avg, median, p95, p99, min, max) = stats.unwrap();
        assert_eq!(min, Duration::from_millis(10));
        assert_eq!(max, Duration::from_millis(25));
        assert!(avg >= Duration::from_millis(10));
        assert!(median >= Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_benchmark_comparison() {
        let benchmarker = PerformanceBenchmarker::new(BenchmarkConfig::default());

        let baseline = BenchmarkResult {
            name: "baseline".to_string(),
            total_duration: Duration::from_secs(10),
            avg_latency: Duration::from_millis(50),
            median_latency: Duration::from_millis(45),
            p95_latency: Duration::from_millis(80),
            p99_latency: Duration::from_millis(100),
            min_latency: Duration::from_millis(10),
            max_latency: Duration::from_millis(200),
            throughput: 100.0,
            memory_usage_mb: 512.0,
            cpu_usage_percent: 25.0,
            error_rate: 0.01,
            latency_distribution: vec![Duration::from_millis(50); 100],
            timestamp: chrono::Utc::now(),
        };

        let current = BenchmarkResult {
            name: "current".to_string(),
            total_duration: Duration::from_secs(8),
            avg_latency: Duration::from_millis(40),
            median_latency: Duration::from_millis(38),
            p95_latency: Duration::from_millis(70),
            p99_latency: Duration::from_millis(90),
            min_latency: Duration::from_millis(8),
            max_latency: Duration::from_millis(150),
            throughput: 125.0,
            memory_usage_mb: 480.0,
            cpu_usage_percent: 22.0,
            error_rate: 0.005,
            latency_distribution: vec![Duration::from_millis(40); 100],
            timestamp: chrono::Utc::now(),
        };

        let comparison = benchmarker.compare_benchmarks(&baseline, &current);
        assert!(comparison.improvement_percent > 0.0);
        assert!(!comparison.recommendations.is_empty());
    }
}

// Implementations for new structs

impl StatisticalAnalyzer {
    /// Create new statistical analyzer
    pub fn new() -> Self {
        Self {
            samples: HashMap::new(),
            analysis_cache: HashMap::new(),
        }
    }

    /// Add sample to dataset
    pub fn add_sample(&mut self, metric_name: &str, value: f64) {
        self.samples.entry(metric_name.to_string())
            .or_insert_with(Vec::new)
            .push(value);
        // Invalidate cache for this metric
        self.analysis_cache.remove(metric_name);
    }

    /// Get statistical analysis for metric
    pub fn analyze(&mut self, metric_name: &str) -> Option<&StatisticalAnalysis> {
        if !self.analysis_cache.contains_key(metric_name) {
            if let Some(samples) = self.samples.get(metric_name) {
                if let Ok(analysis) = self.calculate_analysis(samples) {
                    self.analysis_cache.insert(metric_name.to_string(), analysis);
                }
            }
        }
        self.analysis_cache.get(metric_name)
    }

    /// Calculate statistical analysis from samples
    fn calculate_analysis(&self, samples: &[f64]) -> Result<StatisticalAnalysis> {
        if samples.is_empty() {
            return Err(WarmupError::PredictionEngine {
                message: "No samples provided for statistical analysis".to_string(),
            });
        }

        let n = samples.len() as f64;
        let mean = samples.iter().sum::<f64>() / n;

        let variance = samples.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / (n - 1.0);
        let std_dev = variance.sqrt();

        // Calculate skewness and kurtosis
        let skewness = samples.iter()
            .map(|x| ((x - mean) / std_dev).powi(3))
            .sum::<f64>() / n;

        let kurtosis = samples.iter()
            .map(|x| ((x - mean) / std_dev).powi(4))
            .sum::<f64>() / n - 3.0;

        // 95% confidence interval (assuming normal distribution)
        let margin_of_error = 1.96 * std_dev / n.sqrt();
        let confidence_interval = (mean - margin_of_error, mean + margin_of_error);

        // Outlier detection using IQR method
        let mut sorted_samples = samples.to_vec();
        sorted_samples.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_idx = (n * 0.25) as usize;
        let q3_idx = (n * 0.75) as usize;
        let q1 = sorted_samples[q1_idx.min(samples.len() - 1)];
        let q3 = sorted_samples[q3_idx.min(samples.len() - 1)];
        let iqr = q3 - q1;

        let outlier_count = samples.iter()
            .filter(|&&x| x < q1 - 1.5 * iqr || x > q3 + 1.5 * iqr)
            .count();

        Ok(StatisticalAnalysis {
            mean,
            std_dev,
            variance,
            skewness,
            kurtosis,
            confidence_interval_95: confidence_interval,
            sample_size: samples.len(),
            outlier_count,
        })
    }
}

impl MemoryProfiler {
    /// Create new memory profiler
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            leak_detector: HashMap::new(),
        }
    }

    /// Take memory snapshot
    pub async fn take_snapshot(&mut self, label: &str) -> Result<()> {
        let benchmarker = PerformanceBenchmarker::new(BenchmarkConfig::default());
        let (memory_mb, _) = benchmarker.measure_system_resources().await?;

        let snapshot = MemoryProfile {
            heap_size_bytes: (memory_mb * 1024.0 * 1024.0) as u64,
            peak_heap_size_bytes: (memory_mb * 1024.0 * 1024.0) as u64,
            total_allocations: 0,
            total_deallocations: 0,
            potential_leaks: Vec::new(),
            fragmentation_ratio: 0.0,
            timestamp: Utc::now(),
        };

        self.snapshots.push(snapshot);
        Ok(())
    }

    /// Analyze memory patterns
    pub fn analyze_memory_patterns(&self) -> Vec<String> {
        let mut analysis = Vec::new();

        if self.snapshots.len() < 2 {
            analysis.push("Insufficient snapshots for memory analysis".to_string());
            return analysis;
        }

        let initial = &self.snapshots[0];
        let final_snapshot = &self.snapshots[self.snapshots.len() - 1];

        let memory_growth = final_snapshot.heap_size_bytes as f64 - initial.heap_size_bytes as f64;
        let growth_mb = memory_growth / (1024.0 * 1024.0);

        if growth_mb > 100.0 {
            analysis.push(format!("Significant memory growth detected: {:.2}MB", growth_mb));
        }

        // Simple trend analysis
        let mut increasing_trend = 0;
        for i in 1..self.snapshots.len() {
            if self.snapshots[i].heap_size_bytes > self.snapshots[i - 1].heap_size_bytes {
                increasing_trend += 1;
            }
        }

        if increasing_trend as f64 / self.snapshots.len() as f64 > 0.7 {
            analysis.push("Memory usage shows consistent upward trend - potential leak".to_string());
        }

        analysis
    }
}

impl LoadTester {
    /// Create new load tester
    pub fn new() -> Self {
        Self {
            scenarios: HashMap::new(),
            concurrency_limiter: Arc::new(Semaphore::new(100)), // Default limit
        }
    }

    /// Add load test scenario
    pub fn add_scenario(&mut self, scenario: LoadTestScenario) {
        self.scenarios.insert(scenario.name.clone(), scenario);
    }

    /// Run load test scenario
    pub async fn run_scenario<F, Fut>(
        &self,
        scenario_name: &str,
        request_fn: F,
    ) -> Result<BenchmarkResult>
    where
        F: Fn(WarmupRequest) -> Fut + Send + Sync + Clone,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        let scenario = self.scenarios.get(scenario_name)
            .ok_or_else(|| WarmupError::PredictionEngine {
                message: format!("Load test scenario '{}' not found", scenario_name),
            })?;

        let load_config = LoadTestConfig {
            target_throughput: scenario.target_throughput,
            duration: scenario.duration,
            ramp_up_time: Duration::from_secs(30), // Default ramp-up
            max_concurrent_users: 50, // Default concurrent users
            request_patterns: scenario.patterns.clone(),
        };

        let benchmarker = PerformanceBenchmarker::new(BenchmarkConfig::default());
        benchmarker.run_load_test(scenario_name, &load_config, request_fn).await
    }
}

impl AccuracyValidator {
    /// Create new accuracy validator
    pub fn new() -> Self {
        Self {
            ground_truth: Vec::new(),
            significance_threshold: 0.05, // 5% significance level
        }
    }

    /// Add ground truth test case
    pub fn add_ground_truth(&mut self, test_case: AccuracyTestCase) {
        self.ground_truth.push(test_case);
    }

    /// Validate prediction accuracy
    pub async fn validate_accuracy<F, Fut>(
        &self,
        predictor_fn: F,
    ) -> Result<AccuracyValidation>
    where
        F: Fn(&WarmupRequest) -> Fut,
        Fut: std::future::Future<Output = Result<Option<ModelId>>> + Send,
    {
        let mut true_positives = 0u64;
        let mut true_negatives = 0u64;
        let mut false_positives = 0u64;
        let mut false_negatives = 0u64;

        let mut confidences = Vec::new();

        for test_case in &self.ground_truth {
            let prediction = predictor_fn(&test_case.input).await?;
            confidences.push(test_case.expected_confidence);

            match (prediction.as_ref(), test_case.expected_result.as_ref()) {
                (Some(pred), Some(expected)) if pred == expected => {
                    true_positives += 1;
                }
                (Some(_), Some(_)) => {
                    false_positives += 1;
                }
                (None, None) => {
                    true_negatives += 1;
                }
                (Some(_), None) => {
                    false_positives += 1;
                }
                (None, Some(_)) => {
                    false_negatives += 1;
                }
            }
        }

        let total_predictions = self.ground_truth.len() as u64;
        let correct_predictions = true_positives + true_negatives;
        let overall_accuracy = correct_predictions as f64 / total_predictions as f64;

        let precision = if true_positives + false_positives > 0 {
            true_positives as f64 / (true_positives + false_positives) as f64
        } else {
            0.0
        };

        let recall = if true_positives + false_negatives > 0 {
            true_positives as f64 / (true_positives + false_negatives) as f64
        } else {
            0.0
        };

        let f1_score = if precision + recall > 0.0 {
            2.0 * precision * recall / (precision + recall)
        } else {
            0.0
        };

        // Simple statistical significance test (placeholder)
        let statistical_significance = 0.01; // Would calculate properly
        let confidence_interval = (overall_accuracy - 0.05, overall_accuracy + 0.05);

        Ok(AccuracyValidation {
            overall_accuracy,
            precision,
            recall,
            f1_score,
            true_positives,
            true_negatives,
            false_positives,
            false_negatives,
            statistical_significance,
            confidence_interval,
        })
    }
}

impl StressTester {
    /// Create new stress tester
    pub fn new() -> Self {
        Self {
            saturation_points: Vec::new(),
            system_limits: SystemLimits {
                max_cpu_percent: 90.0,
                max_memory_mb: 8192,
                max_network_mbps: 100.0,
                max_concurrent_requests: 1000,
            },
        }
    }

    /// Run stress test to find saturation point
    pub async fn find_saturation_point<F, Fut>(
        &mut self,
        test_name: &str,
        max_concurrent: usize,
        request_fn: F,
    ) -> Result<()>
    where
        F: Fn() -> Fut + Send + Sync + Clone,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        let benchmarker = PerformanceBenchmarker::new(BenchmarkConfig::default());

        for concurrent in (10..=max_concurrent).step_by(10) {
            let config = BenchmarkConfig {
                iterations: 100,
                warmup_iterations: 10,
                max_duration: Duration::from_secs(30),
                memory_profiling: true,
                cpu_profiling: true,
                detailed_latency: true,
                confidence_level: 0.95,
                concurrent_requests: concurrent,
            };

            let benchmarker = PerformanceBenchmarker::new(config);
            let predictor_fn = || async { request_fn().await };
            let result = benchmarker.benchmark_warmup_predictor(test_name, predictor_fn).await?;

            let saturation_point = SaturationPoint {
                load_level: concurrent as f64,
                response_time: result.avg_latency,
                error_rate: result.error_rate,
                timestamp: Utc::now(),
            };

            self.saturation_points.push(saturation_point);

            // Stop if error rate is too high or latency is too high
            if result.error_rate > 0.1 || result.avg_latency > Duration::from_millis(5000) {
                break;
            }
        }

        Ok(())
    }
}

impl AutomatedMonitor {
    /// Create new automated monitor
    pub fn new() -> Self {
        Self {
            alert_thresholds: HashMap::new(),
            historical_data: Vec::new(),
            regression_detector: RegressionDetector::new(),
        }
    }

    /// Set alert threshold
    pub fn set_alert_threshold(&mut self, metric_name: &str, threshold: f64) {
        self.alert_thresholds.insert(metric_name.to_string(), threshold);
    }

    /// Record performance snapshot
    pub async fn record_snapshot(&mut self, snapshot: PerformanceSnapshot) -> Result<Vec<String>> {
        self.historical_data.push(snapshot.clone());

        let mut alerts = Vec::new();

        // Check thresholds
        if let Some(cpu_threshold) = self.alert_thresholds.get("cpu_percent") {
            if snapshot.system_metrics.cpu_percent > *cpu_threshold {
                alerts.push(format!("CPU usage {:.1}% exceeds threshold {:.1}%",
                    snapshot.system_metrics.cpu_percent, cpu_threshold));
            }
        }

        if let Some(memory_threshold) = self.alert_thresholds.get("memory_mb") {
            if snapshot.system_metrics.memory_mb > *memory_threshold {
                alerts.push(format!("Memory usage {:.1}MB exceeds threshold {:.1}MB",
                    snapshot.system_metrics.memory_mb, memory_threshold));
            }
        }

        if let Some(latency_threshold) = self.alert_thresholds.get("avg_latency_ms") {
            if snapshot.application_metrics.avg_latency_ms > *latency_threshold {
                alerts.push(format!("Average latency {:.1}ms exceeds threshold {:.1}ms",
                    snapshot.application_metrics.avg_latency_ms, latency_threshold));
            }
        }

        Ok(alerts)
    }

    /// Generate performance report
    pub fn generate_report(&self) -> String {
        let mut report = String::from("# Automated Performance Report\n\n");

        if self.historical_data.is_empty() {
            report.push_str("No performance data available.\n");
            return report;
        }

        let latest = &self.historical_data[self.historical_data.len() - 1];
        report.push_str(&format!("Latest snapshot: {}\n\n", latest.timestamp.format("%Y-%m-%d %H:%M:%S")));

        report.push_str("## System Metrics\n");
        report.push_str(&format!("- CPU Usage: {:.1}%\n", latest.system_metrics.cpu_percent));
        report.push_str(&format!("- Memory Usage: {:.1}MB\n", latest.system_metrics.memory_mb));
        report.push_str(&format!("- Disk I/O: {} bytes\n", latest.system_metrics.disk_io_bytes));
        report.push_str(&format!("- Network I/O: {} bytes\n", latest.system_metrics.network_io_bytes));

        report.push_str("\n## Application Metrics\n");
        report.push_str(&format!("- Active Connections: {}\n", latest.application_metrics.active_connections));
        report.push_str(&format!("- Request Rate: {:.1} req/s\n", latest.application_metrics.request_rate));
        report.push_str(&format!("- Error Rate: {:.3}%\n", latest.application_metrics.error_rate * 100.0));
        report.push_str(&format!("- Average Latency: {:.1}ms\n", latest.application_metrics.avg_latency_ms));

        report.push_str("\n## Trends\n");
        if self.historical_data.len() > 1 {
            let prev = &self.historical_data[self.historical_data.len() - 2];
            let cpu_change = latest.system_metrics.cpu_percent - prev.system_metrics.cpu_percent;
            let memory_change = latest.system_metrics.memory_mb - prev.system_metrics.memory_mb;
            let latency_change = latest.application_metrics.avg_latency_ms - prev.application_metrics.avg_latency_ms;

            report.push_str(&format!("- CPU Change: {:.1}%\n", cpu_change));
            report.push_str(&format!("- Memory Change: {:.1}MB\n", memory_change));
            report.push_str(&format!("- Latency Change: {:.1}ms\n", latency_change));
        }

        report
    }
}

impl RegressionDetector {
    /// Create new regression detector
    pub fn new() -> Self {
        Self {
            baselines: HashMap::new(),
            thresholds: HashMap::new(),
        }
    }

    /// Set baseline for metric
    pub fn set_baseline(&mut self, metric_name: &str, value: f64) {
        self.baselines.insert(metric_name.to_string(), value);
    }

    /// Set threshold for regression detection
    pub fn set_threshold(&mut self, metric_name: &str, threshold_percent: f64) {
        self.thresholds.insert(metric_name.to_string(), threshold_percent);
    }

    /// Detect regression
    pub fn detect_regression(&self, metric_name: &str, current_value: f64) -> Option<f64> {
        if let Some(baseline) = self.baselines.get(metric_name) {
            let threshold = self.thresholds.get(metric_name).unwrap_or(&0.05); // 5% default
            let change_percent = (current_value - *baseline) / *baseline;

            if change_percent.abs() > *threshold {
                Some(change_percent)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 1000,
            warmup_iterations: 100,
            max_duration: Duration::from_secs(300),
            memory_profiling: true,
            cpu_profiling: true,
            detailed_latency: true,
            confidence_level: 0.95,
            concurrent_requests: 10,
        }
    }
}
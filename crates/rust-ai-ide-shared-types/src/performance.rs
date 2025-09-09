//! Unified Performance Metrics for Rust AI IDE
//!
//! This module provides a comprehensive, unified performance metrics system
//! that consolidates all disparate PerformanceMetrics definitions found across
//! the various crates in the workspace.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Unified Performance Metrics Structure
///
/// Consolidates all performance metrics from across the codebase into a single,
/// extensible structure. Supports timing, counters, rates, memory usage, and
/// arbitrary custom metrics through the extensions field.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    /// Timestamp when metrics were collected (milliseconds since epoch)
    pub timestamp: u64,

    /// Core timing metrics (in nanoseconds)
    pub timing: TimingMetrics,

    /// Counter metrics (operations, hits, etc.)
    pub counters: CounterMetrics,

    /// Rate/percentage metrics
    pub rates: RateMetrics,

    /// Memory and resource usage
    pub resources: ResourceMetrics,

    /// Analysis-specific metrics
    pub analysis: AnalysisMetrics,

    /// Security-specific metrics
    pub security: SecurityMetrics,

    /// Build and compilation metrics
    pub build: BuildMetrics,

    /// Learning and ML specific metrics
    pub learning: LearningMetrics,

    /// Custom extensions for crate-specific metrics
    pub extensions: HashMap<String, MetricValue>,

    /// Backward compatibility flat fields for rust-ai-ide-performance crate

    /// CPU usage percentage
    pub cpu_usage_percent: Option<f64>,

    /// Memory usage in MB
    pub memory_usage_mb: Option<f64>,

    /// Disk I/O in MB per second
    pub disk_io_mb_per_sec: Option<f64>,

    /// Network I/O in MB per second
    pub network_io_mb_per_sec: Option<f64>,

    /// Response time in milliseconds
    pub response_time_ms: Option<f64>,

    /// Throughput in items per second
    pub throughput_items_per_sec: Option<u64>,
}

/// Core timing metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimingMetrics {
    /// Total execution time
    pub total_time_ns: Option<u64>,
    /// Response time/average response time
    pub response_time_ns: Option<u64>,
    /// Compilation time
    pub compile_time_ns: Option<u64>,
    /// Analysis time
    pub analysis_time_ns: Option<u64>,
    /// Cache operation time
    pub cache_time_ns: Option<u64>,
    /// Encryption/decryption time
    pub crypto_time_ns: Option<u64>,
    /// Latency measurements (vec for multiple samples)
    pub latency_ns: Vec<u64>,
}

/// Counter metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CounterMetrics {
    /// Total number of operations
    pub total_operations: Option<u64>,
    /// Operations completed successfully
    pub successful_operations: Option<u64>,
    /// Operations that failed
    pub failed_operations: Option<u64>,
    /// Cache hits
    pub cache_hits: Option<u64>,
    /// Cache misses
    pub cache_misses: Option<u64>,
    /// Memory allocations analyzed
    pub allocations_analyzed: Option<u64>,
    /// Errors detected/counted
    pub error_count: Option<u64>,
}

/// Rate and percentage metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RateMetrics {
    /// CPU usage percentage
    pub cpu_usage_percent: Option<f64>,
    /// Memory usage percentage
    pub memory_usage_percent: Option<f64>,
    /// Cache hit rate
    pub cache_hit_rate: Option<f64>,
    /// Success rate
    pub success_rate: Option<f64>,
    /// Throughput (operations per second)
    pub throughput_ops_per_sec: Option<f64>,
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceMetrics {
    /// Memory usage in bytes
    pub memory_bytes: Option<u64>,
    /// Peak memory usage
    pub peak_memory_bytes: Option<u64>,
    /// CPU time used
    pub cpu_time_ns: Option<u64>,
    /// Network bytes transferred
    pub network_bytes: Option<u64>,
}

/// Analysis-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisMetrics {
    /// Files analyzed
    pub files_analyzed: Option<u64>,
    /// Lines of code analyzed
    pub lines_analyzed: Option<u64>,
    /// Refactoring suggestions generated
    pub refactoring_suggestions: Option<u64>,
    /// Code quality score
    pub quality_score: Option<f64>,
}

/// Security-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityMetrics {
    /// Encryption operations
    pub encryption_ops: Option<u64>,
    /// Decryption operations
    pub decryption_ops: Option<u64>,
    /// Average encryption time
    pub avg_encryption_time_ns: Option<u64>,
    /// Security scans performed
    pub security_scans: Option<u64>,
    /// Vulnerabilities found
    pub vulnerabilities_found: Option<u64>,
}

/// Build and compilation metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildMetrics {
    /// Total build time
    pub build_time_ns: Option<u64>,
    /// Build success
    pub build_successful: Option<bool>,
    /// Number of compilation warnings
    pub warnings_count: Option<u64>,
    /// Number of compilation errors
    pub errors_count: Option<u64>,
    /// Incremental build flag
    pub incremental_build: Option<bool>,
}

/// Learning and ML specific metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LearningMetrics {
    /// Training operations
    pub training_ops: Option<u64>,
    /// Model predictions
    pub predictions: Option<u64>,
    /// Learning algorithm iterations
    pub learning_iterations: Option<u64>,
    /// Model accuracy
    pub model_accuracy: Option<f64>,
    /// Training loss
    pub training_loss: Option<f64>,
}

/// Flexible metric value for extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricValue {
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// Boolean value
    Bool(bool),
    /// List of integers
    IntList(Vec<i64>),
    /// List of floats
    FloatList(Vec<f64>),
    /// List of strings
    StringList(Vec<String>),
}

impl Default for MetricValue {
    fn default() -> Self {
        MetricValue::Integer(0)
    }
}

impl PerformanceMetrics {
    /// Create a new PerformanceMetrics instance with current timestamp
    pub fn new() -> Self {
        Self {
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            ..Default::default()
        }
    }

    /// Record a timing measurement
    pub fn record_timing(&mut self, timing_type: TimingType, duration: Duration) {
        let ns = duration.as_nanos() as u64;
        match timing_type {
            TimingType::Total => self.timing.total_time_ns = Some(ns),
            TimingType::Response => {
                self.timing.response_time_ns = Some(ns);
                self.response_time_ms = Some(ns as f64 / 1_000_000.0);
            }
            TimingType::Compile => self.timing.compile_time_ns = Some(ns),
            TimingType::Analysis => self.timing.analysis_time_ns = Some(ns),
            TimingType::Cache => self.timing.cache_time_ns = Some(ns),
            TimingType::Crypto => self.timing.crypto_time_ns = Some(ns),
        }
    }

    /// Add latency sample
    pub fn add_latency(&mut self, latency: Duration) {
        self.timing.latency_ns.push(latency.as_nanos() as u64);
    }

    /// Record a counter increment
    pub fn increment_counter(&mut self, counter_type: CounterType) {
        let counter = match counter_type {
            CounterType::TotalOps => &mut self.counters.total_operations,
            CounterType::SuccessOps => &mut self.counters.successful_operations,
            CounterType::FailedOps => &mut self.counters.failed_operations,
            CounterType::CacheHits => &mut self.counters.cache_hits,
            CounterType::CacheMisses => &mut self.counters.cache_misses,
            CounterType::Allocations => &mut self.counters.allocations_analyzed,
            CounterType::Errors => &mut self.counters.error_count,
        };
        *counter = Some(counter.unwrap_or(0) + 1);
    }

    /// Set a rate value
    pub fn set_rate(&mut self, rate_type: RateType, value: f64) {
        match rate_type {
            RateType::CpuUsage => {
                self.rates.cpu_usage_percent = Some(value);
                self.cpu_usage_percent = Some(value);
            }
            RateType::MemoryUsage => self.rates.memory_usage_percent = Some(value),
            RateType::CacheHitRate=> self.rates.cache_hit_rate = Some(value),
            RateType::SuccessRate => self.rates.success_rate = Some(value),
            RateType::Throughput => {
                self.rates.throughput_ops_per_sec = Some(value);
                self.throughput_items_per_sec = Some(value as u64);
            }
        }
    }

    /// Calculate cache hit rate from hit/miss counters
    pub fn calculate_cache_hit_rate(&mut self) {
        if let (Some(hits), Some(misses)) = (self.counters.cache_hits, self.counters.cache_misses) {
            let total = hits + misses;
            if total > 0 {
                self.rates.cache_hit_rate = Some(hits as f64 / total as f64);
            }
        }
    }

    /// Add an extension metric
    pub fn add_extension(&mut self, key: impl Into<String>, value: MetricValue) {
        self.extensions.insert(key.into(), value);
    }

    /// Calculate success rate from success/failure counters
    pub fn calculate_success_rate(&mut self) {
        if let (Some(success), Some(failed)) = (self.counters.successful_operations, self.counters.failed_operations) {
            let total = success + failed;
            if total > 0 {
                self.rates.success_rate = Some(success as f64 / total as f64);
            }
        }
    }

    /// Calculate throughput from operations and time
    pub fn calculate_throughput(&mut self) {
        if let (Some(ops), Some(time_ns)) = (self.counters.total_operations, self.timing.total_time_ns) {
            if time_ns > 0 {
                let time_sec = time_ns as f64 / 1_000_000_000.0;
                self.rates.throughput_ops_per_sec = Some(ops as f64 / time_sec);
            }
        }
    }

    /// Merge another PerformanceMetrics into this one
    pub fn merge(&mut self, other: &PerformanceMetrics) {
        // Merge timing metrics
        if let Some(val) = other.timing.total_time_ns {
            self.timing.total_time_ns = Some(self.timing.total_time_ns.unwrap_or(0).max(val));
        }
        // Extend latency vectors
        self.timing.latency_ns.extend(&other.timing.latency_ns);

        // Add counters
        self.counters.total_operations = add_options(self.counters.total_operations, other.counters.total_operations);
        self.counters.successful_operations = add_options(self.counters.successful_operations, other.counters.successful_operations);
        self.counters.failed_operations = add_options(self.counters.failed_operations, other.counters.failed_operations);
        self.counters.cache_hits = add_options(self.counters.cache_hits, other.counters.cache_hits);
        self.counters.cache_misses = add_options(self.counters.cache_misses, other.counters.cache_misses);
        self.counters.allocations_analyzed = add_options(self.counters.allocations_analyzed, other.counters.allocations_analyzed);
        self.counters.error_count = add_options(self.counters.error_count, other.counters.error_count);

        // For extensions, union with overwrite
        for (key, value) in &other.extensions {
            self.extensions.insert(key.clone(), value.clone());
        }
    }
}

/// Helper function to add Option<u64> values
fn add_options(a: Option<u64>, b: Option<u64>) -> Option<u64> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x + y),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (None, None) => None,
    }
}

/// Timing metric types
#[derive(Debug, Clone)]
pub enum TimingType {
    /// Total execution time
    Total,
    /// Response time
    Response,
    /// Compilation time
    Compile,
    /// Analysis time
    Analysis,
    /// Cache operation time
    Cache,
    /// Cryptographic operation time
    Crypto,
}

/// Counter metric types
#[derive(Debug, Clone)]
pub enum CounterType {
    /// Total operations
    TotalOps,
    /// Successful operations
    SuccessOps,
    /// Failed operations
    FailedOps,
    /// Cache hits
    CacheHits,
    /// Cache misses
    CacheMisses,
    /// Memory allocations analyzed
    Allocations,
    /// Error count
    Errors,
}

/// Rate metric types
#[derive(Debug, Clone)]
pub enum RateType {
    /// CPU usage percentage
    CpuUsage,
    /// Memory usage percentage
    MemoryUsage,
    /// Cache hit rate
    CacheHitRate,
    /// Success rate
    SuccessRate,
    /// Operations per second
    Throughput,
}

/// Performance metrics builder for fluent API
#[derive(Debug, Clone)]
pub struct MetricsBuilder {
    metrics: PerformanceMetrics,
}

impl MetricsBuilder {
    /// Create a new metrics builder
    pub fn new() -> Self {
        Self {
            metrics: PerformanceMetrics::new(),
        }
    }

    /// Add timing measurement
    pub fn with_timing(mut self, timing_type: TimingType, duration: Duration) -> Self {
        self.metrics.record_timing(timing_type, duration);
        self
    }

    /// Add latency sample
    pub fn with_latency(mut self, latency: Duration) -> Self {
        self.metrics.add_latency(latency);
        self
    }

    /// Increment counter
    pub fn with_counter(mut self, counter_type: CounterType) -> Self {
        self.metrics.increment_counter(counter_type);
        self
    }

    /// Set rate value
    pub fn with_rate(mut self, rate_type: RateType, value: f64) -> Self {
        self.metrics.set_rate(rate_type, value);
        self
    }

    /// Add extension
    pub fn with_extension(mut self, key: impl Into<String>, value: MetricValue) -> Self {
        self.metrics.add_extension(key, value);
        self
    }

    /// Build the metrics
    pub fn build(self) -> PerformanceMetrics {
        self.metrics
    }
}

impl Default for MetricsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// TryFrom implementations for compatibility with existing PerformanceMetrics

impl TryFrom<&rust_ai_ide_common::perf_utils::PerformanceMetrics> for PerformanceMetrics {
    type Error = String;

    fn try_from(old: &rust_ai_ide_common::perf_utils::PerformanceMetrics) -> Result<Self, Self::Error> {
        let mut metrics = PerformanceMetrics::new();
        metrics.timing.total_time_ns = Some(old.duration.as_nanos() as u64);
        metrics.add_extension("operation_name", MetricValue::String(old.operation_name.clone()));
        Ok(metrics)
    }
}

// Implementation for rust_ai_ide_cargo commented out due to unresolved module
// impl TryFrom<&rust_ai_ide_cargo::commands::PerformanceMetrics> for PerformanceMetrics {
//     type Error = String;

//     fn try_from(old: &rust_ai_ide_cargo::commands::PerformanceMetrics) -> Result<Self, Self::Error> {
//         let mut metrics = PerformanceMetrics::new();
//         metrics.timing.total_time_ns = Some(old.total_time_ms as u64 * 1_000_000);
//         metrics.timing.compile_time_ns = Some(old.compile_time_ms as u64 * 1_000_000);
//         Ok(metrics)
//     }
// }

// Implementation for rust_ai_ide_performance commented out due to cyclic dependencies
// impl TryFrom<&rust_ai_ide_performance::PerformanceMetrics> for PerformanceMetrics {
//     type Error = String;

//     fn try_from(old: &rust_ai_ide_performance::PerformanceMetrics) -> Result<Self, Self::Error> {
//         let mut metrics = PerformanceMetrics::new();
//         metrics.rates.cpu_usage_percent = Some(old.cpu_usage_percent);
//         metrics.resources.memory_bytes = old.memory_usage_bytes;
//         metrics.timing.total_time_ns = Some(old.total_time_ns);
//         Ok(metrics)
//     }
// }

/// Create a metrics scope that automatically records timing
#[derive(Debug)]
pub struct MetricsScope {
    metrics: PerformanceMetrics,
    start_time: Instant,
    operation_name: String,
}

impl MetricsScope {
    /// Create a new metrics scope
    pub fn new(operation_name: impl Into<String>) -> Self {
        Self {
            metrics: PerformanceMetrics::new(),
            start_time: Instant::now(),
            operation_name: operation_name.into(),
        }
    }

    /// Complete the scope and return the metrics
    pub fn finish(self) -> PerformanceMetrics {
        let mut metrics = self.metrics;
        metrics.timing.total_time_ns = Some(self.start_time.elapsed().as_nanos() as u64);
        metrics.add_extension("operation_name", MetricValue::String(self.operation_name));
        metrics
    }

    /// Record a counter during the scope
    pub fn record_counter(&mut self, counter_type: CounterType) {
        self.metrics.increment_counter(counter_type);
    }

    /// Record a rate during the scope
    pub fn record_rate(&mut self, rate_type: RateType, value: f64) {
        self.metrics.set_rate(rate_type, value);
    }
}
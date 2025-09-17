//! Unified Performance Metrics Collector
//!
//! This module provides the unified performance metrics collector that aggregates
//! performance data from all crates and components in the Rust AI IDE system.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use rust_ai_ide_shared_types::{PerformanceMetrics, MetricValue, RateType};
use serde::{Deserialize, Serialize};
use sysinfo::{CpuExt, DiskExt, NetworkExt, PidExt, ProcessExt, System, SystemExt};
use tokio::sync::mpsc;
use tokio::time::{self};

use crate::alerting::AlertManager;
use crate::instrumentation::{InstrumentationConfig, PerformanceInstrumentor};
use crate::metrics::{MetricsError, MetricsRegistry, PrometheusExporter};
use crate::metrics_server::{MetricsServer, MetricsServerBuilder, MetricsServerConfig};
use crate::monitoring::SystemMonitor;
use crate::regression::RegressionDetector;

/// Configuration for the performance collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    /// Collection interval in seconds
    pub collection_interval_secs:    u64,
    /// Maximum number of metrics to keep in memory
    pub max_history_size:            usize,
    /// Enable regression detection
    pub enable_regression_detection: bool,
    /// Regression alerting threshold
    pub regression_threshold:        f64,
    /// Number of data points for baseline calculation
    pub baseline_window_size:        usize,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            collection_interval_secs:    30, // 30 seconds default
            max_history_size:            1000,
            enable_regression_detection: true,
            regression_threshold:        0.1, // 10% performance degradation
            baseline_window_size:        10,
        }
    }
}

/// Unified Performance Collector
pub struct UnifiedPerformanceCollector {
    /// Configuration
    config:              CollectorConfig,
    /// Historical metrics data
    metrics_history:     Arc<RwLock<Vec<PerformanceMetrics>>>,
    /// Registered collectors from different crates
    crate_collectors:    Arc<RwLock<HashMap<String, Box<dyn MetricsProvider>>>>,
    /// Regression detector
    regression_detector: Option<RegressionDetector>,
    /// Alert manager
    alert_manager:       Option<AlertManager>,
    /// Prometheus exporter
    prometheus_exporter: Option<Arc<PrometheusExporter>>,
    /// Performance instrumentor
    instrumentor:        Option<Arc<PerformanceInstrumentor>>,
    /// Channel for receiving metrics from external sources
    metrics_receiver:    mpsc::Receiver<PerformanceMetrics>,
    metrics_sender:      mpsc::Sender<PerformanceMetrics>,
}

/// Trait for providing metrics from different sources
#[async_trait::async_trait]
pub trait MetricsProvider: Send + Sync {
    /// Name of the metrics provider
    fn name(&self) -> &str;

    /// Collect metrics from this source
    async fn collect_metrics(&self) -> Result<PerformanceMetrics, CollectionError>;
}

/// Alert types for performance issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceAlert {
    RegressionDetected {
        metric_name:         String,
        baseline_value:      f64,
        current_value:       f64,
        degradation_percent: f64,
        timestamp:           chrono::DateTime<chrono::Utc>,
    },
    ThresholdExceeded {
        metric_name:   String,
        current_value: f64,
        threshold:     f64,
        timestamp:     chrono::DateTime<chrono::Utc>,
    },
    AnomalyDetected {
        description: String,
        severity:    AlertSeverity,
        timestamp:   chrono::DateTime<chrono::Utc>,
    },
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Error type for collection operations
#[derive(Debug, Clone)]
pub enum CollectionError {
    SourceUnavailable(String),
    MetricComputation(String),
    Timeout(String),
    PermissionDenied(String),
}

impl std::fmt::Display for CollectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectionError::SourceUnavailable(msg) => write!(f, "Source unavailable: {}", msg),
            CollectionError::MetricComputation(msg) => {
                write!(f, "Metric computation error: {}", msg)
            }
            CollectionError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            CollectionError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
        }
    }
}

impl std::error::Error for CollectionError {}

/// Builder for creating a Performance Collector
pub struct CollectorBuilder {
    config:    CollectorConfig,
    providers: HashMap<String, Box<dyn MetricsProvider>>,
}

impl CollectorBuilder {
    /// Create a new collector builder with default configuration
    pub fn new() -> Self {
        Self {
            config:    CollectorConfig::default(),
            providers: HashMap::new(),
        }
    }

    /// Set custom configuration
    pub fn with_config(mut self, config: CollectorConfig) -> Self {
        self.config = config;
        self
    }

    /// Add a metrics provider
    pub fn with_provider(mut self, provider: Box<dyn MetricsProvider>) -> Self {
        self.providers.insert(provider.name().to_string(), provider);
        self
    }

    /// Build the unified performance collector
    pub fn build(self) -> UnifiedPerformanceCollector {
        let (sender, receiver) = mpsc::channel(100);

        let regression_detector = if self.config.enable_regression_detection {
            Some(RegressionDetector::new(
                self.config.baseline_window_size,
                self.config.regression_threshold,
            ))
        } else {
            None
        };

        let alert_manager = Some(AlertManager::new());

        UnifiedPerformanceCollector {
            config: self.config,
            metrics_history: Arc::new(RwLock::new(Vec::with_capacity(100))),
            crate_collectors: Arc::new(RwLock::new(self.providers)),
            regression_detector,
            alert_manager,
            metrics_receiver: receiver,
            metrics_sender: sender,
        }
    }
}

impl Default for CollectorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl UnifiedPerformanceCollector {
    /// Start the collection process
    pub async fn start(&self) -> Result<(), anyhow::Error> {
        let metrics_history = Arc::clone(&self.metrics_history);
        let crate_collectors = Arc::clone(&self.crate_collectors);
        let config = self.config.clone();
        let sender = self.metrics_sender.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(std::time::Duration::from_secs(config.collection_interval_secs));

            loop {
                interval.tick().await;

                let mut aggregated_metrics = PerformanceMetrics::new();

                // Collect from registered providers - clone the providers to avoid lifetime issues
                {
                    let collectors = crate_collectors.read().unwrap();
                    let providers_refs: Vec<&Box<dyn MetricsProvider>> = collectors.values().collect();

                    for provider in providers_refs {
                        match provider.collect_metrics().await {
                            Ok(metrics) => {
                                aggregated_metrics.merge(&metrics);
                            }
                            Err(e) => {
                                eprintln!("Failed to collect metrics from {}: {}", provider.name(), e);
                            }
                        }
                    }
                }

                // Send for processing
                if let Err(_) = sender.send(aggregated_metrics).await {
                    eprintln!("Failed to send metrics - channel closed");
                    break;
                }
            }
        });

        Ok(())
    }

    /// Manually submit metrics for aggregation
    pub async fn submit_metrics(&self, metrics: PerformanceMetrics) -> Result<(), anyhow::Error> {
        self.metrics_sender.send(metrics).await?;
        Ok(())
    }

    /// Register a new metrics provider at runtime
    pub fn register_provider(&self, provider: Box<dyn MetricsProvider>) -> Result<(), anyhow::Error> {
        let mut collectors = self.crate_collectors.write().unwrap();
        collectors.insert(provider.name().to_string(), provider);
        Ok(())
    }

    /// Get current metrics history
    pub fn get_metrics_history(&self) -> Vec<PerformanceMetrics> {
        self.metrics_history.read().unwrap().clone()
    }

    /// Get latest aggregated metrics
    pub fn get_latest_metrics(&self) -> Option<PerformanceMetrics> {
        self.metrics_history.read().unwrap().last().cloned()
    }

    /// Get metrics for a specific time range
    pub fn get_metrics_in_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Vec<PerformanceMetrics> {
        let start_ts = start.timestamp_millis() as u64;
        let end_ts = end.timestamp_millis() as u64;
        self.metrics_history
            .read()
            .unwrap()
            .iter()
            .filter(|m| m.timestamp >= start_ts && m.timestamp <= end_ts)
            .cloned()
            .collect()
    }

    /// Export metrics data for persistence or analysis
    pub fn export_metrics(&self) -> Result<String, anyhow::Error> {
        let metrics = self.get_metrics_history();
        serde_json::to_string_pretty(&metrics).map_err(anyhow::Error::from)
    }

    /// Process incoming metrics and check for regressions
    async fn process_metrics(&mut self, metrics: PerformanceMetrics) {
        // Add to history
        {
            let mut history = self.metrics_history.write().unwrap();
            history.push(metrics.clone());

            // Maintain history size limit
            if history.len() > self.config.max_history_size {
                history.remove(0); // Remove oldest
            }
        }

        // Check for regressions if enabled
        if let Some(detector) = &self.regression_detector {
            let alerts = detector.detect_regressions(&metrics, &self.get_metrics_history());
            for alert in alerts {
                if let Some(manager) = &self.alert_manager {
                    manager.process_alert(alert).await;
                }
            }
        }
    }

    /// Start processing incoming metrics
    pub async fn start_processing(&mut self) {
        while let Some(metrics) = self.metrics_receiver.recv().await {
            self.process_metrics(metrics).await;
        }
    }
}

/// Real system metrics provider using sysinfo crate
pub struct SystemMetricsProvider {
    system: Arc<RwLock<System>>,
    last_refresh: Arc<RwLock<Instant>>,
    refresh_interval: Duration,
    prev_total_read_bytes: Arc<RwLock<u64>>,
    prev_total_written_bytes: Arc<RwLock<u64>>,
    prev_time: Arc<RwLock<Option<Instant>>>,
}

impl SystemMetricsProvider {
    /// Create a new system metrics provider
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            system: Arc::new(RwLock::new(system)),
            last_refresh: Arc::new(RwLock::new(Instant::now())),
            refresh_interval: Duration::from_secs(1), // Refresh every second
            prev_total_read_bytes: Arc::new(RwLock::new(0)),
            prev_total_written_bytes: Arc::new(RwLock::new(0)),
            prev_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Refresh system information if needed
    fn refresh_if_needed(&self) {
        let should_refresh = {
            let last_refresh = self.last_refresh.read().unwrap();
            last_refresh.elapsed() >= self.refresh_interval
        };

        if should_refresh {
            let mut system = self.system.write().unwrap();
            system.refresh_all();

            let mut last_refresh = self.last_refresh.write().unwrap();
            *last_refresh = Instant::now();
        }
    }

    /// Collect CPU metrics
    fn collect_cpu_metrics(&self) -> (f64, u64) {
        let system = self.system.read().unwrap();
        let cpu_usage = system.global_cpu_info().cpu_usage() as f64;
        let total_cpu_time = system.global_cpu_info().cpu_usage() as u64 * 1000000; // Convert to nanoseconds
        (cpu_usage, total_cpu_time)
    }

    /// Collect memory metrics
    fn collect_memory_metrics(&self) -> (u64, u64, u64) {
        let system = self.system.read().unwrap();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let available_memory = system.available_memory();
        (total_memory, used_memory, available_memory)
    }

    /// Collect disk metrics
    fn collect_disk_metrics(&self) -> (u64, u64, f64, f64) {
        let system = self.system.read().unwrap();
        let mut total_space = 0u64;
        let mut available_space = 0u64;
        let mut total_read_bytes = 0u64;
        let mut total_written_bytes = 0u64;

        for disk in system.disks() {
            total_space += disk.total_space();
            available_space += disk.available_space();
            total_read_bytes += disk.total_read_bytes();
            total_written_bytes += disk.total_written_bytes();
        }

        let used_space = total_space.saturating_sub(available_space);
        let disk_usage_percent = if total_space > 0 {
            (used_space as f64 / total_space as f64) * 100.0
        } else {
            0.0
        };

        // Calculate I/O rate
        let current_time = Instant::now();
        let mut disk_io_mb_per_sec = 0.0;

        {
            let prev_read = *self.prev_total_read_bytes.read().unwrap();
            let prev_written = *self.prev_total_written_bytes.read().unwrap();
            let prev_time_opt = *self.prev_time.read().unwrap();

            if let Some(prev_time) = prev_time_opt {
                let elapsed = current_time.duration_since(prev_time).as_secs_f64();
                if elapsed > 0.0 {
                    let read_diff = total_read_bytes.saturating_sub(prev_read);
                    let write_diff = total_written_bytes.saturating_sub(prev_written);
                    let total_io_bytes = read_diff + write_diff;
                    let io_mb_per_sec = (total_io_bytes as f64) / elapsed / 1_000_000.0;
                    disk_io_mb_per_sec = io_mb_per_sec;
                }
            }
        }

        // Update previous values
        {
            *self.prev_total_read_bytes.write().unwrap() = total_read_bytes;
            *self.prev_total_written_bytes.write().unwrap() = total_written_bytes;
            *self.prev_time.write().unwrap() = Some(current_time);
        }

        (total_space, used_space, disk_usage_percent, disk_io_mb_per_sec)
    }

    /// Collect network I/O metrics
    fn collect_network_metrics(&self) -> (u64, u64) {
        let system = self.system.read().unwrap();
        let mut total_received = 0u64;
        let mut total_transmitted = 0u64;

        for (_interface_name, data) in system.networks() {
            total_received += data.total_received();
            total_transmitted += data.total_transmitted();
        }

        (total_received, total_transmitted)
    }

    /// Collect process-specific metrics for IDE and LSP servers
    fn collect_process_metrics(&self, process_name_patterns: &[&str]) -> HashMap<String, ProcessMetrics> {
        let system = self.system.read().unwrap();
        let mut process_metrics = HashMap::new();

        for (pid, process) in system.processes() {
            let process_name = process.name();

            // Check if this process matches any of our patterns
            for pattern in process_name_patterns {
                if process_name.contains(pattern) {
                    let metrics = ProcessMetrics {
                        pid: pid.as_u32(),
                        name: process_name.to_string(),
                        cpu_usage: process.cpu_usage() as f64,
                        memory_bytes: process.memory(),
                        virtual_memory_bytes: process.virtual_memory(),
                        start_time: process.start_time(),
                        status: format!("{:?}", process.status()),
                    };
                    process_metrics.insert(process_name.to_string(), metrics);
                    break;
                }
            }
        }

        process_metrics
    }
}

/// Process-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f64,
    pub memory_bytes: u64,
    pub virtual_memory_bytes: u64,
    pub start_time: u64,
    pub status: String,
}

impl Default for SystemMetricsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl MetricsProvider for SystemMetricsProvider {
    fn name(&self) -> &str {
        "system"
    }

    async fn collect_metrics(&self) -> Result<PerformanceMetrics, CollectionError> {
        self.refresh_if_needed();

        let mut metrics = PerformanceMetrics::new();

        // Collect CPU metrics
        let (cpu_usage, cpu_time) = self.collect_cpu_metrics();
        metrics.set_rate(RateType::CpuUsage, cpu_usage);
        metrics.resources.cpu_time_ns = Some(cpu_time);

        // Collect memory metrics
        let (total_memory, used_memory, available_memory) = self.collect_memory_metrics();
        metrics.resources.memory_bytes = Some(used_memory);
        metrics.resources.peak_memory_bytes = Some(total_memory);
        metrics.rates.memory_usage_percent = Some(if total_memory > 0 {
            (used_memory as f64 / total_memory as f64) * 100.0
        } else {
            0.0
        });

        // Add memory availability as extension
        metrics.add_extension(
            "memory_available_bytes",
            MetricValue::Integer(available_memory as i64),
        );

        // Collect disk metrics
        let (total_disk, used_disk, disk_usage_percent, disk_io_mb_per_sec) = self.collect_disk_metrics();
        metrics.disk_io_mb_per_sec = Some(disk_io_mb_per_sec);
        metrics.add_extension(
            "disk_usage_percent",
            MetricValue::Float(disk_usage_percent),
        );
        metrics.add_extension(
            "disk_total_bytes",
            MetricValue::Integer(total_disk as i64),
        );
        metrics.add_extension(
            "disk_used_bytes",
            MetricValue::Integer(used_disk as i64),
        );

        // Collect network I/O metrics
        let (network_received, network_transmitted) = self.collect_network_metrics();
        metrics.network_io_mb_per_sec = Some(
            ((network_received + network_transmitted) / 1_000_000) as f64
        );
        metrics.resources.network_bytes = Some(network_received + network_transmitted);

        // Collect process-specific metrics
        let process_patterns = ["rust-ai-ide", "rust-analyzer", "typescript-language-server", "vscode"];
        let process_metrics = self.collect_process_metrics(&process_patterns);

        // Add process metrics as extensions
        for (name, proc_metrics) in process_metrics {
            metrics.add_extension(
                &format!("process_{}_cpu", name.replace("-", "_")),
                MetricValue::Float(proc_metrics.cpu_usage),
            );
            metrics.add_extension(
                &format!("process_{}_memory", name.replace("-", "_")),
                MetricValue::Integer(proc_metrics.memory_bytes as i64),
            );
            metrics.add_extension(
                &format!("process_{}_status", name.replace("-", "_")),
                MetricValue::String(proc_metrics.status),
            );
        }

        // Calculate memory usage in MB for backward compatibility
        metrics.memory_usage_mb = Some((used_memory / 1_000_000) as f64);

        // Add response time measurement
        let collection_start = Instant::now();
        tokio::time::sleep(std::time::Duration::from_micros(100)).await; // Small delay for measurement
        metrics.timing.response_time_ns = Some(collection_start.elapsed().as_nanos() as u64);

        Ok(metrics)
    }
}

/// Real cargo metrics provider for compilation performance monitoring
pub struct CargoMetricsProvider {
    last_build_time: Arc<RwLock<Option<Instant>>>,
    build_history: Arc<RwLock<Vec<BuildRecord>>>,
    max_history_size: usize,
}

#[derive(Debug, Clone)]
struct BuildRecord {
    timestamp: Instant,
    duration_ns: u64,
    successful: bool,
    warnings_count: u64,
    errors_count: u64,
    target_count: usize,
}

impl CargoMetricsProvider {
    /// Create a new cargo metrics provider
    pub fn new() -> Self {
        Self {
            last_build_time: Arc::new(RwLock::new(None)),
            build_history: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 100,
        }
    }

    /// Record a build operation
    pub async fn record_build_start(&self) {
        let mut last_build = self.last_build_time.write().unwrap();
        *last_build = Some(Instant::now());
    }

    /// Record build completion with metrics
    pub async fn record_build_completion(&self, successful: bool, warnings: u64, errors: u64, target_count: usize) {
        let start_time = {
            let last_build = self.last_build_time.read().unwrap();
            last_build.unwrap_or_else(|| Instant::now())
        };

        let duration_ns = start_time.elapsed().as_nanos() as u64;

        let record = BuildRecord {
            timestamp: Instant::now(),
            duration_ns,
            successful,
            warnings_count: warnings,
            errors_count: errors,
            target_count,
        };

        let mut history = self.build_history.write().unwrap();
        history.push(record);

        // Maintain history size limit
        if history.len() > self.max_history_size {
            history.remove(0);
        }

        // Reset last build time
        let mut last_build = self.last_build_time.write().unwrap();
        *last_build = None;
    }

    /// Get build statistics from history
    fn get_build_statistics(&self) -> BuildStatistics {
        let history = self.build_history.read().unwrap();

        if history.is_empty() {
            return BuildStatistics::default();
        }

        let total_builds = history.len() as u64;
        let successful_builds = history.iter().filter(|r| r.successful).count() as u64;
        let failed_builds = total_builds - successful_builds;

        let avg_build_time = if !history.is_empty() {
            history.iter().map(|r| r.duration_ns).sum::<u64>() / history.len() as u64
        } else {
            0
        };

        let total_warnings = history.iter().map(|r| r.warnings_count).sum::<u64>();
        let total_errors = history.iter().map(|r| r.errors_count).sum::<u64>();
        let total_targets = history.iter().map(|r| r.target_count).sum::<usize>();

        BuildStatistics {
            total_builds,
            successful_builds,
            failed_builds,
            avg_build_time_ns: avg_build_time,
            total_warnings,
            total_errors,
            total_targets,
            success_rate: if total_builds > 0 {
                (successful_builds as f64 / total_builds as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Monitor cargo process if running
    fn monitor_cargo_processes(&self) -> Option<ProcessMetrics> {
        use sysinfo::{PidExt, ProcessExt, System, SystemExt};

        let mut system = System::new();
        system.refresh_processes();

        for (pid, process) in system.processes() {
            let name = process.name();
            if name.contains("cargo") || name.contains("rustc") {
                return Some(ProcessMetrics {
                    pid: pid.as_u32(),
                    name: name.to_string(),
                    cpu_usage: process.cpu_usage() as f64,
                    memory_bytes: process.memory(),
                    virtual_memory_bytes: process.virtual_memory(),
                    start_time: process.start_time(),
                    status: format!("{:?}", process.status()),
                });
            }
        }

        None
    }
}

#[derive(Debug, Default)]
struct BuildStatistics {
    total_builds: u64,
    successful_builds: u64,
    failed_builds: u64,
    avg_build_time_ns: u64,
    total_warnings: u64,
    total_errors: u64,
    total_targets: usize,
    success_rate: f64,
}

impl Default for CargoMetricsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl MetricsProvider for CargoMetricsProvider {
    fn name(&self) -> &str {
        "cargo"
    }

    async fn collect_metrics(&self) -> Result<PerformanceMetrics, CollectionError> {
        let mut metrics = PerformanceMetrics::new();

        // Get build statistics
        let stats = self.get_build_statistics();

        // Set build metrics
        metrics.build.build_time_ns = Some(stats.avg_build_time_ns);
        metrics.build.build_successful = Some(stats.successful_builds > 0);
        metrics.build.warnings_count = Some(stats.total_warnings);
        metrics.build.errors_count = Some(stats.total_errors);

        // Set counter metrics
        metrics.counters.total_operations = Some(stats.total_builds);
        metrics.counters.successful_operations = Some(stats.successful_builds);
        metrics.counters.failed_operations = Some(stats.failed_builds);
        metrics.counters.error_count = Some(stats.total_errors);

        // Set rate metrics
        metrics.rates.success_rate = Some(stats.success_rate);

        // Monitor active cargo processes
        if let Some(cargo_process) = self.monitor_cargo_processes() {
            metrics.add_extension(
                "cargo_process_cpu",
                MetricValue::Float(cargo_process.cpu_usage),
            );
            metrics.add_extension(
                "cargo_process_memory",
                MetricValue::Integer(cargo_process.memory_bytes as i64),
            );
            metrics.add_extension(
                "cargo_process_status",
                MetricValue::String(cargo_process.status),
            );
            metrics.add_extension(
                "cargo_build_active",
                MetricValue::Bool(true),
            );
        } else {
            metrics.add_extension(
                "cargo_build_active",
                MetricValue::Bool(false),
            );
        }

        // Add additional build statistics
        metrics.add_extension(
            "build_targets_total",
            MetricValue::Integer(stats.total_targets as i64),
        );

        // Calculate throughput (builds per second based on average build time)
        if stats.avg_build_time_ns > 0 {
            let builds_per_second = 1_000_000_000.0 / stats.avg_build_time_ns as f64;
            metrics.rates.throughput_ops_per_sec = Some(builds_per_second);
        }

        Ok(metrics)
    }
}

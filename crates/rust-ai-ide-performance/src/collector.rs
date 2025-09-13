//! Unified Performance Metrics Collector
//!
//! This module provides the unified performance metrics collector that aggregates
//! performance data from all crates and components in the Rust AI IDE system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

use crate::alerting::AlertManager;
use crate::instrumentation::{InstrumentationConfig, PerformanceInstrumentor};
use crate::metrics::{MetricsError, MetricsRegistry, PrometheusExporter};
use crate::metrics_server::{MetricsServer, MetricsServerBuilder, MetricsServerConfig};
use crate::monitoring::SystemMonitor;
use crate::regression::RegressionDetector;
use rust_ai_ide_shared_types::PerformanceMetrics;

/// Configuration for the performance collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    /// Collection interval in seconds
    pub collection_interval_secs: u64,
    /// Maximum number of metrics to keep in memory
    pub max_history_size: usize,
    /// Enable regression detection
    pub enable_regression_detection: bool,
    /// Regression alerting threshold
    pub regression_threshold: f64,
    /// Number of data points for baseline calculation
    pub baseline_window_size: usize,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            collection_interval_secs: 30, // 30 seconds default
            max_history_size: 1000,
            enable_regression_detection: true,
            regression_threshold: 0.1, // 10% performance degradation
            baseline_window_size: 10,
        }
    }
}

/// Unified Performance Collector
pub struct UnifiedPerformanceCollector {
    /// Configuration
    config: CollectorConfig,
    /// Historical metrics data
    metrics_history: Arc<RwLock<Vec<PerformanceMetrics>>>,
    /// Registered collectors from different crates
    crate_collectors: Arc<RwLock<HashMap<String, Box<dyn MetricsProvider>>>>,
    /// Regression detector
    regression_detector: Option<RegressionDetector>,
    /// Alert manager
    alert_manager: Option<AlertManager>,
    /// Prometheus exporter
    prometheus_exporter: Option<Arc<PrometheusExporter>>,
    /// Performance instrumentor
    instrumentor: Option<Arc<PerformanceInstrumentor>>,
    /// Channel for receiving metrics from external sources
    metrics_receiver: mpsc::Receiver<PerformanceMetrics>,
    metrics_sender: mpsc::Sender<PerformanceMetrics>,
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
        metric_name: String,
        baseline_value: f64,
        current_value: f64,
        degradation_percent: f64,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    ThresholdExceeded {
        metric_name: String,
        current_value: f64,
        threshold: f64,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    AnomalyDetected {
        description: String,
        severity: AlertSeverity,
        timestamp: chrono::DateTime<chrono::Utc>,
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
    config: CollectorConfig,
    providers: HashMap<String, Box<dyn MetricsProvider>>,
}

impl CollectorBuilder {
    /// Create a new collector builder with default configuration
    pub fn new() -> Self {
        Self {
            config: CollectorConfig::default(),
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
            let mut interval = time::interval(Duration::from_secs(config.collection_interval_secs));

            loop {
                interval.tick().await;

                let mut aggregated_metrics = PerformanceMetrics::new();

                // Collect from registered providers - clone the providers to avoid lifetime issues
                {
                    let collectors = crate_collectors.read().unwrap();
                    let providers_refs: Vec<&Box<dyn MetricsProvider>> =
                        collectors.values().collect();

                    for provider in providers_refs {
                        match provider.collect_metrics().await {
                            Ok(metrics) => {
                                aggregated_metrics.merge(&metrics);
                            }
                            Err(e) => {
                                eprintln!(
                                    "Failed to collect metrics from {}: {}",
                                    provider.name(),
                                    e
                                );
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
    pub fn register_provider(
        &self,
        provider: Box<dyn MetricsProvider>,
    ) -> Result<(), anyhow::Error> {
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

/// Default metrics provider for basic system metrics
pub struct SystemMetricsProvider;

#[async_trait::async_trait]
impl MetricsProvider for SystemMetricsProvider {
    fn name(&self) -> &str {
        "system"
    }

    async fn collect_metrics(&self) -> Result<PerformanceMetrics, CollectionError> {
        let mut metrics = PerformanceMetrics::new();

        // For this basic implementation, we'll use placeholder values
        // In a real implementation, this would collect actual system metrics
        metrics.rates.cpu_usage_percent = Some(45.0);
        metrics.resources.memory_bytes = Some(500_000_000); // 500MB

        // Add some timing data
        use std::time::{Duration, Instant};
        let start = Instant::now();
        tokio::time::sleep(Duration::from_millis(1)).await;
        let elapsed = start.elapsed();

        metrics.timing.response_time_ns = Some(elapsed.as_nanos() as u64);

        Ok(metrics)
    }
}

/// Example crate-specific provider for cargo operations
pub struct CargoMetricsProvider;

#[async_trait::async_trait]
impl MetricsProvider for CargoMetricsProvider {
    fn name(&self) -> &str {
        "cargo"
    }

    async fn collect_metrics(&self) -> Result<PerformanceMetrics, CollectionError> {
        let mut metrics = PerformanceMetrics::new();

        // Placeholder cargo metrics
        metrics.counters.total_operations = Some(150);
        metrics.counters.successful_operations = Some(145);
        metrics.build.build_time_ns = Some(45_000_000_000); // 45 seconds
        metrics.build.build_successful = Some(true);

        Ok(metrics)
    }
}

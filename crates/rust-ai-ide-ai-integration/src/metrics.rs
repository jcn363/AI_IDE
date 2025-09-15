//! Metrics and Monitoring Module
//!
//! This module provides comprehensive metrics collection and monitoring capabilities
//! for the AI integration layer, enabling performance tracking and diagnostics.

use std::sync::Arc;

use tokio::sync::RwLock;

/// Main metrics collector for AI integration
pub struct MetricsCollector {
    state:      Arc<RwLock<MetricsState>>,
    collectors: std::collections::HashMap<String, Arc<dyn MetricCollector>>,
}

/// Internal metrics state
pub struct MetricsState {
    /// Global metrics configuration
    config:          MetricsConfig,
    /// Current metrics snapshot
    current_metrics: MetricsSnapshot,
    /// Historical metrics
    historical_data: Vec<HistoricalMetrics>,
    /// Active metric timers
    active_timers:   std::collections::HashMap<String, Timer>,
}

/// Metrics configuration
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled:                  bool,
    /// Collection interval in seconds
    pub collection_interval_secs: u64,
    /// Metrics retention period in hours
    pub retention_hours:          u64,
    /// Performance threshold warnings
    pub performance_thresholds:   PerformanceThresholds,
}

/// Performance threshold definitions
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    /// Maximum acceptable response time in milliseconds
    pub max_response_time_ms:   u64,
    /// Minimum success rate (0.0-1.0)
    pub min_success_rate:       f64,
    /// Maximum error rate (0.0-1.0)
    pub max_error_rate:         f64,
    /// Memory usage warning threshold (bytes)
    pub memory_threshold_bytes: u64,
}

/// Current metrics snapshot
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    /// Timestamp of this snapshot
    pub timestamp:        chrono::DateTime<chrono::Utc>,
    /// LSP bridge metrics
    pub lsp_metrics:      LspMetrics,
    /// Frontend interface metrics
    pub frontend_metrics: FrontendMetrics,
    /// Router metrics
    pub router_metrics:   RouterMetrics,
    /// System resource metrics
    pub system_metrics:   SystemMetrics,
}

/// LSP-specific metrics
#[derive(Debug, Clone)]
pub struct LspMetrics {
    /// Total AI completion requests
    pub completion_requests:      u64,
    /// Completion success rate
    pub completion_success_rate:  f64,
    /// Average completion response time
    pub avg_completion_time_ms:   f64,
    /// Diagnostics enhancement requests
    pub diagnostics_requests:     u64,
    /// Diagnostics success rate
    pub diagnostics_success_rate: f64,
}

/// Frontend-specific metrics
#[derive(Debug, Clone)]
pub struct FrontendMetrics {
    /// Total frontend requests
    pub request_count:          u64,
    /// Response delivery rate
    pub response_delivery_rate: f64,
    /// Average response time
    pub avg_response_time_ms:   f64,
    /// User feedback count
    pub feedback_count:         u64,
}

/// Router-specific metrics
#[derive(Debug, Clone)]
pub struct RouterMetrics {
    /// Total routing decisions
    pub routing_decisions:         u64,
    /// Routing success rate
    pub routing_success_rate:      f64,
    /// Load balancing efficiency
    pub load_balancing_efficiency: f64,
    /// Cache hit rate
    pub cache_hit_rate:            f64,
}

/// System resource metrics
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    /// CPU usage percentage
    pub cpu_usage_percent:      f64,
    /// Memory usage bytes
    pub memory_usage_bytes:     u64,
    /// Available memory bytes
    pub available_memory_bytes: u64,
    /// Network throughput bytes per second
    pub network_throughput_bps: u64,
}

/// Historical metrics for trend analysis
#[derive(Debug, Clone)]
pub struct HistoricalMetrics {
    /// Timestamp range
    pub timestamp_range: (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>),
    /// Metrics summary
    pub summary:         MetricsSummary,
    /// Trend indicators
    pub trends:          Vec<MetricTrend>,
}

/// Metrics summary for historical analysis
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    /// Average response time over period
    pub avg_response_time_ms: f64,
    /// Average success rate over period
    pub avg_success_rate:     f64,
    /// Total requests served
    pub total_requests:       u64,
    /// Error rate over period
    pub error_rate:           f64,
}

/// Metric trend indication
#[derive(Debug, Clone)]
pub struct MetricTrend {
    /// Metric name
    pub metric_name:       String,
    /// Trend direction
    pub direction:         TrendDirection,
    /// Trend magnitude (percentage change)
    pub magnitude_percent: f64,
    /// Confidence in trend (0.0-1.0)
    pub confidence:        f64,
}

/// Trend direction enumeration
#[derive(Debug, Clone)]
pub enum TrendDirection {
    /// Metric is improving
    Improving,
    /// Metric is degrading
    Degrading,
    /// Metric is stable
    Stable,
}

/// Performance timer for measuring operation duration
pub struct Timer {
    /// Start timestamp
    pub start_time:  std::time::Instant,
    /// Timer labels
    pub labels:      std::collections::HashMap<String, String>,
    /// Associated metric name
    pub metric_name: String,
}

/// Metrics collector trait for plugin-based collectors
pub trait MetricCollector {
    /// Collect metrics from this collector
    async fn collect(&self) -> Result<serde_json::Value, MetricsError>;

    /// Get collector name
    fn name(&self) -> &str;

    /// Get collector version
    fn version(&self) -> &str;
}

/// Metrics error type
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    /// Collection failed
    #[error("Metrics collection failed: {0}")]
    Collection(String),
    /// Storage error
    #[error("Metrics storage error: {0}")]
    Storage(String),
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
    /// Timer error
    #[error("Timer error: {0}")]
    Timer(String),
}

impl MetricsCollector {
    /// Create a new metrics collector instance
    #[must_use]
    pub fn new() -> Self {
        MetricsCollector {
            state:      Arc::new(RwLock::new(MetricsState {
                config:          MetricsConfig {
                    enabled:                  true,
                    collection_interval_secs: 60,
                    retention_hours:          24,
                    performance_thresholds:   PerformanceThresholds {
                        max_response_time_ms:   2000,
                        min_success_rate:       0.95,
                        max_error_rate:         0.05,
                        memory_threshold_bytes: 512 * 1024 * 1024, // 512 MB
                    },
                },
                current_metrics: MetricsSnapshot {
                    timestamp:        chrono::Utc::now(),
                    lsp_metrics:      LspMetrics {
                        completion_requests:      0,
                        completion_success_rate:  0.0,
                        avg_completion_time_ms:   0.0,
                        diagnostics_requests:     0,
                        diagnostics_success_rate: 0.0,
                    },
                    frontend_metrics: FrontendMetrics {
                        request_count:          0,
                        response_delivery_rate: 0.0,
                        avg_response_time_ms:   0.0,
                        feedback_count:         0,
                    },
                    router_metrics:   RouterMetrics {
                        routing_decisions:         0,
                        routing_success_rate:      0.0,
                        load_balancing_efficiency: 0.0,
                        cache_hit_rate:            0.0,
                    },
                    system_metrics:   SystemMetrics {
                        cpu_usage_percent:      0.0,
                        memory_usage_bytes:     0,
                        available_memory_bytes: 0,
                        network_throughput_bps: 0,
                    },
                },
                historical_data: Vec::new(),
                active_timers:   std::collections::HashMap::new(),
            })),
            collectors: std::collections::HashMap::new(),
        }
    }

    /// Add a metric collector
    pub fn add_collector(&mut self, collector: Arc<dyn MetricCollector>) {
        self.collectors
            .insert(collector.name().to_string(), collector);
    }

    /// Start a performance timer
    pub fn start_timer(&self, name: &str, labels: std::collections::HashMap<String, String>) -> Timer {
        Timer {
            start_time: std::time::Instant::now(),
            labels,
            metric_name: name.to_string(),
        }
    }

    /// Stop a timer and record the metric
    pub async fn stop_timer(&self, timer: Timer) -> Result<(), MetricsError> {
        let duration = timer.start_time.elapsed().as_millis() as f64;

        let mut state = self.state.write().await;

        // Update relevant metrics based on timer type
        match timer.metric_name.as_str() {
            "lsp_completion" => {
                state.current_metrics.lsp_metrics.completion_requests += 1;
                state.current_metrics.lsp_metrics.avg_completion_time_ms =
                    (state.current_metrics.lsp_metrics.avg_completion_time_ms
                        * (state.current_metrics.lsp_metrics.completion_requests - 1) as f64
                        + duration)
                        / state.current_metrics.lsp_metrics.completion_requests as f64;
            }
            "frontend_response" => {
                state.current_metrics.frontend_metrics.request_count += 1;
                state.current_metrics.frontend_metrics.avg_response_time_ms =
                    (state.current_metrics.frontend_metrics.avg_response_time_ms
                        * (state.current_metrics.frontend_metrics.request_count - 1) as f64
                        + duration)
                        / state.current_metrics.frontend_metrics.request_count as f64;
            }
            _ => {
                // Generic timer recording
            }
        }

        Ok(())
    }

    /// Get current metrics snapshot
    pub async fn get_snapshot(&self) -> Result<MetricsSnapshot, MetricsError> {
        let state = self.state.read().await;
        Ok(state.current_metrics.clone())
    }

    /// Get performance alerts based on thresholds
    pub async fn get_alerts(&self) -> Result<Vec<PerformanceAlert>, MetricsError> {
        let state = self.state.read().await;
        let mut alerts = Vec::new();

        let thresholds = &state.config.performance_thresholds;
        let metrics = &state.current_metrics;

        if metrics.lsp_metrics.avg_completion_time_ms > thresholds.max_response_time_ms as f64 {
            alerts.push(PerformanceAlert {
                alert_type:         AlertType::HighResponseTime,
                severity:           AlertSeverity::Warning,
                message:            format!(
                    "LSP completion response time ({:.2}ms) exceeds threshold ({}.0ms)",
                    metrics.lsp_metrics.avg_completion_time_ms, thresholds.max_response_time_ms
                ),
                affected_component: "lsp_bridge".to_string(),
                recommendation:     "Consider optimizing AI model inference or using load balancing".to_string(),
            });
        }

        if metrics.lsp_metrics.completion_success_rate < thresholds.min_success_rate {
            alerts.push(PerformanceAlert {
                alert_type:         AlertType::LowSuccessRate,
                severity:           AlertSeverity::Critical,
                message:            format!(
                    "LSP completion success rate ({:.2}%) below threshold ({:.0}%)",
                    metrics.lsp_metrics.completion_success_rate * 100.0,
                    thresholds.min_success_rate * 100.0
                ),
                affected_component: "lsp_bridge".to_string(),
                recommendation:     "Investigate model availability and error handling".to_string(),
            });
        }

        Ok(alerts)
    }

    /// Collect all metrics from registered collectors
    pub async fn collect_all_metrics(&self) -> Result<(), MetricsError> {
        let mut state = self.state.write().await;

        // Collect from all registered collectors
        for (name, collector) in &self.collectors {
            match collector.collect().await {
                Ok(metrics) => {
                    // Store collected metrics
                    // In production, this would store in time-series database
                    tracing::info!("Collected metrics from {}: {:?}", name, metrics);
                }
                Err(e) => {
                    tracing::warn!("Failed to collect metrics from {}: {:?}", name, e);
                }
            }
        }

        // Update current metrics timestamp
        state.current_metrics.timestamp = chrono::Utc::now();

        Ok(())
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance alert definition
#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    /// Type of alert
    pub alert_type:         AlertType,
    /// Severity level
    pub severity:           AlertSeverity,
    /// Alert message
    pub message:            String,
    /// Affected component
    pub affected_component: String,
    /// Recommended action
    pub recommendation:     String,
}

/// Alert type enumeration
#[derive(Debug, Clone)]
pub enum AlertType {
    /// High response time
    HighResponseTime,
    /// Low success rate
    LowSuccessRate,
    /// High error rate
    HighErrorRate,
    /// High memory usage
    HighMemoryUsage,
    /// Low cache hit rate
    LowCacheHitRate,
}

/// Alert severity enumeration
#[derive(Debug, Clone)]
pub enum AlertSeverity {
    /// Information level
    Info,
    /// Warning level
    Warning,
    /// Error level
    Error,
    /// Critical level
    Critical,
}

// Removed unused imports: IDEResult, IDEError (not used in this file)
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Types of metrics that can be collected
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MetricType {
    /// Performance metrics
    Performance,
    /// Memory usage metrics
    Memory,
    /// CPU usage metrics
    Cpu,
    /// Network I/O metrics
    NetworkIO,
    /// Custom metrics
    Custom(String),
}

/// A single metric measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    /// Metric name
    pub name: String,
    /// Metric value
    pub value: MetricData,
    /// Timestamp when measured
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional tags/metadata
    pub tags: HashMap<String, String>,
}

/// Data types for metric values
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MetricData {
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// Boolean value
    Boolean(bool),
    /// Duration in milliseconds
    Duration(u64),
}

/// Metric series for tracking over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSeries {
    /// Metric name
    pub name: String,
    /// Type of metric
    pub metric_type: MetricType,
    /// Series of values over time
    pub values: Vec<MetricValue>,
    /// Aggregation method
    pub aggregation: Option<AggregationMethod>,
}

/// Aggregation methods for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationMethod {
    /// Sum of all values
    Sum,
    /// Average of values
    Average,
    /// Minimum value
    Min,
    /// Maximum value
    Max,
    /// Count of values
    Count,
}

/// Metric threshold for alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricThreshold {
    /// Metric name
    pub name: String,
    /// Threshold value
    pub threshold: f64,
    /// Comparison operator
    pub operator: ThresholdOperator,
    /// Severity level for violations
    pub severity: ThresholdSeverity,
}

/// Operators for threshold comparisons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdOperator {
    /// Value must be greater than threshold
    GreaterThan,
    /// Value must be less than threshold
    LessThan,
    /// Value must be equal to threshold
    Equal,
    /// Value must be greater than or equal to threshold
    GreaterEqual,
    /// Value must be less than or equal to threshold
    LessEqual,
}

/// Severity levels for threshold violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ThresholdSeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Performance timer for measuring operation durations
#[derive(Debug)]
pub struct PerformanceTimer {
    /// Timer name
    name: String,
    /// Start time
    start: Instant,
    /// Additional tags
    tags: HashMap<String, String>,
}

impl PerformanceTimer {
    /// Create a new performance timer
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            tags: HashMap::new(),
        }
    }

    /// Add a tag to the timer
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    /// Stop the timer and return elapsed duration
    pub fn stop(self) -> MetricValue {
        let duration = self.start.elapsed();

        MetricValue {
            name: self.name,
            value: MetricData::Duration(duration.as_millis() as u64),
            timestamp: chrono::Utc::now(),
            tags: self.tags,
        }
    }

    /// Get elapsed duration without stopping the timer
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

/// Metric collector for aggregating measurements
pub struct MetricCollector {
    /// Collected metrics
    pub(crate) metrics: std::sync::Arc<std::sync::Mutex<Vec<MetricValue>>>,
    /// Configured thresholds
    pub(crate) thresholds: Vec<MetricThreshold>,
}

impl Default for MetricCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricCollector {
    /// Create a new metric collector
    pub fn new() -> Self {
        Self {
            metrics: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            thresholds: Vec::new(),
        }
    }

    /// Record a metric value
    pub fn record(&self, mut value: MetricValue) {
        if value.timestamp == chrono::DateTime::<chrono::Utc>::MIN_UTC {
            value.timestamp = chrono::Utc::now();
        }
        self.metrics.lock().unwrap().push(value);
    }

    /// Record a simple metric
    pub fn record_simple(&self, name: impl Into<String>, value: MetricData) {
        let metric = MetricValue {
            name: name.into(),
            value,
            timestamp: chrono::Utc::now(),
            tags: HashMap::new(),
        };
        self.record(metric);
    }

    /// Get all recorded metrics
    pub fn get_metrics(&self) -> Vec<MetricValue> {
        self.metrics.lock().unwrap().clone()
    }

    /// Clear all metrics
    pub fn clear(&self) {
        self.metrics.lock().unwrap().clear();
    }

    /// Check if any metrics violate thresholds
    pub fn check_thresholds(&self) -> Vec<ThresholdViolation> {
        let metrics = self.get_metrics();
        let mut violations = Vec::new();

        for threshold in &self.thresholds {
            for metric in &metrics {
                if metric.name == threshold.name {
                    let value = match &metric.value {
                        MetricData::Integer(i) => *i as f64,
                        MetricData::Float(f) => *f,
                        _ => continue,
                    };

                    let violated = match threshold.operator {
                        ThresholdOperator::GreaterThan => value > threshold.threshold,
                        ThresholdOperator::LessThan => value < threshold.threshold,
                        ThresholdOperator::Equal => {
                            (value - threshold.threshold).abs() < f64::EPSILON
                        }
                        ThresholdOperator::GreaterEqual => value >= threshold.threshold,
                        ThresholdOperator::LessEqual => value <= threshold.threshold,
                    };

                    if violated {
                        violations.push(ThresholdViolation {
                            threshold: threshold.clone(),
                            actual_value: value,
                            metric: metric.clone(),
                        });
                    }
                }
            }
        }

        violations
    }

    /// Add a threshold
    pub fn add_threshold(&mut self, threshold: MetricThreshold) {
        self.thresholds.push(threshold);
    }
}

/// Threshold violation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdViolation {
    /// The threshold that was violated
    pub threshold: MetricThreshold,
    /// The actual measured value
    pub actual_value: f64,
    /// The metric that violated the threshold
    pub metric: MetricValue,
}

/// Get or create the global metric collector
///
/// Returns a singleton instance of MetricCollector that can be shared across the application.
/// This function ensures thread-safe access to the global collector instance.
pub fn global_collector() -> std::sync::Arc<MetricCollector> {
    static INIT: std::sync::OnceLock<std::sync::Arc<MetricCollector>> = std::sync::OnceLock::new();
    INIT.get_or_init(|| std::sync::Arc::new(MetricCollector::new()))
        .clone()
}

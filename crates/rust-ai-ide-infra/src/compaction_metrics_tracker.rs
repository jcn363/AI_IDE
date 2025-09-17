//! Compaction metrics tracker for monitoring and performance analysis
//!
//! This module provides comprehensive tracking and reporting for large workspace
//! compaction operations, including performance metrics, success rates, and trend analysis.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use crate::InfraResult;

/// Comprehensive metrics tracker for compaction operations
#[derive(Debug)]
pub struct CompactionMetricsTracker {
    /// Configuration
    config: MetricsConfig,

    /// Metrics storage
    metrics: Arc<RwLock<MetricsStorage>>,

    /// Performance analyzer
    analyzer: Arc<PerformanceAnalyzer>,

    /// Trend analyzer
    trend_analyzer: Arc<TrendAnalyzer>,
}

/// Configuration for metrics tracking
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Metrics retention period (seconds)
    pub retention_period_seconds: u64,

    /// Maximum metrics entries
    pub max_entries: usize,

    /// Enable detailed tracking
    pub detailed_tracking: bool,

    /// Performance threshold for alerts
    pub performance_threshold: f64,

    /// Trend analysis window (entries)
    pub trend_window: usize,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            retention_period_seconds: 3600 * 24 * 7, // 7 days
            max_entries: 10000,
            detailed_tracking: true,
            performance_threshold: 0.8,
            trend_window: 100,
        }
    }
}

/// Storage for compaction metrics
#[derive(Debug)]
struct MetricsStorage {
    /// Individual compaction records
    compaction_records: Vec<CompactionRecord>,

    /// Aggregated statistics
    aggregated_stats: AggregatedStatistics,

    /// Performance baselines
    baselines: PerformanceBaselines,

    /// Alert history
    alerts: Vec<AlertRecord>,
}

/// Performance analyzer for metrics
#[derive(Debug)]
struct PerformanceAnalyzer {
    /// Current analysis state
    state: Arc<RwLock<AnalysisState>>,
}

/// Trend analyzer for long-term patterns
#[derive(Debug)]
struct TrendAnalyzer {
    /// Trend data points
    trend_data: Arc<RwLock<Vec<TrendDataPoint>>>,
}

/// Compaction record for individual operations
#[derive(Debug, Clone)]
pub struct CompactionRecord {
    /// Record timestamp
    pub timestamp: Instant,

    /// Operation ID
    pub operation_id: String,

    /// Strategy used
    pub strategy: super::large_workspace_compactor::CompactionStrategy,

    /// Duration of operation
    pub duration: Duration,

    /// Bytes processed
    pub bytes_processed: usize,

    /// Bytes freed
    pub bytes_freed: usize,

    /// Fragmentation before
    pub fragmentation_before: f64,

    /// Fragmentation after
    pub fragmentation_after: f64,

    /// CPU usage during operation
    pub cpu_usage: f64,

    /// Memory pressure during operation
    pub memory_pressure: f64,

    /// Success status
    pub success: bool,

    /// Error message if failed
    pub error_message: Option<String>,

    /// Performance score (0.0-1.0)
    pub performance_score: f64,

    /// System impact level
    pub system_impact: SystemImpact,
}

/// Aggregated statistics across all operations
#[derive(Debug, Clone)]
pub struct AggregatedStatistics {
    /// Total operations
    pub total_operations: usize,

    /// Successful operations
    pub successful_operations: usize,

    /// Failed operations
    pub failed_operations: usize,

    /// Average operation duration
    pub avg_duration: Duration,

    /// Average bytes processed per operation
    pub avg_bytes_processed: usize,

    /// Average bytes freed per operation
    pub avg_bytes_freed: usize,

    /// Average fragmentation reduction
    pub avg_fragmentation_reduction: f64,

    /// Success rate
    pub success_rate: f64,

    /// Efficiency ratio (bytes freed / bytes processed)
    pub efficiency_ratio: f64,

    /// Average performance score
    pub avg_performance_score: f64,
}

/// Performance baselines for comparison
#[derive(Debug, Clone)]
struct PerformanceBaselines {
    /// Baseline operation duration
    baseline_duration: Duration,

    /// Baseline efficiency ratio
    baseline_efficiency: f64,

    /// Baseline fragmentation reduction
    baseline_reduction: f64,

    /// Last updated timestamp
    last_updated: Instant,
}

/// Alert record for performance issues
#[derive(Debug, Clone)]
struct AlertRecord {
    /// Alert timestamp
    timestamp: Instant,

    /// Alert type
    alert_type: AlertType,

    /// Severity level
    severity: AlertSeverity,

    /// Alert message
    message: String,

    /// Related operation ID
    operation_id: Option<String>,

    /// Recommended action
    recommended_action: String,
}

/// Analysis state for performance monitoring
#[derive(Debug)]
struct AnalysisState {
    /// Current analysis window
    current_window: Vec<CompactionRecord>,

    /// Performance trends
    performance_trends: PerformanceTrends,

    /// Anomaly detection state
    anomaly_state: AnomalyDetectionState,
}

/// Trend data point for long-term analysis
#[derive(Debug, Clone)]
struct TrendDataPoint {
    /// Timestamp
    timestamp: Instant,

    /// Metric value
    value: f64,

    /// Metric type
    metric_type: MetricType,

    /// Confidence level
    confidence: f64,
}

/// Performance trends analysis
#[derive(Debug, Clone)]
struct PerformanceTrends {
    /// Duration trend (seconds per operation)
    duration_trend: TrendDirection,

    /// Efficiency trend
    efficiency_trend: TrendDirection,

    /// Success rate trend
    success_trend: TrendDirection,

    /// System impact trend
    impact_trend: TrendDirection,
}

/// Anomaly detection state
#[derive(Debug, Clone)]
struct AnomalyDetectionState {
    /// Expected performance range
    expected_range: PerformanceRange,

    /// Recent anomalies
    recent_anomalies: Vec<AnomalyRecord>,

    /// Detection sensitivity
    sensitivity: f64,
}

/// Trend direction enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendDirection {
    /// Improving trend
    Improving,

    /// Stable trend
    Stable,

    /// Degrading trend
    Degrading,

    /// Volatile trend
    Volatile,
}

/// System impact level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemImpact {
    /// Minimal impact
    Minimal,

    /// Low impact
    Low,

    /// Moderate impact
    Moderate,

    /// High impact
    High,

    /// Critical impact
    Critical,
}

/// Alert type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlertType {
    /// Performance degradation
    PerformanceDegradation,

    /// High failure rate
    HighFailureRate,

    /// Memory inefficiency
    MemoryInefficiency,

    /// System impact warning
    SystemImpactWarning,

    /// Anomaly detected
    AnomalyDetected,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlertSeverity {
    /// Low severity
    Low,

    /// Medium severity
    Medium,

    /// High severity
    High,

    /// Critical severity
    Critical,
}

/// Metric type for trend analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MetricType {
    /// Operation duration
    Duration,

    /// Efficiency ratio
    Efficiency,

    /// Success rate
    SuccessRate,

    /// Fragmentation reduction
    FragmentationReduction,

    /// System impact
    SystemImpact,
}

/// Performance range for anomaly detection
#[derive(Debug, Clone)]
struct PerformanceRange {
    /// Minimum expected value
    min_value: f64,

    /// Maximum expected value
    max_value: f64,

    /// Metric type
    metric_type: MetricType,
}

/// Anomaly record
#[derive(Debug, Clone)]
struct AnomalyRecord {
    /// Timestamp
    timestamp: Instant,

    /// Anomaly value
    value: f64,

    /// Expected value
    expected_value: f64,

    /// Deviation magnitude
    deviation: f64,

    /// Metric type
    metric_type: MetricType,
}

impl CompactionMetricsTracker {
    /// Create a new metrics tracker
    pub fn new() -> Self {
        Self {
            config: MetricsConfig::default(),
            metrics: Arc::new(RwLock::new(MetricsStorage {
                compaction_records: Vec::new(),
                aggregated_stats: AggregatedStatistics::default(),
                baselines: PerformanceBaselines {
                    baseline_duration: Duration::from_millis(100),
                    baseline_efficiency: 0.5,
                    baseline_reduction: 0.2,
                    last_updated: Instant::now(),
                },
                alerts: Vec::new(),
            })),
            analyzer: Arc::new(PerformanceAnalyzer {
                state: Arc::new(RwLock::new(AnalysisState {
                    current_window: Vec::new(),
                    performance_trends: PerformanceTrends {
                        duration_trend: TrendDirection::Stable,
                        efficiency_trend: TrendDirection::Stable,
                        success_trend: TrendDirection::Stable,
                        impact_trend: TrendDirection::Stable,
                    },
                    anomaly_state: AnomalyDetectionState {
                        expected_range: PerformanceRange {
                            min_value: 0.0,
                            max_value: 1.0,
                            metric_type: MetricType::Efficiency,
                        },
                        recent_anomalies: Vec::new(),
                        sensitivity: 0.7,
                    },
                })),
            }),
            trend_analyzer: Arc::new(TrendAnalyzer {
                trend_data: Arc::new(RwLock::new(Vec::new())),
            }),
        }
    }

    /// Record a compaction operation
    pub async fn record_compaction(&self, result: super::large_workspace_compactor::CompactionResult) {
        let record = CompactionRecord {
            timestamp: Instant::now(),
            operation_id: result.operation_id.clone(),
            strategy: result.strategy,
            duration: result.duration,
            bytes_processed: result.bytes_processed,
            bytes_freed: result.bytes_freed,
            fragmentation_before: result.fragmentation_before,
            fragmentation_after: result.fragmentation_after,
            cpu_usage: 0.5, // Placeholder - would be measured
            memory_pressure: 0.6, // Placeholder - would be measured
            success: result.success,
            error_message: None,
            performance_score: self.calculate_performance_score(&result),
            system_impact: self.assess_system_impact(&result),
        };

        // Add to storage
        {
            let mut metrics = self.metrics.write().await;
            metrics.compaction_records.push(record.clone());

            // Maintain size limits
            self.cleanup_old_records(&mut metrics).await;

            // Update aggregated statistics
            self.update_aggregated_stats(&mut metrics, &record).await;
        }

        // Analyze performance
        self.analyzer.analyze_record(&record).await;

        // Update trends
        self.trend_analyzer.update_trends(&record).await;

        // Check for alerts
        self.check_for_alerts(&record).await;
    }

    /// Get current metrics summary
    pub async fn get_metrics_summary(&self) -> MetricsSummary {
        let metrics = self.metrics.read().await;
        let analysis_state = self.analyzer.state.read().await;

        MetricsSummary {
            total_operations: metrics.aggregated_stats.total_operations,
            success_rate: metrics.aggregated_stats.success_rate,
            avg_duration: metrics.aggregated_stats.avg_duration,
            avg_efficiency: metrics.aggregated_stats.efficiency_ratio,
            avg_fragmentation_reduction: metrics.aggregated_stats.avg_fragmentation_reduction,
            current_trends: analysis_state.performance_trends.clone(),
            active_alerts: metrics.alerts.len(),
            last_operation_timestamp: metrics.compaction_records.last().map(|r| r.timestamp),
        }
    }

    /// Get detailed metrics for a time range
    pub async fn get_detailed_metrics(&self, start_time: Instant, end_time: Instant) -> Vec<CompactionRecord> {
        let metrics = self.metrics.read().await;

        metrics.compaction_records.iter()
            .filter(|record| record.timestamp >= start_time && record.timestamp <= end_time)
            .cloned()
            .collect()
    }

    /// Get performance trends
    pub async fn get_performance_trends(&self) -> PerformanceTrends {
        let analysis_state = self.analyzer.state.read().await;
        analysis_state.performance_trends.clone()
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<AlertRecord> {
        let metrics = self.metrics.read().await;
        metrics.alerts.clone()
    }

    /// Calculate performance score for a compaction result
    fn calculate_performance_score(&self, result: &super::large_workspace_compactor::CompactionResult) -> f64 {
        if !result.success {
            return 0.0;
        }

        let duration_score = if result.duration < Duration::from_millis(100) {
            1.0
        } else if result.duration < Duration::from_millis(1000) {
            0.8
        } else {
            0.5
        };

        let efficiency_score = if result.bytes_processed > 0 {
            (result.bytes_freed as f64 / result.bytes_processed as f64).min(1.0)
        } else {
            0.0
        };

        let fragmentation_score = (result.fragmentation_before - result.fragmentation_after).max(0.0).min(1.0);

        // Weighted combination
        (duration_score * 0.4) + (efficiency_score * 0.4) + (fragmentation_score * 0.2)
    }

    /// Assess system impact of a compaction operation
    fn assess_system_impact(&self, result: &super::large_workspace_compactor::CompactionResult) -> SystemImpact {
        let duration_ms = result.duration.as_millis() as f64;

        if duration_ms < 50.0 {
            SystemImpact::Minimal
        } else if duration_ms < 200.0 {
            SystemImpact::Low
        } else if duration_ms < 500.0 {
            SystemImpact::Moderate
        } else if duration_ms < 1000.0 {
            SystemImpact::High
        } else {
            SystemImpact::Critical
        }
    }

    /// Cleanup old records to maintain size limits
    async fn cleanup_old_records(&self, metrics: &mut MetricsStorage) {
        let cutoff_time = Instant::now() - Duration::from_secs(self.config.retention_period_seconds);

        // Remove old records
        metrics.compaction_records.retain(|record| record.timestamp > cutoff_time);

        // Maintain maximum entries
        if metrics.compaction_records.len() > self.config.max_entries {
            let remove_count = metrics.compaction_records.len() - self.config.max_entries;
            metrics.compaction_records.drain(0..remove_count);
        }
    }

    /// Update aggregated statistics
    async fn update_aggregated_stats(&self, metrics: &mut MetricsStorage, record: &CompactionRecord) {
        metrics.aggregated_stats.total_operations += 1;

        if record.success {
            metrics.aggregated_stats.successful_operations += 1;
        } else {
            metrics.aggregated_stats.failed_operations += 1;
        }

        // Update averages
        let total_ops = metrics.aggregated_stats.total_operations as f64;

        // Duration average
        let current_avg_duration = metrics.aggregated_stats.avg_duration.as_millis() as f64;
        let new_avg_duration = (current_avg_duration * (total_ops - 1.0) + record.duration.as_millis() as f64) / total_ops;
        metrics.aggregated_stats.avg_duration = Duration::from_millis(new_avg_duration as u64);

        // Bytes processed average
        let current_avg_processed = metrics.aggregated_stats.avg_bytes_processed as f64;
        let new_avg_processed = (current_avg_processed * (total_ops - 1.0) + record.bytes_processed as f64) / total_ops;
        metrics.aggregated_stats.avg_bytes_processed = new_avg_processed as usize;

        // Bytes freed average
        let current_avg_freed = metrics.aggregated_stats.avg_bytes_freed as f64;
        let new_avg_freed = (current_avg_freed * (total_ops - 1.0) + record.bytes_freed as f64) / total_ops;
        metrics.aggregated_stats.avg_bytes_freed = new_avg_freed as usize;

        // Fragmentation reduction average
        let frag_reduction = record.fragmentation_before - record.fragmentation_after;
        let current_avg_frag = metrics.aggregated_stats.avg_fragmentation_reduction;
        let new_avg_frag = (current_avg_frag * (total_ops - 1.0) + frag_reduction) / total_ops;
        metrics.aggregated_stats.avg_fragmentation_reduction = new_avg_frag;

        // Success rate
        metrics.aggregated_stats.success_rate = metrics.aggregated_stats.successful_operations as f64 / total_ops;

        // Efficiency ratio
        if record.bytes_processed > 0 {
            let efficiency = record.bytes_freed as f64 / record.bytes_processed as f64;
            let current_efficiency = metrics.aggregated_stats.efficiency_ratio;
            metrics.aggregated_stats.efficiency_ratio = (current_efficiency * (total_ops - 1.0) + efficiency) / total_ops;
        }

        // Performance score average
        let current_perf_score = metrics.aggregated_stats.avg_performance_score;
        metrics.aggregated_stats.avg_performance_score = (current_perf_score * (total_ops - 1.0) + record.performance_score) / total_ops;
    }

    /// Check for performance alerts
    async fn check_for_alerts(&self, record: &CompactionRecord) {
        let mut alerts = Vec::new();

        // Check performance degradation
        if record.performance_score < self.config.performance_threshold {
            alerts.push(AlertRecord {
                timestamp: Instant::now(),
                alert_type: AlertType::PerformanceDegradation,
                severity: AlertSeverity::Medium,
                message: format!("Performance score {:.2} below threshold {:.2}", record.performance_score, self.config.performance_threshold),
                operation_id: Some(record.operation_id.clone()),
                recommended_action: "Review compaction strategy and system resources".to_string(),
            });
        }

        // Check high system impact
        if matches!(record.system_impact, SystemImpact::High | SystemImpact::Critical) {
            alerts.push(AlertRecord {
                timestamp: Instant::now(),
                alert_type: AlertType::SystemImpactWarning,
                severity: AlertSeverity::High,
                message: format!("High system impact detected: {:?}", record.system_impact),
                operation_id: Some(record.operation_id.clone()),
                recommended_action: "Consider reducing compaction frequency or aggressiveness".to_string(),
            });
        }

        // Add alerts to storage
        if !alerts.is_empty() {
            let mut metrics = self.metrics.write().await;
            metrics.alerts.extend(alerts);
        }
    }

    /// Clean up old metrics data
    pub async fn cleanup_old_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        self.cleanup_old_records(&mut metrics).await;

        // Clean up old alerts (keep last 100)
        if metrics.alerts.len() > 100 {
            metrics.alerts.drain(0..(metrics.alerts.len() - 100));
        }
    }

    /// Export metrics data for monitoring
    pub async fn export_metrics(&self) -> serde_json::Value {
        let metrics = self.metrics.read().await;
        let summary = self.get_metrics_summary().await;

        serde_json::json!({
            "summary": {
                "total_operations": summary.total_operations,
                "success_rate": summary.success_rate,
                "avg_duration_ms": summary.avg_duration.as_millis(),
                "avg_efficiency": summary.avg_efficiency,
                "avg_fragmentation_reduction": summary.avg_fragmentation_reduction,
                "active_alerts": summary.active_alerts
            },
            "aggregated_stats": {
                "successful_operations": metrics.aggregated_stats.successful_operations,
                "failed_operations": metrics.aggregated_stats.failed_operations,
                "avg_bytes_processed": metrics.aggregated_stats.avg_bytes_processed,
                "avg_bytes_freed": metrics.aggregated_stats.avg_bytes_freed,
                "avg_performance_score": metrics.aggregated_stats.avg_performance_score
            },
            "recent_records": metrics.compaction_records.iter().rev().take(10).map(|r| {
                serde_json::json!({
                    "timestamp_seconds_ago": r.timestamp.elapsed().as_secs(),
                    "operation_id": r.operation_id,
                    "strategy": format!("{:?}", r.strategy),
                    "duration_ms": r.duration.as_millis(),
                    "bytes_freed": r.bytes_freed,
                    "fragmentation_before": r.fragmentation_before,
                    "fragmentation_after": r.fragmentation_after,
                    "success": r.success,
                    "performance_score": r.performance_score
                })
            }).collect::<Vec<_>>(),
            "alerts": metrics.alerts.iter().rev().take(5).map(|a| {
                serde_json::json!({
                    "timestamp_seconds_ago": a.timestamp.elapsed().as_secs(),
                    "type": format!("{:?}", a.alert_type),
                    "severity": format!("{:?}", a.severity),
                    "message": a.message,
                    "recommended_action": a.recommended_action
                })
            }).collect::<Vec<_>>(),
            "config": {
                "retention_period_seconds": self.config.retention_period_seconds,
                "max_entries": self.config.max_entries,
                "detailed_tracking": self.config.detailed_tracking,
                "performance_threshold": self.config.performance_threshold
            }
        })
    }
}

impl Default for AggregatedStatistics {
    fn default() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            avg_duration: Duration::from_millis(0),
            avg_bytes_processed: 0,
            avg_bytes_freed: 0,
            avg_fragmentation_reduction: 0.0,
            success_rate: 0.0,
            efficiency_ratio: 0.0,
            avg_performance_score: 0.0,
        }
    }
}

/// Summary of metrics for quick overview
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    /// Total operations performed
    pub total_operations: usize,

    /// Success rate (0.0-1.0)
    pub success_rate: f64,

    /// Average operation duration
    pub avg_duration: Duration,

    /// Average efficiency ratio
    pub avg_efficiency: f64,

    /// Average fragmentation reduction
    pub avg_fragmentation_reduction: f64,

    /// Current performance trends
    pub current_trends: PerformanceTrends,

    /// Number of active alerts
    pub active_alerts: usize,

    /// Last operation timestamp
    pub last_operation_timestamp: Option<Instant>,
}

impl PerformanceAnalyzer {
    /// Analyze a new compaction record
    async fn analyze_record(&self, record: &CompactionRecord) {
        let mut state = self.state.write().await;

        // Add to current window
        state.current_window.push(record.clone());

        // Maintain window size
        if state.current_window.len() > 50 {
            state.current_window.remove(0);
        }

        // Update trends
        self.update_trends(&mut state).await;

        // Check for anomalies
        self.detect_anomalies(&mut state, record).await;
    }

    /// Update performance trends
    async fn update_trends(&self, state: &mut AnalysisState) {
        if state.current_window.len() < 5 {
            return; // Need minimum data for trend analysis
        }

        let recent = &state.current_window[state.current_window.len().saturating_sub(10)..];

        // Analyze duration trend
        state.performance_trends.duration_trend = self.analyze_trend(
            recent.iter().map(|r| r.duration.as_millis() as f64).collect()
        );

        // Analyze efficiency trend
        let efficiencies: Vec<f64> = recent.iter()
            .filter_map(|r| {
                if r.bytes_processed > 0 {
                    Some(r.bytes_freed as f64 / r.bytes_processed as f64)
                } else {
                    None
                }
            })
            .collect();
        state.performance_trends.efficiency_trend = self.analyze_trend(efficiencies);

        // Analyze success rate trend
        let success_rates: Vec<f64> = recent.iter().map(|r| if r.success { 1.0 } else { 0.0 }).collect();
        state.performance_trends.success_trend = self.analyze_trend(success_rates);

        // Analyze system impact trend
        let impacts: Vec<f64> = recent.iter().map(|r| match r.system_impact {
            SystemImpact::Minimal => 0.0,
            SystemImpact::Low => 0.25,
            SystemImpact::Moderate => 0.5,
            SystemImpact::High => 0.75,
            SystemImpact::Critical => 1.0,
        }).collect();
        state.performance_trends.impact_trend = self.analyze_trend(impacts);
    }

    /// Analyze trend direction from data points
    fn analyze_trend(&self, values: Vec<f64>) -> TrendDirection {
        if values.len() < 3 {
            return TrendDirection::Stable;
        }

        let first_half = &values[0..values.len() / 2];
        let second_half = &values[values.len() / 2..];

        let first_avg = first_half.iter().sum::<f64>() / first_half.len() as f64;
        let second_avg = second_half.iter().sum::<f64>() / second_half.len() as f64;

        let diff = second_avg - first_avg;
        let threshold = (first_avg * 0.1).abs(); // 10% change threshold

        if diff > threshold {
            TrendDirection::Degrading
        } else if diff < -threshold {
            TrendDirection::Improving
        } else {
            TrendDirection::Stable
        }
    }

    /// Detect performance anomalies
    async fn detect_anomalies(&self, state: &mut AnalysisState, record: &CompactionRecord) {
        // Simple anomaly detection based on deviation from expected range
        let expected_min = state.anomaly_state.expected_range.min_value;
        let expected_max = state.anomaly_state.expected_range.max_value;

        if record.performance_score < expected_min || record.performance_score > expected_max {
            let anomaly = AnomalyRecord {
                timestamp: Instant::now(),
                value: record.performance_score,
                expected_value: (expected_min + expected_max) / 2.0,
                deviation: (record.performance_score - (expected_min + expected_max) / 2.0).abs(),
                metric_type: MetricType::SuccessRate,
            };

            state.anomaly_state.recent_anomalies.push(anomaly);

            // Keep only recent anomalies
            if state.anomaly_state.recent_anomalies.len() > 10 {
                state.anomaly_state.recent_anomalies.remove(0);
            }
        }
    }
}

impl TrendAnalyzer {
    /// Update trend data with new record
    async fn update_trends(&self, record: &CompactionRecord) {
        let mut trend_data = self.trend_data.write().await;

        // Add data points for different metrics
        trend_data.push(TrendDataPoint {
            timestamp: record.timestamp,
            value: record.duration.as_millis() as f64,
            metric_type: MetricType::Duration,
            confidence: 0.8,
        });

        if record.bytes_processed > 0 {
            trend_data.push(TrendDataPoint {
                timestamp: record.timestamp,
                value: record.bytes_freed as f64 / record.bytes_processed as f64,
                metric_type: MetricType::Efficiency,
                confidence: 0.8,
            });
        }

        trend_data.push(TrendDataPoint {
            timestamp: record.timestamp,
            value: if record.success { 1.0 } else { 0.0 },
            metric_type: MetricType::SuccessRate,
            confidence: 0.8,
        });

        trend_data.push(TrendDataPoint {
            timestamp: record.timestamp,
            value: record.fragmentation_before - record.fragmentation_after,
            metric_type: MetricType::FragmentationReduction,
            confidence: 0.8,
        });

        // Maintain trend data size
        if trend_data.len() > 1000 {
            trend_data.drain(0..100);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_tracker_creation() {
        let tracker = CompactionMetricsTracker::new();

        let summary = tracker.get_metrics_summary().await;
        assert_eq!(summary.total_operations, 0);
        assert_eq!(summary.active_alerts, 0);
    }

    #[tokio::test]
    async fn test_compaction_recording() {
        let tracker = CompactionMetricsTracker::new();

        let result = super::large_workspace_compactor::CompactionResult {
            operation_id: "test_op".to_string(),
            bytes_processed: 1024,
            bytes_freed: 512,
            fragmentation_before: 0.5,
            fragmentation_after: 0.25,
            duration: Duration::from_millis(100),
            success: true,
            strategy: super::large_workspace_compactor::CompactionStrategy::Incremental,
        };

        tracker.record_compaction(result).await;

        let summary = tracker.get_metrics_summary().await;
        assert_eq!(summary.total_operations, 1);
        assert_eq!(summary.success_rate, 1.0);
    }
}
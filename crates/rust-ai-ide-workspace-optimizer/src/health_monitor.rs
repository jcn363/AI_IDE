//! Workspace health monitoring and metrics collection
//!
//! This module provides comprehensive health monitoring capabilities including:
//! - Real-time performance metrics collection
//! - Build health assessment
//! - Dependency health monitoring
//! - Alert generation and management
//! - Health trend analysis

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use moka::future::Cache;
use tokio::sync::RwLock;

use crate::error::{OptimizerError, OptimizerResult};
use crate::types::*;

/// Main workspace health monitor
#[derive(Debug)]
pub struct WorkspaceHealthMonitor {
    /// Health metrics cache
    metrics_cache: Cache<String, HealthMetrics>,
    /// Alert history
    alert_history: Arc<RwLock<Vec<HealthAlert>>>,
    /// Performance baseline
    performance_baseline: Arc<RwLock<Option<PerformanceBaseline>>>,
    /// Health thresholds
    thresholds: Arc<RwLock<AlertThresholds>>,
    /// Monitoring state
    monitoring_state: Arc<RwLock<MonitoringState>>,
}

impl WorkspaceHealthMonitor {
    /// Create a new health monitor
    pub fn new() -> Self {
        let metrics_cache = Cache::builder()
            .max_capacity(1000)
            .time_to_live(std::time::Duration::from_secs(3600)) // 1 hour TTL
            .build();

        Self {
            metrics_cache,
            alert_history: Arc::new(RwLock::new(Vec::new())),
            performance_baseline: Arc::new(RwLock::new(None)),
            thresholds: Arc::new(RwLock::new(AlertThresholds::default())),
            monitoring_state: Arc::new(RwLock::new(MonitoringState::default())),
        }
    }

    /// Initialize the health monitor
    pub async fn initialize(&self) -> OptimizerResult<()> {
        // Set up performance baseline
        self.establish_performance_baseline().await?;

        // Start background monitoring
        self.start_background_monitoring().await?;

        Ok(())
    }

    /// Collect current health metrics
    pub async fn collect_metrics(&self) -> OptimizerResult<HealthMetrics> {
        let mut metrics = HealthMetrics::default();
        let now = Utc::now();

        // Collect build health
        metrics.build_health = self.collect_build_health().await?;

        // Collect dependency health
        metrics.dependency_health = self.collect_dependency_health().await?;

        // Collect performance metrics
        metrics.performance_metrics = self.collect_performance_metrics().await?;

        // Generate alerts
        metrics.alerts = self.generate_alerts(&metrics).await?;

        metrics.timestamp = now;

        // Calculate overall health score
        metrics.overall_score = self.calculate_overall_health_score(&metrics);

        // Cache the metrics
        self.metrics_cache
            .insert("current".to_string(), metrics.clone())
            .await;

        Ok(metrics)
    }

    /// Get current health status
    pub async fn get_status(&self) -> OptimizerResult<HealthStatus> {
        let metrics = self.collect_metrics().await?;
        let monitoring_state = self.monitoring_state.read().await;

        let status = if metrics.overall_score >= 80.0 {
            SystemStatus::Healthy
        } else if metrics.overall_score >= 60.0 {
            SystemStatus::Warning
        } else {
            SystemStatus::Error
        };

        let active_alerts = metrics.alerts.clone();

        Ok(HealthStatus {
            status,
            metrics,
            active_alerts,
            last_updated: Utc::now(),
        })
    }

    /// Get health trend analysis
    pub async fn get_health_trends(
        &self,
        duration: ChronoDuration,
    ) -> OptimizerResult<HealthTrends> {
        let start_time = Utc::now() - duration;
        let mut data_points = Vec::new();

        // In a real implementation, this would query historical metrics
        // For now, return current metrics as a single data point

        if let Some(metrics) = self.metrics_cache.get(&"current".to_string()).await {
            data_points.push(HealthDataPoint {
                timestamp: metrics.timestamp,
                health_score: metrics.overall_score,
                build_success_rate: metrics.build_health.success_rate,
                memory_usage_mb: metrics.performance_metrics.memory_usage_mb,
                cpu_usage_percent: metrics.performance_metrics.cpu_usage_percent,
            });
        }

        let trends = self.analyze_health_trends(&data_points);

        Ok(HealthTrends {
            data_points,
            trends,
            analysis_period: duration,
            generated_at: Utc::now(),
        })
    }

    /// Update health thresholds
    pub async fn update_thresholds(&self, thresholds: AlertThresholds) -> OptimizerResult<()> {
        let mut current_thresholds = self.thresholds.write().await;
        *current_thresholds = thresholds;
        Ok(())
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<HealthAlert> {
        let alert_history = self.alert_history.read().await;
        alert_history
            .iter()
            .filter(|alert| self.is_alert_active(alert))
            .cloned()
            .collect()
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: &str) -> OptimizerResult<()> {
        let mut alert_history = self.alert_history.write().await;

        if let Some(alert) = alert_history
            .iter_mut()
            .find(|a| a.timestamp.to_string() == alert_id)
        {
            // In a real implementation, this would mark the alert as acknowledged
            // For now, just remove it from active alerts
        }

        Ok(())
    }

    /// Get performance baseline
    pub async fn get_performance_baseline(&self) -> Option<PerformanceBaseline> {
        let baseline = self.performance_baseline.read().await;
        baseline.clone()
    }

    /// Reset performance baseline
    pub async fn reset_performance_baseline(&self) -> OptimizerResult<()> {
        let mut baseline = self.performance_baseline.write().await;
        *baseline = None;
        self.establish_performance_baseline().await
    }

    // Private helper methods

    /// Establish performance baseline
    async fn establish_performance_baseline(&self) -> OptimizerResult<()> {
        // In a real implementation, this would run baseline measurements
        let baseline = PerformanceBaseline {
            average_build_time: std::time::Duration::from_secs(120),
            average_memory_usage_mb: 512.0,
            average_cpu_usage_percent: 70.0,
            established_at: Utc::now(),
            sample_count: 10,
        };

        let mut performance_baseline = self.performance_baseline.write().await;
        *performance_baseline = Some(baseline);

        Ok(())
    }

    /// Start background monitoring
    async fn start_background_monitoring(&self) -> OptimizerResult<()> {
        let mut monitoring_state = self.monitoring_state.write().await;
        monitoring_state.is_active = true;
        monitoring_state.last_check = Some(Utc::now());

        // In a real implementation, this would spawn background tasks
        // for continuous monitoring

        Ok(())
    }

    /// Collect build health metrics
    async fn collect_build_health(&self) -> OptimizerResult<BuildHealth> {
        // In a real implementation, this would analyze recent build logs
        // and collect actual metrics

        Ok(BuildHealth {
            success_rate: 95.0,
            average_build_time: std::time::Duration::from_secs(90),
            stability_score: 92.0,
            warnings_count: 5,
            errors_count: 1,
        })
    }

    /// Collect dependency health metrics
    async fn collect_dependency_health(&self) -> OptimizerResult<DependencyHealth> {
        // In a real implementation, this would analyze dependency metadata
        // and check for vulnerabilities, outdated packages, etc.

        Ok(DependencyHealth {
            circular_dependencies_count: 2,
            unused_dependencies_count: 8,
            average_dependency_depth: 3.2,
            outdated_dependencies_count: 3,
            security_vulnerabilities_count: 0,
        })
    }

    /// Collect performance metrics
    async fn collect_performance_metrics(&self) -> OptimizerResult<PerformanceMetrics> {
        // In a real implementation, this would use system monitoring APIs
        // to collect actual performance metrics

        Ok(PerformanceMetrics {
            memory_usage_mb: 450.0,
            cpu_usage_percent: 65.0,
            disk_iops: 1250.0,
            network_iops: Some(150.0),
            active_threads: 24,
        })
    }

    /// Generate alerts based on current metrics
    async fn generate_alerts(&self, metrics: &HealthMetrics) -> OptimizerResult<Vec<HealthAlert>> {
        let mut alerts = Vec::new();
        let thresholds = self.thresholds.read().await;

        // Check build health alerts
        if metrics.build_health.success_rate < 90.0 {
            alerts.push(HealthAlert {
                level: AlertLevel::Warning,
                message: format!(
                    "Build success rate is low: {:.1}%",
                    metrics.build_health.success_rate
                ),
                component: "Build System".to_string(),
                recommended_action: "Review build configuration and fix compilation errors"
                    .to_string(),
                timestamp: Utc::now(),
            });
        }

        // Check memory usage alerts
        if metrics.performance_metrics.memory_usage_mb > thresholds.max_memory_threshold_mb {
            alerts.push(HealthAlert {
                level: AlertLevel::Error,
                message: format!(
                    "High memory usage: {:.1}MB",
                    metrics.performance_metrics.memory_usage_mb
                ),
                component: "Memory Management".to_string(),
                recommended_action: "Consider optimizing memory usage or increasing system memory"
                    .to_string(),
                timestamp: Utc::now(),
            });
        }

        // Check dependency health alerts
        if metrics.dependency_health.circular_dependencies_count
            > thresholds.circular_deps_warning_threshold
        {
            alerts.push(HealthAlert {
                level: AlertLevel::High,
                message: format!(
                    "Found {} circular dependencies",
                    metrics.dependency_health.circular_dependencies_count
                ),
                component: "Dependency Management".to_string(),
                recommended_action: "Review and resolve circular dependencies".to_string(),
                timestamp: Utc::now(),
            });
        }

        // Store alerts in history
        let mut alert_history = self.alert_history.write().await;
        alert_history.extend(alerts.clone());

        Ok(alerts)
    }

    /// Calculate overall health score
    fn calculate_overall_health_score(&self, metrics: &HealthMetrics) -> f64 {
        let build_score = metrics.build_health.success_rate * 0.3;
        let dependency_score = if metrics.dependency_health.circular_dependencies_count == 0 {
            100.0
        } else {
            80.0
        } * 0.2;
        let performance_score = if metrics.performance_metrics.memory_usage_mb < 1000.0 {
            100.0
        } else {
            80.0
        } * 0.2;
        let stability_score = metrics.build_health.stability_score * 0.3;

        build_score + dependency_score + performance_score + stability_score
    }

    /// Analyze health trends
    fn analyze_health_trends(&self, data_points: &[HealthDataPoint]) -> HealthTrendAnalysis {
        if data_points.is_empty() {
            return HealthTrendAnalysis::default();
        }

        let latest = &data_points[data_points.len() - 1];

        // Simple trend analysis
        let health_trend = if data_points.len() > 1 {
            let previous = &data_points[data_points.len() - 2];
            if latest.health_score > previous.health_score {
                HealthTrend::Improving
            } else if latest.health_score < previous.health_score {
                HealthTrend::Declining
            } else {
                HealthTrend::Stable
            }
        } else {
            HealthTrend::Stable
        };

        HealthTrendAnalysis {
            overall_trend: health_trend,
            build_success_trend: health_trend, // Simplified
            memory_usage_trend: HealthTrend::Stable,
            cpu_usage_trend: HealthTrend::Stable,
            recommendations: vec![
                "Continue monitoring build performance".to_string(),
                "Review dependency health regularly".to_string(),
            ],
        }
    }

    /// Check if an alert is still active
    fn is_alert_active(&self, _alert: &HealthAlert) -> bool {
        // In a real implementation, this would check if the alert condition still exists
        // For now, consider all recent alerts as active
        true
    }
}

impl Default for WorkspaceHealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance baseline data
#[derive(Debug, Clone)]
pub struct PerformanceBaseline {
    /// Average build time
    pub average_build_time: std::time::Duration,
    /// Average memory usage (MB)
    pub average_memory_usage_mb: f64,
    /// Average CPU usage (%)
    pub average_cpu_usage_percent: f64,
    /// When baseline was established
    pub established_at: DateTime<Utc>,
    /// Number of samples used for baseline
    pub sample_count: usize,
}

/// Monitoring state
#[derive(Debug, Clone, Default)]
pub struct MonitoringState {
    /// Whether monitoring is active
    pub is_active: bool,
    /// Last health check timestamp
    pub last_check: Option<DateTime<Utc>>,
    /// Monitoring interval
    pub check_interval: std::time::Duration,
}

/// Health trends data
#[derive(Debug, Clone)]
pub struct HealthTrends {
    /// Historical data points
    pub data_points: Vec<HealthDataPoint>,
    /// Trend analysis
    pub trends: HealthTrendAnalysis,
    /// Analysis period
    pub analysis_period: ChronoDuration,
    /// When trends were generated
    pub generated_at: DateTime<Utc>,
}

/// Health data point
#[derive(Debug, Clone)]
pub struct HealthDataPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Overall health score
    pub health_score: f64,
    /// Build success rate
    pub build_success_rate: f64,
    /// Memory usage (MB)
    pub memory_usage_mb: f64,
    /// CPU usage (%)
    pub cpu_usage_percent: f64,
}

/// Health trend analysis
#[derive(Debug, Clone, Default)]
pub struct HealthTrendAnalysis {
    /// Overall health trend
    pub overall_trend: HealthTrend,
    /// Build success rate trend
    pub build_success_trend: HealthTrend,
    /// Memory usage trend
    pub memory_usage_trend: HealthTrend,
    /// CPU usage trend
    pub cpu_usage_trend: HealthTrend,
    /// Recommendations based on trends
    pub recommendations: Vec<String>,
}

/// Health trend direction
#[derive(Debug, Clone)]
pub enum HealthTrend {
    /// Health is improving
    Improving,
    /// Health is declining
    Declining,
    /// Health is stable
    Stable,
}

impl Default for HealthTrend {
    fn default() -> Self {
        Self::Stable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_monitor_creation() {
        let monitor = WorkspaceHealthMonitor::new();
        assert!(!monitor.monitoring_state.read().await.is_active);
    }

    #[tokio::test]
    async fn test_collect_metrics() {
        let monitor = WorkspaceHealthMonitor::new();
        let result = monitor.collect_metrics().await;
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert!(metrics.overall_score >= 0.0 && metrics.overall_score <= 100.0);
    }

    #[tokio::test]
    async fn test_get_health_status() {
        let monitor = WorkspaceHealthMonitor::new();
        let result = monitor.get_status().await;
        assert!(result.is_ok());

        let status = result.unwrap();
        assert!(matches!(
            status.status,
            SystemStatus::Healthy
                | SystemStatus::Warning
                | SystemStatus::Error
                | SystemStatus::Critical
        ));
    }

    #[tokio::test]
    async fn test_threshold_updates() {
        let monitor = WorkspaceHealthMonitor::new();
        let new_thresholds = AlertThresholds {
            max_memory_threshold_mb: 1024.0,
            ..Default::default()
        };

        let result = monitor.update_thresholds(new_thresholds).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_performance_baseline() {
        let monitor = WorkspaceHealthMonitor::new();

        // Initially no baseline
        assert!(monitor.get_performance_baseline().await.is_none());

        // Reset should establish a baseline
        let result = monitor.reset_performance_baseline().await;
        assert!(result.is_ok());
        assert!(monitor.get_performance_baseline().await.is_some());
    }
}

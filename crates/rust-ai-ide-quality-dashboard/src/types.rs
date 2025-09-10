//! # Quality Intelligence Dashboard Types
//!
//! This module defines all the core types used throughout the quality intelligence dashboard.
//! These types provide the foundation for metric collection, visualization, and interaction.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core quality metric types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityMetric {
    /// Code quality score (0.0 to 1.0)
    CodeQualityScore(f64),

    /// Performance efficiency metric
    PerformanceEfficiency(PerformanceMetrics),

    /// Security vulnerability count
    SecurityVulnerabilities(i32),

    /// Test coverage percentage
    TestCoverage(f64),

    /// Code duplication percentage
    CodeDuplication(f64),

    /// Maintainability index
    MaintainabilityIndex(f64),

    /// Technical debt hours
    TechnicalDebt(i32),

    /// Cyclomatic complexity
    CyclomaticComplexity(f64),

    /// Custom metric with name and value
    Custom {
        name: String,
        value: MetricValue,
        unit: Option<String>,
    },
}

/// Metric value with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    /// The numeric value
    pub value: f64,

    /// Optional confidence level (0.0 to 1.0)
    pub confidence: Option<f64>,

    /// Collection timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Optional metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Performance metrics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// CPU usage percentage
    pub cpu_usage: f64,

    /// Memory usage in MB
    pub memory_usage: f64,

    /// Disk I/O operations per second
    pub disk_io: f64,

    /// Network latency in milliseconds
    pub network_latency: f64,

    /// Response time in milliseconds
    pub response_time: f64,
}

/// Trend analysis data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPoint {
    /// Timestamp of the measurement
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Metric value at this point
    pub value: f64,

    /// Confidence interval
    pub confidence_interval: Option<(f64, f64)>,
}

/// Time series data for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesData {
    /// Series identifier
    pub id: String,

    /// Series name
    pub name: String,

    /// Data points
    pub points: Vec<TrendPoint>,

    /// Trend analysis result
    pub trend: Option<TrendDirection>,
}

/// Trend direction enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Improving trend
    Improving {
        slope: f64,
        confidence: f64,
    },

    /// Degrading trend
    Degrading {
        slope: f64,
        confidence: f64,
    },

    /// Stable trend
    Stable {
        volatility: f64,
    },
}

/// Quality benchmarking data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkData {
    /// Benchmark category
    pub category: String,

    /// Current project value
    pub current_value: f64,

    /// Industry average
    pub industry_average: f64,

    /// Project percentile ranking
    pub percentile: f64,

    /// Comparable projects count
    pub sample_size: i32,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfiguration {
    /// Dashboard update interval in seconds
    pub update_interval: u64,

    /// Maximum historical data retention in days
    pub retention_days: i32,

    /// Alert thresholds
    pub thresholds: AlertThresholds,

    /// UI preferences
    pub ui_prefs: UiPreferences,

    /// Enabled metric categories
    pub enabled_metrics: Vec<String>,

    /// Custom integrations
    pub Integrations: IntegrationConfig,
}

impl Default for DashboardConfiguration {
    fn default() -> Self {
        Self {
            update_interval: 30, // 30 seconds
            retention_days: 90,  // 90 days
            thresholds: AlertThresholds::default(),
            ui_prefs: UiPreferences::default(),
            enabled_metrics: vec![
                "code_quality".to_string(),
                "performance".to_string(),
                "security".to_string(),
                "testing".to_string(),
                "complexity".to_string(),
            ],
            Integrations: IntegrationConfig::default(),
        }
    }
}

/// Alert threshold configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Critical threshold (red alerts)
    pub critical: Thresholds,

    /// Warning threshold (yellow alerts)
    pub warning: Thresholds,

    /// Info threshold (blue alerts)
    pub info: Thresholds,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            critical: Thresholds {
                code_quality_min: 0.2,
                performance_latent_max: 1000.0,
                security_vulns_max: 10,
                test_coverage_min: 0.1,
                maintainability_min: 0.3,
            },
            warning: Thresholds {
                code_quality_min: 0.5,
                performance_latent_max: 500.0,
                security_vulns_max: 5,
                test_coverage_min: 0.5,
                maintainability_min: 0.6,
            },
            info: Thresholds {
                code_quality_min: 0.7,
                performance_latent_max: 200.0,
                security_vulns_max: 2,
                test_coverage_min: 0.7,
                maintainability_min: 0.8,
            },
        }
    }
}

/// Threshold values for different metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    /// Minimum code quality score
    pub code_quality_min: f64,

    /// Maximum performance latency
    pub performance_latent_max: f64,

    /// Maximum security vulnerabilities
    pub security_vulns_max: i32,

    /// Minimum test coverage
    pub test_coverage_min: f64,

    /// Minimum maintainability index
    pub maintainability_min: f64,
}

/// UI preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    /// Theme (light/dark/auto)
    pub theme: String,

    /// Chart style (lines/bars/area)
    pub chart_style: String,

    /// Default time range (1h/24h/7d/30d)
    pub default_time_range: String,

    /// Layout preference
    pub layout: String,

    /// Auto-refresh enabled
    pub auto_refresh: bool,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            theme: "auto".to_string(),
            chart_style: "lines".to_string(),
            default_time_range: "24h".to_string(),
            layout: "grid".to_string(),
            auto_refresh: true,
        }
    }
}

/// Integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    /// Real-time analysis integration
    pub real_time_analysis: IntegrationSettings,

    /// Predictive maintenance integration
    pub predictive_maintenance: IntegrationSettings,

    /// Collaboration hub settings
    pub collaboration: CollaborationSettings,

    /// Export capabilities
    pub export: ExportSettings,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            real_time_analysis: IntegrationSettings::default(),
            predictive_maintenance: IntegrationSettings::default(),
            collaboration: CollaborationSettings::default(),
            export: ExportSettings::default(),
        }
    }
}

/// Generic integration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationSettings {
    /// Integration enabled
    pub enabled: bool,

    /// Update frequency in seconds
    pub update_frequency: u64,

    /// API endpoint
    pub endpoint: Option<String>,
}

impl Default for IntegrationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            update_frequency: 30,
            endpoint: None,
        }
    }
}

/// Collaboration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSettings {
    /// Team features enabled
    pub team_features: bool,

    /// Sharing enabled
    pub sharing: bool,

    /// Real-time collaboration
    pub real_time: bool,

    /// Maximum team size
    pub max_team_size: i32,
}

impl Default for CollaborationSettings {
    fn default() -> Self {
        Self {
            team_features: true,
            sharing: true,
            real_time: true,
            max_team_size: 50,
        }
    }
}

/// Export settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSettings {
    /// Allow CSV export
    pub csv_export: bool,

    /// Allow JSON export
    pub json_export: bool,

    /// Allow PDF reports
    pub pdf_reports: bool,

    /// Maximum export size in MB
    pub max_export_size: f64,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            csv_export: true,
            json_export: true,
            pdf_reports: true,
            max_export_size: 100.0,
        }
    }
}

/// Dashboard widget definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    /// Widget unique ID
    pub id: String,

    /// Widget type
    pub widget_type: String,

    /// Widget position
    pub position: WidgetPosition,

    /// Widget size
    pub size: WidgetSize,

    /// Widget configuration
    pub config: serde_json::Value,
}

/// Widget position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: i32,
    pub y: i32,
}

/// Widget size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSize {
    pub width: i32,
    pub height: i32,
}

/// Quality score breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScore {
    /// Overall score
    pub overall: f64,

    /// Component scores
    pub components: HashMap<String, f64>,

    /// Score confidence
    pub confidence: f64,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Validation trait for configurations
pub trait Validate {
    fn validate(&self) -> std::result::Result<(), String>;
}

impl Validate for DashboardConfiguration {
    fn validate(&self) -> std::result::Result<(), String> {
        if self.update_interval == 0 {
            return Err("Update interval must be greater than 0".to_string());
        }
        if self.retention_days <= 0 {
            return Err("Retention days must be greater than 0".to_string());
        }
        Ok(())
    }
}
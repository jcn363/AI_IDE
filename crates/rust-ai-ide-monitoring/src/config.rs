//! Configuration management for the monitoring framework

use crate::errors::{MonitoringError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Main configuration structure for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Workspace root directory
    pub workspace_root: PathBuf,

    /// Output directory for reports
    pub report_dir: PathBuf,

    /// Analyzers to enable (if empty, all are enabled)
    pub enabled_analyzers: Vec<String>,

    /// Analyzers to disable
    pub disabled_analyzers: Vec<String>,

    /// Quality score thresholds
    pub thresholds: Thresholds,

    /// Analyzer-specific configurations
    pub analyzer_configs: HashMap<String, AnalyzerConfig>,

    /// Target platforms for cross-compilation analysis
    pub target_platforms: Vec<String>,

    /// Notification settings
    pub notifications: NotificationConfig,

    /// Performance monitoring settings
    pub performance: PerformanceConfig,

    /// Output format preferences
    pub output: OutputConfig,

    /// Analysis timeout in seconds
    pub analysis_timeout: u64,

    /// Verbose output
    pub verbose: bool,

    /// Dry run mode
    pub dry_run: bool,
}

/// Quality score thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    /// Minimum acceptable quality score (0-100)
    pub quality_score_min: f64,

    /// Warning threshold for quality score
    pub quality_score_warn: f64,

    /// Maximum acceptable warning count
    pub max_warnings: usize,

    /// Maximum acceptable error count
    pub max_errors: usize,

    /// Compilation time threshold in seconds
    pub max_compilation_time_secs: u64,

    /// Maximum memory usage in MB
    pub max_memory_mb: usize,

    /// Security vulnerability threshold
    pub security_vulnerability_max: usize,

    /// Dependency audit failures threshold
    pub dependency_failures_max: usize,
}

/// Analyzer-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerConfig {
    /// Whether this analyzer is enabled
    pub enabled: bool,

    /// Timeout for this analyzer
    pub timeout_seconds: Option<u64>,

    /// Custom parameters as key-value pairs
    pub parameters: HashMap<String, serde_json::Value>,

    /// Categories to include in analysis
    pub include_categories: Vec<String>,

    /// Categories to exclude from analysis
    pub exclude_categories: Vec<String>,
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Whether notifications are enabled
    pub enabled: bool,

    /// Notification endpoints (webhooks, etc.)
    pub endpoints: Vec<NotificationEndpoint>,

    /// Minimum severity level to trigger notifications
    pub min_severity: crate::types::Severity,

    /// Whether to notify on improvements
    pub notify_improvements: bool,

    /// Whether to notify on regressions
    pub notify_regressions: bool,

    /// Cool-down period between notifications (in seconds)
    pub cooldown_seconds: u64,
}

/// Notification endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationEndpoint {
    /// Endpoint type (slack, webhook, email, etc.)
    pub endpoint_type: String,

    /// URL for webhook endpoints
    pub url: Option<String>,

    /// Channel or target for notifications
    pub target: Option<String>,

    /// Authentication token or key
    pub auth_token: Option<String>,

    /// Custom headers
    pub headers: HashMap<String, String>,
}

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Whether performance monitoring is enabled
    pub enabled: bool,

    /// Sampling interval in milliseconds
    pub sampling_interval_ms: u64,

    /// Whether to track memory usage
    pub track_memory: bool,

    /// Whether to track CPU usage
    pub track_cpu: bool,

    /// Number of historical data points to keep
    pub history_size: usize,

    /// Statistics window size in minutes
    pub window_size_minutes: usize,
}

/// Output format configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output format ("json", "yaml", "markdown", "html")
    pub format: String,

    /// Pretty print JSON output
    pub pretty_print: bool,

    /// Include detailed findings
    pub include_details: bool,

    /// Include performance metrics
    pub include_performance: bool,

    /// Include system information
    pub include_system_info: bool,

    /// Compression for output files
    pub compress: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            workspace_root: PathBuf::from(".")
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(".")),
            report_dir: PathBuf::from(crate::defaults::DEFAULT_REPORT_DIR),
            enabled_analyzers: Vec::new(),
            disabled_analyzers: Vec::new(),
            thresholds: Thresholds::default(),
            analyzer_configs: HashMap::new(),
            target_platforms: Vec::new(),
            notifications: NotificationConfig::default(),
            performance: PerformanceConfig::default(),
            output: OutputConfig::default(),
            analysis_timeout: crate::defaults::DEFAULT_ANALYSIS_TIMEOUT_SECS,
            verbose: false,
            dry_run: false,
        }
    }
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            quality_score_min: crate::defaults::DEFAULT_THRESHOLD_QUALITY,
            quality_score_warn: 50.0,
            max_warnings: crate::defaults::DEFAULT_THRESHOLD_WARNINGS,
            max_errors: 0,
            max_compilation_time_secs: 300, // 5 minutes
            max_memory_mb: 4096,            // 4GB
            security_vulnerability_max: 0,
            dependency_failures_max: 0,
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoints: Vec::new(),
            min_severity: crate::types::Severity::Medium,
            notify_improvements: true,
            notify_regressions: true,
            cooldown_seconds: 300, // 5 minutes
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling_interval_ms: 1000,
            track_memory: true,
            track_cpu: true,
            history_size: 1000,
            window_size_minutes: 60,
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: "json".to_string(),
            pretty_print: true,
            include_details: true,
            include_performance: true,
            include_system_info: true,
            compress: false,
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content =
            std::fs::read_to_string(path).map_err(|e| MonitoringError::Fs { source: e })?;
        serde_yaml::from_str(&content)
            .map_err(|e| MonitoringError::other(format!("YAML parsing error: {}", e)))
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_yaml::to_string(self)
            .map_err(|e| MonitoringError::other(format!("YAML serialization error: {}", e)))?;
        std::fs::write(path, content).map_err(|e| MonitoringError::Fs { source: e })?;
        Ok(())
    }

    /// Load configuration with defaults merged from file if it exists
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::from_file(path) {
            Ok(config) => config,
            Err(_) => Self::default(),
        }
    }

    /// Get configuration summary for reporting
    pub fn get_config_summary(&self) -> crate::types::ConfigSummary {
        crate::types::ConfigSummary {
            workspace_root: self.workspace_root.clone(),
            enabled_analyzers: self.enabled_analyzers.clone(),
            thresholds: std::collections::HashMap::new(), // Would be populated from config
            target_platforms: self.target_platforms.clone(),
        }
    }

    /// Check if an analyzer is enabled
    pub fn is_analyzer_enabled(&self, analyzer_name: &str) -> bool {
        // If explicit enabled list is not empty, check if analyzer is in it
        if !self.enabled_analyzers.is_empty() {
            if !self.enabled_analyzers.contains(&analyzer_name.to_string()) {
                return false;
            }
        }

        // Check if analyzer is explicitly disabled
        if self.disabled_analyzers.contains(&analyzer_name.to_string()) {
            return false;
        }

        // Check analyzer-specific config
        if let Some(config) = self.analyzer_configs.get(analyzer_name) {
            return config.enabled;
        }

        // Default enabled
        true
    }

    /// Get analyzer configuration
    pub fn get_analyzer_config(&self, analyzer_name: &str) -> AnalyzerConfig {
        self.analyzer_configs
            .get(analyzer_name)
            .cloned()
            .unwrap_or_else(|| AnalyzerConfig {
                enabled: true,
                timeout_seconds: None,
                parameters: HashMap::new(),
                include_categories: Vec::new(),
                exclude_categories: Vec::new(),
            })
    }
}

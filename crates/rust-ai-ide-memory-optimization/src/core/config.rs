//! Configuration for memory optimization system
//! Uses serde for serialization and integrates with workspace-wide config patterns.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Configuration for the memory optimization system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryOptimizationConfig {
    /// Enable memory leak detection
    pub enable_leak_detection: bool,

    /// Leak detection scan interval
    pub leak_scan_interval_seconds: u64,

    /// Enable automatic optimization suggestions
    pub enable_auto_suggestions: bool,

    /// Memory usage alert threshold (percentage of total memory)
    pub memory_alert_threshold_percent: f64,

    /// Enable AI-powered analysis
    pub enable_ai_analysis: bool,

    /// Enable SIMD acceleration for memory analysis
    pub enable_simd_acceleration: bool,

    /// Memory pool size for optimization suggestions
    pub suggestion_cache_size_mb: usize,

    /// Background monitoring interval
    pub monitoring_interval_seconds: u64,

    /// Enable cross-crate memory analysis
    pub enable_cross_crate_analysis: bool,

    /// Memory snapshot retention period in hours
    pub snapshot_retention_hours: u64,

    /// Maximum memory pool size
    pub max_memory_pool_mb: usize,
}

impl Default for MemoryOptimizationConfig {
    fn default() -> Self {
        Self {
            enable_leak_detection:          true,
            leak_scan_interval_seconds:     300, // 5 minutes
            enable_auto_suggestions:        true,
            memory_alert_threshold_percent: 85.0,
            enable_ai_analysis:             true,
            enable_simd_acceleration:       true,
            suggestion_cache_size_mb:       100,
            monitoring_interval_seconds:    60, // 1 minute
            enable_cross_crate_analysis:    true,
            snapshot_retention_hours:       24,
            max_memory_pool_mb:             512,
        }
    }
}

impl MemoryOptimizationConfig {
    /// Create a new configuration with custom settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable/disable leak detection
    pub fn with_leak_detection(mut self, enabled: bool) -> Self {
        self.enable_leak_detection = enabled;
        self
    }

    /// Set leak detection scan interval
    pub fn with_leak_scan_interval(mut self, seconds: u64) -> Self {
        self.leak_scan_interval_seconds = seconds;
        self
    }

    /// Enable/disable automatic optimization suggestions
    pub fn with_auto_suggestions(mut self, enabled: bool) -> Self {
        self.enable_auto_suggestions = enabled;
        self
    }

    /// Set memory alert threshold
    pub fn with_memory_alert_threshold(mut self, percent: f64) -> Self {
        self.memory_alert_threshold_percent = percent;
        self
    }

    /// Enable/disable AI-powered analysis
    pub fn with_ai_analysis(mut self, enabled: bool) -> Self {
        self.enable_ai_analysis = enabled;
        self
    }

    /// Enable/disable SIMD acceleration
    pub fn with_simd_acceleration(mut self, enabled: bool) -> Self {
        self.enable_simd_acceleration = enabled;
        self
    }

    /// Get leak scan interval as Duration
    pub fn get_leak_scan_interval(&self) -> Duration {
        Duration::from_secs(self.leak_scan_interval_seconds)
    }

    /// Get monitoring interval as Duration
    pub fn get_monitoring_interval(&self) -> Duration {
        Duration::from_secs(self.monitoring_interval_seconds)
    }

    /// Get snapshot retention duration
    pub fn get_snapshot_retention(&self) -> Duration {
        Duration::from_secs(self.snapshot_retention_hours * 3600)
    }

    /// Validate configuration parameters
    pub fn validate(&self) -> Result<(), String> {
        if self.memory_alert_threshold_percent < 0.0 || self.memory_alert_threshold_percent > 100.0 {
            return Err("Memory alert threshold must be between 0 and 100".to_string());
        }

        if self.leak_scan_interval_seconds == 0 {
            return Err("Leak scan interval must be greater than 0".to_string());
        }

        if self.monitoring_interval_seconds == 0 {
            return Err("Monitoring interval must be greater than 0".to_string());
        }

        if self.snapshot_retention_hours == 0 {
            return Err("Snapshot retention period must be greater than 0".to_string());
        }

        if self.max_memory_pool_mb == 0 {
            return Err("Maximum memory pool size must be greater than 0".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MemoryOptimizationConfig::default();
        assert!(config.enable_leak_detection);
        assert!(config.enable_auto_suggestions);
        assert!(config.enable_ai_analysis);
        assert!(config.enable_simd_acceleration);
    }

    #[test]
    fn test_config_with_methods() {
        let config = MemoryOptimizationConfig::new()
            .with_leak_detection(false)
            .with_auto_suggestions(false)
            .with_memory_alert_threshold(90.0);

        assert!(!config.enable_leak_detection);
        assert!(!config.enable_auto_suggestions);
        assert_eq!(config.memory_alert_threshold_percent, 90.0);
    }

    #[test]
    fn test_config_validation() {
        // Valid config
        let valid_config = MemoryOptimizationConfig::default();
        assert!(valid_config.validate().is_ok());

        // Invalid memory alert threshold
        let invalid_config = MemoryOptimizationConfig {
            memory_alert_threshold_percent: 150.0,
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_duration_conversions() {
        let config = MemoryOptimizationConfig {
            leak_scan_interval_seconds: 300,
            monitoring_interval_seconds: 60,
            snapshot_retention_hours: 24,
            ..Default::default()
        };

        assert_eq!(config.get_leak_scan_interval(), Duration::from_secs(300));
        assert_eq!(config.get_monitoring_interval(), Duration::from_secs(60));
        assert_eq!(config.get_snapshot_retention(), Duration::from_secs(86400));
    }
}

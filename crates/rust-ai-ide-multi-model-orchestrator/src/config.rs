//! Configuration management for Multi-Model Orchestration
//!
//! This module provides configuration structures and default values for the orchestration system.

use crate::types::{
    ConsensusConfig, FallbackConfig, LoadBalancingConfig, ModelSwitchingConfig,
    OrchestrationConfig, PerformanceThresholds, VotingMechanism,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Default orchestration configuration
impl Default for OrchestrationConfig {
    fn default() -> Self {
        Self {
            performance_thresholds: PerformanceThresholds::default(),
            load_balancing_config: LoadBalancingConfig::default(),
            consensus_config: ConsensusConfig::default(),
            fallback_config: FallbackConfig::default(),
            model_switching_config: ModelSwitchingConfig::default(),
        }
    }
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_latency_ms: 5000.0, // 5 seconds
            min_accuracy: 0.7,      // 70% accuracy threshold
            max_memory_mb: 8192.0,  // 8GB memory limit
            max_cpu_percent: 80.0,  // 80% CPU usage limit
            health_check_interval_secs: 30,
        }
    }
}

impl Default for LoadBalancingConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: num_cpus::get() * 4,
            queue_capacity: 1000,
            load_balance_interval_secs: 10,
            overload_threshold: 0.9, // 90% capacity threshold
        }
    }
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            min_models_for_consensus: 2,
            confidence_threshold: 0.8,   // 80% confidence required
            disagreement_tolerance: 0.3, // Allow 30% disagreement
            voting_mechanism: VotingMechanism::ConfidenceBased,
        }
    }
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            offline_cache_duration_days: 7, // 7 days cache validity
            grace_period_secs: 30,          // 30 second graceful degradation
            fallback_sequence: Vec::new(),  // Will be populated at runtime
        }
    }
}

impl Default for ModelSwitchingConfig {
    fn default() -> Self {
        Self {
            switching_latency_target_ms: 100.0, // <100ms switch time target
            cooldown_duration_secs: 60,         // 1 minute cooldown between switches
            hysteresis_factor: 1.05,            // 5% hysteresis to prevent thrashing
        }
    }
}

/// Configuration builder for fine-tuning orchestration behavior
pub struct OrchestrationConfigBuilder {
    config: OrchestrationConfig,
}

impl OrchestrationConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: OrchestrationConfig::default(),
        }
    }

    pub fn with_performance_thresholds(mut self, thresholds: PerformanceThresholds) -> Self {
        self.config.performance_thresholds = thresholds;
        self
    }

    pub fn with_max_latency(mut self, latency_ms: f64) -> Self {
        self.config.performance_thresholds.max_latency_ms = latency_ms;
        self
    }

    pub fn with_min_accuracy(mut self, accuracy: f64) -> Self {
        self.config.performance_thresholds.min_accuracy = accuracy;
        self
    }

    pub fn with_load_balancing_config(mut self, lb_config: LoadBalancingConfig) -> Self {
        self.config.load_balancing_config = lb_config;
        self
    }

    pub fn with_max_concurrent_requests(mut self, max_requests: usize) -> Self {
        self.config.load_balancing_config.max_concurrent_requests = max_requests;
        self
    }

    pub fn with_consensus_config(mut self, consensus_config: ConsensusConfig) -> Self {
        self.config.consensus_config = consensus_config;
        self
    }

    pub fn with_min_models_for_consensus(mut self, min_models: usize) -> Self {
        self.config.consensus_config.min_models_for_consensus = min_models;
        self
    }

    pub fn with_fallback_config(mut self, fallback_config: FallbackConfig) -> Self {
        self.config.fallback_config = fallback_config;
        self
    }

    pub fn with_model_switching_config(mut self, switching_config: ModelSwitchingConfig) -> Self {
        self.config.model_switching_config = switching_config;
        self
    }

    pub fn build(self) -> OrchestrationConfig {
        self.config
    }
}

/// Validates that configuration values are within acceptable ranges
pub fn validate_config(config: &OrchestrationConfig) -> crate::Result<()> {
    // Performance thresholds validation
    if config.performance_thresholds.max_latency_ms <= 0.0 {
        return Err(crate::OrchestrationError::ConfigError(
            "Maximum latency must be positive".to_string(),
        ));
    }

    if !(0.0..=1.0).contains(&config.performance_thresholds.min_accuracy) {
        return Err(crate::OrchestrationError::ConfigError(
            "Minimum accuracy must be between 0.0 and 1.0".to_string(),
        ));
    }

    if !(0.0..=100.0).contains(&config.performance_thresholds.max_cpu_percent) {
        return Err(crate::OrchestrationError::ConfigError(
            "Maximum CPU usage must be between 0.0 and 100.0".to_string(),
        ));
    }

    // Load balancing validation
    if config.load_balancing_config.max_concurrent_requests == 0 {
        return Err(crate::OrchestrationError::ConfigError(
            "Maximum concurrent requests must be greater than 0".to_string(),
        ));
    }

    if !(0.0..=1.0).contains(&config.load_balancing_config.overload_threshold) {
        return Err(crate::OrchestrationError::ConfigError(
            "Overload threshold must be between 0.0 and 1.0".to_string(),
        ));
    }

    // Consensus validation
    if config.consensus_config.min_models_for_consensus < 1 {
        return Err(crate::OrchestrationError::ConfigError(
            "Minimum models for consensus must be at least 1".to_string(),
        ));
    }

    if !(0.0..=1.0).contains(&config.consensus_config.confidence_threshold) {
        return Err(crate::OrchestrationError::ConfigError(
            "Confidence threshold must be between 0.0 and 1.0".to_string(),
        ));
    }

    // Voting mechanism specific validation
    // Add any validation logic specific to voting mechanisms here if needed

    // Model switching validation
    if config.model_switching_config.switching_latency_target_ms <= 0.0 {
        return Err(crate::OrchestrationError::ConfigError(
            "Switching latency target must be positive".to_string(),
        ));
    }

    if config.model_switching_config.hysteresis_factor < 1.0 {
        return Err(crate::OrchestrationError::ConfigError(
            "Hysteresis factor must be greater than or equal to 1.0".to_string(),
        ));
    }

    Ok(())
}

/// Save configuration to a TOML file
pub fn save_config_to_file<P: AsRef<std::path::Path>>(
    config: &OrchestrationConfig,
    path: P,
) -> crate::Result<()> {
    let toml_string = toml::to_string_pretty(config).map_err(|e| {
        crate::OrchestrationError::ConfigError(format!("Failed to serialize config: {}", e))
    })?;

    std::fs::write(path, toml_string).map_err(|e| {
        crate::OrchestrationError::ConfigError(format!("Failed to write config file: {}", e))
    })?;

    Ok(())
}

/// Load configuration from a TOML file
pub fn load_config_from_file<P: AsRef<std::path::Path>>(
    path: P,
) -> crate::Result<OrchestrationConfig> {
    let config_content = std::fs::read_to_string(path).map_err(|e| {
        crate::OrchestrationError::ConfigError(format!("Failed to read config file: {}", e))
    })?;

    let mut config: OrchestrationConfig = toml::from_str(&config_content).map_err(|e| {
        crate::OrchestrationError::ConfigError(format!("Failed to parse config: {}", e))
    })?;

    // Validate the loaded configuration
    validate_config(&config)?;

    Ok(config)
}

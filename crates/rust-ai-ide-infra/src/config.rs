//! Configuration module for background defragmentation system

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for the background defragmentation system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefragmentationConfig {
    /// Enable background defragmentation
    pub enabled: bool,

    /// Interval between defragmentation cycles
    pub cycle_interval: Duration,

    /// Maximum duration for a single defragmentation cycle
    pub max_cycle_duration: Duration,

    /// Fragmentation threshold to trigger defragmentation (0.0-1.0)
    pub fragmentation_threshold: f64,

    /// Maximum memory pressure before defragmentation is paused
    pub max_memory_pressure: f64,

    /// Minimum time between defragmentation cycles
    pub cooldown_period: Duration,

    /// Maximum number of concurrent defragmentation tasks
    pub max_concurrent_tasks: usize,

    /// Enable metrics collection
    pub enable_metrics: bool,

    /// Algorithm selection
    pub algorithm: DefragmentationAlgorithmType,

    /// Pool-specific configurations
    pub pool_configs: Vec<PoolDefragmentationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DefragmentationAlgorithmType {
    /// Copying garbage collection algorithm
    Copying,
    /// Mark-compact algorithm
    MarkCompact,
    /// Generational algorithm
    Generational,
    /// Pool-specific algorithm
    PoolSpecific,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolDefragmentationConfig {
    /// Pool identifier
    pub pool_id: String,

    /// Specific algorithm for this pool
    pub algorithm: DefragmentationAlgorithmType,

    /// Pool-specific fragmentation threshold
    pub fragmentation_threshold: f64,

    /// Pool-specific cycle interval
    pub cycle_interval: Duration,
}

impl Default for DefragmentationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cycle_interval: Duration::from_secs(300), // 5 minutes
            max_cycle_duration: Duration::from_secs(30), // 30 seconds
            fragmentation_threshold: 0.7, // 70%
            max_memory_pressure: 0.9, // 90%
            cooldown_period: Duration::from_secs(60), // 1 minute
            max_concurrent_tasks: 4,
            enable_metrics: true,
            algorithm: DefragmentationAlgorithmType::Generational,
            pool_configs: Vec::new(),
        }
    }
}

impl DefragmentationConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create configuration optimized for high-performance scenarios
    pub fn high_performance() -> Self {
        Self {
            cycle_interval: Duration::from_secs(60), // 1 minute
            max_cycle_duration: Duration::from_secs(10), // 10 seconds
            fragmentation_threshold: 0.5, // 50%
            max_concurrent_tasks: 8,
            ..Default::default()
        }
    }

    /// Create configuration optimized for memory-constrained environments
    pub fn memory_conservative() -> Self {
        Self {
            cycle_interval: Duration::from_secs(600), // 10 minutes
            max_cycle_duration: Duration::from_secs(60), // 1 minute
            fragmentation_threshold: 0.8, // 80%
            max_concurrent_tasks: 2,
            ..Default::default()
        }
    }

    /// Add a pool-specific configuration
    pub fn with_pool_config(mut self, config: PoolDefragmentationConfig) -> Self {
        self.pool_configs.push(config);
        self
    }

    /// Get pool-specific configuration or fallback to global
    pub fn get_pool_config(&self, pool_id: &str) -> &PoolDefragmentationConfig {
        self.pool_configs
            .iter()
            .find(|config| config.pool_id == pool_id)
            .unwrap_or_else(|| {
                // Create a fallback config based on global settings
                static FALLBACK: std::sync::OnceLock<PoolDefragmentationConfig> = std::sync::OnceLock::new();
                FALLBACK.get_or_init(|| PoolDefragmentationConfig {
                    pool_id: "default".to_string(),
                    algorithm: self.algorithm.clone(),
                    fragmentation_threshold: self.fragmentation_threshold,
                    cycle_interval: self.cycle_interval,
                })
            })
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.fragmentation_threshold < 0.0 || self.fragmentation_threshold > 1.0 {
            return Err("fragmentation_threshold must be between 0.0 and 1.0".to_string());
        }

        if self.max_memory_pressure < 0.0 || self.max_memory_pressure > 1.0 {
            return Err("max_memory_pressure must be between 0.0 and 1.0".to_string());
        }

        if self.max_concurrent_tasks == 0 {
            return Err("max_concurrent_tasks must be greater than 0".to_string());
        }

        if self.cycle_interval < self.max_cycle_duration {
            return Err("cycle_interval must be greater than max_cycle_duration".to_string());
        }

        Ok(())
    }
}
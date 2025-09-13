//! Core configuration management functionality
//!
//! This module provides the main ConfigurationManager and trait definitions
//! for working with configurations across the Rust AI IDE ecosystem.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::audit::AuditTrail;
use crate::cache::ConfigCache;
use crate::hot_reload::HotReloadManager;
use crate::security::SecurityValidator;
use crate::sources::ConfigSource;
use crate::validation::ValidationEngine;

/// Core trait for configuration types
///
/// This trait defines the interface that all configuration structures must implement
/// to work with the unified configuration system.
#[async_trait]
pub trait Config: Send + Sync + Clone + serde::Serialize + serde::de::DeserializeOwned + 'static {
    /// File prefix for configuration files (e.g., "rust-ai-ide")
    const FILE_PREFIX: &'static str;

    /// Human-readable description of the configuration
    const DESCRIPTION: &'static str;

    /// Validate the configuration instance
    ///
    /// Return a vector of validation error messages, empty vector means valid
    fn validate(&self) -> Result<Vec<String>, anyhow::Error>;

    /// Create a default configuration instance
    fn default_config() -> Self;

    /// Get configuration schema (optional for advanced use)
    fn get_schema() -> Option<serde_json::Value> {
        None
    }
}

/// Configuration source types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigSourceType {
    /// File-based configuration
    File(PathBuf),
    /// Environment variables
    Environment,
    /// Database configuration
    Database,
    /// In-memory configuration (for testing)
    Memory,
}

/// Priority levels for configuration sources
/// Higher values take precedence over lower values
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConfigSourcePriority {
    Default     = 0,
    File        = 10,
    Environment = 20,
    Database    = 30,
    Override    = 100,
}

/// Unified Configuration Manager
///
/// The main entry point for configuration management that provides:
/// - Multi-source configuration loading
/// - Security validation
/// - Audit trails
/// - Hot reloading
/// - Intelligent caching
pub struct ConfigurationManager {
    /// Configuration sources by priority
    sources:            HashMap<ConfigSourcePriority, Vec<Arc<dyn ConfigSource>>>,
    /// Security validator
    security_validator: Arc<SecurityValidator>,
    /// Audit trail recorder
    audit_trail:        Arc<AuditTrail>,
    /// Configuration cache
    cache:              Arc<ConfigCache>,
    /// Hot reload manager
    hot_reload:         Arc<HotReloadManager>,
    /// Validation engine
    validator:          Arc<ValidationEngine>,
    /// Manager configuration
    config:             ManagerConfig,
}

/// Configuration manager settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerConfig {
    /// Enable audit logging
    pub enable_audit:       bool,
    /// Enable hot reloading
    pub enable_hot_reload:  bool,
    /// Enable caching
    pub enable_cache:       bool,
    /// Security validation level
    pub security_level:     SecurityLevel,
    /// Configuration directories to scan
    pub config_directories: Vec<PathBuf>,
    /// Environment variable prefix
    pub env_prefix:         String,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self {
            enable_audit:       true,
            enable_hot_reload:  true,
            enable_cache:       true,
            security_level:     SecurityLevel::High,
            config_directories: vec![
                PathBuf::from("./config"),
                dirs::config_dir()
                    .unwrap_or_else(|| PathBuf::from("./config"))
                    .join("rust-ai-ide"),
            ],
            env_prefix:         "RUST_AI_IDE".to_string(),
        }
    }
}

/// Security validation levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Basic validation (syntax + basic security checks)
    Basic,
    /// Standard validation (Basic + threat detection)
    Standard,
    /// High validation (Standard + comprehensive security checks)
    High,
    /// Paranoid validation (High + zero-trust approach)
    Paranoid,
}

impl Default for ConfigurationManager {
    fn default() -> Self {
        Self {
            sources:            HashMap::new(),
            security_validator: Arc::new(crate::SecurityValidator::new(SecurityLevel::High)),
            audit_trail:        Arc::new(crate::AuditTrail::disabled()),
            cache:              Arc::new(crate::ConfigCache::disabled()),
            hot_reload:         Arc::new(crate::HotReloadManager::disabled()),
            validator:          Arc::new(crate::ValidationEngine::new()),
            config:             ManagerConfig::default(),
        }
    }
}

impl ConfigurationManager {
    /// Create a new configuration manager with default settings
    pub async fn new() -> crate::IDEResult<Self> {
        Self::new_with_config(ManagerConfig::default()).await
    }

    /// Create a new configuration manager with custom configuration
    pub async fn new_with_config(config: ManagerConfig) -> crate::IDEResult<Self> {
        let security_validator = Arc::new(SecurityValidator::new(config.security_level));
        let audit_trail = Arc::new(AuditTrail::new().await?);
        let cache = Arc::new(ConfigCache::new().await?);
        let validator = Arc::new(ValidationEngine::new());

        // Initialize hot reload manager
        let hot_reload = if config.enable_hot_reload {
            Arc::new(HotReloadManager::new().await?)
        } else {
            Arc::new(HotReloadManager::disabled())
        };

        Ok(Self {
            sources: HashMap::new(),
            security_validator,
            audit_trail,
            cache,
            hot_reload,
            validator,
            config,
        })
    }

    /// Add a configuration source
    pub fn add_source<T: ConfigSource + 'static>(&mut self, priority: ConfigSourcePriority, source: T) {
        self.sources
            .entry(priority)
            .or_insert_with(Vec::new)
            .push(Arc::new(source));
    }

    /// Load configuration with full security validation and auditing
    pub async fn load_secure<C>(&self, name: &str) -> crate::IDEResult<C>
    where
        C: Config,
    {
        // Check cache first
        if self.config.enable_cache {
            if let Some(cached) = self.cache.get::<C>(name).await? {
                return Ok(cached);
            }
        }

        // Load from sources in priority order
        let mut result = C::default_config();
        let mut source_count = 0;

        for priority in (0..=100).rev() {
            let priority = match priority {
                0 => ConfigSourcePriority::Default,
                10 => ConfigSourcePriority::File,
                20 => ConfigSourcePriority::Environment,
                30 => ConfigSourcePriority::Database,
                100 => ConfigSourcePriority::Override,
                _ => continue,
            };

            if let Some(sources) = self.sources.get(&priority) {
                for source in sources {
                    if let Some(config_value) = source.load(name).await? {
                        let config = serde_json::from_value::<C>(config_value)?;
                        result = self.merge_configs(result, config)?;
                        source_count += 1;
                    }
                }
            }
        }

        // Validate configuration
        self.validator.validate(&result)?;

        // Security validation
        self.security_validator.validate_config(&result)?;

        // Cache the result
        if self.config.enable_cache {
            self.cache.put(name, result.clone()).await?;
        }

        // Audit the load
        if self.config.enable_audit {
            self.audit_trail
                .record_load(name, source_count, "secure_load")
                .await?;
        }

        Ok(result)
    }

    /// Save configuration with security validation and auditing
    pub async fn save_secure<C>(&mut self, name: &str, config: C) -> crate::IDEResult<()>
    where
        C: Config,
    {
        // Validate configuration
        self.validator.validate(&config)?;

        // Security validation
        self.security_validator.validate_config(&config)?;

        // Save to highest priority writable source
        let mut saved = false;
        for priority in (0..=100).rev() {
            let priority = match priority {
                0 => ConfigSourcePriority::Default,
                10 => ConfigSourcePriority::File,
                20 => ConfigSourcePriority::Environment,
                30 => ConfigSourcePriority::Database,
                100 => ConfigSourcePriority::Override,
                _ => continue,
            };

            if let Some(sources) = self.sources.get(&priority) {
                for source in sources {
                    if source.can_save() {
                        let config_value = serde_json::to_value(&config)?;
                        source.save(name, &config_value).await?;
                        saved = true;
                        break;
                    }
                }
            }
            if saved {
                break;
            }
        }

        if !saved {
            return Err(crate::RustAIError::Config(
                rust_ai_ide_errors::ConfigError::new("No writable configuration source available"),
            ));
        }

        // Update cache
        if self.config.enable_cache {
            self.cache.put(name, config).await?;
        }

        // Clear cache for this config
        if self.config.enable_cache {
            self.cache.invalidate_config(name).await?;
        }

        // Audit the save
        if self.config.enable_audit {
            self.audit_trail.record_save(name, "secure_save").await?;
        }

        Ok(())
    }

    /// Get configuration sources status
    pub fn get_sources_info(&self) -> Vec<String> {
        let mut info = Vec::new();

        for (priority, sources) in &self.sources {
            info.push(format!(
                "Priority {:?}: {} sources",
                priority,
                sources.len()
            ));
        }

        info
    }

    /// Merge two configuration instances (higher priority wins)
    fn merge_configs<C>(&self, base: C, override_: C) -> crate::IDEResult<C>
    where
        C: Config,
    {
        // For now, use serde_json to merge - could be enhanced with more sophisticated merging
        let base_json = serde_json::to_value(&base)?;
        let override_json = serde_json::to_value(&override_)?;

        let merged = Self::json_merge(base_json, override_json)?;
        let result = serde_json::from_value(merged)?;

        Ok(result)
    }

    /// Simple JSON merge (override wins)
    fn json_merge(base: serde_json::Value, override_: serde_json::Value) -> serde_json::Result<serde_json::Value> {
        match (base, override_) {
            (serde_json::Value::Object(mut base_obj), serde_json::Value::Object(override_obj)) => {
                for (key, value) in override_obj {
                    base_obj.insert(key, value);
                }
                Ok(serde_json::Value::Object(base_obj))
            }
            (_, override_) => Ok(override_),
        }
    }
}

// Test implementations
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestConfig {
        value:  String,
        number: i32,
    }

    impl Config for TestConfig {
        const FILE_PREFIX: &'static str = "test";
        const DESCRIPTION: &'static str = "Test Configuration";

        fn validate(&self) -> Result<Vec<String>, anyhow::Error> {
            let mut errors = Vec::new();
            if self.number < 0 {
                errors.push("number must be >= 0".to_string());
            }
            Ok(errors)
        }

        fn default_config() -> Self {
            Self {
                value:  "default".to_string(),
                number: 42,
            }
        }
    }

    #[tokio::test]
    async fn test_config_manager_creation() {
        let manager = ConfigurationManager::new().await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_secure_load_default() {
        let manager = ConfigurationManager::new().await.unwrap();
        let config: TestConfig = manager.load_secure("test").await.unwrap();

        assert_eq!(config.value, "default");
        assert_eq!(config.number, 42);
    }
}

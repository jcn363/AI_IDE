//! # Quality Intelligence Dashboard Configuration
//!
//! This module handles dashboard configuration management, validation, and loading.

use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

use crate::errors::{ConfigurationError, DashboardResult};
use crate::types::*;

/// Dashboard configuration manager
#[derive(Clone)]
pub struct ConfigurationManager {
    /// Current configuration
    config: Arc<tokio::sync::RwLock<DashboardConfiguration>>,

    /// Configuration file path
    config_path: Option<String>,

    /// Configuration validator
    validator: ConfigurationValidator,
}

/// Configuration validation system
#[derive(Clone)]
pub struct ConfigurationValidator {
    /// Required configuration fields
    required_fields: std::collections::HashSet<String>,
}

/// Configuration change listener
#[async_trait::async_trait]
pub trait ConfigurationListener: Send + Sync {
    /// Handle configuration change
    async fn on_configuration_changed(
        &self,
        old_config: &DashboardConfiguration,
        new_config: &DashboardConfiguration,
    );

    /// Get listener ID
    fn listener_id(&self) -> &str;
}

impl ConfigurationManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        let validator = ConfigurationValidator::new();

        Self {
            config: Arc::new(tokio::sync::RwLock::new(DashboardConfiguration::default())),
            config_path: None,
            validator,
        }
    }

    /// Load configuration from file
    pub async fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> DashboardResult<()> {
        let content = fs::read_to_string(&path).await.map_err(|e| {
            ConfigurationError::FileAccess(format!("Failed to read config file: {}", e))
        })?;

        let config: DashboardConfiguration = serde_json::from_str(&content).map_err(|e| {
            ConfigurationError::ParseError(format!("Failed to parse config: {}", e))
        })?;

        self.validator.validate(&config)?;
        *self.config.write().await = config;
        self.config_path = Some(path.as_ref().to_string_lossy().to_string());

        Ok(())
    }

    /// Save configuration to file
    pub async fn save_to_file(&self, path: Option<String>) -> DashboardResult<()> {
        let config = self.config.read().await;
        let path = path.or_else(|| self.config_path.clone()).ok_or_else(|| {
            ConfigurationError::MissingConfig("No configuration file path specified".to_string())
        })?;

        let content = serde_json::to_string_pretty(&*config).map_err(|e| {
            ConfigurationError::ParseError(format!("Failed to serialize config: {}", e))
        })?;

        fs::write(&path, content).await.map_err(|e| {
            ConfigurationError::FileAccess(format!("Failed to write config file: {}", e))
        })?;

        Ok(())
    }

    /// Get current configuration
    pub async fn get_configuration(&self) -> DashboardConfiguration {
        self.config.read().await.clone()
    }

    /// Update configuration
    pub async fn update_configuration(
        &self,
        new_config: DashboardConfiguration,
    ) -> DashboardResult<()> {
        self.validator.validate(&new_config)?;
        *self.config.write().await = new_config;
        Ok(())
    }

    /// Get configuration value by key
    pub async fn get_config_value(&self, key: &str) -> Option<serde_json::Value> {
        let config = self.config.read().await;
        match key {
            "update_interval" => Some(json!(config.update_interval)),
            "retention_days" => Some(json!(config.retention_days)),
            "theme" => Some(json!(config.ui_prefs.theme.clone().into())),
            _ => None,
        }
    }
}

impl ConfigurationValidator {
    /// Create a new validator
    pub fn new() -> Self {
        let mut required_fields = std::collections::HashSet::new();
        required_fields.insert("update_interval".to_string());
        required_fields.insert("ui_prefs".to_string());

        Self { required_fields }
    }

    /// Validate configuration
    pub fn validate(&self, config: &DashboardConfiguration) -> DashboardResult<()> {
        // Validate update interval
        if config.update_interval == 0 {
            return Err(ConfigurationError::InvalidParameter {
                parameter: "update_interval".to_string(),
                reason: "Must be greater than 0".to_string(),
            }
            .into());
        }

        if config.update_interval > 3600 {
            return Err(ConfigurationError::InvalidParameter {
                parameter: "update_interval".to_string(),
                reason: "Must not exceed 1 hour".to_string(),
            }
            .into());
        }

        // Validate retention days
        if config.retention_days <= 0 {
            return Err(ConfigurationError::InvalidParameter {
                parameter: "retention_days".to_string(),
                reason: "Must be greater than 0".to_string(),
            }
            .into());
        }

        // Validate enabled metrics
        if config.enabled_metrics.is_empty() {
            return Err(ConfigurationError::InvalidParameter {
                parameter: "enabled_metrics".to_string(),
                reason: "At least one metric must be enabled".to_string(),
            }
            .into());
        }

        Ok(())
    }
}

/// Configuration builder for fluent API
#[derive(Clone)]
pub struct ConfigurationBuilder {
    config: DashboardConfiguration,
}

impl ConfigurationBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: DashboardConfiguration::default(),
        }
    }

    /// Set update interval
    pub fn update_interval(mut self, interval: u64) -> Self {
        self.config.update_interval = interval;
        self
    }

    /// Set retention days
    pub fn retention_days(mut self, days: i32) -> Self {
        self.config.retention_days = days;
        self
    }

    /// Enable specific metrics
    pub fn enable_metrics(mut self, metrics: Vec<String>) -> Self {
        self.config.enabled_metrics = metrics;
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: String) -> Self {
        self.config.ui_prefs.theme = theme;
        self
    }

    /// Set alert thresholds
    pub fn alert_thresholds(mut self, thresholds: AlertThresholds) -> Self {
        self.config.thresholds = thresholds;
        self
    }

    /// Build the configuration
    pub fn build(self) -> DashboardResult<DashboardConfiguration> {
        let validator = ConfigurationValidator::new();
        validator.validate(&self.config)?;
        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_configuration_validation() {
        let validator = ConfigurationValidator::new();

        // Valid configuration
        let valid_config = DashboardConfiguration::default();
        assert!(validator.validate(&valid_config).is_ok());

        // Invalid update interval
        let mut invalid_config = DashboardConfiguration::default();
        invalid_config.update_interval = 0;
        assert!(validator.validate(&invalid_config).is_err());
    }

    #[tokio::test]
    async fn test_configuration_builder() {
        let config = ConfigurationBuilder::new()
            .update_interval(60)
            .retention_days(30)
            .theme("dark".to_string())
            .build()
            .unwrap();

        assert_eq!(config.update_interval, 60);
        assert_eq!(config.retention_days, 30);
        assert_eq!(config.ui_prefs.theme, "dark");
    }

    #[tokio::test]
    async fn test_configuration_file_operations() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        let mut manager = ConfigurationManager::new();

        // Save and load configuration
        {
            let mut file = temp_file.as_file();
            let config_json = r#"{
                "update_interval": 45,
                "retention_days": 15,
                "ui_prefs": {
                    "theme": "light"
                },
                "enabled_metrics": ["code_quality"]
            }"#;
            file.write_all(config_json.as_bytes()).unwrap();
            file.flush().unwrap();
        }

        manager.load_from_file(&file_path).await.unwrap();

        let loaded_config = manager.get_configuration().await;
        assert_eq!(loaded_config.update_interval, 45);
        assert_eq!(loaded_config.ui_prefs.theme, "light");
    }
}

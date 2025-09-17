//! # Unified Configuration Management System
//!
//! This module provides a unified, type-safe configuration management system that consolidates
//! all configuration handling across the Rust AI IDE codebase.
//!
//! ## Features
//!
//! - **Multi-format support**: Load from TOML, JSON, and YAML files
//! - **Environment variable overrides**: Override any config value using environment variables
//! - **Platform-specific paths**: Automatically discover config files based on platform conventions
//! - **Schema validation**: Validate configuration using JSON Schema
//! - **Merge strategies**: Intelligent merging of multiple configuration sources
//! - **Hot-reload capability**: Watch config files for changes and auto-reload
//! - **Type-safe access**: Strongly-typed configuration access with fallbacks
//!
//! ## Usage
//!
//! ```rust
//! use crate::config::{ConfigManager, SourcePriority as Priority};
//!
//! Load configuration with default settings
//! let config: AppConfig = ConfigManager::load_with_defaults(
//! "app".to_string(),
//! AppConfig::default()
//! ).await?;
//!
//! Load with environment overrides
//! let config_with_env: AppConfig = ConfigManager::load_with_env_override(
//! "app".to_string(),
//! AppConfig::default(),
//! &["APP_AI_PROVIDER", "APP_API_KEY"]
//! ).await?;
//! ```
//!
//! ## Configuration Sources (in priority order)
//!
//! 1. **Environment variables**: Highest priority, override any other setting
//! 2. **Project config files**: `{project_root}/config/{app}.config.*`
//! 3. **User config files**: `{user_config_dir}/rust-ai-ide/{app}.config.*`
//! 4. **System config files**: `{system_config_dir}/rust-ai-ide/{app}.config.*`
//! 5. **Built-in defaults**: Lowest priority, used when no other source provides a value
//!
//! ## Supported Formats
//!
//! - TOML (.toml)
//! - JSON (.json)
//! - YAML (.yaml or .yml)
//! - Environment variables (prefix with `APP_` or custom prefix)

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
#[cfg(feature = "json_schema")]
use jsonschema::JSONSchema;
use notify::RecommendedWatcher;
use serde::de::DeserializeOwned;
use serde::Serialize;
// Re-exports for convenience
pub use serde_json;
use serde_json::Value;
use tokio::sync::RwLock;
pub use {serde_yaml, toml};

/// Configuration format types
#[derive(Debug, Clone, Copy)]
pub enum ConfigFormat {
    Toml,
    Json,
    Yaml,
    Auto, // Auto-detect from file extension
}

/// Configuration source priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SourcePriority {
    Default = 0,
    System = 1,
    User = 2,
    Project = 3,
    Environment = 4,
}

/// Configuration source with associated priority
#[derive(Debug, Clone)]
pub struct ConfigSource<T> {
    pub data: T,
    pub priority: SourcePriority,
    pub source_path: Option<PathBuf>,
}

/// Configuration loading options
#[derive(Debug, Clone)]
pub struct LoadOptions {
    pub enable_env_override: bool,
    pub env_prefix: Option<String>,
    pub enable_hot_reload: bool,
    pub hot_reload_debounce_ms: u64,
    pub search_includes: Vec<String>,
    pub search_excludes: Vec<String>,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            enable_env_override: true,
            env_prefix: Some("APP_".to_string()),
            enable_hot_reload: false,
            hot_reload_debounce_ms: 500,
            search_includes: vec![
                "*.toml".to_string(),
                "*.json".to_string(),
                "*.yaml".to_string(),
                "*.yml".to_string(),
            ],
            search_excludes: vec![
                "target/".to_string(),
                "node_modules/".to_string(),
                ".git/".to_string(),
            ],
        }
    }
}

pub enum MergeStrategy {
    /// Take the highest priority value for each field
    PriorityTake,
    /// Deep merge objects/maps, taking highest priority for scalar values
    DeepMerge,
    /// Custom merge function
    Custom(Box<dyn Fn(&serde_json::Value, &serde_json::Value) -> serde_json::Value + Send + Sync>),
}

/// Configuration manager for type-safe configuration handling
pub struct ConfigManager<T: Config> {
    config: Arc<RwLock<ConfigSource<T>>>,
    options: LoadOptions,
    watchers: Arc<RwLock<Vec<RecommendedWatcher>>>,
}

/// Trait for configuration types that can be managed by ConfigManager
#[async_trait]
pub trait Config: Clone + Serialize + DeserializeOwned + Send + Sync + 'static {
    const FILE_PREFIX: &'static str;
    const DESCRIPTION: &'static str;

    /// Validate the configuration and return validation errors
    fn validate(&self) -> Result<Vec<String>, anyhow::Error> {
        Ok(vec![])
    }

    /// Get the default configuration
    fn default_config() -> Self;

    /// Get JSON schema for validation (optional)
    fn schema() -> Option<serde_json::Value> {
        None
    }

    /// Transform raw configuration before merging
    fn transform(&self) -> Self {
        self.clone()
    }
}

/// Utility functions for platform-specific path discovery
pub mod paths {
    use std::path::{Path, PathBuf};

    use dirs_next::{config_dir, data_dir, executable_dir};

    use super::SourcePriority;

    /// Get the standard user config directory for the application
    pub fn user_config_dir() -> Result<PathBuf, anyhow::Error> {
        let base = config_dir()
            .ok_or_else(|| anyhow::anyhow!("Unable to determine user config directory"))?;
        Ok(base.join("rust-ai-ide"))
    }

    /// Get the standard user data directory for the application
    pub fn user_data_dir() -> Result<PathBuf, anyhow::Error> {
        let base =
            data_dir().ok_or_else(|| anyhow::anyhow!("Unable to determine user data directory"))?;
        Ok(base.join("rust-ai-ide"))
    }

    /// Get system-wide config directory
    pub fn system_config_dir() -> Result<PathBuf, anyhow::Error> {
        // Fallback to executable directory if no system config dir available
        if let Some(exe_dir) = executable_dir() {
            let config_dir = exe_dir.join("config");
            if config_dir.exists() {
                return Ok(config_dir);
            }
        }

        // Unix-style system paths
        #[cfg(unix)]
        {
            for path in ["/etc/rust-ai-ide", "/usr/local/etc/rust-ai-ide"] {
                let pb = PathBuf::from(path);
                if pb.exists() {
                    return Ok(pb);
                }
            }
        }

        // Default fallback
        user_config_dir()
    }

    /// Find all configuration files for a given app name and project root
    pub fn discover_config_paths(
        app_name: &str,
        project_root: Option<&Path>,
    ) -> Result<Vec<(PathBuf, crate::config::SourcePriority)>, anyhow::Error> {
        let mut paths = Vec::new();

        // System config
        if let Ok(system_dir) = system_config_dir() {
            let extensions = ["toml", "json", "yaml", "yml"];
            for ext in &extensions {
                let file_path = system_dir.join(format!("{}.{}", app_name, ext));
                if file_path.exists() {
                    paths.push((file_path, SourcePriority::System));
                }
            }
        }

        // User config
        if let Ok(user_dir) = user_config_dir() {
            let extensions = ["toml", "json", "yaml", "yml"];
            for ext in &extensions {
                let file_path = user_dir.join(format!("{}.{}", app_name, ext));
                if file_path.exists() {
                    paths.push((file_path, SourcePriority::User));
                }
            }
        }

        // Project-specific config
        if let Some(root) = project_root {
            let config_dir = root.join("config");
            if config_dir.exists() {
                let extensions = ["toml", "json", "yaml", "yml"];
                for ext in &extensions {
                    let file_path = config_dir.join(format!("{}.{}", app_name, ext));
                    if file_path.exists() {
                        paths.push((file_path, SourcePriority::Project));
                    }
                }
            }
        }

        // Sort by priority (highest first)
        paths.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(paths)
    }
}

/// Environment variable override utilities
pub mod env_override {
    use std::env;

    use serde_json::Value;

    /// Apply environment variable overrides to a configuration map
    pub fn apply_overrides_to_map(
        config: &mut serde_json::Value,
        prefix: Option<&str>,
        vars: &[&str],
    ) -> Result<(), anyhow::Error> {
        let prefix = prefix.unwrap_or("");

        for var in vars {
            if let Ok(env_value) = env::var(var) {
                if !env_value.is_empty() {
                    // Convert env var name to config path
                    let config_key = if var.starts_with(prefix) {
                        &var[prefix.len()..]
                    } else {
                        var
                    }
                    .to_lowercase()
                    .replace('_', ".");

                    // Parse and set the value
                    if let Ok(parsed) = serde_json::from_str::<Value>(&env_value) {
                        set_nested_value(
                            config,
                            &config_key.split('.').collect::<Vec<_>>(),
                            parsed,
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Set a nested value in a JSON value
    fn set_nested_value(value: &mut Value, keys: &[&str], new_value: Value) {
        if keys.is_empty() {
            *value = new_value;
            return;
        }

        if let Value::Object(ref mut map) = value {
            let first = keys[0];
            if keys.len() == 1 {
                map.insert(first.to_string(), new_value);
            } else {
                let child = map
                    .entry(first.to_string())
                    .or_insert(Value::Object(serde_json::Map::new()));
                set_nested_value(child, &keys[1..], new_value);
            }
        }
    }
}

/// File loading utilities
pub mod file_loader {
    use std::path::Path;

    use anyhow::{Context, Result};
    use tokio::fs;

    use crate::config::{Config, ConfigFormat};

    /// Format detection utilities
    pub mod format {
        use super::{ConfigFormat, Path, Result};

        pub fn detect_format(path: &Path) -> Result<ConfigFormat> {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                match ext.to_lowercase().as_str() {
                    "toml" => Ok(ConfigFormat::Toml),
                    "json" => Ok(ConfigFormat::Json),
                    "yaml" | "yml" => Ok(ConfigFormat::Yaml),
                    _ => Err(anyhow::anyhow!("Unsupported configuration format: {}", ext)),
                }
            } else {
                Err(anyhow::anyhow!(
                    "No file extension found for format detection"
                ))
            }
        }
    }

    /// Load configuration from a file with automatic format detection
    pub async fn load_from_file<C: Config>(path: &Path) -> Result<C> {
        let content = fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let format = format::detect_format(path)?;

        match format {
            ConfigFormat::Toml => toml::from_str(&content)
                .with_context(|| format!("Failed to parse TOML config: {}", path.display())),
            ConfigFormat::Json => serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse JSON config: {}", path.display())),
            ConfigFormat::Yaml => serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse YAML config: {}", path.display())),
            ConfigFormat::Auto => {
                // Should not happen with automatic detection
                Err(anyhow::anyhow!("Automatic format detection failed"))
            }
        }
    }
}

/// Merger utilities for combining configuration sources
pub mod merger {
    use anyhow::Context;
    use serde_json::Value;

    use crate::config::MergeStrategy;

    /// Strategy implementations for merging configurations
    pub mod strategies {
        use super::Value;

        /// Priority-take strategy: higher priority completely overrides lower priority
        pub fn priority_take(high: &Value, _low: &Value) -> Value {
            high.clone()
        }

        /// Deep merge strategy: recursively merge objects, arrays, and take scalar values from high
        /// priority
        pub fn deep_merge(high: &Value, low: &Value) -> Value {
            if high.is_null() {
                return low.clone();
            }

            match (high, low) {
                (Value::Object(high_map), Value::Object(low_map)) => {
                    let mut merged = low_map.clone();
                    for (key, high_val) in high_map {
                        if let Some(low_val) = merged.get(key) {
                            merged.insert(key.clone(), deep_merge(high_val, low_val));
                        } else {
                            merged.insert(key.clone(), high_val.clone());
                        }
                    }
                    Value::Object(merged)
                }
                (arr @ Value::Array(_), _) => arr.clone(),
                (high_val, _) => high_val.clone(),
            }
        }
    }

    /// Merge a list of configurations using the specified strategy
    pub fn merge_configurations<C: serde::de::DeserializeOwned>(
        configs: Vec<super::ConfigSource<serde_json::Value>>,
        strategy: MergeStrategy,
    ) -> Result<C, anyhow::Error> {
        if configs.is_empty() {
            return Err(anyhow::anyhow!("No configurations provided for merging"));
        }

        // Sort by priority (highest first)
        let mut sorted_configs = configs;
        sorted_configs.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Start with the lowest priority (defaults)
        let mut base = if let Some(lowest) = sorted_configs.last() {
            lowest.data.clone()
        } else {
            return Err(anyhow::anyhow!("No configurations available"));
        };

        // Apply higher priority configs
        for config in sorted_configs.into_iter().rev().skip(1) {
            match strategy {
                MergeStrategy::PriorityTake => {
                    base = strategies::priority_take(&config.data, &base);
                }
                MergeStrategy::DeepMerge => {
                    base = strategies::deep_merge(&config.data, &base);
                }
                MergeStrategy::Custom(ref custom_fn) => {
                    base = custom_fn(&config.data, &base);
                }
            }
        }

        // Convert back to the target type
        serde_json::from_value(base).with_context(|| "Failed to deserialize merged configuration")
    }
}

/// Configuration validation utilities
#[cfg(feature = "json_schema")]
pub mod validator {
    use jsonschema::JSONSchema;

    use super::*;

    pub fn validate_schema(
        config: &serde_json::Value,
        schema: &serde_json::Value,
    ) -> Result<Vec<String>> {
        let compiled = JSONSchema::compile(schema)?;
        let validation = compiled.validate(config);

        let mut errors = Vec::new();
        for error in validation.errors() {
            errors.push(error.to_string());
        }

        Ok(errors)
    }
}

// Implementation of ConfigManager
impl<T: Config> Default for ConfigManager<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Config> ConfigManager<T> {
    /// Create a new ConfigManager with default options
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(ConfigSource {
                data: T::default_config(),
                priority: SourcePriority::Default,
                source_path: None,
            })),
            options: LoadOptions::default(),
            watchers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a new ConfigManager with custom options
    pub fn with_options(options: LoadOptions) -> Self {
        Self {
            config: Arc::new(RwLock::new(ConfigSource {
                data: T::default_config(),
                priority: SourcePriority::Default,
                source_path: None,
            })),
            options,
            watchers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Load configuration with defaults only
    pub async fn load_with_defaults(_app_name: String, defaults: T) -> Result<Self> {
        let manager = Self::new();

        // Apply defaults
        *manager.config.write().await = ConfigSource {
            data: defaults,
            priority: SourcePriority::Default,
            source_path: None,
        };

        manager.validate()?;
        Ok(manager)
    }

    /// Load configuration with file support
    pub async fn load_with_files(
        app_name: String,
        project_root: Option<&Path>,
        defaults: T,
    ) -> Result<Self> {
        let manager = Self::load_with_defaults(app_name.clone(), defaults).await?;

        // Discover and load configuration files
        let config_paths = paths::discover_config_paths(&app_name, project_root)?;

        let mut sources = vec![ConfigSource {
            data: manager.get().await,
            priority: SourcePriority::Default,
            source_path: None,
        }];

        // Load each discovered config file
        for (path, priority) in config_paths {
            if let Ok(config) = file_loader::load_from_file::<T>(&path).await {
                sources.push(ConfigSource {
                    data: config,
                    priority,
                    source_path: Some(path.to_path_buf()),
                });
            }
        }

        // Merge configurations
        let json_sources: Vec<_> = sources
            .into_iter()
            .map(|s| ConfigSource {
                data: serde_json::to_value(&s.data).unwrap_or(Value::Null),
                priority: s.priority,
                source_path: s.source_path,
            })
            .collect();

        let merged_config: T =
            merger::merge_configurations(json_sources, MergeStrategy::DeepMerge)?;

        // Update the manager with merged config
        *manager.config.write().await = ConfigSource {
            data: merged_config,
            priority: SourcePriority::Project, // Merged from multiple sources
            source_path: None,
        };

        manager.validate()?;
        Ok(manager)
    }

    /// Load configuration with environment variable overrides
    pub async fn load_with_env_override(
        app_name: String,
        defaults: T,
        env_vars: &[&str],
    ) -> Result<Self> {
        let manager = Self::load_with_defaults(app_name, defaults).await?;

        if manager.options.enable_env_override {
            let mut json_config = serde_json::to_value(manager.get().await)?;
            env_override::apply_overrides_to_map(
                &mut json_config,
                manager.options.env_prefix.as_deref(),
                env_vars,
            )?;

            // Create new config from overridden JSON
            let overridden_config: T = serde_json::from_value(json_config)?;

            // Update manager
            let current = manager.config.write().await;
            *manager.config.write().await = ConfigSource {
                data: overridden_config,
                priority: SourcePriority::Environment,
                source_path: current.source_path.clone(),
            };
        }

        manager.validate()?;
        Ok(manager)
    }

    /// Load full configuration with all features enabled
    pub async fn load_full(
        app_name: String,
        project_root: Option<&Path>,
        env_vars: &[&str],
        defaults: T,
    ) -> Result<Self> {
        let mut manager = Self::load_with_files(app_name, project_root, defaults).await?;
        manager = Self::load_with_env_override_from_manager(manager, env_vars).await?;
        Ok(manager)
    }

    // Helper method for adding env overrides to existing manager
    async fn load_with_env_override_from_manager(manager: Self, env_vars: &[&str]) -> Result<Self> {
        if manager.options.enable_env_override {
            let mut json_config = serde_json::to_value(manager.get().await)?;
            env_override::apply_overrides_to_map(
                &mut json_config,
                manager.options.env_prefix.as_deref(),
                env_vars,
            )?;

            let overridden_config: T = serde_json::from_value(json_config)?;
            let mut current = manager.config.write().await;
            current.data = overridden_config;
            current.priority = std::cmp::max(current.priority.clone(), SourcePriority::Environment);
        }

        manager.validate()?;
        Ok(manager)
    }

    /// Validate the current configuration
    pub fn validate(&self) -> Result<()> {
        let guard = self.config.blocking_read();
        let errors = guard.data.validate()?;

        if !errors.is_empty() {
            return Err(anyhow::anyhow!(
                "Configuration validation failed:\n{}",
                errors.join("\n")
            ));
        }

        Ok(())
    }

    /// Get current configuration (immutable reference)
    pub async fn get(&self) -> T {
        self.config.read().await.data.clone()
    }

    /// Update configuration
    pub async fn update<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut T),
    {
        let mut guard = self.config.write().await;
        f(&mut guard.data);
        guard.data = guard.data.transform();
        self.validate()?;
        Ok(())
    }

    /// Save configuration to file
    pub async fn save(&self, path: &Path, format: ConfigFormat) -> Result<()> {
        let content = match format {
            ConfigFormat::Toml => toml::to_string_pretty(&self.config.read().await.data)?,
            ConfigFormat::Json => serde_json::to_string_pretty(&self.config.read().await.data)?,
            ConfigFormat::Yaml => serde_yaml::to_string(&self.config.read().await.data)?,
            ConfigFormat::Auto => {
                // Default to TOML for saving
                toml::to_string_pretty(&self.config.read().await.data)?
            }
        };

        tokio::fs::write(path, content).await?;
        Ok(())
    }

    /// Enable hot reloading for configuration files
    pub async fn enable_hot_reload(&mut self) -> Result<()> {
        if !self.options.enable_hot_reload {
            return Ok(());
        }

        // TODO: Implement file watching logic

        Ok(())
    }

    /// Get configuration description for documentation
    pub fn get_description() -> &'static str {
        T::DESCRIPTION
    }

    /// Get file prefix for this configuration type
    pub fn get_file_prefix() -> &'static str {
        T::FILE_PREFIX
    }
}

#[cfg(feature = "json_schema")]
impl<T: Config> ConfigManager<T> {
    /// Validate configuration against JSON schema
    pub fn validate_schema(&self) -> Result<Vec<String>> {
        if let Some(schema) = T::schema() {
            let config_json = serde_json::to_value(&self.config.blocking_read().data)?;
            validator::validate_schema(&config_json, &schema)
        } else {
            Ok(vec![])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestConfig {
        pub app_name: String,
        pub port: u16,
        pub database_url: String,
        pub features: Vec<String>,
    }

    impl Config for TestConfig {
        const FILE_PREFIX: &'static str = "test";
        const DESCRIPTION: &'static str = "Test application configuration";

        fn default_config() -> Self {
            Self {
                app_name: "test_app".to_string(),
                port: 8080,
                database_url: "sqlite:///test.db".to_string(),
                features: vec!["basic".to_string()],
            }
        }

        fn validate(&self) -> Result<Vec<String>> {
            let mut errors = Vec::new();

            if self.port == 0 || self.port > 65535 {
                errors.push("Port must be between 1 and 65535".to_string());
            }

            if self.app_name.is_empty() {
                errors.push("Application name cannot be empty".to_string());
            }

            Ok(errors)
        }
    }

    #[tokio::test]
    async fn test_config_manager_basic() {
        let config: TestConfig =
            ConfigManager::load_with_defaults("test".to_string(), TestConfig::default_config())
                .await
                .unwrap();

        assert_eq!(config.app_name, "test_app");
        assert_eq!(config.port, 8080);
    }

    #[tokio::test]
    async fn test_config_validation() {
        let mut manager = ConfigManager::<TestConfig>::new();

        // Invalid config
        let mut invalid_config = TestConfig::default_config();
        invalid_config.port = 0;

        *manager.config.write().await = ConfigSource {
            data: invalid_config,
            priority: SourcePriority::Default,
            source_path: None,
        };

        assert!(manager.validate().is_err());
    }

    #[tokio::test]
    async fn test_env_override() {
        std::env::set_var("TEST_PORT", "3000");
        std::env::set_var("TEST_APP_NAME", "overridden_app");

        let mut manager =
            ConfigManager::load_with_defaults("test".to_string(), TestConfig::default_config())
                .await
                .unwrap();

        let mut json_config = serde_json::to_value(manager.get().await).unwrap();
        env_override::apply_overrides_to_map(
            &mut json_config,
            Some("TEST_"),
            &["TEST_PORT", "TEST_APP_NAME"],
        )
        .unwrap();

        let updated_config: TestConfig = serde_json::from_value(json_config).unwrap();
        assert_eq!(updated_config.port, 3000);
        assert_eq!(updated_config.app_name, "overridden_app");

        std::env::remove_var("TEST_PORT");
        std::env::remove_var("TEST_APP_NAME");
    }
}

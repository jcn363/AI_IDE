//!
//! Configuration sources for different input types
//!
//! This module provides implementations for loading configuration from:
//! - Files (TOML, YAML, JSON with auto-detection)
//! - Environment variables (with secure prefixing)
//! - In-memory sources (for testing)

pub mod env;
pub mod file;

pub use env::EnvironmentSource;
pub use file::FileSource;

use async_trait::async_trait;
use std::path::PathBuf;

/// Configuration source trait
///
///
#[async_trait]
pub trait ConfigSource: Send + Sync + std::fmt::Debug {
    /// Load configuration from this source as serde Value
    async fn load(&self, name: &str) -> crate::IDEResult<Option<serde_json::Value>>;

    /// Save configuration to this source from serde Value
    async fn save(&self, name: &str, config: &serde_json::Value) -> crate::IDEResult<()>;

    /// Check if this source can be written to
    fn can_save(&self) -> bool;

    /// Get source priority (higher numbers override lower)
    fn priority(&self) -> super::ConfigSourcePriority;

    /// Get human-readable source description
    fn description(&self) -> String;
}

/// Supported configuration file formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Toml,
    Yaml,
    Json,
}

impl ConfigFormat {
    /// Auto-detect format from file extension
    pub fn from_path(path: &PathBuf) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext.to_lowercase().as_str() {
                "toml" => Some(ConfigFormat::Toml),
                "yaml" | "yml" => Some(ConfigFormat::Yaml),
                "json" => Some(ConfigFormat::Json),
                _ => None,
            })
    }

    /// Auto-detect format from content
    pub fn detect_from_content(content: &str) -> Option<Self> {
        // Try JSON first
        if serde_json::from_str::<serde_json::Value>(content).is_ok() {
            return Some(ConfigFormat::Json);
        }

        // Try YAML
        if serde_yaml::from_str::<serde_yaml::Value>(content).is_ok() {
            return Some(ConfigFormat::Yaml);
        }

        // Try TOML
        if toml::from_str::<toml::Value>(content).is_ok() {
            return Some(ConfigFormat::Toml);
        }

        None
    }
}

/// Common source utilities
pub struct SourceUtils;

impl SourceUtils {
    /// Find configuration files in directories
    pub fn find_config_files(base_name: &str, directories: &[PathBuf]) -> Vec<PathBuf> {
        let mut files = Vec::new();

        for dir in directories {
            for extension in ["toml", "yaml", "yml", "json"] {
                let file_name = format!("{}.{}", base_name, extension);
                let file_path = dir.join(file_name);

                if file_path.exists() {
                    files.push(file_path);
                }
            }
        }

        files
    }

    /// Load and parse configuration from content
    pub fn parse_config<C>(format: ConfigFormat, content: &str) -> crate::IDEResult<C>
    where
        C: crate::Config,
    {
        match format {
            ConfigFormat::Toml => toml::from_str(content).map_err(|e| {
                crate::RustAIError::Serialization(format!("TOML parsing error: {}", e))
            }),
            ConfigFormat::Yaml => serde_yaml::from_str(content).map_err(|e| {
                crate::RustAIError::Serialization(format!("YAML parsing error: {}", e))
            }),
            ConfigFormat::Json => serde_json::from_str(content).map_err(|e| {
                crate::RustAIError::Serialization(format!("JSON parsing error: {}", e))
            }),
        }
    }

    /// Serialize configuration to string
    pub fn serialize_config<C>(format: ConfigFormat, config: &C) -> crate::IDEResult<String>
    where
        C: crate::Config,
    {
        match format {
            ConfigFormat::Toml => toml::to_string(config).map_err(|e| {
                crate::RustAIError::Serialization(format!("TOML serialization error: {}", e))
            }),
            ConfigFormat::Yaml => serde_yaml::to_string(config).map_err(|e| {
                crate::RustAIError::Serialization(format!("YAML serialization error: {}", e))
            }),
            ConfigFormat::Json => serde_json::to_string_pretty(config).map_err(|e| {
                crate::RustAIError::Serialization(format!("JSON serialization error: {}", e))
            }),
        }
    }
}

//! Configuration system for shared types functionality
//!
//! Provides flexible configuration management with presets, validation,
//! and seamless integration with the unified configuration system.

use crate::errors::TypeGenerationError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main generation configuration containing all settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GenerationConfig {
    /// TypeScript generation settings
    pub typescript: TypeScriptConfig,

    /// Plugin system configuration
    #[cfg(feature = "plugins")]
    pub plugins: PluginConfig,

    /// Caching configuration
    pub cache: CacheConfig,

    /// General generation settings
    pub general: GeneralConfig,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            typescript: TypeScriptConfig::default(),
            #[cfg(feature = "plugins")]
            plugins: PluginConfig::default(),
            cache: CacheConfig::default(),
            general: GeneralConfig::default(),
        }
    }
}

/// TypeScript-specific generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TypeScriptConfig {
    /// Output directory for generated files
    pub output_dir: String,

    /// File naming convention
    pub naming_convention: NamingConvention,

    /// Type mapping preferences
    pub type_mappings: HashMap<String, String>,

    /// Generate index files
    pub generate_index: bool,

    /// Add JSDoc comments
    pub generate_docs: bool,

    /// Generate type guards
    pub generate_type_guards: bool,

    /// Use strict null checks
    pub strict_null_checks: bool,

    /// Target TypeScript version
    pub target_version: TypeScriptVersion,

    /// Module system to use
    pub module_system: ModuleSystem,

    /// Custom type overrides
    pub type_overrides: HashMap<String, String>,
}

impl Default for TypeScriptConfig {
    fn default() -> Self {
        Self {
            output_dir: "generated/types".to_string(),
            naming_convention: NamingConvention::default(),
            type_mappings: default_type_mappings(),
            generate_index: true,
            generate_docs: true,
            generate_type_guards: false,
            strict_null_checks: true,
            target_version: TypeScriptVersion::default(),
            module_system: ModuleSystem::default(),
            type_overrides: HashMap::new(),
        }
    }
}

/// Default Rust to TypeScript type mappings
fn default_type_mappings() -> HashMap<String, String> {
    let mut mappings = HashMap::new();
    mappings.insert("String".to_string(), "string".to_string());
    mappings.insert("i32".to_string(), "number".to_string());
    mappings.insert("i64".to_string(), "number".to_string());
    mappings.insert("u32".to_string(), "number".to_string());
    mappings.insert("u64".to_string(), "number".to_string());
    mappings.insert("f32".to_string(), "number".to_string());
    mappings.insert("f64".to_string(), "number".to_string());
    mappings.insert("bool".to_string(), "boolean".to_string());
    mappings.insert("Vec".to_string(), "Array".to_string());
    mappings.insert("HashMap".to_string(), "Record".to_string());
    mappings.insert("Option".to_string(), "undefined".to_string());
    mappings.insert("Result".to_string(), "any".to_string());
    mappings
}

/// Naming convention for generated files and types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamingConvention {
    /// camelCase for variables and functions
    CamelCase,
    /// PascalCase for types and classes
    PascalCase,
    /// snake_case for everything (Rust-style)
    SnakeCase,
}

impl Default for NamingConvention {
    fn default() -> Self {
        NamingConvention::PascalCase
    }
}

/// Target TypeScript version
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TypeScriptVersion {
    ES5,
    ES2015,
    ES2016,
    ES2017,
    ES2018,
    ES2019,
    ES2020,
    ES2021,
    ES2022,
    Latest,
}

impl Default for TypeScriptVersion {
    fn default() -> Self {
        TypeScriptVersion::ES2020
    }
}

/// Module system for generated code
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModuleSystem {
    /// ES modules
    ESModules,
    /// CommonJS
    CommonJS,
    /// AMD
    AMD,
}

impl Default for ModuleSystem {
    fn default() -> Self {
        ModuleSystem::ESModules
    }
}

/// Plugin system configuration
#[cfg(feature = "plugins")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginConfig {
    /// Enable plugin system
    pub enabled: bool,

    /// Plugin directories to search
    pub plugin_dirs: Vec<String>,

    /// Enabled plugins
    pub enabled_plugins: Vec<String>,

    /// Plugin configurations
    pub plugin_configs: HashMap<String, serde_json::Value>,

    /// Allow unsafe plugins
    pub allow_unsafe: bool,
}

#[cfg(feature = "plugins")]
impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            plugin_dirs: vec!["plugins".to_string()],
            enabled_plugins: vec![],
            plugin_configs: HashMap::new(),
            allow_unsafe: false,
        }
    }
}

/// Stub plugin configuration when plugins are disabled
#[cfg(not(feature = "plugins"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig;

#[cfg(not(feature = "plugins"))]
impl Default for PluginConfig {
    fn default() -> Self {
        PluginConfig
    }
}

/// Caching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,

    /// Cache directory
    pub cache_dir: String,

    /// Cache TTL in seconds
    pub ttl_seconds: u64,

    /// Maximum cache size in MB
    pub max_size_mb: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_dir: ".shared-types-cache".to_string(),
            ttl_seconds: 3600, // 1 hour
            max_size_mb: 100,
        }
    }
}

/// General generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Verbose logging
    pub verbose: bool,

    /// Dry run mode
    pub dry_run: bool,

    /// Parallel processing
    pub parallel_processing: bool,

    /// Maximum parallel jobs
    pub max_parallel_jobs: usize,

    /// Continue on errors
    pub continue_on_errors: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            dry_run: false,
            parallel_processing: true,
            max_parallel_jobs: num_cpus::get(),
            continue_on_errors: false,
        }
    }
}

impl GenerationConfig {
    /// Create configuration from a file
    pub fn from_file(path: &std::path::Path) -> Result<Self, TypeGenerationError> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a file
    pub fn to_file(&self, path: &std::path::Path) -> Result<(), TypeGenerationError> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), TypeGenerationError> {
        // Validate TypeScript config
        if self.typescript.output_dir.trim().is_empty() {
            return Err(TypeGenerationError::ConfigError(
                "TypeScript output directory cannot be empty".to_string()
            ));
        }

        // Validate cache config
        if self.cache.enabled && self.cache.max_size_mb == 0 {
            return Err(TypeGenerationError::ConfigError(
                "Cache max size must be greater than 0".to_string()
            ));
        }

        Ok(())
    }

    /// Create preset configurations
    pub fn preset_minimal() -> Self {
        Self {
            typescript: TypeScriptConfig {
                generate_index: false,
                generate_docs: false,
                generate_type_guards: false,
                ..Default::default()
            },
            general: GeneralConfig {
                verbose: false,
                parallel_processing: false,
                max_parallel_jobs: 1,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn preset_development() -> Self {
        Self {
            typescript: TypeScriptConfig {
                generate_docs: true,
                generate_type_guards: true,
                ..Default::default()
            },
            general: GeneralConfig {
                verbose: true,
                continue_on_errors: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn preset_production() -> Self {
        Self {
            typescript: TypeScriptConfig {
                generate_docs: true,
                strict_null_checks: true,
                ..Default::default()
            },
            cache: CacheConfig {
                enabled: true,
                ttl_seconds: 7200, // 2 hours
                max_size_mb: 500,
                ..Default::default()
            },
            general: GeneralConfig {
                parallel_processing: true,
                max_parallel_jobs: num_cpus::get().max(4),
                continue_on_errors: false,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = GenerationConfig::default();
        assert!(config.typescript.generate_docs);
        assert!(config.cache.enabled);
        assert_eq!(config.general.max_parallel_jobs, num_cpus::get());
    }

    #[test]
    fn test_config_validation() {
        let config = GenerationConfig::default();
        assert!(config.validate().is_ok());

        let invalid_config = GenerationConfig {
            typescript: TypeScriptConfig {
                output_dir: "".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = GenerationConfig::preset_development();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: GenerationConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config.typescript.generate_docs, deserialized.typescript.generate_docs);
        assert_eq!(config.general.verbose, deserialized.general.verbose);
    }

    #[test]
    fn test_config_file_operations() {
        let config = GenerationConfig::preset_production();
        let temp_file = NamedTempFile::new().unwrap();

        // Save to file
        config.to_file(temp_file.path()).unwrap();

        // Load from file
        let loaded_config = GenerationConfig::from_file(temp_file.path()).unwrap();

        assert_eq!(config.typescript.target_version, loaded_config.typescript.target_version);
    }

    #[test]
    fn test_default_type_mappings() {
        let mappings = default_type_mappings();
        assert_eq!(mappings.get("String"), Some(&"string".to_string()));
        assert_eq!(mappings.get("bool"), Some(&"boolean".to_string()));
        assert_eq!(mappings.get("Vec"), Some(&"Array".to_string()));
    }
}
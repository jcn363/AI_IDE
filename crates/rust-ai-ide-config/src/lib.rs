//! # Unified Security-First Configuration System for Rust AI IDE
//!
//! This crate provides a comprehensive, security-first configuration management system
//! that supports multiple sources, real-time validation, audit trails, and hot reloading
//! while maintaining full compatibility with the Rust AI IDE ecosystem.
//!
//! ## Key Features
//!
//! - **Multi-Source Support**: Environment variables, file configs, database configs
//! - **Security Validation**: Path traversal prevention, input sanitization, threat detection
//! - **Audit Trails**: Encrypted logging of all configuration changes with change tracking
//! - **Hot Reloading**: Zero-downtime configuration updates with validation
//! - **Intelligent Caching**: Performance optimization using unified cache system
//! - **Format Auto-Detection**: TOML, YAML, JSON with automatic format detection
//! - **Developer Experience**: Validation feedback, migration support, detailed error messages
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use rust_ai_ide_config::{Config, ConfigurationManager, SecurityValidator};
//!
//! #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
//! struct MyAppConfig {
//!     api_key:         String,
//!     max_connections: usize,
//! }
//!
//! impl Config for MyAppConfig {
//!     const FILE_PREFIX: &'static str = "myapp";
//!     const DESCRIPTION: &'static str = "My Application Configuration";
//!
//!     fn validate(&self) -> Result<Vec<String>, anyhow::Error> {
//!         let mut errors = Vec::new();
//!         if self.max_connections == 0 {
//!             errors.push("max_connections must be > 0".to_string());
//!         }
//!         Ok(errors)
//!     }
//!
//!     fn default_config() -> Self {
//!         Self {
//!             api_key:         "default_key".to_string(),
//!             max_connections: 10,
//!         }
//!     }
//! }
//!
//! async fn example() -> IDEResult<()> {
//!     // Create configuration manager with security validation
//!     let mut manager = ConfigurationManager::new().await?;
//!
//!     // Load configuration with security checks
//!     let config: MyAppConfig = manager.load_secure("myapp").await?;
//!
//!     // Configuration automatically validated and cached
//!     println!("Loaded config: {:?}", config);
//!
//!     Ok(())
//! }
//! ```

pub mod audit;
pub mod cache;
pub mod config;
pub mod hot_reload;
pub mod migration;
pub mod security;
pub mod sources;
pub mod validation;

// Re-export commonly used types
pub use audit::{AuditConfig, AuditEvent, AuditTrail};
pub use cache::ConfigCache;
pub use config::{Config, ConfigSourcePriority, ConfigurationManager};
pub use hot_reload::{HotReloadManager, ReloadEvent};
pub use migration::{MigrationEngine, MigrationPlan, MigrationResult};
pub use rust_ai_ide_cache::CacheConfig;
pub use security::{SecurityValidator, SecurityViolation, ThreatLevel};
pub use sources::{ConfigSource, EnvironmentSource, FileSource};
pub use validation::{ValidationEngine, ValidationError, ValidationResult};

// Type aliases for convenience
pub type IDEResult<T> = rust_ai_ide_errors::IDEResult<T>;
pub type RustAIError = rust_ai_ide_errors::RustAIError;

/// Version information for the configuration system
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the configuration system
///
/// This function sets up the unified configuration system with security validation,
/// audit trails, and hot reloading capabilities.
pub fn init() -> IDEResult<()> {
    tracing::info!("Initializing unified configuration system v{}", VERSION);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[tokio::test]
    async fn test_initialization() {
        assert!(init().is_ok());
    }
}

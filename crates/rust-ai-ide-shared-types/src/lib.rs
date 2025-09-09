//! # Rust AI IDE Shared Types
//!
//! A comprehensive shared types crate providing:
//! - Automated TypeScript generation from Rust definitions
//! - Cross-platform type consistency and validation
//! - Plugin system for custom type transformations
//! - Seamless integration with existing unified infrastructure
//! - Comprehensive documentation and usage patterns
//!
//! ## Features
//!
//! - **Type Analysis**: Deep static analysis of Rust type definitions using syn
//! - **TypeScript Generation**: Automated conversion of Rust types to TypeScript
//! - **Plugin Architecture**: Extensible system for custom type transformations
//! - **Cross-Platform Validation**: Type safety across Rust backend and TypeScript frontend
//! - **Caching Integration**: Built-in support for type generation caching
//! - **Error Handling**: Unified error types with contextual information
//! - **Configuration**: Flexible configuration system with presets and overrides
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rust_ai_ide_shared_types::{TypeGenerator, TypeScriptConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a TypeScript generator
//! let generator = TypeGenerator::new(TypeScriptConfig::default())?;
//!
//! // Generate TypeScript from Rust types
//! let ts_code = generator.generate_types(
//!     "src/types.rs",
//!     &["MyStruct", "MyEnum"]
//! ).await?;
//!
//! println!("{}", ts_code);
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! The crate follows a modular architecture:
//!
//! - [`parsing`] - AST parsing and type extraction
//! - [`generation`] - Code generation engines (TypeScript, etc.)
//! - [`bridge`] - Cross-platform type bridging and validation
//! - [`plugins`] - Plugin system for custom transformations
//! - [`transformers`] - Built-in type transformation logic
//! - [`config`] - Configuration and presets
//! - [`errors`] - Unified error handling
//! - [`utils`] - Shared utilities and helpers

#![warn(missing_docs, rust_2018_idioms, unused_qualifications)]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Core configuration for shared types functionality
pub mod config;
/// Cross-platform type bridging and validation
pub mod bridge;
/// Unified error handling
pub mod errors;
/// Code generation engines
pub mod generation;
/// AST parsing and type extraction
pub mod parsing;
/// Plugin system for custom transformations
#[cfg(feature = "plugins")]
pub mod plugins;
/// Plugin system implementation (fallback for when plugins are disabled)
#[cfg(not(feature = "plugins"))]
pub mod plugins_stub;
/// Built-in type transformation logic
pub mod transformers;
/// Performance metrics definitions and utilities
pub mod performance;
/// Core type definitions and metadata
pub mod types;
/// Shared utilities and helpers
pub mod utils;

// Re-exports of commonly used types
pub use config::{TypeScriptConfig, GenerationConfig, PluginConfig};
pub use errors::{TypeGenerationError, TypeBridgeError, PluginError};
pub use generation::{TypeGenerator, TypeScriptGenerator};
pub use parsing::{TypeParser};
pub use performance::{PerformanceMetrics, MetricsBuilder, MetricsScope, TimingType, CounterType, RateType, MetricValue};
pub use types::{ParsedType, TypeMetadata};
pub use bridge::{TypeBridge, ValidationResult};

#[cfg(feature = "plugins")]
pub use plugins::{PluginSystem, TypeTransformerPlugin};

#[cfg(not(feature = "plugins"))]
pub use plugins_stub::{PluginSystem, TypeTransformerPlugin};

/// Version information for the shared types crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default configuration for quick setup
pub fn default_config() -> GenerationConfig {
    GenerationConfig::default()
}

/// Create a new TypeScript generator with default configuration
///
/// # Errors
///
/// Returns an error if the generator cannot be initialized
pub fn create_typescript_generator() -> Result<TypeGenerator, TypeGenerationError> {
    TypeGenerator::new(default_config().typescript)
}

/// Validate types across platforms
///
/// # Errors
///
/// Returns an error if validation fails
pub async fn validate_cross_platform(
    rust_types: &[ParsedType],
    config: &GenerationConfig,
) -> Result<ValidationResult, TypeGenerationError> {
    let bridge = TypeBridge::new(config.clone()).map_err(|e| TypeGenerationError::AnalysisError(format!("Failed to create type bridge: {}", e)))?;
    bridge.validate_types(rust_types).await.map_err(|e| TypeGenerationError::AnalysisError(format!("Validation failed: {}", e)))
}
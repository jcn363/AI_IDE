//! # Rust AI IDE Monitoring Framework
//!
//! A comprehensive monitoring framework for automated workspace analysis,
//! quality regression detection, and performance telemetry.
//!
//! ## Features
//!
//! - **Cargo Check Analysis**: Automated parsing of cargo check JSON output
//! - **Unused Variable Detection**: Intelligent categorization of strategic vs problematic variables
//! - **Performance Monitoring**: Compilation metrics and trend analysis
//! - **Cross-platform Validation**: Build verification across different targets
//! - **Quality Telemetry**: Comprehensive metrics collection and reporting
//! - **Security Integration**: Vulnerability scanning and dependency analysis
//! - **Notification System**: Automated alerts for quality regressions
//!
//! ## Architecture
//!
//! The monitoring framework is built around several key components:
//!
//! - `Monitor`: Main orchestrator that coordinates different analyzers
//! - `Analyzer` trait: Interface for different analysis engines
//! - `Reporter` trait: Interface for different reporting mechanisms
//! - `Config`: Configuration management for monitoring parameters
//! - Quality metrics: Structured data types for various measurements
//!
//! ## Usage
//!
//! ```rust,no_run
//! use rust_ai_ide_monitoring::{Monitor, Config};
//!
//! // Create monitoring configuration
//! let config = Config::default();
//!
//! // Initialize monitor
//! let mut monitor = Monitor::new(config).await?;
//!
//! // Run analysis
//! let report = monitor.run_analysis().await?;
//!
//! // Print quality score
//! println!("Quality Score: {}", report.quality_score);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod analyzers;
pub mod config;
pub mod errors;
pub mod metrics;
pub mod monitor;
pub mod parse;
pub mod reporters;
pub mod types;

pub use config::Config;
pub use errors::{MonitoringError, Result};
pub use metrics::{QualityMetrics, QualityScore};
pub use monitor::Monitor;
pub use types::{AnalysisReport, AnalysisResult, Category, Severity};

// Re-export commonly used types
pub use serde::{Deserialize, Serialize};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default configuration values
pub mod defaults {
    pub const DEFAULT_REPORT_DIR: &str = "monitoring-reports";
    pub const DEFAULT_THRESHOLD_QUALITY: f64 = 75.0;
    pub const DEFAULT_THRESHOLD_WARNINGS: usize = 50;
    pub const DEFAULT_ANALYSIS_TIMEOUT_SECS: u64 = 300; // 5 minutes
}
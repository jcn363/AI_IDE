//! Module containing data models for Cargo integration
//!
//! This module defines the data structures used by the cargo build system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cargo project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoProject {
    pub name: String,
    pub version: String,
    pub edition: String,
    pub authors: Vec<String>,
    pub dependencies: HashMap<String, CargoDepVersion>,
    pub dev_dependencies: HashMap<String, CargoDepVersion>,
    pub build_dependencies: HashMap<String, CargoDepVersion>,
    pub workspace_members: Option<Vec<String>>,
}

/// Cargo dependency version specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CargoDepVersion {
    Simple(String),
    Detailed {
        version: Option<String>,
        path: Option<String>,
        git: Option<String>,
        branch: Option<String>,
        tag: Option<String>,
        features: Option<Vec<String>>,
        default_features: Option<bool>,
    },
}

/// Build output information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub success: bool,
    pub output: String,
    pub error: String,
}

/// Test result information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub success: bool,
    pub total_tests: u32,
    pub passed_tests: u32,
    pub failed_tests: u32,
    pub ignored_tests: u32,
    pub stdout: String,
    pub stderr: String,
}

/// Build metrics for performance analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildMetrics {
    /// Build duration in milliseconds
    pub duration: f64,
    /// Peak memory usage in bytes
    pub peak_memory_usage: u64,
    /// CPU usage during build (0-100)
    pub cpu_usage: f32,
    /// Total target count
    pub total_targets: usize,
    /// Successful compilation count
    pub successful_targets: usize,
    /// Warning count
    pub warning_count: usize,
    /// Error count
    pub error_count: usize,
    /// Cache hit rate (0.0-1.0)
    pub cache_hit_rate: f32,
    /// Incremental compilation flag
    pub incremental: bool,
}

/// Metrics for individual crates within the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateMetrics {
    /// Crate name
    pub name: String,
    /// Crate version
    pub version: String,
    /// Compilation duration in milliseconds
    pub compilation_time: f64,
    /// Source lines of code count
    pub lines_of_code: usize,
    /// Dependencies count
    pub dependency_count: usize,
    /// Whether this crate is published
    pub published: bool,
    /// Crate path
    pub path: String,
    /// Features enabled for this crate
    pub features: Vec<String>,
}

impl Default for CrateMetrics {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: "0.1.0".to_string(),
            compilation_time: 0.0,
            lines_of_code: 0,
            dependency_count: 0,
            published: false,
            path: String::new(),
            features: Vec::new(),
        }
    }
}

/// Target information for build
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInfo {
    /// Target name
    pub name: String,
    /// Target kind (lib, bin, test, example, etc.)
    pub kind: Vec<String>,
    /// Whether this target needs compilation
    pub crate_types: Vec<String>,
    /// Source file path
    pub src_path: String,
    /// Edition
    pub edition: String,
    /// Required features
    pub required_features: Option<Vec<String>>,
    /// Whether doctest
    pub doctest: bool,
}

/// Metadata extracted from Cargo.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoMetadata {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Package description
    pub description: Option<String>,
    /// Package authors
    pub authors: Vec<String>,
    /// Package license
    pub license: Option<String>,
    /// Package repository
    pub repository: Option<String>,
    /// Package documentation
    pub documentation: Option<String>,
    /// Package homepage
    pub homepage: Option<String>,
    /// Package keywords
    pub keywords: Vec<String>,
    /// Package categories
    pub categories: Vec<String>,
    /// Workspace root
    pub workspace_root: String,
    /// All targets in the package
    pub targets: Vec<TargetInfo>,
    /// Features defined in the package
    pub features: HashMap<String, Vec<String>>,
    /// Dependencies
    pub dependencies: HashMap<String, DependencyInfo>,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    /// Dependency name
    pub name: String,
    /// Dependency version requirement
    pub version_req: String,
    /// Resolved version (if available)
    pub version: Option<String>,
    /// Whether this is a dev dependency
    pub dev: bool,
    /// Whether this is a build dependency
    pub build: bool,
    /// Whether this dependency is optional
    pub optional: bool,
    /// Package registry
    pub registry: Option<String>,
}

/// Build plan information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildPlan {
    /// Build targets
    pub targets: Vec<TargetInfo>,
    /// Build profile
    pub profile: String,
    /// Build features
    pub features: Option<Vec<String>>,
    /// Build targets to skip
    pub excluded: Vec<String>,
    /// Whether to run tests
    pub run_tests: bool,
    /// Whether to run benchmarks
    pub run_benches: bool,
    /// Whether to build documentation
    pub build_doc: bool,
}

//! Consolidated Cargo-related types shared between backend and frontend
//!
//! This module provides unified type definitions for Cargo operations that eliminate
//! duplication between the Rust backend and TypeScript frontend.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Core dependency specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CargoDependency {
    /// Version specification (can be exact version, range, or constraint)
    pub version: Option<String>,
    /// Local path dependency
    pub path: Option<String>,
    /// Git repository dependency
    pub git: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub rev: Option<String>,
    /// Advanced dependency options
    pub features: Option<Vec<String>>,
    pub optional: Option<bool>,
    pub default_features: Option<bool>,
    pub package: Option<String>,
    /// Registry and workspace options
    pub registry: Option<String>,
    pub workspace: Option<bool>,
}

impl Default for CargoDependency {
    fn default() -> Self {
        Self {
            version: None,
            path: None,
            git: None,
            branch: None,
            tag: None,
            rev: None,
            features: None,
            optional: None,
            default_features: None,
            package: None,
            registry: None,
            workspace: None,
        }
    }
}

/// Feature usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureUsage {
    pub name: String,
    pub enabled_by_default: bool,
    pub is_used: bool,
    pub used_by: Vec<String>,
    pub is_default: Option<bool>,
}

/// Package metadata for Cargo.toml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CargoPackage {
    pub name: Option<String>,
    pub version: Option<String>,
    pub authors: Option<Vec<String>>,
    pub edition: Option<String>,
    pub rust_version: Option<String>,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub readme: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
    pub license: Option<String>,
    pub license_file: Option<String>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub publish: Option<PublishConfig>,
    pub default_features: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Publishing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PublishConfig {
    Boolean(bool),
    RegistryList(Vec<String>),
}

/// Workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CargoWorkspace {
    pub members: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub default_members: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub dependencies: Option<HashMap<String, CargoDependency>>,
    pub dev_dependencies: Option<HashMap<String, CargoDependency>>,
    pub build_dependencies: Option<HashMap<String, CargoDependency>>,
    pub package: Option<WorkspacePackageConfig>,
    pub resolver: Option<String>,
}

/// Workspace package configuration defaults
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkspacePackageConfig {
    pub version: Option<String>,
    pub authors: Option<Vec<String>>,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
}

/// Profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CargoProfile {
    pub opt_level: Option<OptimizationLevel>,
    pub debug: Option<DebugLevel>,
    pub rpath: Option<bool>,
    pub lto: Option<LinkTimeOptimization>,
    pub codegen_units: Option<u32>,
    pub panic: Option<PanicStrategy>,
    pub incremental: Option<bool>,
    pub overflow_checks: Option<bool>,
    pub debug_assertions: Option<bool>,
    pub split_debuginfo: Option<String>,
}

/// Optimization level
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OptimizationLevel {
    Level(u32),
    String(String),
}

/// Debug level
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DebugLevel {
    Boolean(bool),
    Level(u32),
}

/// Link-time optimization setting
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkTimeOptimization {
    Off,
    Thin,
    Fat,
    #[serde(rename = "false")]
    False,
}

/// Panic strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PanicStrategy {
    Unwind,
    Abort,
}

/// Library target configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LibConfig {
    pub name: Option<String>,
    pub path: Option<String>,
    pub crate_type: Option<Vec<String>>,
    pub edition: Option<String>,
}

/// Binary target configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinConfig {
    pub name: Option<String>,
    pub path: Option<String>,
    pub edition: Option<String>,
    pub required_features: Option<Vec<String>>,
}

/// Example target configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExampleConfig {
    pub name: Option<String>,
    pub path: Option<String>,
    pub edition: Option<String>,
    pub required_features: Option<Vec<String>>,
}

/// Test target configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestConfig {
    pub name: Option<String>,
    pub path: Option<String>,
    pub edition: Option<String>,
    pub harness: Option<bool>,
    pub required_features: Option<Vec<String>>,
}

/// Benchmark target configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchConfig {
    pub name: Option<String>,
    pub path: Option<String>,
    pub edition: Option<String>,
    pub harness: Option<bool>,
    pub required_features: Option<Vec<String>>,
}

/// Target-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CargoTargetConfig {
    pub compilation_target: Option<String>,
    pub rustflags: Option<Vec<String>>,
    pub linker: Option<String>,
}

/// Root Cargo.toml manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoManifest {
    /// Core package configuration
    pub package: Option<CargoPackage>,
    /// Dependencies
    pub dependencies: Option<HashMap<String, CargoDependency>>,
    #[serde(rename = "dev-dependencies")]
    pub dev_dependencies: Option<HashMap<String, CargoDependency>>,
    #[serde(rename = "build-dependencies")]
    pub build_dependencies: Option<HashMap<String, CargoDependency>>,
    pub target: Option<HashMap<String, HashMap<String, CargoDependency>>>,
    /// Feature flags
    pub features: Option<HashMap<String, Vec<String>>>,
    /// Workspace configuration
    pub workspace: Option<CargoWorkspace>,
    /// Profile configuration
    pub profile: Option<HashMap<String, CargoProfile>>,
    /// Library target
    pub lib: Option<LibConfig>,
    /// Binary targets
    pub bin: Option<Vec<BinConfig>>,
    /// Example targets
    pub example: Option<Vec<ExampleConfig>>,
    /// Test targets
    pub test: Option<Vec<TestConfig>>,
    /// Benchmark targets
    pub bench: Option<Vec<BenchConfig>>,
}

impl Default for CargoManifest {
    fn default() -> Self {
        Self {
            package: None,
            dependencies: None,
            dev_dependencies: None,
            build_dependencies: None,
            target: None,
            features: None,
            workspace: None,
            profile: None,
            lib: None,
            bin: None,
            example: None,
            test: None,
            bench: None,
        }
    }
}

/// Dependency section types
#[derive(Debug, Clone, Copy)]
pub enum DependencySection {
    Dependencies,
    DevDependencies,
    BuildDependencies,
}

impl DependencySection {
    pub fn as_str(&self) -> &'static str {
        match self {
            DependencySection::Dependencies => "dependencies",
            DependencySection::DevDependencies => "dev-dependencies",
            DependencySection::BuildDependencies => "build-dependencies",
        }
    }
}

/// Location of a dependency in the manifest
#[derive(Debug)]
pub struct DependencyLocation<'a> {
    pub section: DependencySection,
    pub name: &'a str,
    pub dependency: &'a CargoDependency,
}

/// Feature configuration
#[derive(Debug, Clone)]
pub struct FeatureConfig {
    pub name: String,
    pub enabled_by_default: bool,
    pub dependencies: Vec<String>,
}

/// Dependency update information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyUpdate {
    pub name: String,
    pub current_version: String,
    pub new_version: String,
    pub update_type: UpdateType,
    pub used_in: Vec<UsageLocation>,
    pub changelog_url: Option<String>,
    pub is_updating: bool,
    pub update_error: Option<String>,
}

/// Update type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdateType {
    Major,
    Minor,
    Patch,
}

/// Where a dependency is used
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageLocation {
    pub member: String,
    pub version: String,
}

/// Cargo build metadata from `cargo metadata`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoMetadata {
    pub packages: Vec<CargoPackageMetadata>,
    pub workspace_root: String,
    pub target_directory: String,
    pub resolve: Option<ResolveNode>,
    pub workspace_members: Vec<String>,
}

/// Individual package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoPackageMetadata {
    pub name: String,
    pub version: String,
    pub id: String,
    pub source: Option<String>,
    pub dependencies: Vec<PackageDependency>,
    pub manifest_path: String,
    pub features: HashMap<String, Vec<String>>,
}

/// Package dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDependency {
    pub name: String,
    pub source: Option<String>,
    pub req: String,
    pub kind: Option<String>,
    pub rename: Option<String>,
    pub optional: bool,
    pub uses_default_features: bool,
    pub features: Vec<String>,
}

/// Dependency resolution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveNode {
    pub nodes: Vec<ResolveNodeItem>,
    pub root: Option<String>,
}

/// Individual resolution node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveNodeItem {
    pub id: String,
    pub dependencies: Vec<String>,
}

/// Build information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildInfo {
    pub build_time: u64,
    pub features_used: Vec<String>,
    pub profile: String,
}

/// Target triple information
#[derive(Debug, Clone, Serialize)]
pub struct TargetTriple {
    pub triple: String,
    pub platform: String,
    pub architecture: String,
    pub vendor: String,
    pub os: String,
    pub abi: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_dependency_serialization() {
        let dep = CargoDependency {
            version: Some("1.0".to_string()),
            features: Some(vec!["default".to_string()]),
            ..Default::default()
        };

        let json = serde_json::to_string(&dep).unwrap();
        let deserialized: CargoDependency = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.version, Some("1.0".to_string()));
        assert_eq!(deserialized.features, Some(vec!["default".to_string()]));
    }

    #[test]
    fn test_cargo_package_serialization() {
        let package = CargoPackage {
            name: Some("test-package".to_string()),
            version: Some("1.0.0".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&package).unwrap();
        let deserialized: CargoPackage = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, Some("test-package".to_string()));
        assert_eq!(deserialized.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_dependency_section_conversion() {
        assert_eq!(DependencySection::Dependencies.as_str(), "dependencies");
        assert_eq!(
            DependencySection::DevDependencies.as_str(),
            "dev-dependencies"
        );
        assert_eq!(
            DependencySection::BuildDependencies.as_str(),
            "build-dependencies"
        );
    }
}

//! Cargo-related types for Rust AI IDE
//!
//! This module contains types specific to Cargo project management.

/// Cargo manifest version requirement
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CargoVersionReq {
    pub requirement: String,
    pub compatible:  Vec<String>,
}

/// Cargo feature dependency graph node
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CargoFeatureNode {
    pub name:               String,
    pub enabled_ports:      Vec<String>,
    pub dependent_features: Vec<String>,
}

/// Cargo change type
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ChangeType {
    Added,
    Removed,
    Updated,
    Downgraded,
}

/// Cargo package change information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CargoPackageChange {
    pub package_name:   String,
    pub change_type:    ChangeType,
    pub version_before: Option<String>,
    pub version_after:  Option<String>,
}

/// Cargo audit severity levels
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CargoAuditSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Cargo security advisory
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CargoAdvisory {
    pub id:               String,
    pub package:          String,
    pub title:            String,
    pub severity:         CargoAuditSeverity,
    pub description:      String,
    pub url:              String,
    pub patched_versions: Vec<String>,
}

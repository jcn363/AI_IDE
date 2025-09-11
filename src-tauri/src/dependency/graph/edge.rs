//! Module defining the edge type for the dependency graph

use super::DependencyType;
use serde::{Deserialize, Serialize};

/// Represents an edge in the dependency graph
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyEdge {
    /// Type of dependency
    pub dep_type: DependencyType,
    /// Version requirement
    pub version_req: String,
    /// Whether this is an optional dependency
    pub optional: bool,
    /// Whether default features are used
    pub uses_default_features: bool,
    /// Enabled features for this dependency
    pub features: Vec<String>,
    /// Target platform (if specified)
    pub target: Option<String>,
}

// Builder methods for DependencyEdge
/// Create a new dependency edge
pub fn new(dep_type: DependencyType) -> Self {
    Self {
        dep_type,
        version_req: String::new(),
        optional: false,
        uses_default_features: true,
        features: Vec::new(),
        target: None,
    }
}

/// Set the version requirement
pub fn with_version_req(mut self, req: impl Into<String>) -> Self {
    self.version_req = req.into();
    self
}

/// Set whether the dependency is optional
pub fn with_optional(mut self, optional: bool) -> Self {
    self.optional = optional;
    self
}

/// Set whether to use default features
pub fn with_default_features(mut self, use_default: bool) -> Self {
    self.uses_default_features = use_default;
    self
}

/// Add a feature to the dependency
pub fn with_feature(mut self, feature: impl Into<String>) -> Self {
    self.features.push(feature.into());
    self
}

/// Add multiple features to the dependency
pub fn with_features(mut self, features: impl IntoIterator<Item = String>) -> Self {
    self.features.extend(features);
    self
}

/// Set the target platform
pub fn with_target(mut self, target: impl Into<String>) -> Self {
    self.target = Some(target.into());
    self
}

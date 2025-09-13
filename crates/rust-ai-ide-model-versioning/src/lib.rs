//! Model versioning utilities for Rust AI IDE

use std::collections::HashMap;

use semver::Version;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Represents a versioned model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedModel {
    /// Model identifier
    pub id:       String,
    /// Current version
    pub version:  Version,
    /// Model metadata
    pub metadata: HashMap<String, String>,
}

/// Error type for model versioning operations
#[derive(Error, Debug)]
pub enum ModelVersioningError {
    #[error("Invalid version format: {0}")]
    InvalidVersion(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl VersionedModel {
    /// Create a new versioned model
    pub fn new(id: impl Into<String>, version: impl AsRef<str>) -> Result<Self, ModelVersioningError> {
        let version = Version::parse(version.as_ref())
            .map_err(|_| ModelVersioningError::InvalidVersion(version.as_ref().to_string()))?;

        Ok(Self {
            id: id.into(),
            version,
            metadata: HashMap::new(),
        })
    }

    /// Bump the model version
    pub fn bump_major(&mut self) {
        self.version.major += 1;
        self.version.minor = 0;
        self.version.patch = 0;
    }

    /// Bump the minor version
    pub fn bump_minor(&mut self) {
        self.version.minor += 1;
        self.version.patch = 0;
    }

    /// Bump the patch version
    pub fn bump_patch(&mut self) {
        self.version.patch += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_versioned_model() -> Result<(), ModelVersioningError> {
        let mut model = VersionedModel::new("test-model", "1.0.0")?;
        assert_eq!(model.id, "test-model");
        assert_eq!(model.version.to_string(), "1.0.0");

        model.bump_patch();
        assert_eq!(model.version.to_string(), "1.0.1");

        model.bump_minor();
        assert_eq!(model.version.to_string(), "1.1.0");

        model.bump_major();
        assert_eq!(model.version.to_string(), "2.0.0");

        Ok(())
    }
}

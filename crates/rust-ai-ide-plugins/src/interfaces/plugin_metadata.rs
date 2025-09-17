//! Plugin metadata definitions for the Rust AI IDE plugin system.

use semver::Version;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Metadata for a plugin, containing basic information about its identity and properties.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique identifier for the plugin (UUID)
    pub id: Uuid,
    /// Human-readable name of the plugin
    pub name: String,
    /// Version of the plugin (using semantic versioning)
    pub version: Version,
    /// Author or organization responsible for the plugin
    pub author: String,
    /// Description of the plugin's functionality
    pub description: String,
    /// Repository URL for the plugin
    pub repository: Option<String>,
    /// Website URL for the plugin
    pub homepage: Option<String>,
    /// Minimum version of the IDE required
    pub minimum_ide_version: Option<String>,
}

impl PluginMetadata {
    /// Creates a new PluginMetadata instance.
    pub fn new(
        id: Uuid,
        name: String,
        version: Version,
        author: String,
        description: String,
    ) -> Self {
        Self {
            id,
            name,
            version,
            author,
            description,
            repository: None,
            homepage: None,
            minimum_ide_version: None,
        }
    }

    /// Sets the repository URL.
    pub fn with_repository(mut self, repository: String) -> Self {
        self.repository = Some(repository);
        self
    }

    /// Sets the homepage URL.
    pub fn with_homepage(mut self, homepage: String) -> Self {
        self.homepage = Some(homepage);
        self
    }

    /// Sets the minimum IDE version required.
    pub fn with_minimum_ide_version(mut self, version: String) -> Self {
        self.minimum_ide_version = Some(version);
        self
    }
}

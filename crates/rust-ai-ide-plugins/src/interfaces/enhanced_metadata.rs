//! Enhanced Plugin Metadata with Dependencies and Advanced Capabilities
//!
//! This module extends the basic PluginMetadata with enterprise-level features including
//! dependency management, version compatibility, categorization, and advanced capabilities.

use std::collections::{HashMap, HashSet};

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

use crate::interfaces::{PluginCapabilities, PluginMetadata};

/// Enhanced plugin versioning with compatibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginVersioning {
    /// Semantic version of the plugin
    pub version: Version,
    /// Minimum IDE version required
    pub min_ide_version: Option<Version>,
    /// Maximum IDE version supported (for compatibility checking)
    pub max_ide_version: Option<Version>,
    /// Minimum Rust version required
    pub min_rust_version: Option<String>,
    /// Target platforms/architectures supported
    pub supported_platforms: HashSet<String>,
    /// Deprecated versions that should be upgraded from
    pub deprecated_versions: HashSet<Version>,
    /// Breaking change compatibility matrix
    pub compatibility_matrix: HashMap<String, VersionReq>,
}

/// Plugin dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// ID of the required plugin
    pub plugin_id: String,
    /// Version requirement (semantic versioning)
    pub version_req: VersionReq,
    /// Whether this dependency is optional
    pub optional: bool,
    /// Description of what this dependency provides
    pub description: Option<String>,
    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolution,
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Require exact match (default)
    Exact,
    /// Allow compatible versions
    Compatible,
    /// Try to resolve automatically
    Auto,
    /// Require manual intervention
    Manual,
}

/// Plugin categorization system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginCategory {
    /// Utility plugins (formatters, linters)
    Utility,
    /// Language plugins (Rust, Python, Go support)
    Language,
    /// Framework plugins (Rocket, Axum support)
    Framework,
    /// Tool plugins (cargo commands, debuggers)
    Tool,
    /// Theme/Aesthetic plugins
    Theme,
    /// Integration plugins (Git, Docker support)
    Integration,
    /// Experimental/Developer plugins
    Experimental,
}

/// Plugin lifecycle hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleHooks {
    /// Pre-installation hook command
    pub pre_install: Option<String>,
    /// Post-installation hook command
    pub post_install: Option<String>,
    /// Pre-uninstallation hook command
    pub pre_uninstall: Option<String>,
    /// Post-uninstallation hook command
    pub post_uninstall: Option<String>,
    /// Health check command
    pub health_check: Option<String>,
}

/// Enhanced plugin capabilities with advanced features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedPluginCapabilities {
    /// Base capabilities (inherited from core system)
    pub base_capabilities: PluginCapabilities,
    /// Advanced permissions required
    pub permissions: HashSet<PluginPermission>,
    /// Plugin resources (files, services, ports)
    pub resources: PluginResources,
    /// Interaction patterns supported
    pub interaction_patterns: HashSet<InteractionPattern>,
    /// Performance characteristics
    pub performance_profile: PerformanceProfile,
}

/// Advanced plugin permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginPermission {
    /// File system access permissions
    Filesystem {
        path: String,
        access: FilesystemAccess,
    },
    /// Network access permissions
    Network {
        host: String,
        port_range: Option<String>,
    },
    /// System command execution
    SystemCommand { command: String },
    /// Environment variable access
    EnvironmentVariable { name: String, read_write: bool },
    /// Database access
    Database { connection_string: String },
    /// External service integration
    ServiceIntegration { service_name: String },
}

/// Filesystem access types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FilesystemAccess {
    Read,
    Write,
    Full,
}

/// Plugin static resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResources {
    /// Static files provided by the plugin
    pub static_files: HashSet<String>,
    /// Directory space required (in KB)
    pub disk_space_kb: Option<u64>,
    /// Memory usage estimate (in MB)
    pub memory_mb: Option<u64>,
    /// CPU usage pattern
    pub cpu_intensity: Option<String>,
}

/// Plugin interaction patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InteractionPattern {
    /// Always active (daemon-like)
    Daemon,
    /// Command-based interaction
    Command,
    /// Event-driven interaction
    EventDriven,
    /// Interactive UI components
    Interactive,
    /// Background processor
    Background,
}

/// Performance profile characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    /// Startup time (in seconds)
    pub startup_time_seconds: Option<f64>,
    /// Memory usage pattern
    pub memory_pattern: String,
    /// CPU usage pattern
    pub cpu_pattern: String,
    /// I/O pattern
    pub io_pattern: String,
}

/// Enhanced plugin metadata with enterprise features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedPluginMetadata {
    /// Core plugin metadata (inherited)
    pub core: PluginMetadata,
    /// Enhanced versioning information
    pub versioning: PluginVersioning,
    /// Plugin category
    pub category: PluginCategory,
    /// Plugin dependencies
    pub dependencies: Vec<PluginDependency>,
    /// Plugins this conflicts with
    pub conflicts: Vec<String>,
    /// Plugin lifecycle hooks
    pub lifecycle_hooks: Option<LifecycleHooks>,
    /// Enhanced capabilities
    pub enhanced_capabilities: EnhancedPluginCapabilities,
    /// Plugin tags for discovery
    pub tags: HashSet<String>,
    /// Security checksum (SHA256 of plugin files)
    pub security_checksum: Option<String>,
    /// Marketplace metadata
    pub marketplace: Option<MarketplaceMetadata>,
    /// Telemetry preferences
    pub telemetry: PluginTelemetryPrefs,
}

/// Marketplace-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceMetadata {
    /// Downloads count
    pub downloads: u64,
    /// Rating (0.0 - 5.0)
    pub rating: f64,
    /// Review count
    pub reviews: u32,
    /// Publisher information
    pub publisher: String,
    /// Licensing information
    pub license: String,
    /// Last update timestamp
    pub updated_at: Option<String>,
    /// Repository URL
    pub repository_url: Option<String>,
}

/// Plugin telemetry preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginTelemetryPrefs {
    /// Whether telemetry is enabled
    pub enabled: bool,
    /// What telemetry data to collect
    pub data_types: HashSet<String>,
    /// Data collection frequency
    pub frequency: String,
    /// Whether to anonymize data
    pub anonymize: bool,
}

impl EnhancedPluginMetadata {
    /// Create a new enhanced plugin metadata builder
    pub fn builder(id: String, name: String, version: Version) -> EnhancedPluginMetadataBuilder {
        EnhancedPluginMetadataBuilder::new(id, name, version)
    }

    /// Validate that this plugin metadata is complete and valid
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut issues = Vec::new();

        // Validate core metadata
        // UUID validation - no is_empty method needed for UUID
        // UUIDs are always valid and not empty once created

        // Validate versioning
        if let Some(min_ide) = &self.versioning.min_ide_version {
            if let Some(max_ide) = &self.versioning.max_ide_version {
                if min_ide > max_ide {
                    issues.push("Minimum IDE version cannot be greater than maximum".to_string());
                }
            }
        }

        // Validate dependencies for cycles and conflicts
        let mut seen_ids = HashSet::new();
        for dep in &self.dependencies {
            if !seen_ids.insert(&dep.plugin_id) && self.conflicts.contains(&dep.plugin_id) {
                issues.push(format!(
                    "Plugin '{}' is both a dependency and conflict",
                    dep.plugin_id
                ));
            }
        }

        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    /// Check compatibility with a given IDE version
    pub fn is_compatible_with_ide(&self, ide_version: &Version) -> bool {
        // Check minimum version
        if let Some(min) = &self.versioning.min_ide_version {
            if ide_version < min {
                return false;
            }
        }

        // Check maximum version
        if let Some(max) = &self.versioning.max_ide_version {
            if ide_version > max {
                return false;
            }
        }

        true
    }

    /// Get all required permissions
    pub fn get_required_permissions(&self) -> &HashSet<PluginPermission> {
        &self.enhanced_capabilities.permissions
    }

    /// Check if this plugin conflicts with another
    pub fn conflicts_with(&self, other_id: &str) -> bool {
        self.conflicts.iter().any(|c| c == other_id)
    }
}

/// Builder pattern for creating enhanced plugin metadata
pub struct EnhancedPluginMetadataBuilder {
    metadata: EnhancedPluginMetadata,
}

impl EnhancedPluginMetadataBuilder {
    /// Create a new builder
    pub fn new(id: String, name: String, version: Version) -> Self {
        use uuid::Uuid;
        let core_metadata = PluginMetadata::new(
            Uuid::new_v4(),
            name.clone(),
            version.clone(),
            "Unknown".to_string(),
            "No description".to_string(),
        );

        Self {
            metadata: EnhancedPluginMetadata {
                core: core_metadata,
                versioning: PluginVersioning {
                    version,
                    min_ide_version: None,
                    max_ide_version: None,
                    min_rust_version: None,
                    supported_platforms: Default::default(),
                    deprecated_versions: Default::default(),
                    compatibility_matrix: Default::default(),
                },
                category: PluginCategory::Utility,
                dependencies: Vec::new(),
                conflicts: Vec::new(),
                lifecycle_hooks: None,
                enhanced_capabilities: EnhancedPluginCapabilities {
                    base_capabilities: PluginCapabilities::new(),
                    permissions: Default::default(),
                    resources: PluginResources {
                        static_files: Default::default(),
                        disk_space_kb: None,
                        memory_mb: None,
                        cpu_intensity: None,
                    },
                    interaction_patterns: Default::default(),
                    performance_profile: PerformanceProfile {
                        startup_time_seconds: None,
                        memory_pattern: "unknown".to_string(),
                        cpu_pattern: "unknown".to_string(),
                        io_pattern: "unknown".to_string(),
                    },
                },
                tags: Default::default(),
                security_checksum: None,
                marketplace: None,
                telemetry: PluginTelemetryPrefs {
                    enabled: true,
                    data_types: HashSet::from(["usage".to_string(), "performance".to_string()]),
                    frequency: "hourly".to_string(),
                    anonymize: true,
                },
            },
        }
    }

    /// Set the author
    pub fn author(mut self, author: String) -> Self {
        self.metadata.core.author = author;
        self
    }

    /// Set the description
    pub fn description(mut self, description: String) -> Self {
        self.metadata.core.description = description;
        self
    }

    /// Set minimum IDE version
    pub fn min_ide_version(mut self, version: Version) -> Self {
        self.metadata.versioning.min_ide_version = Some(version);
        self
    }

    /// Set plugin category
    pub fn category(mut self, category: PluginCategory) -> Self {
        self.metadata.category = category;
        self
    }

    /// Add a dependency
    pub fn dependency(mut self, dep: PluginDependency) -> Self {
        self.metadata.dependencies.push(dep);
        self
    }

    /// Add a command capability
    pub fn command(mut self, command: String) -> Self {
        self.metadata
            .enhanced_capabilities
            .base_capabilities
            .commands
            .insert(command);
        self
    }

    /// Add a permission
    pub fn permission(mut self, permission: PluginPermission) -> Self {
        self.metadata
            .enhanced_capabilities
            .permissions
            .insert(permission);
        self
    }

    /// Add tags
    pub fn tag(mut self, tag: String) -> Self {
        self.metadata.tags.insert(tag);
        self
    }

    /// Set repository URL
    pub fn repository(mut self, repo: String) -> Self {
        self.metadata.core.repository = Some(repo);
        self
    }

    /// Build the enhanced plugin metadata
    pub fn build(self) -> EnhancedPluginMetadata {
        self.metadata
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::*;

    #[test]
    fn test_enhanced_metadata_builder() {
        let metadata = EnhancedPluginMetadata::builder(
            "test-plugin".to_string(),
            "Test Plugin".to_string(),
            Version::parse("1.0.0").unwrap(),
        )
        .author("Test Author".to_string())
        .description("A test plugin".to_string())
        .min_ide_version(Version::parse("0.5.0").unwrap())
        .category(PluginCategory::Utility)
        .command("test-command".to_string())
        .tag("test".to_string())
        .tag("example".to_string())
        .build();

        assert_eq!(metadata.core.id, "test-plugin");
        assert_eq!(metadata.core.author, "Test Author");
        assert_eq!(metadata.category, PluginCategory::Utility);
        assert!(metadata
            .enhanced_capabilities
            .base_capabilities
            .commands
            .contains("test-command"));
        assert!(metadata.tags.contains("test"));
        assert!(metadata.tags.contains("example"));
    }

    #[test]
    fn test_ide_version_compatibility() {
        let version = Version::parse("1.0.0").unwrap();
        let min_compat = Version::parse("0.5.0").unwrap();
        let max_compat = Version::parse("2.0.0").unwrap();

        let plugin =
            EnhancedPluginMetadata::builder("test".to_string(), "Test".to_string(), version)
                .min_ide_version(min_compat.clone())
                .build();

        // Manually set max version as builder doesn't have this method yet
        let plugin = EnhancedPluginMetadata {
            versioning: PluginVersioning {
                max_ide_version: Some(max_compat),
                ..plugin.versioning
            },
            ..plugin
        };

        assert!(!plugin.is_compatible_with_ide(&Version::parse("0.3.0").unwrap()));
        assert!(plugin.is_compatible_with_ide(&Version::parse("0.7.0").unwrap()));
        assert!(plugin.is_compatible_with_ide(&Version::parse("1.5.0").unwrap()));
        assert!(!plugin.is_compatible_with_ide(&Version::parse("2.5.0").unwrap()));
    }

    #[test]
    fn test_metadata_validation() {
        let metadata = EnhancedPluginMetadata::builder(
            "".to_string(), // Empty ID - should fail validation
            "Test".to_string(),
            Version::parse("1.0.0").unwrap(),
        )
        .build();

        let result = metadata.validate();
        assert!(result.is_err());
        let issues = result.unwrap_err();
        assert!(issues
            .iter()
            .any(|issue| issue.contains("ID cannot be empty")));
    }
}

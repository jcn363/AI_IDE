//! Dependency management for Cargo projects

use anyhow::{Context, Result};
use cargo_metadata::Package;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod alignment;
pub mod conflict;

pub use alignment::VersionAlignment;
pub use conflict::{ConflictVersion, DependentInfo, VersionConflict};

/// Represents a dependency in the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub name: String,
    pub version: String,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: DependencyKind,
    pub registry: Option<String>,
    pub source: Option<String>,
}

/// The kind of dependency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DependencyKind {
    Normal,
    Development,
    Build,
}

/// Manages project dependencies
#[derive(Debug)]
pub struct DependencyManager {
    workspace_path: String,
    dependencies: Arc<RwLock<HashMap<String, DependencyInfo>>>,
    metadata: Arc<tokio::sync::RwLock<Option<cargo_metadata::Metadata>>>,
}

impl DependencyManager {
    /// Creates a new DependencyManager for the given workspace
    pub fn new(workspace_path: impl Into<String>) -> Self {
        Self {
            workspace_path: workspace_path.into(),
            dependencies: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// Loads dependencies from Cargo.toml
    pub async fn load_dependencies(&self) -> Result<()> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(&format!("{}/Cargo.toml", self.workspace_path))
            .exec()
            .context("Failed to execute cargo metadata")?;

        let mut deps_map = HashMap::new();
        for package in &metadata.packages {
            self.process_dependencies(package, &mut deps_map, DependencyKind::Normal)
                .await;
        }

        // Store metadata for conflict resolution
        {
            let mut deps = self.dependencies.write().await;
            *deps = deps_map;
        }

        // Store metadata
        {
            let mut meta = self.metadata.write().await;
            *meta = Some(metadata);
        }

        Ok(())
    }

    async fn process_dependencies(
        &self,
        package: &Package,
        deps_map: &mut HashMap<String, DependencyInfo>,
        kind: DependencyKind,
    ) {
        use cargo_metadata::DependencyKind as MetaDependencyKind;

        // Get all dependencies from the package
        let deps = package.dependencies.iter().filter(|dep| {
            match kind {
                DependencyKind::Normal => {
                    // Normal dependencies have no specific kind
                    matches!(dep.kind, MetaDependencyKind::Normal)
                }
                DependencyKind::Development => {
                    // Development dependencies have the 'dev' kind
                    matches!(dep.kind, MetaDependencyKind::Development)
                }
                DependencyKind::Build => {
                    // Build dependencies have the 'build' kind
                    matches!(dep.kind, MetaDependencyKind::Build)
                }
            }
        });

        for dep in deps {
            let dep_info = DependencyInfo {
                name: dep.name.clone(),
                version: dep.req.to_string(),
                features: dep.features.clone(),
                optional: dep.optional,
                default_features: dep.uses_default_features,
                target: dep.target.as_ref().map(|t| t.to_string()),
                kind: kind.clone(),
                registry: dep.registry.as_ref().map(|s| s.to_string()),
                source: dep.source.as_ref().map(|s| s.to_string()),
            };

            deps_map.insert(dep.name.clone(), dep_info);
        }
    }

    /// Gets all dependencies
    pub async fn get_dependencies(&self) -> Vec<DependencyInfo> {
        self.dependencies.read().await.values().cloned().collect()
    }

    /// Gets a specific dependency by name
    pub async fn get_dependency(&self, name: &str) -> Option<DependencyInfo> {
        self.dependencies.read().await.get(name).cloned()
    }

    /// Adds a new dependency
    pub async fn add_dependency(&self, dep: DependencyInfo) -> Result<()> {
        // TODO: Implement actual Cargo.toml modification
        self.dependencies
            .write()
            .await
            .insert(dep.name.clone(), dep);
        Ok(())
    }

    /// Updates an existing dependency
    pub async fn update_dependency(&self, name: &str, new_version: &str) -> Result<()> {
        // TODO: Implement actual Cargo.toml modification
        if let Some(dep) = self.dependencies.write().await.get_mut(name) {
            dep.version = new_version.to_string();
        }
        Ok(())
    }

    /// Removes a dependency
    pub async fn remove_dependency(&self, name: &str) -> Result<()> {
        // TODO: Implement actual Cargo.toml modification
        self.dependencies.write().await.remove(name);
        Ok(())
    }
}

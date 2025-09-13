use std::path::Path;

use cargo_edit::{get_dep_version, set_dep_version, upgrade_requirement, LocalManifest};
use cargo_metadata::{Dependency, MetadataCommand, PackageId, SourceKind};
use semver::{Version, VersionReq};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("Failed to read Cargo.toml")]
    ReadManifest(#[from] std::io::Error),
    #[error("Failed to parse Cargo.toml")]
    ParseManifest(#[from] toml::de::Error),
    #[error("Failed to write updated Cargo.toml")]
    WriteManifest(#[from] std::fmt::Error),
    #[error("Failed to fetch crate information: {0}")]
    FetchCrateInfo(String),
    #[error("Failed to update manifest")]
    ManifestUpdateError(#[from] anyhow::Error),
}

pub struct DependencyUpdater {
    manifest_path: std::path::PathBuf,
}

impl DependencyUpdater {
    pub fn new(manifest_path: impl AsRef<Path>) -> Self {
        Self {
            manifest_path: manifest_path.as_ref().to_path_buf(),
        }
    }

    pub async fn update_dependencies(&self, dry_run: bool) -> Result<Vec<DependencyUpdate>, UpdateError> {
        let mut local_manifest = LocalManifest::try_new(&self.manifest_path)?;
        let mut updates = Vec::new();

        let metadata = MetadataCommand::new()
            .manifest_path(&self.manifest_path)
            .exec()
            .map_err(|e| {
                UpdateError::ParseManifest(toml::de::Error::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Metadata load error: {}", e),
                )))
            })?;

        for pkg in &metadata.packages {
            if pkg.manifest_path == self.manifest_path {
                for dep in &pkg.dependencies {
                    if let Some(update) = self.check_for_update(dep.name.as_str(), dep).await? {
                        updates.push(update);
                    }
                }
                break;
            }
        }

        // Apply updates if not in dry run mode
        if !dry_run && !updates.is_empty() {
            self.apply_updates(&updates, &mut local_manifest)?;
        }

        Ok(updates)
    }

    async fn check_for_update(&self, name: &str, dep: &Dependency) -> Result<Option<DependencyUpdate>, UpdateError> {
        // Skip git and path dependencies
        if dep
            .source
            .as_ref()
            .is_some_and(|s| matches!(s.kind, SourceKind::Git(_) | SourceKind::Path(_)))
        {
            return Ok(None);
        }

        // Get current version requirement
        // No longer need separate variable since dep.req is always available

        // Fetch latest version from crates.io
        let latest_version = self.fetch_latest_version(name).await?;

        // Check if update is available
        if let Some(latest) = latest_version {
            if !dep.req.matches(&latest) {
                return Ok(Some(DependencyUpdate {
                    name:            name.to_string(),
                    current_version: dep.req.to_string(),
                    new_version:     latest.to_string(),
                }));
            }
        }

        Ok(None)
    }

    async fn fetch_latest_version(&self, name: &str) -> Result<Option<Version>, UpdateError> {
        // In a real implementation, this would fetch from crates.io API
        // For now, we'll return a dummy version
        Ok(Some(Version::parse("1.0.0").unwrap()))
    }

    fn apply_updates(&self, updates: &[DependencyUpdate], manifest: &mut LocalManifest) -> Result<(), UpdateError> {
        for update in updates {
            // Find the dependency in the manifest
            if let Some(dep_item) = manifest
                .data
                .get_mut("dependencies")
                .and_then(|d| d.get_mut(&update.name))
            {
                set_dep_version(dep_item, &update.new_version)?;
            }
        }
        manifest.write()?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DependencyUpdate {
    pub name:            String,
    pub current_version: String,
    pub new_version:     String,
}

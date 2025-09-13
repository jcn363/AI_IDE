use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use cargo_edit::{get_dep_version, set_dep_version, upgrade_requirement, LocalManifest};
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchUpdateResult {
    pub updated: Vec<DependencyUpdate>,
    pub skipped: Vec<DependencyUpdate>,
    pub failed:  Vec<FailedUpdate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyUpdate {
    pub name:         String,
    pub from_version: String,
    pub to_version:   String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FailedUpdate {
    pub name:  String,
    pub error: String,
}

pub struct BatchUpdater {
    manifest_path: PathBuf,
    dry_run:       bool,
}

impl BatchUpdater {
    pub fn new(manifest_path: impl AsRef<Path>, dry_run: bool) -> Self {
        Self {
            manifest_path: manifest_path.as_ref().to_path_buf(),
            dry_run,
        }
    }

    pub fn update_dependencies(&self, updates: &[(&str, &str)]) -> Result<BatchUpdateResult> {
        let mut manifest = LocalManifest::try_new(&self.manifest_path).context("Failed to load Cargo.toml")?;

        let mut result = BatchUpdateResult {
            updated: Vec::new(),
            skipped: Vec::new(),
            failed:  Vec::new(),
        };

        for (name, version) in updates {
            match self.update_dependency(&mut manifest, name, version) {
                Ok(Some(update)) => result.updated.push(update),
                Ok(None) => {
                    result.skipped.push(DependencyUpdate {
                        name:         name.to_string(),
                        from_version: "unknown".to_string(),
                        to_version:   version.to_string(),
                    });
                }
                Err(e) => {
                    result.failed.push(FailedUpdate {
                        name:  name.to_string(),
                        error: e.to_string(),
                    });
                }
            }
        }

        if !self.dry_run && !result.updated.is_empty() {
            manifest
                .write()
                .context("Failed to write updated Cargo.toml")?;
        }

        Ok(result)
    }

    fn update_dependency(
        &self,
        manifest: &mut LocalManifest,
        name: &str,
        version: &str,
    ) -> Result<Option<DependencyUpdate>> {
        let from_version = manifest
            .data
            .get("dependencies")
            .and_then(|t| t.get(name))
            .and_then(|dep_item| get_dep_version(dep_item).ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Skip if version is the same or invalid
        if from_version == version || Version::parse(version).is_err() {
            return Ok(None);
        }

        // Find the dependency item and update it
        if let Some(dep_item) = manifest
            .data
            .get_mut("dependencies")
            .and_then(|t| t.get_mut(name))
        {
            semver::Version::parse(version).map_err(|e| anyhow::anyhow!("Invalid version {}: {}", version, e))?;
            set_dep_version(dep_item, version)?;
        } else {
            return Err(anyhow::anyhow!("Dependency {} not found", name));
        }

        Ok(Some(DependencyUpdate {
            name: name.to_string(),
            from_version,
            to_version: version.to_string(),
        }))
    }
}

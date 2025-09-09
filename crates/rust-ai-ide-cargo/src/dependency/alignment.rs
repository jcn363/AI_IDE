//! Workspace-wide version alignment for dependencies

use crate::dependency::DependencyManager;
use anyhow::{Context, Result};
use semver::{Version, VersionReq};
use std::collections::HashMap;

/// Represents version alignment information for a dependency
#[derive(Debug, Clone, serde::Serialize)]
pub struct VersionAlignment {
    /// The name of the dependency
    pub name: String,
    /// The current version requirements across the workspace
    pub current_versions: Vec<(String, String)>, // (version_req, source_crate)
    /// The aligned version that should be used
    pub aligned_version: String,
    /// The crates that need to be updated
    pub needs_update: Vec<String>,
}

impl DependencyManager {
    /// Analyzes workspace dependencies and suggests version alignments
    pub async fn analyze_version_alignment(&self) -> Result<Vec<VersionAlignment>> {
        let metadata = self.metadata.read().await;
        let metadata = metadata.as_ref().context("Metadata not loaded")?;

        // Map of package name to (version_req, source_package)
        let mut version_map: HashMap<String, Vec<(String, String)>> = HashMap::new();

        // Collect all dependencies from all workspace members
        for package in &metadata.workspace_members {
            let package = metadata
                .packages
                .iter()
                .find(|p| &p.id == package)
                .context("Package not found in metadata")?;

            // Process normal dependencies
            for dep in &package.dependencies {
                version_map
                    .entry(dep.name.clone())
                    .or_default()
                    .push((dep.req.to_string(), package.name.to_string()));
            }
        }

        // Analyze version requirements and suggest alignments
        let mut alignments = Vec::new();

        for (name, versions) in version_map {
            // Skip if there's only one version requirement
            if versions.len() < 2 {
                continue;
            }

            // Find the highest version that satisfies all requirements
            if let Some(aligned_version) = self.find_compatible_version(&versions) {
                let needs_update: Vec<String> = versions
                    .iter()
                    .filter(|(ver, _)| ver != &aligned_version)
                    .map(|(_, pkg)| pkg.clone())
                    .collect();

                if !needs_update.is_empty() {
                    alignments.push(VersionAlignment {
                        name,
                        current_versions: versions,
                        aligned_version,
                        needs_update,
                    });
                }
            }
        }

        Ok(alignments)
    }

    /// Finds a version that satisfies all version requirements
    fn find_compatible_version(&self, versions: &[(String, String)]) -> Option<String> {
        let parsed_reqs: Result<Vec<VersionReq>, _> = versions
            .iter()
            .map(|(ver, _)| VersionReq::parse(ver))
            .collect();

        let version_reqs = match parsed_reqs {
            Ok(reqs) => reqs,
            Err(_) => return None,
        };

        // Try to find a version that satisfies all requirements
        versions.iter().find_map(|(ver, _)| {
            if let Ok(version) = Version::parse(ver) {
                if version_reqs.iter().all(|req| req.matches(&version)) {
                    return Some(ver.clone());
                }
            }
            None
        })
    }

    /// Applies version alignment to the workspace
    pub async fn apply_version_alignment(&self, alignments: &[VersionAlignment]) -> Result<()> {
        for alignment in alignments {
            for package_name in &alignment.needs_update {
                // In a real implementation, we would update the Cargo.toml files here
                // For now, we'll just log the intended changes
                log::info!(
                    "Would update {} in {} to version {}",
                    alignment.name,
                    package_name,
                    alignment.aligned_version
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cargo_metadata::Dependency as CargoDependency;
    use cargo_metadata::PackageId;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_find_compatible_version() {
        let manager = DependencyManager::new(".");

        let versions = vec![
            (">=1.0.0, <2.0.0".to_string(), "pkg1".to_string()),
            (">=1.2.0, <2.0.0".to_string(), "pkg2".to_string()),
        ];

        let result = manager.find_compatible_version(&versions);
        assert!(result.is_some());
        let version = result.unwrap();
        assert!(version.starts_with(">=1.2.0"));
    }

    // More tests would be added here
}

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use cargo_metadata::Package;
use rust_ai_ide_core::shell_utils;
use semver::Version;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct DependencyInfo {
    pub name:            String,
    pub current_version: String,
    pub latest_version:  String,
    pub update_type:     String, // 'major', 'minor', 'patch'
    pub changelog_url:   Option<String>,
    pub is_direct:       bool,
    pub used_in:         Vec<String>, // Workspace members using this dependency
}

pub struct DependencyUpdateChecker {
    project_path: PathBuf,
}

impl DependencyUpdateChecker {
    pub fn new(project_path: PathBuf) -> Self {
        Self { project_path }
    }

    pub fn check_updates(&self) -> Result<Vec<DependencyInfo>, String> {
        // Get cargo metadata
        let metadata = self.get_cargo_metadata()?;

        // Get outdated dependencies
        let outdated_deps = self.get_outdated_dependencies()?;

        // Process and combine the data
        self.process_dependencies(metadata, outdated_deps)
    }

    fn get_cargo_metadata(&self) -> Result<serde_json::Value, String> {
        let output = Command::new("cargo")
            .current_dir(&self.project_path)
            .args(["metadata", "--format-version=1", "--no-deps"])
            .output()
            .map_err(|e| format!("Failed to run cargo metadata: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "cargo metadata failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        serde_json::from_slice(&output.stdout).map_err(|e| format!("Failed to parse cargo metadata: {}", e))
    }

    fn get_outdated_dependencies(&self) -> Result<Vec<serde_json::Value>, String> {
        let output = Command::new("cargo")
            .current_dir(&self.project_path)
            .args(["outdated", "--format=json"])
            .output()
            .map_err(|e| format!("Failed to run cargo outdated: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "cargo outdated failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .collect())
    }

    fn process_dependencies(
        &self,
        metadata: serde_json::Value,
        outdated_deps: Vec<serde_json::Value>,
    ) -> Result<Vec<DependencyInfo>, String> {
        let mut deps_map = HashMap::new();

        // Get all workspace members
        let empty_members: Vec<&str> = vec![];
        let members: Vec<&str> = if let Some(arr) = metadata["workspace_members"].as_array() {
            arr.iter().filter_map(|m| m.as_str()).collect()
        } else {
            empty_members.clone()
        };

        // Process each package in the workspace
        if let Some(packages) = metadata["packages"].as_array() {
            for package in packages {
                if let Some(package_name) = package["name"].as_str() {
                    // Process dependencies
                    if let Some(deps) = package["dependencies"].as_array() {
                        for dep in deps {
                            if let (Some(name), Some(version)) = (dep["name"].as_str(), dep["req"].as_str()) {
                                let entry = deps_map.entry(name.to_string()).or_insert_with(|| {
                                    let is_direct = members.iter().any(|m| m.starts_with(&format!("{} ", name)));
                                    (version.to_string(), is_direct, Vec::new())
                                });

                                // Track which workspace members use this dependency
                                if !entry.2.contains(&package_name.to_string()) {
                                    entry.2.push(package_name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Process the outdated dependencies
        let mut updates = Vec::new();

        for dep in outdated_deps {
            if let (Some(name), Some(current_version), Some(latest_version), Some(project)) = (
                dep["name"].as_str(),
                dep["project"]
                    .as_str()
                    .or_else(|| dep["compatible"].as_str()),
                dep["latest"].as_str(),
                dep["project"].as_str(),
            ) {
                if let Some((_, is_direct, used_in)) = deps_map.get_mut(name) {
                    let update_type = if let (Ok(current), Ok(latest)) = (
                        Version::parse(current_version.trim_start_matches('^')),
                        Version::parse(latest_version),
                    ) {
                        if current.major < latest.major {
                            "major"
                        } else if current.minor < latest.minor {
                            "minor"
                        } else {
                            "patch"
                        }
                    } else {
                        "unknown"
                    };

                    updates.push(DependencyInfo {
                        name:            name.to_string(),
                        current_version: current_version.to_string(),
                        latest_version:  latest_version.to_string(),
                        update_type:     update_type.to_string(),
                        changelog_url:   self.get_changelog_url(name, project),
                        is_direct:       *is_direct,
                        used_in:         used_in.clone(),
                    });
                }
            }
        }

        Ok(updates)
    }

    fn get_changelog_url(&self, name: &str, version: &str) -> Option<String> {
        // Try to construct a changelog URL based on common patterns
        let base_urls = [
            format!(
                "https://github.com/rust-lang/{}/releases/tag/{}",
                name, version
            ),
            format!(
                "https://github.com/{}/{}/releases/tag/{}",
                name, name, version
            ),
            format!("https://crates.io/crates/{}/{}", name, version),
        ];

        // In a real implementation, we would check if these URLs exist
        // For now, just return the first one that looks reasonable
        base_urls.into_iter().next()
    }
}

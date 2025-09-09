//! Workspace management functionality for Cargo projects

use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Cargo project manager
pub struct CargoManager {
    workspace_root: Option<PathBuf>,
    workspace_members: Vec<PathBuf>,
}

impl CargoManager {
    /// Create a new CargoManager
    pub fn new() -> Self {
        Self {
            workspace_root: None,
            workspace_members: Vec::new(),
        }
    }

    /// Initialize workspace information
    pub async fn initialize_workspace(&mut self, project_path: &Path) -> Result<()> {
        // Find workspace root by looking for Cargo.toml with [workspace] section
        let mut current = project_path.to_path_buf();
        while let Some(parent) = current.parent() {
            let cargo_toml = current.join("Cargo.toml");
            if cargo_toml.exists() {
                let content = std::fs::read_to_string(&cargo_toml)?;
                if content.contains("[workspace]") {
                    self.workspace_root = Some(current);
                    self.load_workspace_members().await?;
                    return Ok(());
                }
            }
            current = parent.to_path_buf();
        }
        Ok(())
    }

    /// Get all workspace members
    pub fn get_workspace_members(&self) -> &[PathBuf] {
        &self.workspace_members
    }

    /// Check if a path is within the current workspace
    pub fn is_in_workspace(&self, path: &Path) -> bool {
        if let Some(workspace_root) = &self.workspace_root {
            path.starts_with(workspace_root)
        } else {
            false
        }
    }

    /// Get a visual representation of the workspace structure
    pub fn get_workspace_tree(&self) -> Result<String> {
        let mut output = String::new();

        if let Some(root) = &self.workspace_root {
            output.push_str(&format!("Workspace Root: {}\n", root.display()));

            for member in &self.workspace_members {
                let relative = member
                    .strip_prefix(root)
                    .unwrap_or(member)
                    .to_string_lossy()
                    .replace(std::path::MAIN_SEPARATOR, "/");
                output.push_str(&format!("├── {}\n", relative));

                // List crate members if this is a workspace member with a Cargo.toml
                if let Ok(cargo_toml) = std::fs::read_to_string(member.join("Cargo.toml")) {
                    if let Ok(value) = toml::from_str::<toml::Value>(&cargo_toml) {
                        if let Some(targets) = value.get("lib").or_else(|| value.get("bin")) {
                            if let Some(name) = targets.get("name").and_then(|n| n.as_str()) {
                                output.push_str(&format!("│   └── {}\n", name));
                            }
                        }
                    }
                }
            }
        }

        Ok(output)
    }

    /// Load all workspace members
    async fn load_workspace_members(&mut self) -> Result<()> {
        if let Some(workspace_root) = &self.workspace_root {
            let output = Command::new("cargo")
                .current_dir(workspace_root)
                .args(["metadata", "--no-deps", "--format-version=1"])
                .output()
                .await?;

            if output.status.success() {
                let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;

                // First, try to get members from the workspace members list
                if let Some(members) = metadata["workspace_members"].as_array() {
                    self.workspace_members = members
                        .iter()
                        .filter_map(|m| m.as_str())
                        .filter_map(|s| {
                            // Format is: package-name version (path+file:///path/to/package)
                            let path_start = s.find('(')?;
                            let path_end = s.rfind(')')?;
                            let path_str = &s[path_start + 1..path_end];
                            let path_str = path_str.trim_start_matches("path+file://");
                            Some(PathBuf::from(path_str))
                        })
                        .filter(|p| p.exists()) // Only keep paths that exist
                        .collect();
                }

                // If no members found, try to get them from the workspace packages
                if self.workspace_members.is_empty() {
                    if let Some(packages) = metadata["packages"].as_array() {
                        self.workspace_members = packages
                            .iter()
                            .filter_map(|p| p["manifest_path"].as_str())
                            .map(PathBuf::from)
                            .filter_map(|p| p.parent().map(Path::to_path_buf)) // Get directory containing Cargo.toml
                            .filter(|p| p.exists())
                            .collect();
                    }
                }
            }
        }
        Ok(())
    }
}

impl Default for CargoManager {
    fn default() -> Self {
        Self::new()
    }
}

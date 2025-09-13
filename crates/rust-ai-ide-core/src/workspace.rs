//! Core workspace management implementation using shared services

use crate::error::IDEError;
use async_trait::async_trait;
use rust_ai_ide_shared_services::{WorkspaceConfig, WorkspaceManager, WorkspaceManagerTrait};
use std::sync::Arc;

/// Core workspace manager with additional functionality
pub struct CoreWorkspaceManager {
    inner: Arc<WorkspaceManager>,
}

impl CoreWorkspaceManager {
    /// Create a new core workspace manager
    pub fn new() -> Self {
        Self {
            inner: Arc::new(WorkspaceManager::new()),
        }
    }

    /// Initialize workspace from configuration
    pub async fn initialize_from_config<P: AsRef<std::path::Path>>(
        &self,
        config_path: P,
    ) -> Result<(), IDEError> {
        // Load workspace configuration from file
        let config_file_path = config_path.as_ref().join(".rust-ai-ide/workspace.json");

        if config_file_path.exists() {
            let config_content =
                std::fs::read_to_string(&config_file_path).map_err(|e| IDEError::from(e))?;

            let config: serde_json::Map<String, serde_json::Value> =
                serde_json::from_str(&config_content).map_err(|e| IDEError::from(e))?;

            // Extract workspace root from config or use directory
            let root_path = config_path.as_ref().to_path_buf();

            let mut workspace_config = WorkspaceConfig::new(&root_path);

            // Apply configuration settings if present
            if let Some(settings) = config.get("settings") {
                if let serde_json::Value::Object(settings_obj) = settings {
                    for (key, value) in settings_obj {
                        workspace_config
                            .set_setting(key.clone(), value.clone())
                            .map_err(|e| IDEError::from(e))?;
                    }
                }
            }

            self.inner.add_workspace(workspace_config).await?;
        } else {
            // No configuration file, create workspace from directory
            let config = WorkspaceConfig::new(config_path.as_ref());
            self.inner.add_workspace(config).await?;
        }

        Ok(())
    }

    /// Get workspace capabilities (e.g., Rust features, analysis tools)
    pub async fn get_workspace_capabilities<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<WorkspaceCapabilities, IDEError> {
        let workspace =
            self.inner
                .get_workspace(&path)
                .await
                .ok_or_else(|| IDEError::FileSystem {
                    message: format!("workspace not found for path: {}", path.as_ref().display()),
                })?;

        let mut capabilities = WorkspaceCapabilities {
            rust_support: false,
            cargo_support: false,
            analysis_support: false,
            file_watching: workspace.watch,
        };

        // Check for Rust project files
        let cargo_toml = workspace.root_path.join("Cargo.toml");
        capabilities.rust_support = cargo_toml.exists();

        // Check for Rust project structure
        let src_dir = workspace.root_path.join("src");
        let main_rs = src_dir.join("main.rs");
        let lib_rs = src_dir.join("lib.rs");
        capabilities.cargo_support = main_rs.exists() || lib_rs.exists();

        // Check for analysis configuration
        capabilities.analysis_support = capabilities.cargo_support;

        Ok(capabilities)
    }

    /// Update workspace settings
    pub async fn update_workspace_settings<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), IDEError> {
        let mut workspace =
            self.inner
                .get_workspace(&path)
                .await
                .ok_or_else(|| IDEError::FileSystem {
                    message: format!("workspace not found for path: {}", path.as_ref().display()),
                })?;

        workspace
            .set_setting(key, value)
            .map_err(|e| IDEError::from(e))?;

        // Re-add the updated workspace
        self.inner.remove_workspace(&path).await;
        self.inner.add_workspace(workspace).await?;

        Ok(())
    }
}

#[async_trait]
impl WorkspaceManagerTrait for CoreWorkspaceManager {
    async fn add_workspace(&self, config: WorkspaceConfig) -> anyhow::Result<()> {
        self.inner.add_workspace(config).await
    }

    async fn remove_workspace(&self, path: &std::path::Path) -> Option<WorkspaceConfig> {
        self.inner.remove_workspace(path).await
    }

    async fn get_workspace(&self, path: &std::path::Path) -> Option<WorkspaceConfig> {
        self.inner.get_workspace(path).await
    }

    async fn list_workspaces(&self) -> Vec<WorkspaceConfig> {
        self.inner.list_workspaces().await
    }

    async fn contains_path(&self, path: &std::path::Path) -> bool {
        self.inner.contains_path(path).await
    }

    async fn workspace_for_path(&self, path: &std::path::Path) -> Option<WorkspaceConfig> {
        self.inner.workspace_for_path(path).await
    }
}

impl Default for CoreWorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Workspace capabilities information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceCapabilities {
    /// Whether Rust development is supported
    pub rust_support: bool,
    /// Whether Cargo is available and configured
    pub cargo_support: bool,
    /// Whether analysis tools are supported
    pub analysis_support: bool,
    /// Whether file watching is enabled
    pub file_watching: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_core_workspace_manager() {
        let temp_dir = tempdir().unwrap();
        let manager = CoreWorkspaceManager::new();

        // Add a workspace
        let config = WorkspaceConfig::new(temp_dir.path());
        manager.add_workspace(config).await.unwrap();

        // Test workspace capabilities
        let capabilities = manager
            .get_workspace_capabilities(temp_dir.path())
            .await
            .unwrap();
        assert_eq!(capabilities.rs_support, false); // No Cargo.toml yet
        assert_eq!(capabilities.cargo_support, false);
        assert_eq!(capabilities.file_watching, true);

        // Test workspace retrieval
        let retrieved = manager.get_workspace(temp_dir.path()).await;
        assert!(retrieved.is_some());
    }
}

use super::IDEResult;
use rust_ai_ide_core_fundamentals::error::IDEError;
use std::path::Path;
use std::sync::Arc;

/// Core workspace manager adapted for file operations - wraps shared services implementation
pub struct CoreWorkspaceManager {
    inner_manager: Arc<rust_ai_ide_shared_services::workspace::WorkspaceManager>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub root_path: std::path::PathBuf,
    pub capabilities: WorkspaceCapabilities,
}

impl Default for CoreWorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreWorkspaceManager {
    pub fn new() -> Self {
        Self {
            inner_manager: Arc::new(
                rust_ai_ide_shared_services::workspace::WorkspaceManager::new(),
            ),
        }
    }

    /// Add a workspace at the given path
    pub async fn add_workspace<P: AsRef<Path>>(&self, path: P) -> IDEResult<()> {
        use rust_ai_ide_shared_services::workspace::WorkspaceConfig;

        let path_buf = path.as_ref().to_path_buf();
        let config = WorkspaceConfig::new(path_buf);
        self.inner_manager
            .add_workspace(config)
            .await
            .map_err(|e| IDEError::FileSystem(e.to_string()))?;
        Ok(())
    }

    /// Get workspace information for a path
    pub async fn get_workspace<P: AsRef<Path>>(&self, path: P) -> Option<WorkspaceInfo> {
        if let Some(config) = self.inner_manager.get_workspace(path).await {
            let capabilities = self
                .analyze_workspace_capabilities(&config.root_path)
                .ok()?;
            Some(WorkspaceInfo {
                root_path: config.root_path,
                capabilities,
            })
        } else {
            None
        }
    }

    /// Analyze workspace capabilities
    pub fn analyze_workspace_capabilities<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> IDEResult<WorkspaceCapabilities> {
        let path = path.as_ref();

        let mut capabilities = WorkspaceCapabilities {
            rust_support: false,
            cargo_support: false,
            analysis_support: false,
        };

        // Check for Rust project files
        let cargo_toml = path.join("Cargo.toml");
        capabilities.rust_support = cargo_toml.exists();

        if capabilities.rust_support {
            // Check for Rust project structure
            let src_dir = path.join("src");
            let main_rs = src_dir.join("main.rs");
            let lib_rs = src_dir.join("lib.rs");
            capabilities.cargo_support = main_rs.exists() || lib_rs.exists();
            capabilities.analysis_support = capabilities.cargo_support;
        }

        Ok(capabilities)
    }

    /// Check if path is within a managed workspace
    pub async fn contains_path<P: AsRef<Path>>(&self, path: P) -> bool {
        self.inner_manager.contains_path(path).await
    }

    /// Find workspace containing the given path
    pub async fn workspace_for_path<P: AsRef<Path>>(&self, path: P) -> Option<WorkspaceInfo> {
        if let Some(config) = self.inner_manager.workspace_for_path(path).await {
            let capabilities = self
                .analyze_workspace_capabilities(&config.root_path)
                .ok()?;
            Some(WorkspaceInfo {
                root_path: config.root_path,
                capabilities,
            })
        } else {
            None
        }
    }

    /// List all managed workspaces
    pub async fn list_workspaces(&self) -> Vec<WorkspaceInfo> {
        let configs = self.inner_manager.list_workspaces().await;
        let mut infos = Vec::new();
        for config in configs {
            if let Some(capabilities) = self.analyze_workspace_capabilities(&config.root_path).ok()
            {
                infos.push(WorkspaceInfo {
                    root_path: config.root_path,
                    capabilities,
                });
            }
        }
        infos
    }
}

/// Workspace capabilities information
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkspaceCapabilities {
    pub rust_support: bool,
    pub cargo_support: bool,
    pub analysis_support: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_workspace_manager_basic() {
        let temp_dir = tempdir().unwrap();
        let manager = CoreWorkspaceManager::new();

        // Add a workspace
        manager.add_workspace(temp_dir.path()).await.unwrap();

        // Check if it's tracked
        assert!(manager.contains_path(temp_dir.path()).await);

        // Get workspace info
        let info = manager.get_workspace(temp_dir.path()).await.unwrap();
        assert_eq!(info.root_path, temp_dir.path());
    }
}

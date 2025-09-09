//! LSP-specific workspace management using shared services foundation

use lsp_types::{DidChangeWatchedFilesParams, FileEvent, Uri, WorkspaceFolder};
use rust_ai_ide_shared_services::{
    WorkspaceConfig as SharedWorkspaceConfig, WorkspaceManagerTrait,
};
use std::path::Path;
use std::sync::Arc;

/// LSP workspace adapter using shared services
pub struct LspWorkspaceAdapter {
    shared_manager: Arc<dyn WorkspaceManagerTrait>,
}

impl LspWorkspaceAdapter {
    /// Create a new LSP workspace adapter
    pub fn new(shared_manager: Arc<dyn WorkspaceManagerTrait>) -> Self {
        Self { shared_manager }
    }

    /// Convert a shared workspace config to LSP workspace folder
    pub fn config_to_workspace_folder(config: &SharedWorkspaceConfig) -> Option<WorkspaceFolder> {
        // Convert path to URI
        url::Url::from_file_path(&config.root_path)
            .ok()
            .and_then(|url| url.as_str().parse::<Uri>().ok())
            .map(|uri| WorkspaceFolder {
                uri,
                name: config
                    .root_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("workspace")
                    .to_string(),
            })
    }

    /// Convert LSP workspace folder to shared workspace config
    pub fn workspace_folder_to_config(folder: &WorkspaceFolder) -> Option<SharedWorkspaceConfig> {
        let url = url::Url::parse(folder.uri.as_str()).ok()?;
        let path = url.to_file_path().ok()?;
        let mut config = SharedWorkspaceConfig::new(path);

        // Set workspace name in settings if it's not the default
        let default_name = config
            .root_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("workspace");

        if folder.name != default_name {
            let _ = config.set_setting("lsp.name", folder.name.clone());
        }

        Some(config)
    }

    /// Get all workspace folders from shared manager
    pub async fn get_workspace_folders(&self) -> Vec<WorkspaceFolder> {
        let configs = self.shared_manager.list_workspaces().await;
        configs
            .iter()
            .filter_map(Self::config_to_workspace_folder)
            .collect()
    }

    /// Add workspace from LSP folder
    pub async fn add_workspace_folder(&self, folder: &WorkspaceFolder) -> anyhow::Result<()> {
        if let Some(config) = Self::workspace_folder_to_config(folder) {
            self.shared_manager.add_workspace(config).await
        } else {
            Err(anyhow::anyhow!("Failed to convert workspace folder"))
        }
    }

    /// Remove workspace by path
    pub async fn remove_workspace_folder(&self, path: &Path) -> Option<SharedWorkspaceConfig> {
        self.shared_manager.remove_workspace(path).await
    }

    /// Handle LSP file change notifications
    pub async fn handle_file_events(&self, params: DidChangeWatchedFilesParams) {
        // Process file change events for diagnostics, analysis updates, etc.
        for event in params.changes {
            self.handle_file_event(event).await;
        }
    }

    /// Handle individual file change event
    async fn handle_file_event(&self, event: FileEvent) {
        let uri = &event.uri;

        if let Ok(url) = url::Url::parse(uri.as_str()) {
            if let Ok(path) = url.to_file_path() {
                // Check if the changed file belongs to any managed workspace
                if self.shared_manager.contains_path(&path).await {
                    match event.typ {
                        lsp_types::FileChangeType::CREATED => {
                            // Handle file creation (e.g., update file indexing)
                            tracing::debug!("File created: {:?}", path);
                        }
                        lsp_types::FileChangeType::CHANGED => {
                            // Handle file modification (e.g., trigger diagnostics)
                            tracing::debug!("File changed: {:?}", path);
                        }
                        lsp_types::FileChangeType::DELETED => {
                            // Handle file deletion (e.g., cleanup diagnostics)
                            tracing::debug!("File deleted: {:?}", path);
                        }
                        _ => {
                            tracing::warn!("Unknown file change type for {:?}", path);
                        }
                    }
                }
            }
        }
    }

    /// Get workspace capabilities for specific features
    pub async fn get_workspace_capabilities(&self, uri: &Uri) -> LspWorkspaceCapabilities {
        if let Ok(url) = url::Url::parse(uri.as_str()) {
            if let Ok(path) = url.to_file_path() {
                if let Some(config) = self.shared_manager.workspace_for_path(&path).await {
                    // Get basic capabilities from shared manager
                    let supports_diagnostics = config.watch;

                    // Additional LSP-specific capabilities
                    let supports_completion = true;
                    let supports_hover = true;
                    let supports_signature_help = true;
                    let supports_definition = true;
                    let supports_references = true;
                    let supports_document_highlight = true;
                    let supports_document_symbol = true;
                    let supports_workspace_symbol = true;
                    let supports_code_action = true;
                    let supports_code_lens = true;
                    let supports_formatting = true;
                    let supports_range_formatting = true;
                    let supports_rename = true;

                    let rust_support = config.root_path.join("Cargo.toml").exists();

                    LspWorkspaceCapabilities {
                        supports_diagnostics,
                        supports_completion,
                        supports_hover,
                        supports_signature_help,
                        supports_definition,
                        supports_references,
                        supports_document_highlight,
                        supports_document_symbol,
                        supports_workspace_symbol,
                        supports_code_action,
                        supports_code_lens,
                        supports_formatting,
                        supports_range_formatting,
                        supports_rename,
                        has_rust_project: rust_support,
                    }
                } else {
                    // No workspace found for this URI
                    Default::default()
                }
            } else {
                // Invalid file path
                Default::default()
            }
        } else {
            // Invalid URI
            Default::default()
        }
    }
}

/// LSP-specific workspace capabilities
#[derive(Debug, Clone, Default)]
pub struct LspWorkspaceCapabilities {
    pub supports_diagnostics: bool,
    pub supports_completion: bool,
    pub supports_hover: bool,
    pub supports_signature_help: bool,
    pub supports_definition: bool,
    pub supports_references: bool,
    pub supports_document_highlight: bool,
    pub supports_document_symbol: bool,
    pub supports_workspace_symbol: bool,
    pub supports_code_action: bool,
    pub supports_code_lens: bool,
    pub supports_formatting: bool,
    pub supports_range_formatting: bool,
    pub supports_rename: bool,
    pub has_rust_project: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_ai_ide_shared_services::WorkspaceManager;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_lsp_adapter() {
        let temp_dir = tempdir().unwrap();
        let shared_manager = Arc::new(WorkspaceManager::new());

        // Create LSP adapter
        let adapter = LspWorkspaceAdapter::new(shared_manager.clone());

        // Create workspace via shared manager
        let config = SharedWorkspaceConfig::new(temp_dir.path());
        shared_manager.add_workspace(config.clone()).await.unwrap();

        // Test getting workspace folders
        let folders = adapter.get_workspace_folders().await;
        assert_eq!(folders.len(), 1);

        let folder = &folders[0];
        assert!(folder.uri.to_string().contains("file://"));
    }

    #[tokio::test]
    async fn test_workspace_capabilities() {
        let temp_dir = tempdir().unwrap();
        let shared_manager = Arc::new(WorkspaceManager::new());
        let adapter = LspWorkspaceAdapter::new(shared_manager.clone());

        // No workspace yet
        let dummy_uri = lsp_types::Uri::from_file_path(temp_dir.path().join("file.rs")).unwrap();
        let caps = adapter.get_workspace_capabilities(&dummy_uri).await;
        assert!(!caps.has_rust_project);

        // Add workspace with Rust project
        let mut config = SharedWorkspaceConfig::new(temp_dir.path());
        config.set_setting("lsp.file_watching", true).unwrap();
        shared_manager.add_workspace(config).await.unwrap();

        let caps = adapter.get_workspace_capabilities(&dummy_uri).await;
        assert!(caps.supports_diagnostics); // From shared manager
                                            // LSP capabilities default to true when workspace exists
        assert!(caps.supports_completion);
        assert!(caps.supports_hover);
    }
}

//! Unified workspace management interfaces and implementations

use async_trait::async_trait;
use rust_ai_ide_common::caching::{Cache, MemoryCache};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

/// Configuration for a workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceConfig {
    /// The root path of the workspace
    pub root_path: PathBuf,
    /// Whether to watch for file changes
    pub watch: bool,
    /// Additional settings specific to this workspace
    pub settings: HashMap<String, serde_json::Value>,
}

impl WorkspaceConfig {
    /// Create a new workspace configuration
    pub fn new<P: AsRef<Path>>(root_path: P) -> Self {
        Self {
            root_path: root_path.as_ref().to_path_buf(),
            watch: true,
            settings: HashMap::new(),
        }
    }

    /// Get a setting value
    pub fn get_setting<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, serde_json::Error> {
        self.settings
            .get(key)
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()
    }

    /// Set a setting value
    pub fn set_setting<T: Serialize>(
        &mut self,
        key: impl Into<String>,
        value: T,
    ) -> Result<(), serde_json::Error> {
        let value = serde_json::to_value(value)?;
        self.settings.insert(key.into(), value);
        Ok(())
    }
}

/// Core workspace management trait
#[async_trait]
pub trait WorkspaceManagerTrait: Send + Sync {
    /// Add a workspace
    async fn add_workspace(&self, config: WorkspaceConfig) -> anyhow::Result<()>;

    /// Remove a workspace by path
    async fn remove_workspace(&self, path: &Path) -> Option<WorkspaceConfig>;

    /// Get workspace configuration by path
    async fn get_workspace(&self, path: &Path) -> Option<WorkspaceConfig>;

    /// Get all workspaces
    async fn list_workspaces(&self) -> Vec<WorkspaceConfig>;

    /// Check if a path belongs to any managed workspace
    async fn contains_path(&self, path: &Path) -> bool;

    /// Get workspace that contains the given path
    async fn workspace_for_path(&self, path: &Path) -> Option<WorkspaceConfig>;
}

/// File metadata cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub size: u64,
    pub modified: SystemTime,
    pub is_dir: bool,
}

/// Caches for workspace metadata and configurations
pub struct WorkspaceCaches {
    pub file_metadata: MemoryCache<String, FileMetadata>,
    pub config_cache: MemoryCache<PathBuf, WorkspaceConfig>,
}

/// Manages multiple workspaces in a thread-safe manner
pub struct WorkspaceManager {
    workspaces: RwLock<HashMap<PathBuf, WorkspaceConfig>>,
    caches: WorkspaceCaches,
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceManager {
    /// Create a new workspace manager
    pub fn new() -> Self {
        Self {
            workspaces: RwLock::new(HashMap::new()),
            caches: WorkspaceCaches {
                file_metadata: MemoryCache::with_capacity(1000),
                config_cache: MemoryCache::with_capacity(100),
            },
        }
    }

    /// Add a workspace to the manager (with cache invalidation)
    pub async fn add_workspace(&self, config: WorkspaceConfig) -> anyhow::Result<()> {
        let path_buf = config.root_path.clone();
        let mut workspaces = self.workspaces.write().await;
        workspaces.insert(path_buf.clone(), config.clone());

        // Update cache
        self.caches
            .config_cache
            .insert(path_buf, config, Some(Duration::from_secs(300)))
            .await;
        Ok(())
    }

    /// Remove a workspace by path (with cache invalidation)
    pub async fn remove_workspace<P: AsRef<Path>>(&self, path: P) -> Option<WorkspaceConfig> {
        let path_buf = path.as_ref().to_path_buf();
        let removed = {
            let mut workspaces = self.workspaces.write().await;
            workspaces.remove(&path_buf)
        };

        // Invalidate cache
        if removed.is_some() {
            self.caches.config_cache.remove(&path_buf).await;
        }
        removed
    }

    /// Get workspace configuration by path (with caching)
    pub async fn get_workspace<P: AsRef<Path>>(&self, path: P) -> Option<WorkspaceConfig> {
        let path_buf = path.as_ref().to_path_buf();

        // Check cache first
        if let Some(config) = self.caches.config_cache.get(&path_buf).await {
            return Some(config);
        }

        // Fallback to storage
        let workspaces = self.workspaces.read().await;
        if let Some(config) = workspaces.get(&path_buf) {
            // Cache for future requests
            self.caches
                .config_cache
                .insert(path_buf, (*config).clone(), Some(Duration::from_secs(300)))
                .await;
            return Some((*config).clone());
        }
        None
    }

    /// Get all workspaces
    pub async fn list_workspaces(&self) -> Vec<WorkspaceConfig> {
        let workspaces = self.workspaces.read().await;
        workspaces.values().cloned().collect()
    }

    /// Check if path belongs to any managed workspace
    pub async fn contains_path<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();
        let workspaces = self.workspaces.read().await;

        for workspace_path in workspaces.keys() {
            if path.starts_with(workspace_path) {
                return true;
            }
        }
        false
    }

    /// Get the workspace that contains the given path
    pub async fn workspace_for_path<P: AsRef<Path>>(&self, path: P) -> Option<WorkspaceConfig> {
        let path = path.as_ref();
        let workspaces = self.workspaces.read().await;

        for (workspace_path, config) in workspaces.iter() {
            if path.starts_with(workspace_path) {
                return Some(config.clone());
            }
        }
        None
    }

    /// Get file metadata with caching
    pub async fn get_file_metadata<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> anyhow::Result<Option<FileMetadata>> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path encoding"))?
            .to_string();

        // Check cache first
        if let Some(metadata) = self.caches.file_metadata.get(&path_str).await {
            return Ok(Some(metadata));
        }

        // Fetch from filesystem
        if let Ok(metadata) = tokio::fs::metadata(&path.as_ref()).await {
            let file_metadata = FileMetadata {
                size: metadata.len(),
                modified: metadata.modified().unwrap_or_else(|_| SystemTime::now()),
                is_dir: metadata.is_dir(),
            };

            // Cache with 60 second TTL
            self.caches
                .file_metadata
                .insert(
                    path_str,
                    file_metadata.clone(),
                    Some(Duration::from_secs(60)),
                )
                .await;
            Ok(Some(file_metadata))
        } else {
            Ok(None)
        }
    }

    /// Invalidate cache when files change
    pub async fn invalidate_cache_on_file_change<P: AsRef<Path>>(&self, path: P) {
        let path_str = path.as_ref().to_str().unwrap_or("").to_string();
        self.caches.file_metadata.remove(&path_str).await;

        // Also invalidate workspace cache if it's a config file
        let path_buf = path.as_ref().to_path_buf();
        self.caches.config_cache.remove(&path_buf).await;
    }

    /// Warm up caches by preloading common data
    pub async fn warmup_workspace(&self, config: &WorkspaceConfig) -> anyhow::Result<()> {
        use rust_ai_ide_common::fs_utils::list_files_recursive;

        // Warm up file metadata for common directories
        let common_paths = vec![
            config.root_path.join("src"),
            config.root_path.join("Cargo.toml"),
            config.root_path.join(".gitignore"),
        ];

        for path in common_paths {
            if path.exists() {
                if path.is_dir() {
                    if let Ok(files) = list_files_recursive(&path, Some(2)).await {
                        for file in files.into_iter().take(50) {
                            // Limit to avoid overwhelming
                            let _ = self.get_file_metadata(&file).await;
                        }
                    }
                } else {
                    let _ = self.get_file_metadata(&path).await;
                }
            }
        }

        Ok(())
    }

    /// Get cache statistics for monitoring
    pub fn get_cache_stats(
        &self,
    ) -> (
        rust_ai_ide_common::caching::CacheStats,
        rust_ai_ide_common::caching::CacheStats,
    ) {
        (
            self.caches.file_metadata.stats(),
            self.caches.config_cache.stats(),
        )
    }
}

#[async_trait]
impl WorkspaceManagerTrait for WorkspaceManager {
    async fn add_workspace(&self, config: WorkspaceConfig) -> anyhow::Result<()> {
        self.add_workspace(config).await
    }

    async fn remove_workspace(&self, path: &Path) -> Option<WorkspaceConfig> {
        self.remove_workspace(path).await
    }

    async fn get_workspace(&self, path: &Path) -> Option<WorkspaceConfig> {
        self.get_workspace(path).await
    }

    async fn list_workspaces(&self) -> Vec<WorkspaceConfig> {
        self.list_workspaces().await
    }

    async fn contains_path(&self, path: &Path) -> bool {
        self.contains_path(path).await
    }

    async fn workspace_for_path(&self, path: &Path) -> Option<WorkspaceConfig> {
        self.workspace_for_path(path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_workspace_config() {
        let temp_dir = tempdir().unwrap();
        let mut config = WorkspaceConfig::new(temp_dir.path());

        // Test setting and getting a value
        config.set_setting("rust.checkOnSave", true).unwrap();
        let check_on_save: Option<bool> = config.get_setting("rust.checkOnSave").unwrap();
        assert_eq!(check_on_save, Some(true));

        // Test getting a non-existent setting
        let missing: Option<bool> = config.get_setting("missing").unwrap();
        assert!(missing.is_none());
    }

    #[tokio::test]
    async fn test_workspace_manager() {
        let temp_dir = tempdir().unwrap();
        let manager = WorkspaceManager::new();

        // Add a workspace
        let config = WorkspaceConfig::new(temp_dir.path());
        manager.add_workspace(config.clone()).await.unwrap();

        // Verify workspace exists
        let retrieved = manager.get_workspace(temp_dir.path()).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().root_path, temp_dir.path());

        // Verify path contains check
        assert!(
            manager
                .contains_path(temp_dir.path().join("src/main.rs"))
                .await
        );

        // Verify workspace for path
        let workspace = manager
            .workspace_for_path(temp_dir.path().join("src"))
            .await;
        assert!(workspace.is_some());
        assert_eq!(workspace.unwrap().root_path, temp_dir.path());

        // List workspaces
        let workspaces = manager.list_workspaces().await;
        assert_eq!(workspaces.len(), 1);

        // Remove workspace
        let removed = manager.remove_workspace(temp_dir.path()).await;
        assert!(removed.is_some());
        assert!(!manager.contains_path(temp_dir.path()).await);
    }

    #[tokio::test]
    async fn test_trait_implementation() {
        let temp_dir = tempdir().unwrap();
        let manager: Box<dyn WorkspaceManagerTrait> = Box::new(WorkspaceManager::new());

        let config = WorkspaceConfig::new(temp_dir.path());
        manager.add_workspace(config).await.unwrap();

        assert!(manager.get_workspace(temp_dir.path()).await.is_some());
    }
}

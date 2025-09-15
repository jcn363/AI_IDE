//! Plugin Discovery System
//!
//! This module provides advanced plugin discovery mechanisms that can scan multiple
//! directories and paths for plugins with different formats (JSON configs, dynamic libraries).

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use tokio::fs;
use walkdir::WalkDir;

use crate::interfaces::{plugin_error, PluginError};

/// Plugin discovery configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Directories to scan for plugins
    pub scan_paths:      Vec<PathBuf>,
    /// File patterns to look for
    pub file_patterns:   Vec<String>,
    /// Maximum depth for directory scanning
    pub max_depth:       usize,
    /// Follow symbolic links
    pub follow_symlinks: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            scan_paths:      vec![
                dirs::home_dir()
                    .map(|d| d.join(".rust-ai-ide").join("plugins"))
                    .unwrap_or_default(),
                dirs::data_local_dir()
                    .map(|d| d.join("rust-ai-ide").join("plugins"))
                    .unwrap_or_default(),
                PathBuf::from("./plugins"),
            ],
            file_patterns:   vec![
                "plugin.json".to_string(),
                "plugin.toml".to_string(),
                "plugin.yaml".to_string(),
                "plugin.yml".to_string(),
            ],
            max_depth:       3,
            follow_symlinks: false,
        }
    }
}

/// Advanced plugin discovery system
pub struct PluginDiscovery {
    config:             DiscoveryConfig,
    discovered_plugins: tokio::sync::RwLock<HashSet<String>>,
}

impl PluginDiscovery {
    /// Create a new plugin discovery instance
    pub fn new(scan_paths: &[PathBuf]) -> Self {
        let mut config = DiscoveryConfig::default();
        config.scan_paths = scan_paths.to_vec();

        Self {
            config,
            discovered_plugins: tokio::sync::RwLock::new(HashSet::new()),
        }
    }

    /// Create a new plugin discovery with full configuration
    pub fn with_config(config: DiscoveryConfig) -> Self {
        Self {
            config,
            discovered_plugins: tokio::sync::RwLock::new(HashSet::new()),
        }
    }

    /// Scan all configured paths for plugins
    pub async fn scan_all_paths(&self) -> Result<Vec<String>, PluginError> {
        let mut all_plugins = Vec::new();
        let mut discovered = self.discovered_plugins.write().await;

        for path in &self.config.scan_paths {
            if path.exists() {
                match self.scan_directory(path).await {
                    Ok(plugins) =>
                        for plugin_id in plugins {
                            if discovered.insert(plugin_id.clone()) {
                                all_plugins.push(plugin_id);
                            }
                        },
                    Err(e) => {
                        eprintln!("Failed to scan directory {}: {:?}", path.display(), e);
                    }
                }
            } else {
                eprintln!("Plugin directory does not exist: {}", path.display());
            }
        }

        Ok(all_plugins)
    }

    /// Scan a specific directory for plugins
    pub async fn scan_directory(&self, directory: &Path) -> Result<Vec<String>, PluginError> {
        let mut plugins = Vec::new();

        // Use WalkDir for efficient directory traversal
        let walker = WalkDir::new(directory)
            .max_depth(self.config.max_depth)
            .follow_links(self.config.follow_symlinks)
            .into_iter();

        for entry in walker.filter_entry(|e| {
            // Skip hidden directories and files
            !e.file_name().to_string_lossy().starts_with('.')
        }) {
            match entry {
                Ok(entry) => {
                    if entry.file_type().is_file() {
                        if let Some(plugin_id) = self.check_plugin_file(&entry.path()).await {
                            plugins.push(plugin_id);
                        }
                    } else if entry.file_type().is_dir() {
                        // Also check for plugin.json in subdirectories
                        let plugin_config = entry.path().join("plugin.json");
                        if plugin_config.exists() {
                            if let Some(plugin_id) = self.check_plugin_file(&plugin_config).await {
                                plugins.push(plugin_id);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error scanning directory: {:?}", e);
                }
            }
        }

        Ok(plugins)
    }

    /// Check if a file is a valid plugin configuration and extract plugin ID
    async fn check_plugin_file(&self, file_path: &Path) -> Option<String> {
        // Check if file matches our patterns
        let file_name = file_path.file_name()?.to_string_lossy();

        if !self
            .config
            .file_patterns
            .iter()
            .any(|pattern| file_name.ends_with(pattern))
        {
            return None;
        }

        // Try to read and parse the plugin configuration
        if let Ok(content) = fs::read_to_string(file_path).await {
            // Simple JSON parsing for now (could be extended for other formats)
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(id) = value.get("id").and_then(|v| v.as_str()) {
                    return Some(id.to_string());
                }
            }
        }

        None
    }

    /// Add a new discovery path
    pub fn add_scan_path(&mut self, path: PathBuf) {
        self.config.scan_paths.push(path);
    }

    /// Get currently discovered plugins
    pub async fn get_discovered_plugins(&self) -> Vec<String> {
        self.discovered_plugins
            .read()
            .await
            .iter()
            .cloned()
            .collect()
    }

    /// Clear the discovered plugins cache
    pub async fn clear_cache(&self) {
        self.discovered_plugins.write().await.clear();
    }

    /// Discover plugins from a remote source (placeholder for future marketplace integration)
    pub async fn discover_from_remote(&self, _url: &str) -> Result<Vec<String>, PluginError> {
        // Placeholder for future implementation
        Err(plugin_error("Remote plugin discovery not implemented yet"))
    }

    /// Monitor a directory for new plugin installations (one-time scan enhancement)
    pub async fn scan_for_new_plugins(&self, directory: &Path) -> Result<Vec<String>, PluginError> {
        let new_plugins = self.scan_directory(directory).await?;
        let mut discovered = self.discovered_plugins.write().await;
        let mut actually_new = Vec::new();

        for plugin in new_plugins {
            if discovered.insert(plugin.clone()) {
                actually_new.push(plugin);
            }
        }

        Ok(actually_new)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tempfile::TempDir;
    use tokio::fs::write;

    use super::*;

    #[tokio::test]
    async fn test_discovery_with_valid_plugin() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_path = temp_dir.path().join("plugin.json");

        let plugin_config = json!({
            "id": "test-plugin-1",
            "name": "Test Plugin",
            "version": "1.0.0"
        });

        write(
            &plugin_path,
            serde_json::to_string_pretty(&plugin_config).unwrap(),
        )
        .await
        .unwrap();

        let discovery = PluginDiscovery::new(&[temp_dir.path().to_path_buf()]);
        let plugins = discovery.scan_all_paths().await.unwrap();

        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0], "test-plugin-1");
    }

    #[tokio::test]
    async fn test_discovery_without_plugins() {
        let temp_dir = TempDir::new().unwrap();

        let discovery = PluginDiscovery::new(&[temp_dir.path().to_path_buf()]);
        let plugins = discovery.scan_all_paths().await.unwrap();

        assert_eq!(plugins.len(), 0);
    }
}

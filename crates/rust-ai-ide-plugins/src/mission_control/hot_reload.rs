//! Hot-Reload System for Plugin Development
//!
//! This module provides real-time monitoring of plugin files and directories,
//! enabling automatic reloading of plugins during development.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use notify::RecommendedWatcher;
use tokio::fs;
use tokio::sync::{broadcast, RwLock};
use tokio::time::Duration;

use crate::interfaces::PluginError;
use crate::registry::PluginRegistry;

/// File modification record
#[derive(Debug, Clone)]
struct FileRecord {
    modified: SystemTime,
    size: u64,
}

/// Hot-reload watcher configuration
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    /// Paths to monitor for changes
    pub watch_paths: Vec<PathBuf>,
    /// File patterns to monitor
    pub watch_patterns: Vec<String>,
    /// Watch interval (how often to check for changes)
    pub poll_interval: Duration,
    /// Debounce time for rapid file changes
    pub debounce_time: Duration,
    /// Maximum depth for directory watching
    pub max_depth: usize,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            watch_paths: vec![
                PathBuf::from("./plugins"),
                dirs::home_dir()
                    .map(|d| d.join(".rust-ai-ide").join("plugins"))
                    .unwrap_or_default(),
            ],
            watch_patterns: vec![
                "*.json".to_string(),
                "*.toml".to_string(),
                "*.yaml".to_string(),
                "*.yml".to_string(),
                "*.so".to_string(),
                "*.dylib".to_string(),
                "*.dll".to_string(),
            ],
            poll_interval: Duration::from_millis(500),
            debounce_time: Duration::from_millis(100),
            max_depth: 3,
        }
    }
}

/// Hot-reload watcher for real-time plugin monitoring
pub struct HotReloadWatcher {
    config: WatcherConfig,
    file_records: Arc<RwLock<HashMap<PathBuf, FileRecord>>>,
    running: Arc<RwLock<bool>>,
    _watcher: Option<RecommendedWatcher>,
}

impl HotReloadWatcher {
    /// Create a new hot-reload watcher
    pub fn new(watch_paths: Vec<PathBuf>, poll_interval: Duration) -> Self {
        let mut config = WatcherConfig::default();
        config.watch_paths = watch_paths;
        config.poll_interval = poll_interval;

        Self {
            config,
            file_records: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            _watcher: None,
        }
    }

    /// Create watcher with full configuration
    pub fn with_config(config: WatcherConfig) -> Self {
        Self {
            config,
            file_records: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            _watcher: None,
        }
    }

    /// Start the hot-reload monitoring system
    pub async fn start(
        &mut self,
        registries: Arc<RwLock<Vec<PluginRegistry>>>,
        event_tx: broadcast::Sender<PluginEvent>,
    ) -> Result<(), PluginError> {
        let mut running = self.running.write().await;
        if *running {
            return Err(PluginError::Other("Watcher is already running".to_string()));
        }
        *running = true;
        drop(running);

        // Initialize file records for all watch paths
        self.initialize_file_records().await?;

        // Start the file watching loop
        self.start_watch_loop(registries, event_tx).await;

        Ok(())
    }

    /// Stop the hot-reload monitoring system
    pub async fn stop(&mut self) {
        let mut running = self.running.write().await;
        *running = false;
    }

    /// Check if the watcher is currently running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Add a new path to watch
    pub fn add_watch_path(&mut self, path: PathBuf) {
        self.config.watch_paths.push(path);
    }

    /// Initialize file modification records for monitoring
    async fn initialize_file_records(&self) -> Result<(), PluginError> {
        let mut records = self.file_records.write().await;

        for watch_path in &self.config.watch_paths {
            if watch_path.exists() {
                self.scan_directory_for_files(watch_path, &mut records)
                    .await?;
            }
        }

        Ok(())
    }

    /// Recursively scan directory for plugin-related files
    async fn scan_directory_for_files(
        &self,
        directory: &Path,
        records: &mut HashMap<PathBuf, FileRecord>,
    ) -> Result<(), PluginError> {
        let mut entries = fs::read_dir(directory).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_file() {
                if self.should_watch_file(&path) {
                    if let Ok(metadata) = fs::metadata(&path).await {
                        let record = FileRecord {
                            modified: metadata.modified().unwrap_or(SystemTime::now()),
                            size: metadata.len(),
                        };
                        records.insert(path, record);
                    }
                }
            } else if path.is_dir() && self.should_watch_directory(&path) {
                Box::pin(self.scan_directory_for_files(&path, records)).await?;
            }
        }

        Ok(())
    }

    /// Check if a file should be monitored based on patterns
    fn should_watch_file(&self, path: &Path) -> bool {
        if let Some(file_name) = path.file_name() {
            let file_str = file_name.to_string_lossy();

            for pattern in &self.config.watch_patterns {
                if pattern.starts_with("*.") {
                    let ext = pattern.strip_prefix("*").unwrap();
                    if file_str.ends_with(ext) {
                        return true;
                    }
                } else if file_str.contains(pattern) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a directory should be monitored
    fn should_watch_directory(&self, path: &Path) -> bool {
        // Skip hidden and certain special directories
        if let Some(dir_name) = path.file_name() {
            let dir_str = dir_name.to_string_lossy();
            !dir_str.starts_with('.') && dir_str != "target" && dir_str != "node_modules"
        } else {
            false
        }
    }

    /// Main watch loop for detecting file changes
    async fn start_watch_loop(
        &self,
        registries: Arc<RwLock<Vec<PluginRegistry>>>,
        event_tx: broadcast::Sender<PluginEvent>,
    ) {
        let file_records = Arc::clone(&self.file_records);
        let config = self.config.clone();
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.poll_interval);

            while *running.read().await {
                interval.tick().await;

                if let Err(e) =
                    Self::check_for_changes(&file_records, &config, &registries, &event_tx).await
                {
                    eprintln!("Error checking for plugin changes: {:?}", e);
                }
            }
        });
    }

    /// Check for file changes and trigger reloads if necessary
    async fn check_for_changes(
        file_records: &Arc<RwLock<HashMap<PathBuf, FileRecord>>>,
        config: &WatcherConfig,
        registries: &Arc<RwLock<Vec<PluginRegistry>>>,
        event_tx: &broadcast::Sender<PluginEvent>,
    ) -> Result<(), PluginError> {
        let mut records = file_records.write().await;
        let mut changed_plugins = Vec::new();

        // Rescan all directories to find changes
        for watch_path in &config.watch_paths {
            if watch_path.exists() {
                Self::scan_for_changes(watch_path, &mut records, config, &mut changed_plugins)
                    .await?;
            }
        }

        // Trigger reloads for changed plugins
        for plugin_path in changed_plugins {
            if let Some(plugin_id) = Self::extract_plugin_id_from_path(&plugin_path) {
                let _ = event_tx.send(PluginEvent::HotReloadTriggered(plugin_id.clone()));

                // Reload the plugin in all registries
                let reload_registries = registries.read().await;
                for registry in reload_registries.iter() {
                    Self::reload_plugin_in_registry(registry, &plugin_id).await;
                }
            }
        }

        Ok(())
    }

    /// Scan a directory for file changes
    async fn scan_for_changes(
        directory: &Path,
        records: &mut HashMap<PathBuf, FileRecord>,
        config: &WatcherConfig,
        changed_plugins: &mut Vec<PathBuf>,
    ) -> Result<(), PluginError> {
        let mut entries = fs::read_dir(directory).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_file()
                && config
                    .watch_patterns
                    .iter()
                    .any(|p| Self::matches_pattern(&path, p))
            {
                let current_modified = if let Ok(metadata) = fs::metadata(&path).await {
                    metadata.modified().unwrap_or(SystemTime::now())
                } else {
                    continue;
                };

                let current_size = fs::metadata(&path).await.ok().map(|m| m.len()).unwrap_or(0);

                let needs_reload = if let Some(existing) = records.get(&path) {
                    existing.modified != current_modified || existing.size != current_size
                } else {
                    // New file
                    true
                };

                if needs_reload {
                    let record = FileRecord {
                        modified: current_modified,
                        size: current_size,
                    };
                    records.insert(path.clone(), record);
                    changed_plugins.push(path);
                }
            } else if path.is_dir() {
                Box::pin(Self::scan_for_changes(
                    &path,
                    records,
                    config,
                    changed_plugins,
                ))
                .await?;
            }
        }

        Ok(())
    }

    /// Check if a file path matches a pattern
    fn matches_pattern(path: &Path, pattern: &str) -> bool {
        if let Some(file_name) = path.file_name() {
            let file_str = file_name.to_string_lossy();

            if pattern.starts_with("*.") {
                let ext = pattern.strip_prefix("*").unwrap();
                file_str.ends_with(ext)
            } else {
                file_str.contains(pattern)
            }
        } else {
            false
        }
    }

    /// Extract plugin ID from file path
    fn extract_plugin_id_from_path(path: &Path) -> Option<String> {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(id) = value.get("id").and_then(|v| v.as_str()) {
                    return Some(id.to_string());
                }
            }
        }
        None
    }

    /// Reload a plugin in a registry
    async fn reload_plugin_in_registry(registry: &PluginRegistry, plugin_id: &str) {
        // This would unload and reload the plugin
        // For now, just log the event
        println!("Hot-reload triggered for plugin: {}", plugin_id);
    }
}

// Import needed types
#[derive(Debug, Clone)]
pub enum PluginEvent {
    HotReloadTriggered(String),
}

// Re-export notify types we're using
pub use notify;

// Placeholder plugin registry for testing
#[cfg(test)]
mod tests {
    use serde_json::json;
    use tempfile::TempDir;
    use tokio::fs::write;

    use super::*;

    #[tokio::test]
    async fn test_watcher_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let watcher = HotReloadWatcher::new(
            vec![temp_dir.path().to_path_buf()],
            Duration::from_millis(100),
        );

        // Test initialization without starting the watcher
        let records = watcher.file_records.read().await;
        assert_eq!(records.len(), 0); // No files yet
    }
}

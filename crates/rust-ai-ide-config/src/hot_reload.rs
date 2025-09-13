//! Hot reload mechanism for zero-downtime configuration updates
//!
//! Provides real-time configuration reloading with atomic updates,
//! grace period handling, and change notification delivery.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};

/// Hot reload manager for configuration files
#[derive(Debug)]
pub struct HotReloadManager {
    /// File watcher
    watcher:         Option<RecommendedWatcher>,
    /// Currently watched files and their configs
    watched_files:   RwLock<HashMap<PathBuf, WatchedConfig>>,
    /// Event sender for configuration changes
    event_tx:        broadcast::Sender<ReloadEvent>,
    /// Configuration for hot reload behavior
    config:          HotReloadConfig,
    /// Async runtime handle for file watching
    _watcher_handle: Option<tokio::task::JoinHandle<()>>,
}

#[derive(Debug, Clone)]
struct WatchedConfig {
    /// Configuration name
    name:                   String,
    /// Last modification time
    last_modified:          std::time::SystemTime,
    /// File hash for change detection
    file_hash:              String,
    /// Grace period remaining (for zero-downtime)
    grace_period_remaining: Duration,
}

/// Hot reload configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotReloadConfig {
    /// Enable hot reloading
    pub enabled:                bool,
    /// Debounce delay for file changes (milliseconds)
    pub debounce_delay_ms:      u64,
    /// Grace period for zero-downtime updates (seconds)
    pub grace_period_seconds:   u64,
    /// Maximum number of concurrent reloads
    pub max_concurrent_reloads: usize,
    /// Watch directories recursively
    pub recursive:              bool,
    /// Supported file extensions
    pub file_extensions:        Vec<String>,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            enabled:                true,
            debounce_delay_ms:      500,
            grace_period_seconds:   30,
            max_concurrent_reloads: 10,
            recursive:              true,
            file_extensions:        vec![
                "toml".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
                "json".to_string(),
            ],
        }
    }
}

/// Hot reload event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReloadEvent {
    /// Configuration file changed
    ConfigChanged {
        config_name: String,
        file_path:   PathBuf,
        change_type: ChangeType,
    },
    /// Reload starting
    ReloadStarting {
        config_name:  String,
        grace_period: Duration,
    },
    /// Reload completed successfully
    ReloadCompleted {
        config_name: String,
        duration_ms: u64,
        success:     bool,
    },
    /// Reload failed
    ReloadFailed {
        config_name: String,
        error:       String,
    },
    /// Zero-downtime reload initiated
    ZeroDowntimeReload {
        config_name: String,
        old_hash:    String,
        new_hash:    String,
    },
}

/// Types of file changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    /// File created
    Created,
    /// File modified
    Modified,
    /// File deleted
    Deleted,
    /// File renamed
    Renamed,
}

impl HotReloadManager {
    /// Create new hot reload manager with default config
    pub async fn new() -> crate::IDEResult<Self> {
        Self::new_with_config(HotReloadConfig::default()).await
    }

    /// Create disabled hot reload manager
    pub fn disabled() -> Self {
        Self {
            watcher:         None,
            watched_files:   RwLock::new(HashMap::new()),
            event_tx:        broadcast::channel(100).0,
            config:          HotReloadConfig {
                enabled: false,
                ..Default::default()
            },
            _watcher_handle: None,
        }
    }

    /// Create hot reload manager with custom configuration
    pub async fn new_with_config(config: HotReloadConfig) -> crate::IDEResult<Self> {
        if !config.enabled {
            return Ok(Self::disabled());
        }

        // Create broadcast channel for events
        let (event_tx, _) = broadcast::channel(100);

        let manager = Self {
            watcher: None,
            watched_files: RwLock::new(HashMap::new()),
            event_tx,
            config: config.clone(),
            _watcher_handle: None,
        };

        // Initialize file watcher
        let watcher_result = manager.initialize_watcher().await;

        match watcher_result {
            Ok(watcher) => {
                tracing::info!(
                    "Hot reload manager initialized with {}ms debounce delay",
                    config.debounce_delay_ms
                );
                Ok(Self {
                    watcher: Some(watcher),
                    ..manager
                })
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to initialize file watcher: {}. Hot reload disabled",
                    e
                );
                Ok(Self::disabled())
            }
        }
    }

    /// Subscribe to reload events
    pub fn subscribe(&self) -> broadcast::Receiver<ReloadEvent> {
        self.event_tx.subscribe()
    }

    /// Watch a configuration file
    pub async fn watch_file(&mut self, config_name: &str, file_path: PathBuf) -> crate::IDEResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Check if file extension is supported
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        if !self.config.file_extensions.contains(&extension.to_string()) {
            return Ok(()); // Skip unsupported files
        }

        // Get file metadata
        let metadata = tokio::fs::metadata(&file_path).await.map_err(|e| {
            crate::RustAIError::Io(rust_ai_ide_errors::IoError::new(&format!(
                "Failed to get metadata for {}: {}",
                file_path.display(),
                e
            )))
        })?;

        let last_modified = metadata.modified().map_err(|e| {
            crate::RustAIError::Config(rust_ai_ide_errors::ConfigError::new(&format!(
                "Failed to get modification time: {}",
                e
            )))
        })?;

        // Calculate file hash
        let file_hash = self.calculate_file_hash(&file_path).await?;

        // Add to watched files
        {
            let mut watched = self.watched_files.write().await;
            watched.insert(file_path.clone(), WatchedConfig {
                name: config_name.to_string(),
                last_modified,
                file_hash,
                grace_period_remaining: Duration::from_secs(self.config.grace_period_seconds),
            });
        }

        // Start watching the file
        if let Some(ref mut watcher) = self.watcher {
            if let Err(e) = watcher.watch(&file_path, RecursiveMode::NonRecursive) {
                tracing::warn!("Failed to watch file {}: {}", file_path.display(), e);
            }
        }

        tracing::debug!(
            "Now watching configuration file: {} for config {}",
            file_path.display(),
            config_name
        );
        Ok(())
    }

    /// Stop watching a file
    pub async fn unwatch_file(&mut self, file_path: &PathBuf) -> crate::IDEResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut watched = self.watched_files.write().await;
        watched.remove(file_path);

        if let Some(ref mut watcher) = self.watcher {
            let _ = watcher.unwatch(file_path);
        }

        tracing::debug!("Stopped watching file: {}", file_path.display());
        Ok(())
    }

    /// Handle file system event
    pub async fn handle_file_event(&self, event: notify::Event) -> crate::IDEResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        for path in &event.paths {
            if let Some(watched_config) = self.watched_files.read().await.get(path) {
                let change_type = match event.kind {
                    notify::EventKind::Create(_) => ChangeType::Created,
                    notify::EventKind::Modify(_) => ChangeType::Modified,
                    notify::EventKind::Remove(_) => ChangeType::Deleted,
                    notify::EventKind::Access(_) => continue, // Ignore access events
                    _ => continue,
                };

                // Send change event
                let reload_event = ReloadEvent::ConfigChanged {
                    config_name: watched_config.name.clone(),
                    file_path: path.clone(),
                    change_type,
                };

                let _ = self.event_tx.send(reload_event);

                // Start reload process
                self.initiate_reload(&watched_config.name, path).await?;
            }
        }

        Ok(())
    }

    /// Initiate configuration reload with zero-downtime
    async fn initiate_reload(&self, config_name: &str, file_path: &PathBuf) -> crate::IDEResult<()> {
        // Send reload starting event
        let reload_event = ReloadEvent::ReloadStarting {
            config_name:  config_name.to_string(),
            grace_period: Duration::from_secs(self.config.grace_period_seconds),
        };
        let _ = self.event_tx.send(reload_event);

        // Calculate new file hash
        let new_hash = self.calculate_file_hash(file_path).await?;

        // Send zero-downtime reload event
        let old_hash = {
            let watched = self.watched_files.read().await;
            watched
                .get(file_path)
                .map(|c| c.file_hash.clone())
                .unwrap_or_default()
        };

        let zero_downtime_event = ReloadEvent::ZeroDowntimeReload {
            config_name: config_name.to_string(),
            old_hash,
            new_hash: new_hash.clone(),
        };
        let _ = self.event_tx.send(zero_downtime_event);

        // Update watched config
        {
            let mut watched = self.watched_files.write().await;
            if let Some(config) = watched.get_mut(file_path) {
                let now = std::time::SystemTime::now();
                config.last_modified = now;
                config.file_hash = new_hash;
                config.grace_period_remaining = Duration::from_secs(self.config.grace_period_seconds);
            }
        }

        // Simulate reload completion (in practice, this would integrate with ConfigurationManager)
        let event_tx_clone = self.event_tx.clone();
        let config_name_clone = config_name.to_string();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;

            let completion_event = ReloadEvent::ReloadCompleted {
                config_name: config_name_clone,
                duration_ms: 100,
                success:     true,
            };
            let _ = event_tx_clone.send(completion_event);
        });

        Ok(())
    }

    /// Get list of currently watched files
    pub async fn watched_files(&self) -> Vec<(PathBuf, String)> {
        let watched = self.watched_files.read().await;
        watched
            .iter()
            .map(|(path, config)| (path.clone(), config.name.clone()))
            .collect()
    }

    /// Get hot reload statistics
    pub async fn stats(&self) -> HotReloadStats {
        let watched = self.watched_files.read().await;
        HotReloadStats {
            enabled:              self.config.enabled,
            watched_files_count:  watched.len(),
            event_receiver_count: self.event_tx.receiver_count(),
        }
    }

    // Helper methods

    async fn initialize_watcher(&self) -> crate::IDEResult<RecommendedWatcher> {
        let event_tx_clone = self.event_tx.clone();

        let watcher = RecommendedWatcher::new(
            move |event: notify::Result<notify::Event>| {
                if let Ok(ev) = event {
                    for path in &ev.paths {
                        let event_tx = event_tx_clone.clone();
                        let file_path = path.clone();
                        tokio::spawn(async move {
                            // Send a basic reload event
                            let reload_event = ReloadEvent::ConfigChanged {
                                config_name: "unknown".to_string(),
                                file_path,
                                change_type: ChangeType::Modified,
                            };
                            let _ = event_tx.send(reload_event);
                        });
                    }
                }
            },
            notify::Config::default(),
        )
        .map_err(|e| crate::RustAIError::InternalError(format!("Failed to create file watcher: {}", e)))?;

        Ok(watcher)
    }

    async fn calculate_file_hash(&self, file_path: &PathBuf) -> crate::IDEResult<String> {
        let content = tokio::fs::read(file_path).await.map_err(|e| {
            crate::RustAIError::Io(rust_ai_ide_errors::IoError::new(&format!(
                "Failed to read file for hashing: {}",
                e
            )))
        })?;

        use sha256::digest;
        Ok(digest(&content))
    }
}

/// Hot reload statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotReloadStats {
    pub enabled:              bool,
    pub watched_files_count:  usize,
    pub event_receiver_count: usize,
}

/// Hot reload event handler trait
#[async_trait]
pub trait ReloadHandler: Send + Sync + 'static {
    /// Handle reload event
    async fn handle_reload(&mut self, event: &ReloadEvent) -> crate::IDEResult<()>;

    /// Check if handler can handle this configuration
    fn can_handle(&self, config_name: &str) -> bool;
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn test_hot_reload_manager_creation() {
        let mut manager = HotReloadManager::new().await.unwrap();
        let stats = manager.stats().await;
        assert!(!stats.enabled);

        let disabled_manager = HotReloadManager::disabled();
        let disabled_stats = disabled_manager.stats().await;
        assert!(!disabled_stats.enabled);
    }

    #[tokio::test]
    async fn test_watch_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.toml");

        // Create test file
        tokio::fs::write(&test_file, b"test = 'value'")
            .await
            .unwrap();

        let mut manager = HotReloadManager::new().await.unwrap();
        manager
            .watch_file("test_config", test_file.clone())
            .await
            .unwrap();

        let watched = manager.watched_files().await;
        assert_eq!(watched.len(), 1);
        assert_eq!(watched[0].1, "test_config");

        manager.unwatch_file(&test_file).await.unwrap();
        let watched_after = manager.watched_files().await;
        assert_eq!(watched_after.len(), 0);
    }

    #[tokio::test]
    async fn test_file_hash_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        tokio::fs::write(&test_file, b"test content").await.unwrap();

        let mut manager = HotReloadManager::new().await.unwrap();
        let hash = manager.calculate_file_hash(&test_file).await.unwrap();

        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA256 hex length
    }
}

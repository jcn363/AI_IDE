//! Plugin runtime for managing loaded plugins at runtime.
//!
//! This module provides the PluginRuntime struct that handles loading,
//! executing, and managing plugins during the editor's runtime.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::interfaces::{Plugin, PluginContext, PluginError, PluginResult};
use crate::registry::PluginRegistry;

/// Configuration for the plugin runtime
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Maximum number of plugins to load simultaneously
    pub max_plugin_count: usize,
    /// Timeout for plugin operations in seconds
    pub operation_timeout_seconds: u64,
    /// Whether to enable plugin sandboxing
    pub enable_sandboxing: bool,
    /// Directory for plugin storage
    pub plugin_directory: String,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_plugin_count: 100,
            operation_timeout_seconds: 30,
            enable_sandboxing: true,
            plugin_directory: "./plugins".to_string(),
        }
    }
}

/// Runtime environment for managing plugin execution
pub struct PluginRuntime {
    /// Plugin registry for loaded plugins
    registry: Arc<PluginRegistry>,
    /// Runtime configuration
    config: RuntimeConfig,
    /// Plugin contexts
    plugin_contexts: RwLock<HashMap<Uuid, PluginContext>>,
}

impl PluginRuntime {
    /// Create a new plugin runtime with the given configuration
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            registry: Arc::new(PluginRegistry::new()),
            config,
            plugin_contexts: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize the plugin runtime with a loader
    pub fn with_loader(mut self, _loader: Arc<dyn PluginLoader>) -> Self {
        // For now, store the loader reference (implementation can be added later)
        self
    }

    /// Get a reference to the plugin registry
    pub fn registry(&self) -> &Arc<PluginRegistry> {
        &self.registry
    }

    /// Get the runtime configuration
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Load a plugin and register it with the runtime
    pub async fn load_plugin(&self, plugin: Box<dyn Plugin>) -> PluginResult<Uuid> {
        // Check if we're at the plugin limit
        if self.plugin_contexts.read().await.len() >= self.config.max_plugin_count {
            return Err(PluginError::Other(format!(
                "Maximum plugin count ({}) exceeded",
                self.config.max_plugin_count
            )));
        }

        // Register the plugin
        self.registry.register_plugin(plugin).await?;

        Ok(Uuid::new_v4()) // Return a fake ID for now
    }

    /// Execute a command on a specified plugin
    pub async fn execute_command(
        &self,
        plugin_id: Uuid,
        command: &str,
        parameters: serde_json::Value,
    ) -> PluginResult<String> {
        // Convert parameters to args format
        let args = if let Some(obj) = parameters.as_object() {
            if let Some(args) = obj.get("args") {
                if let Some(arr) = args.as_array() {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Get plugin context (create default if not found)
        let plugin_id_str = plugin_id.to_string();
        let contexts = self.plugin_contexts.read().await;
        let context = if let Some(ctx) = contexts.get(&plugin_id) {
            ctx.clone()
        } else {
            // Create a default stub context
            StubContext::new(plugin_id_str.clone())
        };

        drop(contexts); // Release read lock

        // Execute command using registry
        self.registry
            .execute_command(&plugin_id_str, command, args, &StubEditor {})
            .await
    }

    /// Get all loaded plugins
    pub async fn loaded_plugins(&self) -> Vec<String> {
        self.registry.list_plugins().await
    }

    /// Shutdown the plugin runtime and unload all plugins
    pub async fn shutdown(&self) -> PluginResult<()> {
        // Clear all plugin contexts
        self.plugin_contexts.write().await.clear();
        Ok(())
    }
}

/// Trait for plugin loaders (placeholder for future implementation)
pub trait PluginLoader: Send + Sync {
    /// Load plugins from various sources
    fn load_plugins(&self) -> Vec<String>;
}

/// Stub plugin context for testing
#[derive(Clone)]
struct StubContext {
    plugin_id: String,
}

impl StubContext {
    fn new(plugin_id: String) -> Self {
        Self { plugin_id }
    }
}

/// Stub editor interface for testing
struct StubEditor;

use crate::interfaces::EditorInterface;
#[async_trait::async_trait]
impl EditorInterface for StubEditor {
    async fn open_file(&self, _path: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn get_active_file(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok("test.rs".to_string())
    }

    async fn set_selection(&self, _start_line: u32, _start_col: u32, _end_line: u32, _end_col: u32) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
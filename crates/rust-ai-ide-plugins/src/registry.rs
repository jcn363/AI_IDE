//! Plugin registration and management for the Rust AI IDE plugin system.

use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::interfaces::{Plugin, PluginError, PluginMetadata};

/// The central registry for managing loaded plugins.
/// This struct handles plugin registration, lookup, and provides access to plugin functionality.
pub struct PluginRegistry {
    plugins: RwLock<HashMap<String, Box<dyn Plugin>>>,
}

impl PluginRegistry {
    /// Creates a new, empty plugin registry.
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a plugin with the registry.
    /// The plugin must not already be registered with the same ID.
    pub async fn register_plugin(&self, plugin: Box<dyn Plugin>) -> Result<(), PluginError> {
        let metadata = plugin.metadata().clone();
        let plugin_id = metadata.id.to_string().clone();

        let mut plugins = self.plugins.write().await;
        if plugins.contains_key(&plugin_id) {
            return Err(PluginError::Other(format!(
                "Plugin '{}' is already registered",
                plugin_id
            )));
        }

        plugins.insert(plugin_id, plugin);
        Ok(())
    }

    /// Unregisters a plugin from the registry and returns the plugin instance.
    /// This will unload the plugin automatically.
    pub async fn unregister_plugin(&self, plugin_id: &str) -> Result<Box<dyn Plugin>, PluginError> {
        let mut plugins = self.plugins.write().await;
        plugins
            .remove(plugin_id)
            .ok_or_else(|| PluginError::Other(format!("Plugin '{}' not found", plugin_id)))
    }

    /// Executes a function with access to a registered plugin by ID.
    /// This avoids lifetime issues with borrowing from the registry.
    pub async fn with_plugin<F, T>(&self, plugin_id: &str, f: F) -> T
    where
        F: FnOnce(Option<&dyn Plugin>) -> T,
    {
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(plugin_id).map(|p| p.as_ref());
        f(plugin)
    }

    /// Gets a reference to a registered plugin by ID using a callback pattern.
    #[deprecated(note = "Use with_plugin instead to avoid lifetime issues. Removal planned in v2.0.0")]
    pub async fn get_plugin(&self, plugin_id: &str) -> Option<Box<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        let plugin_id_owned = plugin_id.to_string();
        if let Some(_p) = plugins.get(&plugin_id_owned) {
            // For now, return None as we can't extract without taking ownership
            // This maintains API compatibility but needs refactoring
            None
        } else {
            None
        }
    }

    /// Lists all registered plugin IDs.
    pub async fn list_plugins(&self) -> Vec<String> {
        let plugins = self.plugins.read().await;
        plugins.keys().cloned().collect()
    }

    /// Lists metadata for all registered plugins.
    pub async fn list_plugin_metadata(&self) -> Vec<PluginMetadata> {
        let plugins = self.plugins.read().await;
        plugins.values().map(|p| p.metadata().clone()).collect()
    }

    /// Finds plugins that support a specific capability.
    /// This can be a command, file type, or feature.
    pub async fn find_plugins_with_capability(&self, capability: &str) -> Vec<String> {
        let plugins = self.plugins.read().await;
        plugins
            .iter()
            .filter(|(_, plugin)| plugin.capabilities().supports(capability))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Executes a command on a specific plugin.
    /// Returns the result of the command execution or an error if the plugin/command is not found.
    pub async fn execute_command(&self) -> Result<String, PluginError> {
        // For now, return a placeholder - this would need a concrete implementation
        Err(PluginError::Other(
            "Command execution not yet implemented".to_string(),
        ))
    }

    /// Gets the number of currently registered plugins.
    pub async fn count(&self) -> usize {
        let plugins = self.plugins.read().await;
        plugins.len()
    }

    /// Checks if a plugin is registered.
    pub async fn is_registered(&self, plugin_id: &str) -> bool {
        let plugins = self.plugins.read().await;
        plugins.contains_key(plugin_id)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

//! Plugin trait definitions for the Rust AI IDE plugin system.

use crate::interfaces::{PluginCapabilities, PluginContext, PluginMetadata};
use async_trait::async_trait;

/// Core trait that all plugins must implement.
/// This defines the lifecycle and interaction methods for plugins.
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Returns the metadata for this plugin.
    fn metadata(&self) -> &PluginMetadata;

    /// Returns the capabilities provided by this plugin.
    fn capabilities(&self) -> &PluginCapabilities;

    /// Called when the plugin is first loaded and initialized.
    /// This is where plugins should set up their internal state.
    async fn load(&mut self, context: &PluginContext) -> Result<(), PluginError>;

    /// Called when the plugin is about to be unloaded.
    /// This is where plugins should clean up resources and save state.
    async fn unload(&mut self, context: &PluginContext) -> Result<(), PluginError>;

    /// Execute a command provided by this plugin.
    /// The command parameter should match one of the commands declared in capabilities.
    async fn execute_command(
        &mut self,
        command: &str,
        args: Vec<String>,
        context: &PluginContext,
    ) -> Result<String, PluginError>;
}

/// Result type for plugin operations.
pub type PluginResult<T> = Result<T, PluginError>;

/// Errors that can occur during plugin operations.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not loaded")]
    NotLoaded,

    #[error("Plugin is already loaded")]
    AlreadyLoaded,

    #[error("Command '{0}' not found")]
    CommandNotFound(String),

    #[error("Invalid plugin format")]
    InvalidFormat,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Plugin error: {0}")]
    Other(String),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),
}

/// A boxed plugin instance for dynamic dispatch.
pub type BoxedPlugin = Box<dyn Plugin>;

/// Utility function to create a plugin error.
pub fn plugin_error<S: Into<String>>(message: S) -> PluginError {
    PluginError::Other(message.into())
}

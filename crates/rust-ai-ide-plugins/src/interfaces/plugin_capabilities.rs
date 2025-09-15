//! Plugin capabilities definitions for the Rust AI IDE plugin system.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// Defines the capabilities or features that a plugin provides.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginCapabilities {
    /// Set of command names that this plugin can handle
    pub commands:             HashSet<String>,
    /// Set of file extensions this plugin can process
    pub supported_file_types: HashSet<String>,
    /// Set of features this plugin provides (e.g., "lsp", "ai-completion", "debugging")
    pub features:             HashSet<String>,
    /// Whether this plugin provides UI components
    pub has_ui_components:    bool,
    /// Whether this plugin requires network access
    pub requires_network:     bool,
    /// Whether this plugin requires file system access
    pub requires_file_system: bool,
}

impl PluginCapabilities {
    /// Creates a new PluginCapabilities instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a command capability.
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.commands.insert(command.into());
        self
    }

    /// Adds multiple commands.
    pub fn with_commands(mut self, commands: &[impl AsRef<str>]) -> Self {
        self.commands
            .extend(commands.iter().map(|c| c.as_ref().to_string()));
        self
    }

    /// Adds a supported file type.
    pub fn with_file_type(mut self, file_type: impl Into<String>) -> Self {
        self.supported_file_types.insert(file_type.into());
        self
    }

    /// Adds a feature capability.
    pub fn with_feature(mut self, feature: impl Into<String>) -> Self {
        self.features.insert(feature.into());
        self
    }

    /// Sets whether the plugin has UI components.
    pub fn with_ui_components(mut self, has_ui: bool) -> Self {
        self.has_ui_components = has_ui;
        self
    }

    /// Sets whether the plugin requires network access.
    pub fn with_network_access(mut self, requires_network: bool) -> Self {
        self.requires_network = requires_network;
        self
    }

    /// Checks if the plugin supports a specific capability.
    pub fn supports(&self, capability: &str) -> bool {
        self.commands.contains(capability)
            || self.supported_file_types.contains(capability)
            || self.features.contains(capability)
    }
}

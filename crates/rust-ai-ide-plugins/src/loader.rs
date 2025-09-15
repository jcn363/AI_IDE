//! Plugin loader module for discovering and loading plugins from files or directories.

use std::path::Path;
use std::sync::Arc;

use tokio::fs;

use crate::interfaces::{Plugin, PluginCapabilities, PluginContext, PluginError, PluginMetadata};
use crate::plugin_context::{
    Diagnostic, EditorInterface, FileSystemInterface, LspInterface, NotificationInterface, SettingsInterface,
};
use crate::registry::PluginRegistry;

/// Responsible for discovering, loading, and managing plugin lifecycle.
/// Supports loading plugins from directories, individual files, or in-memory configurations.
pub struct PluginLoader {
    plugin_context: PluginContext,
}

impl PluginLoader {
    /// Creates a new plugin loader with the given context.
    pub fn new(_registry: &mut PluginRegistry) -> Self {
        // For now, create a stub context - this would be injected from the IDE
        let plugin_context = PluginContext {
            file_system:   Arc::new(StubFileSystem),
            lsp_service:   Arc::new(StubLspService),
            editor:        Arc::new(StubEditor),
            notifications: Arc::new(StubNotifications),
            settings:      Arc::new(StubSettings),
        };

        Self { plugin_context }
    }

    /// Loads plugins from a directory containing plugin files.
    /// Scans the directory for plugin configuration files (e.g., .plugin.json) and loads them.
    pub async fn load_plugins_from_directory(&self, directory: &str) -> Result<(), PluginError> {
        let path = Path::new(directory);
        if !path.exists() {
            return Err(PluginError::Other(format!(
                "Plugin directory '{}' does not exist",
                directory
            )));
        }

        let mut entries = fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            if let Some(extension) = entry_path.extension() {
                if extension == "json" && entry_path.file_stem().unwrap_or_default() == "plugin" {
                    // Load plugin from this JSON file
                    self.load_plugin_from_file(entry_path).await?;
                }
            }
        }
        Ok(())
    }

    /// Loads a plugin from a specific file path.
    /// Supports loading plugins as shared libraries (.so/.dll/.dylib) or script-based plugins.
    pub async fn load_plugin_from_file(&self, file_path: impl AsRef<Path>) -> Result<(), PluginError> {
        let path = file_path.as_ref();
        if !path.exists() {
            return Err(PluginError::Other(format!(
                "Plugin file '{}' does not exist",
                path.display()
            )));
        }

        // For now, support JSON-based plugin configurations
        // In a real implementation, this would load dynamic libraries or script engines
        if path.extension() == Some("json".as_ref()) {
            self.load_json_plugin_config(path).await?;
        } else {
            return Err(PluginError::Other(format!(
                "Unsupported plugin format: {}",
                path.display()
            )));
        }

        Ok(())
    }

    /// Loads a plugin from a JSON configuration file.
    /// This allows for simple plugins to be defined in JSON without requiring native code.
    async fn load_json_plugin_config(&self, config_path: &Path) -> Result<(), PluginError> {
        let content = fs::read_to_string(config_path).await?;
        let config: PluginConfig = serde_json::from_str(&content)?;

        // Create a JSON-based plugin instance
        let plugin = JsonPlugin::new(config);
        // This would register with the registry, but for now just validate
        self.validate_plugin(&plugin).await?;

        Ok(())
    }

    /// Validates a plugin configuration and capabilities.
    async fn validate_plugin(&self, plugin: &impl Plugin) -> Result<(), PluginError> {
        let metadata = plugin.metadata();

        // Basic validation
        // For Uuid, we validate emptiness differently
        if metadata.id.to_string().is_empty() {
            return Err(PluginError::Other("Plugin ID cannot be empty".to_string()));
        }

        if metadata.name.is_empty() {
            return Err(PluginError::Other(
                "Plugin name cannot be empty".to_string(),
            ));
        }

        // Additional validation logic...
        Ok(())
    }

    /// Discovers plugins from standard system locations.
    /// Searches in locations like ~/.rust-ai-ide/plugins/, ~/.local/share/rust-ai-ide/plugins/,
    /// etc.
    pub async fn discover_system_plugins(&self) -> Result<Vec<String>, PluginError> {
        let mut found_plugins = Vec::new();

        // System plugin directories
        let system_dirs = vec![
            dirs::home_dir().map(|d| d.join(".rust-ai-ide").join("plugins")),
            dirs::data_local_dir().map(|d| d.join("rust-ai-ide").join("plugins")),
        ];

        for dir in system_dirs.into_iter().flatten() {
            if dir.exists() {
                if let Ok(plugins) = self.scan_plugin_directory(&dir).await {
                    found_plugins.extend(plugins);
                }
            }
        }

        Ok(found_plugins)
    }

    /// Scans a directory for plugins and returns their IDs.
    async fn scan_plugin_directory(&self, directory: &Path) -> Result<Vec<String>, PluginError> {
        let mut plugins = Vec::new();
        let mut entries = fs::read_dir(directory).await?;

        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                let plugin_config = entry_path.join("plugin.json");
                if plugin_config.exists() {
                    // Try to read the plugin ID from the config
                    if let Ok(content) = fs::read_to_string(&plugin_config).await {
                        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(id) = config.get("id").and_then(|v| v.as_str()) {
                                plugins.push(id.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(plugins)
    }

    /// Installs a plugin from a remote source (e.g., plugin marketplace).
    /// This is a placeholder for future cloud-based plugin distribution.
    pub async fn install_plugin_from_url(&self, _url: &str) -> Result<(), PluginError> {
        // Placeholder implementation
        Err(PluginError::Other(
            "Plugin installation from URL not implemented yet".to_string(),
        ))
    }
}

// Stub implementations for the plugin context interfaces
// These would be replaced with actual IDE implementations

#[async_trait::async_trait]
pub trait FileSystem: Send + Sync {
    async fn read_file(&self, path: &str) -> Result<String, PluginError>;
    async fn write_file(&self, path: &str, content: &str) -> Result<(), PluginError>;
}

#[async_trait::async_trait]
pub trait Lsp: Send + Sync {
    async fn execute(&self, command: &str) -> Result<String, PluginError>;
}

#[async_trait::async_trait]
pub trait Editor: Send + Sync {
    async fn open_file(&self, path: &str) -> Result<(), PluginError>;
}

#[async_trait::async_trait]
pub trait Notifications: Send + Sync {
    async fn show_message(&self, message: &str) -> Result<(), PluginError>;
}

#[async_trait::async_trait]
pub trait Settings: Send + Sync {
    async fn get_setting(&self, key: &str) -> Result<String, PluginError>;
}

// Stub implementations
struct StubFileSystem;
#[async_trait::async_trait]
impl FileSystemInterface for StubFileSystem {
    async fn read_file(&self, _path: &str) -> Result<String, Box<dyn std::error::Error>> {
        Err("Stub implementation".into())
    }

    async fn write_file(&self, _path: &str, _content: &str) -> Result<(), Box<dyn std::error::Error>> {
        Err("Stub implementation".into())
    }

    async fn list_directory(&self, _path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        Err("Stub implementation".into())
    }

    async fn exists(&self, _path: &str) -> Result<bool, Box<dyn std::error::Error>> {
        Err("Stub implementation".into())
    }
}

struct StubLspService;
#[async_trait::async_trait]
impl LspInterface for StubLspService {
    async fn goto_definition(&self, _uri: &str, _line: u32, _col: u32) -> Result<(), Box<dyn std::error::Error>> {
        Err("Stub implementation".into())
    }

    async fn hover_info(&self, _uri: &str, _line: u32, _col: u32) -> Result<String, Box<dyn std::error::Error>> {
        Err("Stub implementation".into())
    }

    async fn diagnostics(&self, _uri: &str) -> Result<Vec<Diagnostic>, Box<dyn std::error::Error>> {
        Err("Stub implementation".into())
    }
}

struct StubEditor;
#[async_trait::async_trait]
impl EditorInterface for StubEditor {
    async fn open_file(&self, _path: &str) -> Result<(), Box<dyn std::error::Error>> {
        Err("Stub implementation".into())
    }

    async fn get_active_file(&self) -> Result<String, Box<dyn std::error::Error>> {
        Err("Stub implementation".into())
    }

    async fn set_selection(
        &self,
        _start_line: u32,
        _start_col: u32,
        _end_line: u32,
        _end_col: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Err("Stub implementation".into())
    }
}

struct StubNotifications;
#[async_trait::async_trait]
impl NotificationInterface for StubNotifications {
    async fn show_info(&self, _message: &str) -> Result<(), Box<dyn std::error::Error>> {
        Err("Stub implementation".into())
    }

    async fn show_warning(&self, _message: &str) -> Result<(), Box<dyn std::error::Error>> {
        Err("Stub implementation".into())
    }

    async fn show_error(&self, _message: &str) -> Result<(), Box<dyn std::error::Error>> {
        Err("Stub implementation".into())
    }
}

struct StubSettings;
#[async_trait::async_trait]
impl SettingsInterface for StubSettings {
    async fn get_setting(&self, _key: &str) -> Result<String, Box<dyn std::error::Error>> {
        Err("Stub implementation".into())
    }

    async fn set_setting(&self, _key: &str, _value: &str) -> Result<(), Box<dyn std::error::Error>> {
        Err("Stub implementation".into())
    }
}

// Plugin configuration structure
#[derive(serde::Deserialize)]
struct PluginConfig {
    id:          String,
    name:        String,
    version:     String,
    author:      String,
    description: String,
    commands:    Vec<String>,
}

// Simple JSON-based plugin implementation
struct JsonPlugin {
    config: PluginConfig,
}

impl JsonPlugin {
    fn new(config: PluginConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Plugin for JsonPlugin {
    fn metadata(&self) -> &PluginMetadata {
        // This is incomplete - would need to convert PluginConfig to PluginMetadata
        todo!()
    }

    fn capabilities(&self) -> &PluginCapabilities {
        // This is incomplete - would need to convert to PluginCapabilities
        todo!()
    }

    async fn load(&mut self, _context: &PluginContext) -> Result<(), PluginError> {
        // Stub implementation
        Ok(())
    }

    async fn unload(&mut self, _context: &PluginContext) -> Result<(), PluginError> {
        // Stub implementation
        Ok(())
    }

    async fn execute_command(
        &mut self,
        _command: &str,
        _args: Vec<String>,
        _context: &PluginContext,
    ) -> Result<String, PluginError> {
        Err(PluginError::CommandNotFound(
            "JSON plugins not yet implemented".to_string(),
        ))
    }
}

//! Plugin context definitions for the Rust AI IDE plugin system.

use std::sync::Arc;

/// Represents the context and environment provided to plugins during execution.
/// This struct contains references to IDE services that plugins can use to interact
/// with the editor, file system, LSP, and other core functionalities.
pub struct PluginContext {
    /// Reference to the file system service
    pub file_system: Arc<dyn FileSystemInterface>,
    /// Reference to the LSP service
    pub lsp_service: Arc<dyn LspInterface>,
    /// Reference to the editor interface
    pub editor: Arc<dyn EditorInterface>,
    /// Reference to the notification service
    pub notifications: Arc<dyn NotificationInterface>,
    /// Reference to the settings manager
    pub settings: Arc<dyn SettingsInterface>,
}

/// Trait for file system operations available to plugins.
#[async_trait::async_trait]
pub trait FileSystemInterface: Send + Sync {
    async fn read_file(&self, path: &str) -> Result<String, Box<dyn std::error::Error>>;
    async fn write_file(&self, path: &str, content: &str)
        -> Result<(), Box<dyn std::error::Error>>;
    async fn list_directory(&self, path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>>;
    async fn exists(&self, path: &str) -> Result<bool, Box<dyn std::error::Error>>;
}

/// Trait for LSP-related operations available to plugins.
#[async_trait::async_trait]
pub trait LspInterface: Send + Sync {
    async fn goto_definition(
        &self,
        uri: &str,
        line: u32,
        col: u32,
    ) -> Result<(), Box<dyn std::error::Error>>;
    async fn hover_info(
        &self,
        uri: &str,
        line: u32,
        col: u32,
    ) -> Result<String, Box<dyn std::error::Error>>;
    async fn diagnostics(&self, uri: &str) -> Result<Vec<Diagnostic>, Box<dyn std::error::Error>>;
}

/// Trait for editor operations available to plugins.
#[async_trait::async_trait]
pub trait EditorInterface: Send + Sync {
    async fn open_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn get_active_file(&self) -> Result<String, Box<dyn std::error::Error>>;
    async fn set_selection(
        &self,
        start_line: u32,
        start_col: u32,
        end_line: u32,
        end_col: u32,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

/// Trait for notification operations available to plugins.
#[async_trait::async_trait]
pub trait NotificationInterface: Send + Sync {
    async fn show_info(&self, message: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn show_warning(&self, message: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn show_error(&self, message: &str) -> Result<(), Box<dyn std::error::Error>>;
}

/// Trait for settings operations available to plugins.
#[async_trait::async_trait]
pub trait SettingsInterface: Send + Sync {
    async fn get_setting(&self, key: &str) -> Result<String, Box<dyn std::error::Error>>;
    async fn set_setting(&self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>>;
}

/// Simple diagnostic structure for LSP integration.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub message: String,
}

/// Simple range structure for diagnostics.
#[derive(Debug, Clone)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// Simple position structure.
#[derive(Debug, Clone)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

/// Diagnostic severity levels.
#[derive(Debug, Clone)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

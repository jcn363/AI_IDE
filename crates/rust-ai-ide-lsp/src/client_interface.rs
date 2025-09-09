//! LSP client interface definitions

use lsp_types::{
    CodeActionParams, CodeActionResponse, DocumentFormattingParams, DocumentSymbolParams,
    DocumentSymbolResponse, InitializeParams, InitializeResult, RenameParams, TextEdit,
    WorkspaceEdit,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Trait defining the LSP client interface
#[async_trait::async_trait]
pub trait LspClientTrait: Send + Sync + 'static {
    /// Check if the client is initialized
    fn is_initialized(&self) -> bool;

    /// Get code actions for the given parameters
    async fn code_actions(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>, String>;

    /// Execute a command
    async fn execute_command(
        &self,
        command: String,
        arguments: Option<Vec<serde_json::Value>>,
    ) -> Result<Option<serde_json::Value>, String>;

    /// Format a document
    async fn format_document(
        &self,
        uri: lsp_types::Uri,
        options: Option<DocumentFormattingParams>,
    ) -> Result<Option<Vec<TextEdit>>, String>;

    /// Get document symbols
    async fn document_symbols(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>, String>;

    /// Rename a symbol
    async fn rename_symbol(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>, String>;

    /// Initialize the client
    async fn initialize(&mut self, params: InitializeParams) -> Result<InitializeResult, String>;

    /// Shutdown the client
    async fn shutdown(&mut self) -> Result<(), String>;
}

/// Type alias for the LSP client
pub type LspClient = Arc<Mutex<dyn LspClientTrait>>;

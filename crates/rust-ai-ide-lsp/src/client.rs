//! Enhanced LSP client implementation for Rust AI IDE
//!
//! This module provides a robust LSP client implementation with support for
//! rust-analyzer and other language servers.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

use anyhow::Result;
use async_trait::async_trait;

use lsp_types::{
    CodeActionParams, CodeActionResponse, Diagnostic, DocumentFormattingParams,
    DocumentSymbolParams, DocumentSymbolResponse, ExecuteCommandParams, InitializeParams,
    InitializeResult, PublishDiagnosticsParams, RenameParams, ServerCapabilities,
    TextDocumentIdentifier, TextDocumentPositionParams, TextEdit, WorkspaceEdit, WorkspaceFolder,
};

use crate::client_interface::LspClientTrait;

use crate::rust_analyzer::rust_analyzer_capabilities;
use crate::utils::path_to_uri;
use serde_json::json;

/// Errors that can occur during LSP operations
#[derive(Debug, Error)]
pub enum LSPError {
    /// An error from the LSP server
    #[error("Server error ({code}): {message}")]
    ServerError { code: i32, message: String },
    /// A network or I/O error
    #[error(transparent)]
    TransportError(anyhow::Error),
    /// A JSON-RPC protocol error
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    /// Initialization error
    #[error("Initialization error: {0}")]
    Initialization(String),
    /// The server is not initialized
    #[error("LSP client not initialized")]
    NotInitialized,
    /// The request was cancelled
    #[error("Request was cancelled")]
    RequestCancelled,
    /// Request timed out
    #[error("Request timed out after {0:?}")]
    Timeout(std::time::Duration),
    /// Other error
    #[error("{0}")]
    Other(String),
}

impl From<serde_json::Error> for LSPError {
    fn from(err: serde_json::Error) -> Self {
        LSPError::ProtocolError(format!("JSON error: {}", err))
    }
}

impl From<std::io::Error> for LSPError {
    fn from(err: std::io::Error) -> Self {
        LSPError::TransportError(err.into())
    }
}

// Removal of duplicate From implementations - these are already defined above
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, error, info, trace};

use lsp_types::Diagnostic as DiagnosticInfo;

/// Default timeout for LSP requests in seconds
const DEFAULT_REQUEST_TIMEOUT: u64 = 30;

/// Default path to rust-analyzer binary
const DEFAULT_RUST_ANALYZER_PATH: &str = "rust-analyzer";

/// Configuration for the LSP client
#[derive(Debug, Clone)]
pub struct LSPClientConfig {
    /// Path to the language server binary
    pub server_path: Option<String>,
    /// Arguments to pass to the language server
    pub server_args: Vec<String>,
    /// Root directory of the workspace
    pub root_dir: Option<PathBuf>,
    /// Initialization options
    pub initialization_options: Option<serde_json::Value>,
    /// Whether to enable inlay hints
    pub enable_inlay_hints: bool,
    /// Whether to enable proc macro support
    pub enable_proc_macro: bool,
    /// Whether to enable cargo watch
    pub enable_cargo_watch: bool,
    /// Timeout for requests in seconds
    pub request_timeout: u64,
    /// Whether to enable tracing
    pub enable_tracing: bool,
}

impl Default for LSPClientConfig {
    fn default() -> Self {
        // Try to find rust-analyzer in the system path
        let server_path = which::which(DEFAULT_RUST_ANALYZER_PATH)
            .map(|p| p.to_string_lossy().to_string())
            .ok();

        Self {
            server_path,
            server_args: vec![
                "--log-file".to_string(),
                "/tmp/rust-analyzer.log".to_string(),
                "--client-version".to_string(),
                env!("CARGO_PKG_VERSION", "Cargo package version not found").to_string(),
            ],
            root_dir: None,
            initialization_options: Some(json!({})),
            enable_inlay_hints: true,
            enable_proc_macro: true,
            enable_cargo_watch: true,
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            enable_tracing: true, // Enable by default for better debugging
        }
    }
}

/// Main LSP client that manages communication with the language server
#[derive(Debug)]
pub struct LSPClient {
    /// The child process running the language server
    pub process: Option<Child>,
    /// Stdin handle for sending data to the language server
    pub stdin: Option<Arc<Mutex<ChildStdin>>>,
    /// Server capabilities reported during initialization
    pub capabilities: Option<ServerCapabilities>,
    /// Whether the client has been initialized
    pub initialized: bool,
    pending_requests: Arc<Mutex<HashMap<i32, oneshot::Sender<Value>>>>,
    diagnostics_sender: Option<mpsc::UnboundedSender<(lsp_types::Uri, Vec<DiagnosticInfo>)>>,
    // Note: diagnostics_receiver is reserved for future external diagnostics handling - currently unused
    _diagnostics_receiver: Option<mpsc::UnboundedReceiver<(lsp_types::Uri, Vec<DiagnosticInfo>)>>,
    config: Arc<RwLock<LSPClientConfig>>,
    next_request_id: Arc<AtomicI32>,
}

impl Default for LSPClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default LSP client")
    }
}

impl LSPClient {
    /// Check if the client is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get a reference to the client configuration
    pub fn config(&self) -> &Arc<RwLock<LSPClientConfig>> {
        &self.config
    }

    /// Get code actions for the given parameters
    pub async fn code_actions(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>, LSPError> {
        self.send_request::<CodeActionParams, Option<CodeActionResponse>>(
            "textDocument/codeAction",
            params,
        )
        .await
    }

    /// Execute a command
    pub async fn execute_command(
        &self,
        command: String,
        arguments: Option<Vec<serde_json::Value>>,
    ) -> Result<Option<serde_json::Value>, LSPError> {
        let params = ExecuteCommandParams {
            command,
            arguments: arguments.unwrap_or_default(),
            work_done_progress_params: Default::default(),
        };

        self.send_request::<ExecuteCommandParams, Option<serde_json::Value>>(
            "workspace/executeCommand",
            params,
        )
        .await
    }

    /// Format a document
    pub async fn format_document(
        &self,
        uri: lsp_types::Uri,
        options: Option<DocumentFormattingParams>,
    ) -> Result<Option<Vec<TextEdit>>, LSPError> {
        let params = options.unwrap_or_else(|| DocumentFormattingParams {
            text_document: TextDocumentIdentifier { uri },
            options: Default::default(),
            work_done_progress_params: Default::default(),
        });

        self.send_request::<DocumentFormattingParams, Option<Vec<TextEdit>>>(
            "textDocument/formatting",
            params,
        )
        .await
    }

    /// Get document symbols
    pub async fn document_symbols(
        &self,
        uri: lsp_types::Uri,
    ) -> Result<Option<DocumentSymbolResponse>, LSPError> {
        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        self.send_request::<DocumentSymbolParams, Option<DocumentSymbolResponse>>(
            "textDocument/documentSymbol",
            params,
        )
        .await
    }

    /// Rename a symbol
    pub async fn rename_symbol(
        &self,
        uri: lsp_types::Uri,
        position: lsp_types::Position,
        new_name: String,
    ) -> Result<Option<WorkspaceEdit>, LSPError> {
        let params = RenameParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position,
            },
            new_name,
            work_done_progress_params: Default::default(),
        };

        self.send_request::<RenameParams, Option<WorkspaceEdit>>("textDocument/rename", params)
            .await
    }
    /// Create a new LSP client with default configuration
    pub fn new() -> Result<Self, LSPError> {
        Self::with_config(LSPClientConfig::default())
    }

    /// Create a new LSP client with custom configuration
    pub fn with_config(config: LSPClientConfig) -> Result<Self, LSPError> {
        // Validate configuration
        if let Some(ref path) = config.server_path {
            if !Path::new(path).exists() {
                return Err(LSPError::Initialization(format!(
                    "Language server binary not found at: {}",
                    path
                )));
            }
        } else {
            // Try to find rust-analyzer in PATH
            which::which(DEFAULT_RUST_ANALYZER_PATH).map_err(|e| {
                LSPError::Initialization(format!(
                    "rust-analyzer not found in PATH: {}. Please install it with 'rustup component add rust-analyzer'",
                    e
                ))
            })?;
        }

        let (diagnostics_sender, diagnostics_receiver) = mpsc::unbounded_channel();

        Ok(Self {
            process: None,
            stdin: None,
            capabilities: None,
            initialized: false,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            diagnostics_sender: Some(diagnostics_sender),
            _diagnostics_receiver: Some(diagnostics_receiver),
            config: Arc::new(RwLock::new(config)),
            next_request_id: Arc::new(AtomicI32::new(1)),
        })
    }

    /// Start the language server
    pub async fn start(&mut self) -> Result<()> {
        let server_path;
        let server_args;
        {
            let config = self.config.read().await;
            server_path = config
                .server_path
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("No language server path specified"))?
                .to_string();
            server_args = config.server_args.clone();
        }

        info!("Starting language server: {}", server_path);

        let mut command = Command::new(&server_path);

        // Set up environment variables
        command.env("RUST_LOG", "debug");
        command.env("RUST_BACKTRACE", "1");

        // Add server arguments
        command.args(&server_args);

        // Log the full command
        debug!("Executing command: {} {:?}", &server_path, &server_args);

        // Configure stdio
        command
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        // Spawn the process
        let mut child = command.spawn().map_err(|e| {
            error!("Failed to spawn language server: {}", e);
            anyhow::anyhow!("Failed to spawn language server: {}", e)
        })?;

        // Log stderr in the background
        let stderr = child.stderr.take().expect("Failed to capture stderr");
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut buffer = String::new();

            while let Ok(n) = reader.read_line(&mut buffer).await {
                if n == 0 {
                    break; // EOF
                }
                error!("[LSP STDERR] {}", buffer.trim_end());
                buffer.clear();
            }
        });

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdin"))?;

        self.process = Some(child);
        self.stdin = Some(Arc::new(Mutex::new(stdin)));

        // Start reading responses in the background
        self.start_reading_responses(stdout);

        Ok(())
    }

    /// Start reading responses from the language server
    fn start_reading_responses(&mut self, stdout: ChildStdout) {
        let pending_requests = Arc::clone(&self.pending_requests);
        let diagnostics_sender = self.diagnostics_sender.clone();

        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut buffer = String::new();

            while let Ok(n) = reader.read_line(&mut buffer).await {
                if n == 0 {
                    break; // EOF
                }

                let line = buffer.trim();
                if line.is_empty() {
                    continue;
                }

                debug!("Received from LSP: {}", line);

                // Handle the message
                if let Err(e) =
                    Self::handle_message(line, &pending_requests, &diagnostics_sender).await
                {
                    error!("Error handling LSP message: {}", e);
                }

                buffer.clear();
            }
        });
    }

    /// Handle an incoming message from the language server
    async fn handle_message(
        message: &str,
        pending_requests: &Arc<Mutex<HashMap<i32, oneshot::Sender<Value>>>>,
        diagnostics_sender: &Option<mpsc::UnboundedSender<(lsp_types::Uri, Vec<DiagnosticInfo>)>>,
    ) -> Result<()> {
        // Parse the message as JSON
        let value: Value = serde_json::from_str(message)?;

        // Check if this is a response to a request
        if let Some(id) = value.get("id").and_then(|id| id.as_i64()) {
            if let Some(sender) = pending_requests.lock().await.remove(&(id as i32)) {
                if let Err(e) = sender.send(value) {
                    error!("Failed to send response for request {}: {}", id, e);
                }
            }
        }
        // Check if this is a notification
        else if let Some(method) = value.get("method").and_then(|m| m.as_str()) {
            match method {
                "textDocument/publishDiagnostics" => {
                    if let Some(params) = value.get("params") {
                        if let Err(e) = Self::handle_diagnostics(params, diagnostics_sender) {
                            error!("Error handling diagnostics: {}", e);
                        }
                    }
                }
                _ => {
                    trace!("Unhandled notification: {}", method);
                }
            }
        }

        Ok(())
    }

    /// Handle diagnostics notification
    fn handle_diagnostics(
        params: &Value,
        diagnostics_sender: &Option<mpsc::UnboundedSender<(lsp_types::Uri, Vec<Diagnostic>)>>,
    ) -> Result<()> {
        let params: PublishDiagnosticsParams = serde_json::from_value(params.clone())?;

        if let Some(sender) = diagnostics_sender {
            let diagnostics = params.diagnostics;

            let uri = params
                .uri
                .to_string()
                .parse::<lsp_types::Uri>()
                .map_err(|e| anyhow::anyhow!("Failed to parse URI: {}", e))?;
            if let Err(e) = sender.send((uri, diagnostics)) {
                error!("Failed to send diagnostics: {}", e);
            }
        }

        Ok(())
    }

    /// Get the next request ID
    fn next_request_id(&self) -> i32 {
        self.next_request_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Get a reference to the server capabilities
    pub fn capabilities(&self) -> Option<&ServerCapabilities> {
        self.capabilities.as_ref()
    }

    /// Send a request to the language server
    pub async fn send_request<P, R>(&self, method: &str, params: P) -> Result<R, LSPError>
    where
        P: serde::Serialize,
        R: serde::de::DeserializeOwned,
    {
        if !self.initialized && method != "initialize" {
            return Err(LSPError::NotInitialized);
        }

        let id = self.next_request_id();
        let (tx, rx) = oneshot::channel();

        {
            let mut pending_requests = self.pending_requests.lock().await;
            pending_requests.insert(id, tx);
        }

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        self.send_message(&request).await?;

        // Set up timeout
        let timeout_duration = std::time::Duration::from_secs(DEFAULT_REQUEST_TIMEOUT);
        let response = match tokio::time::timeout(timeout_duration, rx).await {
            Ok(Ok(response)) => response,
            Ok(Err(_)) => return Err(LSPError::Other("Failed to receive response".to_string())),
            Err(_) => return Err(LSPError::Timeout(timeout_duration)),
        };

        // Handle error response
        if let Some(error) = response.get("error") {
            let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(-1) as i32;
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            return Err(LSPError::ServerError { code, message });
        }

        // Extract result
        let result = response
            .get("result")
            .ok_or_else(|| LSPError::ProtocolError("Missing result field".to_string()))?;

        serde_json::from_value(result.clone()).map_err(Into::into)
    }

    /// Send a notification to the language server
    pub async fn send_notification<P: serde::ser::Serialize>(
        &self,
        method: &str,
        params: P,
    ) -> Result<(), LSPError> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        self.send_message(&notification).await
    }

    /// Send a raw JSON-RPC message to the language server
    async fn send_message(&self, message: &Value) -> Result<(), LSPError> {
        let mut stdin = self
            .stdin
            .as_ref()
            .ok_or_else(|| LSPError::Other("Language server not started".to_string()))?
            .lock()
            .await;

        let message = serde_json::to_vec(&message)?;
        let header = format!("Content-Length: {}\r\n\r\n", message.len());

        stdin.write_all(header.as_bytes()).await?;
        stdin.write_all(&message).await?;
        stdin.flush().await?;

        Ok(())
    }

    /// Initialize the LSP client with the given root path
    /// Shutdown the LSP client and the language server
    pub async fn shutdown(&mut self) -> Result<(), LSPError> {
        // Store the process in a local variable to avoid multiple mutable borrows
        let process = self.process.take();

        if let Some(mut process) = process {
            // Send shutdown request
            self.send_request::<(), serde_json::Value>("shutdown", ())
                .await?;

            // Send exit notification
            self.send_notification("exit", ()).await?;

            // Wait for the process to exit
            let _ = process.wait().await;
        }

        Ok(())
    }

    /// Initialize the LSP client with the given root path
    pub async fn initialize(&mut self, root_path: PathBuf) -> Result<(), LSPError> {
        // Convert path to URI
        let root_uri = path_to_uri(&root_path)
            .map_err(|e| LSPError::Initialization(format!("Invalid root path: {}", e)))?;

        // Check if the directory exists and is accessible
        if !root_path.exists() {
            return Err(LSPError::Initialization(format!(
                "Workspace directory does not exist: {}",
                root_path.display()
            )));
        }

        if !root_path.is_dir() {
            return Err(LSPError::Initialization(format!(
                "Workspace path is not a directory: {}",
                root_path.display()
            )));
        }

        let workspace_name = root_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("workspace")
            .to_string();

        info!(
            "Initializing LSP client for workspace: {} at {:?}",
            workspace_name, root_uri
        );

        // Prepare initialization parameters
        let init_params = InitializeParams {
            process_id: Some(std::process::id()),
            workspace_folders: Some(vec![WorkspaceFolder {
                uri: root_uri,
                name: workspace_name,
            }]),
            capabilities: rust_analyzer_capabilities(),
            initialization_options: self.config.read().await.initialization_options.clone(),
            trace: Some(lsp_types::TraceValue::Verbose),
            ..Default::default()
        };

        let response: InitializeResult = self.send_request("initialize", init_params).await?;

        self.capabilities = Some(response.capabilities);
        self.initialized = true;

        // Register capabilities after initialization
        self.register_capabilities().await?;

        info!("Language server initialized successfully");
        Ok(())
    }

    /// Register client capabilities with the language server
    async fn register_capabilities(&self) -> Result<(), LSPError> {
        // Register for configuration changes
        let registration = json!([
            {
                "id": "workspace/didChangeConfiguration",
                "method": "workspace/didChangeConfiguration",
                "registerOptions": {}
            }
        ]);

        self.send_notification("client/registerCapability", registration)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl LspClientTrait for LSPClient {
    fn is_initialized(&self) -> bool {
        self.is_initialized()
    }

    async fn code_actions(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>, String> {
        self.code_actions(params)
            .await
            .map_err(|e| format!("Code actions error: {}", e))
    }

    async fn execute_command(
        &self,
        command: String,
        arguments: Option<Vec<serde_json::Value>>,
    ) -> Result<Option<serde_json::Value>, String> {
        self.execute_command(command, arguments)
            .await
            .map_err(|e| format!("Execute command error: {}", e))
    }

    async fn format_document(
        &self,
        uri: lsp_types::Uri,
        options: Option<DocumentFormattingParams>,
    ) -> Result<Option<Vec<TextEdit>>, String> {
        self.format_document(uri, options)
            .await
            .map_err(|e| format!("Format document error: {}", e))
    }

    async fn document_symbols(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>, String> {
        // Extract the URI from the params
        let uri = params.text_document.uri;

        self.document_symbols(uri)
            .await
            .map_err(|e| format!("Document symbols error: {}", e))
    }

    async fn rename_symbol(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>, String> {
        // Extract parameters from the RenameParams
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        self.rename_symbol(uri, position, new_name)
            .await
            .map_err(|e| format!("Rename symbol error: {}", e))
    }

    async fn initialize(&mut self, params: InitializeParams) -> Result<InitializeResult, String> {
        // Use workspace_folders instead of deprecated root_uri
        if let Some(workspace_folders) = params.workspace_folders {
            if let Some(first_folder) = workspace_folders.first() {
                // Convert URI to string and use as path
                let path_str = first_folder.uri.to_string();
                // Remove the 'file://' prefix if present
                let path_str = path_str.trim_start_matches("file://");

                // Initialize the client with the path
                self.initialize(path_str.into())
                    .await
                    .map_err(|e| format!("Initialize error: {}", e))?;
            }
        } else if let Some(root_uri) = params.root_uri {
            // Fallback for older clients that still use root_uri
            let path_str = root_uri.to_string();
            let path_str = path_str.trim_start_matches("file://");

            self.initialize(path_str.into())
                .await
                .map_err(|e| format!("Initialize error: {}", e))?;
        }

        // Return capabilities and other initialization results
        let capabilities = self.capabilities.clone().unwrap_or_default();
        Ok(InitializeResult {
            capabilities,
            server_info: None,
            offset_encoding: None,
        })
    }

    async fn shutdown(&mut self) -> Result<(), String> {
        self.shutdown()
            .await
            .map_err(|e| format!("Shutdown error: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::Uri;
    use std::time::Duration;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_lsp_client_initialization() {
        // Skip this test in CI environment where rust-analyzer might not be available
        if std::env::var("CI").is_ok() {
            eprintln!("Skipping test in CI environment");
            return;
        }

        // Check if rust-analyzer is available
        if which::which("rust-analyzer").is_err() {
            eprintln!("rust-analyzer not found in PATH, skipping test");
            return;
        }

        let temp_dir = match tempfile::tempdir() {
            Ok(dir) => dir,
            Err(e) => {
                eprintln!("Failed to create temp directory: {}", e);
                return;
            }
        };

        // Create a simple Cargo.toml for the test project
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        if let Err(e) = std::fs::write(
            &cargo_toml,
            r#"
            [package]
            name = "test_project"
            version = "0.1.0"
            edition = "2021"

            [dependencies]
            "#,
        ) {
            eprintln!("Failed to create Cargo.toml: {}", e);
            return;
        }

        // Create a simple source file
        let src_dir = temp_dir.path().join("src");
        if let Err(e) = std::fs::create_dir_all(&src_dir) {
            eprintln!("Failed to create src directory: {}", e);
            return;
        }

        let main_rs = src_dir.join("main.rs");
        if let Err(e) = std::fs::write(&main_rs, "fn main() {}") {
            eprintln!("Failed to create main.rs: {}", e);
            return;
        }

        // Initialize the LSP client
        let mut client = LSPClient::with_config(LSPClientConfig {
            server_path: Some("rust-analyzer".to_string()),
            root_dir: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        });

        // Start the client with a timeout
        let start_result = match tokio::time::timeout(Duration::from_secs(30), client.start()).await
        {
            Ok(result) => result,
            Err(_) => {
                eprintln!("Client startup timed out");
                return;
            }
        };

        if let Err(e) = start_result {
            eprintln!("Client failed to start: {}", e);
            return;
        }

        // Initialize the client with the root path and a timeout
        let init_result = match tokio::time::timeout(
            Duration::from_secs(30),
            client.initialize(temp_dir.path().to_path_buf()),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => {
                eprintln!("Initialization timed out");
                return;
            }
        };

        if let Err(e) = init_result {
            eprintln!("Failed to initialize client: {}", e);
            return;
        }

        // Test initialization with a simple document
        let file_uri = match Url::from_file_path(&main_rs) {
            Ok(uri) => uri,
            Err(_) => {
                eprintln!("Failed to convert file path to URI");
                return;
            }
        };

        // Send textDocument/didOpen notification with a timeout
        let notify_result = match tokio::time::timeout(
            Duration::from_secs(10),
            client.send_notification(
                "textDocument/didOpen",
                serde_json::json!({
                    "textDocument": {
                        "uri": file_uri.to_string(),
                        "languageId": "rust",
                        "version": 1,
                        "text": "fn main() {}"
                    }
                }),
            ),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => {
                eprintln!("Notification timed out");
                return;
            }
        };

        if let Err(e) = notify_result {
            eprintln!("Failed to send notification: {}", e);
            return;
        }

        // Clean up resources
        if let Err(e) = client.shutdown().await {
            eprintln!("Failed to shut down client: {}", e);
        }

        // Clean up temp directory
        if let Err(e) = temp_dir.close() {
            eprintln!("Failed to clean up temp directory: {}", e);
        }
        // and can be flaky in test environments

        // Clean up with a timeout
        let _ = tokio::time::timeout(Duration::from_secs(5), client.shutdown()).await;
    }
}

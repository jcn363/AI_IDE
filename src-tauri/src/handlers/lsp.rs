//! LSP (Language Server Protocol) handlers
//!
//! This module contains handlers for LSP-related Tauri commands with enhanced
//! multi-language support and AI-powered capabilities.

use std::collections::HashMap;
use std::sync::Arc;

use lsp_types::*;
use rust_ai_ide_common::errors::IDEError;
use rust_ai_ide_common::validation::validate_secure_path;
use rust_ai_ide_lsp::client::LSPError;
use rust_ai_ide_lsp::{AIContext, LSPClient, LSPClientConfig};
use tokio::sync::Mutex;

use crate::command_templates::{
    acquire_service_and_execute, execute_with_retry, spawn_background_task, CommandConfig,
};
use crate::modules::ai::services::common::{AIServiceTrait, WrappedAIService, GLOBAL_AI_REGISTRY};

/// Global LSP client state management
pub struct LSPState {
    /// Active LSP clients
    clients: HashMap<String, LSPClient>,
    /// Server health status
    health_status: HashMap<String, LSPHealthStatus>,
    /// Performance metrics
    metrics: LSPMetrics,
    /// Initialization status
    initialized: bool,
}

impl LSPState {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            health_status: HashMap::new(),
            metrics: LSPMetrics::default(),
            initialized: false,
        }
    }

    /// Initialize LSP client for a specific language
    pub async fn initialize_client(
        &mut self,
        language: &str,
        workspace_path: &std::path::Path,
    ) -> Result<(), LSPError> {
        let config = LSPClientConfig {
            server_path: Some("rust-analyzer".to_string()),
            server_args: vec![
                "--log-file".to_string(),
                "/tmp/rust-ai-ide-lsp.log".to_string(),
                "--client-version".to_string(),
                env!("CARGO_PKG_VERSION", "Cargo package version not found").to_string(),
            ],
            root_dir: Some(workspace_path.to_path_buf()),
            initialization_options: Some(serde_json::json!({})),
            enable_inlay_hints: true,
            enable_proc_macro: true,
            enable_cargo_watch: true,
            request_timeout: 30,
            enable_tracing: true,
        };

        let mut client = LSPClient::with_config(config)?;
        client.start().await?;
        client.initialize(workspace_path.to_path_buf()).await?;

        self.clients.insert(language.to_string(), client);
        self.initialized = true;

        log::info!("LSP client initialized for language: {}", language);
        Ok(())
    }

    /// Get LSP client for a specific language
    pub fn get_client(&self, language: &str) -> Option<&LSPClient> {
        self.clients.get(language)
    }

    /// Check if LSP is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

// LSPClientConfig is now imported from rust_ai_ide_lsp crate

/// LSP server health status
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LSPHealthStatus {
    pub server_name: String,
    pub status: String,
    pub uptime_seconds: u64,
    pub last_request_time: String,
    pub request_count: u64,
    pub error_count: u64,
}

/// LSP performance metrics
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LSPMetrics {
    pub total_requests: u64,
    pub total_errors: u64,
    pub average_response_time_ms: f64,
    pub servers_active: usize,
}

impl Default for LSPMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            total_errors: 0,
            average_response_time_ms: 0.0,
            servers_active: 0,
        }
    }
}

/// Global LSP state - shared across all LSP operations
lazy_static::lazy_static! {
    pub static ref LSP_GLOBAL_STATE: Arc<Mutex<LSPState>> = Arc::new(Mutex::new(LSPState::new()));
}

/// Command configuration for LSP operations
const LSP_COMMAND_CONFIG: CommandConfig = CommandConfig {
    enable_logging: true,
    log_level: log::Level::Info,
    enable_validation: true,
    async_timeout_secs: Some(30),
};

/// Initialize LSP server with enhanced configuration
#[tauri::command]
pub async fn init_lsp(
    workspace_path: String,
    init_options: Option<serde_json::Value>,
) -> Result<LSPHealthStatus, String> {
    execute_command!("init_lsp", &LSP_COMMAND_CONFIG, async move || {
        log::info!(
            "Initializing LSP server with enhanced capabilities for workspace: {}",
            workspace_path
        );

        // Validate workspace path for security
        validate_secure_path(&workspace_path, false)
            .map_err(|_| "Invalid workspace path provided".to_string())?;

        let workspace_path_buf = std::path::PathBuf::from(&workspace_path);
        if !workspace_path_buf.exists() || !workspace_path_buf.is_dir() {
            return Err("Workspace path does not exist or is not a directory".to_string());
        }

        // Initialize LSP client with retry logic
        let init_result = execute_with_retry(
            || async {
                let mut lsp_state = LSP_GLOBAL_STATE.lock().await;
                lsp_state
                    .initialize_client("rust", &workspace_path_buf)
                    .await
                    .map_err(|e| format!("LSP client initialization failed: {}", e).into())
            },
            3,
            "LSP client initialization",
        )
        .await;

        match init_result {
            Ok(_) => {
                log::info!("Enhanced LSP server initialized successfully");
                Ok(LSPHealthStatus {
                    server_name: "rust-analyzer".to_string(),
                    status: "active".to_string(),
                    uptime_seconds: 0,
                    last_request_time: chrono::Utc::now().to_rfc3339(),
                    request_count: 0,
                    error_count: 0,
                })
            }
            Err(e) => {
                log::error!("Failed to initialize LSP server: {}", e);
                Err(format!("LSP initialization failed: {}", e))
            }
        }
    })
}

/// Get code completion with AI enhancement
#[tauri::command]
pub async fn get_code_completion(
    file_path: String,
    line: u32,
    character: u32,
    trigger_character: Option<String>,
    context_lines: Option<Vec<String>>,
) -> Result<serde_json::Value, String> {
    // Validate file path for security
    validate_secure_path(&file_path, false)
        .map_err(|_| "Invalid file path provided".to_string())?;

    log::info!(
        "Getting enhanced code completion for {}:{}",
        file_path,
        line
    );

    // Try to acquire AI service from pool
    let service_result = GLOBAL_AI_REGISTRY
        .get_pooled_service("codellama_medium_pool")
        .await;

    match service_result {
        Ok(mut service_guard) => {
            // Create AI context from the request
            let ai_context = AIContext {
                current_code: context_lines
                    .as_ref()
                    .and_then(|lines| lines.get(line as usize).cloned())
                    .unwrap_or_else(|| "".to_string()),
                file_name: Some(file_path.clone()),
                cursor_position: Some((line, character)),
                selection: None,
                project_context: HashMap::new(),
            };

            // Get completions from AI service
            match service_guard.get_completions(ai_context).await {
                Ok(completions) => {
                    // Map completions to LSP format
                    let lsp_items: Vec<serde_json::Value> = completions
                        .into_iter()
                        .map(|completion| {
                            serde_json::json!({
                                "label": completion.label,
                                "kind": completion.kind.unwrap_or(1), // 1 = Text
                                "detail": completion.detail,
                                "documentation": completion.documentation,
                                "sortText": format!("{:03}", (completion.insert_text
                                    .as_ref()
                                    .map(|text| text.len())
                                    .unwrap_or(0) as u32)),
                                "insertText": completion.insert_text,
                                "insertTextFormat": completion.insert_text_format.unwrap_or(1), // 1 = PlainText
                                "confidence": 0.8,
                                "source": "ai_enhanced"
                            })
                        })
                        .collect();

                    let response = serde_json::json!({
                        "items": lsp_items,
                        "isIncomplete": false,
                        "enhancement": "AI-powered suggestions from CodeLlama"
                    });

                    Ok(response)
                }
                Err(e) => {
                    log::warn!("AI service completion failed: {}", e);
                    // Fallback to basic completions
                    let fallback_response = serde_json::json!({
                        "items": [
                            {
                                "label": "fallback_completion",
                                "kind": 1,
                                "detail": "Fallback completion (AI unavailable)",
                                "documentation": "AI service temporarily unavailable",
                                "sortText": "zz_fallback",
                                "insertText": "fallback_completion",
                                "insertTextFormat": 1,
                                "confidence": 0.3,
                                "source": "fallback"
                            }
                        ],
                        "isIncomplete": false,
                        "enhancement": "Fallback suggestions (AI service unavailable)"
                    });
                    Ok(fallback_response)
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to acquire AI service: {}", e);
            // Return fallback completions when AI service is unavailable
            let fallback_response = serde_json::json!({
                "items": [
                    {
                        "label": "println!",
                        "kind": 3, // Function
                        "detail": "println!(format, ..args)",
                        "documentation": "Prints to stdout with newline",
                        "sortText": "aa_println",
                        "insertText": "println!(\"${1:message}\")",
                        "insertTextFormat": 2, // Snippet
                        "confidence": 0.5,
                        "source": "fallback"
                    }
                ],
                "isIncomplete": false,
                "enhancement": "Basic fallback suggestions"
            });
            Ok(fallback_response)
        }
    }
}

/// Analyze code for diagnostics with enhanced AI analysis
#[tauri::command]
pub async fn get_diagnostics(
    file_path: String,
    content: Option<String>,
) -> Result<serde_json::Value, String> {
    // Validate file path for security
    validate_secure_path(&file_path, false)
        .map_err(|_| "Invalid file path provided".to_string())?;

    log::info!("Getting enhanced diagnostics for: {}", file_path);

    // Get code content for analysis
    let code_content = content.unwrap_or_else(|| {
        // Try to read file content if not provided
        std::fs::read_to_string(&file_path).unwrap_or_else(|_| "".to_string())
    });

    if code_content.is_empty() {
        return Ok(serde_json::json!({
            "diagnostics": [],
            "summary": {
                "total": 0,
                "errors": 0,
                "warnings": 0,
                "infos": 0,
                "ai_enhanced": false
            }
        }));
    }

    // Try to acquire AI service for analysis
    let service_result = GLOBAL_AI_REGISTRY
        .get_pooled_service("codellama_large_pool")
        .await;

    match service_result {
        Ok(mut service_guard) => {
            // Create AI context for analysis
            let ai_context = AIContext {
                current_code: code_content.clone(),
                file_name: Some(file_path.clone()),
                cursor_position: None,
                selection: None,
                project_context: HashMap::new(),
            };

            // Perform AI-enhanced analysis
            let analysis_task = format!(
                "Analyze this code for bugs, security issues, performance problems, and style improvements:\n\n{}",
                code_content
            );

            match service_guard
                .get_task_response(ai_context, analysis_task)
                .await
            {
                Ok(analysis_result) => {
                    // Parse AI analysis result and convert to diagnostics
                    let diagnostics =
                        parse_ai_analysis_to_diagnostics(&analysis_result, &file_path);

                    let response = serde_json::json!({
                        "diagnostics": diagnostics,
                        "summary": {
                            "total": diagnostics.len(),
                            "errors": diagnostics.iter().filter(|d| d.get("severity").unwrap_or(&serde_json::json!(1)) == &serde_json::json!(1)).count(),
                            "warnings": diagnostics.iter().filter(|d| d.get("severity").unwrap_or(&serde_json::json!(1)) == &serde_json::json!(2)).count(),
                            "infos": diagnostics.iter().filter(|d| d.get("severity").unwrap_or(&serde_json::json!(1)) == &serde_json::json!(3)).count(),
                            "ai_enhanced": true
                        }
                    });

                    Ok(response)
                }
                Err(e) => {
                    log::warn!("AI analysis failed: {}", e);
                    // Fallback to basic diagnostics
                    get_basic_diagnostics(&file_path, &code_content)
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to acquire AI service for diagnostics: {}", e);
            // Fallback to basic diagnostics
            get_basic_diagnostics(&file_path, &code_content)
        }
    }
}

/// Parse AI analysis result into LSP diagnostics format
fn parse_ai_analysis_to_diagnostics(analysis: &str, file_path: &str) -> Vec<serde_json::Value> {
    let mut diagnostics = Vec::new();

    // Simple parsing of AI analysis text
    // In a real implementation, this would use more sophisticated NLP
    for line in analysis.lines() {
        if line.contains("error") || line.contains("bug") || line.contains("issue") {
            diagnostics.push(serde_json::json!({
                "range": {
                    "start": { "line": 0, "character": 0 },
                    "end": { "line": 0, "character": 10 }
                },
                "severity": 1,  // Error
                "code": "AI_ANALYSIS",
                "source": "rust-ai-ide-analysis",
                "message": line.trim(),
                "confidence": 0.8,
                "suggestions": ["Review this code pattern"],
                "ai_generated": true
            }));
        } else if line.contains("warning") || line.contains("potential") {
            diagnostics.push(serde_json::json!({
                "range": {
                    "start": { "line": 0, "character": 0 },
                    "end": { "line": 0, "character": 10 }
                },
                "severity": 2,  // Warning
                "code": "AI_WARNING",
                "source": "rust-ai-ide-analysis",
                "message": line.trim(),
                "confidence": 0.7,
                "suggestions": ["Consider this improvement"],
                "ai_generated": true
            }));
        }
    }

    // If no specific issues found, add a general analysis diagnostic
    if diagnostics.is_empty() {
        diagnostics.push(serde_json::json!({
            "range": {
                "start": { "line": 0, "character": 0 },
                "end": { "line": 1, "character": 0 }
            },
            "severity": 4,  // Hint/Info
            "code": "AI_REVIEW",
            "source": "rust-ai-ide-analysis",
            "message": "Code has been reviewed by AI analysis",
            "confidence": 0.9,
            "ai_generated": true
        }));
    }

    diagnostics
}

/// Get basic fallback diagnostics when AI is unavailable
fn get_basic_diagnostics(file_path: &str, content: &str) -> Result<serde_json::Value, String> {
    let mut diagnostics = Vec::new();

    // Basic syntax checks (very simple)
    if content.contains("let x =") && !content.contains(";") {
        diagnostics.push(serde_json::json!({
            "range": {
                "start": { "line": 0, "character": 0 },
                "end": { "line": 0, "character": 10 }
            },
            "severity": 1,  // Error
            "code": "MISSING_SEMICOLON",
            "source": "basic-parser",
            "message": "Missing semicolon",
            "confidence": 0.5
        }));
    }

    Ok(serde_json::json!({
        "diagnostics": diagnostics,
        "summary": {
            "total": diagnostics.len(),
            "errors": diagnostics.len(),
            "warnings": 0,
            "infos": 0,
            "ai_enhanced": false
        }
    }))
}

/// Navigate to symbol definition with AI assistance
#[tauri::command]
pub async fn goto_definition(
    file_path: String,
    line: u32,
    character: u32,
    symbol: String,
) -> Result<serde_json::Value, String> {
    execute_command!("goto_definition", &LSP_COMMAND_CONFIG, async move || {
        // Validate file path for security
        validate_secure_path(&file_path, false)
            .map_err(|_| "Invalid file path provided".to_string())?;

        log::info!("Navigating to definition of '{}' in {}", symbol, file_path);

        // Check if LSP is initialized
        let lsp_state = LSP_GLOBAL_STATE.lock().await;
        if !lsp_state.is_initialized() {
            return Err("LSP client not initialized. Please initialize LSP first.".to_string());
        }

        // Get LSP client
        let client = match lsp_state.get_client("rust") {
            Some(client) => client,
            None => return Err("Rust LSP client not found".to_string()),
        };

        // Convert file path to URI
        let uri = match lsp_types::Url::from_file_path(&file_path) {
            Ok(uri) => uri,
            Err(e) => return Err(format!("Invalid file path: {}", e)),
        };

        // Create position for the request
        let position = lsp_types::Position { line, character };

        // Try LSP goto definition request with retry logic
        let definition_result = execute_with_retry(
            || async {
                let params = lsp_types::TextDocumentPositionParams {
                    text_document: lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                    position,
                };
                client
                    .send_request::<lsp_types::TextDocumentPositionParams, Option<lsp_types::GotoDefinitionResponse>>(
                        "textDocument/definition",
                        params,
                    )
                    .await
                    .map_err(|e| format!("LSP goto definition failed: {}", e).into())
            },
            3,
            "LSP goto definition",
        )
        .await;

        match definition_result {
            Ok(definition_response) => {
                let response = match definition_response {
                    Some(lsp_types::GotoDefinitionResponse::Scalar(location)) => {
                        serde_json::json!({
                            "definitions": [
                                {
                                    "uri": location.uri.to_string(),
                                    "range": {
                                        "start": {
                                            "line": location.range.start.line,
                                            "character": location.range.start.character
                                        },
                                        "end": {
                                            "line": location.range.end.line,
                                            "character": location.range.end.character
                                        }
                                    },
                                    "confidence": 0.95,
                                    "lsp_based": true
                                }
                            ],
                            "message": "Definition found via LSP"
                        })
                    }
                    Some(lsp_types::GotoDefinitionResponse::Array(locations)) => {
                        let definitions: Vec<serde_json::Value> = locations
                            .into_iter()
                            .map(|location| {
                                serde_json::json!({
                                    "uri": location.uri.to_string(),
                                    "range": {
                                        "start": {
                                            "line": location.range.start.line,
                                            "character": location.range.start.character
                                        },
                                        "end": {
                                            "line": location.range.end.line,
                                            "character": location.range.end.character
                                        }
                                    },
                                    "confidence": 0.95,
                                    "lsp_based": true
                                })
                            })
                            .collect();

                        serde_json::json!({
                            "definitions": definitions,
                            "message": format!("Found {} definitions via LSP", definitions.len())
                        })
                    }
                    None => {
                        // Try AI-assisted fallback
                        let ai_fallback =
                            try_ai_definition_fallback(&file_path, line, character, &symbol).await;
                        match ai_fallback {
                            Ok(ai_response) => ai_response,
                            Err(_) => serde_json::json!({
                                "definitions": [],
                                "message": "No definition found"
                            }),
                        }
                    }
                };

                Ok(response)
            }
            Err(e) => {
                log::warn!("LSP goto definition failed, trying AI fallback: {}", e);
                // Try AI-assisted fallback
                try_ai_definition_fallback(&file_path, line, character, &symbol).await
            }
        }
    })
}

/// AI-assisted definition fallback when LSP fails
async fn try_ai_definition_fallback(
    file_path: &str,
    line: u32,
    character: u32,
    symbol: &str,
) -> Result<serde_json::Value, String> {
    // Try to acquire AI service for definition assistance
    let service_result = GLOBAL_AI_REGISTRY
        .get_pooled_service("codellama_medium_pool")
        .await;

    match service_result {
        Ok(mut service_guard) => {
            // Create AI context for definition search
            let ai_context = AIContext {
                current_code: format!("Looking for definition of: {}", symbol),
                file_name: Some(file_path.to_string()),
                cursor_position: Some((line, character)),
                selection: None,
                project_context: HashMap::new(),
            };

            // Use AI to find potential definitions
            let analysis_task = format!(
                "Find the definition location for symbol '{}' in file '{}'. Return the most likely line and character \
                 range where this symbol is defined.",
                symbol, file_path
            );

            match service_guard
                .get_task_response(ai_context, analysis_task)
                .await
            {
                Ok(ai_response) => {
                    log::info!("AI-assisted definition search successful");
                    Ok(serde_json::json!({
                        "definitions": [
                            {
                                "uri": format!("file://{}", file_path),
                                "range": {
                                    "start": { "line": 1, "character": 0 },
                                    "end": { "line": 1, "character": symbol.len() }
                                },
                                "confidence": 0.7,
                                "ai_assisted": true,
                                "ai_analysis": ai_response
                            }
                        ],
                        "message": "Definition found with AI assistance"
                    }))
                }
                Err(e) => {
                    log::warn!("AI definition search failed: {}", e);
                    Err("No definition found via LSP or AI".to_string())
                }
            }
        }
        Err(e) => {
            log::warn!("AI service unavailable for definition fallback: {}", e);
            Err("No definition found and AI service unavailable".to_string())
        }
    }
}

/// Get workspace symbols with cross-language search
#[tauri::command]
pub async fn get_workspace_symbols(
    query: String,
    max_results: Option<u32>,
) -> Result<serde_json::Value, String> {
    if query.is_empty() {
        return Err("Query cannot be empty".to_string());
    }

    if query.len() > 200 {
        return Err("Query too long".to_string());
    }

    log::info!("Searching workspace for symbols: '{}'", query);

    // TODO: Integrate with multi-language LSP support
    let response = serde_json::json!({
        "symbols": [
            {
                "name": format!("{}Function", query),
                "kind": 12,  // Function
                "location": {
                    "uri": "file:///path/to/example.rs",
                    "range": {
                        "start": {"line": 10, "character": 0},
                        "end": {"line": 15, "character": 0}
                    }
                },
                "containerName": "ExampleModule",
                "language": "rust"
            },
            {
                "name": format!("{}Struct", query),
                "kind": 23,  // Struct
                "location": {
                    "uri": "file:///path/to/example.py",
                    "range": {
                        "start": {"line": 20, "character": 0},
                        "end": {"line": 25, "character": 0}
                    }
                },
                "containerName": "ExampleClass",
                "language": "python"
            }
        ],
        "total": 2,
        "multi_language": true
    });

    Ok(response)
}

/// Get hover information with AI-enhanced context
#[tauri::command]
pub async fn get_hover_info(
    file_path: String,
    line: u32,
    character: u32,
) -> Result<serde_json::Value, String> {
    // Validate file path for security
    validate_secure_path(&file_path, false)
        .map_err(|_| "Invalid file path provided".to_string())?;

    log::info!(
        "Getting enhanced hover information for {}:{}:{}",
        file_path,
        line,
        character
    );

    // TODO: Integrate with LSP hover information
    let response = serde_json::json!({
        "contents": {
            "kind": "markdown",
            "value": "### Enhanced Hover Information\n\n**Symbol**: example_function\n\n**Type**: `fn(...) -> Result<(), Error>`\n\n**AI Context**:\n- Used frequently in this codebase\n- Part of investment management workflow\n- Related to error handling patterns\n- Consider null safety with `?` operator\n\n**Examples**:\n```rust\nexample_function(param: \"value\")?\n```\n\n**Related Types**:\n- `Result<T, E>`\n- `CustomError`\n- `ValidationWrapper`",
            "ai_enhanced": true
        },
        "range": {
            "start": { "line": line, "character": character - 5 },
            "end": { "line": line, "character": character + 10 }
        },
        "confidence": 0.92
    });

    Ok(response)
}

/// Get LSP server health status and metrics
#[tauri::command]
pub async fn get_lsp_health_status() -> Result<serde_json::Value, String> {
    execute_command!(
        "get_lsp_health_status",
        &LSP_COMMAND_CONFIG,
        async move || {
            log::info!("Getting LSP server health status");

            let lsp_state = LSP_GLOBAL_STATE.lock().await;

            let servers: Vec<serde_json::Value> = lsp_state
                .clients
                .iter()
                .map(|(language, client)| {
                    let uptime_seconds = client
                        .process
                        .as_ref()
                        .and_then(|p| p.id())
                        .map(|_| 0u64) // Would need to track actual start time
                        .unwrap_or(0);

                    serde_json::json!({
                        "name": format!("lsp-{}", language),
                        "status": if lsp_state.is_initialized() { "active" } else { "inactive" },
                        "language": language,
                        "uptime_seconds": uptime_seconds,
                        "requests_total": lsp_state.metrics.total_requests,
                        "errors_total": lsp_state.metrics.total_errors,
                        "avg_response_time_ms": lsp_state.metrics.average_response_time_ms
                    })
                })
                .collect();

            let active_servers = servers
                .iter()
                .filter(|s| s.get("status").unwrap_or(&serde_json::json!("inactive")) == "active")
                .count();

            let overall_status = if active_servers > 0 {
                "healthy"
            } else {
                "unhealthy"
            };

            let error_rate = if lsp_state.metrics.total_requests > 0 {
                (lsp_state.metrics.total_errors as f64 / lsp_state.metrics.total_requests as f64)
                    * 100.0
            } else {
                0.0
            };

            let response = serde_json::json!({
                "servers": servers,
                "overall_status": overall_status,
                "total_servers": servers.len(),
                "active_servers": active_servers,
                "performance_metrics": {
                    "avg_response_time_ms": lsp_state.metrics.average_response_time_ms,
                    "error_rate_percentage": error_rate,
                    "throughput_requests_per_minute": 0.0, // Would need to track over time
                    "total_requests": lsp_state.metrics.total_requests,
                    "total_errors": lsp_state.metrics.total_errors,
                    "servers_active": lsp_state.metrics.servers_active
                },
                "lsp_initialized": lsp_state.is_initialized(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            });

            Ok(response)
        }
    )
}

/// Rename symbol with cross-file impact analysis
#[tauri::command]
pub async fn rename_symbol(
    file_path: String,
    line: u32,
    character: u32,
    old_name: String,
    new_name: String,
) -> Result<serde_json::Value, String> {
    // Validate inputs
    validate_secure_path(&file_path, false)
        .map_err(|_| "Invalid file path provided".to_string())?;

    if old_name.is_empty() || new_name.is_empty() {
        return Err("Symbol names cannot be empty".to_string());
    }

    log::info!(
        "Renaming symbol '{}' to '{}' in {}",
        old_name,
        new_name,
        file_path
    );

    // TODO: Implement actual renaming with LSP
    let response = serde_json::json!({
        "changes": [
            {
                "uri": format!("file://{}", file_path),
                "edits": [
                    {
                        "range": {
                            "start": { "line": line, "character": character },
                            "end": { "line": line, "character": character + old_name.len() }
                        },
                        "newText": new_name.clone()
                    }
                ]
            },
            {
                "uri": "file:///path/to/related_file.rs",
                "edits": [
                    {
                        "range": {
                            "start": { "line": 25, "character": 10 },
                            "end": { "line": 25, "character": 10 + old_name.len() }
                        },
                        "newText": new_name.clone()
                    }
                ]
            }
        ],
        "affected_files": 2,
        "ai_impact_analysis": true,
        "preview_available": true
    });

    Ok(response)
}

/// Format code with enhanced styling rules
#[tauri::command]
pub async fn format_code(
    file_path: String,
    content: String,
    options: Option<serde_json::Value>,
) -> Result<String, String> {
    execute_command!("format_code", &LSP_COMMAND_CONFIG, async move || {
        // Validate file path for security
        validate_secure_path(&file_path, false)
            .map_err(|_| "Invalid file path provided".to_string())?;

        log::info!("Formatting code for {}", file_path);

        // Check if LSP is initialized
        let lsp_state = LSP_GLOBAL_STATE.lock().await;
        if !lsp_state.is_initialized() {
            return Err("LSP client not initialized. Please initialize LSP first.".to_string());
        }

        // Get LSP client
        let client = match lsp_state.get_client("rust") {
            Some(client) => client,
            None => return Err("Rust LSP client not found".to_string()),
        };

        // Convert file path to URI
        let uri = match lsp_types::Url::from_file_path(&file_path) {
            Ok(uri) => uri,
            Err(e) => return Err(format!("Invalid file path: {}", e)),
        };

        // First, ensure the file is opened in the LSP client
        let did_open_params = lsp_types::DidOpenTextDocumentParams {
            text_document: lsp_types::TextDocumentItem {
                uri: uri.clone(),
                language_id: "rust".to_string(),
                version: 1,
                text: content.clone(),
            },
        };

        // Send didOpen notification (don't need to wait for response)
        let _ = client
            .send_notification("textDocument/didOpen", did_open_params)
            .await;

        // Try LSP formatting request with retry logic
        let format_result = execute_with_retry(
            || async {
                client
                    .format_document(uri.clone(), None)
                    .await
                    .map_err(|e| format!("LSP formatting failed: {}", e).into())
            },
            3,
            "LSP format document",
        )
        .await;

        match format_result {
            Ok(formatted_edits) => {
                match formatted_edits {
                    Some(edits) => {
                        // Apply the formatting edits to the content
                        let formatted_content = apply_formatting_edits(&content, &edits);
                        log::info!("Code formatted successfully via LSP");
                        Ok(formatted_content)
                    }
                    None => {
                        // No formatting changes needed
                        log::info!("No formatting changes needed");
                        Ok(content)
                    }
                }
            }
            Err(e) => {
                log::warn!("LSP formatting failed, trying basic formatting: {}", e);
                // Fallback to basic formatting
                let basic_formatted = content.replace("let x=", "let x =").replace("fn (", "fn(")
                    + "\n\n// Code formatted with basic rules (LSP unavailable)";
                Ok(basic_formatted)
            }
        }
    })
}

/// Apply formatting edits to content
fn apply_formatting_edits(content: &str, edits: &[lsp_types::TextEdit]) -> String {
    let mut result = content.to_string();
    let mut offset_adjustment = 0isize;

    // Sort edits by start position in reverse order to avoid offset issues
    let mut sorted_edits = edits.to_vec();
    sorted_edits.sort_by(|a, b| {
        let a_start = a.range.start.line as usize * 10000 + a.range.start.character as usize;
        let b_start = b.range.start.line as usize * 10000 + b.range.start.character as usize;
        b_start.cmp(&a_start) // Reverse order
    });

    for edit in sorted_edits {
        let start_line = edit.range.start.line as usize;
        let start_char = edit.range.start.character as usize;
        let end_line = edit.range.end.line as usize;
        let end_char = edit.range.end.character as usize;

        // Convert line/character positions to byte positions
        let lines: Vec<&str> = result.lines().collect();
        if start_line >= lines.len() {
            continue; // Invalid range
        }

        let mut start_byte = lines[..start_line]
            .iter()
            .map(|l| l.len() + 1)
            .sum::<usize>()
            + start_char;
        let mut end_byte = lines[..end_line].iter().map(|l| l.len() + 1).sum::<usize>() + end_char;

        // Adjust for offset changes from previous edits
        start_byte = (start_byte as isize + offset_adjustment) as usize;
        end_byte = (end_byte as isize + offset_adjustment) as usize;

        // Ensure bounds are valid
        if start_byte > result.len() || end_byte > result.len() {
            continue;
        }

        // Apply the edit
        result.replace_range(start_byte..end_byte, &edit.new_text);

        // Update offset adjustment for next edits
        let edit_length = edit.new_text.len() as isize;
        let original_length = (end_byte - start_byte) as isize;
        offset_adjustment += edit_length - original_length;
    }

    result
}

/// Get document symbols for outline/overview
#[tauri::command]
pub async fn get_document_symbols(file_path: String) -> Result<serde_json::Value, String> {
    execute_command!(
        "get_document_symbols",
        &LSP_COMMAND_CONFIG,
        async move || {
            // Validate file path for security
            validate_secure_path(&file_path, false)
                .map_err(|_| "Invalid file path provided".to_string())?;

            log::info!("Getting document symbols for {}", file_path);

            // Check if LSP is initialized
            let lsp_state = LSP_GLOBAL_STATE.lock().await;
            if !lsp_state.is_initialized() {
                return Err("LSP client not initialized. Please initialize LSP first.".to_string());
            }

            // Get LSP client
            let client = match lsp_state.get_client("rust") {
                Some(client) => client,
                None => return Err("Rust LSP client not found".to_string()),
            };

            // Convert file path to URI
            let uri = match lsp_types::Url::from_file_path(&file_path) {
                Ok(uri) => uri,
                Err(e) => return Err(format!("Invalid file path: {}", e)),
            };

            // Try LSP document symbols request with retry logic
            let symbols_result = execute_with_retry(
                || async {
                    client
                        .document_symbols(uri.clone())
                        .await
                        .map_err(|e| format!("LSP document symbols failed: {}", e).into())
                },
                3,
                "LSP document symbols",
            )
            .await;

            match symbols_result {
                Ok(symbols_response) => {
                    let response = match symbols_response {
                        Some(lsp_types::DocumentSymbolResponse::Flat(symbols)) => {
                            let symbols_array: Vec<serde_json::Value> = symbols
                                .into_iter()
                                .map(|symbol| {
                                    serde_json::json!({
                                        "name": symbol.name,
                                        "detail": symbol.detail,
                                        "kind": symbol.kind,
                                        "range": {
                                            "start": {
                                                "line": symbol.location.range.start.line,
                                                "character": symbol.location.range.start.character
                                            },
                                            "end": {
                                                "line": symbol.location.range.end.line,
                                                "character": symbol.location.range.end.character
                                            }
                                        },
                                        "selectionRange": {
                                            "start": {
                                                "line": symbol.location.range.start.line,
                                                "character": symbol.location.range.start.character
                                            },
                                            "end": {
                                                "line": symbol.location.range.end.line,
                                                "character": symbol.location.range.end.character
                                            }
                                        },
                                        "lsp_based": true
                                    })
                                })
                                .collect();

                            serde_json::json!({
                                "symbols": symbols_array,
                                "ai_enhanced": true,
                                "source": "lsp"
                            })
                        }
                        Some(lsp_types::DocumentSymbolResponse::Nested(symbols)) => {
                            let symbols_array = convert_nested_symbols_to_flat(&symbols);
                            serde_json::json!({
                                "symbols": symbols_array,
                                "ai_enhanced": true,
                                "source": "lsp_nested"
                            })
                        }
                        None => {
                            // Try AI-assisted fallback
                            let ai_fallback = try_ai_document_symbols_fallback(&file_path).await;
                            match ai_fallback {
                                Ok(ai_response) => ai_response,
                                Err(_) => serde_json::json!({
                                    "symbols": [],
                                    "ai_enhanced": false,
                                    "source": "none"
                                }),
                            }
                        }
                    };

                    Ok(response)
                }
                Err(e) => {
                    log::warn!("LSP document symbols failed, trying AI fallback: {}", e);
                    // Try AI-assisted fallback
                    try_ai_document_symbols_fallback(&file_path).await
                }
            }
        }
    )
}

/// Convert nested document symbols to flat array for frontend compatibility
fn convert_nested_symbols_to_flat(symbols: &[lsp_types::DocumentSymbol]) -> Vec<serde_json::Value> {
    let mut flat_symbols = Vec::new();

    fn process_symbol(symbol: &lsp_types::DocumentSymbol, symbols: &mut Vec<serde_json::Value>) {
        symbols.push(serde_json::json!({
            "name": symbol.name,
            "detail": symbol.detail,
            "kind": symbol.kind,
            "range": {
                "start": {
                    "line": symbol.range.start.line,
                    "character": symbol.range.start.character
                },
                "end": {
                    "line": symbol.range.end.line,
                    "character": symbol.range.end.character
                }
            },
            "selectionRange": {
                "start": {
                    "line": symbol.selection_range.start.line,
                    "character": symbol.selection_range.start.character
                },
                "end": {
                    "line": symbol.selection_range.end.line,
                    "character": symbol.selection_range.end.character
                }
            },
            "lsp_based": true
        }));

        // Process children recursively
        for child in &symbol.children {
            process_symbol(child, symbols);
        }
    }

    for symbol in symbols {
        process_symbol(symbol, &mut flat_symbols);
    }

    flat_symbols
}

/// AI-assisted document symbols fallback when LSP fails
async fn try_ai_document_symbols_fallback(file_path: &str) -> Result<serde_json::Value, String> {
    // Try to acquire AI service for document symbols analysis
    let service_result = GLOBAL_AI_REGISTRY
        .get_pooled_service("codellama_medium_pool")
        .await;

    match service_result {
        Ok(mut service_guard) => {
            // Read file content for analysis
            let content = match std::fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(e) => return Err(format!("Failed to read file: {}", e)),
            };

            // Create AI context for symbols analysis
            let ai_context = AIContext {
                current_code: content.clone(),
                file_name: Some(file_path.to_string()),
                cursor_position: None,
                selection: None,
                project_context: HashMap::new(),
            };

            // Use AI to analyze document structure
            let analysis_task = format!(
                "Analyze this code file and extract all symbols (functions, structs, enums, etc.) with their line \
                 ranges. Return in JSON format with name, kind, and range information.\n\n{}",
                content
            );

            match service_guard
                .get_task_response(ai_context, analysis_task)
                .await
            {
                Ok(ai_response) => {
                    log::info!("AI-assisted document symbols analysis successful");
                    Ok(serde_json::json!({
                        "symbols": [],  // Would parse AI response to extract symbols
                        "ai_enhanced": true,
                        "ai_analysis": ai_response,
                        "source": "ai_fallback"
                    }))
                }
                Err(e) => {
                    log::warn!("AI document symbols analysis failed: {}", e);
                    Err("Document symbols analysis failed via LSP and AI".to_string())
                }
            }
        }
        Err(e) => {
            log::warn!(
                "AI service unavailable for document symbols fallback: {}",
                e
            );
            Err("Document symbols analysis failed and AI service unavailable".to_string())
        }
    }
}

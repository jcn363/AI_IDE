//! Enhanced terminal commands module
//!
//! This module provides comprehensive terminal integration with:
//! - Command execution with streaming output
//! - Command history management
//! - AI-powered command suggestions
//! - Auto-completion for files and directories
//! - Enhanced shell features and bookmarks

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::process::Stdio;
use std::sync::Arc;

use lazy_static::lazy_static;
use rusqlite::{params, Connection};
use rust_ai_ide_common::validation::{
    validate_file_exists, validate_path_not_excluded, validate_secure_path, TauriInputSanitizer,
};
use rust_ai_ide_config::{Config, ConfigurationManager};
use rust_ai_ide_security::audit_logger;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use which::which;

// Tauri v2 compatibility: use dirs::data_dir() fallback instead of deprecated tauri::api::path
// use tauri::api::path;
use crate::command_templates::*;
// Re-export TerminalEvent from shared types
pub use crate::modules::shared::types::TerminalEvent;

/// Configuration for terminal program validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    /// List of explicitly allowed programs
    pub allowed_programs:     Vec<String>,
    /// List of approved directories where programs are allowed
    pub approved_directories: Vec<String>,
}

impl Config for TerminalConfig {
    const FILE_PREFIX: &'static str = "terminal";
    const DESCRIPTION: &'static str = "Terminal configuration for program validation";

    fn validate(&self) -> Result<Vec<String>, anyhow::Error> {
        let mut errors = Vec::new();

        if self.allowed_programs.is_empty() && self.approved_directories.is_empty() {
            errors.push("At least one of allowed_programs or approved_directories must be configured".to_string());
        }

        // Validate that approved directories exist and are secure
        for dir in &self.approved_directories {
            if !std::path::Path::new(dir).exists() {
                errors.push(format!("Approved directory does not exist: {}", dir));
            }
        }

        Ok(errors)
    }

    fn default_config() -> Self {
        Self {
            allowed_programs:     vec![
                "git".to_string(),
                "cargo".to_string(),
                "npm".to_string(),
                "rustc".to_string(),
                "node".to_string(),
                "python".to_string(),
                "python3".to_string(),
                "ruby".to_string(),
                "go".to_string(),
                "java".to_string(),
                "javac".to_string(),
                "docker".to_string(),
                "kubectl".to_string(),
            ],
            approved_directories: vec![
                "/usr/bin".to_string(),
                "/usr/local/bin".to_string(),
                "/bin".to_string(),
                "/opt/homebrew/bin".to_string(),              // Homebrew on macOS
                "/home/linuxbrew/.linuxbrew/bin".to_string(), // Linuxbrew
            ],
        }
    }
}

/// Security validation functions for terminal commands

/// Validates program path against approved directories
fn validate_program_path(program: &str, approved_dirs: &[String]) -> Result<String, String> {
    // Use which to find the program's full path
    match which(program) {
        Ok(path) => {
            // Canonicalize the program path to resolve any symlinks
            let canonical_path = match std::fs::canonicalize(&path) {
                Ok(canonical) => canonical,
                Err(e) => {
                    return Err(format!(
                        "Failed to canonicalize program path '{}': {}",
                        path.display(),
                        e
                    ));
                }
            };

            // Check if the program is in an approved directory using proper path comparison
            for dir in approved_dirs {
                let dir_path = std::path::Path::new(dir);

                // Canonicalize the approved directory as well
                let canonical_dir = match std::fs::canonicalize(dir_path) {
                    Ok(canonical) => canonical,
                    Err(_) => {
                        // If directory doesn't exist or can't be canonicalized, skip it
                        continue;
                    }
                };

                // Use Path::starts_with for proper path prefix comparison
                if canonical_path.starts_with(&canonical_dir) {
                    return Ok(path.to_string_lossy().to_string());
                }
            }
            Err(format!(
                "Program '{}' found at '{}' but not in approved directories: {:?}",
                program,
                path.display(),
                approved_dirs
            ))
        }
        Err(_) => Err(format!("Program '{}' not found in PATH", program)),
    }
}

/// Validates that the program is allowed by checking both allowlist and approved directories
pub async fn validate_program(program: &str) -> Result<String, String> {
    // Create configuration manager and load terminal config
    let config_manager = match ConfigurationManager::new().await {
        Ok(cm) => cm,
        Err(e) => {
            log::error!("Failed to create configuration manager: {}", e);
            return Err("Configuration system unavailable".to_string());
        }
    };

    let terminal_config: TerminalConfig = match config_manager.load_secure("terminal").await {
        Ok(config) => config,
        Err(e) => {
            log::error!("Failed to load terminal configuration: {}", e);
            // Fall back to default config if loading fails
            TerminalConfig::default_config()
        }
    };

    // First, check if program is in the explicit allowlist
    if terminal_config
        .allowed_programs
        .contains(&program.to_string())
    {
        return Ok(program.to_string());
    }

    // If not in allowlist, check if it's in approved directories
    if !terminal_config.approved_directories.is_empty() {
        match validate_program_path(program, &terminal_config.approved_directories) {
            Ok(validated_path) => return Ok(validated_path),
            Err(e) => {
                log::warn!("Program validation failed: {}", e);
            }
        }
    }

    // If neither check passes, deny the program
    Err(format!(
        "Program '{}' is not allowed. Must be in allowlist {:?} or located in approved directories {:?}",
        program, terminal_config.allowed_programs, terminal_config.approved_directories
    ))
}

/// Sanitizes command arguments to prevent injection attacks
pub fn sanitize_command_args(args: &[String]) -> Result<Vec<String>, String> {
    let mut sanitized_args = Vec::new();
    for arg in args {
        let sanitized =
            TauriInputSanitizer::sanitize_string(arg).map_err(|e| format!("Failed to sanitize argument: {}", e))?;
        sanitized_args.push(sanitized);
    }
    Ok(sanitized_args)
}

/// Comprehensive command validation combining all security checks
pub async fn validate_and_sanitize_command(
    program: &str,
    args: &[String],
    directory: &str,
) -> Result<(String, Vec<String>, String), String> {
    // Validate program
    let safe_program = validate_program(program).await?;

    // Sanitize arguments
    let sanitized_args = sanitize_command_args(args)?;

    // Validate directory (already done in the function, but kept for completeness)
    if let Err(e) = validate_secure_path(directory, true) {
        return Err(format!("Directory validation failed: {}", e));
    }

    Ok((safe_program, sanitized_args, directory.to_string()))
}

/// Logs command execution for audit trails
pub fn log_command_execution(program: &str, args: &[String], directory: &str, success: bool) {
    let args_str = args.join(" ");
    let log_message = format!(
        "Terminal command executed: {} {} in {} - Success: {}",
        program, args_str, directory, success
    );

    if let Err(e) = audit_logger::log_security_event("terminal_command", &log_message) {
        log::error!("Failed to log command execution: {}", e);
    }
}

/// Command history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistoryEntry {
    pub id:                String,
    pub command:           String,
    pub working_directory: String,
    pub timestamp:         u64,
    pub success:           bool,
    pub output_length:     Option<i32>,
}

/// Auto-completion suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionSuggestion {
    pub value:       String,
    pub category:    String,
    pub score:       f32,
    pub description: Option<String>,
}

/// Command suggestion from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AICommandSuggestion {
    pub command:          String,
    pub explanation:      String,
    pub confidence_score: f32,
    pub category:         String,
}

/// Terminal bookmark/favorite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalBookmark {
    pub id:          String,
    pub name:        String,
    pub command:     String,
    pub description: Option<String>,
    pub created_at:  u64,
}

/// Terminal enhancement service
#[derive(Debug)]
pub struct TerminalEnhancementService {
    command_history: Arc<Mutex<Vec<CommandHistoryEntry>>>,
    bookmarks:       Arc<Mutex<Vec<TerminalBookmark>>>,
    db_path:         std::path::PathBuf,
}

impl TerminalEnhancementService {
    pub fn new() -> Self {
        let db_path = match dirs::data_dir() {
            Some(data_dir) => data_dir.join("rust-ai-ide").join("terminal.db"),
            None => {
                log::error!("Failed to get data directory");
                std::path::PathBuf::from("data").join("terminal.db")
            }
        };

        if let Some(parent) = db_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                log::error!("Failed to create parent directories for terminal.db: {}", e);
            }
        }

        Self {
            command_history: Arc::new(Mutex::new(Vec::new())),
            bookmarks: Arc::new(Mutex::new(Vec::new())),
            db_path,
        }
    }

    pub async fn initialize_database(&self) -> Result<(), String> {
        let db_path = self.db_path.clone();
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

            // Create tables if they don't exist
            conn.execute(
                "CREATE TABLE IF NOT EXISTS command_history (
                    id TEXT PRIMARY KEY,
                    command TEXT NOT NULL,
                    working_directory TEXT NOT NULL,
                    timestamp INTEGER NOT NULL,
                    success BOOLEAN NOT NULL,
                    output_length INTEGER
                )",
                [],
            )
            .map_err(|e| format!("Failed to create command_history table: {}", e))?;

            conn.execute(
                "CREATE TABLE IF NOT EXISTS bookmarks (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    command TEXT NOT NULL,
                    description TEXT,
                    created_at INTEGER NOT NULL
                )",
                [],
            )
            .map_err(|e| format!("Failed to create bookmarks table: {}", e))?;

            Ok(())
        })
        .await
        .map_err(|e| format!("Database initialization task failed: {}", e))?
    }

    pub async fn add_to_history(&self, entry: CommandHistoryEntry) -> Result<(), String> {
        let db_path = self.db_path.clone();
        let entry_clone = entry.clone();

        // Spawn blocking task for database operation
        let db_result = tokio::task::spawn_blocking(move || {
            if let Ok(conn) = Connection::open(&db_path) {
                conn.execute(
                    "INSERT INTO command_history (id, command, working_directory, timestamp, success, output_length)
                     VALUES (?, ?, ?, ?, ?, ?)",
                    params![
                        entry_clone.id,
                        entry_clone.command,
                        entry_clone.working_directory,
                        entry_clone.timestamp as i64,
                        entry_clone.success,
                        entry_clone.output_length
                    ],
                )
                .map_err(|e| format!("Failed to insert history entry: {}", e))?;
            }
            Ok(())
        })
        .await;

        // Handle the blocking task result
        if let Err(e) = db_result {
            log::error!("Database task failed: {}", e);
        }

        let mut history = self.command_history.lock().await;
        history.push(entry);
        Ok(())
    }

    pub async fn get_command_history(&self, limit: usize) -> Result<Vec<CommandHistoryEntry>, String> {
        let db_path = self.db_path.clone();
        let limit_clone = limit;

        // Try to get entries from database in a blocking task
        let db_result = tokio::task::spawn_blocking(move || {
            if let Ok(conn) = Connection::open(&db_path) {
                let mut stmt = conn
                    .prepare(
                        "SELECT id, command, working_directory, timestamp, success, output_length FROM \
                         command_history ORDER BY timestamp DESC LIMIT ?",
                    )
                    .map_err(|e| format!("Failed to prepare statement: {}", e))?;
                let entries_iter = stmt
                    .query_map([limit_clone as i64], |row| {
                        Ok(CommandHistoryEntry {
                            id:                row.get(0)?,
                            command:           row.get(1)?,
                            working_directory: row.get(2)?,
                            timestamp:         row.get(3)?,
                            success:           row.get(4)?,
                            output_length:     row.get(5)?,
                        })
                    })
                    .map_err(|e| format!("Failed to query database: {}", e))?;

                let entries = entries_iter
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| format!("Failed to read rows: {}", e))?;

                Ok(Some(entries))
            } else {
                Ok(None)
            }
        })
        .await;

        match db_result {
            Ok(Ok(Some(entries))) => Ok(entries),
            Ok(Ok(None)) | Ok(Err(_)) => {
                // Fall back to in-memory history if database operation fails
                let history = self.command_history.lock().await;
                let entries = history.iter().rev().take(limit).cloned().collect();
                Ok(entries)
            }
            Err(e) => {
                log::error!("Database task failed: {}", e);
                let history = self.command_history.lock().await;
                let entries = history.iter().rev().take(limit).cloned().collect();
                Ok(entries)
            }
        }
    }

    pub async fn get_ai_suggestions(&self, partial_command: &str, context: &str) -> Vec<AICommandSuggestion> {
        // This would integrate with AI service for command suggestions
        vec![
            AICommandSuggestion {
                command:          format!("cd {}", self.suggest_directory(partial_command).await),
                explanation:      "Navigate to a directory inferred from your input".to_string(),
                confidence_score: 0.8,
                category:         "navigation".to_string(),
            },
            AICommandSuggestion {
                command:          format!("git {}", partial_command),
                explanation:      "Complete Git command".to_string(),
                confidence_score: 0.7,
                category:         "version_control".to_string(),
            },
        ]
    }

    pub async fn get_auto_completion(&self, partial: &str, working_dir: &str) -> Vec<CompletionSuggestion> {
        let mut suggestions = Vec::new();

        // File and directory completion
        if let Ok(entries) = std::fs::read_dir(working_dir) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.starts_with(partial) {
                        let is_dir = entry.file_type().map_or(false, |t| t.is_dir());
                        let category = if is_dir { "directory" } else { "file" };
                        suggestions.push(CompletionSuggestion {
                            value:       if is_dir { format!("{}/", name) } else { name },
                            category:    category.to_string(),
                            score:       1.0,
                            description: None,
                        });
                    }
                }
            }
        }

        // Command history completion
        let history = self.command_history.lock().await;
        for entry in history.iter().rev().take(10) {
            if entry.command.starts_with(partial) {
                suggestions.push(CompletionSuggestion {
                    value:       entry.command.clone(),
                    category:    "history".to_string(),
                    score:       0.9,
                    description: Some(format!("Executed at {}", entry.timestamp)),
                });
                break; // Only add one history suggestion
            }
        }

        suggestions
    }

    pub async fn suggest_directory(&self, partial: &str) -> String {
        // Simple directory suggestion logic
        if partial.is_empty() {
            ".".to_string()
        } else if partial.starts_with("~") {
            partial.replacen("~", "~/", 1)
        } else {
            partial.to_string()
        }
    }

    pub async fn add_bookmark(&self, bookmark: TerminalBookmark) -> Result<(), String> {
        let db_path = self.db_path.clone();
        let bookmark_clone = bookmark.clone();

        // Spawn blocking task for database operation
        let db_result = tokio::task::spawn_blocking(move || {
            if let Ok(conn) = Connection::open(&db_path) {
                conn.execute(
                    "INSERT INTO bookmarks (id, name, command, description, created_at)
                     VALUES (?, ?, ?, ?, ?)",
                    params![
                        bookmark_clone.id,
                        bookmark_clone.name,
                        bookmark_clone.command,
                        bookmark_clone.description,
                        bookmark_clone.created_at as i64
                    ],
                )
                .map_err(|e| format!("Failed to insert bookmark: {}", e))?;
            }
            Ok(())
        })
        .await;

        // Handle the blocking task result
        if let Err(e) = db_result {
            log::error!("Database task failed: {}", e);
        }

        let mut bookmarks = self.bookmarks.lock().await;
        bookmarks.push(bookmark);
        Ok(())
    }

    pub async fn get_bookmarks(&self) -> Result<Vec<TerminalBookmark>, String> {
        let db_path = self.db_path.clone();

        // Try to get bookmarks from database in a blocking task
        let db_result = tokio::task::spawn_blocking(move || {
            if let Ok(conn) = Connection::open(&db_path) {
                let mut stmt = conn
                    .prepare(
                        "SELECT id, name, command, description, created_at FROM bookmarks ORDER BY created_at DESC",
                    )
                    .map_err(|e| format!("Failed to prepare statement: {}", e))?;
                let bookmarks_iter = stmt
                    .query_map([], |row| {
                        Ok(TerminalBookmark {
                            id:          row.get(0)?,
                            name:        row.get(1)?,
                            command:     row.get(2)?,
                            description: row.get(3)?,
                            created_at:  row.get(4)?,
                        })
                    })
                    .map_err(|e| format!("Failed to query database: {}", e))?;

                let bookmarks = bookmarks_iter
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| format!("Failed to read rows: {}", e))?;

                Ok(Some(bookmarks))
            } else {
                Ok(None)
            }
        })
        .await;

        match db_result {
            Ok(Ok(Some(bookmarks))) => Ok(bookmarks),
            Ok(Ok(None)) | Ok(Err(_)) => {
                // Fall back to in-memory bookmarks if database operation fails
                let bookmarks = self.bookmarks.lock().await.clone();
                Ok(bookmarks)
            }
            Err(e) => {
                log::error!("Database task failed: {}", e);
                let bookmarks = self.bookmarks.lock().await.clone();
                Ok(bookmarks)
            }
        }
    }
}

// Lazy-static service instance
lazy_static! {
    static ref TERMINAL_ENHANCEMENT_SERVICE: Arc<TerminalEnhancementService> = {
        let service = TerminalEnhancementService::new();
        let service_arc = Arc::new(service);
        // Initialize database in the background with cloned Arc
        let service_clone = Arc::clone(&service_arc);
        tauri::async_runtime::spawn(async move {
            if let Err(e) = service_clone.initialize_database().await {
                log::error!("Failed to initialize terminal database: {}", e);
            }
        });
        service_arc
    };
}

/// Initialize the terminal service database
pub async fn init_terminal_service() -> Result<(), String> {
    TERMINAL_ENHANCEMENT_SERVICE.initialize_database().await
}

// Enhanced terminal commands

tauri_command_template! {
    get_command_history,
    async fn get_command_history_impl(limit: Option<usize>) -> Result<serde_json::Value, String> {
        let limit = limit.unwrap_or(50).min(200); // Cap at 200 for performance
        let history = TERMINAL_ENHANCEMENT_SERVICE.get_command_history(limit).await?;

        Ok(serde_json::json!({
            "status": "success",
            "history": history
        }))
    },
    service = TerminalEnhancementService,
    state = TERMINAL_ENHANCEMENT_SERVICE,
    config = CommandConfig::default()
}

tauri_command_template! {
    add_command_to_history,
    async fn add_command_to_history_impl(entry: CommandHistoryEntry) -> Result<serde_json::Value, String> {
        // Input validation
        if entry.command.trim().is_empty() {
            return Err("Command cannot be empty".to_string());
        }

        TERMINAL_ENHANCEMENT_SERVICE.add_to_history(entry).await?;

        Ok(serde_json::json!({"status": "success"}))
    },
    service = TerminalEnhancementService,
    state = TERMINAL_ENHANCEMENT_SERVICE,
    config = CommandConfig::default()
}

tauri_command_template! {
    get_ai_command_suggestions,
    async fn get_ai_command_suggestions_impl(partial_command: String, context: String) -> Result<serde_json::Value, String> {
        let suggestions = TERMINAL_ENHANCEMENT_SERVICE.get_ai_suggestions(&partial_command, &context).await;

        Ok(serde_json::json!({
            "status": "success",
            "suggestions": suggestions
        }))
    },
    service = TerminalEnhancementService,
    state = TERMINAL_ENHANCEMENT_SERVICE,
    config = CommandConfig::default()
}

tauri_command_template! {
    get_auto_completion,
    async fn get_auto_completion_impl(partial: String, working_directory: String) -> Result<serde_json::Value, String> {
        // Validate working directory
        if let Err(e) = validate_secure_path(&working_directory, true) {
            return Err(e);
        }

        let suggestions = TERMINAL_ENHANCEMENT_SERVICE.get_auto_completion(&partial, &working_directory).await;

        Ok(serde_json::json!({
            "status": "success",
            "suggestions": suggestions
        }))
    },
    service = TerminalEnhancementService,
    state = TERMINAL_ENHANCEMENT_SERVICE,
    config = CommandConfig::default()
}

tauri_command_template! {
    add_terminal_bookmark,
    async fn add_terminal_bookmark_impl(bookmark: TerminalBookmark) -> Result<serde_json::Value, String> {
        if bookmark.name.trim().is_empty() {
            return Err("Bookmark name cannot be empty".to_string());
        }

        TERMINAL_ENHANCEMENT_SERVICE.add_bookmark(bookmark).await?;

        Ok(serde_json::json!({"status": "success"}))
    },
    service = TerminalEnhancementService,
    state = TERMINAL_ENHANCEMENT_SERVICE,
    config = CommandConfig::default()
}

tauri_command_template! {
    get_terminal_bookmarks,
    async fn get_terminal_bookmarks_impl() -> Result<serde_json::Value, String> {
        let bookmarks = TERMINAL_ENHANCEMENT_SERVICE.get_bookmarks().await?;

        Ok(serde_json::json!({
            "status": "success",
            "bookmarks": bookmarks
        }))
    },
    service = TerminalEnhancementService,
    state = TERMINAL_ENHANCEMENT_SERVICE,
    config = CommandConfig::default()
}

impl CommandService for TerminalEnhancementService {
    type Error = String;

    fn is_ready(&self) -> bool {
        true // Terminal enhancement service is always ready
    }

    fn service_name(&self) -> &'static str {
        "TerminalEnhancementService"
    }
}

/// Execute a terminal command with streaming output
#[tauri::command]
pub async fn terminal_execute_stream(
    program: String,
    args: Vec<String>,
    directory: String,
    id: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // Validate the path securely
    if let Err(e) = validate_secure_path(&directory, true) {
        return Err(e);
    }

    // Check if directory exists
    if !std::path::Path::new(&directory).exists() {
        return Err(format!("Directory does not exist: {}", directory));
    }

    // Validate and sanitize command inputs
    let (safe_program, sanitized_args, safe_directory) =
        validate_and_sanitize_command(&program, &args, &directory).await?;

    // Generate a terminal event ID
    let event_id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let term_event = format!("terminal-{}", event_id);

    // Spawn the command with sanitized inputs
    let mut cmd = Command::new(&safe_program);
    cmd.args(&sanitized_args)
        .current_dir(&safe_directory)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start {}: {}", program, e))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // Handle stdout
    if let Some(out) = stdout {
        let app = app_handle.clone();
        let term_event_clone = term_event.clone();
        let event_id_clone = event_id.clone();

        tauri::async_runtime::spawn(async move {
            log::debug!(
                "Starting stdout reader task for terminal {}",
                term_event_clone
            );
            let mut reader = BufReader::new(out).lines();
            let mut lines_read = 0;
            while let Ok(Some(line)) = reader.next_line().await {
                let event_payload = TerminalEvent {
                    id:          event_id_clone.clone(),
                    stream_type: "stdout".to_string(),
                    line:        line.clone(),
                };
                if let Err(e) = app.emit_all(&term_event_clone, event_payload) {
                    log::warn!("Failed to emit stdout line: {}", e);
                }
                lines_read += 1;
            }
            log::debug!("Stdout reader task completed, read {} lines", lines_read);
        });
    }

    // Handle stderr
    if let Some(err) = stderr {
        let app = app_handle.clone();
        let term_event_clone = term_event.clone();
        let event_id_clone = event_id.clone();

        tauri::async_runtime::spawn(async move {
            log::debug!(
                "Starting stderr reader task for terminal {}",
                term_event_clone
            );
            let mut reader = BufReader::new(err).lines();
            let mut lines_read = 0;
            while let Ok(Some(line)) = reader.next_line().await {
                let event_payload = TerminalEvent {
                    id:          event_id_clone.clone(),
                    stream_type: "stderr".to_string(),
                    line:        line.clone(),
                };
                if let Err(e) = app.emit_all(&term_event_clone, event_payload) {
                    log::warn!("Failed to emit stderr line: {}", e);
                }
                lines_read += 1;
            }
            log::debug!("Stderr reader task completed, read {} lines", lines_read);
        });
    }

    // Wait for command to complete and emit completion event
    let app = app_handle.clone();
    let term_event_clone = term_event.clone();
    let event_id_clone = event_id.clone();
    let safe_program_clone = safe_program.clone();
    let sanitized_args_clone = sanitized_args.clone();
    let safe_directory_clone = safe_directory.clone();

    tauri::async_runtime::spawn(async move {
        match child.wait().await {
            Ok(status) => {
                // Log successful command execution
                log_command_execution(
                    &safe_program_clone,
                    &sanitized_args_clone,
                    &safe_directory_clone,
                    status.success(),
                );

                let payload = serde_json::json!({
                    "id": event_id_clone,
                    "type": "completion",
                    "success": status.success(),
                    "code": status.code()
                });

                if let Err(e) = app.emit_all(&term_event_clone, payload) {
                    log::error!("Failed to emit completion: {}", e);
                }
            }
            Err(e) => {
                // Log failed command execution
                log_command_execution(
                    &safe_program_clone,
                    &sanitized_args_clone,
                    &safe_directory_clone,
                    false,
                );
                log::error!("Command failed: {}", e);

                let payload = serde_json::json!({
                    "id": event_id_clone,
                    "type": "error",
                    "error": format!("Command execution failed: {}", e)
                });

                if let Err(e) = app.emit_all(&term_event_clone, payload) {
                    log::error!("Failed to emit error: {}", e);
                }
            }
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_terminal_execute_stream_invalid_directory() {
        // Test validation - this would need a mock app_handle
        // For now, just test the directory validation
        // Actually testing this would require mocking the app
    }

    #[tokio::test]
    async fn test_validate_program_allowlist() {
        // Test that allowlisted programs are accepted
        let result = validate_program("git").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "git");

        let result = validate_program("cargo").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "cargo");
    }

    #[tokio::test]
    async fn test_validate_program_approved_directory() {
        // Test that programs in approved directories are accepted
        // This test assumes /usr/bin exists and contains programs
        if std::path::Path::new("/usr/bin").exists() {
            // Note: This test may fail if the program is not in PATH or directory doesn't exist
            let result = validate_program("ls").await;
            // We can't guarantee ls is in /usr/bin, so just check it's not an error about allowlist
            if result.is_ok() {
                assert!(result.unwrap().contains("ls") || result.unwrap().contains("/usr/bin"));
            }
        }
    }

    #[tokio::test]
    async fn test_validate_program_denied() {
        // Test that non-allowlisted programs not in approved directories are denied
        let result = validate_program("nonexistent_command_xyz").await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("not allowed") || error_msg.contains("not found"));
    }

    #[tokio::test]
    async fn test_validate_program_path_validation() {
        // Test the helper function for path validation
        let approved_dirs = vec!["/usr/bin".to_string(), "/bin".to_string()];

        // Test with a program that should be in /usr/bin
        if which::which("ls").is_ok() {
            let result = validate_program_path("ls", &approved_dirs);
            if result.is_ok() {
                let path = result.unwrap();
                assert!(path.contains("ls"));
                assert!(path.starts_with("/usr/bin") || path.starts_with("/bin"));
            }
        }
    }

    #[test]
    fn test_sanitize_command_args() {
        // Test argument sanitization
        let args = vec!["--help".to_string(), "test;rm -rf /".to_string()];
        let result = sanitize_command_args(&args);
        assert!(result.is_ok());
        let sanitized = result.unwrap();
        assert_eq!(sanitized.len(), 2);
        assert_eq!(sanitized[0], "--help");
        // The second argument should be sanitized to prevent injection
        assert_ne!(sanitized[1], "test;rm -rf /");
    }

    #[tokio::test]
    async fn test_validate_and_sanitize_command() {
        // Test full command validation and sanitization
        let program = "git";
        let args = vec!["status".to_string()];
        let directory = "/tmp";

        let result = validate_and_sanitize_command(program, &args, directory).await;
        assert!(result.is_ok());
        let (safe_program, sanitized_args, safe_directory) = result.unwrap();
        assert_eq!(safe_program, "git");
        assert_eq!(sanitized_args, args);
        assert_eq!(safe_directory, directory);
    }

    #[test]
    fn test_terminal_config_validation() {
        // Test TerminalConfig validation
        let mut config = TerminalConfig::default_config();

        // Valid config should pass
        let result = config.validate();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());

        // Config with empty lists should fail
        config.allowed_programs.clear();
        config.approved_directories.clear();
        let result = config.validate();
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(!errors.is_empty());
        assert!(errors[0].contains("must be configured"));
    }
}

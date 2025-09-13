//! Enhanced terminal commands module
//!
//! This module provides comprehensive terminal integration with:
//! - Command execution with streaming output
//! - Command history management
//! - AI-powered command suggestions
//! - Auto-completion for files and directories
//! - Enhanced shell features and bookmarks

use crate::command_templates::*;
use lazy_static::lazy_static;
use rusqlite::{params, Connection};
use rust_ai_ide_core::security::{audit_action, audit_logger};
use rust_ai_ide_core::validation::validate_secure_path;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

use rust_ai_ide_common::validation::{validate_file_exists, validate_path_not_excluded};

// Re-export TerminalEvent from shared types
pub use crate::modules::shared::types::TerminalEvent;

/// Command history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistoryEntry {
    pub id: String,
    pub command: String,
    pub working_directory: String,
    pub timestamp: u64,
    pub success: bool,
    pub output_length: Option<i32>,
}

/// Auto-completion suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionSuggestion {
    pub value: String,
    pub category: String,
    pub score: f32,
    pub description: Option<String>,
}

/// Command suggestion from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AICommandSuggestion {
    pub command: String,
    pub explanation: String,
    pub confidence_score: f32,
    pub category: String,
}

/// Terminal bookmark/favorite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalBookmark {
    pub id: String,
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub created_at: u64,
}

/// Terminal enhancement service
#[derive(Debug)]
pub struct TerminalEnhancementService {
    command_history: Arc<Mutex<Vec<CommandHistoryEntry>>>,
    bookmarks: Arc<Mutex<Vec<TerminalBookmark>>>,
    db_path: std::path::PathBuf,
}

impl TerminalEnhancementService {
    pub fn new() -> Self {
        let db_path = std::path::PathBuf::from("data/terminal.db");
        std::fs::create_dir_all(db_path.parent().unwrap()).ok();

        Self {
            command_history: Arc::new(Mutex::new(Vec::new())),
            bookmarks: Arc::new(Mutex::new(Vec::new())),
            db_path,
        }
    }

    pub async fn initialize_database(&self) -> Result<(), String> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

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
    }

    pub async fn add_to_history(&self, entry: CommandHistoryEntry) -> Result<(), String> {
        if let Ok(conn) = Connection::open(&self.db_path) {
            conn.execute(
                "INSERT INTO command_history (id, command, working_directory, timestamp, success, output_length)
                 VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    entry.id,
                    entry.command,
                    entry.working_directory,
                    entry.timestamp as i64,
                    entry.success,
                    entry.output_length
                ],
            ).map_err(|e| format!("Failed to insert history entry: {}", e))?;
        }

        let mut history = self.command_history.lock().await;
        history.push(entry);
        Ok(())
    }

    pub async fn get_command_history(&self, limit: usize) -> Vec<CommandHistoryEntry> {
        if let Ok(conn) = Connection::open(&self.db_path) {
            let mut stmt = conn.prepare("SELECT id, command, working_directory, timestamp, success, output_length FROM command_history ORDER BY timestamp DESC LIMIT ?").unwrap();
            let entries_iter = stmt
                .query_map([limit as i64], |row| {
                    Ok(CommandHistoryEntry {
                        id: row.get(0)?,
                        command: row.get(1)?,
                        working_directory: row.get(2)?,
                        timestamp: row.get(3)?,
                        success: row.get(4)?,
                        output_length: row.get(5)?,
                    })
                })
                .unwrap();

            entries_iter.filter_map(|r| r.ok()).collect()
        } else {
            let history = self.command_history.lock().await;
            history.iter().rev().take(limit).cloned().collect()
        }
    }

    pub async fn get_ai_suggestions(
        &self,
        partial_command: &str,
        context: &str,
    ) -> Vec<AICommandSuggestion> {
        // This would integrate with AI service for command suggestions
        vec![
            AICommandSuggestion {
                command: format!("cd {}", self.suggest_directory(partial_command).await),
                explanation: "Navigate to a directory inferred from your input".to_string(),
                confidence_score: 0.8,
                category: "navigation".to_string(),
            },
            AICommandSuggestion {
                command: format!("git {}", partial_command),
                explanation: "Complete Git command".to_string(),
                confidence_score: 0.7,
                category: "version_control".to_string(),
            },
        ]
    }

    pub async fn get_auto_completion(
        &self,
        partial: &str,
        working_dir: &str,
    ) -> Vec<CompletionSuggestion> {
        let mut suggestions = Vec::new();

        // File and directory completion
        if let Ok(entries) = std::fs::read_dir(working_dir) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.starts_with(partial) {
                        let is_dir = entry.file_type().map_or(false, |t| t.is_dir());
                        let category = if is_dir { "directory" } else { "file" };
                        suggestions.push(CompletionSuggestion {
                            value: if is_dir { format!("{}/", name) } else { name },
                            category: category.to_string(),
                            score: 1.0,
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
                    value: entry.command.clone(),
                    category: "history".to_string(),
                    score: 0.9,
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
        if let Ok(conn) = Connection::open(&self.db_path) {
            conn.execute(
                "INSERT INTO bookmarks (id, name, command, description, created_at)
                 VALUES (?, ?, ?, ?, ?)",
                params![
                    bookmark.id,
                    bookmark.name,
                    bookmark.command,
                    bookmark.description,
                    bookmark.created_at as i64
                ],
            )
            .map_err(|e| format!("Failed to insert bookmark: {}", e))?;
        }

        let mut bookmarks = self.bookmarks.lock().await;
        bookmarks.push(bookmark);
        Ok(())
    }

    pub async fn get_bookmarks(&self) -> Vec<TerminalBookmark> {
        if let Ok(conn) = Connection::open(&self.db_path) {
            let mut stmt = conn.prepare("SELECT id, name, command, description, created_at FROM bookmarks ORDER BY created_at DESC").unwrap();
            let bookmarks_iter = stmt
                .query_map([], |row| {
                    Ok(TerminalBookmark {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        command: row.get(2)?,
                        description: row.get(3)?,
                        created_at: row.get(4)?,
                    })
                })
                .unwrap();

            bookmarks_iter.filter_map(|r| r.ok()).collect()
        } else {
            let bookmarks = self.bookmarks.lock().await.clone();
            bookmarks
        }
    }
}

// Lazy-static service instance
lazy_static! {
    static ref TERMINAL_ENHANCEMENT_SERVICE: TerminalEnhancementService = {
        let service = TerminalEnhancementService::new();
        // Initialize database in the background
        tauri::async_runtime::spawn(async move {
            if let Err(e) = service.initialize_database().await {
                log::error!("Failed to initialize terminal database: {}", e);
            }
        });
        service
    };
}

// Enhanced terminal commands

tauri_command_template! {
    get_command_history,
    async fn get_command_history_impl(limit: Option<usize>) -> Result<serde_json::Value, String> {
        let limit = limit.unwrap_or(50).min(200); // Cap at 200 for performance
        let history = TERMINAL_ENHANCEMENT_SERVICE.get_command_history(limit).await;

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
        let bookmarks = TERMINAL_ENHANCEMENT_SERVICE.get_bookmarks().await;

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

    // Generate a terminal event ID
    let event_id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let term_event = format!("terminal-{}", event_id);

    // Spawn the command
    let mut cmd = Command::new(&program);
    cmd.args(&args)
        .current_dir(&directory)
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
            while let Ok(Some(line)) = reader.try_next().await {
                let event_payload = TerminalEvent {
                    id: event_id_clone.clone(),
                    stream_type: "stdout".to_string(),
                    line: line.clone(),
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
            while let Ok(Some(line)) = reader.try_next().await {
                let event_payload = TerminalEvent {
                    id: event_id_clone.clone(),
                    stream_type: "stderr".to_string(),
                    line: line.clone(),
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

    tauri::async_runtime::spawn(async move {
        match child.wait().await {
            Ok(status) => {
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
}

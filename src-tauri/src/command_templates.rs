//! Generic command template for standardized Tauri command handlers
//!
//! This module provides macros, traits, and utilities to standardize command implementations,
//! reducing boilerplate code and ensuring consistent error handling and validation patterns.

use rust_ai_ide_common::validation::{
    validate_directory_exists, validate_file_exists, validate_file_size_content,
    validate_path_not_excluded,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use tauri::{AppHandle, State};
use uuid;

pub use anyhow::{anyhow, Result};

/// Configuration for command behavior
#[derive(Debug, Clone)]
pub struct CommandConfig {
    pub enable_logging: bool,
    pub log_level: log::Level,
    pub enable_validation: bool,
    pub async_timeout_secs: Option<u64>,
}

impl Default for CommandConfig {
    fn default() -> Self {
        Self {
            enable_logging: true,
            log_level: log::Level::Info,
            enable_validation: true,
            async_timeout_secs: None,
        }
    }
}

/// Standard command context for accessing services and state
pub struct CommandContext<'r, T> {
    pub service: &'r mut T,
    pub app_handle: Option<AppHandle>,
}

impl<'r, T> CommandContext<'r, T> {
    pub fn new(service: &'r mut T) -> Self {
        Self {
            service,
            app_handle: None,
        }
    }

    pub fn with_app_handle(self, app_handle: AppHandle) -> Self {
        Self {
            app_handle: Some(app_handle),
            ..self
        }
    }
}

/// Trait for services that can be used in commands
pub trait CommandService {
    type Error: fmt::Display;

    /// Check if the service is initialized and ready
    fn is_ready(&self) -> bool;

    /// Get service name for logging
    fn service_name(&self) -> &'static str;
}

/// Standardized error type for commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandError {
    pub message: String,
    pub code: Option<String>,
    pub details: Option<String>,
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CommandError {}

/// Helper for standard error formatting
pub fn format_command_error<E: fmt::Display>(error: E, operation: &str) -> String {
    format!("{} failed: {}", operation, error)
}

/// Helper for async service acquisition with timeout
pub async fn acquire_service<'a, T, F, Fut>(
    state: State<'a, T>,
    config: &'a CommandConfig,
    acquire_fn: F,
) -> Result<State<'a, T>>
where
    T: Send + Sync + 'static,
    F: FnOnce(&'a mut T) -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    let timeout_duration = config
        .async_timeout_secs
        .map(std::time::Duration::from_secs);

    if let Some(timeout) = timeout_duration {
        log::debug!("DEBUG: acquire_service processing with lifetime parameter handling");
        tokio::time::timeout(timeout, acquire_fn(state.inner()))
            .await
            .map_err(|_| anyhow!("Service acquisition timed out"))?;
    }

    Ok(state)
}

// Using consolidated validation from rust-ai-ide-common
pub use validate_file_exists as validate_file_exists_legacy;

// validate_directory_exists removed - use the one imported from rust_ai_ide_common

/// Validate file size is within limits (legacy wrapper for String error type)
pub fn validate_file_size(
    content: &[u8],
    max_size_kb: usize,
    operation: &str,
) -> Result<(), String> {
    match validate_file_size_content(content, max_size_kb, operation) {
        Ok(()) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

// validate_path_not_excluded removed - use the one imported from rust_ai_ide_common

/// Execute command with standard logging and error handling
#[macro_export]
macro_rules! execute_command {
    ($command_name:expr, $config:expr, $closure:expr) => {{
        if $config.enable_logging {
            log::log!($config.log_level, "[{}] Executing command", $command_name);
        }

        let result = $closure();

        if $config.enable_logging {
            match &result {
                Ok(_) => log::log!(
                    $config.log_level,
                    "[{}] Command completed successfully",
                    $command_name
                ),
                Err(e) => log::error!("[{}] Command failed: {}", $command_name, e),
            }
        }

        result
    }};
}

/// Standard async command template macro
#[macro_export]
macro_rules! tauri_command_template {
    (
        $command_name:ident,
        $async_fn:block,
        service = $service_type:ty,
        state = $state_ident:ident,
        config = $config:expr
    ) => {
        #[tauri::command]
        pub async fn $command_name(
            $state_ident: State<'_, $service_type>,
            $($arg:ident: $arg_type:ty),*
        ) -> Result<String, String> {
            execute_command!(stringify!($command_name), &$config, async move || {
                $async_fn
            })
        }
    };
}

/// Command template with result return type
#[macro_export]
macro_rules! tauri_command_template_with_result {
    (
        $command_name:ident,
        $return_type:ty,
        $async_fn:block,
        service = $service_type:ty,
        state = $state_ident:ident,
        config = $config:expr
    ) => {
        #[tauri::command]
        pub async fn $command_name(
            $state_ident: State<'_, $service_type>,
            $($arg:ident: $arg_type:ty),*
        ) -> Result<$return_type, String> {
            execute_command!(stringify!($command_name), &$config, async move || {
                $async_fn
            })
        }
    };
}

/// Service acquisition and error handling macro
#[macro_export]
macro_rules! acquire_service_and_execute {
    (
        $service_state:expr,
        $service_type:ty,
        $closure:block
    ) => {{
        let service_guard = $service_state.lock().await;
        let service = service_guard.as_ref().ok_or(format_command_error(
            "Service not initialized",
            stringify!($service_type),
        ))?;

        $closure
    }};
}

/// Validation helper macro
#[macro_export]
macro_rules! validate_commands {
    ($($validation:stmt;)*) => {{
        let mut errors = Vec::new();
        $(
            match $validation {
                Ok(_) => {},
                Err(e) => errors.push(e),
            }
        )*

        if !errors.is_empty() {
            return Err(errors.join("; "));
        }
    }};
}

/// Helper for executing operations with retries
pub async fn execute_with_retry<T, F, Fut>(
    mut operation: F,
    max_retries: usize,
    operation_name: &str,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut last_error = None;

    for attempt in 0..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries => {
                log::warn!(
                    "Attempt {} of {} for {} failed: {}",
                    attempt + 1,
                    max_retries + 1,
                    operation_name,
                    e
                );
                last_error = Some(e);
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    100 * (attempt + 1) as u64,
                ))
                .await;
            }
            Err(e) => {
                log::error!(
                    "{} failed after {} attempts: {}",
                    operation_name,
                    max_retries + 1,
                    e
                );
                return Err(e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow!("Unknown error during retry")))
}

/// Background task execution helper
pub fn spawn_background_task<F>(task: F, task_name: &str) -> String
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    let task_id = format!("bg_{}_{}", task_name, uuid::Uuid::new_v4());
    log::info!("Spawning background task: {}", task_id);

    let value = task_id.clone();
    tokio::spawn(async move {
        if let Err(e) = tokio::time::timeout(
            std::time::Duration::from_secs(3600), // 1 hour timeout
            task,
        )
        .await
        {
            log::error!("Background task {} failed or timed out: {}", value, e);
        }
    });

    task_id
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_validate_file_exists() {
        let path = PathBuf::from("Cargo.toml");
        let result = validate_file_exists(&path, "test operation");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_size() {
        let content = b"Hello, world!".as_slice();
        let result = validate_file_size(content, 1, "test operation");
        assert!(result.is_ok());

        let large_content = vec![0; 2049]; // > 2KB
        let result = validate_file_size(&large_content, 2, "test operation");
        assert!(result.is_err());
    }
}

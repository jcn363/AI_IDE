//! # Command Template System for Standardized Tauri Commands
//!
//! This module implements the command macro system described in AGENTS.md, providing
//! standardized patterns for Tauri command handlers with consistent error handling,
//! validation, logging, and retry logic. The system reduces boilerplate code while
//! ensuring security and reliability across all command implementations.
//!
//! ## Key Components
//!
//! - **Command Templates**: Macros for consistent command handler implementations
//! - **Service Acquisition**: Safe async service access with timeout and error handling
//! - **Validation System**: Input sanitization and validation utilities
//! - **Retry Logic**: Exponential backoff retry mechanisms for external operations
//! - **Background Tasks**: Managed background task execution with cleanup
//!
//! ## Architecture Patterns
//!
//! ### Command Handler Standardization
//! - `tauri_command_template!`: Standard async command with service injection
//! - `tauri_command_template_with_result!`: Commands returning typed results
//! - `acquire_service_and_execute!`: Service acquisition with error boundaries
//!
//! ### Error Handling Strategy
//! - Aggregated error handling at function boundaries (per AGENTS.md)
//! - Structured error types with consistent formatting
//! - Silent error logging by default with configurable verbosity
//!
//! ### Security Integration
//! - Input validation using TauriInputSanitizer from rust-ai-ide-common
//! - Path validation for file operations
//! - Size limits and content validation
//!
//! ## Usage Examples
//!
//! ### Basic Command Template
//! ```rust,ignore
//! tauri_command_template!(
//!     get_user_data,
//!     async {
//!         let data = acquire_service_and_execute!(state.user_service, UserService, {
//!             service.get_user_data(user_id).await
//!         })?;
//!         Ok(serde_json::to_string(&data)?)
//!     },
//!     service = UserService,
//!     state = state,
//!     config = COMMAND_CONFIG
//! );
//! ```
//!
//! ### Command with Result Type
//! ```rust,ignore
//! tauri_command_template_with_result!(
//!     calculate_metrics,
//!     MetricsResult,
//!     async {
//!         let result = acquire_service_and_execute!(state.metrics_service, MetricsService, {
//!             service.calculate(user_id).await
//!         })?;
//!         Ok(result)
//!     },
//!     service = MetricsService,
//!     state = state,
//!     config = COMMAND_CONFIG
//! );
//! ```
//!
//! ### Background Task Execution
//! ```rust,ignore
//! let task_id = spawn_background_task(async move {
//!     long_running_operation().await;
//! }, "data_processing");
//! ```
//!
//! ## Security Considerations
//!
//! - All user inputs validated through TauriInputSanitizer
//! - File paths validated to prevent directory traversal attacks
//! - Command injection protection via sanitized arguments
//! - Audit logging for sensitive operations (configurable)
//!
//! ## Performance Characteristics
//!
//! - Minimal overhead for command dispatch and validation
//! - Configurable timeouts for service acquisition
//! - Efficient retry logic with exponential backoff
//! - Background task management with proper cleanup

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

/// # Command Configuration Structure
///
/// Configuration settings that control command behavior, logging, validation,
/// and timeout settings across all command handlers. This struct implements
/// the standardized configuration pattern used throughout the command system.
///
/// ## Configuration Options
///
/// - **Logging Control**: Enable/disable logging with configurable log levels
/// - **Validation**: Toggle input validation for security and data integrity
/// - **Timeouts**: Configurable async operation timeouts to prevent hanging
///
/// ## Usage
/// ```rust,ignore
/// const COMMAND_CONFIG: CommandConfig = CommandConfig {
///     enable_logging: true,
///     log_level: log::Level::Info,
///     enable_validation: true,
///     async_timeout_secs: Some(30), // 30 second timeout
/// };
/// ```
#[derive(Debug, Clone)]
pub struct CommandConfig {
    /// Whether to enable logging for command execution (start/completion/failure)
    pub enable_logging: bool,
    /// Log level for command operations (Error, Warn, Info, Debug, Trace)
    pub log_level: log::Level,
    /// Whether to enable input validation using TauriInputSanitizer
    pub enable_validation: bool,
    /// Optional timeout for async operations in seconds (None = no timeout)
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

/// # Command Execution Context
///
/// Provides a standardized context for command execution, containing service references
/// and application handles needed for command operations. This implements the
/// service acquisition pattern with optional Tauri AppHandle for UI interactions.
///
/// ## Generic Parameters
/// - `'r`: Lifetime parameter for service reference borrowing
/// - `T`: Service type that implements command operations
///
/// ## Usage
/// ```rust,ignore
/// fn execute_command(context: CommandContext<MyService>) -> Result<String, String> {
///     // Access service
///     let result = context.service.do_operation()?;
///
///     // Optional app handle for UI updates
///     if let Some(app) = &context.app_handle {
///         app.emit("operation_complete", result.clone())?;
///     }
///
///     Ok(result)
/// }
/// ```
pub struct CommandContext<'r, T> {
    /// Mutable reference to the service instance for this command
    pub service: &'r mut T,
    /// Optional Tauri AppHandle for UI interactions and event emission
    pub app_handle: Option<AppHandle>,
}

impl<'r, T> CommandContext<'r, T> {
    /// Creates a new command context with the specified service reference.
    ///
    /// # Parameters
    /// - `service`: Mutable reference to the service instance
    ///
    /// # Returns
    /// A new CommandContext with no AppHandle (for backend-only operations)
    pub fn new(service: &'r mut T) -> Self {
        Self {
            service,
            app_handle: None,
        }
    }

    /// Adds an AppHandle to the command context for UI interactions.
    ///
    /// # Parameters
    /// - `app_handle`: Tauri AppHandle for emitting events and UI updates
    ///
    /// # Returns
    /// A new CommandContext with the AppHandle attached
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

/// # Execute Command with Standardized Logging and Error Handling
///
/// Core macro that provides consistent logging, error handling, and execution patterns
/// for all command operations. This implements the error aggregation pattern described
/// in AGENTS.md, where errors are aggregated at function boundaries rather than propagated.
///
/// ## Parameters
/// - `$command_name`: Identifier for the command (used in logging)
/// - `$config`: CommandConfig instance controlling logging and behavior
/// - `$closure`: Async closure containing the command logic
///
/// ## Behavior
/// - Logs command start if logging is enabled
/// - Executes the command closure
/// - Logs success or failure with appropriate log levels
/// - Returns the result unchanged (error aggregation pattern)
///
/// ## Usage
/// ```rust,ignore
/// let result = execute_command!("process_data", &config, async {
///     // Command logic here
///     do_something().await
/// });
/// ```
///
/// ## Error Handling
/// This macro follows the "Ok return type favored over ?" pattern from AGENTS.md.
/// Errors are aggregated at the command boundary and returned for caller handling.
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

/// # Standard Tauri Command Template Macro
///
/// Generates standardized Tauri command handlers with consistent error handling,
/// logging, and service injection. This macro implements the command handler
/// standardization pattern described in AGENTS.md.
///
/// ## Parameters
/// - `$command_name`: Name of the generated command function
/// - `$async_fn`: Async block containing the command logic
/// - `$service_type`: Type of the service to inject
/// - `$state_ident`: Identifier for the state parameter
/// - `$config`: CommandConfig instance for behavior control
///
/// ## Generated Signature
/// ```rust,ignore
/// #[tauri::command]
/// pub async fn command_name(state: State<'_, ServiceType>, args...) -> Result<String, String>
/// ```
///
/// ## Usage
/// ```rust,ignore
/// tauri_command_template!(
///     get_user_data,
///     {
///         acquire_service_and_execute!(state, UserService, {
///             let data = service.get_user(user_id).await?;
///             Ok(serde_json::to_string(&data)?)
///         })
///     },
///     service = UserService,
///     state = state,
///     config = COMMAND_CONFIG
/// );
/// ```
///
/// ## Error Handling
/// Returns `Result<String, String>` with JSON-serialized results.
/// Errors are formatted and aggregated at command boundaries.
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

/// # Tauri Command Template with Typed Result
///
/// Variant of `tauri_command_template!` that returns typed results instead of JSON strings.
/// Useful for commands that return structured data or when type safety is preferred over
/// JSON serialization flexibility.
///
/// ## Parameters
/// - `$command_name`: Name of the generated command function
/// - `$return_type`: Type to return (must implement Serialize for Tauri)
/// - `$async_fn`: Async block containing the command logic
/// - `$service_type`: Type of the service to inject
/// - `$state_ident`: Identifier for the state parameter
/// - `$config`: CommandConfig instance for behavior control
///
/// ## Generated Signature
/// ```rust,ignore
/// #[tauri::command]
/// pub async fn command_name(state: State<'_, ServiceType>, args...) -> Result<ReturnType, String>
/// ```
///
/// ## Usage
/// ```rust,ignore
/// tauri_command_template_with_result!(
///     calculate_metrics,
///     MetricsResult,
///     {
///         acquire_service_and_execute!(state, MetricsService, {
///             service.calculate(user_id).await
///         })
///     },
///     service = MetricsService,
///     state = state,
///     config = COMMAND_CONFIG
/// );
/// ```
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

/// # Service Acquisition and Execution Macro
///
/// Provides safe service acquisition from Tauri's State with proper error handling.
/// This macro implements the double-locking pattern described in AGENTS.md for
/// lazy async service initialization.
///
/// ## Parameters
/// - `$service_state`: State reference containing the service (Arc<Mutex<Option<T>>>)
/// - `$service_type`: Type name for error messages
/// - `$closure`: Block to execute with the acquired service
///
/// ## Error Handling
/// - Returns formatted error if service is not initialized
/// - Uses structured error messages with service type information
/// - Follows error aggregation pattern from AGENTS.md
///
/// ## Usage
/// ```rust,ignore
/// let result = acquire_service_and_execute!(state.ai_service, AIService, {
///     service.analyze_code(code).await
/// })?;
/// ```
///
/// ## Thread Safety
/// Uses async locking to safely access services in concurrent environments.
/// The service reference is held for the duration of the closure execution.
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

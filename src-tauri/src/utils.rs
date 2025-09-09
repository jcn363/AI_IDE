use crate::errors::{IDEError, IDEResult};
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};
use std::ffi::OsStr;
use tokio::sync::broadcast;
use tokio::signal;
use tokio::time::{Duration, interval};
use rust_ai_ide_lsp::AIService;

// Implement Clone for AIService since it's not Clone in external crate
impl Clone for AIService {
    fn clone(&self) -> Self {
        AIService {} // Placeholder implementation for empty struct
    }
}

// Note: Removing std::process::Command import, now using unified shell utilities

const LEARNING_DB_FILENAME: &str = "ai_learning.db";
const DIAGNOSTIC_CACHE_SIZE: usize = 1000;
const EXPLANATION_CACHE_SIZE: usize = 500;

/// Get the model path from environment or default
pub fn get_model_path() -> String {
    std::env::var("AI_MODEL_PATH").unwrap_or_else(|_| "./models/ai-model.gguf".to_string())
}

/// Get AI endpoint from environment or default
pub fn get_ai_endpoint() -> String {
    std::env::var("AI_ENDPOINT").unwrap_or_else(|_| "http://localhost:11434/v1/completions".to_string())
}

use std::process::{Command, Stdio};

/// Unified wrapper for running git commands - consolidated to single location
pub fn git_run(dir: &str, args: &[&str]) -> IDEResult<(String, String)> {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| IDEError::CommandExecution(format!("Failed to run git: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(IDEError::CommandExecution(format!("git {:?} failed: {}", args, stderr)));
    }

    Ok((stdout, stderr))
}

/// Background cache cleanup task
pub async fn initialize_cache_cleanup_task(
    diagnostic_cache_state: &crate::DiagnosticCacheState,
    explanation_cache_state: &crate::ExplanationCacheState,
) -> IDEResult<()> {
    log::info!("Starting cache cleanup task");

    let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);
    let mut cleanup_interval = interval(Duration::from_secs(300)); // 5 minutes

    // Setup signal handling for graceful shutdown
    tokio::spawn({
        let shutdown_tx_signal = shutdown_tx.clone();
        async move {
            let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to install SIGTERM handler");
            let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())
                .expect("Failed to install SIGINT handler");

            tokio::select! {
                _ = sigterm.recv() => {
                    log::info!("Received SIGTERM, initiating graceful shutdown");
                }
                _ = sigint.recv() => {
                    log::info!("Received SIGINT, initiating graceful shutdown");
                }
            }

            if let Err(e) = shutdown_tx_signal.send(()) {
                log::warn!("Failed to send shutdown signal: {}. Shutdown may not be properly coordinated.", e);
            } else {
                log::debug!("Shutdown signal sent successfully to listeners");
            }
        }
    });

    loop {
        tokio::select! {
            _ = cleanup_interval.tick() => {
                // Cleanup diagnostic cache
                {
                    let mut diagnostic_cache = diagnostic_cache_state.write().await;
                    diagnostic_cache.cleanup();
                    log::debug!("Diagnostic cache cleanup completed");
                }

                // Cleanup explanation cache
                {
                    let mut explanation_cache = explanation_cache_state.write().await;
                    explanation_cache.cleanup();
                    log::debug!("Explanation cache cleanup completed");
                }

                log::debug!("Cache cleanup cycle completed");
            }
            _ = shutdown_rx.recv() => {
                log::info!("Cache cleanup task received shutdown signal, exiting gracefully");
                break;
            }
        }
    }

    Ok(())
}

/// Initialize AI service on startup
pub async fn initialize_ai_service_on_startup(
    ai_service_state: &crate::AIServiceState
) -> IDEResult<()> {
    log::info!("Initializing AI service on startup");

    // Create default AI service configuration with environment-configurable endpoints
    let default_provider = rust_ai_ide_lsp::AIProvider::Local {
        model_path: get_model_path(),
    };

    let mut service = AIService {
        // Placeholder service creation - replace with proper initialization when AIService is implemented
    };

    // Initialize learning system if enabled
    let db_path = Some(std::path::PathBuf::from("ai_learning.db"));
    if let Err(e) = service.initialize_learning_system(db_path).await {
        log::warn!("Failed to initialize learning system during startup: {}", e);
    } else {
        log::info!("Learning system initialized successfully");
    }

    let mut ai_service_guard = ai_service_state.0.lock().await;
    *ai_service_guard = Some(service);

    log::info!("AI service initialized successfully on startup");

    Ok(())
}

pub async fn get_or_create_ai_service(
    ai_service_state: &crate::AIServiceState
) -> IDEResult<rust_ai_ide_lsp::AIService> {
    let mut ai_service_guard = ai_service_state.0.lock().await;
    if ai_service_guard.is_none() {
        log::info!("Creating AI service instance");
        let default_provider = rust_ai_ide_lsp::AIProvider::Local {
            model_path: get_model_path(),
        };
        let service = AIService {
            // Placeholder service creation - replace with proper initialization when AIService is implemented
        };

        // Initialize learning system if enabled
        let config_dir = dirs::config_dir()
            .ok_or_else(|| IDEError::Configuration("Unable to get config directory".to_string()))?
            .join("rust-ai-ide");

        if let Err(e) = std::fs::create_dir_all(&config_dir) {
            log::error!("Failed to create config directory '{}': {}. This may be due to permission issues or insufficient disk space.", config_dir.display(), e);
            return Err(IDEError::Configuration(format!("Failed to create config directory '{}': {}. Check permissions and available disk space.", config_dir.display(), e)));
        }

        let db_path = config_dir.join(LEARNING_DB_FILENAME);

        match service.initialize_learning_system(Some(db_path)).await {
            Ok(_) => log::info!("Learning system initialized successfully"),
            Err(e) => log::warn!("Failed to initialize learning system: {:?}", e),
        }

        *ai_service_guard = Some(service);
    }

    if let Some(ref service) = *ai_service_guard {
        Ok(service.clone())
    } else {
        Err(IDEError::AIService("Failed to create AI service".to_string()))
    }
}

/// Utility for reading files with size limits
pub fn read_file_with_limit(path: &str, max_size: u64) -> IDEResult<String> {
    crate::errors::validate_path_security(path)?;
    crate::errors::validate_file_size(path, max_size)?;

    std::fs::read_to_string(path).map_err(|e| IDEError::FileOperation(format!("Failed to read file: {}", e)))
}

/// Path existence and type checks
pub fn ensure_directory_exists<P: AsRef<Path>>(path: P) -> IDEResult<()> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(IDEError::FileOperation(format!("Directory does not exist: {}", path.display())));
    }
    if !path.is_dir() {
        return Err(IDEError::FileOperation(format!("Path exists but is not a directory: {}", path.display())));
    }
    Ok(())
}

pub fn validate_rust_project<P: AsRef<Path>>(path: P) -> IDEResult<()> {
    let path = path.as_ref();
    ensure_directory_exists(path)?;
    if !path.join("Cargo.toml").exists() {
        return Err(IDEError::Cargo("Not a valid Rust project - Cargo.toml not found".to_string()));
    }
    Ok(())
}

/// Gets the current timestamp in seconds since Unix epoch
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Validates if a path is within a base directory to prevent directory traversal
pub fn is_path_within_directory<P: AsRef<Path>>(path: P, base: P) -> io::Result<bool> {
    let path = fs::canonicalize(path)?;
    let base = fs::canonicalize(base)?;
    
    for ancestor in path.ancestors() {
        if ancestor == base {
            return Ok(true);
        }
    }
    
    Ok(false)
}

/// Gets the file extension from a path, if any
pub fn get_file_extension<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .extension()
        .and_then(OsStr::to_str)
        .map(|s| s.to_lowercase())
}

/// Creates a backup of a file with a timestamp
pub fn create_backup<P: AsRef<Path>>(file_path: P) -> IDEResult<PathBuf> {
    let path = file_path.as_ref();
    let backup_path = path.with_extension(format!("bak.{}", current_timestamp()));
    
    fs::copy(path, &backup_path)
        .map_err(|e| IDEError::FileOperation(format!("Failed to create backup of {}: {}", path.display(), e)))?;
    
    Ok(backup_path)
}

/// Recursively finds files with a specific extension in a directory
pub fn find_files_by_extension<P: AsRef<Path>>(
    dir: P,
    extension: &str,
) -> io::Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            result.extend(find_files_by_extension(&path, extension)?);
        } else if path.extension()
            .and_then(OsStr::to_str)
            .map_or(false, |ext| ext.eq_ignore_ascii_case(extension)) {
            result.push(path);
        }
    }
    
    Ok(result)
}

/// Calculates the size of a directory recursively
pub fn calculate_directory_size<P: AsRef<Path>>(path: P) -> io::Result<u64> {
    let mut total_size = 0;
    
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        
        if metadata.is_dir() {
            total_size += calculate_directory_size(entry.path())?;
        } else {
            total_size += metadata.len();
        }
    }
    
    Ok(total_size)
}

/// Executes a command with timeout
pub async fn execute_with_timeout<F, T, E>(
    future: F,
    duration: Duration,
    error: E,
) -> Result<T, IDEError>
where
    F: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    match tokio::time::timeout(duration, future).await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(e)) => Err(IDEError::CommandExecution(e.to_string())),
        Err(_) => Err(IDEError::CommandExecution("Command timed out".to_string())),
    }
}

// ==================== SECURITY HARDENING UTILITIES ====================

/// Sanitize Tauri command inputs with comprehensive security checks
pub mod input_sanitization {

    use crate::errors::{IDEError, IDEResult};
    use std::path::Path;

    /// Comprehensive input sanitizer for Tauri commands
    pub struct TauriInputSanitizer {
        pub max_string_length: usize,
        pub max_path_length: usize,
        pub allowed_extensions: Vec<String>,
    }

    impl Default for TauriInputSanitizer {
        fn default() -> Self {
            Self {
                max_string_length: 50*1024, // 50KB
                max_path_length: 4096,
                allowed_extensions: vec![
                    "rs".to_string(),
                    "toml".to_string(),
                    "json".to_string(),
                    "txt".to_string(),
                    "md".to_string(),
                    "js".to_string(),
                    "ts".to_string(),
                    "html".to_string(),
                    "css".to_string(),
                ],
            }
        }
    }

    impl TauriInputSanitizer {
        /// Sanitize file path with security checks
        pub fn sanitize_file_path(&self, input_path: &str) -> IDEResult<String> {
            if input_path.is_empty() {
                return Err(IDEError::Validation("File path cannot be empty".to_string()));
            }

            if input_path.len() > self.max_path_length {
                return Err(IDEError::Validation("File path too long".to_string()));
            }

            // Remove path traversal attempts
            let clean_path = input_path
                .replace("..", "")
                .replace("\\", "/") // Normalize to forward slashes
                .trim_start_matches('/')
                .to_string();

            // Security checks for dangerous patterns
            let dangerous_patterns = ["<", ">", "|", "*", "?", "\"", "'", "`", "\0"];

            for pattern in &dangerous_patterns {
                if clean_path.contains(pattern) {
                    return Err(IDEError::Validation(format!("Dangerous pattern '{}' detected in path", pattern)));
                }
            }

            Ok(clean_path)
        }

        /// Sanitize general string input for API endpoints
        pub fn sanitize_api_string(&self, input: &str) -> IDEResult<String> {
            if input.len() > self.max_string_length {
                return Err(IDEError::Validation("Input too long".to_string()));
            }

            // Remove null bytes
            let clean = input.replace('\0', "");

            // XSS protection - basic script tags
            let clean = clean
                .replace("<script", "<script")
                .replace("</script>", "</script>");

            // Remove newlines and control characters (except spaces and tabs)
            let clean = clean
                .chars()
                .filter(|&c| c.is_whitespace() || !c.is_control())
                .collect::<String>()
                .replace('\n', "")
                .replace('\r', "");

            Ok(clean.trim().to_string())
        }

        /// Validate file extension for security
        pub fn validate_file_extension(&self, path: &str) -> IDEResult<()> {
            if let Some(ext) = Path::new(path).extension() {
                if let Some(ext_str) = ext.to_str() {
                    let ext_lower = ext_str.to_lowercase();
                    if !self.allowed_extensions.contains(&ext_lower) {
                        return Err(IDEError::Validation(format!("File extension '{}' not allowed", ext_upper)));
                    }
                }
            }
            Ok(())
        }

        /// Sanitize and validate command line arguments
        pub fn sanitize_command_args(&self, args: &[String]) -> IDEResult<Vec<String>> {
            let mut sanitized = Vec::new();

            for arg in args {
                if arg.len() > self.max_string_length / 16 { // Shorter limit for args
                    return Err(IDEError::Validation("Command argument too long".to_string()));
                }

                // Remove shell injection patterns
                let clean_arg = arg
                    .replace(";", "_")
                    .replace("|", "_")
                    .replace("&", "_")
                    .replace("`", "_")
                    .replace("$", "_");

                sanitized.push(clean_arg);
            }

            Ok(sanitized)
        }
    }

    /// Global sanitizer function for Tauri commands
    pub fn get_tauri_sanitizer() -> &'static TauriInputSanitizer {
        static TAURI_SANITIZER: std::sync::OnceLock<TauriInputSanitizer> = std::sync::OnceLock::new();
        TAURI_SANITIZER.get_or_init(TauriInputSanitizer::default)
    }

    /// Convenience function for sanitizing Tauri inputs
    pub fn sanitize_tauri_input(input: &str, input_type: SanitizeType) -> IDEResult<String> {
        let sanitizer = get_tauri_sanitizer();
        match input_type {
            SanitizeType::FilePath => sanitizer.sanitize_file_path(input),
            SanitizeType::ApiString => sanitizer.sanitize_api_string(input),
        }
    }

    /// Types of sanitization
    pub enum SanitizeType {
        FilePath,
        ApiString,
    }
}

/// Re-export common security functions
pub use input_sanitization::{TauriInputSanitizer, sanitize_tauri_input, SanitizeType};
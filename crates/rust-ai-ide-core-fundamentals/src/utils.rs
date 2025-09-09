//! Shared Utility Functions and Macros
//! =====================================
//!
//! This module provides shared utility functions for validation, sanitization,
//! type conversion, and common operations used across the Rust AI IDE.

use crate::error::{IDEError, IDEResult};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Sanitization utilities
pub mod sanitization {
    use super::*;

    /// Sanitize user input for file paths
    pub fn sanitize_path_input(input: &str) -> IDEResult<String> {
        // Remove leading/trailing whitespace
        let trimmed = input.trim();

        // Check for path traversal attempts
        if trimmed.contains("..") || trimmed.contains("\\") {
            return Err(IDEError::Validation(
                "Path contains traversal sequences (..) or backslashes".to_string(),
            ));
        }

        // Validate length
        if trimmed.is_empty() {
            return Err(IDEError::Validation("Path cannot be empty".to_string()));
        }

        // Maximum path length check
        const MAX_PATH_LENGTH: usize = 4096;
        if trimmed.len() > MAX_PATH_LENGTH {
            return Err(IDEError::Validation(format!(
                "Path exceeds maximum length of {} characters",
                MAX_PATH_LENGTH
            )));
        }

        Ok(trimmed.to_string())
    }

    /// Sanitize user input for generic strings
    pub fn sanitize_string_input(
        input: &str,
        max_len: usize,
        allow_special_chars: bool,
    ) -> IDEResult<String> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Err(IDEError::Validation("Input cannot be empty".to_string()));
        }

        if trimmed.len() > max_len {
            return Err(IDEError::Validation(format!(
                "Input exceeds maximum length of {} characters",
                max_len
            )));
        }

        if !allow_special_chars
            && trimmed
                .chars()
                .any(|c| !c.is_alphanumeric() && c != '_' && c != '-')
        {
            return Err(IDEError::Validation(
                "Input contains invalid characters".to_string(),
            ));
        }

        Ok(trimmed.to_string())
    }

    /// Sanitize code input for AI processing
    pub fn sanitize_code_input(code: &str, _language: &str) -> IDEResult<String> {
        // Remove null bytes
        let sanitized = code.replace('\0', "");

        if sanitized.is_empty() {
            return Err(IDEError::Validation(
                "Code input cannot be empty".to_string(),
            ));
        }

        // Language-specific validation could be added here

        Ok(sanitized)
    }
}

/// Validation utilities
pub mod validation {
    use super::*;

    static EMAIL_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap());

    /// Validate an email address
    pub fn validate_email(email: &str) -> IDEResult<()> {
        if !EMAIL_REGEX.is_match(email) {
            return Err(IDEError::Validation(
                "Invalid email address format".to_string(),
            ));
        }
        Ok(())
    }

    /// Validate a file path
    pub fn validate_file_path(path: &Path, allow_absolute: bool) -> IDEResult<()> {
        if !allow_absolute && path.is_absolute() {
            return Err(IDEError::Validation(
                "Absolute paths are not allowed".to_string(),
            ));
        }

        // Check for path traversal
        let path_str = path.to_string_lossy();
        if path_str.contains("..") {
            return Err(IDEError::Validation("Path traversal detected".to_string()));
        }

        Ok(())
    }

    /// Validate a project path
    pub fn validate_project_path(project_path: &Path) -> IDEResult<PathBuf> {
        if !project_path.exists() {
            return Err(IDEError::Validation(format!(
                "Project path does not exist: {:?}",
                project_path
            )));
        }

        if !project_path.is_dir() {
            return Err(IDEError::Validation(format!(
                "Project path is not a directory: {:?}",
                project_path
            )));
        }

        let cargo_toml = project_path.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Err(IDEError::Validation(
                "Not a valid Rust project - Cargo.toml not found".to_string(),
            ));
        }

        Ok(cargo_toml)
    }

    /// Validate URL format
    pub fn validate_url(url: &str) -> IDEResult<()> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(IDEError::Validation(
                "URL must start with http:// or https://".to_string(),
            ));
        }

        // Basic length check
        if url.len() > 2048 {
            return Err(IDEError::Validation(
                "URL exceeds maximum length".to_string(),
            ));
        }

        Ok(())
    }
}

/// Type conversion utilities
pub mod conversion {
    use super::*;
    use std::str::FromStr;

    /// Convert a string to a value of type T
    pub fn parse_from_string<T>(s: &str) -> IDEResult<T>
    where
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Display,
    {
        s.parse::<T>().map_err(|e| {
            IDEError::Generic(format!(
                "Failed to parse '{}' as {}: {}",
                s,
                std::any::type_name::<T>(),
                e
            ))
        })
    }

    /// Try to convert a value to another type
    pub fn try_convert<V1, V2>(value: &V1) -> IDEResult<V2>
    where
        V2: TryFrom<V1>,
        <V2 as TryFrom<V1>>::Error: std::fmt::Display,
        V1: Clone,
    {
        V2::try_from(value.clone())
            .map_err(|e| IDEError::Generic(format!("Failed to convert value: {}", e)))
    }

    /// Convert hashmap keys to camel case
    pub fn convert_keys_to_camel_case(map: &mut serde_json::Map<String, serde_json::Value>) {
        let keys_to_convert: Vec<String> =
            map.keys().filter(|k| k.contains('_')).cloned().collect();

        for key in keys_to_convert {
            if let Some(value) = map.remove(&key) {
                let camel_key = to_camel_case(&key);
                map.insert(camel_key, value);
            }
        }
    }

    /// Convert string from snake_case to camelCase
    pub fn to_camel_case(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut capitalize_next = false;

        for ch in input.chars() {
            if ch == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(ch.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Convert value to JSON string
    pub fn to_json_string<T>(value: &T) -> IDEResult<String>
    where
        T: serde::Serialize,
    {
        serde_json::to_string_pretty(value)
            .map_err(|e| IDEError::Generic(format!("JSON serialization error: {}", e)))
    }

    /// Parse from JSON string
    pub fn from_json_string<T>(json: &str) -> IDEResult<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        serde_json::from_str(json)
            .map_err(|e| IDEError::Generic(format!("JSON deserialization error: {}", e)))
    }
}

/// Path manipulation utilities
pub mod path_utils {
    use super::*;

    /// Get relative path from base to target
    pub fn get_relative_path(base: &Path, target: &Path) -> IDEResult<PathBuf> {
        let base = base
            .canonicalize()
            .map_err(|e| IDEError::Generic(format!("Failed to canonicalize base path: {}", e)))?;

        let rel_path = target.strip_prefix(&base).map_err(|_| {
            IDEError::Generic(format!(
                "Target path {:?} is not under base path {:?}",
                target, base
            ))
        })?;

        Ok(rel_path.to_path_buf())
    }

    /// Ensure a directory exists, creating it if necessary
    pub fn ensure_directory_exists(dir: &Path) -> IDEResult<()> {
        if !dir.exists() {
            std::fs::create_dir_all(dir).map_err(|e| {
                IDEError::Generic(format!("Failed to create directory {:?}: {}", dir, e))
            })?;
        } else if !dir.is_dir() {
            return Err(IDEError::Generic(format!(
                "Path {:?} exists but is not a directory",
                dir
            )));
        }

        Ok(())
    }

    /// Get file extension - DEPRECATED: Use rust_ai_ide_shared_utils::get_extension instead
    #[deprecated(since = "0.2.0", note = "Use rust_ai_ide_shared_utils::get_extension")]
    pub fn get_extension(path: &Path) -> Option<&str> {
        rust_ai_ide_shared_utils::get_extension(path)
    }

    /// Check if path is a supported code file - DEPRECATED: Use rust_ai_ide_shared_utils::is_code_file instead
    #[deprecated(since = "0.2.0", note = "Use rust_ai_ide_shared_utils::is_code_file")]
    pub fn is_code_file(path: &Path) -> bool {
        rust_ai_ide_shared_utils::is_code_file(path)
    }

    /// Get the size of a file or directory
    pub fn get_size(path: &Path) -> IDEResult<u64> {
        if path.is_file() {
            let metadata = path
                .metadata()
                .map_err(|e| IDEError::Generic(format!("Failed to get file metadata: {}", e)))?;
            Ok(metadata.len())
        } else if path.is_dir() {
            let mut total_size = 0u64;
            for entry in std::fs::read_dir(path)
                .map_err(|e| IDEError::Generic(format!("Failed to read directory: {}", e)))?
            {
                let entry = entry.map_err(|e| {
                    IDEError::Generic(format!("Failed to read directory entry: {}", e))
                })?;

                let size = get_size(&entry.path())?;
                total_size = total_size.saturating_add(size);
            }
            Ok(total_size)
        } else {
            Ok(0)
        }
    }
}

/// Async utilities for common patterns
pub mod async_utils {
    use super::*;
    use tokio::time::{timeout, Duration};

    /// Timeout wrapper for operations
    pub async fn with_timeout<T, Fut>(future: Fut, timeout_duration: Duration) -> IDEResult<T>
    where
        Fut: std::future::Future<Output = IDEResult<T>>,
    {
        match timeout(timeout_duration, future).await {
            Ok(result) => result,
            Err(_) => Err(IDEError::Timeout(format!(
                "Operation timed out after {} seconds",
                timeout_duration.as_secs()
            ))),
        }
    }

    /// Retry with exponential backoff
    pub async fn retry_with_backoff<F, Fut, T>(
        mut operation: F,
        max_attempts: u32,
        base_delay_ms: u64,
        max_delay_ms: u64,
    ) -> IDEResult<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = IDEResult<T>>,
    {
        let mut attempt = 0;
        let mut last_error = None;

        while attempt < max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    attempt += 1;

                    if attempt >= max_attempts {
                        break;
                    }

                    let delay = std::cmp::min(base_delay_ms * (2u64.pow(attempt)), max_delay_ms);

                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }
            }
        }

        Err(last_error.unwrap())
    }
}

/// Performance utilities
pub mod perf_utils {
    use std::time::Instant;

    /// Simple performance timer
    pub struct Timer {
        start: Instant,
        label: String,
    }

    impl Timer {
        pub fn new(label: impl Into<String>) -> Self {
            Self {
                start: Instant::now(),
                label: label.into(),
            }
        }

        pub fn elapsed(&self) -> std::time::Duration {
            self.start.elapsed()
        }

        pub fn elapsed_ms(&self) -> f64 {
            self.elapsed().as_secs_f64() * 1000.0
        }

        pub fn log(&self) {
            log::debug!("Timer '{}' elapsed: {:.2}ms", self.label, self.elapsed_ms());
        }
    }

    /// Measure execution time of a function
    pub fn time_function<F, R>(label: &str, f: F) -> (R, std::time::Duration)
    where
        F: FnOnce() -> R,
    {
        let timer = Timer::new(label);
        let result = f();
        let duration = timer.elapsed();

        log::debug!(
            "Function '{}' took {:.2}ms",
            label,
            duration.as_secs_f64() * 1000.0
        );

        (result, duration)
    }
}

/// ID generation utilities
pub mod id_utils {
    use uuid::Uuid;

    /// Generate a new UUID as a simple string
    pub fn generate_uuid() -> String {
        Uuid::new_v4().simple().to_string()
    }

    /// Generate a UUID with the provided prefix
    pub fn generate_prefixed_id(prefix: &str) -> String {
        format!("{}_{}", prefix, generate_uuid())
    }

    /// Generate a timestamp-based ID
    pub fn generate_timestamp_id(prefix: &str) -> String {
        let timestamp = chrono::Utc::now().timestamp_millis();
        format!("{}_{}", prefix, timestamp)
    }
}

/// Configuration merge utilities
pub mod config_merge {
    use super::*;
    use serde_json::Value;

    /// Deep merge two JSON values
    pub fn deep_merge(base: &mut Value, other: Value) {
        match (&*base, other) {
            (Value::Object(_), Value::Object(other_map)) => {
                if let Value::Object(base_map) = base {
                    for (key, value) in other_map {
                        match base_map.get_mut(&key) {
                            Some(base_value) => deep_merge(base_value, value),
                            None => {
                                base_map.insert(key, value);
                            }
                        }
                    }
                }
            }
            (_base, other) => *base = other,
        }
    }

    /// Merge two configuration hashmaps
    pub fn merge_configs(
        base: &mut HashMap<String, serde_json::Value>,
        other: HashMap<String, serde_json::Value>,
    ) {
        for (key, value) in other {
            match base.get_mut(&key) {
                Some(existing) => {
                    if let Value::Object(other_obj) = value {
                        if let Value::Object(ref mut base_obj) = existing {
                            for (sub_key, sub_value) in other_obj {
                                base_obj.insert(sub_key, sub_value);
                            }
                        } else {
                            base.insert(key, Value::Object(other_obj));
                        }
                    } else {
                        base.insert(key, value);
                    }
                }
                None => {
                    base.insert(key, value);
                }
            }
        }
    }
}

/// Logging helpers with structured fields
pub mod log_utils {
    use log::Level;
    use std::collections::HashMap;

    /// Logger that supports structured logging with key-value metadata fields
    pub struct StructuredLogger {
        /// Additional metadata fields to include in log messages
        fields: HashMap<String, serde_json::Value>,
    }

    impl Default for StructuredLogger {
        fn default() -> Self {
            Self::new()
        }
    }

    impl StructuredLogger {
        /// Creates a new StructuredLogger with no fields
        pub fn new() -> Self {
            Self {
                fields: HashMap::new(),
            }
        }

        /// Adds a key-value field to be included in log messages
        pub fn with_field<K: Into<String>, V: Into<serde_json::Value>>(
            mut self,
            key: K,
            value: V,
        ) -> Self {
            self.fields.insert(key.into(), value.into());
            self
        }

        /// Logs a message at the specified level with structured fields
        pub fn log(&self, level: Level, message: &str) {
            let fields_str = serde_json::to_string(&self.fields).unwrap_or_default();
            match level {
                Level::Error => log::error!("{} | {}", message, fields_str),
                Level::Warn => log::warn!("{} | {}", message, fields_str),
                Level::Info => log::info!("{} | {}", message, fields_str),
                Level::Debug => log::debug!("{} | {}", message, fields_str),
                Level::Trace => log::trace!("{} | {}", message, fields_str),
            }
        }

        /// Logs an info-level message with structured fields
        pub fn info(&self, message: &str) {
            self.log(Level::Info, message);
        }

        /// Logs an error-level message with structured fields
        pub fn error(&self, message: &str) {
            self.log(Level::Error, message);
        }

        /// Logs a warning-level message with structured fields
        pub fn warn(&self, message: &str) {
            self.log(Level::Warn, message);
        }

        /// Logs a debug-level message with structured fields
        pub fn debug(&self, message: &str) {
            self.log(Level::Debug, message);
        }
    }

    /// Create a structured logger with common AI operation fields
    pub fn ai_logger(operation: &str, model: &str) -> StructuredLogger {
        StructuredLogger::new()
            .with_field("operation", operation)
            .with_field("model", model)
            .with_field("timestamp", chrono::Utc::now().timestamp_millis())
    }

    /// Create a structured logger for file operations
    pub fn file_logger(operation: &str, path: &str) -> StructuredLogger {
        StructuredLogger::new()
            .with_field("operation", operation)
            .with_field("path", path)
            .with_field("timestamp", chrono::Utc::now().timestamp_millis())
    }
}

/// System information utilities
pub mod system_info {
    use super::*;
    /// Get CPU count suitable for parallel operations
    pub fn get_optimal_thread_count() -> usize {
        let available_cores = num_cpus::get();
        // Leave at least one core for system operations
        if available_cores > 1 {
            available_cores - 1
        } else {
            available_cores
        }
    }

    /// Get available memory in MB
    pub fn get_available_memory_mb() -> IDEResult<u64> {
        let system = sysinfo::System::new_all();

        // Check if system information was successfully retrieved
        // (sysinfo may have limitations on some platforms)
        if system.available_memory() == 0 {
            return Err(IDEError::Generic(
                "Unable to get system memory information".to_string(),
            ));
        }

        Ok(system.available_memory() / 1024 / 1024)
    }

    /// Check if running in constrained memory environment
    pub fn is_memory_constrained(max_memory_mb: u64) -> IDEResult<bool> {
        let available = get_available_memory_mb()?;
        Ok(available < max_memory_mb * 2)
    }
}

/// Macro utilities for common patterns
pub mod macros {
    use super::*;

    // Note: These are utilities that could be used as macro helpers
    // but are implemented as functions for simplicity

    /// Create a new hashmap with provided key-value pairs
    pub fn hashmap<K: std::hash::Hash + Eq, V>(pairs: Vec<(K, V)>) -> HashMap<K, V> {
        pairs.into_iter().collect()
    }

    /// Create a vector from an iterator of results, collecting only successes
    pub fn collect_ok<T, E>(iter: impl Iterator<Item = Result<T, E>>) -> Vec<T> {
        iter.filter_map(|r| r.ok()).collect()
    }

    /// Create a vector from an iterator of results, collecting only errors
    pub fn collect_err<T, E>(iter: impl Iterator<Item = Result<T, E>>) -> Vec<E> {
        iter.filter_map(|r| r.err()).collect()
    }
}

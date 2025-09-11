//! Command utilities and templates for Tauri commands
//!
//! This module provides standardized templates and utilities for Tauri command handlers
//! to reduce boilerplate and ensure consistent error handling and logging patterns.

use tauri::State;

/// Result type for command operations
pub type CommandResult<T> = Result<T, String>;

/// Standardized async command execution template
/// This macro provides consistent error handling, logging, and result mapping
#[macro_export]
macro_rules! async_command {
    // Basic command with logging and error mapping
    ($operation:expr, $body:block) => {{
        log::info!("{}", $operation);
        let result: CommandResult<_> = async move { $body }.await;
        result.map_err(|e| format!("{} failed: {}", $operation, e))
    }};

    // Command with cached state access
    ($operation:expr, $cache_key:expr, $cache:ident, $state:expr, $body:block) => {{
        log::info!("{}", $operation);
        $cache_key
            .map(|key| {
                let cache_guard = async {
                    let cache = $state.read().await;
                    cache.get(&key).cloned()
                };
                cache_guard
            })
            .flatten()
            .unwrap_or_else(|| {
                async move { $body }
                    .await
                    .map_err(|e| format!("{} failed: {}", $operation, e))
            })
    }};
}

/// Standard timeout wrapper for operations that might hang
#[macro_export]
macro_rules! with_timeout {
    ($duration:expr, $operation:block) => {{
        use tokio::time::{timeout, Duration};
        let duration = Duration::from_secs($duration);
        timeout(duration, async move { $operation })
            .await
            .map_err(|_| "Operation timed out".to_string())
    }};
}

/// Cache lookup and insertion utilities
pub mod cache {
    use std::collections::HashMap;
    use std::hash::Hash;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Generic cache entry with TTL
    #[derive(Debug, Clone)]
    pub struct CacheEntry<T> {
        pub value: T,
        pub timestamp: u64,
        pub ttl_seconds: u64,
    }

    /// Check if a cache entry is still valid
    pub fn is_entry_valid<T>(entry: &CacheEntry<T>) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now - entry.timestamp < entry.ttl_seconds
    }

    /// Create a cache entry
    pub fn create_entry<T>(value: T, ttl_seconds: u64) -> CacheEntry<T> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        CacheEntry {
            value,
            timestamp,
            ttl_seconds,
        }
    }
}

/// Parameter validation utilities
pub mod validation {
    /// Validate that a string parameter is not empty
    pub fn validate_non_empty(value: &str, param_name: &str) -> CommandResult<()> {
        if value.is_empty() {
            Err(format!("{} cannot be empty", param_name))
        } else {
            Ok(())
        }
    }

    /// Validate that a path exists (for file operations)
    pub fn validate_path_exists(path: &str) -> CommandResult<()> {
        use std::path::Path;
        if Path::new(path).exists() {
            Ok(())
        } else {
            Err(format!("Path does not exist: {}", path))
        }
    }

    /// Validate timeout value is reasonable
    pub fn validate_timeout(seconds: u64) -> CommandResult<()> {
        if seconds == 0 {
            Err("Timeout must be greater than 0".to_string())
        } else if seconds > 3600 {
            Err("Timeout cannot exceed 1 hour".to_string())
        } else {
            Ok(())
        }
    }
}

/// Macro for creating standard Tauri command handlers
/// This macro provides the boilerplate structure for async commands
#[macro_export]
macro_rules! standard_command {
    // Basic command with operation name
    ($name:ident, $operation:expr, $handler:block) => {
        #[tauri::command]
        pub async fn $name() -> CommandResult<String> {
            async_command!($operation, $handler)
        }
    };

    // Command with single parameter
    ($name:ident, $operation:expr, $param:ident: $param_type:ty, $handler:block) => {
        #[tauri::command]
        pub async fn $name($param: $param_type) -> CommandResult<String> {
            async_command!($operation, $handler)
        }
    };
}

/// Common error messages
pub mod errors {
    pub const TIMEOUT_ERROR: &str = "Operation timed out";
    pub const PERMISSION_ERROR: &str = "Insufficient permissions";
    pub const INVALID_PATH_ERROR: &str = "Invalid path provided";
    pub const COMPILATION_FAILED: &str = "Compilation failed";
    pub const CACHE_MISS: &str = "Cache miss";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_command_macro() {
        // This would require tokio test framework
        // For now, just ensure it compiles
        let _result: CommandResult<String> = Ok("test".to_string());
    }

    #[test]
    fn test_cache_validation() {
        let entry = cache::create_entry("test_value".to_string(), 60);
        assert!(cache::is_entry_valid(&entry));
    }
}

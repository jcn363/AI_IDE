//! Error types for the lazy loading infrastructure

use std::fmt;

use thiserror::Error;

/// Errors that can occur during lazy loading operations
#[derive(Error, Debug, Clone)]
pub enum LazyLoadingError {
    #[error("Component initialization failed: {component_name} - {reason}")]
    InitializationFailed {
        component_name: String,
        reason:         String,
    },

    #[error("Component loading timeout after {timeout_seconds} seconds: {component_name}")]
    LoadingTimeout {
        component_name:  String,
        timeout_seconds: u64,
    },

    #[error("Component not found: {component_name}")]
    ComponentNotFound { component_name: String },

    #[error("Memory pool exhausted: {pool_type} - requested {requested}, available {available}")]
    MemoryPoolExhausted {
        pool_type: String,
        requested: usize,
        available: usize,
    },

    #[error("Concurrent load limit exceeded: {current} / {max}")]
    ConcurrentLoadLimitExceeded { current: usize, max: usize },

    #[error("Component unload failed: {component_name} - {reason}")]
    UnloadFailed {
        component_name: String,
        reason:         String,
    },

    #[error("Memory limit exceeded: used {used_bytes}, limit {limit_bytes}")]
    MemoryLimitExceeded {
        used_bytes:  usize,
        limit_bytes: usize,
    },

    #[error("Invalid configuration: {field} - {reason}")]
    InvalidConfiguration { field: String, reason: String },

    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl LazyLoadingError {
    /// Create an initialization error
    pub fn initialization_failed(component_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InitializationFailed {
            component_name: component_name.into(),
            reason:         reason.into(),
        }
    }

    /// Create a loading timeout error
    pub fn loading_timeout(component_name: impl Into<String>, timeout_seconds: u64) -> Self {
        Self::LoadingTimeout {
            component_name: component_name.into(),
            timeout_seconds,
        }
    }

    /// Create a component not found error
    pub fn component_not_found(component_name: impl Into<String>) -> Self {
        Self::ComponentNotFound {
            component_name: component_name.into(),
        }
    }

    /// Create a memory pool exhausted error
    pub fn memory_pool_exhausted(pool_type: impl Into<String>, requested: usize, available: usize) -> Self {
        Self::MemoryPoolExhausted {
            pool_type: pool_type.into(),
            requested,
            available,
        }
    }

    /// Create a concurrent load limit exceeded error
    pub fn concurrent_load_limit_exceeded(current: usize, max: usize) -> Self {
        Self::ConcurrentLoadLimitExceeded { current, max }
    }

    /// Create an unload failed error
    pub fn unload_failed(component_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::UnloadFailed {
            component_name: component_name.into(),
            reason:         reason.into(),
        }
    }

    /// Create a memory limit exceeded error
    pub fn memory_limit_exceeded(used_bytes: usize, limit_bytes: usize) -> Self {
        Self::MemoryLimitExceeded {
            used_bytes,
            limit_bytes,
        }
    }

    /// Create an invalid configuration error
    pub fn invalid_configuration(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidConfiguration {
            field:  field.into(),
            reason: reason.into(),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}

/// Result type alias for lazy loading operations
pub type LazyResult<T> = Result<T, LazyLoadingError>;

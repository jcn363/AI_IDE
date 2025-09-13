//! Consolidated shared types module
//!
//! This module provides a unified location for all shared type definitions
//! that are used across different modules in the backend.

pub mod cargo;

// Re-export commonly used types for convenience
pub use cargo::*;

// Common result and error types that can be used across modules
pub type Result<T> = std::result::Result<T, crate::errors::IDEError>;

// Common types for backend-frontend communication
pub type CommandResult<T> = Result<T>;
pub type AsyncResult<T> = futures::future::Ready<Result<T>>;

// Common collection types
pub type StringMap<T> = std::collections::HashMap<String, T>;
pub type StringSet = std::collections::HashSet<String>;

// Common ID types (assuming they're strings, but could be changed to UUIDs)
pub type ModuleId = String;
pub type CommandId = String;
pub type StreamId = String;
pub type SessionId = String;

// Common path types
pub type ProjectPath = std::path::PathBuf;
pub type FilePath = std::path::PathBuf;

// Common event types for streaming
pub type EventEmitter = tokio::sync::broadcast::Sender<serde_json::Value>;
pub type EventReceiver = tokio::sync::broadcast::Receiver<serde_json::Value>;

// Utility macros for type safety
#[macro_export]
macro_rules! impl_from_for_error {
    ($error_type:ty) => {
        impl From<$error_type> for crate::errors::IDEError {
            fn from(err: $error_type) -> Self {
                <$error_type>::from(err)
            }
        }
    };
}

// Specific type aliases for performance and testing
pub type Timestamp = chrono::DateTime<chrono::Utc>;
pub type DurationMs = u64;

// Version-related types
pub type VersionString = String;
pub type SemverVersion = semver::Version;
pub type VersionReq = semver::VersionReq;

// URL and identifier types
pub type UrlString = String;
pub type EmailString = String;
pub type PathString = String;

// Common enums for status and state
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Error,
    Warning,
    Info,
    Pending,
    Running,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

// Common data transfer objects
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiResponse<T> {
    pub status:    Status,
    pub data:      Option<T>,
    pub message:   Option<String>,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PaginatedResponse<T> {
    pub items:       Vec<T>,
    pub total_count: usize,
    pub page:        usize,
    pub per_page:    usize,
    pub has_more:    bool,
}

// Utility functions for creating responses
pub mod responses {
    use super::*;

    pub fn ok<T>(data: T) -> ApiResponse<T> {
        ApiResponse {
            status:    Status::Ok,
            data:      Some(data),
            message:   None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn error<T>(message: String) -> ApiResponse<T> {
        ApiResponse {
            status:    Status::Error,
            data:      None,
            message:   Some(message),
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn warning<T>(data: Option<T>, message: String) -> ApiResponse<T> {
        ApiResponse {
            status: Status::Warning,
            data,
            message: Some(message),
            timestamp: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_creation() {
        let response: ApiResponse<String> = responses::ok("test".to_string());
        assert!(matches!(response.status, Status::Ok));
        assert_eq!(response.data, Some("test".to_string()));
        assert!(response.timestamp <= chrono::Utc::now());

        let error_response: ApiResponse<String> = responses::error("something went wrong".to_string());
        assert!(matches!(error_response.status, Status::Error));
        assert_eq!(error_response.data, None);
        assert_eq!(
            error_response.message,
            Some("something went wrong".to_string())
        );
    }

    #[test]
    fn test_paginated_response_structure() {
        let response = PaginatedResponse {
            items:       vec!["item1".to_string(), "item2".to_string()],
            total_count: 10,
            page:        1,
            per_page:    5,
            has_more:    true,
        };

        assert_eq!(response.items.len(), 2);
        assert_eq!(response.total_count, 10);
        assert_eq!(response.page, 1);
        assert!(response.has_more);
    }

    #[test]
    fn test_status_serialization() {
        let json = serde_json::to_string(&Status::Ok).unwrap();
        assert_eq!(json, "\"ok\"");

        let deserialized: Status = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, Status::Ok));
    }

    #[test]
    fn test_priority_serialization() {
        let json = serde_json::to_string(&Priority::High).unwrap();
        assert_eq!(json, "\"high\"");

        let deserialized: Priority = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, Priority::High));
    }
}

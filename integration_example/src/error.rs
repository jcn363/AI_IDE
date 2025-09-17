//! Error handling types and utilities
//!
//! This module contains error-related types and functions for the integration example.
//! These types are designed to be serializable and provide structured error information.

use serde::{Deserialize, Serialize};

/// Error response
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorResponse {
    /// Error code
    pub code: String,

    /// Error message
    pub message: String,

    /// Additional error details
    pub details: Option<serde_json::Value>,
}
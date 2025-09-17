//! Error handling types for integration tests
//!
//! This module contains error types and utilities specific to integration testing.
//! Currently minimal, but can be expanded as more error handling is needed.

use std::fmt;

/// Integration test error
#[derive(Debug, Clone)]
pub struct IntegrationTestError {
    pub message: String,
    pub test_name: Option<String>,
    pub details: Option<String>,
}

impl IntegrationTestError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            test_name: None,
            details: None,
        }
    }

    pub fn with_test_name(mut self, test_name: &str) -> Self {
        self.test_name = Some(test_name.to_string());
        self
    }

    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }
}

impl fmt::Display for IntegrationTestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Integration test error")?;
        if let Some(test_name) = &self.test_name {
            write!(f, " in test '{}'", test_name)?;
        }
        write!(f, ": {}", self.message)?;
        if let Some(details) = &self.details {
            write!(f, " ({})", details)?;
        }
        Ok(())
    }
}

impl std::error::Error for IntegrationTestError {}
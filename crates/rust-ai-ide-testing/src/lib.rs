//! # Rust AI IDE Testing Crate
//!
//! This crate provides comprehensive testing utilities and integration tests
//! for the Rust AI IDE project.
//!
//! ## Features
//!
//! - Integration tests covering AI inference, analysis commands, terminal execution, and security
//! - Test utilities for common testing patterns
//! - Performance testing helpers
//! - Cross-crate integration validation

pub mod integration_tests;

/// Re-export testing utilities for convenience
pub use integration_tests::*;

/// Test configuration for integration tests
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub timeout_seconds:          u64,
    pub enable_performance_tests: bool,
    pub enable_security_tests:    bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            timeout_seconds:          30,
            enable_performance_tests: true,
            enable_security_tests:    true,
        }
    }
}

/// Initialize testing environment
pub fn init_test_environment() -> TestConfig {
    // Set up test environment variables and configuration
    TestConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_config_creation() {
        let config = TestConfig::default();
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.enable_performance_tests);
        assert!(config.enable_security_tests);
    }

    #[test]
    fn test_init_test_environment() {
        let config = init_test_environment();
        assert_eq!(config.timeout_seconds, 30);
    }
}

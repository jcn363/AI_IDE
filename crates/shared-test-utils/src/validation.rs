use crate::error::{TestError, ValidationError};
use std::path::Path;

/// Utility functions for validating test data and setup
pub struct ValidationUtils;

impl ValidationUtils {
    /// Validates that a path exists and meets security requirements
    pub fn validate_path_security(path: &Path) -> Result<(), TestError> {
        if !path.exists() {
            return Err(TestError::Validation(ValidationError::path_validation(
                format!("Path does not exist: {:?}", path),
            )));
        }

        // Check for dangerous path components
        if path.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        }) {
            return Err(TestError::Validation(ValidationError::security_validation(
                "Path contains dangerous components (.. or .)",
            )));
        }

        Ok(())
    }

    /// Validates content against expected patterns
    pub fn validate_content<T: AsRef<str>>(
        content: T,
        expected_patterns: &[&str],
    ) -> Result<(), TestError> {
        let content = content.as_ref();

        for pattern in expected_patterns {
            if !content.contains(pattern) {
                return Err(TestError::Validation(ValidationError::content_validation(
                    format!("Expected pattern '{}' not found in content", pattern),
                )));
            }
        }

        Ok(())
    }

    /// Validates that a test setup has required components
    pub fn validate_test_setup<T: Clone + std::fmt::Debug>(
        components: &[Option<T>],
        names: &[&str],
    ) -> Result<(), TestError> {
        for (i, component) in components.iter().enumerate() {
            if component.is_none() {
                return Err(TestError::Validation(ValidationError::invalid_setup(
                    format!(
                        "Missing required component: {}",
                        names.get(i).map_or("unknown", |v| v)
                    ),
                )));
            }
        }

        Ok(())
    }

    /// Validates Tauri command payload structure
    pub fn validate_command_payload<T: serde::Serialize>(
        payload: &T,
        required_fields: &[&str],
    ) -> Result<(), TestError> {
        let serialized =
            serde_json::to_value(payload).map_err(|e| TestError::Serialization(e.to_string()))?;

        if let serde_json::Value::Object(map) = serialized {
            for field in required_fields {
                if !map.contains_key(*field) {
                    return Err(TestError::Validation(ValidationError::invalid_setup(
                        format!("Missing required field in payload: {}", field),
                    )));
                }
            }
        } else {
            return Err(TestError::Validation(ValidationError::invalid_setup(
                "Payload is not a JSON object",
            )));
        }

        Ok(())
    }

    /// Validates that a result matches expected value
    pub fn validate_result<T: PartialEq + std::fmt::Debug>(
        actual: &T,
        expected: &T,
    ) -> Result<(), TestError> {
        if actual != expected {
            return Err(TestError::Validation(ValidationError::content_validation(
                format!("Expected {:?}, got {:?}", expected, actual),
            )));
        }
        Ok(())
    }
}

/// Macros for common validations
#[macro_export]
macro_rules! assert_validate_path {
    ($path:expr) => {
        $crate::validation::ValidationUtils::validate_path_security($path)
            .expect_test("Path validation failed");
    };
}

#[macro_export]
macro_rules! assert_validate_content {
    ($content:expr, $($pattern:expr),*) => {
        $crate::validation::ValidationUtils::validate_content(
            $content,
            &[$($pattern),*]
        ).expect_test("Content validation failed");
    };
}

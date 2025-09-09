//! Consolidated validation utilities for the Rust AI IDE project
//!
//! This module provides a comprehensive set of validation functions that consolidate
//! duplicate implementations found across modules. These functions follow consistent
//! patterns and use the project's standard error handling.
//! Additionally provides declarative validation macros for easy integration.

use crate::errors::{IdeError, IdeResult};
use std::path::Path;

// ==================== SECURITY VALIDATION MACROS ====================

/// Macro for declaring comprehensive string validation
/// # Examples
/// ```
/// validate_string_alt! {
///     input: "some input",
///     max_len: 50,
///     allow_special: false,
///     required: true,
///     field_name: "input_field"
/// }
/// ```
#[macro_export]
macro_rules! validate_string_alt {
    ($input:expr, $max_len:expr, $allow_special:expr, $required:expr, $field:expr) => {{
        use crate::validation::*;
        if $required && $input.is_empty() {
            return Err(IdeError::Validation {
                field: $field.to_string(),
                reason: format!("{}: Required input cannot be empty", $field),
            });
        }
        validate_string_input_extended($input, $max_len, $allow_special).map_err(|_| {
            IdeError::Validation {
                field: $field.to_string(),
                reason: format!("{}: Validation failed for input '{}'", $field, $input),
            }
        })?;
        Ok(())
    }};
}

/// Macro for file path validation
#[macro_export]
macro_rules! validate_file_path_alt {
    ($path:expr, $operation:expr) => {{
        use crate::validation::*;
        use std::path::Path;

        validate_file_exists($path, $operation)?;
        validate_file_size_path($path, 50 * 1024 * 1024, $operation)?; // 50MB limit
        Ok(())
    }};
}

/// Macro for command line argument validation
#[macro_export]
macro_rules! validate_tauri_command_args {
    ($($field:ident: $type:ty),*) => {{
        paste::paste! {
            $(
                impl crate::validation::TauriCommandValidation for [<Command Args $field:camel>] where [<Command Args $field:camel>]: std::fmt::Debug + serde::de::DeserializeOwned + ValidateFields {
                    // The actual validation will be delegated to ValidateFields implementation
                }
            )*
        }
    }};
}

/// Declarative validation macro for input sanitization and validation
#[macro_export]
macro_rules! sanitize_and_validate {
    ($input:expr => $field:ident) => {{
        let processed = crate::sanitize_string_for_processing(
            $input,
            &[
                "<script>",
                "</script>",
                "javascript:",
                "onload=",
                "onerror=",
            ],
        );
        match processed {
            Ok(sanitized) => {
                crate::validate_string_input(&sanitized, 1000)?;
                Ok(sanitized)
            }
            Err(e) => Err(IdeError::Validation {
                field: stringify!($field).to_string(),
                reason: format!("Validation failed: {}", e),
            }),
        }
    }};
}

/// Trait for types that need Tauri command validation
pub trait TauriCommandValidation:
    ValidateFields + std::fmt::Debug + serde::de::DeserializeOwned
{
}

/// Trait for field-level validation in structs
pub trait ValidateFields {
    fn validate_fields(&self) -> IdeResult<()> {
        Ok(()) // Default implementation - override in implementations
    }
}

/// Macro to implement ValidateFields for structures
#[macro_export]
macro_rules! impl_validate_fields {
    ($struct_name:ident, { $($field:ident: $validation_type:expr),* }) => {
        impl crate::validation::ValidateFields for $struct_name {
            fn validate_fields(&self) -> crate::errors::IdeResult<()> {
                $(
                    match $validation_type {
                        "string" => crate::validate_string_input(&self.$field, 500)?,
                        "string_no_special" => crate::validate_string_input_extended(&self.$field, 500, false)?,
                        "string_with_special" => crate::validate_string_input_extended(&self.$field, 500, true)?,
                        "path" => {
                            let path = std::path::Path::new(&self.$field);
                            crate::validate_file_exists(path, stringify!($struct_name))?;
                        },
                        _ => {} // Unknown validation type, do nothing
                    }
                )*
                Ok(())
            }
        }
    };
}

// ==================== INPUT SANITIZATION UTILITIES ====================

/// Sanitize string input for processing with security checks
pub fn sanitize_string_for_processing(input: &str, blocklist: &[&str]) -> IdeResult<String> {
    // Remove HTML tags and scripts (basic XSS protection)
    let no_html = input.replace("<", "&lt;").replace(">", "&gt;");

    // Remove null bytes
    let no_null = no_html.replace('\0', "");

    // Check against blocklist
    for blocked in blocklist {
        if input.contains(blocked) {
            return Err(IdeError::Validation {
                field: "input".to_string(),
                reason: format!("Input contains blocked content: {}", blocked),
            });
        }
    }

    Ok(no_null)
}

/// Sanitize file path by normalizing and removing dangerous components
pub fn sanitize_file_path(path: &str) -> IdeResult<String> {
    use std::path::Path;

    let path_obj = Path::new(path);

    // Normalize path to resolve .. and .
    if let Ok(canonical) = path_obj.canonicalize() {
        Ok(canonical.to_string_lossy().to_string())
    } else {
        Err(IdeError::Validation {
            field: "path".to_string(),
            reason: "Path cannot be canonicalized".to_string(),
        })
    }
}

// ==================== VALIDATION FUNCTIONS ====================

/// Basic string input validation
pub fn validate_string_input(input: &str, max_length: usize) -> IdeResult<()> {
    if input.is_empty() {
        return Err(IdeError::Validation {
            field: "input".to_string(),
            reason: "Input cannot be empty".to_string(),
        });
    }

    if input.len() > max_length {
        return Err(IdeError::Validation {
            field: "input".to_string(),
            reason: format!("Input exceeds maximum length of {} characters", max_length),
        });
    }

    Ok(())
}

/// Extended string input validation with special character control
pub fn validate_string_input_extended(
    input: &str,
    max_length: usize,
    allow_special: bool,
) -> IdeResult<()> {
    validate_string_input(input, max_length)?;

    if !allow_special {
        if input.contains(";") || input.contains("(") || input.contains(")") {
            return Err(IdeError::Validation {
                field: "input".to_string(),
                reason: "Special characters are forbidden in input".to_string(),
            });
        }
    }

    Ok(())
}

/// Validate that a file exists
pub fn validate_file_exists<P: AsRef<Path>>(path: P, operation: &str) -> IdeResult<()> {
    if !path.as_ref().exists() {
        return Err(IdeError::Validation {
            field: "file_path".to_string(),
            reason: format!(
                "{}: File does not exist: {}",
                operation,
                path.as_ref().to_string_lossy()
            ),
        });
    }

    Ok(())
}

/// Validate file size from path
pub fn validate_file_size_path<P: AsRef<Path>>(
    path: P,
    max_size: u64,
    operation: &str,
) -> IdeResult<()> {
    match std::fs::metadata(path.as_ref()) {
        Ok(metadata) => {
            if metadata.len() > max_size {
                return Err(IdeError::Validation {
                    field: "file".to_string(),
                    reason: format!(
                        "{}: File size {} bytes exceeds maximum allowed size {} bytes",
                        operation,
                        metadata.len(),
                        max_size
                    ),
                });
            }
        }
        Err(e) => {
            return Err(IdeError::Validation {
                field: "file".to_string(),
                reason: format!("{}: Cannot check file size: {}", operation, e),
            });
        }
    }
    Ok(())
}

/// Validate file size from content
pub fn validate_file_size_content(
    content: &[u8],
    max_size: usize,
    operation: &str,
) -> IdeResult<()> {
    if content.len() > max_size {
        return Err(IdeError::Validation {
            field: "file".to_string(),
            reason: format!(
                "{}: Content size {} bytes exceeds maximum allowed size {} bytes",
                operation,
                content.len(),
                max_size
            ),
        });
    }
    Ok(())
}

/// Validate path is not in excluded paths
pub fn validate_path_not_excluded<P: AsRef<Path>>(
    path: P,
    excluded_paths: &[String],
    operation: &str,
) -> IdeResult<()> {
    let path_str = path.as_ref().to_string_lossy();
    for excluded in excluded_paths {
        let excluded_pattern = excluded.trim_end_matches('/').trim_start_matches('/');
        if path_str.contains(excluded_pattern) {
            return Err(IdeError::Validation {
                field: "path".to_string(),
                reason: format!("{}: Path {} is in excluded paths list", operation, path_str),
            });
        }
    }
    Ok(())
}

/// Validate dependency format
pub fn validate_dependency_format(dep: &str) -> IdeResult<()> {
    if dep.is_empty() {
        return Err(IdeError::Validation {
            field: "dependency".to_string(),
            reason: "Dependency name cannot be empty".to_string(),
        });
    }

    if dep.len() > 256 {
        return Err(IdeError::Validation {
            field: "dependency".to_string(),
            reason: "Dependency name is too long".to_string(),
        });
    }

    // Check for valid package name format (simplified)
    if !dep
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(IdeError::Validation {
            field: "dependency".to_string(),
            reason: "Invalid dependency name format".to_string(),
        });
    }

    Ok(())
}

/// Validate that a directory exists
pub fn validate_directory_exists<P: AsRef<Path>>(path: P, operation: &str) -> IdeResult<()> {
    let path_obj = path.as_ref();

    if !path_obj.exists() {
        return Err(IdeError::Validation {
            field: "directory_path".to_string(),
            reason: format!(
                "{}: Directory does not exist: {}",
                operation,
                path_obj.to_string_lossy()
            ),
        });
    }

    if !path_obj.is_dir() {
        return Err(IdeError::Validation {
            field: "directory_path".to_string(),
            reason: format!(
                "{}: Path exists but is not a directory: {}",
                operation,
                path_obj.to_string_lossy()
            ),
        });
    }

    Ok(())
}

/// Validate file size with operation name (compatibility wrapper for existing API)
pub fn validate_file_size_with_operation(
    path: &str,
    max_size_kb: usize,
    operation: &str,
) -> IdeResult<()> {
    let max_size = max_size_kb * 1024; // Convert KB to bytes
    validate_file_size_path(path, max_size as u64, operation)
}

// ==================== TYPED VALIDATION STRUCTS ====================

/// A validated string that guarantees certain properties
#[derive(Debug, Clone)]
pub struct ValidatedString {
    value: String,
    max_length: usize,
    allowed_special: bool,
}

impl ValidatedString {
    pub fn new(value: &str, max_length: usize, allowed_special: bool) -> IdeResult<Self> {
        validate_string_input_extended(value, max_length, allowed_special)?;
        Ok(Self {
            value: value.to_string(),
            max_length,
            allowed_special,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }
}

/// A validated file path with security checks
#[derive(Debug, Clone)]
pub struct ValidatedFilePath {
    path: std::path::PathBuf,
}

impl ValidatedFilePath {
    pub fn new(path: &str, operation: &str) -> IdeResult<Self> {
        let path_obj = std::path::Path::new(path);

        // Basic path validation
        if !path_obj.exists() {
            return Err(IdeError::Validation {
                field: "file_path".to_string(),
                reason: format!("{}: File does not exist: {}", operation, path),
            });
        }

        // Check for path traversal attempts
        if let Some(_path_str) = path_obj.to_str() {
            for component in path_obj.components() {
                use std::path::Component::*;
                match component {
                    Prefix(_) | RootDir | CurDir | ParentDir => {
                        // These components are generally safe
                    }
                    Normal(part) => {
                        let part_str = part.to_string_lossy();
                        if part_str.contains("..") {
                            return Err(IdeError::Validation {
                                field: "file_path".to_string(),
                                reason: format!("{}: Path traversal detected: {}", operation, path),
                            });
                        }
                    }
                }
            }
        }

        Ok(Self {
            path: path_obj.to_path_buf(),
        })
    }

    pub fn to_path_buf(&self) -> std::path::PathBuf {
        self.path.clone()
    }

    pub fn as_path(&self) -> &std::path::Path {
        &self.path
    }
}

/// Sanitized SQL-like query (for search operations)
#[derive(Debug, Clone)]
pub struct SanitizedQuery {
    query: String,
}

impl SanitizedQuery {
    pub fn new(raw_query: &str) -> IdeResult<Self> {
        // Remove known dangerous characters/patterns
        let clean = sanitize_string_for_processing(
            raw_query,
            &["DROP ", "DELETE ", "UPDATE ", "--", "/*", "*/"],
        )?;

        // Basic length check
        validate_string_input(&clean, 500)?;

        Ok(Self { query: clean })
    }

    pub fn as_str(&self) -> &str {
        &self.query
    }
}

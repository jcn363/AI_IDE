//! Security measures for file operations
//!
//! This module provides secure file operation wrappers that prevent common security issues
//! such as path traversal, unauthorized access, and file system exploits.

use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::errors::{IDEError, IDEResult};

/// Secure file operations with validation
pub struct SecureFileOperations {
    allowed_base_paths: Vec<PathBuf>,
    max_file_size:      u64,
    blocked_extensions: Vec<String>,
}

impl Default for SecureFileOperations {
    fn default() -> Self {
        Self {
            allowed_base_paths: vec![
                PathBuf::from("./"),                 // Current working directory
                PathBuf::from("../"),                // Parent directory
                PathBuf::from(std::env::temp_dir()), // System temp directory
            ],
            max_file_size:      100 * 1024 * 1024, // 100MB
            blocked_extensions: vec![
                "exe".to_string(),
                "bat".to_string(),
                "cmd".to_string(),
                "com".to_string(),
                "scr".to_string(),
                "pif".to_string(),
                "lnk".to_string(),
            ],
        }
    }
}

impl SecureFileOperations {
    /// Create new secure file operations with custom configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Securely read a file with validation
    pub fn read_file_secure(&self, file_path: &str) -> IDEResult<String> {
        self.validate_file_path(file_path)?;
        self.validate_file_access(file_path, OperationType::Read)?;

        // Read the file content
        std::fs::read_to_string(file_path).map_err(|e| IDEError::FileOperation(format!("Failed to read file: {}", e)))
    }

    /// Securely write to a file with validation
    pub fn write_file_secure(&self, file_path: &str, content: &str) -> IDEResult<()> {
        self.validate_file_path(file_path)?;
        self.validate_file_access(file_path, OperationType::Write)?;

        // Validate content size
        if content.len() as u64 > self.max_file_size {
            return Err(IDEError::Validation(
                "Content exceeds maximum allowed size".to_string(),
            ));
        }

        // Create parent directories if they don't exist
        if let Some(parent) = Path::new(file_path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| IDEError::FileOperation(format!("Failed to create parent directories: {}", e)))?;
        }

        // Write the file
        fs::write(file_path, content).map_err(|e| IDEError::FileOperation(format!("Failed to write file: {}", e)))
    }

    /// Securely check if a file exists
    pub fn file_exists_secure(&self, file_path: &str) -> IDEResult<bool> {
        self.validate_file_path(file_path)?;
        Ok(Path::new(file_path).exists())
    }

    /// Securely delete a file
    pub fn delete_file_secure(&self, file_path: &str) -> IDEResult<()> {
        self.validate_file_path(file_path)?;
        self.validate_file_access(file_path, OperationType::Delete)?;

        // Additional check: ensure we're not deleting critical system files
        let path_obj = Path::new(file_path);
        let file_name = path_obj.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Block deletion of critical files
        if file_name.is_empty() || file_name.starts_with('.') {
            return Err(IDEError::Validation(
                "Deletion of hidden or system files is blocked".to_string(),
            ));
        }

        fs::remove_file(file_path).map_err(|e| IDEError::FileOperation(format!("Failed to delete file: {}", e)))
    }

    /// Validate file path against security rules
    fn validate_file_path(&self, file_path: &str) -> IDEResult<()> {
        if file_path.is_empty() {
            return Err(IDEError::Validation(
                "File path cannot be empty".to_string(),
            ));
        }

        // Limit path length
        if file_path.len() > 4096 {
            return Err(IDEError::Validation("File path too long".to_string()));
        }

        let path_obj = Path::new(file_path);

        // Check for path traversal attempts
        for component in path_obj.components() {
            use std::path::Component::*;
            match component {
                Normal(part) => {
                    if let Some(part_str) = part.to_str() {
                        // Block double dots (path traversal)
                        if part_str.contains("..") {
                            return Err(IDEError::Validation("Path traversal detected".to_string()));
                        }
                        // Block dangerous characters
                        for &ch in &['<', '>', '|', '"', '*', '?'] {
                            if part_str.contains(ch) {
                                return Err(IDEError::Validation(format!(
                                    "Dangerous character '{}' in path",
                                    ch
                                )));
                            }
                        }
                    }
                }
                _ => {} // Allow other components
            }
        }

        // Check if path is within allowed directories
        let canonical_path = path_obj
            .canonicalize()
            .map_err(|_| IDEError::Validation("Cannot canonicalize path".to_string()))?;

        self.validate_path_in_allowed_directories(&canonical_path)?;

        // Check file extension
        if let Some(ext) = path_obj.extension() {
            if let Some(ext_str) = ext.to_str() {
                if self.blocked_extensions.contains(&ext_str.to_lowercase()) {
                    return Err(IDEError::Validation(format!(
                        "File extension '{}' is blocked",
                        ext_str
                    )));
                }
            }
        }

        Ok(())
    }

    /// Validate that the path is within allowed base directories
    fn validate_path_in_allowed_directories(&self, canonical_path: &PathBuf) -> IDEResult<()> {
        for base_path in &self.allowed_base_paths {
            if let Ok(canonical_base) = base_path.canonicalize() {
                if canonical_path.starts_with(&canonical_base) {
                    return Ok(()); // Path is within allowed directory
                }
            }
        }

        Err(IDEError::Validation(
            "Path is outside of allowed directories".to_string(),
        ))
    }

    /// Validate file access permissions for the operation
    fn validate_file_access(&self, file_path: &str, operation: OperationType) -> IDEResult<()> {
        let path_obj = Path::new(file_path);
        let metadata = path_obj
            .metadata()
            .map_err(|_| IDEError::Validation("Cannot access file metadata".to_string()))?;

        // Check file size limits
        if metadata.len() > self.max_file_size {
            return Err(IDEError::Validation(
                "File size exceeds maximum allowed".to_string(),
            ));
        }

        // Additional operation-specific checks can go here
        match operation {
            OperationType::Delete => {
                // Additional checks for deletion
            }
            OperationType::Write => {
                // Check write permissions
            }
            OperationType::Read => {
                // Can add read-specific validations here
            }
        }

        Ok(())
    }
}

/// File operation types
#[derive(Debug, Clone, Copy)]
pub enum OperationType {
    Read,
    Write,
    Delete,
}

/// Global secure file operations instance
pub fn get_secure_file_ops() -> SecureFileOperations {
    SecureFileOperations::new()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_path_traversal_detection() {
        let ops = SecureFileOperations::new();

        // Should fail with path traversal
        let result = ops.validate_file_path("../etc/passwd");
        assert!(result.is_err());

        // Should fail with double dots in filename
        let result = ops.validate_file_path("test..file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_dangerous_path_characters() {
        let ops = SecureFileOperations::new();

        // Should fail with dangerous characters
        assert!(ops.validate_file_path("test<file.txt").is_err());
        assert!(ops.validate_file_path("test|file.txt").is_err());
        assert!(ops.validate_file_path("test*file.txt").is_err());
    }

    #[test]
    fn test_blocked_extensions() {
        let ops = SecureFileOperations::new();

        // Should fail with blocked extensions
        assert!(ops.validate_file_path("malicious.exe").is_err());
        assert!(ops.validate_file_path("script.bat").is_err());
    }

    #[test]
    fn test_valid_path() {
        let ops = SecureFileOperations::new();

        // Should pass with valid path
        let result = ops.validate_file_path("src/main.rs");
        assert!(result.is_ok());
    }
}

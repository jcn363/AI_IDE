//! Filesystem module for the Rust AI IDE backend
//!
//! This module provides file system operations, utilities for
//! file management, and path handling for the IDE.

use crate::errors::IDEServiceError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;

/// File system service
pub struct FileSystemService;

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub size: u64,
    pub is_directory: bool,
    pub modified: Option<u64>,
}

impl FileSystemService {
    /// Create a new filesystem service
    pub fn new() -> Self {
        Self
    }

    /// List files in a directory
    pub async fn list_files(&self, path: &Path) -> Result<Vec<FileMetadata>, IDEServiceError> {
        if !path.exists() {
            return Err(IDEServiceError::FileNotFound {
                path: path.to_path_buf(),
            });
        }

        let mut entries = Vec::new();
        let mut dir_entries = async_fs::read_dir(path).await?;

        while let Some(entry) = dir_entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            let modified = metadata.modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs());

            let file_metadata = FileMetadata {
                path: entry.path(),
                size: metadata.len(),
                is_directory: metadata.is_dir(),
                modified,
            };
            entries.push(file_metadata);
        }

        Ok(entries)
    }

    /// Read file content
    pub async fn read_file(&self, path: &Path) -> Result<String, IDEServiceError> {
        if !path.exists() {
            return Err(IDEServiceError::FileNotFound {
                path: path.to_path_buf(),
            });
        }

        async_fs::read_to_string(path).await
            .map_err(|e| IDEServiceError::IOError {
                message: e.to_string(),
            })
    }

    /// Write file content
    pub async fn write_file(&self, path: &Path, content: &str) -> Result<(), IDEServiceError> {
        async_fs::write(path, content).await
            .map_err(|e| IDEServiceError::IOError {
                message: e.to_string(),
            })
    }
}

impl Default for FileSystemService {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the filesystem module
pub fn init() -> Result<(), String> {
    log::info!("Initializing filesystem module");
    Ok(())
}
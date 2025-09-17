//! Comprehensive filesystem utilities for the Rust AI IDE
//!
//! This module consolidates all filesystem-related operations including:
//! - File I/O (reading, writing, copying)
//! - Directory operations (creation, listing, removal)
//! - Path manipulation (normalization, validation, utilities)
//! - File metadata and permissions
//! - Temporary files and atomic operations
//! - File watching

use std::path::{Path, PathBuf};

use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::errors::IdeError;
use crate::platform::{normalize_path_separators, Platform, PlatformFileSystem, PlatformMemoryHints};

/// ===== FILE EXISTENCE AND BASIC CHECKS =====

/// Asynchronous file existence check
pub async fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    fs::metadata(path).await.is_ok()
}

/// Asynchronous directory existence check
pub async fn dir_exists<P: AsRef<Path>>(path: P) -> Result<bool, IdeError> {
    match fs::metadata(path).await {
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(IdeError::Io {
            message: e.to_string(),
        }),
    }
}

/// Generic path existence check
pub async fn exists<P: AsRef<Path>>(path: P) -> Result<bool, IdeError> {
    match fs::metadata(path).await {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(IdeError::Io {
            message: e.to_string(),
        }),
    }
}

/// ===== FILE READING =====

/// Safe read entire file to string
pub async fn read_file_to_string<P: AsRef<Path>>(path: P) -> Result<String, IdeError> {
    let content = fs::read_to_string(path).await.map_err(|e| IdeError::Io {
        message: e.to_string(),
    })?;
    Ok(content)
}

/// Safe read entire file to bytes
pub async fn read_file_to_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, IdeError> {
    let content = fs::read(path).await.map_err(|e| IdeError::Io {
        message: e.to_string(),
    })?;
    Ok(content)
}

/// Read file with size limit to prevent memory exhaustion
pub async fn read_file_with_limit<P: AsRef<Path>>(
    path: P,
    max_size: u64,
) -> Result<Vec<u8>, IdeError> {
    let metadata = get_metadata(&path).await?;
    if metadata.len() > max_size {
        return Err(IdeError::Validation {
            field: "file_size".to_string(),
            reason: format!("File size {} exceeds limit {}", metadata.len(), max_size),
        });
    }

    read_file_to_bytes(path).await
}

/// ===== FILE WRITING =====

/// Write string to file atomically (via temp file)
pub async fn write_string_to_file<P: AsRef<Path>>(path: P, content: &str) -> Result<(), IdeError> {
    let path = path.as_ref();
    let temp_path = path.with_extension("tmp");

    // Write to temp file first
    fs::write(&temp_path, content)
        .await
        .map_err(|e| IdeError::Io {
            message: e.to_string(),
        })?;

    // Atomic rename
    fs::rename(&temp_path, path)
        .await
        .map_err(|e| IdeError::Io {
            message: e.to_string(),
        })?;

    Ok(())
}

/// Write bytes to file atomically
pub async fn write_bytes_to_file<P: AsRef<Path>>(path: P, content: &[u8]) -> Result<(), IdeError> {
    let path = path.as_ref();
    let temp_path = path.with_extension("tmp");

    fs::write(&temp_path, content)
        .await
        .map_err(|e| IdeError::Io {
            message: e.to_string(),
        })?;

    fs::rename(&temp_path, path)
        .await
        .map_err(|e| IdeError::Io {
            message: e.to_string(),
        })?;

    Ok(())
}

/// Append string to file
pub async fn append_to_file<P: AsRef<Path>>(path: P, content: &str) -> Result<(), IdeError> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
        .map_err(|e| IdeError::Io {
            message: e.to_string(),
        })?;

    file.write_all(content.as_bytes())
        .await
        .map_err(|e| IdeError::Io {
            message: e.to_string(),
        })?;

    Ok(())
}

/// ===== FILE OPERATIONS =====

/// Copy file with progress tracking (placeholder for large files)
pub async fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<u64, IdeError> {
    let result = fs::copy(from, to).await.map_err(|e| IdeError::Io {
        message: e.to_string(),
    })?;
    Ok(result)
}

/// Move/rename file
pub async fn move_file<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<(), IdeError> {
    fs::rename(from, to).await.map_err(|e| IdeError::Io {
        message: e.to_string(),
    })?;
    Ok(())
}

/// Delete file safely with existence check
pub async fn remove_file<P: AsRef<Path>>(path: P) -> Result<(), IdeError> {
    if file_exists(&path).await {
        fs::remove_file(path).await.map_err(|e| IdeError::Io {
            message: e.to_string(),
        })?;
    }
    Ok(())
}

/// ===== DIRECTORY OPERATIONS =====

/// Create directory (fails if exists)
pub async fn create_dir<P: AsRef<Path>>(path: P) -> Result<(), IdeError> {
    fs::create_dir(path).await.map_err(|e| IdeError::Io {
        message: e.to_string(),
    })?;
    Ok(())
}

/// Create directory and all parent directories
pub async fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<(), IdeError> {
    fs::create_dir_all(path).await.map_err(|e| IdeError::Io {
        message: e.to_string(),
    })?;
    Ok(())
}

/// Delete directory (must be empty)
pub async fn remove_dir<P: AsRef<Path>>(path: P) -> Result<(), IdeError> {
    if dir_exists(&path).await? {
        fs::remove_dir(path).await.map_err(|e| IdeError::Io {
            message: e.to_string(),
        })?;
    }
    Ok(())
}

/// Delete directory recursively
pub async fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<(), IdeError> {
    if dir_exists(&path).await? {
        fs::remove_dir_all(path).await.map_err(|e| IdeError::Io {
            message: e.to_string(),
        })?;
    }
    Ok(())
}

/// List all entries in a directory
pub async fn read_dir<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>, IdeError> {
    let mut entries = Vec::new();
    let mut reader = fs::read_dir(path).await.map_err(|e| IdeError::Io {
        message: e.to_string(),
    })?;

    while let Some(entry) = reader.next_entry().await.map_err(|e| IdeError::Io {
        message: e.to_string(),
    })? {
        entries.push(entry.path());
    }

    Ok(entries)
}

/// Read directory entries as filtered vec
pub async fn read_dir_filtered<P: AsRef<Path>, F>(
    path: P,
    filter: F,
) -> Result<Vec<PathBuf>, IdeError>
where
    F: Fn(&tokio::fs::DirEntry) -> bool,
{
    let mut entries = Vec::new();
    let mut reader = fs::read_dir(path).await.map_err(|e| IdeError::Io {
        message: e.to_string(),
    })?;

    while let Some(entry) = reader.next_entry().await.map_err(|e| IdeError::Io {
        message: e.to_string(),
    })? {
        if filter(&entry) {
            entries.push(entry.path());
        }
    }

    Ok(entries)
}

/// ===== METADATA =====

/// Get file metadata with caching optimization hints
pub async fn get_metadata<P: AsRef<Path>>(path: P) -> Result<std::fs::Metadata, IdeError> {
    let metadata = fs::metadata(path).await.map_err(|e| IdeError::Io {
        message: e.to_string(),
    })?;
    Ok(metadata)
}

/// Get file size in bytes
pub async fn get_file_size<P: AsRef<Path>>(path: P) -> Result<u64, IdeError> {
    let metadata = get_metadata(path).await?;
    Ok(metadata.len())
}

/// Check if path is a file
pub async fn is_file<P: AsRef<Path>>(path: P) -> Result<bool, IdeError> {
    match fs::metadata(path).await {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(IdeError::Io {
            message: e.to_string(),
        }),
    }
}

/// Check if path is a directory
pub async fn is_directory<P: AsRef<Path>>(path: P) -> Result<bool, IdeError> {
    match fs::metadata(path).await {
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(IdeError::Io {
            message: e.to_string(),
        }),
    }
}

/// ===== PATH UTILITIES =====

/// Ensure parent directories exist
pub async fn ensure_parent_dirs<P: AsRef<Path>>(path: P) -> Result<(), IdeError> {
    if let Some(parent) = path.as_ref().parent() {
        ensure_directory(parent).await?;
    }
    Ok(())
}

/// Ensure directory exists, creating it if necessary (async version of
/// path_utils::ensure_directory)
pub async fn ensure_directory<P: AsRef<Path>>(path: P) -> Result<(), IdeError> {
    let path = path.as_ref();
    if !path.exists() {
        fs::create_dir_all(path).await.map_err(|e| IdeError::Io {
            message: e.to_string(),
        })?;
    } else if !path.is_dir() {
        return Err(IdeError::Validation {
            field: "path".to_string(),
            reason: format!("Path exists and is a file, not a directory: {:?}", path),
        });
    }
    Ok(())
}

/// Normalize a path by resolving '.' and '..' components consistently
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut result = PathBuf::new();
    let path = path.as_ref();

    for component in path.components() {
        match component {
            std::path::Component::Normal(c) => {
                result.push(c);
            }
            std::path::Component::ParentDir => {
                result.pop();
            }
            std::path::Component::CurDir => {
                // Skip '.'
            }
            std::path::Component::Prefix(prefix) => {
                result.push(prefix.as_os_str());
            }
            std::path::Component::RootDir => {
                result.push(component);
            }
        }
    }

    result
}

/// Calculate relative path from base to target
pub fn relative_path_from<P: AsRef<Path>, Q: AsRef<Path>>(base: P, target: Q) -> Option<PathBuf> {
    let base = normalize_path(base);
    let target = normalize_path(target);

    let mut base_components: Vec<_> = base.components().collect();
    let mut target_components: Vec<_> = target.components().collect();

    // Remove common prefix
    while let (Some(base_comp), Some(target_comp)) =
        (base_components.first(), target_components.first())
    {
        if base_comp == target_comp {
            base_components.remove(0);
            target_components.remove(0);
        } else {
            break;
        }
    }

    // Add '..' for each remaining base component
    let mut result = PathBuf::new();
    for _ in base_components {
        result.push("..");
    }

    // Add remaining target components
    for comp in target_components {
        result.push(comp);
    }

    Some(result)
}

/// Ensure path has consistent separators for the platform
pub fn ensure_platform_path<P: AsRef<Path>>(path: P) -> PathBuf {
    normalize_path_separators(path)
}

/// Merge two paths without panic
pub fn safe_path_join<P: AsRef<Path>, Q: AsRef<Path>>(base: P, relative: Q) -> PathBuf {
    let mut result = base.as_ref().to_path_buf();
    result.push(relative);
    result
}

/// Check if a path is absolute
pub fn is_absolute_path<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_absolute()
}

/// Get the parent directory safely
pub fn parent_directory<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
    path.as_ref().parent().map(|p| p.to_path_buf())
}

/// Convert PathBuf to string safely, handling unicode properly
pub fn path_to_string<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref().to_str().map(|s| s.to_string())
}

/// Convert string to PathBuf safely
pub fn string_to_path(s: &str) -> PathBuf {
    PathBuf::from(s)
}

/// Validate if a path exists and is accessible
pub fn validate_path<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let path = path.as_ref();
    if path.exists() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Path does not exist: {:?}", path),
        ))
    }
}

/// Safe canonicalize wrapper that handles non-existent paths
pub fn safe_canonicalize<P: AsRef<Path>>(path: P) -> std::io::Result<PathBuf> {
    let path = path.as_ref();
    std::fs::canonicalize(path).or_else(|_| Ok(path.to_path_buf()))
}

/// ===== ATOMIC OPERATIONS =====

/// Atomic file update pattern
pub async fn update_file_atomically<P: AsRef<Path>, F, Fut, T>(
    path: P,
    updater: F,
) -> Result<T, IdeError>
where
    F: FnOnce(String) -> Fut,
    Fut: std::future::Future<Output = (String, T)>,
{
    let path = path.as_ref();
    let original_content = read_file_to_string(path).await?;
    let (new_content, result) = updater(original_content).await;
    write_string_to_file(path, &new_content).await?;
    Ok(result)
}

/// ===== UTILITY FUNCTIONS =====

/// List files recursively with max depth
pub async fn list_files_recursive<P: AsRef<Path>>(
    path: P,
    max_depth: Option<usize>,
) -> Result<Vec<PathBuf>, IdeError> {
    let mut results = Vec::new();
    let mut stack = vec![(path.as_ref().to_path_buf(), 0)];

    while let Some((current_path, depth)) = stack.pop() {
        if let Some(max) = max_depth {
            if depth > max {
                continue;
            }
        }

        if let Ok(entries) = read_dir_filtered(&current_path, |_| true).await {
            for entry in entries {
                if let Ok(is_file) = is_file(&entry).await {
                    if is_file {
                        results.push(entry);
                    } else if let Ok(is_dir) = is_directory(&entry).await {
                        if is_dir {
                            stack.push((entry, depth + 1));
                        }
                    }
                }
            }
        }
    }

    Ok(results)
}

/// Create a unique temporary path within a directory
pub fn temporary_path_in<P: AsRef<Path>>(dir: P, prefix: &str, suffix: &str) -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let mut result = dir.as_ref().to_path_buf();
    result.push(format!("{}_{}{}", prefix, timestamp, suffix));
    result
}

/// ===== FILE WATCHING =====

/// File watching helper (integrates with workspace patterns)
pub async fn watch_file_changes<P: AsRef<Path>, F>(path: P, callback: F) -> Result<(), IdeError>
where
    F: Fn(&notify::Event),
{
    use std::sync::mpsc::channel;

    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

    let (tx, rx) = channel();

    let mut watcher =
        RecommendedWatcher::new(tx, Config::default()).map_err(|e| IdeError::Generic {
            message: format!("Watcher error: {:?}", e),
        })?;

    watcher
        .watch(path.as_ref(), RecursiveMode::Recursive)
        .map_err(|e| IdeError::Generic {
            message: format!("Watch error: {:?}", e),
        })?;

    while let Ok(event_result) = rx.recv() {
        match event_result {
            Ok(event) => callback(&event),
            Err(e) => log::error!("Error receiving file watch event: {:?}", e),
        }
    }

    Ok(())
}

/// ===== CROSS-PLATFORM FILE SYSTEM MANAGER =====

/// Cross-platform file system manager that handles platform differences
pub struct CrossPlatformFileSystem;

impl CrossPlatformFileSystem {
    /// Create platform-optimized file with appropriate buffer sizes
    pub async fn create_optimized_file<P: AsRef<Path>>(
        path: P,
        content: &[u8],
    ) -> Result<(), IdeError> {
        let buffer_size = PlatformFileSystem::optimal_buffer_size();
        let path = path.as_ref();

        if content.len() > buffer_size {
            // For large files, use streaming approach
            Self::write_large_file(path, content).await
        } else {
            // For small files, use standard approach
            write_bytes_to_file(path, content).await
        }
    }

    /// Optimized writing for large files using platform-specific buffer sizes
    async fn write_large_file(path: &Path, content: &[u8]) -> Result<(), IdeError> {
        use tokio::io::AsyncWriteExt;

        let temp_path = path.with_extension("tmp");
        let buffer_size = PlatformFileSystem::optimal_buffer_size();

        {
            let mut file = fs::File::create(&temp_path).await.map_err(|e| IdeError::Io {
                message: format!("Failed to create temp file: {}", e),
            })?;

            for chunk in content.chunks(buffer_size) {
                file.write_all(chunk).await.map_err(|e| IdeError::Io {
                    message: format!("Failed to write chunk: {}", e),
                })?;
            }

            file.flush().await.map_err(|e| IdeError::Io {
                message: format!("Failed to flush file: {}", e),
            })?;
        }

        // Atomic rename
        fs::rename(&temp_path, path).await.map_err(|e| IdeError::Io {
            message: format!("Failed to rename temp file: {}", e),
        })?;

        Ok(())
    }

    /// Cross-platform secure file deletion with platform-specific handling
    pub async fn secure_delete<P: AsRef<Path>>(path: P) -> Result<(), IdeError> {
        let path = path.as_ref();

        #[cfg(target_os = "windows")]
        {
            // On Windows, use secure deletion if available
            if let Ok(()) = Self::secure_delete_windows(path).await {
                return Ok(());
            }
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, use srm or secure deletion
            if let Ok(()) = Self::secure_delete_unix(path, "srm").await {
                return Ok(());
            }
        }

        #[cfg(target_os = "linux")]
        {
            // On Linux, use shred or secure deletion
            if let Ok(()) = Self::secure_delete_unix(path, "shred").await {
                return Ok(());
            }
        }

        // Fallback to regular deletion
        remove_file(path).await
    }

    #[cfg(target_os = "windows")]
    async fn secure_delete_windows(path: &Path) -> Result<(), IdeError> {
        use tokio::process::Command;

        let output = Command::new("cipher")
            .args(&["/w", &path.to_string_lossy()])
            .output()
            .await
            .map_err(|e| IdeError::Io {
                message: format!("Failed to run cipher: {}", e),
            })?;

        if output.status.success() {
            remove_file(path).await
        } else {
            Err(IdeError::Io {
                message: "Secure deletion failed".to_string(),
            })
        }
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    async fn secure_delete_unix(path: &Path, command: &str) -> Result<(), IdeError> {
        use tokio::process::Command;

        let output = Command::new(command)
            .args(&["-u", &path.to_string_lossy()])
            .output()
            .await
            .map_err(|e| IdeError::Io {
                message: format!("Failed to run {}: {}", command, e),
            })?;

        if output.status.success() {
            Ok(())
        } else {
            Err(IdeError::Io {
                message: format!("Secure deletion with {} failed", command),
            })
        }
    }

    /// Get platform-specific user directories
    pub fn get_user_dirs() -> UserDirectories {
        UserDirectories {
            home: Platform::get_user_home_dir().unwrap_or_else(|_| PathBuf::from("/tmp")),
            config: Platform::get_app_data_dir("rust-ai-ide").unwrap_or_else(|_| PathBuf::from("/tmp")),
            cache: Platform::get_cache_dir("rust-ai-ide").unwrap_or_else(|_| PathBuf::from("/tmp")),
            temp: Platform::get_temp_dir(),
        }
    }

    /// Check if path is valid for current platform and security constraints
    pub fn validate_path_security<P: AsRef<Path>>(path: P) -> Result<(), IdeError> {
        use crate::platform::is_valid_path;

        let path = path.as_ref();

        if !is_valid_path(path) {
            return Err(IdeError::Validation {
                field: "path".to_string(),
                reason: format!("Path contains invalid characters for platform: {:?}", path),
            });
        }

        // Check for path traversal attempts
        if path.to_string_lossy().contains("..") {
            return Err(IdeError::Security {
                reason: "Path traversal attempt detected".to_string(),
            });
        }

        Ok(())
    }
}

/// Platform-specific user directory structure
pub struct UserDirectories {
    pub home: PathBuf,
    pub config: PathBuf,
    pub cache: PathBuf,
    pub temp: PathBuf,
}

impl UserDirectories {
    /// Get the platform-specific config file path for an application
    pub fn config_file(&self, app_name: &str, filename: &str) -> PathBuf {
        self.config.join(app_name).join(filename)
    }

    /// Get the platform-specific cache file path
    pub fn cache_file(&self, app_name: &str, filename: &str) -> PathBuf {
        self.cache.join(app_name).join(filename)
    }

    /// Get the platform-specific temp file path
    pub fn temp_file(&self, app_name: &str, filename: &str) -> PathBuf {
        self.temp.join(format!("{}_{}", app_name, filename))
    }
}

/// ===== TESTS =====

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn test_basic_file_operations() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        // Test writing
        write_string_to_file(&test_file, "Hello, World!")
            .await
            .unwrap();

        // Test reading
        let content = read_file_to_string(&test_file).await.unwrap();
        assert_eq!(content, "Hello, World!");

        // Test existence
        assert!(file_exists(&test_file).await);
        assert!(exists(&test_file).await.unwrap());
    }

    #[tokio::test]
    async fn test_directory_operations() {
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path().join("test_dir");
        let nested_file = test_dir.join("nested.txt");

        // Create directory
        create_dir_all(&test_dir).await.unwrap();
        assert!(dir_exists(&test_dir).await.unwrap());

        // Write to nested file
        write_string_to_file(&nested_file, "Nested content")
            .await
            .unwrap();

        // List directory
        let entries = read_dir(&test_dir).await.unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_path_utilities() {
        let path = PathBuf::from("./foo/../bar/./baz");
        let normalized = normalize_path(path);
        assert_eq!(normalized, PathBuf::from("bar/baz"));

        let base = PathBuf::from("/home/user/project");
        let target = PathBuf::from("/home/user/project/src/main.rs");
        let relative = relative_path_from(base, target).unwrap();
        assert_eq!(relative, PathBuf::from("src/main.rs"));
    }
}

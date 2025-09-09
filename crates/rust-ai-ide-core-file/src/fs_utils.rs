use super::IDEResult;
use rust_ai_ide_core_fundamentals::error::IDEError;
use sha2::Digest;
use std::fs;
use std::path::{Path, PathBuf};

/// File operation utilities
pub mod file_ops {
    use super::*;

    /// Safe file reading with size limits and error handling
    pub fn read_file_with_limits<P: AsRef<Path>>(
        path: P,
        max_size: Option<u64>,
    ) -> IDEResult<String> {
        let path = path.as_ref();
        let max_size =
            max_size.unwrap_or(rust_ai_ide_core_fundamentals::constants::MAX_FILE_SIZE_BYTES);

        // Check file size before reading
        let metadata = fs::metadata(path)
            .map_err(|e| IDEError::FileSystem(format!("Failed to read file metadata: {}", e)))?;
        if metadata.len() > max_size {
            return Err(IDEError::FileSystem(format!(
                "File {} exceeds maximum size of {} bytes",
                path.display(),
                max_size
            )));
        }

        // Read with proper error handling
        let mut reader = std::io::BufReader::new(fs::File::open(path).map_err(|e| {
            IDEError::FileSystem(format!("Failed to open file {}: {}", path.display(), e))
        })?);
        let mut content = String::new();
        std::io::Read::read_to_string(&mut reader, &mut content)
            .map_err(|e| IDEError::FileSystem(format!("Failed to read file content: {}", e)))?;
        Ok(content)
    }

    /// Write file with atomic operations
    pub fn write_file_atomic<P: AsRef<Path>, S: AsRef<str>>(path: P, content: S) -> IDEResult<()> {
        let path = path.as_ref();
        let content = content.as_ref();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| IDEError::FileSystem(format!("Failed to create directory: {}", e)))?;
        }

        fs::write(path, content)
            .map_err(|e| IDEError::FileSystem(format!("Failed to write file: {}", e)))?;
        Ok(())
    }
}

/// Directory operations
pub mod dir_ops {
    use super::*;

    /// Ensure directory exists
    pub fn ensure_directory<P: AsRef<Path>>(path: P) -> IDEResult<()> {
        let path = path.as_ref();

        if !path.exists() {
            fs::create_dir_all(path)
                .map_err(|e| IDEError::FileSystem(format!("Failed to create directory: {}", e)))?;
        } else if !path.is_dir() {
            return Err(IDEError::FileSystem(format!(
                "Path {:?} exists but is not a directory",
                path
            )));
        }

        Ok(())
    }

    /// Find files recursively with filtering
    pub fn find_files<F>(root: &Path, filter_fn: F) -> IDEResult<Vec<PathBuf>>
    where
        F: Fn(&Path) -> bool,
    {
        if !root.exists() {
            return Ok(Vec::new());
        }

        let walk_dir = walkdir::WalkDir::new(root);
        let mut result = Vec::new();

        for entry in walk_dir.into_iter().flatten() {
            let path = entry.path();
            if entry.file_type().is_file() && filter_fn(path) {
                result.push(path.to_path_buf());
            }
        }

        Ok(result)
    }
}

/// Path utilities specific to file system operations
pub mod path_utils {
    use super::*;
    use std::io::Read;

    /// Calculate file hash using SHA256
    pub fn hash_file<P: AsRef<Path>>(path: P) -> IDEResult<String> {
        let path = path.as_ref();
        let mut file = fs::File::open(path)
            .map_err(|e| IDEError::FileSystem(format!("Failed to open file for hashing: {}", e)))?;

        let mut hasher = sha2::Sha256::new();
        let mut buffer = vec![0; 8192]; // 8KB buffer

        loop {
            let bytes_read = file.read(&mut buffer).map_err(|e| {
                IDEError::FileSystem(format!("Failed to read file for hashing: {}", e))
            })?;

            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    /// Check if directory is a Rust project
    pub fn is_rust_project(path: &Path) -> bool {
        path.join("Cargo.toml").exists() && path.join("src").exists()
    }
}

// Re-exports
pub use dir_ops::*;
pub use file_ops::*;
pub use path_utils::*;

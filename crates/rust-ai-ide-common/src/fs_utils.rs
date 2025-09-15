/// ! Filesystem abstraction utilities for consistent async file operations
use std::path::{Path, PathBuf};

use tokio::fs;

use crate::errors::IdeError;

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

/// Copy file with progress tracking (placeholder for large files)
pub async fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<u64, IdeError> {
    let result = fs::copy(from, to).await.map_err(|e| IdeError::Io {
        message: e.to_string(),
    })?;
    Ok(result)
}

/// Read directory entries as filtered vec
pub async fn read_dir_filtered<P: AsRef<Path>, F>(path: P, filter: F) -> Result<Vec<PathBuf>, IdeError>
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

/// Get file metadata with caching optimization hints
pub async fn get_metadata<P: AsRef<Path>>(path: P) -> Result<std::fs::Metadata, IdeError> {
    let metadata = fs::metadata(path).await.map_err(|e| IdeError::Io {
        message: e.to_string(),
    })?;
    Ok(metadata)
}

/// Ensure parent directories exist
pub async fn ensure_parent_dirs<P: AsRef<Path>>(path: P) -> Result<(), IdeError> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent).await.map_err(|e| IdeError::Io {
            message: e.to_string(),
        })?;
    }
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

/// Delete directory recursively
pub async fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<(), IdeError> {
    if dir_exists(&path).await? {
        fs::remove_dir_all(path).await.map_err(|e| IdeError::Io {
            message: e.to_string(),
        })?;
    }
    Ok(())
}

/// Atomic file update pattern
pub async fn update_file_atomically<P: AsRef<Path>, F, Fut, T>(path: P, updater: F) -> Result<T, IdeError>
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

/// List files recursively with max depth
pub async fn list_files_recursive<P: AsRef<Path>>(path: P, max_depth: Option<usize>) -> Result<Vec<PathBuf>, IdeError> {
    let mut results = Vec::new();
    let mut stack = vec![(path.as_ref().to_path_buf(), 0)];

    while let Some((current_path, depth)) = stack.pop() {
        if let Some(max) = max_depth {
            if depth > max {
                continue;
            }
        }

        let entries = read_dir_filtered(&current_path, |_| true).await?;
        for entry in entries {
            if entry.is_file() {
                results.push(entry);
            } else if entry.is_dir() {
                stack.push((entry, depth + 1));
            }
        }
    }

    Ok(results)
}

/// File watching helper (integrates with workspace patterns)
pub async fn watch_file_changes<P: AsRef<Path>, F>(path: P, callback: F) -> Result<(), IdeError>
where
    F: Fn(&notify::Event),
{
    use std::sync::mpsc::channel;

    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default()).map_err(|e| IdeError::Generic {
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

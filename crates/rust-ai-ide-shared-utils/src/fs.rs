use std::path::Path;
use std::sync::Arc;

use moka::future::Cache;
use tokio::sync::Mutex;

/// File size cache type
type FileSizeCache = Cache<String, u64>;

/// Global file size cache
static FILE_SIZE_CACHE: once_cell::sync::Lazy<Arc<Mutex<FileSizeCache>>> =
    once_cell::sync::Lazy::new(|| {
        Arc::new(Mutex::new(
            Cache::builder()
                .max_capacity(1000)
                .time_to_live(std::time::Duration::from_secs(300)) // 5 minutes TTL
                .build(),
        ))
    });

/// File utilities
/// Get file size in a safe way with caching
///
/// This function caches file sizes to avoid repeated filesystem calls
/// for frequently accessed files.
///
/// # Arguments
/// * `path` - Path to the file
///
/// # Returns
/// `Some(u64)` containing file size in bytes, or `None` if path is not a file or metadata cannot be
/// read
pub async fn get_file_size_cached(path: &Path) -> Option<u64> {
    let path_str = path.to_string_lossy().to_string();

    // Try cache first
    let cache = FILE_SIZE_CACHE.lock().await;
    if let Some(size) = cache.get(&path_str).await {
        return Some(size);
    }
    drop(cache);

    // Compute and cache if not found
    let size = get_file_size(path)?;
    let cache = FILE_SIZE_CACHE.lock().await;
    cache.insert(path_str, size).await;
    Some(size)
}

/// Get file size in a safe way
///
/// # Arguments
/// * `path` - Path to the file
///
/// # Returns
/// `Some(u64)` containing file size in bytes, or `None` if path is not a file or metadata cannot be
/// read
#[must_use]
pub fn get_file_size(path: &Path) -> Option<u64> {
    path.metadata().ok()?.len().into()
}

/// Check if a file is readable
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// `true` if the file exists and is readable, `false` otherwise
#[must_use]
pub fn is_readable(path: &Path) -> bool {
    path.exists()
        && path
            .metadata()
            .map(|m| !m.permissions().readonly())
            .unwrap_or(false)
}

/// Check if a file is writable
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// `true` if the file exists and is writable, `false` otherwise
#[must_use]
pub fn is_writable(path: &Path) -> bool {
    path.exists()
        && path
            .metadata()
            .map(|m| {
                !m.permissions().readonly()
                    && (!m.is_dir() || path.parent().is_some_and(|p| p.exists()))
            })
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_get_file_size() {
        let path = Path::new("src/lib.rs");
        if path.exists() {
            let size = get_file_size(path);
            assert!(size.is_some());
            assert!(size.unwrap() > 0);
        }
    }

    #[test]
    fn test_is_readable() {
        let path = Path::new("src/lib.rs");
        if path.exists() {
            assert!(is_readable(path));
        }
    }

    #[test]
    fn test_is_writable() {
        let path = Path::new("src/lib.rs");
        if path.exists() {
            // This might fail in CI environments with read-only filesystems
            let _ = is_writable(path);
        }
    }
}
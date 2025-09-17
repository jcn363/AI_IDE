/// ! Path manipulation utilities for consistent cross-platform path handling
use std::path::{Path, PathBuf};

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
    path.as_ref().to_path_buf()
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

/// Ensure directory exists, creating it if necessary
pub async fn ensure_directory<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        tokio::fs::create_dir_all(path).await
    } else if path.is_file() {
        Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("Path exists and is a file, not a directory: {:?}", path),
        ))
    } else {
        Ok(())
    }
}

/// Safe canonicalize wrapper that handles non-existent paths
pub fn safe_canonicalize<P: AsRef<Path>>(path: P) -> std::io::Result<PathBuf> {
    let path = path.as_ref();
    std::fs::canonicalize(path).or_else(|_| Ok(path.to_path_buf()))
}

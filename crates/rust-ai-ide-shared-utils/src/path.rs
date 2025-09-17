use std::path::Path;

use dunce;
use pathdiff;

/// Path utilities
/// Normalize a path for consistent handling across platforms
///
/// This function ensures paths use consistent separators and handles
/// relative paths appropriately.
///
/// # Arguments
/// * `path` - The path to normalize
///
/// # Returns
/// A normalized path as a String
#[must_use]
pub fn normalize_path(path: &Path) -> String {
    dunce::canonicalize(path)
        .unwrap_or_else(|_| path.to_owned())
        .to_string_lossy()
        .into_owned()
}

/// Get the relative path from a base directory to a target path
///
/// # Arguments
/// * `base` - The base path
/// * `target` - The target path
///
/// # Returns
/// `Some(String)` containing the relative path, or `None` if computation fails
#[must_use]
pub fn relative_path(base: &Path, target: &Path) -> Option<String> {
    pathdiff::diff_paths(target, base).and_then(|p| p.to_str().map(String::from))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_normalize_path() {
        let path = Path::new("./src/main.rs");
        let normalized = normalize_path(path);
        assert!(normalized.ends_with("main.rs"));
    }
}
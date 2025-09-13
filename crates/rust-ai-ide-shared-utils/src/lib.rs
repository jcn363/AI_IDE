#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

//! Shared utilities for Rust AI IDE
//!
//! This crate provides canonical implementations of utility functions
//! used across multiple crates in the Rust AI IDE codebase.

use std::path::Path;

/// Get the file extension from a path
///
/// This is the canonical implementation of file extension extraction
/// that should be used across all crates in the Rust AI IDE codebase.
///
/// # Arguments
/// * `path` - The path to extract extension from
///
/// # Returns
/// * `Some(&str)` containing the extension (without the dot)
/// * `None` if the path has no extension or is a directory
///
/// # Examples
/// ```
/// use rust_ai_ide_shared_utils::get_extension;
/// use std::path::Path;
///
/// let path = Path::new("file.rs");
/// assert_eq!(get_extension(path), Some("rs"));
///
/// let path = Path::new("file.tar.gz");
/// assert_eq!(get_extension(path), Some("gz"));
///
/// let path = Path::new("file_no_ext");
/// assert_eq!(get_extension(path), None);
/// ```
#[must_use]
pub fn get_extension(path: &Path) -> Option<&str> {
    path.extension()?.to_str()
}

/// Check if a file is a code file based on its extension
///
/// This function considers common programming language extensions
/// that are likely to contain code that should be analyzed by the IDE.
///
/// # Arguments
/// * `path` - The path to check
///
/// # Returns
/// `true` if the file is likely a code file, `false` otherwise
///
/// # Examples
/// ```
/// use rust_ai_ide_shared_utils::is_code_file;
/// use std::path::Path;
///
/// let rust_file = Path::new("main.rs");
/// assert!(is_code_file(rust_file));
///
/// let plaintext_file = Path::new("readme.txt");
/// assert!(!is_code_file(plaintext_file));
///
/// let dir = Path::new("src/");
/// assert!(!is_code_file(dir));
/// ```
#[must_use]
pub fn is_code_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    get_extension(path).is_some_and(|ext| {
        matches!(
            ext,
            // Rust
            "rs" | "rlib" |
        // Python
        "py" | "pyi" | "pyc" |
        // JavaScript/TypeScript
        "js" | "jsx" | "ts" | "tsx" | "json" | "mjs" |
        // Java
        "java" | "class" | "jar" |
        // C/C++
        "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "hxx" |
        // C#
        "cs" | "csx" |
        // Go
        "go" |
        // PHP
        "php" |
        // Ruby
        "rb" |
        // Swift
        "swift" |
        // Kotlin
        "kt" | "kts" |
        // Scala
        "scala" | "sc" |
        // Lua
        "lua" |
        // Perl
        "pl" | "pm" |
        // Haskell
        "hs" | "lhs" |
        // OCaml
        "ml" | "mli" |
        // F#
        "fs" | "fsx" |
        // Elixir
        "ex" | "exs" |
        // Clojure
        "clj" | "cljs" | "cljc" |
        // Erlang
        "erl" | "hrl" |
        // Shell scripts
        "sh" | "bash" | "zsh" | "fish" |
        // Configuration files
        "toml" | "yaml" | "yml" | "xml" | "ini" | "cfg" |
        // Build files
        "gradle" | "maven" | "sbt" | "make" | "cmake" |
        // Documentation
        "md" | "rst" | "adoc" | "tex" |
        // Other common extensions
        "sql" | "csv" | "log"
        )
    })
}

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

/// File utilities
/// Get file size in a safe way
///
/// # Arguments
/// * `path` - Path to the file
///
/// # Returns
/// `Some(u64)` containing file size in bytes, or `None` if path is not a file or metadata cannot be read
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_get_extension() {
        assert_eq!(get_extension(Path::new("file.rs")), Some("rs"));
        assert_eq!(get_extension(Path::new("file.tar.gz")), Some("gz"));
        assert_eq!(get_extension(Path::new("no_extension")), None);
        assert_eq!(get_extension(Path::new(".hidden")), None);
    }

    #[test]
    fn test_is_code_file() {
        assert!(is_code_file(Path::new("main.rs")));
        assert!(is_code_file(Path::new("script.py")));
        assert!(is_code_file(Path::new("config.json")));
        assert!(!is_code_file(Path::new("readme.txt")));
        assert!(!is_code_file(Path::new("image.png")));
        assert!(!is_code_file(Path::new("src/")));
    }

    #[test]
    fn test_normalize_path() {
        let path = Path::new("./src/main.rs");
        let normalized = normalize_path(path);
        assert!(normalized.ends_with("main.rs"));
    }
}

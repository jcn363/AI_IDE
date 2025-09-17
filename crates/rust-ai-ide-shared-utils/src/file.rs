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
/// use std::path::Path;
///
/// use rust_ai_ide_shared_utils::file::get_extension;
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
/// use std::path::Path;
///
/// use rust_ai_ide_shared_utils::file::is_code_file;
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
    // If the file exists and is not a file (e.g., directory), return false
    if path.exists() && !path.is_file() {
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
        // Documentation (removed md from code files)
        "rst" | "adoc" | "tex" |
        // Other common extensions
        "sql" | "csv" | "log"
        )
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_get_extension() {
        assert_eq!(get_extension(Path::new("file.rs")), Some("rs"));
        assert_eq!(get_extension(Path::new("file.tar.gz")), Some("gz"));
        assert_eq!(get_extension(Path::new("no_extension")), None);
        assert_eq!(get_extension(Path::new(".hidden")), None);
    }

    #[test]
    fn test_is_code_file() {
        // Test with file extensions that should be code files
        assert!(is_code_file(Path::new("test.rs")));
        assert!(is_code_file(Path::new("test.py")));
        assert!(is_code_file(Path::new("test.json")));
        assert!(is_code_file(Path::new("test.js")));
        assert!(is_code_file(Path::new("test.go")));
        assert!(is_code_file(Path::new("test.java")));

        // Test with file extensions that should NOT be code files
        assert!(!is_code_file(Path::new("readme.txt")));
        assert!(!is_code_file(Path::new("image.png")));
        assert!(!is_code_file(Path::new("document.pdf")));
        assert!(!is_code_file(Path::new("music.mp3")));

        // Test with directory (should return false)
        assert!(!is_code_file(Path::new("src/")));
    }
}
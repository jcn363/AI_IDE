//! Utility functions for the specification generation system

use std::path::Path;

use anyhow;

/// Sanitizes input strings to valid Rust identifiers by allowing only ASCII letters and digits.
/// Leading digits are handled by prefixing an underscore.
/// Reserved keywords receive a trailing underscore.
/// Sanitizing identifiers ensures generated names are valid Rust identifiers to prevent compilation
/// errors. Additionally, if the input is empty or contains only invalid characters, the function
/// returns a valid fallback identifier such as an underscore.
///
/// # Examples
/// ```
/// assert_eq!(sanitize_identifier("valid_name"), "valid_name");
/// assert_eq!(sanitize_identifier("123abc"), "_123abc");
/// assert_eq!(sanitize_identifier("fn"), "fn_");
/// assert_eq!(sanitize_identifier("hello🎉"), "hello_");
/// ```
pub fn sanitize_identifier(name: &str) -> String {
    if name.is_empty() {
        return "_".to_string();
    }

    let mut result = String::with_capacity(name.len());
    let mut chars = name.chars();

    // Handle first character
    if let Some(first) = chars.next() {
        if first.is_ascii_alphabetic() || first == '_' {
            result.push(first);
        } else if first.is_ascii_digit() {
            result.push('_');
            result.push(first);
        } else {
            result.push('_');
        }
    }

    // Handle remaining characters
    for c in chars {
        if c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '_' {
            result.push(c);
        } else if !result.ends_with('_') {
            result.push('_');
        }
    }

    // Ensure it doesn't start with digit (additional safety)
    if let Some(b) = result.as_bytes().first() {
        if !b.is_ascii_alphabetic() && *b != b'_' {
            result.insert(0, '_');
        }
    }

    // Ensure the identifier is not a Rust keyword
    if is_rust_keyword(&result) {
        result.push('_');
    }

    result
}

/// Check if a string is a Rust keyword
pub fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "try"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
    )
}

/// Convert a string to snake_case
pub fn to_snake_case(s: &str) -> String {
    use heck::ToSnakeCase;
    s.to_snake_case()
}

/// Convert a string to PascalCase
pub fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' || c == '.' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Convert a string to camelCase
pub fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    let mut first_char = true;

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' || c == '.' {
            capitalize_next = true;
        } else if capitalize_next || (!first_char && c.is_uppercase()) {
            result.extend(c.to_uppercase());
            capitalize_next = false;
            first_char = false;
        } else if first_char {
            result.extend(c.to_lowercase());
            first_char = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Normalize a file path to use forward slashes
pub fn normalize_path(path: &Path) -> String {
    let mut result = String::new();

    for component in path.components() {
        if !result.is_empty() && !result.ends_with('/') {
            result.push('/');
        }
        result.push_str(&component.as_os_str().to_string_lossy());
    }

    result
}

/// Extract the base name from a path
pub fn base_name(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}

/// Get the directory name from a path
pub fn dir_name(path: &str) -> String {
    Path::new(path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_string()
}

/// Ensure a directory exists, creating it if necessary
pub fn ensure_dir(path: &str) -> std::io::Result<()> {
    if !Path::new(path).exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Format Rust code using rustfmt. rustfmt must be available in the system PATH.
/// This function assumes rustfmt is installed and accessible.
/// For non-blocking operation, consider running this in a timeout context.
/// An optional feature flag can be used to disable formatting in CI environments.
///
/// # Note
/// - rustfmt is required and must be executable.
/// - In blocking scenarios, wrap calls in timeout mechanisms.
/// - Consider using a feature flag for conditional formatting.
pub fn format_rust_code(code: &str) -> Result<String, anyhow::Error> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    // Create a child process with rustfmt
    let mut child = Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn rustfmt process: {}", e))?;

    // Write the code to rustfmt's stdin
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(code.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to write code to rustfmt stdin: {}", e))?;
        drop(stdin); // explicitly drop to prevent hanging
    }

    // Read the formatted output
    let output = child
        .wait_with_output()
        .map_err(|e| anyhow::anyhow!("Failed to wait for rustfmt: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!(
            "rustfmt failed with exit code {}: {}",
            output.status,
            stderr
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_identifier() {
        assert_eq!(sanitize_identifier("valid_identifier"), "valid_identifier");
        assert_eq!(sanitize_identifier("123invalid"), "_123invalid");
        assert_eq!(sanitize_identifier("with spaces"), "with_spaces");
        assert_eq!(sanitize_identifier("with-dashes"), "with_dashes");
        assert_eq!(sanitize_identifier("with.dots"), "with_dots");
        assert_eq!(sanitize_identifier("fn"), "fn_");
        assert_eq!(sanitize_identifier(""), "_");
        assert_eq!(sanitize_identifier("hello🎉"), "hello_");
        assert_eq!(sanitize_identifier("!invalid"), "_invalid");
        // Test more Unicode characters
        assert_eq!(sanitize_identifier("🦀 rust"), "_rust");
        assert_eq!(sanitize_identifier("🚀launch"), "_launch");
        assert_eq!(sanitize_identifier("αβγ"), "_");
        assert_eq!(sanitize_identifier("valid🐛"), "valid_");
    }

    #[test]
    fn test_is_rust_keyword() {
        assert!(is_rust_keyword("fn"));
        assert!(is_rust_keyword("struct"));
        assert!(!is_rust_keyword("not_a_keyword"));
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("camelCase"), "camel_case");
        assert_eq!(to_snake_case("PascalCase"), "pascal_case");
        assert_eq!(to_snake_case("kebab-case"), "kebab_case");
        assert_eq!(to_snake_case("with spaces"), "with_spaces");
        assert_eq!(to_snake_case("already_snake_case"), "already_snake_case");
        assert_eq!(to_snake_case("HTTPRequest"), "http_request");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("snake_case"), "SnakeCase");
        assert_eq!(to_pascal_case("kebab-case"), "KebabCase");
        assert_eq!(to_pascal_case("with spaces"), "WithSpaces");
        assert_eq!(to_pascal_case("alreadyPascalCase"), "AlreadyPascalCase");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("snake_case"), "snakeCase");
        assert_eq!(to_camel_case("kebab-case"), "kebabCase");
        assert_eq!(to_camel_case("with spaces"), "withSpaces");
        assert_eq!(to_camel_case("alreadyCamelCase"), "alreadyCamelCase");
        assert_eq!(to_camel_case("PascalCase"), "pascalCase");
    }

    #[test]
    fn test_normalize_path() {
        let path = Path::new("some/dir/../file.txt");
        let normalized = normalize_path(path);
        assert_eq!(normalized, "some/dir/../file.txt");
    }

    #[cfg(windows)]
    #[test]
    fn test_normalize_path_windows() {
        let path_str = "C:\\path\\to\\file.txt";
        let path = Path::new(path_str);
        let normalized = normalize_path(path);
        assert_eq!(normalized, "C:/path/to/file.txt");
    }

    #[test]
    fn test_base_name() {
        assert_eq!(base_name("path/to/file.txt"), "file");
        assert_eq!(base_name("file.txt"), "file");
        assert_eq!(base_name("path/to/dir/"), "");
    }

    #[test]
    fn test_dir_name() {
        assert_eq!(dir_name("path/to/file.txt"), "path/to");
        assert_eq!(dir_name("file.txt"), "");
        assert_eq!(dir_name("path/to/dir/"), "path/to");
    }
}

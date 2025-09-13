//! Common test helpers for Rust AI IDE

/// Test environment configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub verbose: bool,
    pub cleanup: bool,
    pub timeout: std::time::Duration,
    pub workspace_root: Option<std::path::PathBuf>,
}

/// Generate a temporary test directory
#[cfg(feature = "filesystem")]
pub fn create_temp_dir(name: &str) -> std::io::Result<std::path::PathBuf> {
    use std::path::PathBuf;

    let temp_dir = std::env::temp_dir().join(format!("rust-ai-ide-test-{}", name));
    std::fs::create_dir_all(&temp_dir)?;
    temp_dir.canonicalize()
}

/// Create a sample Rust file for testing
#[cfg(feature = "filesystem")]
pub fn create_sample_rust_file(
    dir: &std::path::Path,
    name: &str,
) -> std::io::Result<std::path::PathBuf> {
    use std::fs;
    use std::path::PathBuf;

    let file_path = dir.join(format!("src/{}.rs", name));
    fs::create_dir_all(file_path.parent().unwrap())?;

    let content = format!(
        r#"//! Sample Rust file for testing: {}

/// A simple function to test code analysis
pub fn example_function_{}() -> String {{
    "Hello from test".to_string()
}}

/// Another function
pub fn another_function_{}() -> i32 {{
    println!("This is a test function");
    42
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_example_function_{}() {{
        assert_eq!(example_function_{}().contains("Hello"), true);
    }}

    #[test]
    fn test_another_function_{}() {{
        assert_eq!(another_function_{}(), 42);
    }}
}}
"#,
        name, name, name, name, name, name, name
    );

    fs::write(&file_path, content)?;
    Ok(file_path)
}

/// Create a sample Cargo.toml for testing
#[cfg(feature = "filesystem")]
pub fn create_sample_cargo_toml(
    dir: &std::path::Path,
    name: &str,
) -> std::io::Result<std::path::PathBuf> {
    use std::fs;
    use std::path::PathBuf;

    let file_path = dir.join("Cargo.toml");
    let content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
authors = ["Test Author <test@example.com>"]
description = "Test project for Rust AI IDE"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
tokio = {{ version = "1.0", features = ["full"] }}

[features]
default = []
test_feature = []

[lib]
path = "src/lib.rs"

[[bin]]
name = "main"
path = "src/main.rs"

[package.metadata]
custom_field = "test_value"
"#,
        name
    );

    fs::write(&file_path, content)?;
    Ok(file_path)
}

/// Create a complete test project structure
#[cfg(feature = "filesystem")]
pub fn create_test_project(name: &str) -> std::io::Result<std::path::PathBuf> {
    let project_dir = create_temp_dir(name)?;

    // Create Cargo.toml
    create_sample_cargo_toml(&project_dir, name)?;

    // Create src directory structure
    std::fs::create_dir_all(project_dir.join("src"))?;

    // Create lib.rs
    create_sample_rust_file(&project_dir, "lib")?;

    // Create main.rs
    let main_rs_path = project_dir.join("src/main.rs");
    let main_content = format!(
        r#"//! Test main.rs for {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    println!("Hello from {}!", "{}");
    Ok(())
}}
"#,
        name, name, name
    );
    std::fs::write(&main_rs_path, main_content)?;

    Ok(project_dir)
}

/// Wait for async operation to complete with timeout
#[cfg(feature = "async")]
pub async fn wait_with_timeout<T, F>(future: F, timeout: std::time::Duration) -> Option<T>
where
    F: std::future::Future<Output = T>,
{
    tokio::time::timeout(timeout, future).await.ok()
}

/// Mock async delay for testing
#[cfg(feature = "async")]
pub async fn mock_delay(duration: std::time::Duration) {
    tokio::time::sleep(duration).await;
}

/// Collect test output into a string
#[cfg(feature = "metrics")]
pub fn collect_test_output<F>(f: F) -> String
where
    F: FnOnce(&mut Vec<u8>) -> std::io::Result<()>,
{
    let mut buffer = Vec::new();
    let _ = f(&mut buffer);
    String::from_utf8_lossy(&buffer).into_owned()
}

/// Generate compilation error samples
#[cfg(feature = "std")]
pub fn generate_sample_compilation_error() -> String {
    r#"error[E0308]: mismatched types
  --> src/lib.rs:3:12
   |
3  |     42
   |     ^^ expected `i32`, found `i64`
help: you can cast `42 as i32` or change the type annotation"#
        .to_string()
}

/// Generate sample cargo output
#[cfg(feature = "std")]
pub fn generate_sample_cargo_output() -> String {
    r#"Compiling test v0.1.0 (/tmp/test)
Finished dev [unoptimized + debuginfo] target(s) in 0.45s
Running `target/debug/test`
Hello, World!"#
        .to_string()
}

/// Create sample JSON for testing
#[cfg(feature = "metrics")]
pub fn create_sample_json_data() -> serde_json::Value {
    serde_json::json!({
        "workspace": {
            "name": "test-workspace",
            "packages": [
                {
                    "name": "test-package",
                    "version": "0.1.0",
                    "dependencies": ["serde", "tokio"]
                }
            ]
        },
        "diagnostics": [
            {
                "file": "src/main.rs",
                "line": 5,
                "message": "Unused variable",
                "level": "warning"
            }
        ]
    })
}

/// Test trait for mocking services
#[cfg(feature = "std")]
pub trait TestableService {
    fn is_mock(&self) -> bool;
}

/// Default implementation for services
#[cfg(feature = "std")]
impl<T> TestableService for T {
    default fn is_mock(&self) -> bool {
        false
    }
}

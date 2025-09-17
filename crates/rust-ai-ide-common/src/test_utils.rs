//! # Test Utilities Module
//!
//! This module provides shared test utilities across the Rust AI IDE codebase to:
//! - Eliminate duplication in test setup/teardown code
//! - Standardize common mock implementations
//! - Provide reusable test data generators
//! - Assist with integration testing patterns
//! - Support performance testing utilities
//!
//! ## Usage
//!
//! Add to your test file:
//! ```rust
//! use rust_ai_ide_common::test_utils::*;
//! ```
//!
//! ## Features
//!
//! - **Test Setup Helpers**: Convenient setup for temporary directories, files, and project
//!   structures
//! - **Mock Implementations**: Reusable mock objects for common dependencies
//! - **Test Data Generators**: Factory functions for creating sample data (files, manifests, etc.)
//! - **Integration Helpers**: Utilities for testing command handlers and event systems
//! - **Performance Utilities**: Helpers for benchmarking and performance profiling

#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;
#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use tokio::sync::Mutex;

#[cfg(test)]
pub mod setup {
    //! # Test Setup and Teardown Utilities
    //!
    //! Provides common setup/cleanup patterns for tests including:
    //! - Temporary directory management
    //! - Test file creation
    //! - Project structure initialization

    use tempfile::{tempdir, TempDir};

    use super::*;

    /// Wrapper for temporary directory that provides convenient cleanup
    // Reusable shared test utilities - may be used across crates
    pub struct TestTempDir {
        path: PathBuf,
        _dir: TempDir,
    }

    impl TestTempDir {
        /// Create a new temporary directory for testing
        pub fn new(prefix: &str) -> Self {
            let dir = tempdir().expect("Could not create temporary directory");
            let path = dir.path().to_path_buf();

            TestTempDir { path, _dir: dir }
        }

        /// Get the path of the temporary directory
        pub fn path(&self) -> &Path {
            &self.path
        }

        /// Initialize a Rust project structure in this directory
        pub fn init_rust_project(&self, name: &str, version: &str) -> PathBuf {
            self.create_cargo_toml(name, version);
            self.create_main_rs();
            self.path().join("src")
        }

        /// Create a Cargo.toml file with basic configuration
        pub fn create_cargo_toml(&self, name: &str, version: &str) {
            let cargo_toml_path = self.path().join("Cargo.toml");
            let cargo_toml_content = format!(
                r#"[package]
name = "{}"
version = "{}"
edition = "2021"

[dependencies]
serde = "1.0"
"#,
                name, version
            );

            fs::write(&cargo_toml_path, cargo_toml_content).expect("Could not write Cargo.toml");
        }

        /// Create a basic lib.rs file
        pub fn create_main_rs(&self) {
            let src_dir = self.path().join("src");
            fs::create_dir_all(&src_dir).expect("Could not create src directory");

            let lib_rs_path = src_dir.join("lib.rs");
            let lib_rs_content = r#"pub fn hello_world() -> String {
    "Hello World!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        assert_eq!(hello_world(), "Hello World!");
    }
}
"#;

            fs::write(lib_rs_path, lib_rs_content).expect("Could not write lib.rs");
        }

        /// Create arbitrary test content in a file
        pub fn create_file<P: AsRef<Path>>(&self, path: P, content: &str) {
            let full_path = self.path().join(path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).expect("Could not create parent directories");
            }
            fs::write(full_path, content).expect("Could not write test file");
        }
    }

    /// Setup helper for creating test files with auto-cleanup
    // Reusable shared test utilities - may be used across crates
    pub fn with_test_file<F>(content: &str, test_fn: F)
    where
        F: FnOnce(&Path) -> (),
    {
        let temp_dir = tempdir().expect("Could not create temp directory");
        let file_path = temp_dir.path().join("test-file");

        fs::write(&file_path, content).expect("Could not write test file");
        test_fn(&file_path);
    }
}

#[cfg(test)]
pub mod mocks {
    //! # Common Mock Implementations
    //!
    //! Provides reusable mock objects and traits for testing:
    //! - File system operations
    //! - Network responses
    //! - Time operations
    //! - Event emitters

    use std::io::{self, Read};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    /// Mock file system with in-memory storage
    // Reusable shared test utilities - may be used across crates
    pub struct MockFileSystem {
        files: HashMap<PathBuf, Vec<u8>>,
        root: PathBuf,
    }

    impl MockFileSystem {
        /// Create a new mock file system with given root
        pub fn new(root: PathBuf) -> Self {
            MockFileSystem {
                files: HashMap::new(),
                root,
            }
        }

        /// Write data to a mock file
        pub fn write_file(&mut self, path: PathBuf, data: &[u8]) {
            self.files.insert(path, data.to_vec());
        }

        /// Read data from a mock file
        pub fn read_file(&self, path: &Path) -> Option<Vec<u8>> {
            self.files.get(path).cloned()
        }

        /// Check if mock file exists
        pub fn file_exists(&self, path: &Path) -> bool {
            self.files.contains_key(path)
        }

        /// Get list of all file paths
        pub fn list_files(&self) -> Vec<PathBuf> {
            self.files.keys().cloned().collect()
        }
    }

    /// Mock time provider that allows time manipulation
    // Reusable shared test utilities - may be used across crates
    pub struct MockTimeProvider {
        current_time: std::time::SystemTime,
    }

    impl MockTimeProvider {
        /// Create with current system time
        pub fn new() -> Self {
            MockTimeProvider {
                current_time: SystemTime::now(),
            }
        }

        /// Get current time
        pub fn now(&self) -> SystemTime {
            self.current_time
        }

        /// Advance time by specified duration
        pub fn advance(&mut self, duration: std::time::Duration) {
            self.current_time += duration;
        }

        /// Set to a specific time
        pub fn set_time(&mut self, time: SystemTime) {
            self.current_time = time;
        }
    }

    /// Mock network response builder
    // Reusable shared test utilities - may be used across crates
    pub struct MockHttpResponse {
        status: u16,
        body: Vec<u8>,
        headers: HashMap<String, String>,
    }

    impl MockHttpResponse {
        /// Create a successful response
        pub fn ok(content_type: &str, body: &str) -> Self {
            let mut headers = HashMap::new();
            headers.insert("content-type".to_string(), content_type.to_string());

            MockHttpResponse {
                status: 200,
                body: body.as_bytes().to_vec(),
                headers,
            }
        }

        /// Create an error response
        pub fn error(status: u16, body: &str) -> Self {
            MockHttpResponse {
                status,
                body: body.as_bytes().to_vec(),
                headers: HashMap::new(),
            }
        }

        /// Get response status
        pub fn status(&self) -> u16 {
            self.status
        }

        /// Get response body
        pub fn body(&self) -> &[u8] {
            &self.body
        }

        /// Get a header value
        pub fn header(&self, key: &str) -> Option<&String> {
            self.headers.get(key)
        }
    }
}

#[cfg(test)]
pub mod generators {
    //! # Test Data Generators
    //!
    //! Factory functions for creating sample test data including:
    //! - Sample Rust source files
    //! - Cargo.toml manifests
    //! - Project structures
    //! - Configuration data

    use serde_json;

    use super::*;

    /// Generate sample Rust source code
    // Reusable shared test utilities - may be used across crates
    pub fn sample_rust_code() -> String {
        r#"pub fn calculate_factorial(n: u64) -> u64 {
    if n <= 1 {
        return 1;
    }
    n * calculate_factorial(n - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_factorial() {
        assert_eq!(calculate_factorial(0), 1);
        assert_eq!(calculate_factorial(1), 1);
        assert_eq!(calculate_factorial(5), 120);
    }
}"#
        .to_string()
    }

    /// Generate sample Cargo.toml manifest
    // Reusable shared test utilities - may be used across crates
    pub fn sample_cargo_toml(name: &str, version: &str) -> String {
        format!(
            r#"[package]
name = "{}"
version = "{}"
edition = "2021"
authors = ["Test Author <test@example.com>"]
description = "Test Rust project"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
tokio = {{ version = "1.0", features = ["full"] }}
anyhow = "1.0"

[features]
default = []
full = ["serde", "tokio"]
"#,
            name, version
        )
    }

    /// Generate sample JSON configuration
    // Reusable shared test utilities - may be used across crates
    pub fn sample_config_json() -> String {
        r#"{
    "analysis": {
        "enabled": true,
        "timeout": 60
    },
    "logging": {
        "level": "info",
        "file": "debug.log"
    },
    "features": [
        "code-analysis",
        "invoke-ai",
        "refactoring"
    ]
}"#
        .to_string()
    }

    /// Generate sample test error responses
    // Reusable shared test utilities - may be used across crates
    pub fn sample_error_response(code: &str, message: &str) -> String {
        serde_json::json!({
            "error": {
                "code": code,
                "message": message,
                "details": {}
            },
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
        .to_string()
    }

    /// Generate random test data of specified size
    // Reusable shared test utilities - may be used across crates
    pub fn random_data(size: usize) -> Vec<u8> {
        use std::iter;
        iter::repeat(())
            .take(size)
            .map(|_| rand::random::<u8>())
            .collect()
    }
}

#[cfg(test)]
pub mod integration {
    //! # Integration Test Helpers
    //!
    //! Utilities for testing command handlers and event systems:
    //! - Command execution helpers
    //! - Event bus testing
    //! - State management helpers

    use std::future::Future;
    use std::pin::Pin;

    use serde_json::Value;

    use super::*;

    /// Helper for executing and validating commands in integration tests
    // Reusable shared test utilities - may be used across crates
    pub struct CommandTester {
        results: Vec<Result<Value, String>>,
    }

    impl CommandTester {
        /// Create new command tester
        pub fn new() -> Self {
            CommandTester {
                results: Vec::new(),
            }
        }

        /// Execute a command and store result
        pub async fn execute_command<F, Fut>(&mut self, command_fn: F, input: Value) -> &mut Self
        where
            F: FnOnce(Value) -> Fut,
            Fut: Future<Output = Result<Value, String>>,
        {
            let result = command_fn(input).await;
            self.results.push(result);
            self
        }

        /// Assert that the last command succeeded with expected output
        pub fn assert_success(&self, expected: &str) {
            let last_result = self.results.last().expect("No command executed");
            match last_result {
                Ok(value) => {
                    let json_str = value.to_string();
                    assert!(
                        json_str.contains(expected),
                        "Output '{}' doesn't contain '{}'",
                        json_str,
                        expected
                    );
                }
                Err(err) => panic!("Command failed: {}", err),
            }
        }

        /// Assert that the last command failed
        pub fn assert_failure(&self) {
            let last_result = self.results.last().expect("No command executed");
            assert!(last_result.is_err(), "Command should have failed");
        }

        /// Get all results for detailed analysis
        pub fn results(&self) -> &[Result<Value, String>] {
            &self.results
        }
    }

    /// Mock event emitter for testing event-driven systems
    // Reusable shared test utilities - may be used across crates
    pub struct MockEventEmitter {
        events: Arc<Mutex<Vec<(String, Value)>>>,
    }

    impl MockEventEmitter {
        /// Create new mock event emitter
        pub fn new() -> Self {
            MockEventEmitter {
                events: Arc::new(Mutex::new(Vec::new())),
            }
        }

        /// Emit an event
        pub async fn emit(&self, event_type: &str, data: Value) {
            let mut events = self.events.lock().await;
            events.push((event_type.to_string(), data));
        }

        /// Get all emitted events
        pub async fn events(&self) -> Vec<(String, Value)> {
            let events = self.events.lock().await;
            events.clone()
        }

        /// Assert that a specific event was emitted
        pub async fn assert_event_emitted(&self, event_type: &str, expected_data: Option<Value>) {
            let events = self.events.lock().await;
            let event_emitted = events.iter().any(|(type_, data)| {
                type_ == event_type
                    && match expected_data {
                        Some(ref expected) => data == expected,
                        None => true,
                    }
            });
            assert!(
                event_emitted,
                "Event '{}' was not emitted as expected",
                event_type
            );
        }
    }
}

#[cfg(test)]
pub mod performance {
    //! # Performance Testing Utilities
    //!
    //! Helpers for benchmarking and performance profiling:
    //! - Timing utilities
    //! - Memory usage tracking
    //! - Benchmark result analysis

    use std::time::{Duration, Instant};

    use super::*;

    /// Simple performance timer
    // Reusable shared test utilities - may be used across crates
    pub struct PerformanceTimer {
        name: String,
        start: Instant,
    }

    impl PerformanceTimer {
        /// Start timing a named operation
        pub fn start(name: &str) -> Self {
            PerformanceTimer {
                name: name.to_string(),
                start: Instant::now(),
            }
        }

        /// Stop timing and return duration
        pub fn stop(self) -> Duration {
            let duration = self.start.elapsed();
            println!("{} took: {:2} μs", self.name, duration.as_micros());
            duration
        }

        /// Get elapsed time without stopping
        pub fn elapsed(&self) -> Duration {
            self.start.elapsed()
        }
    }

    /// Benchmark runner for repeated operations
    // Reusable shared test utilities - may be used across crates
    pub struct BenchmarkRunner {
        iterations: usize,
        setup_fn: Option<Box<dyn Fn()>>,
        teardown_fn: Option<Box<dyn Fn()>>,
    }

    impl BenchmarkRunner {
        /// Create benchmark runner with given iterations
        pub fn new(iterations: usize) -> Self {
            BenchmarkRunner {
                iterations,
                setup_fn: None,
                teardown_fn: None,
            }
        }

        /// Add setup function to run before benchmarks
        pub fn with_setup<F>(mut self, setup_fn: F) -> Self
        where
            F: Fn() + 'static,
        {
            self.setup_fn = Some(Box::new(setup_fn));
            self
        }

        /// Run benchmark on closure
        pub fn run<F>(mut self, benchmark_fn: F) -> BenchmarkResult
        where
            F: Fn(),
        {
            if let Some(setup) = &self.setup_fn {
                setup();
            }

            let mut times = Vec::with_capacity(self.iterations);

            for _ in 0..self.iterations {
                let timer = PerformanceTimer::start("");
                benchmark_fn();
                times.push(timer.stop());
            }

            BenchmarkResult::new(times)
        }
    }

    /// Results of a benchmark run
    // Reusable shared test utilities - may be used across crates
    pub struct BenchmarkResult {
        times: Vec<Duration>,
    }

    impl BenchmarkResult {
        /// Create from measurement times
        pub fn new(times: Vec<Duration>) -> Self {
            BenchmarkResult { times }
        }

        /// Get average time
        pub fn average(&self) -> Duration {
            let total: Duration = self.times.iter().sum();
            total / self.times.len() as u32
        }

        /// Get minimum time
        pub fn min(&self) -> Duration {
            *self.times.iter().min().unwrap()
        }

        /// Get maximum time
        pub fn max(&self) -> Duration {
            *self.times.iter().max().unwrap()
        }

        /// Get all individual times
        pub fn times(&self) -> &[Duration] {
            &self.times
        }

        /// Print detailed results
        pub fn print(&self, operation_name: &str) {
            println!("Benchmark results for '{}':", operation_name);
            println!("  Iterations: {}", self.times.len());
            println!("  Average: {:2} μs", self.average().as_micros());
            println!("  Min: {:2} μs", self.min().as_micros());
            println!("  Max: {:2} μs", self.max().as_micros());
        }
    }
}

// Re-export all utilities at the module level for convenience
#[cfg(test)]
pub use generators::*;
#[cfg(test)]
pub use integration::*;
#[cfg(test)]
pub use mocks::*;
#[cfg(test)]
pub use performance::*;
#[cfg(test)]
pub use setup::*;

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_test_temp_dir_creation() {
        let temp_dir = setup::TestTempDir::new("test-prefix");
        assert!(temp_dir.path().exists());
        assert!(temp_dir.path().is_dir());
    }

    #[test]
    fn test_mock_file_system() {
        let mut fs = mocks::MockFileSystem::new(PathBuf::from("/test"));
        let path = PathBuf::from("test.txt");
        let data = b"Hello World";

        fs.write_file(path.clone(), data);
        assert_eq!(fs.read_file(&path), Some(data.to_vec()));
        assert!(fs.file_exists(&path));
    }

    #[test]
    fn test_performance_timer() {
        let timer = performance::PerformanceTimer::start("test");
        // Simulate some work
        std::thread::sleep(Duration::from_millis(1));
        let duration = timer.stop();
        assert!(duration.as_millis() >= 1);
    }
}

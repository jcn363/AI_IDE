//! Unified mock framework for testing
//!
//! Provides common mock patterns, behaviors, and utilities that can be
//! reused across different testing scenarios in the workspace.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg(feature = "mock")]
use mockall::mock;

use crate::error::TestError;

/// Generic mock implementation with configurable behavior
pub struct GenericMock<T> {
    behaviors:    Arc<Mutex<HashMap<String, Box<dyn Fn(&[&dyn std::any::Any]) -> T + Send + Sync>>>>,
    call_history: Arc<Mutex<Vec<(String, Vec<String>)>>>,
}

impl<T> std::fmt::Debug for GenericMock<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let call_history = self.call_history.lock().unwrap();
        f.debug_struct("GenericMock")
            .field("call_history", &*call_history)
            .finish_non_exhaustive()
    }
}

impl<T: Clone + 'static> GenericMock<T> {
    pub fn new() -> Self {
        Self {
            behaviors:    Arc::new(Mutex::new(HashMap::new())),
            call_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register behavior for a method
    pub fn when<F>(&self, method: &str, behavior: F)
    where
        F: Fn(&[&dyn std::any::Any]) -> T + Send + Sync + 'static,
    {
        self.behaviors
            .lock()
            .unwrap()
            .insert(method.to_string(), Box::new(behavior));
    }

    /// Execute a mock method
    pub fn execute(&self, method: &str, args: &[&dyn std::any::Any]) -> T {
        self.call_history.lock().unwrap().push((
            method.to_string(),
            args.iter().map(|arg| format!("{:?}", arg)).collect(),
        ));

        if let Some(behavior) = self.behaviors.lock().unwrap().get(method) {
            behavior(args)
        } else {
            panic!("No behavior defined for method: {}", method);
        }
    }

    /// Verify that a method was called
    pub fn verify_called(&self, method: &str) -> bool {
        self.call_history
            .lock()
            .unwrap()
            .iter()
            .any(|(m, _)| m == method)
    }

    /// Verify that a method was called with specific arguments
    pub fn verify_called_with(&self, method: &str, args: &[&str]) -> bool {
        self.call_history.lock().unwrap().iter().any(|(m, a)| {
            m == method
                && a.len() == args.len()
                && a.iter()
                    .zip(args)
                    .all(|(actual, expected)| actual.contains(expected))
        })
    }

    /// Get call history
    pub fn call_history(&self) -> Vec<(String, Vec<String>)> {
        self.call_history.lock().unwrap().clone()
    }

    /// Reset the mock state
    pub fn reset(&self) {
        self.call_history.lock().unwrap().clear();
    }
}

impl<T: Clone + 'static> Default for GenericMock<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Behavior presets for common mock scenarios
pub struct MockBehaviors;

impl MockBehaviors {
    /// Return a success result
    ///
    /// This works with both `Result` and non-`Result` return types.
    /// For `Result` types, it will return `Ok(value)`. For non-`Result` types,
    /// it will return the value directly.
    pub fn success<T, U, F>(value_fn: F) -> impl Fn(&[&dyn std::any::Any]) -> T
    where
        F: Fn() -> U + 'static,
        U: Into<T>,
        T: 'static,
    {
        move |_| value_fn().into()
    }

    /// Return an error
    pub fn error<T>(error: String) -> impl Fn(&[&dyn std::any::Any]) -> Result<T, TestError> {
        move |_| {
            Err(TestError::Validation(
                crate::ValidationError::invalid_setup(error.clone()),
            ))
        }
    }

    /// Return based on argument value
    pub fn conditional<T, F>(condition: F, true_value: T, false_value: T) -> impl Fn(&[&dyn std::any::Any]) -> T
    where
        F: Fn(&[&dyn std::any::Any]) -> bool,
        T: Clone,
    {
        move |args| {
            if condition(args) {
                true_value.clone()
            } else {
                false_value.clone()
            }
        }
    }

    /// Sequence of return values
    pub fn sequence<T: Clone>(values: Vec<T>) -> impl Fn(&[&dyn std::any::Any]) -> T {
        let counter = Arc::new(Mutex::new(0));
        move |_| {
            let mut count = counter.lock().unwrap();
            let value = values[*count % values.len()].clone();
            *count += 1;
            value
        }
    }

    /// Throw panic on call
    pub fn panic<T>(message: String) -> impl Fn(&[&dyn std::any::Any]) -> T {
        move |_| panic!("{}", message)
    }
}

/// Mock factory for creating and managing multiple mocks
pub struct MockFactory {
    mocks: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl MockFactory {
    pub fn new() -> Self {
        Self {
            mocks: HashMap::new(),
        }
    }

    /// Register a mock with a name
    pub fn register<T: 'static + Send + Sync>(&mut self, name: &str, mock: T) {
        self.mocks
            .insert(name.to_string(), Box::new(Arc::new(Mutex::new(mock))));
    }

    /// Get a mock by name
    pub fn get<T: 'static>(&self, name: &str) -> Option<&Arc<Mutex<T>>> {
        self.mocks.get(name)?.downcast_ref::<Arc<Mutex<T>>>()
    }

    /// Create all registered mocks with default behaviors
    pub fn setup_defaults(&mut self) -> Result<(), TestError> {
        // Setup default behavior for common mock types
        Ok(())
    }

    /// Reset all mocks
    pub fn reset_all(&self) {
        // Reset logic would go here for mockall mocks
    }
}

impl Default for MockFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock scenario builder for complex test setups
pub struct MockScenario {
    name:           String,
    setup_steps:    Vec<Box<dyn FnOnce() -> Result<(), TestError> + Send>>,
    teardown_steps: Vec<Box<dyn FnOnce() -> Result<(), TestError> + Send>>,
}

impl MockScenario {
    pub fn new(name: &str) -> Self {
        Self {
            name:           name.to_string(),
            setup_steps:    Vec::new(),
            teardown_steps: Vec::new(),
        }
    }

    /// Add a setup step
    pub fn with_setup<F>(mut self, setup_fn: F) -> Self
    where
        F: FnOnce() -> Result<(), TestError> + Send + 'static,
    {
        self.setup_steps.push(Box::new(setup_fn));
        self
    }

    /// Add a teardown step
    pub fn with_teardown<F>(mut self, teardown_fn: F) -> Self
    where
        F: FnOnce() -> Result<(), TestError> + Send + 'static,
    {
        self.teardown_steps.push(Box::new(teardown_fn));
        self
    }

    /// Execute setup steps
    pub fn setup(mut self) -> Result<Self, TestError> {
        for step in self.setup_steps.drain(..) {
            step()?;
        }
        Ok(self)
    }

    /// Execute teardown steps
    pub fn teardown(mut self) -> Result<(), TestError> {
        for step in self.teardown_steps.drain(..) {
            step()?;
        }
        Ok(())
    }
}

/// HTTP mock utilities for API testing
pub struct HttpMock {
    endpoints: HashMap<String, MockHttpResponse>,
    history:   Arc<Mutex<Vec<MockHttpRequest>>>,
}

#[derive(Debug, Clone)]
pub struct MockHttpRequest {
    pub method:  String,
    pub url:     String,
    pub headers: HashMap<String, String>,
    pub body:    Option<String>,
}

#[derive(Debug, Clone)]
pub struct MockHttpResponse {
    pub status:  u16,
    pub headers: HashMap<String, String>,
    pub body:    Option<String>,
}

impl HttpMock {
    pub fn new() -> Self {
        Self {
            endpoints: HashMap::new(),
            history:   Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a mock HTTP endpoint
    pub fn mock_endpoint(&mut self, method: &str, path: &str, response: MockHttpResponse) {
        let key = format!("{} {}", method.to_uppercase(), path);
        self.endpoints.insert(key, response);
    }

    /// Simulate an HTTP request
    pub fn request(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<String>,
    ) -> Option<MockHttpResponse> {
        let request = MockHttpRequest {
            method:  method.to_string(),
            url:     url.to_string(),
            headers: headers.clone(),
            body:    body.clone(),
        };

        self.history.lock().unwrap().push(request);

        let key = format!("{} {}", method.to_uppercase(), url);
        self.endpoints.get(&key).cloned()
    }

    /// Get request history
    pub fn request_history(&self) -> Vec<MockHttpRequest> {
        self.history.lock().unwrap().clone()
    }

    /// Verify endpoint was called
    pub fn verify_called(&self, method: &str, url: &str) -> bool {
        self.history
            .lock()
            .unwrap()
            .iter()
            .any(|req| req.method.to_uppercase() == method.to_uppercase() && req.url == url)
    }
}

impl Default for HttpMock {
    fn default() -> Self {
        Self::new()
    }
}

/// File system mock for testing file operations
pub struct FileSystemMock {
    files:       Arc<Mutex<HashMap<String, String>>>,
    directories: Arc<Mutex<HashMap<String, Vec<String>>>>,
    history:     Arc<Mutex<Vec<FileOperation>>>,
}

#[derive(Debug, Clone)]
pub enum FileOperation {
    Read(String),
    Write(String, String),
    Delete(String),
    CreateDir(String),
    ListDir(String),
}

impl FileSystemMock {
    pub fn new() -> Self {
        Self {
            files:       Arc::new(Mutex::new(HashMap::new())),
            directories: Arc::new(Mutex::new(HashMap::new())),
            history:     Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Read a file
    pub fn read_file(&self, path: &str) -> Result<String, TestError> {
        self.history
            .lock()
            .unwrap()
            .push(FileOperation::Read(path.to_string()));

        if let Some(content) = self.files.lock().unwrap().get(path) {
            Ok(content.clone())
        } else {
            Err(TestError::Validation(
                crate::ValidationError::invalid_setup(format!("File not found: {}", path)),
            ))
        }
    }

    /// Write to a file
    pub fn write_file(&self, path: &str, content: &str) -> Result<(), TestError> {
        self.history
            .lock()
            .unwrap()
            .push(FileOperation::Write(path.to_string(), content.to_string()));
        self.files
            .lock()
            .unwrap()
            .insert(path.to_string(), content.to_string());
        Ok(())
    }

    /// Check if file exists
    pub fn file_exists(&self, path: &str) -> bool {
        self.files.lock().unwrap().contains_key(path)
    }

    /// Create directory
    pub fn create_dir(&self, path: &str) -> Result<(), TestError> {
        self.history
            .lock()
            .unwrap()
            .push(FileOperation::CreateDir(path.to_string()));
        self.directories
            .lock()
            .unwrap()
            .insert(path.to_string(), Vec::new());
        Ok(())
    }

    /// List directory contents
    pub fn list_dir(&self, path: &str) -> Result<Vec<String>, TestError> {
        self.history
            .lock()
            .unwrap()
            .push(FileOperation::ListDir(path.to_string()));

        if let Some(contents) = self.directories.lock().unwrap().get(path) {
            Ok(contents.clone())
        } else {
            Ok(Vec::new())
        }
    }

    /// Add initial file to the mock
    pub fn add_file(&self, path: &str, content: &str) {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_string(), content.to_string());
    }

    /// Add initial directory to the mock
    pub fn add_directory(&self, path: &str, contents: Vec<String>) {
        self.directories
            .lock()
            .unwrap()
            .insert(path.to_string(), contents);
    }

    /// Get operation history
    pub fn operation_history(&self) -> Vec<FileOperation> {
        self.history.lock().unwrap().clone()
    }
}

impl Default for FileSystemMock {
    fn default() -> Self {
        Self::new()
    }
}

/// Predefined mock scenarios
pub struct MockPresets;

impl MockPresets {
    /// Create a successful API mock
    pub fn successful_api() -> HttpMock {
        let mut mock = HttpMock::new();
        mock.mock_endpoint("GET", "/api/status", MockHttpResponse {
            status:  200,
            headers: HashMap::new(),
            body:    Some(r#"{"status": "ok"}"#.to_string()),
        });
        mock
    }

    /// Create a file system with common Rust project structure
    pub fn rust_project_fs() -> FileSystemMock {
        let fs = FileSystemMock::new();
        fs.add_file(
            "Cargo.toml",
            r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021""#,
        );
        fs.add_file("src/lib.rs", "pub fn hello() -> &'static str { \"Hello\" }");
        fs.add_directory("src", vec!["lib.rs".to_string()]);
        fs
    }

    /// Create a database mock with common operations
    pub fn database_mock() -> GenericMock<Result<serde_json::Value, TestError>> {
        let mock = GenericMock::<Result<serde_json::Value, TestError>>::new();
        mock.when(
            "query",
            MockBehaviors::success(|| Ok(serde_json::json!({"results": []}))),
        );
        mock.when(
            "insert",
            MockBehaviors::success(|| Ok(serde_json::json!({"inserted": 1}))),
        );
        mock.when(
            "update",
            MockBehaviors::success(|| Ok(serde_json::json!({"updated": 1}))),
        );
        mock.when(
            "delete",
            MockBehaviors::success(|| Ok(serde_json::json!({"deleted": 1}))),
        );
        mock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_mock() {
        let mock = GenericMock::<String>::new();
        mock.when(
            "greet",
            MockBehaviors::success(|| "Hello World".to_string()),
        );

        let result = mock.execute("greet", &[]);
        assert_eq!(result, "Hello World");
        assert!(mock.verify_called("greet"));
    }

    #[test]
    fn test_http_mock() {
        let mut mock = HttpMock::new();
        mock.mock_endpoint("GET", "/test", MockHttpResponse {
            status:  200,
            headers: HashMap::new(),
            body:    Some("success".to_string()),
        });

        let response = mock.request("GET", "/test", HashMap::new(), None).unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body.unwrap(), "success");
    }

    #[test]
    fn test_filesystem_mock() {
        let fs = FileSystemMock::new();
        fs.write_file("test.txt", "content").unwrap();

        let content = fs.read_file("test.txt").unwrap();
        assert_eq!(content, "content");

        let history = fs.operation_history();
        assert_eq!(history.len(), 2); // write + read
    }

    #[test]
    fn test_mock_behaviors_sequence() {
        let mock = GenericMock::<i32>::new();
        mock.when("counter", MockBehaviors::sequence(vec![1, 2, 3]));

        assert_eq!(mock.execute("counter", &[]), 1);
        assert_eq!(mock.execute("counter", &[]), 2);
        assert_eq!(mock.execute("counter", &[]), 3);
        assert_eq!(mock.execute("counter", &[]), 1); // cycles back
    }
}

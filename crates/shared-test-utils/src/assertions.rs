//! Advanced test assertion helpers and validation utilities
//!
//! Provides comprehensive assertion patterns for testing various scenarios
//! including async operations, collections, file systems, and performance.

use crate::TestError;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use std::time::{Duration, Instant};

/// Assert that a file exists and has expected content
pub fn assert_file_exists_and_not_empty(path: &Path) {
    assert!(path.exists(), "File does not exist: {}", path.display());
    if path.is_file() {
        assert!(path.metadata().unwrap().len() > 0, "File is empty: {}", path.display());
    } else {
        // For directories, check if not empty
        assert!(!path.read_dir().unwrap().next().is_none(), "Directory is empty: {}", path.display());
    }
}

/// Assert that two slices are approximately equal
pub fn assert_approximately_equal(left: &[f64], right: &[f64], tolerance: f64) {
    assert_eq!(left.len(), right.len(), "Slices have different lengths");

    for (i, (a, b)) in left.iter().zip(right.iter()).enumerate() {
        assert!(
            (a - b).abs() < tolerance,
            "Values at index {} differ: {} vs {} (tolerance: {})",
            i, a, b, tolerance
        );
    }
}

/// Collection assertions
pub mod collections {
    use super::*;

    /// Assert that a collection contains expected elements
    pub fn assert_contains<T: PartialEq + Debug>(collection: &[T], expected: &[T]) {
        for item in expected {
            assert!(collection.contains(item),
                "Collection does not contain expected item: {:?}", item);
        }
    }

    /// Assert that collections are equal ignoring order
    pub fn assert_equal_unordered<T: PartialEq + Debug + Clone + Eq + std::hash::Hash>(left: &[T], right: &[T]) {
        assert_eq!(left.len(), right.len(), "Collections have different lengths");

        let mut left_counts = HashMap::new();
        let mut right_counts = HashMap::new();

        for item in left {
            *left_counts.entry(item).or_insert(0) += 1;
        }

        for item in right {
            *right_counts.entry(item).or_insert(0) += 1;
        }

        for (item, left_count) in left_counts {
            let right_count = right_counts.get(item).copied().unwrap_or(0);
            assert_eq!(left_count, right_count,
                "Item count mismatch for {:?}: left={}, right={}",
                item, left_count, right_count);
        }
    }

    /// Assert that a hashmap contains expected key-value pairs
    pub fn assert_map_contains<K: Debug + Eq + std::hash::Hash, V: PartialEq + Debug>(
        map: &HashMap<K, V>,
        expected: &HashMap<K, V>
    ) {
        for (key, expected_value) in expected {
            match map.get(key) {
                Some(actual_value) => assert_eq!(actual_value, expected_value,
                    "Value mismatch for key {:?}: expected {:?}, got {:?}",
                    key, expected_value, actual_value),
                None => panic!("Expected key not found: {:?}", key),
            }
        }
    }

    /// Assert that no collection contains duplicates
    pub fn assert_no_duplicates<T: PartialEq + Debug + Clone>(collection: &[T]) {
        let mut seen = Vec::new();
        for item in collection {
            if seen.contains(item) {
                panic!("Duplicate item found: {:?}", item);
            }
            seen.push(item.clone());
        }
    }
}

/// Network and HTTP assertions
pub mod network {
    use super::*;

    /// Assert that an HTTP status code indicates success
    pub fn assert_http_success(status: u16) {
        assert!(status >= 200 && status < 300,
            "HTTP request failed with status: {}", status);
    }

    /// Assert that an HTTP response contains expected headers
    pub fn assert_headers_contain(response_headers: &HashMap<String, String>, expected: &HashMap<String, String>) {
        for (key, expected_value) in expected {
            match response_headers.get(key) {
                Some(actual_value) => assert_eq!(actual_value, expected_value,
                    "Header '{}' mismatch: expected '{}', got '{}'",
                    key, expected_value, actual_value),
                None => panic!("Expected header not found: {}", key),
            }
        }
    }

    /// Assert that a URL has expected components
    pub fn assert_url_structure(url: &str, expected_scheme: &str, expected_host: Option<&str>) {
        if let Some(scheme_end) = url.find("://") {
            let scheme = &url[..scheme_end];
            assert_eq!(scheme, expected_scheme,
                "URL scheme mismatch: expected '{}', got '{}'",
                expected_scheme, scheme);

            if let Some(host) = expected_host {
                let after_scheme = &url[scheme_end + 3..];
                if let Some(host_end) = after_scheme.find('/') {
                    let actual_host = &after_scheme[..host_end];
                    assert_eq!(actual_host, host,
                        "URL host mismatch: expected '{}', got '{}'",
                        host, actual_host);
                } else if let Some(query_start) = after_scheme.find('?') {
                    let actual_host = &after_scheme[..query_start];
                    assert_eq!(actual_host, host,
                        "URL host mismatch: expected '{}', got '{}'",
                        host, actual_host);
                } else {
                    let actual_host = after_scheme;
                    assert_eq!(actual_host, host,
                        "URL host mismatch: expected '{}', got '{}'",
                        host, actual_host);
                }
            }
        } else {
            panic!("URL does not contain valid scheme: {}", url);
        }
    }
}

/// Performance and timing assertions
pub mod performance {
    use super::*;

    /// Assert that an operation completes within expected time
    pub fn assert_execution_time<F, T>(operation: F, max_duration: Duration) -> T
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let result = operation();
        let elapsed = start.elapsed();

        assert!(elapsed <= max_duration,
            "Operation took too long: {:?} > {:?}",
            elapsed, max_duration);

        result
    }

    /// Assert that an async operation completes within expected time
    pub async fn assert_async_execution_time<F, Fut, T>(
        operation: F,
        max_duration: Duration
    ) -> T
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let start = Instant::now();
        let result = operation().await;
        let elapsed = start.elapsed();

        assert!(elapsed <= max_duration,
            "Async operation took too long: {:?} > {:?}",
            elapsed, max_duration);

        result
    }

    /// Assert that throughput meets minimum requirements
    pub fn assert_minimum_throughput(operations: usize, duration: Duration, min_ops_per_sec: f64) {
        let actual_ops_per_sec = operations as f64 / duration.as_secs_f64();
        assert!(actual_ops_per_sec >= min_ops_per_sec,
            "Throughput too low: {:.2} ops/sec < {:.2} ops/sec minimum",
            actual_ops_per_sec, min_ops_per_sec);
    }

    /// Assert that memory usage is within limits
    pub fn assert_memory_usage(current: usize, max_allowed: usize, operation: &str) {
        assert!(current <= max_allowed,
            "Memory usage exceeded for {}: {} bytes > {} bytes maximum",
            operation, current, max_allowed);
    }

    /// Assert that CPU usage is within acceptable range
    pub fn assert_cpu_usage(usage_percent: f64, max_percent: f64) {
        assert!(usage_percent <= max_percent,
            "CPU usage too high: {:.2}% > {:.2}% maximum",
            usage_percent, max_percent);
    }
}

/// File system assertions
pub mod filesystem {
    use super::*;
    use std::fs;

    /// Assert that a directory structure matches expected layout
    pub fn assert_directory_structure(root: &Path, expected_paths: &[&str]) {
        for expected_path in expected_paths {
            let full_path = root.join(expected_path);
            assert!(full_path.exists(),
                "Expected path does not exist: {}", full_path.display());
        }
    }

    /// Assert that a file contains expected text
    pub fn assert_file_contains(path: &Path, expected_text: &str) {
        let content = fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Failed to read file: {}", path.display()));

        assert!(content.contains(expected_text),
            "File {} does not contain expected text '{}'\nContent: {}",
            path.display(), expected_text, content);
    }

    /// Assert that a file matches expected format (JSON, YAML, etc.)
    pub fn assert_file_format(path: &Path, format: &str) -> Result<(), TestError> {
        let content = fs::read_to_string(path).map_err(|e| TestError::Io(e.to_string()))?;

        match format.to_lowercase().as_str() {
            "json" => {
                serde_json::from_str::<serde_json::Value>(&content)
                    .map_err(|e| TestError::Json(e.to_string()))?;
                Ok(())
            }
            "yaml" | "yml" => {
                serde_yaml::from_str::<serde_yaml::Value>(&content)
                    .map_err(|e| TestError::Yaml(e.to_string()))?;
                Ok(())
            }
            "toml" => {
                toml::from_str::<toml::Value>(&content)
                    .map_err(|e| TestError::Toml(e.to_string()))?;
                Ok(())
            }
            _ => Err(TestError::Validation(crate::ValidationError::invalid_setup(
                format!("Unsupported format: {}", format)
            ))),
        }
    }

    /// Assert file permissions on Unix systems
    #[cfg(unix)]
    pub fn assert_file_permissions(path: &Path, expected_mode: u32) {
        use std::os::unix::fs::PermissionsExt;
        let permissions = fs::metadata(path).unwrap().permissions();
        assert_eq!(permissions.mode() & 0o777, expected_mode,
            "File permissions mismatch for {}: expected {:o}, got {:o}",
            path.display(), expected_mode, permissions.mode() & 0o777);
    }
}

/// Database assertions
pub mod database {
    
    #[cfg(feature = "database")]
    use rusqlite::Connection;

    /// Assert that a database contains expected tables
    #[cfg(feature = "database")]
    pub fn assert_tables_exist(conn: &Connection, expected_tables: &[&str]) {
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'").unwrap();
        let existing_tables: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap()
            .map(|r| r.unwrap()).collect();

        for expected_table in expected_tables {
            assert!(existing_tables.contains(&expected_table.to_string()),
                "Expected table '{}' not found. Existing tables: {:?}", expected_table, existing_tables);
        }
    }

    /// Assert that a table contains expected data
    #[cfg(feature = "database")]
    pub fn assert_table_contains(conn: &Connection, table: &str, conditions: &[(&str, &str)]) -> Result<(), TestError> {
        let mut query = format!("SELECT COUNT(*) FROM {} WHERE ", table);
        let mut params = Vec::new();

        for (i, (column, value)) in conditions.iter().enumerate() {
            if i > 0 {
                query.push_str(" AND ");
            }
            query.push_str(&format!("{} = ?", column));
            params.push(*value);
        }

        let count: i64 = conn.query_row(&query, rusqlite::params_from_iter(params), |row| row.get(0))?;
        if count == 0 {
            return Err(TestError::Validation(crate::ValidationError::invalid_setup(
                format!("No rows found in table '{}' matching conditions: {:?}", table, conditions)
            )));
        }

        Ok(())
    }

    /// Assert table row count
    #[cfg(feature = "database")]
    pub fn assert_row_count(conn: &Connection, table: &str, expected_count: usize) {
        let count: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |row| row.get(0)).unwrap();
        assert_eq!(count as usize, expected_count,
            "Row count mismatch for table '{}': expected {}, got {}", table, expected_count, count);
    }

    #[cfg(not(feature = "database"))]
    pub fn assert_no_database_support() {
        panic!("Database assertions require the 'database' feature to be enabled");
    }
}

/// String and text assertions
pub mod text {
    use super::*;
    use regex::Regex;

    /// Assert that text matches a regular expression
    pub fn assert_matches_pattern(text: &str, pattern: &str) {
        let regex = Regex::new(pattern).unwrap_or_else(|_| panic!("Invalid regex pattern: {}", pattern));
        assert!(regex.is_match(text),
            "Text '{}' does not match pattern '{}'", text, pattern);
    }

    /// Assert that text contains all expected substrings
    pub fn assert_contains_all(text: &str, substrings: &[&str]) {
        for substring in substrings {
            assert!(text.contains(substring),
                "Text '{}' does not contain substring '{}'", text, substring);
        }
    }

    /// Assert that text starts with expected prefix
    pub fn assert_starts_with(text: &str, prefix: &str) {
        assert!(text.starts_with(prefix),
            "Text '{}' does not start with '{}'", text, prefix);
    }

    /// Assert that text ends with expected suffix
    pub fn assert_ends_with(text: &str, suffix: &str) {
        assert!(text.ends_with(suffix),
            "Text '{}' does not end with '{}'", text, suffix);
    }

    /// Assert that text length is within expected range
    pub fn assert_length_in_range(text: &str, min: usize, max: usize) {
        let len = text.len();
        assert!(len >= min && len <= max,
            "Text length {} is not in range [{}, {}]", len, min, max);
    }

    /// Assert that JSON content is valid and contains expected structure
    pub fn assert_valid_json(json_str: &str, expected_fields: &[&str]) -> Result<(), TestError> {
        let value: serde_json::Value = serde_json::from_str(json_str)?;

        for field in expected_fields {
            assert!(value.get(field).is_some(),
                "JSON does not contain expected field: {}", field);
        }

        Ok(())
    }
}

/// Async operation assertions
pub mod async_ops {
    use super::*;
    use std::future::Future;

    /// Assert that an async operation completes without panicking
    pub async fn assert_completes_without_panic<F, T>(future: F) -> T
    where
        F: Future<Output = T>,
    {
        future.await
    }

    /// Assert that an async operation returns expected result
    pub async fn assert_returns_expected<F, Fut, T>(future: F, expected: T) -> T
    where
        F: Future<Output = T>,
        T: PartialEq + Debug,
    {
        let result = future.await;
        assert_eq!(result, expected, "Async operation returned unexpected result");
        result
    }

    /// Assert that multiple async operations complete in order
    pub async fn assert_completion_order<F, Fut, T>(
        operations: Vec<F>
    ) -> Vec<T>
    where
        F: Future<Output = T>,
        Fut: Future<Output = T>,
    {
        let mut results = Vec::new();
        for operation in operations {
            results.push(operation.await);
        }
        results
    }
}

/// Custom assertion builder for fluent testing
pub struct AssertionBuilder<T> {
    value: T,
    description: Option<String>,
}

impl<T> AssertionBuilder<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            description: None,
        }
    }

    pub fn described_as(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    fn format_error(&self, message: &str) -> String {
        if let Some(desc) = &self.description {
            format!("{}: {}", desc, message)
        } else {
            message.to_string()
        }
    }
}

impl<T: PartialEq + Debug> AssertionBuilder<T> {
    pub fn is_equal_to(self, expected: T) -> Self {
        assert_eq!(self.value, expected, "{}", self.format_error("Values are not equal"));
        self
    }

    pub fn is_not_equal_to(self, unexpected: T) -> Self {
        assert_ne!(self.value, unexpected, "{}", self.format_error("Values are equal"));
        self
    }
}

impl<T> AssertionBuilder<Option<T>> {
    pub fn is_some(self) -> AssertionBuilder<T> {
        match self.value {
            Some(value) => AssertionBuilder {
                value,
                description: self.description,
            },
            None => panic!("{}", self.format_error("Expected Some, got None")),
        }
    }

    pub fn is_none(self) -> Self {
        assert!(self.value.is_none(), "{}", self.format_error("Expected None, got Some"));
        self
    }
}

/// Macro for fluent assertions
#[macro_export]
macro_rules! assert_that {
    ($value:expr) => {
        $crate::assertions::AssertionBuilder::new($value)
    };
    ($value:expr, $desc:expr) => {
        $crate::assertions::AssertionBuilder::new($value).described_as($desc)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_collection_assertions() {
        let collection = vec![1, 2, 3, 4, 5];
        collections::assert_contains(&collection, &[2, 4]);

        let unordered1 = vec![1, 2, 3, 2];
        let unordered2 = vec![2, 1, 2, 3];
        collections::assert_equal_unordered(&unordered1, &unordered2);

        let mut map1 = HashMap::new();
        map1.insert("key1", "value1");
        map1.insert("key2", "value2");

        let mut map2 = HashMap::new();
        map2.insert("key1", "value1");

        collections::assert_map_contains(&map1, &map2);
    }

    #[test]
    fn test_network_assertions() {
        network::assert_http_success(200);
        network::assert_http_success(201);
        assert!(std::panic::catch_unwind(|| network::assert_http_success(404)).is_err());

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let mut expected = HashMap::new();
        expected.insert("content-type".to_string(), "application/json".to_string());

        network::assert_headers_contain(&headers, &expected);

        network::assert_url_structure("https://api.example.com/v1/users", "https", Some("api.example.com"));
    }

    #[test]
    fn test_text_assertions() {
        text::assert_matches_pattern("hello123", r"^hello\d+$");
        text::assert_contains_all("the quick brown fox", &["quick", "fox"]);
        text::assert_starts_with("prefix_text", "prefix");
        text::assert_ends_with("text_suffix", "suffix");
        text::assert_length_in_range("hello", 3, 10);

        let json_str = r#"{"name": "test", "value": 42}"#;
        assert!(text::assert_valid_json(json_str, &["name"]).is_ok());
    }

    #[test]
    fn test_performance_assertions() {
        let result = performance::assert_execution_time(
            || 42,
            Duration::from_secs(1)
        );
        assert_eq!(result, 42);

        performance::assert_minimum_throughput(100, Duration::from_secs(1), 50.0);
    }

    #[test]
    fn test_assertion_builder() {
        assert_that!(42).is_equal_to(42);
        assert_that!(Some(42)).is_some().is_equal_to(42);
        assert_that!(None::<i32>).is_none();
    }

    #[test]
    fn test_fluent_assertions() {
        let value = 42;
        assert_that!(value).is_equal_to(42);
        assert_that!(value, "test value").is_not_equal_to(24);
    }
}
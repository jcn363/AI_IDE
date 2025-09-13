//! Test helpers and utilities for the test suite

use std::path::Path;

/// Helper function to get the path to a test fixture file
pub fn fixture_path(relative_path: &str) -> String {
    let mut path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test-fixtures");

    if !relative_path.is_empty() {
        path = path.join(relative_path);
    }

    path.to_string_lossy().into_owned()
}

/// Helper function to create a temporary directory for testing
pub fn temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}

/// Helper macro to assert that a result is an error
#[macro_export]
macro_rules! assert_error {
    ($result:expr, $pat:pat) => {
        match $result {
            Err($pat) => {}
            Ok(_) => panic!("Expected error, got Ok"),
            Err(e) => panic!("Expected error matching pattern, got {:?}", e),
        }
    };
}

/// Helper macro to assert that a result is ok
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        assert!($result.is_ok(), "Expected Ok, got {:?}", $result);
    };
}

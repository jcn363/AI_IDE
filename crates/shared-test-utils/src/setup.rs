//! Test setup and teardown utilities

use std::env;
use std::path::{Path, PathBuf};

/// Get the project root directory from CARGO_MANIFEST_DIR
pub fn get_project_root() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()))
}

/// Get a test directory relative to the project root
pub fn get_test_directory(name: &str) -> PathBuf {
    get_project_root().join("target").join("tests").join(name)
}

/// Create a test directory with cleanup on drop
pub struct TestDir {
    path: PathBuf,
}

impl TestDir {
    pub fn new(name: &str) -> std::io::Result<Self> {
        let path = get_test_directory(name);
        std::fs::create_dir_all(&path)?;
        Ok(TestDir { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_setup() {
        let root = get_project_root();
        assert!(root.exists());

        let test_path = get_test_directory("test-utils");
        assert!(test_path.to_string_lossy().contains("tests"));
    }
}
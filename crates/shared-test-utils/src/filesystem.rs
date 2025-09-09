use std::path::{Path, PathBuf};
use std::fs;
use tempfile::{TempDir, NamedTempFile};
use crate::error::TestError;

/// A temporary workspace for tests that automatically cleans up after test completion
// Prepared for shared test workspace management across crates
#[derive(Debug)]
pub struct TempWorkspace {
    /// The underlying temporary directory
    _temp_dir: TempDir,
    /// Path to the temporary workspace directory
    workspace_path: PathBuf,
}

impl TempWorkspace {
    /// Creates a new temporary workspace with a unique directory
    pub fn new() -> Result<Self, TestError> {
        let _temp_dir = TempDir::new().map_err(|e| TestError::Io(e.to_string()))?;
        let workspace_path = _temp_dir.path().to_path_buf();

        Ok(TempWorkspace {
            _temp_dir,
            workspace_path,
        })
    }

    /// Creates a workspace in a specific base directory
    pub fn with_base(base: &Path) -> Result<Self, TestError> {
        let _temp_dir = TempDir::new_in(base).map_err(|e| TestError::Io(e.to_string()))?;
        let workspace_path = _temp_dir.path().to_path_buf();

        Ok(TempWorkspace {
            _temp_dir,
            workspace_path,
        })
    }

    /// Returns the root path of the workspace
    pub fn path(&self) -> &Path {
        &self.workspace_path
    }

    /// Creates a directory inside the workspace
    pub fn create_dir(&self, path: &Path) -> Result<(), TestError> {
        let full_path = self.workspace_path.join(path);
        fs::create_dir_all(full_path).map_err(|e| TestError::Io(e.to_string()))
    }

    /// Creates a file with given content inside the workspace
    pub fn create_file(&self, path: &Path, content: &str) -> Result<(), TestError> {
        let full_path = self.workspace_path.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).map_err(|e| TestError::Io(e.to_string()))?;
        }
        fs::write(full_path, content).map_err(|e| TestError::Io(e.to_string()))
    }

    /// Reads a file from the workspace
    pub fn read_file(&self, path: &Path) -> Result<String, TestError> {
        let full_path = self.workspace_path.join(path);
        fs::read_to_string(full_path).map_err(|e| TestError::Io(e.to_string()))
    }

    /// Checks if a file exists in the workspace
    pub fn file_exists(&self, path: &Path) -> bool {
        self.workspace_path.join(path).exists()
    }

    /// Creates a temporary file within the workspace
    pub fn create_temp_file(&self, _prefix: &str) -> Result<NamedTempFile, TestError> {
        NamedTempFile::new_in(self.path())
            .map_err(|e| TestError::Io(e.to_string()))
    }

    /// Returns all files and directories in the workspace
    pub fn contents(&self) -> Result<Vec<PathBuf>, TestError> {
        let mut entries = Vec::new();
        self.collect_entries(&mut entries, &self.workspace_path)?;
        Ok(entries)
    }

    fn collect_entries(&self, entries: &mut Vec<PathBuf>, dir: &Path) -> Result<(), TestError> {
        for entry in fs::read_dir(dir).map_err(|e| TestError::Io(e.to_string()))? {
            let entry = entry.map_err(|e| TestError::Io(e.to_string()))?;
            entries.push(entry.path());

            if entry.file_type().map_err(|e| TestError::Io(e.to_string()))?.is_dir() {
                self.collect_entries(entries, &entry.path())?;
            }
        }
        Ok(())
    }

    /// Runs an async function with the workspace
    pub async fn with_async<F, Fut, T>(&self, f: F) -> Result<T, TestError>
    where
        F: FnOnce(&TempWorkspace) -> Fut,
        Fut: std::future::Future<Output = Result<T, TestError>>,
    {
        f(self).await
    }
}

impl TempWorkspace {
    /// Utility function to set up a basic project structure for tests
    pub fn setup_basic_project(&self) -> Result<(), TestError> {
        self.create_dir(Path::new("src"))?;
        self.create_dir(Path::new("tests"))?;
        self.create_file(Path::new("Cargo.toml"),
            r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = "1.0"
"#)?;
        self.create_file(Path::new("src/lib.rs"),
            r#"pub fn hello() -> &'static str {
    "Hello from test project!"
}"#)?;
        Ok(())
    }

    /// Prepares a workspace for a specific test scenario
    pub fn setup_scenario(&self, scenario: TestScenario) -> Result<(), TestError> {
        match scenario {
            TestScenario::BasicRust => self.setup_basic_project(),
            TestScenario::WithTests => {
                self.setup_basic_project()?;
                self.create_dir(Path::new("tests/integration"))?;
                self.create_file(Path::new("tests/lib_test.rs"),
                    r#"#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello(), "Hello from test project!");
    }
}"#)
            }
            TestScenario::Empty => Ok(()),
        }
    }
}

/// Test scenarios for workspace setup
#[derive(Debug, Clone)]
pub enum TestScenario {
    /// Creates a basic Rust project structure with src/ and tests/ directories
    BasicRust,
    /// Creates a project with tests included in addition to basic structure
    WithTests,
    /// Creates an empty workspace with no initial files or directories
    Empty,
}

/// Extension methods for working with temporary files
pub trait TempFileExt {
    fn with_content(&self, content: &str) -> Result<(), TestError>;
    fn read_content(&self) -> Result<String, TestError>;
}

impl TempFileExt for NamedTempFile {
    fn with_content(&self, content: &str) -> Result<(), TestError> {
        std::fs::write(self.path(), content).map_err(|e| TestError::Io(e.to_string()))
    }

    fn read_content(&self) -> Result<String, TestError> {
        std::fs::read_to_string(self.path()).map_err(|e| TestError::Io(e.to_string()))
    }
}
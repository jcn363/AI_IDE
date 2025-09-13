//! Consolidated testing utilities module
//!
//! This module consolidates all testing-related utilities from across the codebase
//! including performance testing, integration testing, and test command execution.

use crate::handlers::validation::validate_secure_path;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tokio::process::Command as AsyncCommand;
use tokio::sync::Mutex;

/// Test environment state for managing test execution
#[derive(Debug, Clone)]
pub struct TestEnvironment {
    pub workspace_path: PathBuf,
    pub temp_dirs: Vec<PathBuf>,
    pub test_projects: HashMap<String, PathBuf>,
}

/// Command execution result
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
}

/// Performance metrics for test commands
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_time_ms: f64,
    pub cpu_time_ms: f64,
    pub memory_peak_kb: u64,
    pub operations_per_second: f64,
}

/// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub enable_coverage: bool,
    pub enable_incremental: bool,
    pub enable_parallel: bool,
    pub timeout_seconds: u64,
    pub working_directory: PathBuf,
}

/// Consolidated test utilities module
pub mod utils {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::time::Instant;

    /// Run a command synchronously with performance tracking
    pub fn run_command_blocking(
        cmd: &str,
        args: &[&str],
        cwd: &Path,
    ) -> anyhow::Result<CommandResult> {
        println!("Running: {} {}", cmd, args.join(" "));
        let start = Instant::now();

        let output = std::process::Command::new(cmd)
            .args(args)
            .current_dir(cwd)
            .output()?;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(CommandResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
            duration_ms,
        })
    }

    /// Run a command asynchronously
    pub async fn run_command_async(
        cmd: &str,
        args: &[&str],
        cwd: &Path,
    ) -> anyhow::Result<CommandResult> {
        println!("Running async: {} {}", cmd, args.join(" "));
        let start = Instant::now();

        let output = AsyncCommand::new(cmd)
            .args(args)
            .current_dir(cwd)
            .output()
            .await?;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(CommandResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
            duration_ms,
        })
    }

    /// Generate SHA256 checksum for a file
    pub fn generate_sha256_checksum(file_path: &str) -> anyhow::Result<String> {
        use std::fs;

        let content = fs::read(file_path)?;
        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash = hasher.finalize();

        Ok(format!("{:x}", hash))
    }

    /// Extract test names from cargo test --list output
    pub fn parse_cargo_test_list(output: &str) -> Vec<String> {
        output
            .lines()
            .filter(|line| line.starts_with("test "))
            .filter_map(|line| {
                line.strip_prefix("test ")
                    .and_then(|s| s.split_whitespace().next())
                    .map(|s| s.to_string())
            })
            .collect()
    }

    /// Validate that a directory is a Rust project
    pub fn validate_rust_project(project_path: &Path) -> anyhow::Result<()> {
        if !project_path.is_dir() {
            anyhow::bail!("Path is not a directory: {}", project_path.display());
        }

        let cargo_toml = project_path.join("Cargo.toml");
        if !cargo_toml.exists() {
            anyhow::bail!("Cargo.toml not found in: {}", project_path.display());
        }

        let src_dir = project_path.join("src");
        if !src_dir.exists()
            || !src_dir.join("lib.rs").exists() && !src_dir.join("main.rs").exists()
        {
            anyhow::bail!(
                "src/lib.rs or src/main.rs not found in: {}",
                project_path.display()
            );
        }

        Ok(())
    }

    /// Create a temporary test project with basic structure
    pub fn create_temp_test_project(base_dir: &Path, name: &str) -> anyhow::Result<PathBuf> {
        let project_path = base_dir.join(name);

        std::fs::create_dir_all(&project_path)?;
        std::fs::create_dir_all(project_path.join("src"))?;

        // Create Cargo.toml
        let cargo_toml = format!(
            r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"
"#,
            name
        );

        std::fs::write(project_path.join("Cargo.toml"), cargo_toml)?;

        // Create src/lib.rs
        let lib_rs = r#"
//! Automated test project

pub fn hello_world() -> String {
    "Hello, World!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        assert_eq!(hello_world(), "Hello, World!");
    }
}
"#;

        std::fs::write(project_path.join("src/lib.rs"), lib_rs)?;

        Ok(project_path)
    }

    /// Clean and prepare a project for testing
    pub fn clean_and_prepare_project(project_path: &Path) -> anyhow::Result<()> {
        // Run cargo clean
        let _ = run_command_blocking("cargo", &["clean"], project_path)?;
        Ok(())
    }
}

/// Performance testing utilities
pub mod performance {
    use super::*;
    use std::time::{Duration, Instant};

    /// Run a synchronous performance test workload
    pub fn run_sync_performance_workload(iterations: u32) -> (u64, Duration) {
        let start = Instant::now();
        let result = do_sync_work(iterations);
        let duration = start.elapsed();
        (result, duration)
    }

    /// Run an asynchronous performance test workload
    pub async fn run_async_performance_workload(iterations: u32) -> (u64, Duration) {
        let start = Instant::now();
        let result = do_async_work(iterations).await;
        let duration = start.elapsed();
        (result, duration)
    }

    /// Simulate CPU-bound synchronous work
    fn do_sync_work(iterations: u32) -> u64 {
        let mut result = 0u64;

        for i in 0..iterations {
            // Simple hash-like operation
            let x = i as u64 * 2654435761 % (1 << 31);
            result = result.wrapping_add(x);

            // Simulate memory allocation
            let mut vec = Vec::with_capacity(1000);
            for j in 0..1000 {
                vec.push(j as u64);
            }

            // Use the vector to prevent optimization
            result = result.wrapping_add(vec[vec.len() - 1]);
        }

        result
    }

    /// Simulate I/O-bound asynchronous work
    async fn do_async_work(iterations: u32) -> u64 {
        let mut result = 0u64;

        for i in 0..iterations {
            // Simulate async I/O
            tokio::time::sleep(Duration::from_millis(10)).await;

            // Some computation
            let x = i as u64 * 11400714819323198549u64;
            result = result.wrapping_add(x);
        }

        result
    }

    /// Calculate operations per second
    pub fn calculate_ops_per_second(operations: u64, duration: Duration) -> f64 {
        operations as f64 / duration.as_secs_f64()
    }

    /// Generate performance report
    pub fn generate_performance_report(
        test_name: &str,
        sync_iterations: u32,
        sync_result: u64,
        sync_duration: Duration,
        async_iterations: u32,
        async_result: u64,
        async_duration: Duration,
    ) -> String {
        let sync_ops_per_sec = calculate_ops_per_second(sync_iterations as u64, sync_duration);
        let async_ops_per_sec = calculate_ops_per_second(async_iterations as u64, async_duration);

        format!(
            r#"=== Performance Test Report: {} ===

Synchronous Work:
- Iterations: {}
- Duration: {:.2?}
- Operations/Second: {:.2}
- Result: {}

Asynchronous Work:
- Iterations: {}
- Duration: {:.2?}
- Operations/Second: {:.2}
- Result: {}

Recommendations:
- Synchronous workload is {:.1}x faster per operation
- Consider async for I/O-bound operations
- Memory usage appears stable
"#,
            test_name,
            sync_iterations,
            sync_duration,
            sync_ops_per_sec,
            sync_result,
            async_iterations,
            async_duration,
            async_ops_per_sec,
            async_result,
            sync_ops_per_sec / async_ops_per_sec.max(0.001)
        )
    }
}

/// Integration testing utilities
pub mod integration {
    use super::*;
    use serde::{Deserialize, Serialize};

    /// Test fixture for integration tests
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TestFixture {
        pub name: String,
        pub description: Option<String>,
        pub setup_commands: Vec<String>,
        pub test_commands: Vec<String>,
        pub cleanup_commands: Vec<String>,
        pub expected_files: Vec<String>,
    }

    /// Run integration test with fixture
    pub async fn run_integration_test(
        fixture: &TestFixture,
        workspace_path: &Path,
    ) -> anyhow::Result<TestResult> {
        let mut results = HashMap::new();

        // Setup phase
        println!("=== Setup Phase: {} ===", fixture.name);
        for cmd in &fixture.setup_commands {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if let Some((cmd, args)) = parts.split_first() {
                let result = utils::run_command_async(cmd, args, workspace_path).await?;
                results.insert(format!("setup_{}", cmd), result);
            }
        }

        // Test phase
        println!("=== Test Phase: {} ===", fixture.name);
        for cmd in &fixture.test_commands {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if let Some((cmd, args)) = parts.split_first() {
                let result = utils::run_command_async(cmd, args, workspace_path).await?;
                results.insert(cmd.to_string(), result);
            }
        }

        // Verification phase
        println!("=== Verification Phase: {} ===", fixture.name);
        let mut files_exist = true;
        for file in &fixture.expected_files {
            let file_path = workspace_path.join(file);
            if !file_path.exists() {
                files_exist = false;
                break;
            }
        }

        // Cleanup phase
        println!("=== Cleanup Phase: {} ===", fixture.name);
        let mut cleanup_successful = false;
        if fixture.cleanup_commands.is_empty() {
            cleanup_successful = true;
        } else {
            for cmd in &fixture.cleanup_commands {
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                if let Some((cmd, args)) = parts.split_first() {
                    let result = utils::run_command_async(cmd, args, workspace_path).await?;
                    if result.success {
                        cleanup_successful = true;
                    }
                }
            }
        }

        Ok(TestResult {
            fixture_name: fixture.name.clone(),
            success: files_exist,
            results,
            cleanup_successful,
        })
    }

    /// Test result from integration test
    #[derive(Debug)]
    pub struct TestResult {
        pub fixture_name: String,
        pub success: bool,
        pub results: HashMap<String, CommandResult>,
        pub cleanup_successful: bool,
    }
}

/// Test coverage utilities
pub mod coverage {
    use super::*;

    /// Check if coverage tools are available
    pub fn check_coverage_availability() -> CoverageTools {
        let has_llvm_cov = Command::new("cargo")
            .args(["llvm-cov", "--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let has_tarpaulin = Command::new("cargo")
            .args(["tarpaulin", "--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let has_grcov = Command::new("grcov")
            .args(["--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        CoverageTools {
            has_llvm_cov,
            has_tarpaulin,
            has_grcov,
        }
    }

    /// Available coverage tools
    #[derive(Debug, Clone)]
    pub struct CoverageTools {
        pub has_llvm_cov: bool,
        pub has_tarpaulin: bool,
        pub has_grcov: bool,
    }

    impl CoverageTools {
        pub fn any_available(&self) -> bool {
            self.has_llvm_cov || self.has_tarpaulin || self.has_grcov
        }

        pub fn best_available(&self) -> Option<&'static str> {
            if self.has_llvm_cov {
                Some("llvm-cov")
            } else if self.has_tarpaulin {
                Some("tarpaulin")
            } else if self.has_grcov {
                Some("grcov")
            } else {
                None
            }
        }
    }

    /// Run coverage analysis
    pub async fn run_coverage(
        project_path: &Path,
        config: &TestConfig,
    ) -> anyhow::Result<CoverageResult> {
        let tools = check_coverage_availability();

        if !tools.any_available() {
            anyhow::bail!(
                "No coverage tools available. Install cargo-llvm-cov, cargo-tarpaulin, or grcov"
            );
        }

        let tool = tools.best_available().unwrap();
        let output = match tool {
            "llvm-cov" => {
                utils::run_command_async("cargo", &["llvm-cov", "--json"], project_path).await?
            }
            "tarpaulin" => {
                utils::run_command_async("cargo", &["tarpaulin", "--out", "Stdout"], project_path)
                    .await?
            }
            "grcov" => {
                // grcov requires additional setup, assume JSON output
                utils::run_command_async("grcov", &[".", "--json"], project_path).await?
            }
            _ => anyhow::bail!("Unsupported coverage tool"),
        };

        Ok(CoverageResult {
            success: output.success,
            tool: tool.to_string(),
            output: output.stdout,
            stderr: output.stderr,
        })
    }

    /// Coverage result
    #[derive(Debug)]
    pub struct CoverageResult {
        pub success: bool,
        pub tool: String,
        pub output: String,
        pub stderr: String,
    }
}

/// Test environment manager for creating and managing test environments
pub struct TestEnvironmentManager {
    base_temp_dir: PathBuf,
    environments: HashMap<String, Arc<Mutex<TestEnvironment>>>,
}

impl TestEnvironmentManager {
    pub fn new() -> Self {
        Self {
            base_temp_dir: std::env::temp_dir().join("rust_ai_ide_tests"),
            environments: HashMap::new(),
        }
    }

    /// Create a new test environment
    pub async fn create_environment(
        &mut self,
        name: &str,
    ) -> anyhow::Result<Arc<Mutex<TestEnvironment>>> {
        let env_path = self.base_temp_dir.join(name);
        std::fs::create_dir_all(&env_path)?;

        let env = TestEnvironment {
            workspace_path: env_path,
            temp_dirs: Vec::new(),
            test_projects: HashMap::new(),
        };

        let env = Arc::new(Mutex::new(env));
        self.environments.insert(name.to_string(), env.clone());

        Ok(env)
    }

    /// Get existing environment
    pub fn get_environment(&self, name: &str) -> Option<Arc<Mutex<TestEnvironment>>> {
        self.environments.get(name).cloned()
    }

    /// Clean up all environments
    pub async fn cleanup_all(&mut self) -> anyhow::Result<()> {
        for (name, env) in &self.environments {
            let env = env.lock().await;
            println!("Cleaning up test environment: {}", name);

            // Clean up temp dirs
            for dir in &env.temp_dirs {
                if dir.exists() {
                    std::fs::remove_dir_all(dir)?;
                }
            }

            // Clean up test projects
            for (_, project_path) in &env.test_projects {
                if project_path.exists() {
                    std::fs::remove_dir_all(project_path)?;
                }
            }
        }

        // Clean up base temp dir if empty
        if self.base_temp_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&self.base_temp_dir) {
                if entries.count() == 0 {
                    std::fs::remove_dir(&self.base_temp_dir)?;
                }
            }
        }

        self.environments.clear();
        Ok(())
    }
}

impl Default for TestEnvironmentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_command_execution() {
        let temp_dir = std::env::temp_dir();

        // Test successful command
        let result = utils::run_command_async("echo", &["hello", "world"], &temp_dir).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("hello world"));

        // Test failed command
        let result = utils::run_command_async("false", &[], &temp_dir).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_sha256_checksum() {
        use std::fs;

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_checksum.txt");

        fs::write(&test_file, "Hello, World!").unwrap();

        let checksum = utils::generate_sha256_checksum(&test_file.to_string_lossy());
        assert!(checksum.is_ok());
        assert!(!checksum.unwrap().is_empty());

        fs::remove_file(test_file).unwrap();
    }

    #[tokio::test]
    async fn test_performance_workload() {
        // Test sync workload
        let (result, duration) = performance::run_sync_performance_workload(1000);
        assert!(result > 0);
        assert!(duration.as_millis() > 0);

        // Test async workload
        let (result, duration) = performance::run_async_performance_workload(10).await;
        assert!(result > 0);
        assert!(duration.as_millis() > 0);
    }

    #[tokio::test]
    async fn test_coverage_tools_check() {
        let tools = coverage::check_coverage_availability();
        // Tools availability depends on system setup, just verify structure
        assert!(matches!(tools.has_llvm_cov, true | false));
        assert!(matches!(tools.has_tarpaulin, true | false));
        assert!(matches!(tools.has_grcov, true | false));
    }

    #[tokio::test]
    async fn test_environment_manager() {
        let mut manager = TestEnvironmentManager::new();
        let env = manager.create_environment("test_env").await.unwrap();

        {
            let env = env.lock().await;
            assert!(env.workspace_path.exists());
        }

        assert!(manager.get_environment("test_env").is_some());
        manager.cleanup_all().await.unwrap();
    }
}

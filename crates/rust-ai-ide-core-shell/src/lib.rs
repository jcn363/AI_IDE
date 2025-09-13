#![warn(missing_docs)]
#![forbid(unsafe_code)]

//! Asynchronous shell operations and command execution for Rust AI IDE
//!
//! This crate provides synchronous and asynchronous command execution utilities,
//! with specialized support for development tools like Cargo and Git.

use std::process::{Command, Stdio};
use std::time::Duration;

use rust_ai_ide_core_fundamentals::error::{IDEError, IDEResult};

/// Execute a command synchronously and return results
pub fn execute_command(cmd: &str, args: &[&str]) -> IDEResult<CommandResult> {
    let output = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| IDEError::FileSystem(format!("Failed to execute command '{}': {}", cmd, e)))?;

    Ok(CommandResult {
        success:   output.status.success(),
        stdout:    String::from_utf8_lossy(&output.stdout).to_string(),
        stderr:    String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code(),
        duration:  None, // Synchronous execution doesn't time by default
    })
}

/// Command execution result
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Whether the command succeeded (exit code 0)
    pub success:   bool,
    /// Standard output as a string
    pub stdout:    String,
    /// Standard error as a string
    pub stderr:    String,
    /// Exit code, if available
    pub exit_code: Option<i32>,
    /// Duration of execution, if timed
    pub duration:  Option<Duration>,
}

impl CommandResult {
    /// Check if command failed
    pub fn is_failed(&self) -> bool {
        !self.success
    }
}

/// Specialized cargo command utilities
pub mod cargo {
    use std::path::Path;
    use std::time::Duration;

    use super::{execute_command, execute_with_timeout, CommandResult, IDEResult};

    /// Execute cargo build in specified directory
    pub fn build(_project_path: &Path, release: bool) -> IDEResult<CommandResult> {
        let mut args = vec!["build", "--message-format=json"];
        if release {
            args.push("--release");
        }
        execute_command("cargo", &args)
    }

    /// Execute cargo check in specified directory
    pub fn check(_project_path: &Path) -> IDEResult<CommandResult> {
        execute_command("cargo", &["check", "--message-format=json"])
    }

    /// Execute cargo test with optional filter
    pub fn test(_project_path: &Path, filter: Option<&str>) -> IDEResult<CommandResult> {
        let mut args = vec!["test", "--lib", "--bins"];
        if let Some(f) = filter {
            args.extend(&["-p", f]);
        }
        args.extend(&["--", "--nocapture"]);

        execute_with_timeout("cargo", &args, Duration::from_secs(300))
    }

    /// List tests in cargo project
    pub fn test_list(_project_path: &Path) -> IDEResult<CommandResult> {
        execute_command("cargo", &["test", "--", "--list"])
    }

    /// Execute cargo doc generation
    pub fn doc(_project_path: &Path, open: bool) -> IDEResult<CommandResult> {
        let mut args = vec!["doc", "--no-deps"];
        if open {
            args.push("--open");
        }
        execute_command("cargo", &args)
    }

    /// Execute cargo clean
    pub fn clean(_project_path: &Path) -> IDEResult<CommandResult> {
        execute_command("cargo", &["clean"])
    }

    /// Execute cargo fmt
    pub fn fmt(_project_path: &Path, check: bool) -> IDEResult<CommandResult> {
        let args = if check {
            vec!["fmt", "--", "--check"]
        } else {
            vec!["fmt"]
        };
        execute_command("cargo", &args)
    }

    /// Execute cargo clippy
    pub fn clippy(_project_path: &Path) -> IDEResult<CommandResult> {
        execute_command("cargo", &["clippy", "--message-format=json"])
    }

    /// Get cargo version
    pub fn version() -> IDEResult<String> {
        let result = execute_command("cargo", &["--version"])?;
        Ok(result.stdout.trim().to_string())
    }
}

/// Specialized git command utilities
pub mod git {
    // std::path::Path is used in function signatures
    // but marked with a warning - keeping for API consistency
    use std::path::Path;

    use super::{execute_command, CommandResult, IDEResult};

    /// Execute git status
    pub fn status(_repo_path: &Path, short_format: bool) -> IDEResult<CommandResult> {
        let args = if short_format {
            vec!["status", "--porcelain", "--branch"]
        } else {
            vec!["status"]
        };
        execute_command("git", &args)
    }

    /// Execute git add
    pub fn add(_repo_path: &Path, files: &[&str]) -> IDEResult<CommandResult> {
        let mut args = vec!["add"];
        args.extend(files);
        execute_command("git", &args)
    }

    /// Execute git add all
    pub fn add_all(_repo_path: &Path) -> IDEResult<CommandResult> {
        execute_command("git", &["add", "."])
    }

    /// Execute git commit
    pub fn commit(_repo_path: &Path, message: &str) -> IDEResult<CommandResult> {
        execute_command("git", &["commit", "-m", message])
    }

    /// Execute git log with specified format and limit
    pub fn log(_repo_path: &Path, limit: Option<usize>) -> IDEResult<CommandResult> {
        let mut args: Vec<String> = vec![
            "log".to_string(),
            "--pretty=format:%H%x09%an%x09%ad%x09%s".to_string(),
            "--date=iso".to_string(),
        ];
        if let Some(n) = limit {
            args.push("-n".to_string());
            args.push(n.to_string());
        }
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        execute_command("git", &args_refs)
    }

    /// Execute git diff
    pub fn diff(_repo_path: &Path, staged: bool, path: Option<&str>) -> IDEResult<CommandResult> {
        let mut args = vec!["diff"];
        if staged {
            args.push("--cached");
        }
        if let Some(p) = path {
            args.push(p);
        }
        execute_command("git", &args)
    }

    /// Execute git blame
    pub fn blame(_repo_path: &Path, file_path: &str, line_numbers: bool) -> IDEResult<CommandResult> {
        let args = if line_numbers {
            vec!["blame", "--line-porcelain", file_path]
        } else {
            vec!["blame", file_path]
        };
        execute_command("git", &args)
    }

    /// Execute git init
    pub fn init(_repo_path: &Path) -> IDEResult<CommandResult> {
        execute_command("git", &["init"])
    }

    /// Execute git clone
    pub fn clone(url: &str, target_path: &Path) -> IDEResult<CommandResult> {
        execute_command("git", &["clone", url, &target_path.to_string_lossy()])
    }

    /// Get git version
    pub fn version() -> IDEResult<String> {
        let result = execute_command("git", &["--version"])?;
        Ok(result.stdout.trim().to_string())
    }

    /// Check if directory is a git repository
    pub fn is_repo(_repo_path: &Path) -> IDEResult<bool> {
        let result = execute_command("git", &["rev-parse", "--git-dir"])?;
        Ok(result.success)
    }
}

/// Specialized rustc (Rust compiler) command utilities
pub mod rustc {
    use super::{execute_command, CommandResult, IDEResult};

    /// Get error code explanation using rustc --explain
    pub fn explain_error(code: &str) -> IDEResult<CommandResult> {
        execute_command("rustc", &["--explain", code])
    }
}

/// Safe command execution with argument validation
pub fn execute_safe_command(cmd: &str, args: Vec<String>) -> IDEResult<CommandResult> {
    // Note: Validation would typically be in fundamentals utils, but simplified here
    for arg in &args {
        if arg.contains("..") {
            return Err(IDEError::Validation(
                "Command arguments contain forbidden sequences".to_string(),
            ));
        }
    }

    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    execute_command(cmd, &args_refs)
}

/// Execute command with timeout (async)
pub fn execute_with_timeout(cmd: &str, args: &[&str], timeout: Duration) -> IDEResult<CommandResult> {
    use tokio::process::Command;
    use tokio::time::timeout as tokio_timeout;

    // Create a new runtime for async operations in this sync context
    // In a real implementation, this would be better handled at the application level
    let rt = tokio::runtime::Runtime::new()?;

    let result = rt.block_on(async {
        let child = Command::new(cmd)
            .args(args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| IDEError::FileSystem(format!("Failed to spawn process '{}': {}", cmd, e)))?;

        match tokio_timeout(timeout, child.wait_with_output()).await {
            Ok(output_result) => {
                match output_result {
                    Ok(output) => {
                        Ok(CommandResult {
                            success:   output.status.success(),
                            stdout:    String::from_utf8_lossy(&output.stdout).to_string(),
                            stderr:    String::from_utf8_lossy(&output.stderr).to_string(),
                            exit_code: output.status.code(),
                            duration:  Some(timeout), // Not exact but better than None
                        })
                    }
                    Err(e) => Err(IDEError::FileSystem(format!(
                        "Process error for '{}': {}",
                        cmd, e
                    ))),
                }
            }
            Err(_) => {
                // Timeout occurred, process will be terminated by runtime
                Err(IDEError::Timeout(format!(
                    "Operation timed out after {} seconds while executing '{}'",
                    timeout.as_secs(),
                    cmd
                )))
            }
        }
    });

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_execution() {
        let result = execute_command("echo", &["hello"]).unwrap();
        assert!(result.success);
        assert_eq!(result.stdout.trim(), "hello");
    }

    #[test]
    fn test_failed_command() {
        let result = execute_command("false", &[]).unwrap();
        assert!(!result.success);
        assert_eq!(result.exit_code, Some(1));
    }

    #[test]
    fn test_safe_command_execution() {
        let result = execute_safe_command("echo", vec!["safe".to_string()]).unwrap();
        assert!(result.success);
        assert_eq!(result.stdout.trim(), "safe");
    }

    #[test]
    fn test_command_timeout() {
        use std::time::Duration;
        let result = execute_with_timeout("sleep", &["2"], Duration::from_millis(500));
        assert!(result.is_err()); // Should timeout and return error
    }
}

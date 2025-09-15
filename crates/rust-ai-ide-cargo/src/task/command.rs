//! Command execution functionality for Cargo tasks

use std::process::Stdio;

use anyhow::Result;
use log::{debug, error};
use thiserror::Error;
use tokio::process::Command;

use super::{CargoTask, TaskResult};
use crate::task::SystemTime;

/// Error type for command execution
#[derive(Debug, Error, Clone)]
pub enum CommandError {
    /// Error during command execution
    #[error("Command execution failed: {0}")]
    ExecutionError(String),
    /// Command failed with non-zero exit code
    #[error("Command failed with exit code: {0}")]
    CommandError(i32),
    /// Command was terminated
    #[error("Command was terminated")]
    CommandTerminated,
}

// impl std::error::Error for CommandError is now handled by #[derive(Debug, Error, Clone)]

// Using a custom method to convert from io::Error
impl CommandError {
    pub fn from_io_error(err: std::io::Error) -> Self {
        CommandError::ExecutionError(err.to_string())
    }
}

/// Result type for command execution
pub type CommandResult = Result<TaskResult, CommandError>;

/// Status of a command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Handles execution of Cargo commands
pub struct CommandExecutor {
    task:   CargoTask,
    status: CommandStatus,
    result: Option<CommandResult>,
}

impl CommandExecutor {
    /// Create a new command executor for a task
    pub fn new(task: CargoTask) -> Self {
        Self {
            task,
            status: CommandStatus::Pending,
            result: None,
        }
    }

    /// Execute the command asynchronously
    pub async fn execute(&mut self) -> CommandResult {
        self.status = CommandStatus::Running;
        let mut result = TaskResult::new(self.task.clone());

        let mut cmd = Command::new("cargo");
        cmd.arg(&self.task.command);

        if self.task.release {
            cmd.arg("--release");
        }

        for arg in &self.task.args {
            cmd.arg(arg);
        }

        cmd.current_dir(&self.task.working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for (key, value) in &self.task.env {
            cmd.env(key, value);
        }

        debug!(
            "Executing command: {:?} in {:?}",
            cmd, self.task.working_dir
        );

        match cmd.spawn() {
            Ok(_child) => {
                let output = cmd.output().await.map_err(CommandError::from_io_error)?;

                result.exit_code = output.status.code();
                result.stdout = String::from_utf8_lossy(&output.stdout).to_string();
                result.stderr = String::from_utf8_lossy(&output.stderr).to_string();
                result.end_time = Some(std::time::SystemTime::now());

                if let Some(code) = result.exit_code {
                    if code != 0 {
                        error!("Command failed with code {}: {}", code, result.stderr);
                        self.status = CommandStatus::Failed;
                        result.success = false;
                        return Err(CommandError::CommandError(code));
                    }
                } else {
                    result.end_time = Some(SystemTime::now());
                    result.success = true;
                    return Err(CommandError::CommandTerminated);
                }

                self.status = CommandStatus::Completed;
                result.success = true;
                self.result = Some(Ok(result.clone()));
                Ok(result)
            }
            Err(e) => {
                error!("Failed to execute command: {}", e);
                self.status = CommandStatus::Failed;
                let err = CommandError::from_io_error(e);
                self.result = Some(Err(err.clone()));
                Err(err)
            }
        }
    }

    /// Get the current status of the command
    pub fn status(&self) -> CommandStatus {
        self.status
    }

    /// Get the result of the command execution, if available
    pub fn result(&self) -> Option<&CommandResult> {
        self.result.as_ref()
    }

    /// Cancel the command execution
    pub fn cancel(&mut self) {
        self.status = CommandStatus::Cancelled;
        // TODO: Implement actual process termination
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn test_command_execution() {
        let temp_dir = tempdir().unwrap();
        let task = CargoTask::new("version").working_dir(temp_dir.path());

        let mut executor = CommandExecutor::new(task);
        let result = executor.execute().await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("cargo"));
    }
}

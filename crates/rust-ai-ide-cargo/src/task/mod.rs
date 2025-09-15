//! Task automation module for Cargo commands
//!
//! This module provides functionality for executing, chaining, and monitoring
//! Task automation for Cargo commands

mod command;
mod history;
mod monitor;
mod task_chain;
mod types;

use std::path::PathBuf;
use std::time::SystemTime;

pub use command::{CommandError, CommandExecutor, CommandResult, CommandStatus};
pub use history::CommandHistory;
pub use monitor::{TaskInfo, TaskMonitor};
use serde::{Deserialize, Serialize};
pub use task_chain::TaskChain;
pub use types::{ExecutionStrategy, TaskStatus};

/// Represents a Cargo task to be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoTask {
    /// The Cargo command to run (e.g., "build", "test", "run")
    pub command: String,
    /// Arguments to pass to the command
    pub args: Vec<String>,
    /// Working directory for the command
    pub working_dir: PathBuf,
    /// Whether to build in release mode
    pub release: bool,
    /// Environment variables to set for the command
    pub env: Vec<(String, String)>,
}

impl CargoTask {
    /// Create a new Cargo task
    pub fn new<S: Into<String>>(command: S) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            working_dir: std::env::current_dir().unwrap_or_default(),
            release: false,
            env: Vec::new(),
        }
    }

    /// Add an argument to the command
    pub fn arg<S: Into<String>>(mut self, arg: S) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Set the working directory
    pub fn working_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.working_dir = dir.into();
        self
    }

    /// Set release mode
    pub fn release(mut self, release: bool) -> Self {
        self.release = release;
        self
    }

    /// Add an environment variable
    pub fn env<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }
}

/// Represents the result of a task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// The task that was executed
    pub task: CargoTask,
    /// Exit code of the command, if available
    pub exit_code: Option<i32>,
    /// Standard output of the command
    pub stdout: String,
    /// Standard error of the command
    pub stderr: String,
    /// When the command started
    pub start_time: SystemTime,
    /// When the command finished
    pub end_time: Option<SystemTime>,
    /// Whether the command was successful
    pub success: bool,
}

impl TaskResult {
    /// Create a new task result
    pub fn new(task: CargoTask) -> Self {
        Self {
            task,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            start_time: SystemTime::now(),
            end_time: None,
            success: false,
        }
    }

    /// Calculate the duration of the task
    pub fn duration(&self) -> Option<std::time::Duration> {
        self.end_time?.duration_since(self.start_time).ok()
    }

    /// Create a new successful task result
    pub fn success(task: CargoTask, stdout: String, stderr: String) -> Self {
        let now = SystemTime::now();
        Self {
            task,
            exit_code: Some(0),
            stdout,
            stderr,
            start_time: now,
            end_time: Some(now),
            success: true,
        }
    }

    /// Create a new failed task result
    pub fn failure(
        task: CargoTask,
        exit_code: Option<i32>,
        stdout: String,
        stderr: String,
    ) -> Self {
        let now = SystemTime::now();
        Self {
            task,
            exit_code,
            stdout,
            stderr,
            start_time: now,
            end_time: Some(now),
            success: false,
        }
    }

    /// Convert from a CommandResult to a TaskResult
    pub fn from_command_result(result: CommandResult) -> Self {
        match result {
            Ok(task_result) => task_result,
            Err(e) => {
                let task = match e {
                    CommandError::ExecutionError(_) => CargoTask::new("unknown"),
                    CommandError::CommandError(code) => {
                        CargoTask::new("unknown").arg(code.to_string())
                    }
                    CommandError::CommandTerminated => CargoTask::new("unknown").arg("terminated"),
                };
                Self::failure(task, None, String::new(), e.to_string())
            }
        }
    }
}

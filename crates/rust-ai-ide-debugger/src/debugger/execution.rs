//! Debugger execution control and backend management

use super::DebuggerBackendTrait;
use crate::debugger::types::{DebuggerConfig, DebuggerEvent, StackFrame, VariableInfo};
use crate::debugger::DebuggerError;
use log::{debug, error, info};
use serde_json::Value;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

/// Supported debugger backends
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DebuggerBackendType {
    Gdb,
    Lldb,
}

impl Default for DebuggerBackendType {
    fn default() -> Self {
        // Default to GDB as it's more widely available
        DebuggerBackendType::Gdb
    }
}

/// Debugger backend implementation
pub struct DebuggerBackend {
    /// The debugger process
    process: Option<Child>,
    /// Debugger type (GDB/LLDB)
    backend_type: DebuggerBackendType,
    /// Debugger configuration
    config: Option<DebuggerConfig>,
    /// Event sender for debugger events
    event_sender: Option<tokio::sync::mpsc::UnboundedSender<DebuggerEvent>>,
    /// Output buffer for debugger output
    _output_buffer: String,
}

/// Result type for debugger operations
pub type DebuggerResult<T> = Result<T, String>;

#[async_trait::async_trait]
impl DebuggerBackendTrait for DebuggerBackend {
    async fn start(&mut self, config: &DebuggerConfig) -> Result<(), DebuggerError> {
        self.start(config.clone()).await?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), DebuggerError> {
        self.stop().await?;
        Ok(())
    }

    async fn add_breakpoint(&mut self, file: &str, line: u32) -> Result<u32, DebuggerError> {
        self.set_breakpoint(file, line).await?;
        // In a real implementation, we would return the actual breakpoint ID
        Ok(1)
    }

    async fn remove_breakpoint(&mut self, id: u32) -> Result<(), DebuggerError> {
        self.remove_breakpoint(id).await?;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.process.is_some()
    }

    async fn step_over(&mut self) -> Result<(), DebuggerError> {
        self.step_over().await?;
        Ok(())
    }

    async fn step_into(&mut self) -> Result<(), DebuggerError> {
        self.step_into().await?;
        Ok(())
    }

    async fn step_out(&mut self) -> Result<(), DebuggerError> {
        self.step_out().await?;
        Ok(())
    }

    async fn continue_execution(&mut self) -> Result<(), DebuggerError> {
        self.continue_execution().await?;
        Ok(())
    }

    async fn pause(&mut self) -> Result<(), DebuggerError> {
        // TODO: Implementation depends on the specific debugger backend
        // This is a placeholder implementation
        Ok(())
    }

    async fn get_stack_trace(&self) -> Result<Vec<StackFrame>, DebuggerError> {
        // TODO: Implementation depends on the specific debugger backend
        // This is a placeholder implementation
        Ok(Vec::new())
    }

    async fn get_variables(
        &self,
        _frame_id: Option<u32>,
    ) -> Result<Vec<VariableInfo>, DebuggerError> {
        // TODO: Implementation depends on the specific debugger backend
        // This is a placeholder implementation
        Ok(Vec::new())
    }

    async fn evaluate_expression(
        &self,
        _expression: &str,
        _frame_id: Option<u32>,
    ) -> Result<VariableInfo, DebuggerError> {
        // TODO: Implementation depends on the specific debugger backend
        // This is a placeholder implementation
        Err(DebuggerError::NotImplemented)
    }

    fn set_event_sender(&mut self, sender: tokio::sync::mpsc::UnboundedSender<DebuggerEvent>) {
        self.event_sender = Some(sender);
    }
}

impl DebuggerBackend {
    /// Create a new debugger backend
    pub fn new(backend_type: DebuggerBackendType) -> Self {
        Self {
            process: None,
            backend_type,
            config: None,
            event_sender: None,
            _output_buffer: String::new(),
        }
    }

    /// Set the event sender for debugger events
    pub fn set_event_sender(&mut self, sender: tokio::sync::mpsc::UnboundedSender<DebuggerEvent>) {
        self.event_sender = Some(sender);
    }

    /// Start the debugger process
    pub async fn start(&mut self, config: DebuggerConfig) -> DebuggerResult<()> {
        self.config = Some(config.clone());
        let debugger = match self.backend_type {
            DebuggerBackendType::Gdb => "gdb",
            DebuggerBackendType::Lldb => "lldb",
        };

        info!("Starting {} debugger...", debugger);

        let mut command = Command::new(debugger);

        // Set up the command with appropriate arguments
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("--interpreter=mi2"); // Use MI2 interface

        // Add target program if specified
        if !config.target.is_empty() {
            command.arg(&config.target);
        }

        // Set working directory if specified
        if let Some(working_dir) = &config.working_dir {
            command.current_dir(working_dir);
        }

        // Set environment variables
        command.envs(&config.env);

        // Spawn the process
        let mut child = command
            .spawn()
            .map_err(|e| format!("Failed to start {}: {}", debugger, e))?;

        // Start output monitoring
        self.start_output_monitoring(child.stdout.take())
            .await
            .map_err(|e| format!("Failed to start output monitoring: {}", e))?;

        // Store the process handle
        self.process = Some(child);

        // Initialize debugger
        self.initialize_debugger().await?;

        info!("{} debugger started successfully", debugger);
        Ok(())
    }

    /// Initialize the debugger with common settings
    async fn initialize_debugger(&mut self) -> DebuggerResult<()> {
        // Set up common debugger settings
        self.send_command("-gdb-set confirm off").await?;
        self.send_command("-gdb-set print pretty on").await?;
        self.send_command("-gdb-set print object on").await?;
        self.send_command("-gdb-set print static-members on")
            .await?;
        self.send_command("-gdb-set print vtbl on").await?;

        Ok(())
    }

    /// Start monitoring debugger output
    async fn start_output_monitoring(
        &mut self,
        stdout: Option<tokio::process::ChildStdout>,
    ) -> DebuggerResult<()> {
        let stdout = stdout.ok_or("No stdout handle available")?;
        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                debug!("Debugger output: {}", line);

                // Parse MI output and send appropriate events
                if let Some(event) = Self::parse_mi_output(&line) {
                    if let Some(sender) = &event_sender {
                        let _ = sender.send(event);
                    }
                }
            }
        });

        Ok(())
    }

    /// Parse MI output and convert to debugger events
    fn parse_mi_output(output: &str) -> Option<DebuggerEvent> {
        // This is a simplified parser - in a real implementation, you'd want to
        // TODO: properly parse the MI2 output format
        if output.starts_with("*stopped") {
            // Parse stop reason and location
            Some(DebuggerEvent::StateChanged(
                crate::debugger::types::DebuggerState::Paused {
                    reason: "Breakpoint hit".to_string(),
                    location: None, // Would parse this from the output
                },
            ))
        } else if output.starts_with("=breakpoint-created") {
            // Parse breakpoint creation
            None // Would create a BreakpointCreated event
        } else {
            None
        }
    }

    /// Stop the debugger process
    pub async fn stop(&mut self) -> DebuggerResult<()> {
        if let Some(mut child) = self.process.take() {
            info!("Stopping debugger...");

            // Try to gracefully terminate first
            if let Err(e) = child.kill().await {
                error!("Failed to stop debugger: {}", e);
                return Err(format!("Failed to stop debugger: {}", e));
            }

            // Wait for the process to exit
            if let Err(e) = child.wait().await {
                error!("Error waiting for debugger to exit: {}", e);
                return Err(format!("Error waiting for debugger to exit: {}", e));
            }

            info!("Debugger stopped successfully");
        }

        Ok(())
    }

    /// Send a command to the debugger
    pub async fn send_command(&mut self, command: &str) -> DebuggerResult<String> {
        let process = self
            .process
            .as_mut()
            .ok_or("Debugger process not running")?;

        let stdin = process
            .stdin
            .as_mut()
            .ok_or("Debugger stdin not available")?;

        debug!("Sending command: {}", command);

        // Send the command
        use tokio::io::AsyncWriteExt;
        stdin
            .write_all(command.as_bytes())
            .await
            .map_err(|e| format!("Failed to send command: {}", e))?;
        stdin
            .write_all(b"\n")
            .await
            .map_err(|e| format!("Failed to send newline: {}", e))?;
        stdin
            .flush()
            .await
            .map_err(|e| format!("Failed to flush stdin: {}", e))?;

        // For now, return an empty string as the response
        // TODO: read and parse the response
        Ok(String::new())
    }

    /// Execute a command and wait for the result
    pub async fn execute_command(&mut self, command: &str) -> DebuggerResult<Value> {
        let response = self.send_command(command).await?;

        // Parse the MI output into a structured format
        // This is a simplified version - in a real implementation, you'd want
        // TODO: to properly parse the MI2 output format
        serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse debugger output: {}", e))
    }

    /// Set a breakpoint at the specified location
    pub async fn set_breakpoint(&mut self, file: &str, line: u32) -> DebuggerResult<()> {
        let cmd = format!("-break-insert {}:{}", file, line);
        self.send_command(&cmd).await?;
        Ok(())
    }

    /// Remove a breakpoint by ID
    pub async fn remove_breakpoint(&mut self, breakpoint_id: u32) -> DebuggerResult<()> {
        let cmd = format!("-break-delete {}", breakpoint_id);
        self.send_command(&cmd).await?;
        Ok(())
    }

    /// Continue execution
    pub async fn continue_execution(&mut self) -> DebuggerResult<()> {
        self.send_command("-exec-continue").await?;
        Ok(())
    }

    /// Step over the next source line
    pub async fn step_over(&mut self) -> DebuggerResult<()> {
        self.send_command("-exec-next").await?;
        Ok(())
    }

    /// Step into the next function call
    pub async fn step_into(&mut self) -> DebuggerResult<()> {
        self.send_command("-exec-step").await?;
        Ok(())
    }

    /// Step out of the current function
    pub async fn step_out(&mut self) -> DebuggerResult<()> {
        self.send_command("-exec-finish").await?;
        Ok(())
    }
}

impl Default for DebuggerBackend {
    fn default() -> Self {
        Self::new(DebuggerBackendType::default())
    }
}

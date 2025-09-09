//! Debugger backend trait and implementations

use crate::debugger::types::{DebuggerConfig, DebuggerEvent, StackFrame, VariableInfo};
use crate::debugger::DebuggerError;
use async_trait::async_trait;

/// Trait defining the interface for debugger backends
#[async_trait]
pub trait DebuggerBackendTrait: Send + Sync {
    /// Start the debugger with the given configuration
    async fn start(&mut self, config: &DebuggerConfig) -> Result<(), DebuggerError>;

    /// Stop the debugger
    async fn stop(&mut self) -> Result<(), DebuggerError>;

    /// Add a breakpoint at the specified location
    async fn add_breakpoint(&mut self, file: &str, line: u32) -> Result<u32, DebuggerError>;

    /// Remove a breakpoint by ID
    async fn remove_breakpoint(&mut self, id: u32) -> Result<(), DebuggerError>;

    /// Check if the debugger is running
    fn is_running(&self) -> bool;

    /// Step over the next source line
    async fn step_over(&mut self) -> Result<(), DebuggerError>;

    /// Step into the next function call
    async fn step_into(&mut self) -> Result<(), DebuggerError>;

    /// Step out of the current function
    async fn step_out(&mut self) -> Result<(), DebuggerError>;

    /// Continue execution
    async fn continue_execution(&mut self) -> Result<(), DebuggerError>;

    /// Pause execution
    async fn pause(&mut self) -> Result<(), DebuggerError>;

    /// Get the current stack trace
    async fn get_stack_trace(&self) -> Result<Vec<StackFrame>, DebuggerError>;

    /// Get variables in the current scope
    async fn get_variables(
        &self,
        frame_id: Option<u32>,
    ) -> Result<Vec<VariableInfo>, DebuggerError>;

    /// Evaluate an expression in the current context
    async fn evaluate_expression(
        &self,
        expression: &str,
        frame_id: Option<u32>,
    ) -> Result<VariableInfo, DebuggerError>;

    /// Set the event sender for debugger events
    fn set_event_sender(&mut self, sender: tokio::sync::mpsc::UnboundedSender<DebuggerEvent>);
}

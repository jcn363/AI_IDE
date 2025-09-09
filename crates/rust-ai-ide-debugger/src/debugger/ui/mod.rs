//! User interface components for the debugger

use crate::debugger::types::DebuggerEvent;

/// Handles debugger user interface
pub struct DebuggerUI {
    /// Channel for sending events to the debugger
    _event_sender: tokio::sync::mpsc::UnboundedSender<DebuggerEvent>,
}

impl DebuggerUI {
    /// Create a new DebuggerUI instance
    pub fn new(event_sender: tokio::sync::mpsc::UnboundedSender<DebuggerEvent>) -> Self {
        Self {
            _event_sender: event_sender,
        }
    }

    /// Show a notification to the user
    pub fn show_notification(&self, message: &str) {
        // In a real implementation, this would display a notification in the UI
        println!("[DEBUGGER] {}", message);
    }

    /// Update the debugger state display
    pub fn update_state(&self, state: &crate::debugger::types::DebuggerState) {
        // In a real implementation, this would update the UI to reflect the current state
        println!("[DEBUGGER] State updated: {:?}", state);
    }

    /// Update the call stack display
    pub fn update_call_stack(&self, stack: &[crate::debugger::types::StackFrame]) {
        // In a real implementation, this would update the call stack view
        println!("[DEBUGGER] Call stack updated ({} frames)", stack.len());
    }

    /// Update the variables display
    pub fn update_variables(&self, variables: &[crate::debugger::types::VariableInfo]) {
        // In a real implementation, this would update the variables view
        println!(
            "[DEBUGGER] Variables updated ({} variables)",
            variables.len()
        );
    }

    /// Show an error message
    pub fn show_error(&self, message: &str) {
        // In a real implementation, this would display an error dialog
        eprintln!("[DEBUGGER ERROR] {}", message);
    }
}

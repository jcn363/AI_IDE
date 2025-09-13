//! Debugger state management

use std::collections::HashMap;

use crate::debugger::types::{DebuggerEvent, DebuggerState, StackFrame, VariableInfo};

/// Manages the debugger's state
pub struct StateManager {
    /// Current debugger state
    state:         DebuggerState,
    /// Current call stack
    call_stack:    Vec<StackFrame>,
    /// Current variables in scope
    variables:     HashMap<String, VariableInfo>,
    /// Current frame index in the call stack
    current_frame: Option<usize>,
}

impl StateManager {
    /// Create a new StateManager
    pub fn new() -> Self {
        Self {
            state:         DebuggerState::Disconnected,
            call_stack:    Vec::new(),
            variables:     HashMap::new(),
            current_frame: None,
        }
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StateManager {
    /// Update the debugger state
    pub fn set_state(&mut self, state: DebuggerState) -> Option<DebuggerEvent> {
        self.state = state.clone();
        Some(DebuggerEvent::StateChanged(state))
    }

    /// Get the current debugger state
    pub fn get_state(&self) -> &DebuggerState {
        &self.state
    }

    /// Update the call stack
    pub fn update_call_stack(&mut self, stack: Vec<StackFrame>) -> Option<DebuggerEvent> {
        self.call_stack = stack;
        if !self.call_stack.is_empty() && self.current_frame.is_none() {
            self.current_frame = Some(0);
        }
        Some(DebuggerEvent::CallStackUpdated(self.call_stack.clone()))
    }

    /// Update variables in the current scope
    pub fn update_variables(&mut self, variables: Vec<VariableInfo>) -> Option<DebuggerEvent> {
        self.variables.clear();
        for var in &variables {
            self.variables.insert(var.name.clone(), var.clone());
        }
        Some(DebuggerEvent::VariablesUpdated(variables))
    }

    /// Select the current stack frame
    pub fn select_frame(&mut self, frame_idx: usize) -> Option<DebuggerEvent> {
        if frame_idx < self.call_stack.len() {
            self.current_frame = Some(frame_idx);
            // In a real implementation, we would update variables for the selected frame
            Some(DebuggerEvent::CallStackUpdated(self.call_stack.clone()))
        } else {
            None
        }
    }

    /// Get the current stack frame
    pub fn get_current_frame(&self) -> Option<&StackFrame> {
        self.current_frame.and_then(|idx| self.call_stack.get(idx))
    }

    /// Get all variables in the current scope
    pub fn get_variables(&self) -> Vec<&VariableInfo> {
        self.variables.values().collect()
    }

    /// Get a specific variable by name
    pub fn get_variable(&self, name: &str) -> Option<&VariableInfo> {
        self.variables.get(name)
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.state = DebuggerState::Disconnected;
        self.call_stack.clear();
        self.variables.clear();
        self.current_frame = None;
    }
}

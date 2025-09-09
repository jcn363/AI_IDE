//! Expression evaluation and watch expression management

use log::{debug, trace};
use std::collections::HashMap;

use crate::debugger::error::{DebuggerError, Result as DebuggerResult};
use crate::debugger::types::{DebuggerEvent, DebuggerState, VariableInfo};

/// Manages watch expressions in the debugger
///
/// Handles the lifecycle of watch expressions, including adding, removing,
/// and evaluating expressions in the current debugging context.
#[derive(Debug)]
pub struct ExpressionManager {
    /// Map of watch expressions to their current values and metadata
    watch_expressions: HashMap<String, VariableInfo>,

    /// Next available watch expression ID
    next_id: u32,

    /// Whether to automatically evaluate expressions on stop events
    auto_evaluate: bool,
}

impl ExpressionManager {
    /// Create a new ExpressionManager
    pub fn new() -> Self {
        Self {
            watch_expressions: HashMap::new(),
            next_id: 1,
            auto_evaluate: true,
        }
    }
}

impl Default for ExpressionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionManager {
    /// Get the next available watch expression ID and increment the counter
    pub fn get_next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Set whether to automatically evaluate expressions on stop events
    pub fn set_auto_evaluate(&mut self, enabled: bool) {
        self.auto_evaluate = enabled;
        debug!(
            "Auto-evaluation of watch expressions {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }

    /// Add a new watch expression
    ///
    /// # Arguments
    /// * `expression` - The expression to watch (e.g., variable name or more complex expression)
    ///
    /// # Returns
    /// The ID of the newly created watch expression
    ///
    /// # Errors
    /// Returns an error if the expression is invalid or already exists
    pub fn add_watch_expression(&mut self, expression: impl Into<String>) -> DebuggerResult<u32> {
        let expr = expression.into();

        // Validate expression (basic validation - could be enhanced)
        if expr.trim().is_empty() {
            return Err(DebuggerError::eval_error("Expression cannot be empty"));
        }

        // Check for duplicate expressions
        if self.watch_expressions.contains_key(&expr) {
            return Err(DebuggerError::eval_error(format!(
                "Expression '{}' is already being watched",
                expr
            )));
        }

        let id = self.next_id;
        self.next_id += 1;

        let var_info = VariableInfo {
            id: Some(id),
            name: format!("${}", id), // Temporary name until evaluated
            value: String::new(),
            type_name: String::new(),
            in_scope: false,
            children: Vec::new(),
            expression: Some(expr.clone()),
        };

        debug!("Added watch expression: {}", expr);
        self.watch_expressions.insert(expr, var_info);

        Ok(id)
    }

    /// Remove a watch expression by its ID
    ///
    /// # Arguments
    /// * `id` - The ID of the watch expression to remove
    ///
    /// # Returns
    /// The removed watch expression if it existed
    ///
    /// # Errors
    /// Returns an error if no expression with the given ID exists
    pub fn remove_watch_expression_by_id(&mut self, id: u32) -> DebuggerResult<VariableInfo> {
        let expr = self.find_expression_by_id(id)?;
        self.watch_expressions
            .remove(&expr)
            .ok_or_else(|| DebuggerError::eval_error("Watch expression not found"))
    }

    /// Remove a watch expression by its text
    ///
    /// # Arguments
    /// * `expression` - The expression text to remove
    ///
    /// # Returns
    /// The removed watch expression if it existed
    pub fn remove_watch_expression(&mut self, expression: &str) -> Option<VariableInfo> {
        let result = self.watch_expressions.remove(expression);
        if result.is_some() {
            debug!("Removed watch expression: {}", expression);
        }
        result
    }

    /// Update the value of a watch expression
    ///
    /// # Arguments
    /// * `id` - The ID of the watch expression to update
    /// * `value` - The new value information
    ///
    /// # Returns
    /// The previous value if the expression existed
    ///
    /// # Errors
    /// Returns an error if no expression with the given ID exists
    pub fn update_watch_expression_by_id(
        &mut self,
        id: u32,
        value: VariableInfo,
    ) -> DebuggerResult<Option<VariableInfo>> {
        let expr = self.find_expression_by_id(id)?;

        // Make sure the ID matches
        if value.id != Some(id) {
            return Err(DebuggerError::eval_error("ID mismatch in update"));
        }

        Ok(self.watch_expressions.insert(expr, value))
    }

    /// Find an expression by its ID
    pub fn find_expression_by_id(&self, id: u32) -> DebuggerResult<String> {
        self.watch_expressions
            .iter()
            .find(|(_, v)| v.id == Some(id))
            .map(|(expr, _)| expr.clone())
            .ok_or_else(|| DebuggerError::eval_error("Watch expression not found"))
    }

    /// Get all watch expressions
    ///
    /// # Returns
    /// A vector of references to all watch expressions, sorted by ID
    pub fn get_watch_expressions(&self) -> Vec<&VariableInfo> {
        let mut expressions: Vec<_> = self.watch_expressions.values().collect();
        expressions.sort_by_key(|v| v.id);
        expressions
    }

    /// Get a watch expression by its ID
    ///
    /// # Arguments
    /// * `id` - The ID of the watch expression to retrieve
    ///
    /// # Returns
    /// A reference to the watch expression if found
    ///
    /// # Errors
    /// Returns an error if no expression with the given ID exists
    pub fn get_watch_expression(&self, id: u32) -> DebuggerResult<&VariableInfo> {
        self.watch_expressions
            .values()
            .find(|v| v.id == Some(id))
            .ok_or_else(|| DebuggerError::eval_error("Watch expression not found"))
    }

    /// Evaluate all watch expressions in the current debugger context
    ///
    /// # Arguments
    /// * `state` - The current debugger state
    ///
    /// # Returns
    /// A vector of debugger events resulting from the evaluation
    ///
    /// # Errors
    /// Returns an error if evaluation fails for any expression
    pub fn evaluate_all(&mut self, state: &DebuggerState) -> DebuggerResult<Vec<DebuggerEvent>> {
        let mut events = Vec::new();

        // Skip evaluation if not in a state where expressions can be evaluated
        if !matches!(state, DebuggerState::Paused { .. }) {
            trace!("Skipping expression evaluation - debugger not paused");
            return Ok(events);
        }

        // First, collect all expressions and their current state
        let expressions: Vec<(String, VariableInfo)> = self
            .watch_expressions
            .iter()
            .map(|(expr, var_info)| (expr.clone(), var_info.clone()))
            .collect::<Vec<(String, VariableInfo)>>();

        // Then evaluate each expression and track changes
        for (expr, mut var_info) in expressions {
            match self.evaluate_expression(&expr, state) {
                Ok(new_var_info) => {
                    // Only update and send event if the value changed
                    if var_info.value != new_var_info.value
                        || var_info.in_scope != new_var_info.in_scope
                    {
                        // Update the stored variable info
                        if let Some(stored_info) = self.watch_expressions.get_mut(&expr) {
                            *stored_info = new_var_info.clone();
                            events.push(DebuggerEvent::VariablesUpdated(vec![new_var_info]));
                        }
                    }
                }
                Err(e) => {
                    // Log the error but continue with other expressions
                    debug!("Failed to evaluate expression '{}': {}", expr, e);

                    // Update the variable info to show the error
                    let old_value = var_info.value.clone();
                    let old_in_scope = var_info.in_scope;
                    var_info.value = format!("Error: {}", e);
                    var_info.in_scope = false;

                    // Only send an event if the value or scope changed
                    if old_value != var_info.value || old_in_scope != var_info.in_scope {
                        // Update the stored variable info
                        if let Some(stored_info) = self.watch_expressions.get_mut(&expr) {
                            *stored_info = var_info.clone();
                            events.push(DebuggerEvent::VariablesUpdated(vec![var_info]));
                        }
                    }
                }
            }
        }

        Ok(events)
    }

    /// Evaluate a single expression
    /// Evaluate an expression in the current debug context
    pub fn evaluate_expression(
        &self,
        expression: &str,
        _state: &DebuggerState,
    ) -> DebuggerResult<VariableInfo> {
        // TODO: Implement actual expression evaluation using the debugger backend
        // For now, return a placeholder value
        Ok(VariableInfo {
            id: None, // Will be set by the caller
            name: expression.to_string(),
            value: "<evaluating...>".to_string(),
            type_name: "unknown".to_string(),
            in_scope: true,
            children: Vec::new(),
            expression: Some(expression.to_string()),
        })
    }
}

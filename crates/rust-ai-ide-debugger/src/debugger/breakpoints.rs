//! Breakpoint management for the debugger

use log::{debug, info};
use std::collections::HashMap;
use std::path::Path;

use crate::debugger::error::{DebuggerError, Result as DebuggerResult};
use crate::debugger::types::BreakpointInfo;

/// Manages breakpoints in the debugger
pub struct BreakpointManager {
    /// Map of breakpoint IDs to breakpoint info
    breakpoints: HashMap<u32, BreakpointInfo>,
    /// Next available breakpoint ID
    next_id: u32,
}

impl BreakpointManager {
    /// Create a new BreakpointManager
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
            next_id: 1,
        }
    }
}

impl Default for BreakpointManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BreakpointManager {
    /// Add a new breakpoint
    ///
    /// # Arguments
    /// * `file` - Path to the source file
    /// * `line` - Line number in the source file
    /// * `condition` - Optional condition for conditional breakpoint
    /// * `log_message` - Optional log message to display when breakpoint is hit
    ///
    /// # Returns
    /// The ID of the newly created breakpoint
    ///
    /// # Errors
    /// Returns an error if the file path is invalid or line number is zero
    pub fn add_breakpoint(
        &mut self,
        file: impl AsRef<Path>,
        line: u32,
        condition: Option<String>,
        log_message: Option<String>,
    ) -> DebuggerResult<u32> {
        if line == 0 {
            return Err(DebuggerError::breakpoint_error(
                "Line number must be greater than zero",
            ));
        }

        // Normalize the file path
        let file_path = file.as_ref().canonicalize().map_err(|_| {
            DebuggerError::breakpoint_error(format!("File not found: {}", file.as_ref().display()))
        })?;

        let id = self.next_id;
        self.next_id += 1;

        let bp = BreakpointInfo {
            id,
            file: Some(file_path.clone()),
            line,
            condition,
            enabled: true,
            hit_count: 0,
            log_message,
        };

        debug!(
            "Added breakpoint: {}:{} (id={})",
            file_path.display(),
            bp.line,
            bp.id
        );
        self.breakpoints.insert(id, bp);

        Ok(id)
    }

    /// Remove a breakpoint by ID
    ///
    /// # Arguments
    /// * `id` - The ID of the breakpoint to remove
    ///
    /// # Returns
    /// The removed breakpoint if it existed
    ///
    /// # Errors
    /// Returns an error if the breakpoint doesn't exist
    pub fn remove_breakpoint(&mut self, id: u32) -> DebuggerResult<BreakpointInfo> {
        self.breakpoints
            .remove(&id)
            .ok_or_else(|| DebuggerError::breakpoint_error(format!("Breakpoint {} not found", id)))
    }

    /// Get a breakpoint by ID
    ///
    /// # Arguments
    /// * `id` - The ID of the breakpoint to retrieve
    ///
    /// # Returns
    /// A reference to the breakpoint if it exists
    ///
    /// # Errors
    /// Returns an error if the breakpoint doesn't exist
    pub fn get_breakpoint(&self, id: u32) -> DebuggerResult<&BreakpointInfo> {
        self.breakpoints
            .get(&id)
            .ok_or_else(|| DebuggerError::breakpoint_error(format!("Breakpoint {} not found", id)))
    }

    /// Get a mutable reference to a breakpoint by ID
    ///
    /// # Arguments
    /// * `id` - The ID of the breakpoint to retrieve
    ///
    /// # Returns
    /// A mutable reference to the breakpoint if it exists
    ///
    /// # Errors
    /// Returns an error if the breakpoint doesn't exist
    pub fn get_breakpoint_mut(&mut self, id: u32) -> DebuggerResult<&mut BreakpointInfo> {
        self.breakpoints
            .get_mut(&id)
            .ok_or_else(|| DebuggerError::breakpoint_error(format!("Breakpoint {} not found", id)))
    }

    /// Get all breakpoints
    ///
    /// # Returns
    /// A vector of references to all breakpoints
    pub fn get_breakpoints(&self) -> Vec<&BreakpointInfo> {
        self.breakpoints.values().collect()
    }

    /// Get all breakpoints for a specific file
    ///
    /// # Arguments
    /// * `file` - The file path to get breakpoints for
    ///
    /// # Returns
    /// A vector of references to breakpoints in the specified file
    pub fn get_breakpoints_for_file(&self, file: impl AsRef<Path>) -> Vec<&BreakpointInfo> {
        let file_path = match file.as_ref().canonicalize() {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        self.breakpoints
            .values()
            .filter(|bp| {
                if let Some(bp_file) = &bp.file {
                    if let Ok(bp_path) = bp_file.canonicalize() {
                        return bp_path == file_path;
                    }
                }
                false
            })
            .collect()
    }

    /// Toggle the enabled state of a breakpoint
    ///
    /// # Arguments
    /// * `id` - The ID of the breakpoint to toggle
    ///
    /// # Returns
    /// The new enabled state of the breakpoint
    ///
    /// # Errors
    /// Returns an error if the breakpoint doesn't exist
    pub fn toggle_breakpoint(&mut self, id: u32) -> DebuggerResult<bool> {
        let bp = self.get_breakpoint_mut(id)?;
        bp.enabled = !bp.enabled;

        debug!(
            "Breakpoint {} {}",
            id,
            if bp.enabled { "enabled" } else { "disabled" }
        );

        Ok(bp.enabled)
    }

    /// Update a breakpoint's condition
    ///
    /// # Arguments
    /// * `id` - The ID of the breakpoint to update
    /// * `condition` - The new condition (or None to remove the condition)
    ///
    /// # Returns
    /// The previous condition, if any
    ///
    /// # Errors
    /// Returns an error if the breakpoint doesn't exist
    pub fn update_breakpoint_condition(
        &mut self,
        id: u32,
        condition: Option<String>,
    ) -> DebuggerResult<Option<String>> {
        let bp = self.get_breakpoint_mut(id)?;
        let prev_condition = std::mem::replace(&mut bp.condition, condition);

        debug!("Updated breakpoint {} condition: {:?}", id, bp.condition);

        Ok(prev_condition)
    }

    /// Increment the hit count for a breakpoint
    ///
    /// # Arguments
    /// * `id` - The ID of the breakpoint to update
    ///
    /// # Returns
    /// The new hit count
    ///
    /// # Errors
    /// Returns an error if the breakpoint doesn't exist
    pub fn increment_hit_count(&mut self, id: u32) -> DebuggerResult<u64> {
        let bp = self.get_breakpoint_mut(id)?;
        bp.hit_count = bp.hit_count.saturating_add(1);

        debug!("Breakpoint {} hit (total: {})", id, bp.hit_count);

        if let Some(ref msg) = bp.log_message {
            info!("Breakpoint {}: {}", id, msg);
        }

        Ok(bp.hit_count as u64)
    }

    /// Remove all breakpoints for a specific file
    ///
    /// # Arguments
    /// * `file` - The file path to remove breakpoints from
    ///
    /// # Returns
    /// The number of breakpoints removed
    pub fn remove_breakpoints_for_file(&mut self, file: impl AsRef<Path>) -> usize {
        let file_path = match file.as_ref().canonicalize() {
            Ok(p) => p,
            Err(_) => return 0,
        };

        let mut removed = 0;
        let mut ids_to_remove = Vec::new();

        // First collect all matching breakpoint IDs
        for (id, bp) in &self.breakpoints {
            if let Some(bp_file) = &bp.file {
                if let Ok(bp_path) = bp_file.canonicalize() {
                    if bp_path == file_path {
                        ids_to_remove.push(*id);
                    }
                }
            }
        }

        // Then remove them
        for id in ids_to_remove {
            if self.breakpoints.remove(&id).is_some() {
                removed += 1;
            }
        }

        if removed > 0 {
            debug!(
                "Removed {} breakpoints for {}",
                removed,
                file_path.display()
            );
        }

        removed
    }
}

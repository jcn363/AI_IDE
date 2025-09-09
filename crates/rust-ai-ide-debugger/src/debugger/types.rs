//! Core types used throughout the debugger

use serde::{Deserialize, Serialize};
use super::thread_debugging::ThreadDebuggerEvent;
use std::collections::HashMap;
use super::performance_profiling::PerformanceProfileEvent;
use super::memory_profiling::MemoryProfileEvent;
use super::thread_debugging::ThreadDebuggerEvent;

use std::path::PathBuf;

/// Represents a breakpoint in the debugger
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BreakpointInfo {
    /// Unique identifier for the breakpoint
    pub id: u32,
    /// Source file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,
    /// Line number in the source file
    pub line: u32,
    /// Optional condition expression
    pub condition: Option<String>,
    /// Whether the breakpoint is currently enabled
    pub enabled: bool,
    /// Number of times the breakpoint has been hit
    pub hit_count: u32,
    /// Optional log message to display when hit (for tracepoints)
    pub log_message: Option<String>,
}

/// Represents a variable in the debugger
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VariableInfo {
    /// Optional unique identifier for the variable (used for tracking)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    /// Variable name
    pub name: String,
    /// String representation of the variable's value
    pub value: String,
    /// Type name of the variable
    pub type_name: String,
    /// Whether the variable is currently in scope
    pub in_scope: bool,
    /// Child variables (for structs, enums, etc.)
    pub children: Vec<VariableInfo>,
    /// Optional expression used to evaluate this variable (for watch expressions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression: Option<String>,
}

/// Represents a stack frame in the debugger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// Frame ID
    pub id: u32,
    /// Function name
    pub function: String,
    /// Source file path
    pub file: String,
    /// Line number in the source file
    pub line: u32,
    /// Column number (if available)
    pub column: Option<u32>,
    /// Function arguments
    pub args: Vec<VariableInfo>,
    /// Local variables
    pub locals: Vec<VariableInfo>,
    /// Timestamp when the frame was created
    #[serde(default)]
    pub created_at: Option<u64>,
    /// Timestamp when the frame exited (if it has exited)
    #[serde(default)]
    pub exited_at: Option<u64>,
    /// Interaction history for this frame (new for enhanced visualization)
    #[serde(default)]
    pub interaction_history: Vec<FrameInteraction>,
    /// Timeline of variable changes (new for enhanced tracking)
    #[serde(default)]
    pub variable_timeline: Vec<VariableTimelineEntry>,
}

/// Interaction with a stack frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameInteraction {
    /// Timestamp of interaction
    pub timestamp: u64,
    /// Type of interaction
    pub interaction_type: FrameInteractionType,
    /// Description of the interaction
    pub description: String,
    /// Details about the interaction
    pub details: String,
}

/// Types of frame interactions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FrameInteractionType {
    /// Frame was entered
    Entered,
    /// Frame exited normally
    Exited,
    /// An exception occurred in the frame
    Exception,
    /// Variables were modified
    VariableModified,
    /// Breakpoint hit in frame
    BreakpointHit,
    /// Step operation performed
    StepPerformed,
}

/// Timeline entry for variable changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableTimelineEntry {
    /// Timestamp of change
    pub timestamp: u64,
    /// Variable name
    pub variable_name: String,
    /// New value
    pub new_value: String,
    /// Old value (if available)
    pub old_value: Option<String>,
    /// Source of change
    pub change_source: VariableChangeSource,
}

/// Source of a variable value change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VariableChangeSource {
    /// Assignment operation
    Assignment,
    /// Parameter passing
    ParameterPassing,
    /// Initialization
    Initialization,
    /// Function return
    ReturnValue,
    /// Memory write
    MemoryWrite,
    /// Other source
    Other(String),
}

/// Debugger state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebuggerState {
    /// Debugger is initializing
    Initializing {
        /// Target being debugged
        target: String,
    },
    /// Debugger is not attached to a target
    Disconnected,
    /// Target is running
    Running,
    /// Target is paused at a breakpoint
    Paused {
        /// Reason for pausing (breakpoint hit, step, etc.)
        reason: String,
        /// Current location (file, line)
        location: Option<(String, u32)>,
    },
    /// Target has terminated
    Terminated {
        /// Exit code if available
        exit_code: Option<i32>,
    },
    /// Error state
    Error {
        /// Error message
        message: String,
    },
}

/// Events emitted by the debugger to notify about state changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebuggerEvent {
    /// Debugger state has changed
    StateChanged(DebuggerState),
    /// A breakpoint was hit
    BreakpointHit {
        /// Information about the breakpoint that was hit
        breakpoint: BreakpointInfo,
        /// Current stack frame at the breakpoint
        stack_frame: StackFrame,
    },
    /// A breakpoint was added, removed, or modified
    BreakpointChanged(BreakpointInfo),
    /// Debugger output (stdout/stderr)
    OutputReceived(String),
    /// Debugger error message
    ErrorReceived(String),
    /// Variables have been updated
    VariablesUpdated(Vec<VariableInfo>),
    /// Call stack has changed
    CallStackUpdated(Vec<StackFrame>),
    /// Thread debugger event (new additions)
    ThreadDebugger(ThreadDebuggerEvent),
    /// Memory profiling event (new additions)
    MemoryProfile(MemoryProfileEvent),
    /// Performance profiling event (new additions)
    PerformanceProfile(PerformanceProfileEvent),
}

/// Debugger configuration
#[derive(Debug, Clone)]
pub struct DebuggerConfig {
    /// Path to the debugger executable (gdb/lldb)
    pub debugger_path: Option<String>,
    /// Target binary to debug
    pub target: String,
    /// Working directory for the debugger
    pub working_dir: Option<String>,
    /// Command line arguments for the target
    pub args: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Whether to break at the program entry point
    pub stop_at_entry: bool,
}

impl Default for DebuggerConfig {
    fn default() -> Self {
        Self {
            debugger_path: None,
            target: String::new(),
            working_dir: None,
            args: Vec::new(),
            env: HashMap::new(),
            stop_at_entry: true,
        }
    }

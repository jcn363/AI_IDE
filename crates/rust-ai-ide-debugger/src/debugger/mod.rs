//! Main debugger module that coordinates all debugger components
//!
//! This module provides the main [Debugger] struct which serves as the central
//! coordinator for all debugger functionality, including breakpoint management,
//! expression evaluation, and interaction with the debugger backend.

use log::{debug, error, info, trace, warn};
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, ChildStdin};
use tokio::sync::mpsc;

// Re-export submodules
pub mod backend;
pub mod breakpoints;
pub mod cache;
pub mod commands;
pub mod error;
pub mod event_loop;
pub mod execution;
pub mod expressions;
pub mod state;
pub mod thread_debugging;
pub mod memory_profiling;
pub mod performance_profiling;
pub mod types;
pub mod ui;
use crate::debugger::memory_profiling::{MemoryProfiler, Allocation, HeapStatistics, LeakClassification, HeapVisualization};
use crate::debugger::performance_profiling::PerformanceProfiler;
use crate::debugger::thread_debugging::{ThreadDebugger, ThreadState, DeadlockInfo};

pub use thread_debugging::ThreadDebugger;
pub use backend::DebuggerBackendTrait;
pub use performance_profiling::PerformanceProfiler;
pub use breakpoints::BreakpointManager;
pub use memory_profiling::MemoryProfiler;
pub use cache::DebuggerCache;
pub use commands::*;
pub use execution::DebuggerBackend;
pub use expressions::ExpressionManager;
pub use state::StateManager;
pub use types::*;

// Re-export error types
pub use error::{DebuggerError, Result as DebuggerResult};

// Re-export UI
pub use ui::DebuggerUI;

/// Main debugger structure that coordinates all debugger functionality
///
/// The [Debugger] struct serves as the main interface for interacting with the debugger.
/// It coordinates between the UI, debugger backend, and various managers to provide
/// a complete debugging experience.
///
/// # Example
/// ```no_run
/// use rust_ai_ide_debugger::debugger::{Debugger, DebuggerConfig};
/// use tokio::sync::mpsc;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let (event_tx, _) = mpsc::unbounded_channel();
/// let config = DebuggerConfig::default();
/// let mut debugger = Debugger::new();
/// debugger.start_session(config, event_tx).await?;
    /// Advanced thread debugger for async/await code
    thread_debugger: ThreadDebugger,
    /// Advanced memory profiler for comprehensive profiling
    /// Advanced performance profiler for CPU profiling, bottleneck detection, and flame graphs
    performance_profiler: PerformanceProfiler,
    memory_profiler: MemoryProfiler,
/// # Ok(())
/// # }
/// ```
pub struct Debugger {
    /// Debugger backend (GDB/LLDB)
    backend: DebuggerBackend,

    /// Breakpoint manager
    breakpoints: BreakpointManager,

    /// Expression manager for watch expressions
    expressions: ExpressionManager,

    /// State manager
    state: StateManager,

    /// Debugger process handle
    process: Option<Child>,

    /// Debugger process stdin
    stdin: Option<ChildStdin>,

    /// Channel for sending debugger events
    event_sender: Option<mpsc::UnboundedSender<DebuggerEvent>>,

    /// Next breakpoint ID
    next_breakpoint_id: u32,
    /// Debugger configuration
    config: DebuggerConfig,
    /// Call stack
    call_stack: Vec<StackFrame>,
    /// Current frame index
    current_frame: Option<usize>,
    /// Current variables
    variables: HashMap<String, VariableInfo>,
    /// Debugger output buffer
    output: Vec<String>,
}

impl Debugger {
    /// Create a new debugger instance with default configuration
    ///
    /// # Returns
    /// A new instance of the debugger with default settings
    pub fn new() -> Self {
        debug!("Creating new debugger instance");

        Self {
            backend: DebuggerBackend::default(),
            breakpoints: BreakpointManager::new(),
            expressions: ExpressionManager::new(),
            state: StateManager::new(),
            memory_profiler: MemoryProfiler::new(None), // Will be updated when event sender is set
            performance_profiler: PerformanceProfiler::new(None), // Will be updated when event sender is set
            thread_debugger: ThreadDebugger::new(None), // Will be updated when event sender is set
            process: None,
            stdin: None,
            event_sender: None,
            next_breakpoint_id: 1,
            config: DebuggerConfig::default(),
            call_stack: Vec::new(),
            current_frame: None,
            variables: HashMap::new(),
            output: Vec::new(),
        }
    }

    /// Start a new debugging session
    ///
    /// # Arguments
    /// * `config` - Configuration for the debugger session
    /// * `event_sender` - Channel for sending debugger events to the UI
    ///
    /// # Returns
    /// `Ok(())` if the session was started successfully
    ///
    /// # Errors
    /// Returns an error if the debugger backend fails to start or initialize
    pub async fn start_session(
        &mut self,
        config: DebuggerConfig,
        event_sender: mpsc::UnboundedSender<DebuggerEvent>,
    ) -> DebuggerResult<()> {
        info!("Starting new debugger session");

        // Validate configuration
        if config.target.is_empty() {
            return Err(DebuggerError::state_error("Target program cannot be empty"));
        }

        // Store configuration
        self.config = config;

        // Start the debugger backend
        self.backend.start(self.config.clone()).await.map_err(|e| {
            DebuggerError::process_error(format!("Failed to start debugger backend: {}", e))
        })?;

        // Store the event sender for later use
        self.event_sender = Some(event_sender.clone());

        // Update event senders for specialized profilers
        if let Some(ref mut md) = &mut self.memory_profiler { md.event_sender = Some(event_sender.clone()); }
        if let Some(ref mut pd) = &mut self.performance_profiler { pd.event_sender = Some(event_sender); }
        if let Some(ref mut td) = &mut self.thread_debugger { td.event_sender = Some(event_sender.clone()); }

        // Send initial state event
        self.send_event(DebuggerEvent::StateChanged(DebuggerState::Initializing {
            target: self.config.target.clone(),
        }))?;

        info!("Debugger session started successfully");
        Ok(())
    }

    /// Send a command to the debugger
    ///
    /// # Arguments
    /// * `command` - The command to send to the debugger backend
    ///
    /// # Returns
    /// `Ok(())` if the command was sent successfully
    ///
    /// # Errors
    /// Returns an error if the debugger is not running or the command fails
    pub async fn send_command(&mut self, command: &str) -> DebuggerResult<()> {
        trace!("Sending command to debugger: {}", command);

        match &mut self.stdin {
            Some(stdin) => {
                stdin.write_all(command.as_bytes()).await.map_err(|e| {
                    DebuggerError::process_error(format!(
                        "Failed to write command to debugger stdin: {}",
                        e
                    ))
                })?;

                stdin.write_all(b"\n").await.map_err(|e| {
                    DebuggerError::process_error(format!(
                        "Failed to write newline to debugger stdin: {}",
                        e
                    ))
                })?;

                trace!("Command sent successfully");
                Ok(())
            }
            None => {
                let err =
                    DebuggerError::state_error("Cannot send command - debugger is not running");
                error!("{}", err);
                Err(err)
            }
        }
    }

    /// Helper method to send events through the event channel
    pub fn send_event(&self, event: DebuggerEvent) -> DebuggerResult<()> {
        self.event_sender
            .as_ref()
            .ok_or_else(|| DebuggerError::state_error("Event sender not initialized"))?
            .send(event)
            .map_err(|e| DebuggerError::process_error(format!("Failed to send event: {}", e)))?;
        Ok(())
    }

    /// Run the debugged program
    ///
    /// # Returns
    /// `Ok(())` if the program was started successfully
    ///
    /// # Errors
    /// Returns an error if the program fails to start
    pub async fn run(&mut self) -> DebuggerResult<()> {
        info!("Starting program execution");
        if let Some(event) = self.state.set_state(DebuggerState::Running) {
            self.send_event(event)?;
        }
        self.send_command("run").await?;
        Ok(())
    }

    /// Continue execution until the next breakpoint
    ///
    /// # Returns
    /// `Ok(())` if execution was continued successfully
    ///
    /// # Errors
    /// Returns an error if the debugger is not in a valid state
    pub async fn continue_execution(&mut self) -> DebuggerResult<()> {
        info!("Continuing program execution");
        if let Some(event) = self.state.set_state(DebuggerState::Running) {
            self.send_event(event)?;
        }
        self.send_command("continue").await?;
        Ok(())
    }

    /// Step over the next source line
    ///
    /// # Returns
    /// `Ok(())` if the step was executed successfully
    ///
    /// # Errors
    /// Returns an error if the debugger is not in a valid state
    pub async fn step_over(&mut self) -> DebuggerResult<()> {
        info!("Stepping over next line");
        if let Some(event) = self.state.set_state(DebuggerState::Running) {
            self.send_event(event)?;
        }
        self.send_command("next").await?;
        Ok(())
    }

    /// Step into the next function call
    ///
    /// # Returns
    /// `Ok(())` if the step was executed successfully
    ///
    /// # Errors
    /// Returns an error if the debugger is not in a valid state
    pub async fn step_into(&mut self) -> DebuggerResult<()> {
        info!("Stepping into function");
        if let Some(event) = self.state.set_state(DebuggerState::Running) {
            self.send_event(event)?;
        }
        self.send_command("step").await?;
        Ok(())
    }

    /// Step out of the current function
    ///
    /// # Returns
    /// `Ok(())` if the step was executed successfully
    ///
    /// # Errors
    /// Returns an error if the debugger is not in a valid state
    pub async fn step_out(&mut self) -> DebuggerResult<()> {
        info!("Stepping out of current function");
        if let Some(event) = self.state.set_state(DebuggerState::Running) {
            self.send_event(event)?;
        }
        self.send_command("finish").await?;
        Ok(())
    }

    /// Pause the debugged program
    ///
    /// # Returns
    /// `Ok(())` if the program was paused successfully
    ///
    /// # Errors
    /// Returns an error if the program cannot be paused
    pub async fn pause(&mut self) -> DebuggerResult<()> {
        info!("Pausing program execution");

        match &self.process {
            Some(process) => {
                // Send SIGINT to pause the program
                if let Some(pid) = process.id() {
                    nix::sys::signal::kill(
                        nix::unistd::Pid::from_raw(pid as i32),
                        nix::sys::signal::Signal::SIGINT,
                    )
                    .map_err(|e| {
                        DebuggerError::process_error(format!(
                            "Failed to send SIGINT to process: {}",
                            e
                        ))
                    })?;

                    if let Some(event) = self.state.set_state(DebuggerState::Paused {
                        reason: "User requested pause".to_string(),
                        location: None,
                    }) {
                        self.send_event(event)?;
                    }

                    Ok(())
                } else {
                    let err = DebuggerError::process_error("Process ID not available");
                    error!("{}", err);
                    Err(err)
                }
            }
            None => {
                let err = DebuggerError::state_error("Cannot pause - no process is running");
                error!("{}", err);
                Err(err)
            }
        }
    }

    /// Stop the debugger and clean up all resources
    ///
    /// This will terminate the debugger process and clean up all internal state.
    /// After calling this method, the debugger will be in a clean state and can be reused.
    ///
    /// # Returns
    /// `Ok(())` if the debugger was stopped successfully
    ///
    /// # Errors
    /// Returns an error if there are issues during cleanup
    pub async fn stop(&mut self) -> DebuggerResult<()> {
        info!("Stopping debugger and cleaning up resources");

        // Send state changed event before cleaning up
        if let Some(event) = self.state.set_state(DebuggerState::Disconnected) {
            let _ = self.send_event(event);
        }

        // Stop the debugger process if it's running
        if let Some(mut process) = self.process.take() {
            // Try to gracefully terminate first
            if let Err(e) = process.kill().await {
                warn!("Failed to gracefully stop debugger process: {}", e);
                // Fall back to force kill if graceful termination fails
                if let Err(e) = process.start_kill() {
                    error!("Failed to force kill debugger process: {}", e);
                    return Err(DebuggerError::process_error(format!(
                        "Failed to stop debugger process: {}",
                        e
                    )));
                }
            }
        }

        // Clean up resources
        self.stdin = None;

        // Clear breakpoints and other resources
        self.breakpoints = BreakpointManager::new();
        self.expressions = ExpressionManager::new();

        // Reset state
        self.state = StateManager::new();

        // Clear other state
        self.next_breakpoint_id = 1;
        self.call_stack.clear();
        self.current_frame = None;
        self.variables.clear();
        self.output.clear();

        // Clear event sender
        self.event_sender = None;

        info!("Debugger stopped and resources cleaned up");
        Ok(())
    }
}

    /// Get access to the thread debugger for advanced async/await debugging
    pub fn thread_debugger(&self) -> &ThreadDebugger {
        &self.thread_debugger
    }

    /// Get mutable access to the thread debugger
    pub fn thread_debugger_mut(&mut self) -> &mut ThreadDebugger {
        &mut self.thread_debugger
    }

    /// Track a new thread in the thread debugger
    pub fn track_thread(&mut self, thread_id: u32, name: String) -> DebuggerResult<()> {
        self.thread_debugger.track_thread(thread_id, name)
            .map_err(|e| DebuggerError::process_error(format!("Failed to track thread: {}", e)))
    }

    /// Track an async task in the thread debugger
    pub fn track_async_task(&mut self, task_name: String, thread_id: Option<u32>) -> DebuggerResult<Option<u32>> {
        self.thread_debugger.track_async_task(task_name, thread_id)
            .map(Some)
            .map_err(|e| DebuggerError::process_error(format!("Failed to track async task: {}", e)))
    }

    /// Update thread state in the thread debugger
    pub fn update_thread_state_in_debugger(&mut self, thread_id: u32, new_state: ThreadState) -> DebuggerResult<()> {
        self.thread_debugger.update_thread_state(thread_id, new_state)
            .map_err(|e| DebuggerError::process_error(format!("Failed to update thread state: {}", e)))
    }

    /// Acquire a lock in the thread debugger
    pub fn acquire_lock_in_debugger(&mut self, thread_id: u32, lock_id: u64) -> DebuggerResult<()> {
        self.thread_debugger.acquire_lock(thread_id, lock_id)
            .map_err(|e| DebuggerError::process_error(format!("Failed to acquire lock: {}", e)))
    }

    /// Release a lock in the thread debugger
    pub fn release_lock_in_debugger(&mut self, thread_id: u32, lock_id: u64) -> DebuggerResult<()> {
        self.thread_debugger.release_lock(thread_id, lock_id)
            .map_err(|e| DebuggerError::process_error(format!("Failed to release lock: {}", e)))
    }

    /// Wait for a lock in the thread debugger
    pub fn wait_for_lock_in_debugger(&mut self, thread_id: u32, lock_id: u64) -> DebuggerResult<()> {
        self.thread_debugger.wait_for_lock(thread_id, lock_id)
            .map_err(|e| DebuggerError::process_error(format!("Failed to wait for lock: {}", e)))
    }

    /// Get async execution visualization data
    pub fn get_async_visualization(&self) -> String {
        self.thread_debugger.get_async_visualization_data()
    }

    /// Detect deadlocks using the thread debugger
    pub fn detect_deadlocks(&self) -> Vec<DeadlockInfo> {
        self.thread_debugger.detect_deadlocks()
    }
}
    /// Get access to the memory profiler for advanced memory analysis
    pub fn memory_profiler(&self) -> &MemoryProfiler {
        &self.memory_profiler
    }

    /// Get mutable access to the memory profiler
    pub fn memory_profiler_mut(&mut self) -> &mut MemoryProfiler {
        &mut self.memory_profiler
    }

    /// Track a memory allocation in the profiler
    pub fn track_allocation_in_profiler(&mut self, allocation: Allocation) -> DebuggerResult<()> {
        self.memory_profiler.track_allocation(allocation)
            .map_err(|e| DebuggerError::process_error(format!("Failed to track allocation: {}", e)))
    }

    /// Track a memory deallocation in the profiler
    pub fn track_deallocation_in_profiler(
        &mut self,
        address: usize,
        deallocated_at: u64,
        deallocation_stack: Vec<String>,
        thread_id: Option<u32>,
    ) -> DebuggerResult<()> {
        self.memory_profiler.track_deallocation(address, deallocated_at, deallocation_stack, thread_id)
            .map_err(|e| DebuggerError::process_error(format!("Failed to track deallocation: {}", e)))
    }

    /// Update heap statistics in the profiler
    pub fn update_heap_statistics(&mut self, stats: HeapStatistics) -> DebuggerResult<()> {
        self.memory_profiler.update_heap_statistics(stats)
            .map_err(|e| DebuggerError::process_error(format!("Failed to update heap statistics: {}", e)))
    }

    /// Analyze memory leaks using the profiler
    pub fn analyze_memory_leaks(&mut self) -> DebuggerResult<Vec<LeakClassification>> {
        self.memory_profiler.analyze_potential_leaks()
            .map_err(|e| DebuggerError::process_error(format!("Failed to analyze memory leaks: {}", e)))
    }

    /// Generate heap visualization data
    pub fn generate_heap_visualization(&self) -> HeapVisualization {
        self.memory_profiler.generate_heap_visualization()
    }

    /// Get current heap statistics
    pub fn get_heap_stats(&self) -> &HeapStatistics {
        self.memory_profiler.get_heap_statistics()
    /// Get access to the performance profiler for advanced CPU profiling
    pub fn performance_profiler(&self) -> &PerformanceProfiler {
        &self.performance_profiler
    }

    /// Get mutable access to the performance profiler
    pub fn performance_profiler_mut(&mut self) -> &mut PerformanceProfiler {
        &mut self.performance_profiler
    }

    /// Start performance profiling
    pub fn start_performance_profiling(&mut self) -> DebuggerResult<()> {
        self.performance_profiler.start_profiling()
            .map_err(|e| DebuggerError::process_error(format!("Failed to start performance profiling: {}", e)))
    }

    /// Stop performance profiling and analyze bottlenecks
    pub fn stop_performance_profiling(&mut self) -> DebuggerResult<BottleneckAnalysis> {
        use crate::debugger::performance_profiling::BottleneckAnalysis;
        self.performance_profiler.stop_profiling()
            .map_err(|e| DebuggerError::process_error(format!("Failed to stop performance profiling: {}", e)))
    }
}
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}

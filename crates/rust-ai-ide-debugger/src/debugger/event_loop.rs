//! Debugger event loop implementation
//!
//! This module contains the main event loop that processes debugger events,
//! commands, and state changes in an asynchronous manner. The event loop serves
//! as the central coordinator between the debugger backend and the UI, ensuring
//! that all operations are processed in a thread-safe way and that the UI remains
//! responsive.
//!
//! # Architecture
//!
//! The event loop runs in its own task and processes commands from a channel.
//! Commands are queued and processed in the order they are received. The event
//! loop also periodically checks for debugger state changes and updates the UI
//! accordingly.
//!
//! # Example
//! ```no_run
//! use rust_ai_ide_debugger::debugger::{
//!     Debugger, DebuggerCommand, DebuggerEvent, DebuggerConfig,
//!     event_loop::DebuggerEventLoop
//! };
//! use std::sync::Arc;
//! use tokio::sync::{mpsc, Mutex};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create channels for commands and events
//! let (command_sender, command_receiver) = mpsc::unbounded_channel();
//! let (event_sender, _) = mpsc::unbounded_channel();
//!
//! // Create and start the debugger event loop
//! let debugger = Arc::new(Mutex::new(Debugger::new()));
//! let _event_loop = DebuggerEventLoop::new(
//!     debugger,
//!     command_receiver,
//!     event_sender
//! );
//! # Ok(())
//! # }

use log::{debug, info, trace};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::time as tokio_time;

use crate::debugger::{
    Debugger, DebuggerError, DebuggerEvent, DebuggerResult, DebuggerState, VariableInfo,
};

/// The main debugger event loop.
///
/// This struct manages the main event processing loop for the debugger.
/// It handles incoming commands, processes debugger events, and updates the UI
/// with the current debugger state.
///
/// The event loop runs in a separate task and communicates with the rest of the
/// application through channels. Commands are sent to the event loop through
/// the command channel, and events are emitted through the event channel.
///
/// # Example
/// ```no_run
/// # use rust_ai_ide_debugger::debugger::{
/// #     Debugger, DebuggerCommand, DebuggerEvent,
/// #     event_loop::DebuggerEventLoop
/// # };
/// # use std::sync::Arc;
/// # use tokio::sync::{mpsc, Mutex};
/// #
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let (command_sender, command_receiver) = mpsc::unbounded_channel();
/// let (event_sender, _) = mpsc::unbounded_channel();
/// let debugger = Arc::new(Mutex::new(Debugger::new()));
///
/// let event_loop = DebuggerEventLoop::new(
///     debugger,
///     command_receiver,
///     event_sender
/// );
/// # Ok(())
/// # }
/// ```
pub struct DebuggerEventLoop {
    /// The debugger instance
    debugger: Arc<Mutex<Debugger>>,
    /// Channel for receiving commands
    command_receiver: mpsc::UnboundedReceiver<DebuggerCommand>,
    /// Channel for sending events to the UI
    event_sender: mpsc::UnboundedSender<DebuggerEvent>,
    /// Queue of pending commands
    command_queue: VecDeque<DebuggerCommand>,
    /// Whether the event loop is currently running
    running: bool,
}

/// Commands that can be sent to the debugger event loop.
///
/// These commands represent all the operations that can be performed on the debugger.
/// Each command is processed asynchronously by the event loop.
#[derive(Debug, Clone)]
pub enum DebuggerCommand {
    /// Start a new debug session
    StartSession {
        /// Debugger configuration
        config: super::DebuggerConfig,
    },
    /// Send a command to the debugger
    SendCommand(String),
    /// Run the debugged program
    Run,
    /// Continue execution
    Continue,
    /// Step over the next source line
    StepOver,
    /// Step into the next function call
    StepInto,
    /// Step out of the current function
    StepOut,
    /// Pause the debugged program
    Pause,
    /// Stop the debugger and clean up
    Stop,
    /// Evaluate an expression
    EvaluateExpression(String),
    /// Set a variable value
    SetVariable {
        /// The name of the variable to set
        name: String,
        /// The value to assign to the variable
        value: String,
    },
    /// Add a breakpoint
    AddBreakpoint {
        /// The source file path
        file: String,
        /// The line number in the source file
        line: u32,
    },
    /// Remove a breakpoint
    RemoveBreakpoint(u32),
    /// Toggle a breakpoint
    ToggleBreakpoint(u32),
    /// Add a watch expression
    AddWatchExpression(String),
    /// Remove a watch expression
    RemoveWatchExpression(u32),
    /// Select a stack frame
    SelectFrame(u32),
}

impl DebuggerEventLoop {
    /// Create a new debugger event loop.
    ///
    /// # Arguments
    /// * `debugger` - A thread-safe reference to the debugger instance
    /// * `command_receiver` - Channel for receiving commands from the UI
    /// * `event_sender` - Channel for sending events to the UI
    ///
    /// # Returns
    /// A new `DebuggerEventLoop` instance ready to be started with `run()`
    ///
    /// # Example
    /// ```no_run
    /// # use rust_ai_ide_debugger::debugger::{
    /// #     Debugger, DebuggerCommand, DebuggerEvent,
    /// #     event_loop::DebuggerEventLoop
    /// # };
    /// # use std::sync::Arc;
    /// # use tokio::sync::{mpsc, Mutex};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let (command_sender, command_receiver) = mpsc::unbounded_channel();
    /// let (event_sender, _) = mpsc::unbounded_channel();
    /// let debugger = Arc::new(Mutex::new(Debugger::new()));
    ///
    /// let event_loop = DebuggerEventLoop::new(
    ///     debugger,
    ///     command_receiver,
    ///     event_sender
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        debugger: Arc<Mutex<Debugger>>,
        command_receiver: mpsc::UnboundedReceiver<DebuggerCommand>,
        event_sender: mpsc::UnboundedSender<DebuggerEvent>,
    ) -> Self {
        Self {
            debugger,
            command_receiver,
            event_sender,
            command_queue: VecDeque::new(),
            running: false,
        }
    }

    /// Start the event loop.
    ///
    /// This method will run the main event processing loop until the debugger is stopped.
    /// It processes commands from the command channel and handles debugger events.
    ///
    /// # Returns
    /// `Ok(())` if the event loop exits normally, or an error if something goes wrong.
    ///
    /// # Errors
    /// Returns an error if the event loop is already running or if a fatal error occurs.
    ///
    /// # Example
    /// ```no_run
    /// # use rust_ai_ide_debugger::debugger::{
    /// #     Debugger, DebuggerCommand, DebuggerEvent,
    /// #     event_loop::DebuggerEventLoop
    /// # };
    /// # use std::sync::Arc;
    /// # use tokio::sync::{mpsc, Mutex};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let (command_sender, command_receiver) = mpsc::unbounded_channel();
    /// # let (event_sender, _) = mpsc::unbounded_channel();
    /// # let debugger = Arc::new(Mutex::new(Debugger::new()));
    /// #
    /// let event_loop = DebuggerEventLoop::new(
    ///     debugger,
    ///     command_receiver,
    ///     event_sender
    /// );
    ///
    /// // Run the event loop in a separate task
    /// tokio::spawn(async move {
    ///     if let Err(e) = event_loop.run().await {
    ///         eprintln!("Debugger event loop error: {}", e);
    ///     }
    /// });
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(mut self) -> DebuggerResult<()> {
        if self.running {
            return Err(DebuggerError::state_error("Debugger is not running"));
        }

        self.running = true;
        info!("Debugger event loop started");

        // Main event loop
        while self.running {
            // Process all available commands
            self.process_commands().await?;

            // Process debugger events if no commands are pending
            if self.command_queue.is_empty() {
                self.process_debugger_events().await?;
            }

            // Small delay to prevent busy waiting
            tokio_time::sleep(Duration::from_millis(10)).await;
        }

        info!("Debugger event loop stopped");
        Ok(())
    }

    /// Process all available commands
    async fn process_commands(&mut self) -> DebuggerResult<()> {
        // Process all available commands
        while let Some(command) = self.command_receiver.recv().await {
            self.handle_command(command).await?;

            // Update debugger state after each command
            let mut debugger = self.debugger.lock().await;
            self.check_state_changes(&mut debugger).await?;
        }

        Ok(())
    }

    /// Process debugger events from the debugger backend
    async fn process_debugger_events(&mut self) -> DebuggerResult<()> {
        let mut debugger = self.debugger.lock().await;

        // Process any pending output from the debugger
        self.process_debugger_output(&mut debugger).await?;

        // Check for state changes
        self.check_state_changes(&mut debugger).await?;

        // Check for breakpoint hits
        self.check_breakpoint_hits(&mut debugger).await?;

        // Update variables and stack
        self.update_variables_and_stack(&mut debugger).await?;

        Ok(())
    }

    /// Process output from the debugger backend
    async fn process_debugger_output(&self, _debugger: &mut Debugger) -> DebuggerResult<()> {
        // TODO: Implement actual output processing from the debugger backend
        Ok(())
    }

    /// Check for and handle debugger state changes
    async fn check_state_changes(&self, debugger: &mut Debugger) -> DebuggerResult<()> {
        let current_state = debugger.state.get_state().clone();

        // Always send state change events for now
        debug!("Debugger state: {:?}", current_state);

        // Notify UI of state change
        let _ = self
            .event_sender
            .send(DebuggerEvent::StateChanged(current_state));

        // Update variables and stack if we're paused
        if matches!(debugger.state.get_state(), DebuggerState::Paused { .. }) {
            self.update_variables_and_stack(debugger).await?;
        }

        Ok(())
    }

    /// Check for breakpoint hits and handle them
    async fn check_breakpoint_hits(&self, debugger: &mut Debugger) -> DebuggerResult<()> {
        // Check if we're in a paused state
        if let DebuggerState::Paused { .. } = debugger.state.get_state() {
            // TODO: Implement breakpoint hit detection
            // This should check if we've hit a breakpoint and send appropriate events
        }
        Ok(())
    }

    /// Update variables and call stack for the current frame
    async fn update_variables_and_stack(&self, debugger: &mut Debugger) -> DebuggerResult<()> {
        // Only update if we're in a paused state
        if !matches!(debugger.state.get_state(), DebuggerState::Paused { .. }) {
            return Ok(());
        }

        // Clone expressions to avoid borrow issues
        let expressions: Vec<(u32, String)> = debugger
            .expressions
            .get_watch_expressions()
            .into_iter()
            .filter_map(|e| {
                if let (Some(id), Some(expr_str)) = (e.id, e.expression.as_ref()) {
                    Some((id, expr_str.clone()))
                } else {
                    None
                }
            })
            .collect();

        // Evaluate each expression and update its value
        for (id, expr_str) in expressions {
            match debugger
                .expressions
                .evaluate_expression(&expr_str, debugger.state.get_state())
            {
                Ok(updated_var) => {
                    // Update the expression value in the manager
                    let _ = debugger
                        .expressions
                        .update_watch_expression_by_id(id, updated_var);
                }
                Err(e) => {
                    debug!("Failed to evaluate expression '{}': {}", expr_str, e);
                }
            }
        }

        // Get current stack trace (placeholder for now)
        let stack = Vec::new();

        // Get variables for the current frame (placeholder for now)
        let mut variables = Vec::new();

        // Add watch expressions to variables
        let watch_vars = debugger.expressions.get_watch_expressions();
        variables.extend(watch_vars.into_iter().cloned());

        // Notify UI about the updated variables and stack
        let _ = self
            .event_sender
            .send(DebuggerEvent::VariablesUpdated(variables));
        let _ = self
            .event_sender
            .send(DebuggerEvent::CallStackUpdated(stack));

        Ok(())
    }

    /// Handle a single command
    async fn handle_command(&mut self, command: DebuggerCommand) -> DebuggerResult<()> {
        trace!("Processing command: {:?}", command);

        match command {
            DebuggerCommand::StartSession { config } => {
                info!("Starting debug session with config: {:?}", config);

                // Initialize debugger state
                let mut debugger = self.debugger.lock().await;

                // Start the debugger backend
                debugger
                    .start_session(config, self.event_sender.clone())
                    .await
                    .map_err(|e| DebuggerError::process_error(e.to_string()))?;

                info!("Debug session started successfully");
            }
            DebuggerCommand::SendCommand(cmd) => {
                trace!("Sending command to debugger: {}", cmd);
                let mut debugger = self.debugger.lock().await;
                debugger.send_command(&cmd).await?;
            }
            DebuggerCommand::Run => {
                info!("Running program");
                let mut debugger = self.debugger.lock().await;
                debugger.run().await?;
            }
            DebuggerCommand::Continue => {
                info!("Continuing execution");
                let mut debugger = self.debugger.lock().await;
                debugger.continue_execution().await?;
            }
            DebuggerCommand::StepOver => {
                info!("Stepping over");
                let mut debugger = self.debugger.lock().await;
                debugger.step_over().await?;
            }
            DebuggerCommand::StepInto => {
                info!("Stepping into");
                let mut debugger = self.debugger.lock().await;
                debugger.step_into().await?;
            }
            DebuggerCommand::StepOut => {
                info!("Stepping out");
                let mut debugger = self.debugger.lock().await;
                debugger.step_out().await?;
            }
            DebuggerCommand::Pause => {
                info!("Pausing execution");
                let mut debugger = self.debugger.lock().await;
                debugger.pause().await?;
            }
            DebuggerCommand::Stop => {
                info!("Stopping debugger");
                let mut debugger = self.debugger.lock().await;
                debugger.stop().await?;
                self.running = false;
            }
            DebuggerCommand::EvaluateExpression(expr) => {
                trace!("Evaluating expression: {}", expr);
                let debugger = self.debugger.lock().await;
                let result = debugger
                    .expressions
                    .evaluate_expression(&expr, debugger.state.get_state())?;
                let _ = self
                    .event_sender
                    .send(DebuggerEvent::OutputReceived(result.value));
            }
            DebuggerCommand::SetVariable { name, value } => {
                trace!("Setting variable {} = {}", name, value);
                // TODO: Implement variable setting in the debugger
                debug!(
                    "Setting variable {} to {} is not yet implemented",
                    name, value
                );
            }
            DebuggerCommand::AddBreakpoint { file, line } => {
                info!("Adding breakpoint at {}:{}", file, line);
                let mut debugger = self.debugger.lock().await;
                let bp_id = debugger
                    .breakpoints
                    .add_breakpoint(&file, line, None, None)?;
                let bp = debugger.breakpoints.get_breakpoint(bp_id)?.clone();
                info!("Breakpoint added: {:?}", bp);
                // Notify UI about the new breakpoint
                let _ = self.event_sender.send(DebuggerEvent::BreakpointChanged(bp));
            }
            DebuggerCommand::RemoveBreakpoint(id) => {
                info!("Removing breakpoint {}", id);
                let mut debugger = self.debugger.lock().await;
                let bp = debugger.breakpoints.remove_breakpoint(id)?;
                // Notify UI about the removed breakpoint
                let _ = self.event_sender.send(DebuggerEvent::BreakpointChanged(bp));
            }
            DebuggerCommand::ToggleBreakpoint(id) => {
                info!("Toggling breakpoint {}", id);
                let mut debugger = self.debugger.lock().await;
                let new_state = debugger.breakpoints.toggle_breakpoint(id)?;
                // Get the updated breakpoint to send in the event
                let bp = debugger.breakpoints.get_breakpoint(id)?.clone();
                info!(
                    "Breakpoint {} toggled to {}",
                    id,
                    if new_state { "enabled" } else { "disabled" }
                );
                // Notify UI about the updated breakpoint
                let _ = self.event_sender.send(DebuggerEvent::BreakpointChanged(bp));
            }
            DebuggerCommand::AddWatchExpression(expr) => {
                info!("Adding watch expression: {}", expr);
                let mut debugger = self.debugger.lock().await;
                let id = debugger.expressions.get_next_id();
                let var_info = VariableInfo {
                    id: Some(id),
                    name: expr.clone(),
                    value: "".to_string(),
                    type_name: "unknown".to_string(),
                    in_scope: true,
                    children: Vec::new(),
                    expression: Some(expr.clone()),
                };
                let id = debugger.expressions.add_watch_expression(expr)?;
                info!("Watch expression added with id: {}", id);
                // Notify UI about the new watch expression
                let _ = self
                    .event_sender
                    .send(DebuggerEvent::VariablesUpdated(vec![var_info]));
            }
            DebuggerCommand::RemoveWatchExpression(id) => {
                info!("Removing watch expression {}", id);
                let mut debugger = self.debugger.lock().await;
                // First get the expression text to find it in the map
                let expr = debugger.expressions.find_expression_by_id(id)?;
                if let Some(_expr_info) = debugger.expressions.remove_watch_expression(&expr) {
                    // Notify UI about the removed watch expression
                    let _ = self
                        .event_sender
                        .send(DebuggerEvent::VariablesUpdated(Vec::new()));
                }
            }
            DebuggerCommand::SelectFrame(frame_id) => {
                info!("Selecting frame {}", frame_id);
                let mut debugger = self.debugger.lock().await;
                if let Some(event) = debugger.state.select_frame(frame_id as usize) {
                    let _ = self.event_sender.send(event);
                    // After changing frames, update variables for the new frame
                    let _ = self.update_variables_and_stack(&mut debugger).await;
                }
            }
        }

        Ok(())
    }
}

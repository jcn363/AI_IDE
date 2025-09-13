use rust_ai_ide_debugger::debugger::{
    Debugger, DebuggerBackendTrait, DebuggerCommand, DebuggerConfig, DebuggerError, DebuggerEvent,
    DebuggerState,
};
use std::time::Duration;
use tokio::sync::mpsc;

// Mock debugger backend for testing
pub struct MockDebuggerBackend {
    sender: mpsc::UnboundedSender<()>,
    running: bool,
}

impl MockDebuggerBackend {
    pub fn new(sender: mpsc::UnboundedSender<()>) -> Self {
        Self {
            sender,
            running: false,
        }
    }
}

#[async_trait::async_trait]
impl DebuggerBackendTrait for MockDebuggerBackend {
    fn set_event_sender(
        &mut self,
        _sender: tokio::sync::mpsc::UnboundedSender<crate::debugger::DebuggerEvent>,
    ) {
        // Store the sender if needed for testing
    }

    async fn start(&mut self, _config: &DebuggerConfig) -> Result<(), DebuggerError> {
        self.running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), DebuggerError> {
        self.running = false;
        Ok(())
    }

    async fn add_breakpoint(&mut self, _file: &str, _line: u32) -> Result<u32, DebuggerError> {
        Ok(1)
    }

    async fn remove_breakpoint(&mut self, _id: u32) -> Result<(), DebuggerError> {
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    async fn step_over(&mut self) -> Result<(), DebuggerError> {
        Ok(())
    }

    async fn step_into(&mut self) -> Result<(), DebuggerError> {
        Ok(())
    }

    async fn step_out(&mut self) -> Result<(), DebuggerError> {
        Ok(())
    }

    async fn continue_execution(&mut self) -> Result<(), DebuggerError> {
        Ok(())
    }

    async fn pause(&mut self) -> Result<(), DebuggerError> {
        Ok(())
    }

    async fn get_stack_trace(
        &self,
    ) -> Result<Vec<crate::debugger::types::StackFrame>, DebuggerError> {
        Ok(Vec::new())
    }

    async fn get_variables(
        &self,
        _frame_id: Option<u32>,
    ) -> Result<Vec<crate::debugger::types::VariableInfo>, DebuggerError> {
        Ok(Vec::new())
    }

    async fn evaluate_expression(
        &self,
        _expression: &str,
        _frame_id: Option<u32>,
    ) -> Result<crate::debugger::types::VariableInfo, DebuggerError> {
        Err(DebuggerError::NotImplemented)
    }
}

// Helper function to create a test debugger instance
async fn create_test_debugger() -> (
    std::sync::Arc<tokio::sync::Mutex<Debugger>>,
    mpsc::UnboundedReceiver<DebuggerEvent>,
) {
    let debugger = std::sync::Arc::new(tokio::sync::Mutex::new(Debugger::new()));
    let (event_sender, event_receiver) = mpsc::unbounded_channel();

    // Set up the debugger with a mock backend
    let (backend_sender, _) = mpsc::unbounded_channel();
    let mock_backend = Box::new(MockDebuggerBackend::new(backend_sender));

    let mut debugger_guard = debugger.lock().await;
    debugger_guard.state.set_state(DebuggerState::Running);
    debugger_guard.backend = Some(Box::new(mock_backend));
    debugger_guard.event_sender = Some(event_sender);

    (debugger, event_receiver)
}

#[tokio::test]
async fn test_breakpoint_commands() {
    let (debugger, _) = create_test_debugger().await;

    // Create the event loop with a new event channel
    let (command_sender, command_receiver) = mpsc::unbounded_channel();
    let (event_sender, mut event_receiver) = mpsc::unbounded_channel();

    // Create the event loop
    let event_loop = crate::debugger::event_loop::DebuggerEventLoop::new(
        debugger.clone(),
        command_receiver,
        event_sender,
    );

    // Start the event loop in a background task
    let handle = tokio::spawn(async move {
        let _ = event_loop.run().await;
    });

    // Wait for the event loop to start
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Test adding a breakpoint
    command_sender
        .send(DebuggerCommand::AddBreakpoint {
            file: "main.rs".to_string(),
            line: 42,
        })
        .expect("Failed to add breakpoint");

    // Wait for the breakpoint to be added
    let _ = tokio::time::timeout(Duration::from_millis(100), event_receiver.recv()).await;

    // Test removing a breakpoint
    command_sender
        .send(DebuggerCommand::RemoveBreakpoint(1))
        .expect("Failed to remove breakpoint");

    // Clean up
    command_sender.send(DebuggerCommand::Stop).ok();

    // Wait for the event loop to finish with a timeout
    let _ = tokio::time::timeout(Duration::from_secs(1), handle).await;
}

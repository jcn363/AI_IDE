use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, mpsc, oneshot};
use std::collections::{HashMap, VecDeque};
use rust_ai_ide_common::{IDEError, IDEErrorKind};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::time::{timeout, Duration};
use std::process::{Stdio, Command};
use serde::{Deserialize, Serialize};
use regex::Regex;

/// MI Protocol adapter for communicating with GDB/LLDB via Machine Interface commands
pub struct MIProtocolAdapter {
    pub(crate) process: Arc<Mutex<Option<tokio::process::Child>>>,
    pub(crate) reader: Arc<Mutex<Option<BufReader<tokio::process::ChildStdout>>>>,
    pub(crate) writer: Arc<Mutex<Option<tokio::process::ChildStdin>>>>,
    pub(crate) command_queue: Arc<Mutex<VecDeque<MICommand>>>,
    pub(crate) response_handlers: Arc<Mutex<HashMap<String, oneshot::Sender<MIResponse>>>>,
    pub(crate) event_stream: mpsc::UnboundedSender<DebugEvent>,
    pub(crate) next_token: Arc<Mutex<u64>>,
    pub(crate) async_state: Arc<Mutex<AdapterState>>,
}

impl MIProtocolAdapter {
    pub fn new(event_sender: mpsc::UnboundedSender<DebugEvent>) -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            reader: Arc::new(Mutex::new(None)),
            writer: Arc::new(Mutex::new(None)),
            command_queue: Arc::new(Mutex::new(VecDeque::new())),
            response_handlers: Arc::new(Mutex::new(HashMap::new())),
            event_stream: event_sender,
            next_token: Arc::new(Mutex::new(1)),
            async_state: Arc::new(Mutex::new(AdapterState::Disconnected)),
        }
    }

    pub async fn start_debugger(&self, debugger_path: &str, args: &[&str]) -> Result<(), IDEError> {
        let mut child = Command::new(debugger_path)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| IDEError::new(IDEErrorKind::ProcessError, "Failed to start debugger")
                .with_source(e))?;

        let stdout = child.stdout.take().ok_or_else(|| {
            IDEError::new(IDEErrorKind::ProcessError, "Failed to get debugger stdout")
        })?;

        let stdin = child.stdin.take().ok_or_else(|| {
            IDEError::new(IDEErrorKind::ProcessError, "Failed to get debugger stdin")
        })?;

        *self.process.lock().await = Some(child);
        *self.reader.lock().await = Some(BufReader::new(stdout));
        *self.writer.lock().await = Some(stdin);
        *self.async_state.lock().await = AdapterState::Connected;

        // Start the response reader
        let reader_clone = Arc::clone(&self.reader);
        let response_handlers_clone = Arc::clone(&self.response_handlers);
        let event_stream_clone = self.event_stream.clone();

        tokio::spawn(async move {
            Self::response_reader_loop(reader_clone, response_handlers_clone, event_stream_clone).await;
        });

        Ok(())
    }

    pub async fn send_command(&self, command: MICommand) -> Result<MIResponse, IDEError> {
        let token = self.get_next_token().await;
        let tokenized_command = MICommandWithToken { token, command };

        let (tx, rx) = oneshot::channel();

        {
            let mut handlers = self.response_handlers.lock().await;
            handlers.insert(token.to_string(), tx);
        }

        {
            let mut queue = self.command_queue.lock().await;
            queue.push_back(tokenized_command);
        }

        self.process_command_queue().await?;

        let response = timeout(Duration::from_secs(30), rx).await
            .map_err(|_| IDEError::new(IDEErrorKind::Timeout, "MI command timeout"))?
            .map_err(|_| IDEError::new(IDEErrorKind::CommunicationError, "Response channel closed"))?;

        Ok(response)
    }

    async fn process_command_queue(&self) -> Result<(), IDEError> {
        let mut writer = self.writer.lock().await;
        let writer = writer.as_mut().ok_or_else(|| {
            IDEError::new(IDEErrorKind::StateError, "Debugger writer not available")
        })?;

        let mut queue = self.command_queue.lock().await;

        while let Some(command) = queue.front() {
            let command_str = format!("{}-cmd\n", command.to_string());
            writer.write_all(command_str.as_bytes()).await.map_err(|e| {
                IDEError::new(IDEErrorKind::CommunicationError, "Failed to send MI command")
                    .with_source(e)
            })?;
            writer.flush().await.map_err(|e| {
                IDEError::new(IDEErrorKind::CommunicationError, "Failed to flush MI command")
                    .with_source(e)
            })?;
            queue.pop_front();
        }

        Ok(())
    }

    async fn response_reader_loop(
        reader: Arc<Mutex<Option<BufReader<tokio::process::ChildStdout>>>>,
        handlers: Arc<Mutex<HashMap<String, oneshot::Sender<MIResponse>>>>,
        event_sender: mpsc::UnboundedSender<DebugEvent>,
    ) {
        let mut lines = Vec::new();
        let mut reader_guard = reader.lock().await;
        let reader = match reader_guard.as_mut() {
            Some(r) => r,
            None => return,
        };

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    lines.push(line.trim().to_string());

                    if line.contains("done") || line.contains("error") || line.contains("running") {
                        if let Some(response) = Self::parse_response(&lines) {
                            // Handle response
                            let mut handlers_guard = handlers.lock().await;
                            if let Some(handler) = handlers_guard.remove(&response.token) {
                                let _ = handler.send(response);
                            }
                        }
                        lines.clear();
                    } else if line.contains("*stopped") || line.contains("*running") {
                        if let Some(event) = Self::parse_event(&line) {
                            let _ = event_sender.send(event);
                        }
                    }
                }
                Err(_) => break,
            }
        }
    }

    fn parse_response(lines: &[String]) -> Option<MIResponse> {
        if lines.is_empty() {
            return None;
        }

        let response_regex = Regex::new(r"^(\d+)\^(done|error|running)").unwrap();

        for line in lines {
            if let Some(captures) = response_regex.captures(line) {
                let token = captures.get(1)?.as_str().to_string();
                let result_class = captures.get(2)?.as_str();

                return Some(MIResponse {
                    token,
                    result_class: result_class.to_string(),
                    results: vec![], // Would parse additional results
                });
            }
        }

        None
    }

    fn parse_event(line: &str) -> Option<DebugEvent> {
        if line.contains("*stopped") {
            Some(DebugEvent::BreakpointHit {
                thread_id: 0,
                file: "unknown".to_string(),
                line: 0,
            })
        } else if line.contains("*running") {
            Some(DebugEvent::TargetRunning)
        } else {
            None
        }
    }

    async fn get_next_token(&self) -> String {
        let mut token = self.next_token.lock().await;
        let current = *token;
        *token += 1;
        current.to_string()
    }

    pub async fn set_breakpoint(&self, file: &str, line: u32) -> Result<BreakpointInfo, IDEError> {
        let command = MICommand::BreakInsert { file: file.to_string(), line };
        let response = self.send_command(command).await?;

        Ok(BreakpointInfo {
            id: 1, // Would parse from response
            file: file.to_string(),
            line,
            enabled: true,
        })
    }

    pub async fn remove_breakpoint(&self, breakpoint_id: u32) -> Result<(), IDEError> {
        let command = MICommand::BreakDelete { breakpoint: breakpoint_id };
        self.send_command(command).await?;
        Ok(())
    }

    pub async fn step_over(&self) -> Result<(), IDEError> {
        let command = MICommand::ExecNext { thread: None };
        self.send_command(command).await?;
        Ok(())
    }

    pub async fn step_into(&self) -> Result<(), IDEError> {
        let command = MICommand::ExecStep { thread: None };
        self.send_command(command).await?;
        Ok(())
    }

    pub async fn step_out(&self) -> Result<(), IDEError> {
        let command = MICommand::ExecFinish { thread: None };
        self.send_command(command).await?;
        Ok(())
    }

    pub async fn continue_execution(&self) -> Result<(), IDEError> {
        let command = MICommand::ExecContinue { all_threads: true };
        self.send_command(command).await?;
        Ok(())
    }

    pub async fn get_stack_trace(&self, thread_id: u32) -> Result<Vec<StackFrame>, IDEError> {
        let command = MICommand::StackListFrames { thread: thread_id };
        let _response = self.send_command(command).await?;

        // Parse stack frames from response - simplified
        Ok(vec![StackFrame {
            level: 0,
            address: String::new(),
            function: "unknown".to_string(),
            file: "unknown".to_string(),
            line: 0,
        }])
    }

    pub async fn get_variable_value(&self, variable: &str) -> Result<String, IDEError> {
        let command = MICommand::DataEvaluateExpression { expression: variable.to_string() };
        let response = self.send_command(command).await?;

        // Parse value from response - simplified
        Ok("unknown".to_string())
    }

    pub async fn disconnect(&self) -> Result<(), IDEError> {
        let command = MICommand::GDBExit;
        let _ = self.send_command(command).await;

        let mut process = self.process.lock().await;
        if let Some(child) = process.as_mut() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }

        *process = None;
        *self.reader.lock().await = None;
        *self.writer.lock().await = None;
        *self.async_state.lock().await = AdapterState::Disconnected;

        Ok(())
    }
}

/// Async debug session that provides non-blocking debug operations
pub struct AsyncDebugSession {
    pub(crate) adapter: Arc<MIProtocolAdapter>,
    pub(crate) breakpoints: Arc<RwLock<HashMap<u32, BreakpointInfo>>>,
    pub(crate) variables: Arc<RwLock<HashMap<String, VariableInfo>>>,
    pub(crate) current_thread: Arc<Mutex<Option<u32>>>,
    pub(crate) session_state: Arc<Mutex<DebugState>>,
}

impl AsyncDebugSession {
    pub fn new(adapter: Arc<MIProtocolAdapter>) -> Self {
        Self {
            adapter,
            breakpoints: Arc::new(RwLock::new(HashMap::new())),
            variables: Arc::new(RwLock::new(HashMap::new())),
            current_thread: Arc::new(Mutex::new(None)),
            session_state: Arc::new(Mutex::new(DebugState::NotStarted)),
        }
    }

    pub async fn start_session(&self, executable: &str, args: &[&str]) -> Result<(), IDEError> {
        let args_with_executable = &[executable].iter().chain(args.iter()).cloned().collect::<Vec<_>>();
        self.adapter.start_debugger("gdb", &args_with_executable).await?;
        *self.session_state.lock().await = DebugState::Running;
        Ok(())
    }

    pub async fn add_breakpoint_async(&self, file: &str, line: u32) -> Result<u32, IDEError> {
        let breakpoint_info = self.adapter.set_breakpoint(file, line).await?;
        let id = breakpoint_info.id;

        let mut breakpoints = self.breakpoints.write().await;
        breakpoints.insert(id, breakpoint_info);

        Ok(id)
    }

    pub async fn remove_breakpoint_async(&self, breakpoint_id: u32) -> Result<(), IDEError> {
        self.adapter.remove_breakpoint(breakpoint_id).await?;

        let mut breakpoints = self.breakpoints.write().await;
        breakpoints.remove(&breakpoint_id);

        Ok(())
    }

    pub async fn get_breakpoints_async(&self) -> HashMap<u32, BreakpointInfo> {
        let breakpoints = self.breakpoints.read().await;
        breakpoints.clone()
    }

    pub async fn step_over_async(&self) -> Result<(), IDEError> {
        self.adapter.step_over().await?;
        *self.session_state.lock().await = DebugState::Stepping;
        Ok(())
    }

    pub async fn step_into_async(&self) -> Result<(), IDEError> {
        self.adapter.step_into().await?;
        *self.session_state.lock().await = DebugState::Stepping;
        Ok(())
    }

    pub async fn step_out_async(&self) -> Result<(), IDEError> {
        self.adapter.step_out().await?;
        *self.session_state.lock().await = DebugState::Stepping;
        Ok(())
    }

    pub async fn continue_async(&self) -> Result<(), IDEError> {
        self.adapter.continue_execution().await?;
        *self.session_state.lock().await = DebugState::Running;
        Ok(())
    }

    pub async fn pause_async(&self) -> Result<(), IDEError> {
        // GDB MI doesn't have a direct pause command, but could use interrupt
        // For now, this is a placeholder
        *self.session_state.lock().await = DebugState::Paused;
        Ok(())
    }

    pub async fn get_stack_trace_async(&self, thread_id: u32) -> Result<Vec<StackFrame>, IDEError> {
        self.adapter.get_stack_trace(thread_id).await
    }

    pub async fn evaluate_expression_async(&self, expression: &str) -> Result<String, IDEError> {
        self.adapter.get_variable_value(expression).await
    }

    pub async fn get_local_variables_async(&self) -> Result<Vec<VariableInfo>, IDEError> {
        // Would query GDB for local variables - placeholder
        Ok(vec![])
    }

    pub async fn set_variable_value_async(&self, _name: &str, _value: &str) -> Result<(), IDEError> {
        // Would use data-assign MI command - placeholder
        Ok(())
    }

    pub async fn get_session_state_async(&self) -> DebugState {
        *self.session_state.lock().await
    }

    pub async fn end_session_async(&self) -> Result<(), IDEError> {
        self.adapter.disconnect().await?;
        let mut breakpoints = self.breakpoints.write().await;
        let mut variables = self.variables.write().await;
        breakpoints.clear();
        variables.clear();
        *self.session_state.lock().await = DebugState::Ended;
        Ok(())
    }
}

/// Debug event stream for real-time debug event handling
pub struct DebugEventStream {
    pub(crate) receiver: mpsc::UnboundedReceiver<DebugEvent>,
}

impl DebugEventStream {
    pub fn new() -> (mpsc::UnboundedSender<DebugEvent>, Self) {
        let (tx, rx) = mpsc::unbounded_channel();
        (tx, Self { receiver: rx })
    }

    pub async fn next_event(&mut self) -> Option<DebugEvent> {
        self.receiver.recv().await
    }

    pub fn try_next_event(&mut self) -> Option<DebugEvent> {
        self.receiver.try_recv().ok()
    }
}

// Data structures

#[derive(Debug, Clone)]
pub enum MICommand {
    BreakInsert { file: String, line: u32 },
    BreakDelete { breakpoint: u32 },
    ExecNext { thread: Option<u32> },
    ExecStep { thread: Option<u32> },
    ExecFinish { thread: Option<u32> },
    ExecContinue { all_threads: bool },
    StackListFrames { thread: u32 },
    DataEvaluateExpression { expression: String },
    GDBExit,
}

#[derive(Debug, Clone)]
pub struct MICommandWithToken {
    pub token: String,
    pub command: MICommand,
}

impl std::fmt::Display for MICommandWithToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let command_str = match &self.command {
            MICommand::BreakInsert { file, line } => format!(" break-insert {}:{}", file, line),
            MICommand::BreakDelete { breakpoint } => format!(" break-delete {}", breakpoint),
            MICommand::ExecNext { thread } => {
                if let Some(t) = thread {
                    format!(" exec-next {}", t)
                } else {
                    " exec-next".to_string()
                }
            },
            MICommand::ExecStep { thread } => {
                if let Some(t) = thread {
                    format!(" exec-step {}", t)
                } else {
                    " exec-step".to_string()
                }
            },
            MICommand::ExecFinish { thread } => {
                if let Some(t) = thread {
                    format!(" exec-finish {}", t)
                } else {
                    " exec-finish".to_string()
                }
            },
            MICommand::ExecContinue { all_threads } => {
                if *all_threads {
                    " exec-continue --all".to_string()
                } else {
                    " exec-continue".to_string()
                }
            },
            MICommand::StackListFrames { thread } => format!(" stack-list-frames {}", thread),
            MICommand::DataEvaluateExpression { expression } => format!(" data-evaluate-expression \"{}\"", expression),
            MICommand::GDBExit => " gdb-exit".to_string(),
        };
        write!(f, "{}{}", self.token, command_str)
    }
}

#[derive(Debug, Clone)]
pub struct MIResponse {
    pub token: String,
    pub result_class: String,
    pub results: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointInfo {
    pub id: u32,
    pub file: String,
    pub line: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub level: u32,
    pub address: String,
    pub function: String,
    pub file: String,
    pub line: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInfo {
    pub name: String,
    pub value: String,
    pub type_info: String,
}

#[derive(Debug, Clone)]
pub enum DebugEvent {
    BreakpointHit {
        thread_id: u32,
        file: String,
        line: u32,
    },
    TargetRunning,
    TargetStopped,
    ThreadCreated(u32),
    ThreadExited(u32),
    VariableChanged(String),
    SignalReceived(String),
}

#[derive(Debug, Clone)]
pub enum DebugState {
    NotStarted,
    Running,
    Paused,
    Stepping,
    Ended,
}

#[derive(Debug, Clone)]
pub enum AdapterState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}
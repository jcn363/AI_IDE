use std::collections::HashMap;
use tokio::sync::{Mutex, mpsc};
use serde::{Deserialize, Serialize};
use rust_ai_ide_plugins::debugger::{
    Debugger, DebuggerConfig, VariableInfo, StackFrame, BreakpointInfo, DebuggerState
};

// Placeholder types for IDEState - these should match lib.rs definitions
#[derive(Debug)]
pub struct Workspace;
#[derive(Debug)]
pub struct Project;
#[derive(Debug)]
pub struct File;

// Copy IDEState from lib.rs to make it visible
#[derive(Default)]
pub struct IDEState {
    pub current_workspace: Option<Workspace>,
    pub open_files: HashMap<String, File>,
    pub current_project: Option<Project>,
    pub debugger: Arc<Mutex<Debugger>>,
}

// Uncomment if FileWatcher is used
// pub mod file_watcher;
// pub type FileWatcher = file_watcher::FileWatcher;

impl IDEState {
    pub fn new() -> Self {
        Self {
            current_workspace: None,
            open_files: HashMap::new(),
            current_project: None,
            debugger: Arc::new(Mutex::new(Debugger::new())),
        }
    }
}

// Import types needed for debugger commands
#[derive(Debug, Serialize, Deserialize)]
pub struct TerminalEvent {
    pub id: String,
    pub stream_type: String,
    pub line: String,
}

// Design pattern comments from task:
// Session Management: 8 commands (start_debug_session to debug_stop)
// Breakpoints: 4 commands (debugger_set_breakpoint to debugger_toggle_breakpoint, debugger_get_breakpoints)
// Variable Inspection: 6 commands (debugger_evaluate to debugger_get_call_stack, debugger_get_state)
// Variable Objects: 3 commands (debugger_var_create to debugger_var_children)

#[tauri::command]
pub async fn start_debug_session(
    executable_path: String,
    working_directory: String,
    args: Vec<String>,
    state: tauri::State<'_, Arc<Mutex<IDEState>>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let ide_state = state.lock().await;
    let mut debugger = ide_state.debugger.lock().await;

    let backend =
        Debugger::detect_backend().ok_or("No debugger backend (GDB/LLDB) found on system")?;

    let config = DebuggerConfig {
        backend,
        executable_path,
        working_directory,
        args,
        environment: HashMap::new(),
        stop_on_entry: true,
    };

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    // Start the debugger session
    debugger.start_session(config, tx).await?;

    // Forward debugger events to the frontend via Tauri events
    let app = app_handle.clone();
    let debugger_event_handle = tauri::async_runtime::spawn(async move {
        log::debug!("Starting debugger event forwarding task");
        while let Some(event) = rx.recv().await {
            if let Err(e) = app.emit_all("debugger-event", &event) {
                log::warn!("Failed to emit debugger event: {}", e);
            }
        }
        log::debug!("Debugger event forwarding task completed");
    });

    // Spawn monitoring task for debugger events
    tauri::async_runtime::spawn(async move {
        match debugger_event_handle.await {
            Ok(_) => log::debug!("Debugger event forwarding task completed without panic"),
            Err(e) => log::error!("Debugger event forwarding task panicked: {:?}", e),
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn debug_run(state: tauri::State<'_, Arc<Mutex<IDEState>>>) -> Result<(), String> {
    let ide_state = state.lock().await;
    let mut debugger = ide_state.debugger.lock().await;
    debugger.run().await
}

#[tauri::command]
pub async fn debug_continue(state: tauri::State<'_, Arc<Mutex<IDEState>>>) -> Result<(), String> {
    let ide_state = state.lock().await;
    let mut debugger = ide_state.debugger.lock().await;
    debugger.continue_execution().await
}

#[tauri::command]
pub async fn debug_step_over(state: tauri::State<'_, Arc<Mutex<IDEState>>>) -> Result<(), String> {
    let ide_state = state.lock().await;
    let mut debugger = ide_state.debugger.lock().await;
    debugger.step_over().await
}

#[tauri::command]
pub async fn debug_step_into(state: tauri::State<'_, Arc<Mutex<IDEState>>>) -> Result<(), String> {
    let ide_state = state.lock().await;
    let mut debugger = ide_state.debugger.lock().await;
    debugger.step_into().await
}

#[tauri::command]
pub async fn debug_step_out(state: tauri::State<'_, Arc<Mutex<IDEState>>>) -> Result<(), String> {
    let ide_state = state.lock().await;
    let mut debugger = ide_state.debugger.lock().await;
    debugger.step_out().await
}

// Additional Debugger Commands
#[tauri::command]
pub async fn debug_pause(state: tauri::State<'_, Arc<Mutex<IDEState>>>) -> Result<(), String> {
    let ide_state = state.lock().await;
    let mut debugger = ide_state.debugger.lock().await;
    debugger.pause().await
}

#[tauri::command]
pub async fn debug_stop(state: tauri::State<'_, Arc<Mutex<IDEState>>>) -> Result<(), String> {
    let ide_state = state.lock().await;
    let mut debugger = ide_state.debugger.lock().await;
    debugger.stop().await
}

#[tauri::command]
pub async fn debugger_set_breakpoint(
    file: String,
    line: u32,
    condition: Option<String>,
    state: tauri::State<'_, Arc<Mutex<IDEState>>>,
) -> Result<u32, String> {
    let ide_state = state.lock().await;
    let mut debugger = ide_state.debugger.lock().await;
    debugger.set_breakpoint(&file, line, condition).await
}

#[tauri::command]
pub async fn debugger_remove_breakpoint(
    id: u32,
    state: tauri::State<'_, Arc<Mutex<IDEState>>>,
) -> Result<(), String> {
    let ide_state = state.lock().await;
    let mut debugger = ide_state.debugger.lock().await;
    debugger.remove_breakpoint(id).await
}

#[tauri::command]
pub async fn debugger_toggle_breakpoint(
    id: u32,
    state: tauri::State<'_, Arc<Mutex<IDEState>>>,
) -> Result<(), String> {
    let ide_state = state.lock().await;
    let mut debugger = ide_state.debugger.lock().await;
    debugger.toggle_breakpoint(id).await
}

#[tauri::command]
pub async fn debugger_evaluate(
    expression: String,
    state: tauri::State<'_, Arc<Mutex<IDEState>>>,
) -> Result<String, String> {
    let ide_state = state.lock().await;
    let debugger = ide_state.debugger.lock().await;
    debugger.evaluate_expression(&expression).await
}

#[tauri::command]
pub async fn debugger_set_variable(
    name: String,
    value: String,
    state: tauri::State<'_, Arc<Mutex<IDEState>>>,
) -> Result<(), String> {
    let ide_state = state.lock().await;
    let mut debugger = ide_state.debugger.lock().await;
    debugger.set_variable(&name, &value).await
}

#[tauri::command]
pub async fn debugger_select_frame(
    frame_id: u32,
    state: tauri::State<'_, Arc<Mutex<IDEState>>>,
) -> Result<(), String> {
    let ide_state = state.lock().await;
    let mut debugger = ide_state.debugger.lock().await;
    debugger.select_frame(frame_id).await
}

#[tauri::command]
pub async fn debugger_get_variables(
    scope: Option<String>,
    state: tauri::State<'_, Arc<Mutex<IDEState>>>,
) -> Result<Vec<VariableInfo>, String> {
    let ide_state = state.lock().await;
    let debugger = ide_state.debugger.lock().await;

    // Use provided scope or default to "local", with explicit handling
    let scope = match scope {
        Some(s) => {
            if s.is_empty() {
                log::warn!("Empty scope provided for debugger variables, using 'local'");
                "local".to_string()
            } else {
                s
            }
        }
        None => {
            let default_scope = "local".to_string();
            log::debug!("No scope provided for debugger variables, using '{}'", default_scope);
            default_scope
        }
    };

    debugger.get_variables(&scope).await
}

#[tauri::command]
pub async fn debugger_get_call_stack(
    state: tauri::State<'_, Arc<Mutex<IDEState>>>,
) -> Result<Vec<StackFrame>, String> {
    let ide_state = state.lock().await;
    let debugger = ide_state.debugger.lock().await;
    debugger.get_call_stack().await
}

#[tauri::command]
pub async fn debugger_get_breakpoints(
    state: tauri::State<'_, Arc<Mutex<IDEState>>>,
) -> Result<Vec<BreakpointInfo>, String> {
    let ide_state = state.lock().await;
    let debugger = ide_state.debugger.lock().await;
    Ok(debugger
        .get_breakpoints()
        .into_iter()
        .cloned()
        .collect())
}

#[tauri::command]
pub async fn debugger_get_state(
    state: tauri::State<'_, Arc<Mutex<IDEState>>>,
) -> Result<DebuggerState, String> {
    let ide_state = state.lock().await;
    let debugger = ide_state.debugger.lock().await;
    Ok(debugger.get_state().clone())
}

// Variable Object Commands (on-demand variable expansion)
#[tauri::command]
pub async fn debugger_var_create(
    expression: String,
    state: tauri::State<'_, Arc<Mutex<IDEState>>>,
) -> Result<String, String> {
    let ide_state = state.lock().await;
    let mut debugger = ide_state.debugger.lock().await;
    debugger.create_var_object(&expression).await
}

#[tauri::command]
pub async fn debugger_var_delete(
    name: String,
    state: tauri::State<'_, Arc<Mutex<IDEState>>>,
) -> Result<(), String> {
    let ide_state = state.lock().await;
    let mut debugger = ide_state.debugger.lock().await;
    debugger.delete_var_object(&name).await
}

#[tauri::command]
pub async fn debugger_var_children(
    name: String,
    all_values: Option<bool>,
    state: tauri::State<'_, Arc<Mutex<IDEState>>>,
) -> Result<Vec<VariableInfo>, String> {
    let ide_state = state.lock().await;
    let mut debugger = ide_state.debugger.lock().await;
    debugger.list_var_children(&name, all_values.unwrap_or(true)).await
}
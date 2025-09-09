//! Application state management for the Tauri app.
//!
//! This module handles the global application state including
//! AI services, analysis progress, and other shared data.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use rust_ai_ide_debugger::{Debugger, DebuggerConfig, DebuggerState, BreakpointInfo, StackFrame, VariableInfo};
use crate::file_watcher::FileWatcher;
use crate::infra::{EventBus, RateLimiter, ConnectionPool};
use rust_ai_ide_lsp::pool::LanguageServerPool as LspPool;

/// Workspace information
#[derive(Clone, Debug)]
pub struct Workspace {
    pub path: String,
    pub name: String,
}

/// Project information
#[derive(Clone, Debug)]
pub struct Project {
    pub path: String,
    pub name: String,
    pub workspace_root: Option<String>,
}

/// Enhanced Application state structure integrating IDEState
#[derive(Clone)]
pub struct AppState {
    // Original AI service fields
    ai_service: Arc<Mutex<Option<rust_ai_ide_lsp::AIService>>>,
    analysis_progress: Arc<Mutex<std::collections::HashMap<String, f64>>>,

    // Extracted IDEState fields
    current_workspace: Arc<Mutex<Option<Workspace>>>,
    open_files: Arc<Mutex<HashMap<String, rust_ai_ide_core::File>>>,
    current_project: Arc<Mutex<Option<Project>>>,
    debugger: Arc<Mutex<Debugger>>,
    file_watcher: Arc<Mutex<Option<FileWatcher>>>,

    // Infrastructure components
    event_bus: EventBus,
    rate_limiter: RateLimiter,
    lsp_pool: LspPool,

    // Performance monitoring components
    performance_monitor: Arc<Mutex<Option<rust_ai_ide_performance_monitoring::PerformanceMonitor>>>,
    memory_optimizer: Arc<Mutex<Option<rust_ai_ide_performance_monitoring::memory::MemoryOptimizer>>>,
    battery_monitor: Arc<Mutex<Option<rust_ai_ide_performance_monitoring::battery::BatteryMonitor>>>,
}

impl AppState {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            ai_service: Arc::new(Mutex::new(None)),
            analysis_progress: Arc::new(Mutex::new(std::collections::HashMap::new())),
            current_workspace: Arc::new(Mutex::new(None)),
            open_files: Arc::new(Mutex::new(HashMap::new())),
            current_project: Arc::new(Mutex::new(None)),
            debugger: Arc::new(Mutex::new(Debugger::new())),
            file_watcher: Arc::new(Mutex::new(None)),
            event_bus: EventBus::new(1000), // Buffer size for event bus
            rate_limiter: RateLimiter::new(100, std::time::Duration::from_secs(60)), // 100 requests per minute
            lsp_pool: LspPool::new(10), // Pool of 10 LSP connections
            performance_monitor: Arc::new(Mutex::new(None)),
            memory_optimizer: Arc::new(Mutex::new(None)),
            battery_monitor: Arc::new(Mutex::new(None)),
        })
    }

    // AI service methods
    pub async fn set_ai_service(&self, service: rust_ai_ide_lsp::AIService) {
        *self.ai_service.lock().await = Some(service);
    }

    pub async fn get_ai_service(&self) -> Option<rust_ai_ide_lsp::AIService> {
        self.ai_service.lock().await.clone()
    }

    pub async fn set_analysis_progress(&self, task_id: String, progress: f64) {
        self.analysis_progress.lock().await.insert(task_id, progress);
    }

    pub async fn get_analysis_progress(&self, task_id: &str) -> Option<f64> {
        self.analysis_progress.lock().await.get(task_id).copied()
    }

    // IDE state methods
    pub async fn set_workspace(&self, workspace: Workspace) {
        *self.current_workspace.lock().await = Some(workspace);
    }

    pub async fn get_workspace(&self) -> Option<Workspace> {
        self.current_workspace.lock().await.clone()
    }

    pub async fn set_project(&self, project: Project) {
        *self.current_project.lock().await = Some(project);
    }

    pub async fn get_project(&self) -> Option<Project> {
        self.current_project.lock().await.clone()
    }

    pub async fn add_open_file(&self, path: String, file: rust_ai_ide_core::File) {
        self.open_files.lock().await.insert(path, file);
    }

    pub async fn remove_open_file(&self, path: &str) {
        self.open_files.lock().await.remove(path);
    }

    pub async fn get_open_file(&self, path: &str) -> Option<rust_ai_ide_core::File> {
        self.open_files.lock().await.get(path).cloned()
    }

    pub async fn get_debugger(&self) -> Arc<Mutex<Debugger>> {
        Arc::clone(&self.debugger)
    }

    pub async fn set_file_watcher(&self, watcher: FileWatcher) {
        *self.file_watcher.lock().await = Some(watcher);
    }

    pub async fn get_file_watcher(&self) -> Option<FileWatcher> {
        self.file_watcher.lock().await.clone()
    }

    // Infrastructure component accessors
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    pub fn rate_limiter(&self) -> &RateLimiter {
        &self.rate_limiter
    }

    pub fn lsp_pool(&self) -> &ConnectionPool {
        &self.lsp_pool
    }

    // Performance monitoring accessors
    pub async fn set_performance_monitor(&self, monitor: rust_ai_ide_performance_monitoring::PerformanceMonitor) {
        *self.performance_monitor.lock().await = Some(monitor);
    }

    pub async fn get_performance_monitor(&self) -> Option<rust_ai_ide_performance_monitoring::PerformanceMonitor> {
        self.performance_monitor.lock().await.clone()
    }

    pub async fn set_memory_optimizer(&self, optimizer: rust_ai_ide_performance_monitoring::memory::MemoryOptimizer) {
        *self.memory_optimizer.lock().await = Some(optimizer);
    }

    pub async fn get_memory_optimizer(&self) -> Option<rust_ai_ide_performance_monitoring::memory::MemoryOptimizer> {
        self.memory_optimizer.lock().await.clone()
    }

    pub async fn set_battery_monitor(&self, monitor: rust_ai_ide_performance_monitoring::battery::BatteryMonitor) {
        *self.battery_monitor.lock().await = Some(monitor);
    }

    pub async fn get_battery_monitor(&self) -> Option<rust_ai_ide_performance_monitoring::battery::BatteryMonitor> {
        self.battery_monitor.lock().await.clone()
    }
}

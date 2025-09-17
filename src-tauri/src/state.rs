//! # Application State Management Module
//!
//! This module implements the centralized state management system for the Rust AI IDE application.
//! It coordinates multiple subsystems including AI services, debugging, file watching, LSP
//! connections, and performance monitoring through a unified, thread-safe interface.
//!
//! ## Architecture Overview
//!
//! The state management follows the double-locking pattern described in AGENTS.md for lazy
//! async initialization. All state is wrapped in `Arc<Mutex<T>>` to ensure thread safety
//! in the async Tokio runtime.
//!
//! ### Key Components
//!
//! - **AppState**: Central state container holding all application subsystems
//! - **Workspace**: Represents a user's workspace/project root
//! - **Project**: Represents individual projects within a workspace
//! - **Service Coordination**: Manages initialization and access to all services
//!
//! ## State Structure
//!
//! ```text
//! AppState
//! ├── AI Services (LSP, analysis, codegen)
//! ├── IDE State (workspace, projects, open files)
//! ├── Debugging (debugger, breakpoints, call stack)
//! ├── Infrastructure (events, rate limiting, connections)
//! ├── Performance (monitoring, memory, battery)
//! └── File Watching (change detection, debouncing)
//! ```
//!
//! ## Usage Patterns
//!
//! ### Service Initialization
//! ```rust,ignore
//! let state = AppState::new()?;
//!
//! // Initialize AI service
//! let ai_service = AIService::new(config).await?;
//! state.set_ai_service(ai_service).await;
//!
//! // Initialize debugger
//! let debugger = Debugger::new();
//! state.get_debugger().lock().await.replace(debugger);
//! ```
//!
//! ### Concurrent Access
//! ```rust,ignore
//! // Thread-safe access to AI service
//! let ai_result = state.get_ai_service()
//!     .ok_or("AI service not initialized")?
//!     .analyze_code(code).await?;
//!
//! // Safe concurrent file operations
//! let file_content = state.get_open_file("main.rs").await;
//! ```
//!
//! ### Event-Driven Updates
//! ```rust,ignore
//! // Emit events for UI updates
//! state.event_bus().emit("file_changed", json!({
//!     "path": "/project/main.rs",
//!     "action": "modified"
//! })).await?;
//! ```
//!
//! ## Thread Safety
//!
//! All state access is protected by async mutexes:
//!
//! - **Arc<Mutex<>>**: Provides thread-safe shared ownership
//! - **Async Mutexes**: Prevent race conditions in async contexts
//! - **Clone Semantics**: State can be safely shared across tasks
//!
//! ## Initialization Order
//!
//! Services must be initialized in the correct order:
//!
//! 1. Infrastructure (EventBus, RateLimiter, LSP Pool)
//! 2. Core IDE state (Workspace, Projects)
//! 3. AI services (requires LSP infrastructure)
//! 4. Performance monitoring (optional, can initialize late)
//!
//! ## Error Handling
//!
//! State operations follow the error aggregation pattern:
//!
//! - Service unavailability returns descriptive errors
//! - Initialization failures prevent partial states
//! - All operations are atomic where possible
//!
//! ## Performance Considerations
//!
//! - **Lazy Initialization**: Services initialize only when needed
//! - **Connection Pooling**: LSP connections are pooled for reuse
//! - **Event Batching**: File changes are debounced to reduce noise
//! - **Memory Management**: Large workspaces use virtual memory management

use std::collections::HashMap;
use std::sync::Arc;

use rust_ai_ide_debugger::{BreakpointInfo, Debugger, DebuggerConfig, DebuggerState, StackFrame, VariableInfo};
use rust_ai_ide_lsp::pool::LanguageServerPool as LspPool;
use rust_ai_ide_observability::ObservabilityManager;
use tokio::sync::Mutex;

use crate::file_watcher::FileWatcher;
use crate::infra::{ConnectionPool, EventBus, RateLimiter};

/// # Workspace Information Structure
///
/// Represents a user's workspace, which is typically the root directory of a project
/// or a collection of related projects. Workspaces provide the context for operations
/// like file watching, project discovery, and scope-limited analysis.
///
/// ## Fields
/// - `path`: Absolute path to the workspace root directory
/// - `name`: Human-readable name for the workspace
///
/// ## Usage
/// ```rust,ignore
/// let workspace = Workspace {
///     path: "/home/user/projects/rust-ai-ide".to_string(),
///     name: "Rust AI IDE".to_string(),
/// };
///
/// state.set_workspace(workspace).await;
/// ```
#[derive(Clone, Debug)]
pub struct Workspace {
    /// Absolute path to the workspace root directory
    pub path: String,
    /// Human-readable display name for the workspace
    pub name: String,
}

/// # Project Information Structure
///
/// Represents an individual project within a workspace. Projects may have their own
/// configuration, dependencies, and build settings that differ from the workspace.
///
/// ## Fields
/// - `path`: Absolute path to the project directory
/// - `name`: Human-readable name for the project
/// - `workspace_root`: Optional reference to parent workspace
///
/// ## Usage
/// ```rust,ignore
/// let project = Project {
///     path: "/home/user/projects/rust-ai-ide/crates/my-crate".to_string(),
///     name: "my-crate".to_string(),
///     workspace_root: Some("/home/user/projects/rust-ai-ide".to_string()),
/// };
///
/// state.set_project(project).await;
/// ```
#[derive(Clone, Debug)]
pub struct Project {
    /// Absolute path to the project directory
    pub path:           String,
    /// Human-readable name for the project
    pub name:           String,
    /// Optional path to the parent workspace root
    pub workspace_root: Option<String>,
}

/// # Central Application State Container
///
/// AppState is the unified state management structure for the Rust AI IDE, coordinating
/// all major subsystems through thread-safe, async-aware state containers. It implements
/// the state management patterns described in AGENTS.md with double-locking for lazy
/// initialization and Arc<Mutex<>> for thread safety.
///
/// ## State Organization
///
/// The state is organized into logical groups for maintainability:
///
/// ### Core IDE State
/// - **Workspace/Project**: Current workspace and project context
/// - **Open Files**: Currently loaded files with their content
/// - **Debugger**: Debugging session state and breakpoints
/// - **File Watcher**: File system change monitoring
///
/// ### AI Services
/// - **AI Service**: Primary AI analysis and code generation service
/// - **Analysis Progress**: Progress tracking for long-running AI operations
/// - **LSP Pool**: Connection pool for language server protocol services
///
/// ### Infrastructure
/// - **EventBus**: Pub-sub communication system for inter-module events
/// - **RateLimiter**: Request rate limiting for resource protection
///
/// ### Performance & Monitoring
/// - **Performance Monitor**: System performance tracking
/// - **Memory Optimizer**: Memory usage optimization
/// - **Battery Monitor**: Power management and optimization
///
/// ## Thread Safety
///
/// All fields use `Arc<Mutex<T>>` for thread-safe shared access:
///
/// - **Arc**: Atomic reference counting for shared ownership
/// - **Mutex**: Exclusive access control in async contexts
/// - **Clone**: State can be safely shared across tasks
///
/// ## Initialization Pattern
///
/// Services are initialized lazily using the double-locking pattern:
///
/// ```rust,ignore
/// // First check (fast path)
/// if let Some(service) = state.get_ai_service().await {
///     return service;
/// }
///
/// // Second check with lock (slow path)
/// let mut ai_service_guard = state.ai_service.lock().await;
/// if ai_service_guard.is_none() {
///     *ai_service_guard = Some(AIService::new(config).await?);
/// }
/// ai_service_guard.as_ref().unwrap()
/// ```
///
/// ## Usage Examples
///
/// ### Basic State Access
/// ```rust,ignore
/// let state = AppState::new()?;
///
/// // Set workspace context
/// let workspace = Workspace { path: "/project".into(), name: "My Project".into() };
/// state.set_workspace(workspace).await;
///
/// // Access AI service
/// if let Some(ai) = state.get_ai_service().await {
///     let result = ai.analyze_code(code).await?;
/// }
/// ```
///
/// ### Event Emission
/// ```rust,ignore
/// state.event_bus().emit("file_saved", json!({
///     "path": file_path,
///     "timestamp": Utc::now()
/// })).await?;
/// ```
///
/// ### Performance Monitoring
/// ```rust,ignore
/// let monitor = PerformanceMonitor::new(config);
/// state.set_performance_monitor(monitor).await;
///
/// // Later access for metrics
/// if let Some(perf) = state.get_performance_monitor().await {
///     let metrics = perf.collect_metrics().await?;
/// }
/// ```
///
/// ## Memory Management
///
/// - **Lazy Loading**: Services initialize only when first accessed
/// - **Resource Pooling**: LSP connections are pooled for reuse
/// - **Cleanup**: Proper resource cleanup on application shutdown
/// - **Virtual Memory**: Large workspaces use virtual memory management
///
/// ## Error Handling
///
/// State operations follow the error aggregation pattern from AGENTS.md:
///
/// - Service unavailable returns descriptive errors
/// - Initialization failures are propagated with context
/// - All operations maintain state consistency
///
/// ## Performance Considerations
///
/// - **Lock Contention**: Minimize time spent holding locks
/// - **Async Design**: All operations are async to prevent blocking
/// - **Connection Pooling**: LSP connections are reused efficiently
/// - **Event Batching**: File changes are debounced to reduce event noise
#[derive(Clone)]
pub struct AppState {
    // Original AI service fields
    /// Primary AI service for code analysis and generation
    ai_service:        Arc<Mutex<Option<rust_ai_ide_lsp::AIService>>>,
    /// Progress tracking for long-running AI analysis operations
    analysis_progress: Arc<Mutex<std::collections::HashMap<String, f64>>>,

    // Extracted IDEState fields
    /// Current workspace context (project root and metadata)
    current_workspace: Arc<Mutex<Option<Workspace>>>,
    /// Map of currently open files (path -> File content)
    open_files:        Arc<Mutex<HashMap<String, rust_ai_ide_core::File>>>,
    /// Current project context within the workspace
    current_project:   Arc<Mutex<Option<Project>>>,
    /// Integrated debugger for code debugging and breakpoints
    debugger:          Arc<Mutex<Debugger>>,
    /// File system watcher for detecting changes (optional, lazy-loaded)
    file_watcher:      Arc<Mutex<Option<FileWatcher>>>,

    // Infrastructure components
    /// Event bus for inter-module pub-sub communication
    event_bus:    EventBus,
    /// Rate limiter for protecting against resource exhaustion
    rate_limiter: RateLimiter,
    /// Connection pool for LSP (Language Server Protocol) services
    lsp_pool:     LspPool,

    // Performance monitoring components
    /// System performance monitoring service
    performance_monitor: Arc<Mutex<Option<rust_ai_ide_performance_monitoring::PerformanceMonitor>>>,
    /// Memory optimization and monitoring service
    memory_optimizer:    Arc<Mutex<Option<rust_ai_ide_performance_monitoring::memory::MemoryOptimizer>>>,
    /// Battery and power management monitoring
    battery_monitor:     Arc<Mutex<Option<rust_ai_ide_performance_monitoring::battery::BatteryMonitor>>>,

    // Observability and monitoring
    /// Comprehensive observability manager for metrics, tracing, and health checks
    observability_manager: Arc<Mutex<Option<ObservabilityManager>>>,

    // Model warmup prediction system
    /// Main model warmup predictor service
    warmup_predictor: Arc<Mutex<Option<rust_ai_ide_warmup_predictor::ModelWarmupPredictor>>>,
    /// Advanced pattern analyzer for user behavior
    pattern_analyzer: Arc<Mutex<Option<rust_ai_ide_warmup_predictor::advanced_patterns::AdvancedPatternAnalyzer>>>,
    /// ML model trainer for prediction accuracy
    ml_trainer: Arc<Mutex<Option<rust_ai_ide_warmup_predictor::ml_trainer::MLModelTrainer>>>,
    /// ML model evaluator for performance assessment
    ml_evaluator: Arc<Mutex<Option<rust_ai_ide_warmup_predictor::ml_evaluator::MLModelEvaluator>>>,
    /// Performance benchmarker for system evaluation
    benchmarker: Arc<Mutex<Option<rust_ai_ide_warmup_predictor::benchmark_tools::PerformanceBenchmarker>>>,
}

impl AppState {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            ai_service:            Arc::new(Mutex::new(None)),
            analysis_progress:     Arc::new(Mutex::new(std::collections::HashMap::new())),
            current_workspace:     Arc::new(Mutex::new(None)),
            open_files:            Arc::new(Mutex::new(HashMap::new())),
            current_project:       Arc::new(Mutex::new(None)),
            debugger:              Arc::new(Mutex::new(Debugger::new())),
            file_watcher:          Arc::new(Mutex::new(None)),
            event_bus:             EventBus::new(1000), // Buffer size for event bus
            rate_limiter:          RateLimiter::new(100, std::time::Duration::from_secs(60)), // 100 requests per minute
            lsp_pool:              LspPool::new(10),    // Pool of 10 LSP connections
            performance_monitor:   Arc::new(Mutex::new(None)),
            memory_optimizer:      Arc::new(Mutex::new(None)),
            battery_monitor:       Arc::new(Mutex::new(None)),
            observability_manager: Arc::new(Mutex::new(None)),
            warmup_predictor:      Arc::new(Mutex::new(None)),
            pattern_analyzer:      Arc::new(Mutex::new(None)),
            ml_trainer:            Arc::new(Mutex::new(None)),
            ml_evaluator:          Arc::new(Mutex::new(None)),
            benchmarker:           Arc::new(Mutex::new(None)),
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
        self.analysis_progress
            .lock()
            .await
            .insert(task_id, progress);
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

    // Observability accessors
    pub async fn set_observability_manager(&self, manager: ObservabilityManager) {
        *self.observability_manager.lock().await = Some(manager);
    }

    pub async fn get_observability_manager(&self) -> Option<ObservabilityManager> {
        self.observability_manager.lock().await.clone()
    }

    // Warmup predictor service accessors
    pub async fn set_warmup_predictor(&self, predictor: rust_ai_ide_warmup_predictor::ModelWarmupPredictor) {
        *self.warmup_predictor.lock().await = Some(predictor);
    }

    pub async fn get_warmup_predictor(&self) -> Option<rust_ai_ide_warmup_predictor::ModelWarmupPredictor> {
        self.warmup_predictor.lock().await.clone()
    }

    pub async fn set_pattern_analyzer(&self, analyzer: rust_ai_ide_warmup_predictor::advanced_patterns::AdvancedPatternAnalyzer) {
        *self.pattern_analyzer.lock().await = Some(analyzer);
    }

    pub async fn get_pattern_analyzer(&self) -> Option<rust_ai_ide_warmup_predictor::advanced_patterns::AdvancedPatternAnalyzer> {
        self.pattern_analyzer.lock().await.clone()
    }

    pub async fn set_ml_trainer(&self, trainer: rust_ai_ide_warmup_predictor::ml_trainer::MLModelTrainer) {
        *self.ml_trainer.lock().await = Some(trainer);
    }

    pub async fn get_ml_trainer(&self) -> Option<rust_ai_ide_warmup_predictor::ml_trainer::MLModelTrainer> {
        self.ml_trainer.lock().await.clone()
    }

    pub async fn set_ml_evaluator(&self, evaluator: rust_ai_ide_warmup_predictor::ml_evaluator::MLModelEvaluator) {
        *self.ml_evaluator.lock().await = Some(evaluator);
    }

    pub async fn get_ml_evaluator(&self) -> Option<rust_ai_ide_warmup_predictor::ml_evaluator::MLModelEvaluator> {
        self.ml_evaluator.lock().await.clone()
    }

    pub async fn set_benchmarker(&self, benchmarker: rust_ai_ide_warmup_predictor::benchmark_tools::PerformanceBenchmarker) {
        *self.benchmarker.lock().await = Some(benchmarker);
    }

    pub async fn get_benchmarker(&self) -> Option<rust_ai_ide_warmup_predictor::benchmark_tools::PerformanceBenchmarker> {
        self.benchmarker.lock().await.clone()
    }
}

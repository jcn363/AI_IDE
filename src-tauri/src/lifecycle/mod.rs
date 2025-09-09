//! Application lifecycle management
//!
//! This module provides a structured approach to managing the application's
//! lifecycle phases: startup, runtime, and cleanup. This separation allows
//! for better observability, controllability, and maintainability of the
//! application state.

pub mod startup;
pub mod runtime;
pub mod cleanup;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Lifecycle phase tracking
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LifecyclePhase {
    Initializing,
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

impl std::fmt::Display for LifecyclePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifecyclePhase::Initializing => write!(f, "Initializing"),
            LifecyclePhase::Starting => write!(f, "Starting"),
            LifecyclePhase::Running => write!(f, "Running"),
            LifecyclePhase::Stopping => write!(f, "Stopping"),
            LifecyclePhase::Stopped => write!(f, "Stopped"),
            LifecyclePhase::Failed => write!(f, "Failed"),
        }
    }
}

/// Lifecycle event for observability
#[derive(Debug, Clone)]
pub struct LifecycleEvent {
    pub phase: LifecyclePhase,
    pub message: String,
    pub timestamp: std::time::SystemTime,
    pub success: bool,
    pub metadata: serde_json::Value,
}

impl Default for LifecycleEvent {
    fn default() -> Self {
        Self {
            phase: LifecyclePhase::Initializing,
            message: String::new(),
            timestamp: std::time::SystemTime::now(),
            success: true,
            metadata: serde_json::Value::Null,
        }
    }
}

/// Lifecycle manager that orchestrates the application lifecycle
pub struct LifecycleManager {
    current_phase: Arc<Mutex<LifecyclePhase>>,
    event_listeners: Arc<Mutex<Vec<Box<dyn Fn(LifecycleEvent) + Send + Sync>>>>,
}

impl LifecycleManager {
    pub fn new() -> Self {
        Self {
            current_phase: Arc::new(Mutex::new(LifecyclePhase::Initializing)),
            event_listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn current_phase(&self) -> LifecyclePhase {
        *self.current_phase.lock().await
    }

    pub async fn update_phase(&self, phase: LifecyclePhase) {
        *self.current_phase.lock().await = phase;
        log::info!("Lifecycle phase transitioned to: {}", phase);
    }

    pub async fn emit_event(&self, event: LifecycleEvent) {
        log::info!("Lifecycle event: {} - {} ({})",
                   event.phase,
                   event.message,
                   if event.success { "success" } else { "failure" });

        let listeners = self.event_listeners.lock().await;
        for listener in listeners.iter() {
            listener(event.clone());
        }
    }

    pub async fn add_event_listener<F>(&self, listener: F)
    where
        F: Fn(LifecycleEvent) + Send + Sync + 'static,
    {
        self.event_listeners.lock().await.push(Box::new(listener));
    }

    /// Orchestrates the complete application lifecycle
    pub async fn run_lifecycle(&self, app: &tauri::App) -> Result<()> {
        let startup = startup::StartupPhase::new();
        let runtime = runtime::RuntimePhase::new();
        let cleanup = cleanup::CleanupPhase::new();

        // Startup phase
        if let Err(e) = startup.execute(app, Arc::clone(&self.current_phase)).await {
            self.update_phase(LifecyclePhase::Failed).await;
            self.emit_event(LifecycleEvent {
                phase: LifecyclePhase::Failed,
                message: format!("Startup failed: {}", e),
                success: false,
                metadata: serde_json::json!({ "error": e.to_string() }),
                ..Default::default()
            }).await;
            return Err(e);
        }

        self.update_phase(LifecyclePhase::Running).await;

        // Runtime phase (runs concurrently)
        let runtime_handle = tokio::spawn(async move {
            runtime.run(Arc::clone(&self.current_phase)).await
        });

        // Wait for shutdown signal
        self.wait_for_shutdown().await;

        // Cleanup phase
        self.update_phase(LifecyclePhase::Stopping).await;
        if let Err(e) = cleanup.execute().await {
            log::error!("Cleanup failed: {}", e);
        }

        self.update_phase(LifecyclePhase::Stopped).await;
        self.emit_event(LifecycleEvent {
            phase: LifecyclePhase::Stopped,
            message: "Application shutdown complete".to_string(),
            success: true,
            ..Default::default()
        }).await;

        Ok(())
    }

    async fn wait_for_shutdown(&self) {
        // In a real application, this would listen for shutdown signals
        // For now, we'll just wait for the runtime to finish
        tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        log::info!("Shutdown signal received");
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}
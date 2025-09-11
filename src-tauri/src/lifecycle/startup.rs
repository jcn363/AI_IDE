//! Startup phase implementation
//!
//! This module handles the application startup phase, including:
//! - Tauri plugin initialization
//! - State management setup
//! - Service initialization
//! - Command handler registration
//! - Background task startup

use super::{LifecycleEvent, LifecyclePhase};
use crate::commands;
use crate::utils;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct StartupPhase {
    event_listeners: Vec<Box<dyn Fn(LifecycleEvent) + Send + Sync>>,
}

impl StartupPhase {
    pub fn new() -> Self {
        Self {
            event_listeners: Vec::new(),
        }
    }

    pub async fn execute(
        &self,
        app: &tauri::App,
        phase_state: Arc<Mutex<LifecyclePhase>>,
    ) -> Result<()> {
        log::info!("Starting application startup phase");

        // Update phase state
        *phase_state.lock().await = LifecyclePhase::Starting;

        // Phase 1: Tauri plugin initialization
        self.init_plugins(app).await?;

        // Phase 2: State management setup
        self.setup_app_state(app).await?;

        // Phase 3: Command handler initialization
        self.init_command_handlers(app).await?;

        // Phase 4: Service initialization
        self.init_services(app).await?;

        // Phase 5: Background tasks
        self.start_background_tasks(app).await?;

        log::info!("Startup phase completed successfully");
        Ok(())
    }

    async fn init_plugins(&self, app: &tauri::App) -> Result<()> {
        log::debug!("Initializing Tauri plugins");

        // Note: Plugins are typically initialized before app builder setup
        // This is just for observability of what plugins are loaded

        self.emit_event(LifecycleEvent {
            phase: LifecyclePhase::Starting,
            message: "Tauri plugins initialized".to_string(),
            success: true,
            metadata: serde_json::json!({
                "plugins": ["log", "fs", "dialog"]
            }),
            ..Default::default()
        })
        .await;

        Ok(())
    }

    async fn setup_app_state(&self, app: &tauri::App) -> Result<()> {
        log::debug!("Setting up application state management");

        // State is already managed in the builder, so we'll just emit observability event
        self.emit_event(LifecycleEvent {
            phase: LifecyclePhase::Starting,
            message: "Application state management configured".to_string(),
            success: true,
            metadata: serde_json::json!({
                "states": ["IDEState", "AIServiceState", "AnalysisProgressState", "DiagnosticCache", "ExplanationCache"],
                "caches": ["diagnostic", "explanation"]
            }),
            ..Default::default()
        }).await;

        Ok(())
    }

    async fn init_command_handlers(&self, app: &tauri::App) -> Result<()> {
        log::debug!("Command handlers should be initialized in main setup function");

        self.emit_event(LifecycleEvent {
            phase: LifecyclePhase::Starting,
            message: "Command handlers initialization delegated to main setup".to_string(),
            success: true,
            metadata: serde_json::json!({
                "handlers_initialized": false,
                "delegated": true
            }),
            ..Default::default()
        })
        .await;

        Ok(())
    }

    async fn init_services(&self, app: &tauri::App) -> Result<()> {
        log::debug!("Initializing application services");

        // Initialize AI service
        let ai_service_state = app.state::<crate::commands::ai::services::AIServiceState>();
        let init_result = utils::initialize_ai_service_on_startup(ai_service_state).await;

        match init_result {
            Ok(_) => {
                log::info!("AI service initialized during startup");
                self.emit_event(LifecycleEvent {
                    phase: LifecyclePhase::Starting,
                    message: "AI service initialized".to_string(),
                    success: true,
                    metadata: serde_json::json!({
                        "service": "ai",
                        "status": "ready"
                    }),
                    ..Default::default()
                })
                .await;
            }
            Err(e) => {
                log::warn!("AI service initialization failed during startup: {}", e);
                self.emit_event(LifecycleEvent {
                    phase: LifecyclePhase::Starting,
                    message: "AI service initialization failed".to_string(),
                    success: false,
                    metadata: serde_json::json!({
                        "service": "ai",
                        "error": e.to_string()
                    }),
                    ..Default::default()
                })
                .await;
            }
        }

        Ok(())
    }

    async fn start_background_tasks(&self, app: &tauri::App) -> Result<()> {
        log::debug!("Starting background tasks");

        // Initialize cache cleanup task
        let diagnostic_cache_state =
            app.state::<crate::modules::shared::diagnostics::DiagnosticCacheState>();
        let explanation_cache_state =
            app.state::<crate::modules::shared::diagnostics::ExplanationCacheState>();

        let cache_cleanup_result =
            utils::initialize_cache_cleanup_task(diagnostic_cache_state, explanation_cache_state)
                .await;

        match cache_cleanup_result {
            Ok(_) => {
                self.emit_event(LifecycleEvent {
                    phase: LifecyclePhase::Starting,
                    message: "Background tasks started".to_string(),
                    success: true,
                    metadata: serde_json::json!({
                        "tasks": ["cache_cleanup"]
                    }),
                    ..Default::default()
                })
                .await;
            }
            Err(e) => {
                log::error!("Failed to start cache cleanup task: {}", e);
                self.emit_event(LifecycleEvent {
                    phase: LifecyclePhase::Starting,
                    message: "Background tasks startup failed".to_string(),
                    success: false,
                    metadata: serde_json::json!({
                        "error": e.to_string()
                    }),
                    ..Default::default()
                })
                .await;
            }
        }

        Ok(())
    }

    async fn emit_event(&self, event: LifecycleEvent) {
        log::info!("Startup event: {} - {}", event.phase, event.message);
        // In a real implementation, this would notify registered listeners
    }
}

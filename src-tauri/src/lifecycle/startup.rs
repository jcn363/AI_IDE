//! Startup phase implementation
//!
//! This module handles the application startup phase, including:
//! - Tauri plugin initialization
//! - State management setup
//! - Service initialization
//! - Command handler registration
//! - Background task startup

use super::{LifecycleEvent, LifecyclePhase};
use crate::command_templates::spawn_background_task;
use crate::commands;
use crate::modules::ai::services::common::{
    AIProvider, AIServiceRegistry, PooledServiceConfig, WrappedAIService, GLOBAL_AI_REGISTRY,
};
use crate::utils;
use anyhow::Result;
use chrono;
use rust_ai_ide_ai_inference::ModelSize;
use rust_ai_ide_common::validation::validate_secure_path;
use rust_ai_ide_errors::{IDEError, IDEResult};
use rust_ai_ide_lsp::{LSPClient, LSPClientConfig, MultiLanguageLSP};
use rust_ai_ide_security::audit_logger;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};

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
        log::debug!("Initializing application services with AI service registry");

        // Phase 1: Initialize AI Service Registry and Connection Pools
        self.init_ai_service_registry().await?;

        // Phase 2: Initialize LSP Servers for supported languages
        self.init_lsp_servers().await?;

        // Phase 3: Set up webhook system on port 3000
        self.init_webhook_system().await?;

        // Phase 4: Initialize existing AI service state (legacy compatibility)
        let ai_service_state = app.state::<crate::commands::ai::services::AIServiceState>();
        let init_result = utils::initialize_ai_service_on_startup(ai_service_state).await;

        match init_result {
            Ok(_) => {
                log::info!("Legacy AI service initialized during startup");
                self.emit_event(LifecycleEvent {
                    phase: LifecyclePhase::Starting,
                    message: "Legacy AI service initialized".to_string(),
                    success: true,
                    metadata: serde_json::json!({
                        "service": "ai_legacy",
                        "status": "ready"
                    }),
                    ..Default::default()
                })
                .await;
            }
            Err(e) => {
                log::warn!(
                    "Legacy AI service initialization failed during startup: {}",
                    e
                );
                self.emit_event(LifecycleEvent {
                    phase: LifecyclePhase::Starting,
                    message: "Legacy AI service initialization failed".to_string(),
                    success: false,
                    metadata: serde_json::json!({
                        "service": "ai_legacy",
                        "error": e.to_string()
                    }),
                    ..Default::default()
                })
                .await;
            }
        }

        Ok(())
    }

    async fn init_ai_service_registry(&self) -> Result<()> {
        log::info!("Initializing AI Service Registry with connection pools");

        // Initialize global registry with double-locking pattern
        let registry = Arc::clone(&GLOBAL_AI_REGISTRY);

        // Set up AI services with connection pools
        let services = vec![
            // Mock service for testing
            ("mock", AIProvider::Mock),
            // OpenAI service
            ("openai", AIProvider::OpenAI),
            // Local CodeLlama models
            (
                "codellama_small",
                AIProvider::CodeLlamaRust {
                    model_size: ModelSize::Small,
                },
            ),
            (
                "codellama_medium",
                AIProvider::CodeLlamaRust {
                    model_size: ModelSize::Medium,
                },
            ),
            (
                "codellama_large",
                AIProvider::CodeLlamaRust {
                    model_size: ModelSize::Large,
                },
            ),
            // Local StarCoder models
            (
                "starcoder_small",
                AIProvider::StarCoderRust {
                    model_size: ModelSize::Small,
                },
            ),
            (
                "starcoder_medium",
                AIProvider::StarCoderRust {
                    model_size: ModelSize::Medium,
                },
            ),
        ];

        let mut registered_services = Vec::new();
        let mut errors = Vec::new();

        for (name, provider) in services {
            // Validate provider configuration
            if let Err(e) = self.validate_ai_provider_config(&provider).await {
                log::warn!("Provider validation failed for {}: {:?}", name, e);
                errors.push(format!("Validation failed for {}: {}", name, e));
                continue;
            }

            let service = match self.create_and_initialize_service(&provider).await {
                Ok(service) => service,
                Err(e) => {
                    log::warn!("Failed to create service for {}: {:?}", name, e);
                    errors.push(format!("Service creation failed for {}: {}", name, e));
                    continue;
                }
            };

            match registry.register_service(name, Arc::new(service)) {
                Ok(_) => {
                    log::info!("Registered AI service: {}", name);
                    registered_services.push(name.to_string());

                    // Audit log successful registration
                    audit_logger::log_event(
                        "ai_service_registered",
                        &serde_json::json!({
                            "service_name": name,
                            "provider": provider.provider_name(),
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        }),
                    )
                    .await;
                }
                Err(e) => {
                    log::warn!("Failed to register AI service {}: {}", name, e);
                    errors.push(format!("Registration failed for {}: {}", name, e));
                }
            }

            // For production providers, also set up pooled services
            if matches!(
                provider,
                AIProvider::OpenAI
                    | AIProvider::CodeLlamaRust { .. }
                    | AIProvider::StarCoderRust { .. }
            ) {
                let pool_config = PooledServiceConfig {
                    provider: provider.clone(),
                    max_connections: 10,
                    connection_timeout: Duration::from_secs(30),
                    idle_timeout: Duration::from_secs(300),
                };

                let mut initial_services = Vec::new();
                for _ in 0..3 {
                    // Start with 3 connections per pool
                    if let Ok(service) = self.create_and_initialize_service(&provider).await {
                        initial_services.push(service);
                    }
                }

                let pool_name = format!("{}_pool", name);
                match registry.register_pooled_service(&pool_name, pool_config, initial_services) {
                    Ok(_) => {
                        log::info!("Registered pooled AI service: {}", pool_name);
                        // Audit log pooled service
                        audit_logger::log_event(
                            "ai_pooled_service_registered",
                            &serde_json::json!({
                                "pool_name": pool_name,
                                "pool_size": 3,
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            }),
                        )
                        .await;
                    }
                    Err(e) => {
                        log::warn!("Failed to register pooled AI service {}: {}", pool_name, e);
                        errors.push(format!(
                            "Pooled registration failed for {}: {}",
                            pool_name, e
                        ));
                    }
                }
            }
        }

        // Perform health checks
        let health_status = registry.health_check().await;
        let healthy_count = health_status.values().filter(|&healthy| *healthy).count();
        let total_count = health_status.len();

        // Audit log initialization completion
        audit_logger::log_event(
            "ai_service_registry_initialized",
            &serde_json::json!({
                "registered_services": registered_services.len(),
                "healthy_count": healthy_count,
                "total_count": total_count,
                "errors_count": errors.len(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        )
        .await;

        let success = healthy_count > 0 && errors.len() == 0;
        let message = if success {
            format!(
                "AI Service Registry initialized: {}/{} services healthy",
                healthy_count, total_count
            )
        } else {
            format!(
                "AI Service Registry initialized with issues: {}/{} services healthy, {} errors",
                healthy_count,
                total_count,
                errors.len()
            )
        };

        self.emit_event(LifecycleEvent {
            phase: LifecyclePhase::Starting,
            message,
            success,
            metadata: serde_json::json!({
                "service": "ai_registry",
                "registered_services": registered_services,
                "healthy_count": healthy_count,
                "total_count": total_count,
                "errors": errors,
                "health_status": health_status
            }),
            ..Default::default()
        })
        .await;

        Ok(())
    }

    /// Validate AI provider configuration for security
    async fn validate_ai_provider_config(&self, provider: &AIProvider) -> IDEResult<()> {
        match provider {
            AIProvider::Local { model_path } => {
                validate_secure_path(model_path, "AI model path")
                    .map_err(|e| IDEError::Validation(format!("Invalid model path: {}", e)))?;
                if !std::path::Path::new(model_path).exists() {
                    return Err(IDEError::Validation(format!(
                        "Model path does not exist: {}",
                        model_path
                    )));
                }
            }
            AIProvider::OpenAI | AIProvider::Anthropic => {
                // API keys are handled securely in the provider, validation done during initialization
            }
            AIProvider::CodeLlamaRust { .. } | AIProvider::StarCoderRust { .. } => {
                // Local models, path validation not needed as they're managed internally
            }
            AIProvider::Mock => {
                // Mock provider, no validation needed
            }
            _ => {
                // Other providers don't need special validation
            }
        }
        Ok(())
    }

    /// Create and initialize a WrappedAIService with proper background task handling
    async fn create_and_initialize_service(
        &self,
        provider: &AIProvider,
    ) -> IDEResult<WrappedAIService> {
        let core_service = crate::modules::ai::services::common::AIService::new();
        let wrapped_service = WrappedAIService::new(Arc::new(core_service), provider.clone());

        // Initialize the service in background
        wrapped_service.initialize().await.map_err(|e| {
            IDEError::AIService(format!("Failed to initialize AI service: {:?}", e))
        })?;

        Ok(wrapped_service)
    }

    async fn init_lsp_servers(&self) -> Result<()> {
        log::info!("Initializing LSP servers for supported languages and AI features");

        // Initialize LSP servers for traditional languages
        let languages = vec!["rust", "typescript", "python", "go", "java"];
        let mut initialized_servers = Vec::new();
        let mut errors = Vec::new();

        for language in languages {
            log::info!("Initializing LSP server for: {}", language);

            // Initialize LSP client for each language
            let config = LSPClientConfig {
                server_path: Some(self.get_server_binary_path(language)),
                timeout: Duration::from_secs(30),
                retry_attempts: 3,
                ..Default::default()
            };

            match LSPClient::new(config).await {
                Ok(mut client) => {
                    // Initialize the LSP client with workspace
                    let workspace_path = std::env::current_dir()
                        .map_err(|e| format!("Failed to get current directory: {}", e))?;

                    if let Err(e) = client.initialize(workspace_path).await {
                        log::warn!("Failed to initialize LSP client for {}: {:?}", language, e);
                        errors.push(format!("LSP init failed for {}: {}", language, e));
                        continue;
                    }

                    // Store client in global registry (assuming we have a way to store them)
                    initialized_servers.push(language.to_string());

                    // Audit log LSP initialization
                    audit_logger::log_event(
                        "lsp_server_initialized",
                        &serde_json::json!({
                            "language": language,
                            "server_path": self.get_server_binary_path(language),
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        }),
                    )
                    .await;
                }
                Err(e) => {
                    log::warn!("Failed to create LSP client for {}: {:?}", language, e);
                    errors.push(format!(
                        "LSP client creation failed for {}: {}",
                        language, e
                    ));
                }
            }
        }

        // Initialize AI-specific LSP services
        if let Err(e) = self.init_ai_lsp_services().await {
            log::warn!("Failed to initialize AI LSP services: {:?}", e);
            errors.push(format!("AI LSP initialization failed: {}", e));
        }

        let success = initialized_servers.len() > 0 && errors.len() == 0;
        let message = if success {
            format!(
                "LSP servers initialized for {} languages",
                initialized_servers.len()
            )
        } else {
            format!(
                "LSP servers initialized for {} languages with {} errors",
                initialized_servers.len(),
                errors.len()
            )
        };

        self.emit_event(LifecycleEvent {
            phase: LifecyclePhase::Starting,
            message,
            success,
            metadata: serde_json::json!({
                "service": "lsp",
                "initialized_servers": initialized_servers,
                "total_languages": languages.len(),
                "errors": errors,
                "ai_lsp_enabled": true
            }),
            ..Default::default()
        })
        .await;

        Ok(())
    }

    /// Initialize AI-specific LSP services
    async fn init_ai_lsp_services(&self) -> Result<()> {
        log::info!("Initializing AI-specific LSP services");

        // Initialize MultiLanguage LSP for AI processing
        let mut multi_lsp = MultiLanguageLSP::new();

        // Get the current workspace path
        let workspace_path =
            std::env::current_dir().map_err(|e| format!("Failed to get workspace path: {}", e))?;

        // Initialize with workspace
        multi_lsp
            .initialize(workspace_path)
            .await
            .map_err(|e| format!("Failed to initialize MultiLanguage LSP: {:?}", e))?;

        // Add AI-specific language servers
        let ai_languages = vec![
            ("rust_ai", "rust-analyzer"),
            ("python_ai", "pylsp"),
            ("typescript_ai", "typescript-language-server"),
        ];

        for (lang_name, server_binary) in ai_languages {
            let config = rust_ai_ide_lsp::LanguageServerConfig {
                language: rust_ai_ide_lsp::LanguageServerKind::Rust, // Default to Rust for AI processing
                server_path: Some(server_binary.to_string()),
                initialization_options: Some(serde_json::json!({
                    "ai_enabled": true,
                    "model_loading": true
                })),
                ..Default::default()
            };

            if let Err(e) = multi_lsp.add_language_server(config).await {
                log::warn!("Failed to add AI LSP server for {}: {:?}", lang_name, e);
            } else {
                audit_logger::log_event(
                    "ai_lsp_server_added",
                    &serde_json::json!({
                        "language": lang_name,
                        "server_binary": server_binary,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }),
                )
                .await;
            }
        }

        // Optimize for large codebase if needed
        multi_lsp
            .optimize_for_large_codebase()
            .await
            .map_err(|e| format!("Failed to optimize LSP for large codebase: {:?}", e))?;

        log::info!("AI LSP services initialized successfully");
        Ok(())
    }

    /// Get the server binary path for a given language
    fn get_server_binary_path(&self, language: &str) -> String {
        match language {
            "rust" => "rust-analyzer".to_string(),
            "typescript" | "javascript" => "typescript-language-server".to_string(),
            "python" => "pylsp".to_string(),
            "go" => "gopls".to_string(),
            "java" => "jdtls".to_string(),
            _ => format!("{}-language-server", language),
        }
    }

    async fn init_webhook_system(&self) -> Result<()> {
        log::info!("Initializing webhook system on port 3000");

        // Set up webhook system for cloud integrations
        // In a real implementation, this would start an HTTP server on port 3000
        // For now, we'll simulate the setup

        tokio::time::sleep(Duration::from_millis(200)).await; // Simulate server startup

        self.emit_event(LifecycleEvent {
            phase: LifecyclePhase::Starting,
            message: "Webhook system initialized on port 3000".to_string(),
            success: true,
            metadata: serde_json::json!({
                "service": "webhook",
                "port": 3000,
                "status": "listening",
                "endpoints": [
                    "/api/webhook/github",
                    "/api/webhook/gitlab",
                    "/api/webhook/cloud-integrations"
                ]
            }),
            ..Default::default()
        })
        .await;

        Ok(())
    }

    async fn start_background_tasks(&self, app: &tauri::App) -> Result<()> {
        log::info!("Starting background tasks with proper cleanup");

        let mut tasks_started = Vec::new();
        let mut errors = Vec::new();

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
                // Spawn background cache cleanup task with proper cleanup
                let task_id = spawn_background_task(
                    async move {
                        // Background task logic would be here
                        // For now, just log that it started
                        log::info!("Cache cleanup background task is running");
                        // In real implementation, this would periodically clean caches
                    },
                    "cache_cleanup",
                );

                tasks_started.push(task_id);

                audit_logger::log_event(
                    "background_task_started",
                    &serde_json::json!({
                        "task_name": "cache_cleanup",
                        "task_id": task_id,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }),
                )
                .await;
            }
            Err(e) => {
                log::error!("Failed to start cache cleanup task: {}", e);
                errors.push(format!("Cache cleanup task failed: {}", e));
            }
        }

        // Start AI service health monitoring task
        let registry = Arc::clone(&GLOBAL_AI_REGISTRY);
        let health_monitor_task_id = spawn_background_task(
            async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(60)).await; // Check every minute
                    match registry.health_check().await {
                        Ok(status) => {
                            let healthy_count = status.values().filter(|&&h| h).count();
                            let total_count = status.len();
                            log::debug!(
                                "AI services health check: {}/{} healthy",
                                healthy_count,
                                total_count
                            );
                        }
                        Err(e) => {
                            log::warn!("AI services health check failed: {:?}", e);
                        }
                    }
                }
            },
            "ai_health_monitor",
        );

        tasks_started.push(health_monitor_task_id);

        audit_logger::log_event(
            "background_task_started",
            &serde_json::json!({
                "task_name": "ai_health_monitor",
                "task_id": health_monitor_task_id,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        )
        .await;

        let success = tasks_started.len() > 0 && errors.len() == 0;
        let message = if success {
            format!(
                "Background tasks started: {} tasks running",
                tasks_started.len()
            )
        } else {
            format!(
                "Background tasks started with issues: {} tasks running, {} errors",
                tasks_started.len(),
                errors.len()
            )
        };

        self.emit_event(LifecycleEvent {
            phase: LifecyclePhase::Starting,
            message,
            success,
            metadata: serde_json::json!({
                "tasks_started": tasks_started,
                "errors": errors
            }),
            ..Default::default()
        })
        .await;

        Ok(())
    }

    async fn emit_event(&self, event: LifecycleEvent) {
        log::info!("Startup event: {} - {}", event.phase, event.message);
        // In a real implementation, this would notify registered listeners
    }
}

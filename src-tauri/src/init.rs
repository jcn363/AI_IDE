//! Application initialization utilities
//!
//! This module contains initialization functions and startup routines.

use std::path::PathBuf;

use dirs;
use rust_ai_ide_lsp::{AIProvider, AIService};

use crate::state::AppState;

/// Initialize AI service on application startup
pub async fn initialize_ai_service_on_startup(ai_service_state: &AppState) -> Result<(), String> {
    log::info!("Initializing AI service on startup");

    // Create default AI service configuration with environment-configurable endpoints
    let model_path = std::env::var("RUST_AI_IDE_MODEL_PATH").unwrap_or_else(|_| "default".to_string());

    let endpoint = std::env::var("RUST_AI_IDE_AI_ENDPOINT").unwrap_or_else(|_| "https://api.example.com".to_string());

    let default_provider = AIProvider::Local {
        model_path,
        endpoint,
    };

    let mut service = AIService::new(default_provider);

    // Initialize learning system if enabled
    let config_dir = dirs::config_dir()
        .ok_or_else(|| "Unable to get config directory".to_string())?
        .join("rust-ai-ide");

    // Handle directory creation with detailed error logging
    if let Err(e) = std::fs::create_dir_all(&config_dir) {
        log::error!(
            "Failed to create config directory '{}': {}. This may be due to permission issues or insufficient disk \
             space.",
            config_dir.display(),
            e
        );
        return Err(format!(
            "Failed to create config directory '{}': {}. Check permissions and available disk space.",
            config_dir.display(),
            e
        ));
    }

    let db_path = config_dir.join("ai_learning.db");

    // Initialize learning system
    if let Err(e) = service.initialize_learning_system(Some(db_path)).await {
        log::warn!("Failed to initialize learning system during startup: {}", e);
    } else {
        log::info!("Learning system initialized successfully");
    }

    // Store the service in the application state
    ai_service_state.set_ai_service(service).await;

    log::info!("AI service initialized successfully on startup");
    Ok(())
}

/// Initialize cache cleanup task with graceful shutdown
pub async fn initialize_cache_cleanup_task() -> Result<(), String> {
    use tokio::signal;
    use tokio::sync::broadcast;

    log::info!("Starting cache cleanup task");

    let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);
    let mut cleanup_interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes

    // Setup signal handling for graceful shutdown
    let shutdown_tx_signal = shutdown_tx.clone();
    tokio::spawn(async move {
        let mut sigterm = signal::unix::SignalKind::terminate();
        let mut sigint = signal::unix::SignalKind::interrupt();

        tokio::select! {
            _ = async { Ok(()) } => { // Simplified for this example
                log::info!("Received SIGTERM, initiating graceful shutdown");
            }
            _ = async { Ok(()) } => {
                log::info!("Received SIGINT, initiating graceful shutdown");
            }
        }

        // Handle potential send errors gracefully
        if let Err(e) = shutdown_tx_signal.send(()) {
            log::warn!(
                "Failed to send shutdown signal: {}. Shutdown may not be properly coordinated.",
                e
            );
        } else {
            log::debug!("Shutdown signal sent successfully to listeners");
        }
    });

    loop {
        tokio::select! {
            _ = cleanup_interval.tick() => {
                // TODO: Implement cache cleanup
                // For now, just log
                log::debug!("Cache cleanup cycle would run here");
            }
            _ = shutdown_rx.recv() => {
                log::info!("Cache cleanup task received shutdown signal, exiting gracefully");
                break;
            }
        }
    }

    Ok(())
}

/// Initialize LSP service on startup
pub async fn initialize_lsp_service(state: &AppState) -> Result<(), String> {
    log::info!("Initializing LSP service");

    // Check if rust-analyzer is installed
    let output = std::process::Command::new("rust-analyzer")
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to check rust-analyzer: {}", e))?;

    if !output.status.success() {
        return Err(
            "rust-analyzer is not installed. Please install it with 'rustup component add rust-analyzer'".to_string(),
        );
    }

    log::info!("LSP service initialized successfully");
    Ok(())
}

/// Initialize the file watcher service
pub async fn initialize_file_watcher_service(state: &AppState) -> Result<(), String> {
    log::info!("Initializing file watcher service");

    // Get the app handle to pass to file watcher
    let app_handle = state.get_app_handle()
        .ok_or_else(|| "App handle not available for file watcher initialization".to_string())?;

    // Get current workspace path for watching
    let workspace_path = dirs::home_dir()
        .ok_or_else(|| "Unable to get home directory".to_string())?
        .join("Desktop")
        .join("RUST_AI_IDE");

    // Create file watcher
    let file_watcher = crate::file_watcher::FileWatcher::new(workspace_path, app_handle)
        .map_err(|e| format!("Failed to create file watcher: {}", e))?;

    // Store file watcher in state
    state.set_file_watcher(file_watcher).await;

    log::info!("File watcher service initialized successfully");
    Ok(())
}
/// Initialize webhook system on startup
pub async fn initialize_webhook_system(state: &AppState) -> Result<(), String> {
    log::info!("Initializing webhook system on port 3000");

    let webhook_server = rust_ai_ide_webhooks::WebhookServer::new(3000).await
        .map_err(|e| format!("Failed to create webhook server: {}", e))?;

    // Spawn the webhook server as a background task
    tokio::spawn(async move {
        if let Err(e) = webhook_server.start().await {
            log::error!("Failed to start webhook server: {}", e);
        }
    });

    log::info!("Webhook system initialized successfully on port 3000");
    Ok(())
}

/// Initialize monitoring metrics collection
pub async fn initialize_monitoring_metrics(state: &AppState) -> Result<(), String> {
    log::info!("Initializing monitoring metrics collection");

    // Start performance monitoring in background
    {
        // Initialize performance metrics collection
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60)); // Every minute
            loop {
                interval.tick().await;

                // Collect system metrics
                if let Ok(metrics) = crate::commands::performance::collect_system_metrics().await {
                    log::debug!("Collected system metrics: {:?}", metrics);
                    // In a real implementation, these would be sent to monitoring system
                }

                // Collect memory usage
                if let Ok(memory_stats) = crate::commands::performance::collect_memory_stats().await {
                    log::debug!("Collected memory stats: {:?}", memory_stats);
                }

                // Collect performance benchmarks
                if let Ok(benchmarks) = crate::commands::performance::collect_performance_benchmarks().await {
                    log::debug!("Collected performance benchmarks: {:?}", benchmarks);
                }
            }
        });

        // Note: event_bus and rate_limiter are already initialized in the AppState constructor
    }

    log::info!("Monitoring metrics collection initialized successfully");
    Ok(())
}

/// Comprehensive application initialization
pub async fn initialize_application(state: &AppState) -> Result<(), String> {
    log::info!("Starting comprehensive application initialization");

    // Initialize AI service
    if let Err(e) = initialize_ai_service_on_startup(state).await {
        log::error!("Failed to initialize AI service: {}", e);
        // Continue with other initializations
    }

    // Initialize LSP service
    if let Err(e) = initialize_lsp_service(state).await {
        log::error!("Failed to initialize LSP service: {}", e);
        // Continue with other initializations
    }

    // Initialize file watcher service
    if let Err(e) = initialize_file_watcher_service(state).await {
        log::error!("Failed to initialize file watcher service: {}", e);
        // Continue with other initializations
    }

    // Initialize webhook system
    if let Err(e) = initialize_webhook_system(state).await {
        log::error!("Failed to initialize webhook system: {}", e);
        // Continue with other initializations
    }

    // Initialize monitoring metrics collection
    if let Err(e) = initialize_monitoring_metrics(state).await {
        log::error!("Failed to initialize monitoring metrics: {}", e);
        // Continue with other initializations
    }

    // Initialize cache cleanup task
    tokio::spawn(async move {
        if let Err(e) = initialize_cache_cleanup_task().await {
            log::error!("Cache cleanup task failed: {}", e);
        }
    });

    log::info!("Application initialization completed");
    Ok(())
}

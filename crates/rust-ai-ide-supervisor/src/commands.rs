//! Tauri command integration for supervisor operations

use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

use crate::{
    error::SupervisorResult, ipc_recovery::IpcMonitor, service_supervisor::Supervisor,
    state_persistence::StatePersistence, types::*,
};
use rust_ai_ide_common::validation::TauriInputSanitizer;

/// Shared state for Tauri commands
pub struct SupervisorState {
    pub supervisor: Arc<Mutex<Option<Supervisor>>>,
    pub persistence: Arc<Mutex<Option<StatePersistence>>>,
    pub ipc_monitor: Arc<Mutex<Option<IpcMonitor>>>,
}

/// Initialize supervisor system
#[tauri::command]
pub async fn init_supervisor(
    state: State<'_, Arc<Mutex<SupervisorState>>>,
    config_path: String,
) -> Result<String, String> {
    let config_path = TauriInputSanitizer::sanitize_path(&config_path)
        .map_err(|e| format!("Path sanitization failed: {:?}", e))?;

    let mut guard = state.lock().await;

    // Initialize persistence layer
    let persistence = StatePersistence::new(
        &format!("{}/supervisor.db", config_path),
        &format!("{}/checkpoints", config_path),
    )
    .await
    .map_err(|e| format!("Failed to initialize persistence: {:?}", e))?;

    guard.persistence = Some(persistence);

    // Initialize IPC monitor
    guard.ipc_monitor = Some(IpcMonitor::new());

    // Initialize supervisor (requires persistence to be available)
    let supervisor =
        Supervisor::new().map_err(|e| format!("Failed to create supervisor: {:?}", e))?;
    guard.supervisor = Some(supervisor);

    Ok(serde_json::json!({"status": "initialized"}).to_string())
}

/// Register a service for monitoring
#[tauri::command]
pub async fn register_service(
    state: State<'_, Arc<Mutex<SupervisorState>>>,
    service_id: String,
    service_name: String,
    command: String,
    args: Vec<String>,
    restart_policy: String,
) -> Result<String, String> {
    let mut guard = state.lock().await;
    let supervisor = guard
        .supervisor
        .as_mut()
        .ok_or("Supervisor not initialized")?;

    let service_config = ServiceConfig {
        id: service_id,
        name: service_name,
        command: TauriInputSanitizer::sanitize_path(&command).map_err(|e| format!("{:?}", e))?,
        args: args
            .into_iter()
            .map(|arg| TauriInputSanitizer::sanitize_string(&arg))
            .collect::<Result<_, _>>()
            .map_err(|e| format!("{:?}", e))?,
        working_dir: None,
        environment: std::env::vars().collect(),
        health_check_timeout: std::time::Duration::from_secs(30),
        restart_policy: match restart_policy.as_str() {
            "Never" => RestartPolicy::Never,
            "Always" => RestartPolicy::Always,
            "ExponentialBackoff" => RestartPolicy::ExponentialBackoff {
                base_delay: std::time::Duration::from_secs(1),
                max_delay: std::time::Duration::from_secs(60),
                max_attempts: 5,
            },
            "FixedDelay" => RestartPolicy::FixedDelay {
                delay: std::time::Duration::from_secs(5),
                max_attempts: 3,
            },
            _ => RestartPolicy::Never,
        },
        shutdown_timeout: std::time::Duration::from_secs(10),
        critical: false,
    };

    supervisor
        .register_service(service_config)
        .await
        .map_err(|e| format!("Failed to register service: {:?}", e))?;

    Ok(serde_json::json!({"status": "registered"}).to_string())
}

/// Start monitoring all services
#[tauri::command]
pub async fn start_supervisor_monitoring(
    state: State<'_, Arc<Mutex<SupervisorState>>>,
) -> Result<String, String> {
    let mut guard = state.lock().await;
    let supervisor = guard
        .supervisor
        .as_mut()
        .ok_or("Supervisor not initialized")?;

    supervisor
        .start_monitoring()
        .await
        .map_err(|e| format!("Failed to start monitoring: {:?}", e))?;

    if let Some(ipc_monitor) = &mut guard.ipc_monitor {
        ipc_monitor
            .start_monitoring()
            .await
            .map_err(|e| format!("Failed to start IPC monitoring: {:?}", e))?;
    }

    Ok(serde_json::json!({"status": "monitoring_started"}).to_string())
}

/// Get supervisor health status
#[tauri::command]
pub async fn get_supervisor_health(
    state: State<'_, Arc<Mutex<SupervisorState>>>,
) -> Result<String, String> {
    let guard = state.lock().await;
    let supervisor = guard
        .supervisor
        .as_ref()
        .ok_or("Supervisor not initialized")?;

    let stats = supervisor
        .get_supervisor_health()
        .await
        .map_err(|e| format!("Failed to get health: {:?}", e))?;

    serde_json::to_string(&stats).map_err(|e| format!("Failed to serialize health data: {:?}", e))
}

/// Create a checkpoint
#[tauri::command]
pub async fn create_checkpoint(
    state: State<'_, Arc<Mutex<SupervisorState>>>,
) -> Result<String, String> {
    let mut guard = state.lock().await;
    let persistence = guard
        .persistence
        .as_ref()
        .ok_or("Persistence not initialized")?;
    let supervisor = guard
        .supervisor
        .as_ref()
        .ok_or("Supervisor not initialized")?;

    // Get current service states
    let services = HashMap::new(); // This would be populated from actual service states
    let operations = vec![]; // This would be populated from pending operations

    let checkpoint_id = persistence
        .create_checkpoint(&services, &operations)
        .await
        .map_err(|e| format!("Failed to create checkpoint: {:?}", e))?;

    Ok(serde_json::json!({
        "status": "checkpoint_created",
        "checkpoint_id": checkpoint_id.to_string()
    })
    .to_string())
}

/// Load latest checkpoint
#[tauri::command]
pub async fn load_checkpoint(
    state: State<'_, Arc<Mutex<SupervisorState>>>,
) -> Result<String, String> {
    let mut guard = state.lock().await;
    let persistence = guard
        .persistence
        .as_ref()
        .ok_or("Persistence not initialized")?;

    let snapshot = persistence
        .load_latest_checkpoint()
        .await
        .map_err(|e| format!("Failed to load checkpoint: {:?}", e))?;

    serde_json::to_string(&snapshot).map_err(|e| format!("Failed to serialize snapshot: {:?}", e))
}

/// Get database statistics
#[tauri::command]
pub async fn get_database_stats(
    state: State<'_, Arc<Mutex<SupervisorState>>>,
) -> Result<String, String> {
    let guard = state.lock().await;
    let persistence = guard
        .persistence
        .as_ref()
        .ok_or("Persistence not initialized")?;

    let stats = persistence
        .get_statistics()
        .await
        .map_err(|e| format!("Failed to get statistics: {:?}", e))?;

    serde_json::to_string(&stats).map_err(|e| format!("Failed to serialize statistics: {:?}", e))
}

/// Stop supervisor system
#[tauri::command]
pub async fn stop_supervisor(
    state: State<'_, Arc<Mutex<SupervisorState>>>,
) -> Result<String, String> {
    let mut guard = state.lock().await;

    if let Some(supervisor) = &guard.supervisor {
        // Stop all services gracefully
        // This would iterate through all services and stop them
    }

    guard.supervisor = None;
    guard.persistence = None;
    guard.ipc_monitor = None;

    Ok(serde_json::json!({"status": "stopped"}).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    async fn create_test_state() -> Arc<Mutex<SupervisorState>> {
        Arc::new(Mutex::new(SupervisorState {
            supervisor: Arc::new(Mutex::new(None)),
            persistence: Arc::new(Mutex::new(None)),
            ipc_monitor: Arc::new(Mutex::new(None)),
        }))
    }

    #[tokio::test]
    async fn test_supervisor_state_creation() {
        let state = create_test_state().await;
        assert!(state.lock().await.supervisor.lock().await.is_none());
    }
}

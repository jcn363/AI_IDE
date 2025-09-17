//! Tauri commands for orchestration layer management

use tauri::{AppHandle, State};

use crate::error::OrchestrationError;
use crate::orchestrator::ServiceOrchestrator;
use crate::types::*;

// Tauri command for initializing the orchestrator
#[tauri::command]
pub async fn init_orchestrator(
    orchestrator: State<'_, ServiceOrchestrator>,
) -> Result<String, String> {
    orchestrator
        .initialize()
        .await
        .map(|_| serde_json::json!({"status": "ok"}).to_string())
        .map_err(|e| e.to_string())
}

// Tauri command for getting orchestrator status
#[tauri::command]
pub async fn get_orchestrator_status(
    orchestrator: State<'_, ServiceOrchestrator>,
) -> Result<String, String> {
    let status = orchestrator.get_status().await;
    Ok(serde_json::to_string(&status).unwrap_or_else(|_| "Serialization failed".to_string()))
}

// Tauri command to register a service
#[tauri::command]
pub async fn register_orchestration_service(
    orchestrator: State<'_, ServiceOrchestrator>,
    registration_json: String,
) -> Result<String, String> {
    let registration: ServiceRegistration = serde_json::from_str(&registration_json)
        .map_err(|e| format!("Invalid registration: {}", e))?;

    orchestrator
        .register_service(registration)
        .await
        .map(|_| serde_json::json!({"status": "ok"}).to_string())
        .map_err(|e| e.to_string())
}

// Tauri command to list services
#[tauri::command]
pub async fn list_orchestration_services(
    orchestrator: State<'_, ServiceOrchestrator>,
) -> Result<String, String> {
    let services = orchestrator.service_registry().list_services().await;
    Ok(serde_json::to_string(&services).unwrap_or_else(|_| "Serialization failed".to_string()))
}

/// Placeholder command implementation as per existing patterns
pub fn placeholder_orchestration_command() -> serde_json::Value {
    serde_json::json!({"status": "ok"})
}

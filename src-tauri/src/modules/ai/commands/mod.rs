//! AI commands module for the Rust AI IDE
//!
//! This module provides a centralized aggregation point for all AI-related
//! command functionality, organized into specialized submodules for better
//! maintainability and separation of concerns.
//!
//! ## Submodules
//!
//! - `completion`: Code completion and generation commands
//! - `analysis`: Code analysis, documentation, and refactoring commands
//! - `models`: AI model interaction and messaging commands
//! - `training`: Model training and fine-tuning commands
//! - `registry`: AI service registration and discovery commands
//!
//! ## Usage
//!
//! Functions from each submodule are re-exported at this level for convenience,
//! allowing consumers to import all AI commands with a single import:
//!
//! ```rust
//! use rust_ai_ide::modules::ai::commands::*;
//! ```
//!
//! Or access specific functionality:
//!
//! ```rust
//! use rust_ai_ide::modules::ai::commands::completion::*;
//! ```
//!
//! ## Architecture
//!
//! The module follows these design principles:
//!
//! - Each submodule handles a specific domain of AI functionality
//! - Error handling follows the IDE's error handling patterns using `Result<String, String>`
//! - Input validation is performed at command boundaries
//! - State management uses Tauri state management patterns
//! - Async operations follow Rust's async patterns with proper error propagation

// Module declarations for specialized AI functionality
pub mod analysis;
pub mod completion;
pub mod models;
pub mod registry;
pub mod training;

// Re-exports for convenience (users can import from this module)
pub use analysis::*;
pub use completion::*;
pub use models::*;
pub use registry::*;
pub use training::*;

// Common imports used across submodules
use crate::commands::ai::services::AIServiceState;
use crate::errors::IDEError;
use crate::utils;

/// Get AI system health status
///
/// Provides an overview of the AI system components status.
///
/// # Returns
/// * `Result<AISystemHealth, String>` - Overall system health information
///
/// # Errors
/// Returns error if AI system status cannot be determined
#[tauri::command]
pub async fn ai_system_health(
    ai_service_state: tauri::State<'_, AIServiceState>,
) -> Result<AISystemHealth, String> {
    log::info!("Checking AI system health");

    let _ai_service = match utils::get_or_create_ai_service(&ai_service_state).await {
        Ok(service) => {
            log::debug!("AI service is available");
            service
        }
        Err(e) => {
            log::error!("AI service unavailable: {}", e);
            return Ok(AISystemHealth {
                overall_status: SystemStatus::Unhealthy,
                service_available: false,
                registry_size: 0,
                active_jobs: 0,
                last_error: Some(e),
            });
        }
    };

    // In a real implementation, this would check actual system components
    Ok(AISystemHealth {
        overall_status: SystemStatus::Healthy,
        service_available: true,
        registry_size: 5, // Placeholder
        active_jobs: 2,   // Placeholder
        last_error: None,
    })
}

/// AI system health information
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AISystemHealth {
    pub overall_status: SystemStatus,
    pub service_available: bool,
    pub registry_size: usize,
    pub active_jobs: usize,
    pub last_error: Option<String>,
}

/// System status indicator
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum SystemStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

// Additional utility functions can be added here as needed

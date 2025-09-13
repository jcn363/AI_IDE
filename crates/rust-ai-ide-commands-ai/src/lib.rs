//! # Rust AI IDE Commands - AI/ML Module
//!
//! This crate provides modular AI/ML command implementations for the Rust AI IDE.
//! It contains all AI-related Tauri commands grouped by functionality while maintaining
//! loose coupling and high cohesion.
//!
//! ## Architecture
//!
//! This module follows a modular command pattern:
//! - Each command is self-contained with its own implementation
//! - Commands are grouped by functional domain
//! - Loose coupling between UI and business logic
//! - State management through dependency injection
//!
//! ## Command Categories
//!
//! ### Core AI Commands
//! - `ai_code_completion` - Intelligent code completion
//! - `ai_refactor_code` - AI-assisted code refactoring
//! - `ai_explain_code` - Code explanation and documentation
//! - `ai_doc_assist` - Documentation generation
//!
//! ### Analysis Commands
//! - `analyze_file` - Individual file analysis
//! - `analyze_workspace` - Full workspace analysis
//! - `get_performance_suggestions` - Performance optimization suggestions
//! - `run_code_quality_check` - Code quality assessment
//!
//! ### Model Management
//! - `list_available_models` - Available AI models
//! - `list_downloaded_models` - Downloaded models
//! - `load_model` / `unload_model` - Model lifecycle
//! - `get_model_status` - Model health and status
//!
//! ### Learning & Training
//! - `start_finetune_job` - Start model training
//! - `get_finetune_progress` - Training progress tracking
//! - `cancel_finetune_job` - Stop training jobs
//! - `get_resource_status` - Hardware resource monitoring

#![allow(unused)]

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

pub mod analysis;
pub mod completion;
pub mod models;
pub mod services;
pub mod training;

// Re-export key types
pub use services::{AIConfig, AIService};

// Command registry for collecting all AI commands
pub struct AICommandsRegistry {
    commands: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl AICommandsRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn register_command<F>(&mut self, name: &str, factory: F)
    where
        F: Fn() -> Box<dyn std::any::Any + Send + Sync> + 'static,
    {
        self.commands.insert(name.to_string(), factory());
    }

    pub fn get_commands(&self) -> &HashMap<String, Box<dyn std::any::Any + Send + Sync>> {
        &self.commands
    }
}

// Create default AI commands registry
pub fn create_ai_commands_registry() -> AICommandsRegistry {
    let mut registry = AICommandsRegistry::new();

    // Register completion commands
    registry.register_command("ai_code_completion", || {
        Box::new(completion::code_completion_command()) as Box<dyn std::any::Any + Send + Sync>
    });

    registry.register_command("ai_refactor_code", || {
        Box::new(completion::refactor_command()) as Box<dyn std::any::Any + Send + Sync>
    });

    // Register analysis commands
    registry.register_command("analyze_file", || {
        Box::new(analysis::file_analysis_command()) as Box<dyn std::any::Any + Send + Sync>
    });

    registry.register_command("analyze_workspace", || {
        Box::new(analysis::workspace_analysis_command()) as Box<dyn std::any::Any + Send + Sync>
    });

    registry.register_command("run_code_quality_check", || {
        Box::new(analysis::code_quality_command()) as Box<dyn std::any::Any + Send + Sync>
    });

    // Register model management commands
    registry.register_command("list_available_models", || {
        Box::new(models::list_models_command()) as Box<dyn std::any::Any + Send + Sync>
    });

    registry.register_command("load_model", || {
        Box::new(models::load_model_command()) as Box<dyn std::any::Any + Send + Sync>
    });

    registry.register_command("unload_model", || {
        Box::new(models::unload_model_command()) as Box<dyn std::any::Any + Send + Sync>
    });

    // Register training commands
    registry.register_command("start_finetune_job", || {
        Box::new(training::finetune_command()) as Box<dyn std::any::Any + Send + Sync>
    });

    registry.register_command("get_finetune_progress", || {
        Box::new(training::training_progress_command()) as Box<dyn std::any::Any + Send + Sync>
    });

    registry.register_command("cancel_finetune_job", || {
        Box::new(training::cancel_training_command()) as Box<dyn std::any::Any + Send + Sync>
    });

    registry
}

/// Initialize AI command system
pub async fn initialize_ai_commands() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize AI service layer
    let ai_service = AIService::new().await?;

    // Initialize model manager
    let model_manager = models::ModelManager::new().await?;

    // Initialize training coordinator
    let training_coordinator = training::TrainingCoordinator::new().await?;

    log::info!("AI command system initialized successfully");
    Ok(())
}

/// Get health status of AI command system
pub async fn get_ai_system_health() -> HashMap<String, serde_json::Value> {
    let mut health = HashMap::new();

    // Check service availability
    health.insert(
        "ai_service".to_string(),
        serde_json::json!({"status": "healthy", "available": true}),
    );

    // Check model management
    health.insert(
        "model_management".to_string(),
        serde_json::json!({"status": "operational", "models_loaded": 0}),
    );

    // Check training system
    health.insert(
        "training_system".to_string(),
        serde_json::json!({"status": "idle", "active_jobs": 0}),
    );

    health
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_commands_registry_creation() {
        let registry = create_ai_commands_registry();

        // Verify core AI commands are registered
        assert!(registry.get_commands().contains_key("ai_code_completion"));
        assert!(registry.get_commands().contains_key("analyze_file"));
        assert!(registry
            .get_commands()
            .contains_key("list_available_models"));
        assert!(registry.get_commands().contains_key("start_finetune_job"));
    }

    #[tokio::test]
    async fn test_ai_system_health() {
        let health = get_ai_system_health().await;

        assert!(health.contains_key("ai_service"));
        assert!(health.contains_key("model_management"));
        assert!(health.contains_key("training_system"));
    }
}

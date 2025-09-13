//! Main AI commands module
//!
//! This module serves as the main entry point for AI-related functionality,
//! organizing commands into focused submodules and providing delegation bridges
//! to the rust-ai-ide-commands-ai crate.

pub mod analysis;
pub mod learning;
pub mod services;

use crate::commands::ai::services::AIServiceState;
use crate::utils;
use std::{collections::HashMap, sync::Arc};

// Import delegations to rust-ai-ide-commands-ai for better implementations
use rust_ai_ide_commands_ai::{analysis::AnalysisService, models::ModelManager};

// Type aliases for state management bridge
/// Bridge state that holds both the original AIServiceState (Mutex) and new RwLock-based services
#[derive(Default)]
pub struct AIStateBridge {
    /// Original Mutex-based AI service state (for compatibility)
    pub original_state: AIServiceState,
    /// Commands-ai ModelManager (RwLock-based)
    pub model_manager: Option<Arc<ModelManager>>,
    /// Commands-ai AnalysisService (RwLock-based)
    pub analysis_service: Option<Arc<AnalysisService>>,
}

impl AIStateBridge {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut bridge = Self {
            original_state: Default::default(),
            model_manager: None,
            analysis_service: None,
        };

        // Initialize commands-ai services
        if let Ok(model_mgr) = ModelManager::new().await {
            bridge.model_manager = Some(Arc::new(model_mgr));
        }

        // Create analysis service (requires an AI service instance)
        if let Some(model_mgr) = &bridge.model_manager {
            let ai_service = Arc::new(tokio::sync::RwLock::new(
                rust_ai_ide_commands_ai::services::AIService::new().await?,
            ));

            if let Ok(analysis_svc) = AnalysisService::new(ai_service).await {
                bridge.analysis_service = Some(Arc::new(analysis_svc));
            }
        }

        Ok(bridge)
    }

    /// Get ModelManager reference, initializing if needed
    pub async fn model_manager(&mut self) -> Result<Arc<ModelManager>, String> {
        if self.model_manager.is_none() {
            let manager = ModelManager::new()
                .await
                .map_err(|e| format!("Failed to initialize ModelManager: {}", e))?;
            self.model_manager = Some(Arc::new(manager));
        }

        Ok(self.model_manager.as_ref().unwrap().clone())
    }

    /// Get AnalysisService reference, initializing if needed
    pub async fn analysis_service(&mut self) -> Result<Arc<AnalysisService>, String> {
        if self.analysis_service.is_none() {
            // Ensure we have a model manager for analysis service
            let model_mgr = self.model_manager().await?;
            let ai_service = Arc::new(tokio::sync::RwLock::new(
                rust_ai_ide_commands_ai::services::AIService::new()
                    .await
                    .map_err(|e| format!("Failed to create AI service: {}", e))?,
            ));

            let analysis_svc = AnalysisService::new(ai_service)
                .await
                .map_err(|e| format!("Failed to initialize AnalysisService: {}", e))?;
            self.analysis_service = Some(Arc::new(analysis_svc));
        }

        Ok(self.analysis_service.as_ref().unwrap().clone())
    }
}

/// Type alias for clarity - the bridged state used in Tauri commands
pub type AIBridgeState = Arc<tokio::sync::Mutex<AIStateBridge>>;

// send_ai_message defined in modules/ai/commands/mod.rs

// ai_code_completion defined in modules/ai/commands/mod.rs

// AI command functions are defined in submodules:
// - ai_generate_code, ai_doc_assist, ai_refactor_code, ai_explain_code, ai_context_help
//   are defined in modules/ai/commands/mod.rs
// - Additional commands are imported from analysis, diagnostics, etc.

// Re-export commands from submodules for use in main lib.rs

// Core AI commands from this module - commented out due to missing implementations
// pub use send_ai_message;
// pub use ai_code_completion;
// pub use ai_generate_code;
// pub use ai_doc_assist;
// pub use ai_refactor_code;
// pub use ai_explain_code;
// pub use ai_context_help;

// AI service management commands
pub use services::{
    download_model, get_ai_config, get_loaded_models, get_model_status, get_resource_status,
    initialize_ai_service, load_model, unload_model, update_ai_config, validate_model_config,
};

// Fine-tuning commands
pub use services::finetune::{
    cancel_finetune_job, get_finetune_progress, list_finetune_jobs, prepare_dataset,
    start_finetune_job,
};

// Code analysis commands
pub use analysis::{analyze_file, apply_ai_suggestion, get_performance_suggestions};

// Diagnostics commands
pub use analysis::diagnostics::{
    explain_error_code, get_compiler_diagnostics, resolve_errors_with_ai,
};

// Learning system commands
pub use learning::{
    analyze_learning_patterns, apply_learned_pattern, get_learned_patterns,
    get_learning_statistics, get_learning_system_health, record_successful_fix,
    update_learning_preferences,
};

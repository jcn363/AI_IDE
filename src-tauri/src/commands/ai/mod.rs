//! Main AI commands module
//!
//! This module serves as the main entry point for AI-related functionality,
//! organizing commands into focused submodules.

pub mod services;
pub mod analysis;
pub mod learning;

use std::{collections::HashMap};
use crate::commands::ai::services::AIServiceState;
use crate::utils;

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
    initialize_ai_service,
    get_ai_config,
    update_ai_config,
    get_loaded_models,
    load_model,
    unload_model,
    get_model_status,
    get_resource_status,
    validate_model_config,
    download_model,
};

// Fine-tuning commands
pub use services::finetune::{
    start_finetune_job,
    get_finetune_progress,
    cancel_finetune_job,
    list_finetune_jobs,
    prepare_dataset,
};

// Code analysis commands
pub use analysis::{
    analyze_file,
    get_performance_suggestions,
    apply_ai_suggestion,
};

// Diagnostics commands
pub use analysis::diagnostics::{
    get_compiler_diagnostics,
    resolve_errors_with_ai,
    explain_error_code,
};

// Learning system commands
pub use learning::{
    record_successful_fix,
    get_learned_patterns,
    update_learning_preferences,
    get_learning_statistics,
    analyze_learning_patterns,
    apply_learned_pattern,
    get_learning_system_health,
};
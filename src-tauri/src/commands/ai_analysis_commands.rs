//! Core AI analysis commands module
//!
//! This module provides the main entry point for AI analysis functionality.
//! All core analysis features have been moved to focused submodules:
//!
//! - `services/` - AI service management and fine-tuning
//! - `analysis/` - Code analysis and diagnostics
//! - `learning/` - Learning system functionality

// Re-export all AI commands from the organized submodules

// Service management commands
pub use crate::commands::ai::services::*;
pub use crate::commands::ai::services::finetune::*;

// Analysis commands
pub use crate::commands::ai::analysis::*;

// Diagnostics commands
pub use crate::commands::ai::analysis::diagnostics::*;

// Learning commands
pub use crate::commands::ai::learning::*;

// Core types that may be needed by frontend
pub use crate::commands::ai::services::{
    AIServiceState,
    AIAnalysisConfig,
    LearningPreferences,
    CompilerIntegrationConfig,
};

// Legacy exports for backward compatibility
pub use crate::commands::ai::{
    send_ai_message,
    ai_code_completion,
    ai_generate_code,
    ai_doc_assist,
    ai_refactor_code,
    ai_explain_code,
    ai_context_help,
};

// Core command structure for workspace analysis (placeholder implementation)
// TODO: Move full implementation from original file when complete

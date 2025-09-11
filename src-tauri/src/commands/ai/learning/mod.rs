//! Learning system operations for AI-powered code analysis
//!
//! This module manages the learning system functionality including pattern
//! recognition, successful fix recording, learning preferences, and statistics.

use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::State;

use rust_ai_ide_lsp::{AIService, ErrorPattern, FixSuggestion, LearnedPattern, LearningSystemData};

/// AI service state (re-exported for convenience)
pub type AIServiceState = Arc<tokio::sync::Mutex<Option<AIService>>>;

/// Learning system request
#[derive(Debug, Deserialize)]
pub struct LearningSystemRequest {
    pub error_pattern: ErrorPattern,
    pub applied_fix: FixSuggestion,
    pub success: bool,
    pub user_feedback: Option<String>, // "positive", "negative", "neutral"
    pub context: String,
}

/// Update learning preferences request
#[derive(Debug, Deserialize)]
pub struct UpdateLearningPreferencesRequest {
    pub preferences: crate::commands::ai::services::LearningPreferences,
}

/// Record successful fix for learning
#[tauri::command]
pub async fn record_successful_fix(
    request: LearningSystemRequest,
    ai_service: State<'_, AIServiceState>,
) -> Result<String, String> {
    log::info!("Recording successful fix for learning");

    acquire_service_and_execute!(ai_service, AIServiceState, {
        service
            .record_successful_fix(request.error_pattern, request.applied_fix)
            .await
            .map_err(|e| format!("Failed to record fix: {}", e))?;

        Ok("Fix recorded successfully for learning".to_string())
    })
}

/// Get learned patterns for similar errors
#[tauri::command]
pub async fn get_learned_patterns(
    error_context: String,
    ai_service: State<'_, AIServiceState>,
) -> Result<Vec<LearnedPattern>, String> {
    log::info!("Getting learned patterns for error context");

    acquire_service_and_execute!(ai_service, AIServiceState, {
        service
            .get_learned_patterns(&error_context)
            .await
            .map_err(|e| format!("Failed to get learned patterns: {}", e))
    })
}

/// Update learning preferences
#[tauri::command]
pub async fn update_learning_preferences(
    request: UpdateLearningPreferencesRequest,
) -> Result<String, String> {
    log::info!("Updating learning preferences");

    execute_command!(
        "update_learning_preferences",
        &CommandConfig::default(),
        async move || {
            // In a real implementation, this would update the learning system configuration
            // and persist the preferences to storage
            Ok("Learning preferences updated successfully".to_string())
        }
    )
}

/// Get learning system statistics
#[tauri::command]
pub async fn get_learning_statistics(
    ai_service: State<'_, AIServiceState>,
) -> Result<LearningSystemData, String> {
    log::info!("Getting learning statistics");

    execute_command!(
        "get_learning_statistics",
        &CommandConfig::default(),
        async move || {
            acquire_service_and_execute!(ai_service, AIServiceState, {
                // In a real implementation, this would query the learning system for statistics
                let stats = LearningSystemData {
                    learned_patterns: Vec::new(),
                    user_preferences: crate::commands::ai::services::LearningPreferences::default(),
                    statistics: rust_ai_ide_lsp::learning::LearningStatistics {
                        total_patterns_learned: 0,
                        successful_fixes_applied: 0,
                        average_confidence_score: 0.0,
                        last_updated: chrono::Utc::now(),
                    },
                };
                Ok(stats)
            })
        }
    )
}

/// Re-export learning-related types
pub use crate::commands::ai::services::{LearningPreferences, PrivacyMode};

/// Helper structures for learning system

/// Learning statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStatistics {
    pub total_patterns_learned: usize,
    pub successful_fixes_applied: usize,
    pub average_confidence_score: f32,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Learning pattern analysis request
#[derive(Debug, Deserialize)]
pub struct PatternAnalysisRequest {
    pub error_description: String,
    pub code_context: String,
    pub existing_patterns: Vec<LearnedPattern>,
}

/// Analyze patterns from learning data
#[tauri::command]
pub async fn analyze_learning_patterns(
    request: PatternAnalysisRequest,
    ai_service: State<'_, AIServiceState>,
) -> Result<Vec<LearnedPattern>, String> {
    log::info!("Analyzing learning patterns");

    acquire_service_and_execute!(ai_service, AIServiceState, {
        // In a real implementation, this would analyze existing patterns and generate new insights
        // For now, return the existing patterns
        Ok(request.existing_patterns)
    })
}

/// Apply learned pattern to new code
#[tauri::command]
pub async fn apply_learned_pattern(
    pattern_id: String,
    code_context: String,
    ai_service: State<'_, AIServiceState>,
) -> Result<FixSuggestion, String> {
    log::info!("Applying learned pattern: {}", pattern_id);

    acquire_service_and_execute!(ai_service, AIServiceState, {
        // Get the pattern from learning system
        let patterns = service
            .get_learned_patterns(&code_context)
            .await
            .map_err(|e| format!("Failed to get patterns: {}", e))?;

        // Find the pattern by ID
        let pattern = patterns
            .into_iter()
            .find(|p| p.id.to_string() == pattern_id)
            .ok_or_else(|| format!("Pattern {} not found", pattern_id))?;

        // Return the successful fix from the pattern
        Ok(rust_ai_ide_lsp::FixSuggestion {
            id: format!("applied_{}", pattern_id),
            title: pattern.successful_fix.title,
            description: format!(
                "Applied learned pattern (confidence: {:.2})",
                pattern.confidence
            ),
            fix_type: pattern.successful_fix.fix_type,
            changes: pattern.successful_fix.changes,
            confidence: pattern.confidence,
            estimated_effort: pattern.successful_fix.estimated_effort,
            benefits: pattern.successful_fix.benefits,
            risks: pattern.successful_fix.risks,
        })
    })
}

/// Get learning system health status
#[tauri::command]
pub async fn get_learning_system_health(
    ai_service: State<'_, AIServiceState>,
) -> Result<LearningSystemHealth, String> {
    log::info!("Getting learning system health");

    acquire_service_and_execute!(ai_service, AIServiceState, {
        // In a real implementation, this would check the learning system's health
        Ok(LearningSystemHealth {
            is_healthy: true,
            storage_status: StorageHealth::Good,
            pattern_count: 0,
            last_sync: Some(chrono::Utc::now()),
            errors: vec![],
        })
    })
}

/// Learning system health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSystemHealth {
    pub is_healthy: bool,
    pub storage_status: StorageHealth,
    pub pattern_count: usize,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub errors: Vec<String>,
}

/// Storage health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageHealth {
    Good,
    Degraded,
    Failing,
    Unavailable,
}

/// Helper macros and types from original file

/// Placeholder command config type
pub struct CommandConfig;

impl Default for CommandConfig {
    fn default() -> Self {
        CommandConfig
    }
}

/// Helper macros from original file
macro_rules! acquire_service_and_execute {
    ($service:expr, $state_type:ty, $operation:block) => {{
        let service_guard = $service.lock().await;
        let service = service_guard.as_ref().ok_or("AI service not initialized")?;
        $operation
    }};
}

macro_rules! execute_command {
    ($name:expr, $config:expr, $operation:expr) => {{
        log::info!("Executing command: {}", $name);
        $operation
    }};
}

macro_rules! format_command_error {
    ($error:expr, $context:expr) => {
        format!("{} failed: {}", $context, $error)
    };
}

pub(crate) use acquire_service_and_execute;
pub(crate) use execute_command;
pub(crate) use format_command_error;

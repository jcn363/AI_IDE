//! Core AI Refactoring Service
//!
//! This module provides the main integration point for AI-driven refactoring operations.
//! It serves as the bridge between LSP requests and the comprehensive refactoring
//! capabilities implemented across the various operation modules.

use crate::analysis::RefactoringAnalysisEngine;
use crate::operations::*;
use crate::types::*;
use crate::SuggestionEngine;
// Define RefactoringSuggestion locally since it's not in suggestions.rs yet
#[derive(Debug, Clone)]
pub struct RefactoringSuggestion {
    pub operation_type:   String,
    pub confidence_score: f64,
    pub description:      String,
}

use std::collections::HashMap;
use std::sync::Arc;

#[cfg(feature = "lsp")]
use lsp_types::{Position, Range, TextDocumentIdentifier, TextEdit, Uri, WorkspaceEdit};
use tokio::sync::Mutex;

use crate::confidence::{ConfidenceScorer, ScoringStrategy};
use crate::enhanced_backup::EnhancedBackupManager;
use crate::logging::RefactoringLogger;
use crate::progress::ProgressTracker;
use crate::safety::SafetyAnalyzer;
use crate::suggestions::SuggestionContext;
use crate::utils::RefactoringUtils;

/// Core AI-powered refactoring service
///
/// This is the main integration point that coordinates all refactoring operations,
/// providing a unified interface for LSP and other clients to access the full
/// spectrum of AI-enhanced refactoring capabilities.
pub struct RefactoringService {
    /// Factory for creating refactoring operations
    operation_factory:  RefactoringOperationFactory,
    /// Analysis engine for impact assessment
    analysis_engine:    RefactoringAnalysisEngine,
    /// Suggestion engine for AI-powered recommendations
    suggestion_engine:  Arc<Mutex<dyn SuggestionEngine>>,
    /// Confidence scoring system
    confidence_scorer:  ConfidenceScorer,
    /// Progress tracking
    progress_tracker:   ProgressTracker,
    /// Safety analyzer
    safety_analyzer:    SafetyAnalyzer,
    /// Logging system
    logger:             RefactoringLogger,
    /// Backup manager
    backup_manager:     EnhancedBackupManager,
    /// Cache for operation results
    operation_cache:    HashMap<String, (RefactoringResult, std::time::Instant)>,
    /// Security validator
    security_validator: SecurityValidator,
}

/// Security validation for refactoring operations
struct SecurityValidator;

impl SecurityValidator {
    fn new() -> Self {
        SecurityValidator
    }

    fn validate_request(&self, request: &RefactoringRequest) -> Result<(), String> {
        // Validate file paths to prevent path traversal attacks
        if request.file_path.contains("..") || !request.file_path.starts_with('/') {
            return Err("Invalid file path: potential security vulnerability".to_string());
        }

        // Validate operation type
        if request.operation_type.is_empty() {
            return Err("Empty operation type not allowed".to_string());
        }

        // Additional security checks can be added here

        Ok(())
    }
}

impl RefactoringService {
    /// Create a new instance of the refactoring service
    pub fn new() -> Self {
        Self {
            operation_factory:  RefactoringOperationFactory,
            analysis_engine:    RefactoringAnalysisEngine::new(),
            suggestion_engine:  Arc::new(Mutex::new(crate::suggestions::AISuggestionEngine::new())),
            confidence_scorer:  ConfidenceScorer::new(ScoringStrategy::default()),
            progress_tracker:   ProgressTracker::new(),
            safety_analyzer:    SafetyAnalyzer::new(),
            logger:             RefactoringLogger::new(),
            backup_manager:     EnhancedBackupManager::new(),
            operation_cache:    HashMap::new(),
            security_validator: SecurityValidator::new(),
        }
    }

    /// Execute a refactoring operation
    pub async fn execute_operation(&self, request: &RefactoringRequest) -> Result<RefactoringOperationResult, String> {
        // Security validation
        self.security_validator.validate_request(request)?;

        // Start progress tracking
        let operation_id = self
            .progress_tracker
            .start_operation(request.operation_type.clone())
            .await;

        // Log operation start
        self.logger
            .log_operation_start(&request.operation_type, operation_id)
            .await;

        let result = match self.execute_operation_internal(request).await {
            Ok(result) => {
                self.logger
                    .log_operation_success(&request.operation_type, operation_id)
                    .await;
                self.progress_tracker.complete_operation(operation_id).await;
                Ok(result)
            }
            Err(e) => {
                self.logger
                    .log_operation_error(&request.operation_type, operation_id, &e)
                    .await;
                self.progress_tracker.fail_operation(operation_id, &e).await;
                Err(e)
            }
        };

        result
    }

    /// Execute operation internally with proper error handling
    async fn execute_operation_internal(
        &self,
        request: &RefactoringRequest,
    ) -> Result<RefactoringOperationResult, String> {
        // Parse operation type
        let operation_type = match RefactoringType::try_from(request.operation_type.as_str()) {
            Ok(op_type) => op_type,
            Err(_) =>
                return Err(format!(
                    "Unknown operation type: {}",
                    request.operation_type
                )),
        };

        // Create the operation instance
        let operation = self.operation_factory.create_operation(&operation_type)?;

        // Convert request to context
        let context = convert_request_to_context(request);

        // Safety analysis
        if let Err(safety_err) = self.safety_analyzer.analyze_safety(&context).await {
            return Err(format!("Safety analysis failed: {}", safety_err));
        }

        // Create backup before execution
        if let Err(backup_err) = self.backup_manager.create_backup(&context.file_path).await {
            self.logger
                .log_warning(&format!("Backup creation failed: {}", backup_err))
                .await;
            // Continue execution but log warning
        }

        // Execute the operation
        let result = operation
            .execute(
                &context,
                &convert_options_to_refactoring_options(&request.options),
            )
            .await?;

        Ok(result)
    }

    /// Get refactoring suggestions for a context
    pub async fn get_suggestions(&self, request: &RefactoringRequest) -> Result<Vec<RefactoringSuggestion>, String> {
        // Security validation
        self.security_validator.validate_request(request)?;

        let context = convert_request_to_context(request);
        let suggestion_context = SuggestionContext {
            file_path:       context.file_path,
            symbol_name:     context.symbol_name,
            symbol_kind:     context.symbol_kind,
            project_context: HashMap::new(), // Can be enhanced
        };

        let suggestions = self
            .suggestion_engine
            .lock()
            .await
            .get_suggestions(&suggestion_context)
            .await?;

        // Score suggestions for confidence
        let mut scored_suggestions = Vec::new();
        for suggestion in suggestions {
            let confidence = self
                .confidence_scorer
                .score_suggestion(&suggestion, &context)
                .await?;
            if confidence >= 0.3 {
                // Only return confidence suggestions
                let mut scored = suggestion.clone();
                scored.confidence_score = confidence;
                scored_suggestions.push(scored);
            }
        }

        // Sort by confidence
        scored_suggestions.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap());

        Ok(scored_suggestions)
    }

    /// Validate an operation before execution
    pub async fn validate_operation(
        &self,
        request: &RefactoringRequest,
    ) -> Result<RefactoringValidationResult, String> {
        // Security validation
        self.security_validator.validate_request(request)?;

        let context = convert_request_to_context(request);
        let options = convert_options_to_refactoring_options(&request.options);

        // Analyze the operation
        let analysis = self
            .analysis_engine
            .analyze_operation(&context, &options)
            .await?;

        // Check safety
        let safety_check = self.safety_analyzer.analyze_safety(&context).await;

        let valid = analysis.is_safe && safety_check.is_ok();
        let mut errors = Vec::new();

        if !analysis.is_safe {
            errors.push("Operation analysis indicates potential safety issues".to_string());
        }

        if let Err(safety_err) = safety_check {
            errors.push(format!("Safety check failed: {}", safety_err));
        }

        Ok(RefactoringValidationResult {
            valid,
            errors,
            warnings: analysis.warnings,
            suggestions: analysis.suggestions,
        })
    }

    /// Get available refactoring operations for a context
    pub async fn get_available_operations(&self, context: &RefactoringContext) -> Result<Vec<RefactoringType>, String> {
        let mut available_ops = Vec::new();

        for op_type in RefactoringOperationFactory::available_refactorings() {
            if let Ok(operation) = self.operation_factory.create_operation(&op_type) {
                if let Ok(true) = operation.is_applicable(context, None).await {
                    available_ops.push(op_type);
                }
            }
        }

        Ok(available_ops)
    }

    /// Get operation metadata
    pub fn get_operation_metadata(&self, operation_type: &RefactoringType) -> Option<RefactoringOperationMetadata> {
        match self.operation_factory.create_operation(operation_type) {
            Ok(operation) => Some(RefactoringOperationMetadata {
                name:           operation.name().to_string(),
                description:    operation.description().to_string(),
                operation_type: operation.refactoring_type(),
                experimental:   false, // Can be enhanced
            }),
            Err(_) => None,
        }
    }

    /// Batch execute multiple operations
    pub async fn batch_execute(&self, operations: Vec<RefactoringRequest>) -> Result<BatchRefactoringResult, String> {
        let mut results = Vec::new();
        let mut errors = Vec::new();

        for operation in operations {
            match self.execute_operation(&operation).await {
                Ok(result) => results.push(result),
                Err(err) => errors.push(format!(
                    "Operation {} failed: {}",
                    operation.operation_type, err
                )),
            }
        }

        Ok(BatchRefactoringResult {
            successful_operations: results.len(),
            failed_operations: errors.len(),
            results,
            errors,
        })
    }
}

/// Convert LSP request to refactoring context
fn convert_request_to_context(request: &RefactoringRequest) -> RefactoringContext {
    RefactoringContext {
        file_path:        request.file_path.clone(),
        cursor_line:      0, // Can be enhanced from LSP position
        cursor_character: 0,
        selection:        None, // Can be enhanced
        symbol_name:      None, // Can be extracted from options
        symbol_kind:      None,
    }
}

/// Convert hashmap to refactoring options
fn convert_options_to_refactoring_options(options: &HashMap<String, serde_json::Value>) -> RefactoringOptions {
    RefactoringOptions {
        create_backup:            true,
        generate_tests:           true,
        apply_to_all_occurrences: false,
        preserve_references:      true,
        ignore_safe_operations:   false,
        extra_options:            Some(options.clone()),
    }
}

/// Result of batch refactoring operations
#[derive(Debug, Clone)]
pub struct BatchRefactoringResult {
    pub successful_operations: usize,
    pub failed_operations:     usize,
    pub results:               Vec<RefactoringOperationResult>,
    pub errors:                Vec<String>,
}

/// Metadata about a refactoring operation
#[derive(Debug, Clone)]
pub struct RefactoringOperationMetadata {
    pub name:           String,
    pub description:    String,
    pub operation_type: RefactoringType,
    pub experimental:   bool,
}

/// Validation result for refactoring operations
#[derive(Debug, Clone)]
pub struct RefactoringValidationResult {
    pub valid:       bool,
    pub errors:      Vec<String>,
    pub warnings:    Vec<String>,
    pub suggestions: Vec<String>,
}

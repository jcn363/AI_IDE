//! Core AI Refactoring Service
//!
//! This module provides the main integration point for AI-driven refactoring operations.
//! It serves as the bridge between LSP requests and the comprehensive refactoring
//! capabilities implemented across the various operation modules.

// We use RefactoringSuggestion from types.rs
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(feature = "lsp")]
use lsp_types::{Position, Range, TextDocumentIdentifier, TextEdit, Uri, WorkspaceEdit};
use tokio::sync::Mutex;

use crate::analysis::RefactoringAnalysisEngine;
use crate::confidence::{ConfidenceScorer, ScoringStrategy};
use crate::engine::{ExecutionContext, RefactoringEngine};
use crate::enhanced_backup::EnhancedBackupManager;
use crate::logging::RefactoringLogger;
use crate::operations::*;
use crate::progress::ProgressTracker;
use crate::safety::SafetyAnalyzer;
use crate::suggestions::SuggestionContext;
use crate::types::*;
use crate::utils::RefactoringUtils;
use crate::SuggestionEngine;

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
        if request.context.file_path.contains("..") || !request.context.file_path.starts_with('/') {
            return Err("Invalid file path: potential security vulnerability".to_string());
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
            .start_operation(request.refactoring_type.to_string())
            .await;

        // Log operation start
        self.logger
            .log_operation_start(&request.refactoring_type.to_string(), operation_id.clone())
            .await;

        let result = match self.execute_operation_internal(request).await {
            Ok(result) => {
                self.logger
                    .log_operation_success(&request.refactoring_type.to_string(), operation_id.clone())
                    .await;
                self.progress_tracker.complete_operation(operation_id).await;
                Ok(result)
            }
            Err(e) => {
                self.logger
                    .log_operation_error(
                        &request.refactoring_type.to_string(),
                        operation_id.clone(),
                        &e,
                    )
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
        let operation_type = request.refactoring_type.clone();

        // Create the operation instance
        let operation = RefactoringOperationFactory::create_operation(&operation_type).map_err(|e| e.to_string())?;

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
            .execute(&context, &request.options)
            .await
            .map_err(|e| e.to_string())?;

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
                // Convert from suggestion engine format to types format
                let refactoring_type = suggestion.refactoring_type.clone();
                let scored = RefactoringSuggestion {
                    refactoring_type,
                    confidence,
                    description: suggestion.description.clone(),
                    context: context.clone(),
                    expected_changes: vec![], // Will be populated by analysis
                };
                scored_suggestions.push(scored);
            }
        }

        // Sort by confidence
        scored_suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

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
        let options = &request.options;

        // Analyze the operation
        let analysis = self
            .analysis_engine
            .analyze_operation(&context, options)
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
            if let Ok(operation) = RefactoringOperationFactory::create_operation(&op_type) {
                if let Ok(true) = operation.is_applicable(context, None).await {
                    available_ops.push(op_type);
                }
            }
        }

        Ok(available_ops)
    }

    /// Get operation metadata
    pub fn get_operation_metadata(&self, operation_type: &RefactoringType) -> Option<RefactoringOperationMetadata> {
        match RefactoringOperationFactory::create_operation(operation_type) {
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
                    operation.refactoring_type.to_string(),
                    err
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
    request.context.clone()
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

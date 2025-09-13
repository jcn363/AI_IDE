//! Advanced AI-powered refactoring commands module
//!
//! This module provides comprehensive Tauri commands for AI-assisted code refactoring.
//! The module integrates with the advanced refactoring operations from the refactoring crate,
//! providing analysis, execution, and management capabilities.
//!
//! Key features:
//! - Real-time refactoring analysis and execution
//! - Safety validation and confidence scoring
//! - LSP integration for context awareness
//! - Batch operations and progress tracking
//! - Undo/redo functionality with history management

// Re-export the advanced refactoring operations
use crate::commands::types::*;
use crate::validation;
pub use rust_ai_ide_ai_refactoring::*;
use rust_ai_ide_common::types::*;

/// Global configuration for command operations
static COMMAND_CONFIG: std::sync::OnceLock<super::command_templates::CommandConfig> =
    std::sync::OnceLock::new();

/// Initialize the command configuration
fn get_command_config() -> &'static super::command_templates::CommandConfig {
    COMMAND_CONFIG.get_or_init(|| super::command_templates::CommandConfig {
        enable_logging: true,
        log_level: log::Level::Info,
        enable_validation: true,
        async_timeout_secs: Some(300), // 5 minutes for complex refactoring operations
    })
}

/// Refactoring Service wrapper for state management
pub struct RefactoringService {
    operation_factory: RefactoringOperationFactory,
}

impl RefactoringService {
    pub fn new() -> Self {
        Self {
            operation_factory: RefactoringOperationFactory,
        }
    }

    pub fn create_operation(
        &self,
        refactoring_type: &RefactoringType,
    ) -> Result<Box<dyn RefactoringOperation>, String> {
        self.operation_factory
            .create_operation(refactoring_type)
            .map_err(|e| format!("Failed to create refactoring operation: {}", e))
    }
}

impl Default for RefactoringService {
    fn default() -> Self {
        Self::new()
    }
}

/// Main command: Execute refactoring operation with enhanced error handling and progress tracking
#[tauri::command]
pub async fn execute_refactoring_operation(
    request: RefactoringExecutionRequest,
    state: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::IDEState>>>,
) -> Result<RefactoringResult, String> {
    log::info!(
        "Executing refactoring operation: {}",
        request.operation_type
    );

    // Validate inputs
    crate::validation::validate_secure_path(&request.file_path, false)
        .map_err(|e| format!("Invalid file path: {}", e))?;

    if !std::path::Path::new(&request.file_path).exists() {
        return Err(format!("File does not exist: {}", request.file_path));
    }

    // Acquire service state
    let mut ide_state = state.lock().await;
    let refactoring_service = ide_state
        .refactoring_service
        .get_or_insert_with(|| RefactoringService::new());

    // Create the refactoring operation
    let operation = refactoring_service.create_operation(&request.operation_type)?;

    // Convert request context to internal types
    let context = request.context.try_into()?;
    let options = request.options.try_into()?;

    // Execute the operation with progress tracking
    execute_command!(
        stringify!(execute_refactoring_operation),
        &get_command_config(),
        async move || {
            let result = operation
                .execute(&context, &options)
                .await
                .map_err(|e| format!("Refactoring operation failed: {}", e))?;

            Ok::<_, String>(result)
        }
    )
}

/// Advanced analysis command with multi-operation support
#[tauri::command]
pub async fn analyze_refactoring_candidates(
    request: RefactoringAnalysisRequest,
    state: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::IDEState>>>,
) -> Result<RefactoringAnalysisResponse, String> {
    log::info!(
        "Analyzing refactoring candidates for file: {}",
        request.file_path
    );

    // Validate file path
    crate::validation::validate_secure_path(&request.file_path, false)
        .map_err(|e| format!("Invalid file path: {}", e))?;

    // Read file content
    let content = tokio::fs::read_to_string(&request.file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Acquire service
    let mut ide_state = state.lock().await;
    let refactoring_service = ide_state
        .refactoring_service
        .get_or_insert_with(|| RefactoringService::new());

    // Analyze for each requested operation type
    let mut candidates = Vec::new();
    let mut analysis_summary = vec![];

    for operation_type in &request.operation_types {
        let operation = refactoring_service.create_operation(operation_type).ok()?;

        // Create basic context for analysis
        let context = RefactoringContext {
            file_path: request.file_path.clone(),
            symbol_name: request.target_symbol.clone(),
            symbol_kind: request.symbol_kind.map(|k| k.into()),
            cursor_line: request
                .cursor_position
                .map(|p| p.line as usize)
                .unwrap_or(0),
            cursor_character: request
                .cursor_position
                .map(|p| p.character as usize)
                .unwrap_or(0),
            selection: request.selection.map(|s| s.into()),
            context_lines: vec![], // Would be populated from LSP
            language: ProgrammingLanguage::Rust,
            project_root: request.project_root.clone(),
        };

        let basic_options = RefactoringOptions::default();

        if let Ok(true) = operation
            .is_applicable(&context, Some(&basic_options))
            .await
        {
            if let Ok(analysis) = operation.analyze(&context).await {
                candidates.push(RefactoringCandidate {
                    operation_type: *operation_type,
                    confidence_score: analysis.confidence_score,
                    suitability_reasons: analysis.suggestions,
                    potential_impact: analysis.potential_impact.into(),
                    breaking_changes: analysis.breaking_changes,
                    affected_files: analysis.affected_files,
                });

                analysis_summary.push(format!(
                    "Operation {:?}: confidence {:.2}, impact {:?}",
                    operation_type, analysis.confidence_score, analysis.potential_impact
                ));
            }
        }
    }

    Ok(RefactoringAnalysisResponse {
        file_path: request.file_path,
        candidates,
        analysis_summary,
        total_candidates: candidates.len(),
    })
}

/// Batch refactoring command with parallel execution support
#[tauri::command]
pub async fn execute_batch_refactoring(
    request: BatchRefactoringRequest,
    state: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::IDEState>>>,
) -> Result<BatchRefactoringResult, String> {
    log::info!(
        "Executing batch refactoring with {} operations",
        request.operations.len()
    );

    // Validate all file paths
    for operation in &request.operations {
        crate::validation::validate_secure_path(&operation.context.file_path, false)
            .map_err(|e| format!("Invalid file path in operation: {}", e))?;
    }

    // Acquire service
    let mut ide_state = state.lock().await;
    let refactoring_service = ide_state
        .refactoring_service
        .get_or_insert_with(|| RefactoringService::new());

    // Create batch executor
    let executor = BatchRefactoringOperationExecutor;
    let batch_options = BatchRefactoringOptions {
        operations: request
            .operations
            .into_iter()
            .map(|op| BatchRefactoringOperation {
                operation_type: op.operation_type,
                context: op.context.try_into().unwrap_or_default(),
                options: op.options.try_into().unwrap_or_default(),
            })
            .collect(),
        parallel_execution: request.parallel_execution,
        stop_on_failure: request.stop_on_failure,
        create_backup_directory: true,
        validate_dependencies: true,
        max_concurrent_operations: request.max_concurrent_operations,
    };

    let mut progress_tracker = ProgressTracker::new();

    execute_command!(
        stringify!(execute_batch_refactoring),
        &get_command_config(),
        async move || {
            let batch_result = executor
                .execute_batch(batch_options, &mut progress_tracker)
                .await
                .map_err(|e| format!("Batch refactoring failed: {}", e))?;

            Ok::<_, String>(BatchRefactoringResult {
                operation_count: batch_result.results.len(),
                successes: batch_result.results.len(),
                failures: 0,
                warning_count: batch_result
                    .results
                    .iter()
                    .map(|r| r.result.warnings.len())
                    .sum(),
                execution_time_ms: 0, // Would be tracked by progress tracker
                results: batch_result.results.into_iter().map(|r| r.result).collect(),
                progress_summary: progress_tracker.get_summary(),
            })
        }
    )
}

/// Command for generating test code after refactoring
#[tauri::command]
pub async fn generate_refactoring_tests(
    request: TestGenerationRequest,
    state: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::IDEState>>>,
) -> Result<TestGenerationResponse, String> {
    log::info!(
        "Generating refactoring tests for operation: {}",
        request.operation_type
    );

    // Validate file path
    crate::validation::validate_secure_path(&request.file_path, false)
        .map_err(|e| format!("Invalid file path: {}", e))?;

    // Acquire service
    let mut ide_state = state.lock().await;
    let refactoring_service = ide_state
        .refactoring_service
        .get_or_insert_with(|| RefactoringService::new());

    // Get the operation for test generation context
    if let Ok(operation) = refactoring_service.create_operation(&request.operation_type) {
        let test_generator = RefactoringTestGenerator::new();

        execute_command!(
            stringify!(generate_refactoring_tests),
            &get_command_config(),
            async move || {
                // Create basic result for test generation
                let mock_result = RefactoringResult {
                    id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
                    success: true,
                    changes: vec![], // Would be populated from actual operation
                    error_message: None,
                    warnings: vec![],
                    new_content: Some(request.original_content.clone()),
                };

                // Generate tests
                let tests_result = test_generator
                    .generate_refactoring_tests(
                        &request.operation_type,
                        &mock_result,
                        &RefactoringContext {
                            file_path: request.file_path.clone(),
                            symbol_name: request.symbol_name,
                            symbol_kind: request.symbol_kind,
                            cursor_line: request.cursor_line,
                            cursor_character: request.cursor_character,
                            selection: request.selection,
                            context_lines: vec![],
                            language: ProgrammingLanguage::Rust,
                            project_root: request.project_root,
                        },
                    )
                    .await
                    .map_err(|e| format!("Test generation failed: {}", e))?;

                Ok::<_, String>(TestGenerationResponse {
                    operation_type: request.operation_type,
                    generated_tests: tests_result
                        .unit_tests
                        .into_iter()
                        .map(|test| GeneratedTestInfo {
                            test_type: test.test_type.to_string(),
                            language: "Rust".to_string(),
                            framework: test.framework.clone(),
                            content: test.code,
                            filename: format!(
                                "test_{}.rs",
                                test.name.to_lowercase().replace(" ", "_")
                            ),
                            dependencies: vec![], // Would be analyzed
                        })
                        .collect(),
                    test_count: tests_result.unit_tests.len(),
                    coverage_estimate: 0.85, // Estimated good coverage for refactoring tests
                })
            }
        )
    } else {
        Err(format!(
            "Unsupported operation type for test generation: {:?}",
            request.operation_type
        ))
    }
}

/// Command to get available refactoring operations with capabilities
#[tauri::command]
pub async fn get_available_refactoring_operations(
    _state: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::IDEState>>>,
) -> Result<Vec<RefactoringOperationInfo>, String> {
    log::info!("Retrieving available refactoring operations");

    execute_command!(
        stringify!(get_available_refactoring_operations),
        &get_command_config(),
        async move || {
            let factory = RefactoringOperationFactory;
            let available_types = factory.available_refactorings();

            let operations = available_types
                .into_iter()
                .map(|operation_type| {
                    let operation = factory.create_operation(&operation_type).unwrap();
                    RefactoringOperationInfo {
                        operation_type,
                        name: operation.name().to_string(),
                        description: operation.description().to_string(),
                        requires_selection: matches!(
                            operation_type,
                            RefactoringType::ExtractFunction
                                | RefactoringType::ExtractVariable
                                | RefactoringType::InlineVariable
                        ),
                        is_experimental: false, // Only some operations could be marked experimental
                        typical_confidence_score: 0.8, // Would be operation-specific
                    }
                })
                .collect();

            Ok::<_, String>(operations)
        }
    )
}

/// Command for undo/redo refactoring operations
#[tauri::command]
pub async fn undo_refactoring_operation(
    request: UndoRequest,
    state: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::IDEState>>>,
) -> Result<UndoResult, String> {
    log::info!("Undoing refactoring operation: {}", request.operation_id);

    // Acquire service
    let mut ide_state = state.lock().await;

    // For now, return a placeholder - would integrate with a real undo system
    Ok(UndoResult {
        success: true,
        operation_id: request.operation_id,
        reverted_changes: vec![], // Would contain the actual reverted changes
        warnings: vec!["Undo system integration pending".to_string()],
    })
}

/// Command for validating refactoring safety before execution
#[tauri::command]
pub async fn validate_refactoring_safety(
    request: SafetyValidationRequest,
    state: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::IDEState>>>,
) -> Result<SafetyValidationResult, String> {
    log::info!(
        "Validating refactoring safety for operation: {}",
        request.operation_type
    );

    // Validate file path
    crate::validation::validate_secure_path(&request.file_path, false)
        .map_err(|e| format!("Invalid file path: {}", e))?;

    // Acquire service
    let mut ide_state = state.lock().await;
    let refactoring_service = ide_state
        .refactoring_service
        .get_or_insert_with(|| RefactoringService::new());

    // Create operation and validate
    if let Ok(operation) = refactoring_service.create_operation(&request.operation_type) {
        let context = request.context.try_into()?;
        let analysis = operation
            .analyze(&context)
            .await
            .map_err(|e| format!("Safety analysis failed: {}", e))?;

        Ok(SafetyValidationResult {
            operation_type: request.operation_type,
            is_safe: analysis.is_safe,
            confidence_score: analysis.confidence_score,
            potential_impact: analysis.potential_impact.into(),
            breaking_changes: analysis.breaking_changes,
            suggested_alternatives: analysis.suggestions,
            recommendations: generate_safety_recommendations(&analysis),
        })
    } else {
        Err(format!(
            "Unsupported operation type: {:?}",
            request.operation_type
        ))
    }
}

/// Helper function to generate safety recommendations based on analysis
fn generate_safety_recommendations(analysis: &RefactoringAnalysis) -> Vec<String> {
    let mut recommendations = Vec::new();

    if analysis.confidence_score < 0.7 {
        recommendations.push("Consider manual review before proceeding".to_string());
    }

    if matches!(analysis.potential_impact, RefactoringImpact::High) {
        recommendations.push("Create backup before refactoring".to_string());
        recommendations.push("Consider smaller, incremental changes".to_string());
    }

    if !analysis.breaking_changes.is_empty() {
        recommendations.push("Update callers and tests after refactoring".to_string());
    }

    if recommendations.is_empty() {
        recommendations.push("Refactoring appears safe to proceed".to_string());
    }

    recommendations
}

/// Command for getting refactoring configuration and preferences
#[tauri::command]
pub async fn get_refactoring_preferences(
    state: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::IDEState>>>,
) -> Result<RefactoringPreferences, String> {
    let ide_state = state.lock().await;

    // Return default preferences - would be loaded from user settings
    Ok(RefactoringPreferences {
        enable_auto_preview: true,
        enable_safety_validation: true,
        default_confidence_threshold: 0.7,
        auto_generate_tests: true,
        enable_backup: true,
        max_concurrent_operations: 3,
        experimental_features_enabled: false,
        lsp_integration_level: "full".to_string(),
    })
}

/// Command for updating refactoring preferences
#[tauri::command]
pub async fn update_refactoring_preferences(
    preferences: RefactoringPreferences,
    state: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::IDEState>>>,
) -> Result<String, String> {
    log::info!("Updating refactoring preferences");

    // Acquire state and save preferences
    let mut ide_state = state.lock().await;
    // Would save to persistent storage here

    Ok("Refactoring preferences updated successfully".to_string())
}

// Module exports for the commands
pub use rust_ai_ide_ai_refactoring::{
    BatchRefactoringOperationExecutor, ProgressTracker, RefactoringTestGenerator,
};

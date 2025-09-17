//! Main refactoring engine with semantic analysis capabilities

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::analysis::RefactoringAnalysisEngine;
use crate::enhanced_backup::EnhancedBackupManager;
use crate::safety::SafetyAnalyzer;
use crate::types::*;
use crate::RefactoringOperation;

/// Main refactoring engine - central orchestration component
pub struct RefactoringEngine {
    /// Analysis engine for impact assessment
    analysis_engine:    RefactoringAnalysisEngine,
    /// Safety analyzer for risk assessment
    safety_analyzer:    SafetyAnalyzer,
    /// Backup manager for rollback capabilities
    backup_manager:     EnhancedBackupManager,
    /// Operation registry
    operation_registry: HashMap<RefactoringType, Arc<dyn RefactoringOperation>>,
    /// Performance metrics
    metrics:            Mutex<EngineMetrics>,
}

/// Engine performance metrics
#[derive(Debug, Clone, Default)]
pub struct EngineMetrics {
    pub total_operations:          u64,
    pub successful_operations:     u64,
    pub failed_operations:         u64,
    pub average_execution_time_ms: f64,
    pub total_backup_size_bytes:   u64,
}

/// Refactoring execution context
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub operation_id:   String,
    pub start_time:     std::time::Instant,
    pub user_id:        Option<String>,
    pub workspace_root: String,
    pub dry_run:        bool,
}

/// Semantic analysis result
#[derive(Debug, Clone)]
pub struct SemanticAnalysisResult {
    pub symbols_affected:      Vec<SymbolReference>,
    pub dependencies:          Vec<Dependency>,
    pub semantic_impact_score: f64,
    pub recommended_actions:   Vec<String>,
}

/// Symbol reference for semantic analysis
#[derive(Debug, Clone)]
pub struct SymbolReference {
    pub name:       String,
    pub kind:       SymbolKind,
    pub location:   CodeLocation,
    pub references: Vec<CodeLocation>,
}

/// Dependency information
#[derive(Debug, Clone)]
pub struct Dependency {
    pub from:            SymbolReference,
    pub to:              SymbolReference,
    pub dependency_type: DependencyType,
}

/// Symbol kinds for semantic analysis
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Module,
    Variable,
    Field,
    Method,
    TypeAlias,
}

/// Dependency types
#[derive(Debug, Clone, PartialEq)]
pub enum DependencyType {
    Calls,
    Inherits,
    Implements,
    Uses,
    References,
    Contains,
}

/// Code location
#[derive(Debug, Clone)]
pub struct CodeLocation {
    pub file_path: String,
    pub line:      usize,
    pub column:    usize,
    pub length:    usize,
}

impl RefactoringEngine {
    /// Create a new refactoring engine
    pub fn new() -> Self {
        Self {
            analysis_engine:    RefactoringAnalysisEngine::new(),
            safety_analyzer:    SafetyAnalyzer::new(),
            backup_manager:     EnhancedBackupManager::new(),
            operation_registry: HashMap::new(),
            metrics:            Mutex::new(EngineMetrics::default()),
        }
    }

    /// Register a refactoring operation
    pub fn register_operation(&mut self, operation_type: RefactoringType, operation: Arc<dyn RefactoringOperation>) {
        self.operation_registry.insert(operation_type, operation);
    }

    /// Execute a refactoring operation with full orchestration
    pub async fn execute_refactoring(
        &self,
        request: &RefactoringRequest,
        context: &ExecutionContext,
    ) -> Result<RefactoringExecutionResult, String> {
        let start_time = std::time::Instant::now();

        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            metrics.total_operations += 1;
        }

        println!(
            "🔧 Starting refactoring execution: {}",
            request.refactoring_type.to_string()
        );

        // Step 1: Semantic Analysis
        let semantic_result = self.perform_semantic_analysis(request, context).await?;

        // Step 2: Safety Analysis
        let safety_result = self
            .safety_analyzer
            .perform_comprehensive_analysis(&convert_request_to_context(request))
            .await
            .map_err(|e| format!("Safety analysis failed: {}", e))?;

        // Step 3: Impact Analysis
        let analysis_result = self
            .analysis_engine
            .analyze_operation(
                &convert_request_to_context(request),
                &convert_options_to_refactoring_options(&request.options),
            )
            .await?;

        // Step 4: Create Backup (unless dry run)
        let backup_metadata = if !context.dry_run {
            Some(
                self.backup_manager
                    .create_backup(&request.context.file_path)
                    .await?,
            )
        } else {
            None
        };

        // Step 5: Execute Operation (skip if dry run)
        let execution_result = if !context.dry_run {
            self.execute_operation_with_fallback(request, context)
                .await?
        } else {
            // For dry run, simulate the execution
            RefactoringOperationResult {
                id:            Some(context.operation_id.clone()),
                success:       true,
                changes:       vec![], // Would be populated in real execution
                error_message: None,
                warnings:      vec!["This is a dry run - no changes applied".to_string()],
                new_content:   None,
            }
        };

        // Step 6: Validation and Verification
        let validation_result = if execution_result.success && !context.dry_run {
            self.validate_execution(&execution_result).await?
        } else {
            ValidationResult {
                passed:          true,
                issues:          vec![],
                recommendations: vec![],
            }
        };

        let execution_time = start_time.elapsed();

        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            if execution_result.success {
                metrics.successful_operations += 1;
            } else {
                metrics.failed_operations += 1;
            }
            metrics.average_execution_time_ms = (metrics.average_execution_time_ms
                * (metrics.total_operations - 1) as f64
                + execution_time.as_millis() as f64)
                / metrics.total_operations as f64;
        }

        Ok(RefactoringExecutionResult {
            operation_id: context.operation_id.clone(),
            success: execution_result.success,
            execution_time_ms: execution_time.as_millis() as u64,
            semantic_analysis: semantic_result,
            safety_analysis: safety_result,
            impact_analysis: analysis_result,
            backup_metadata,
            execution_result,
            validation_result,
        })
    }

    /// Perform semantic analysis
    async fn perform_semantic_analysis(
        &self,
        request: &RefactoringRequest,
        context: &ExecutionContext,
    ) -> Result<SemanticAnalysisResult, String> {
        // Parse the source file
        let content = std::fs::read_to_string(&request.context.file_path)
            .map_err(|e| format!("Failed to read source file: {}", e))?;

        let syntax = syn::parse_str::<syn::File>(&content).map_err(|e| format!("Failed to parse Rust syntax: {}", e))?;

        // Analyze symbols and dependencies
        let symbols_affected = self.extract_symbols(&syntax, request)?;
        let dependencies = self.analyze_dependencies(&syntax, &symbols_affected)?;

        // Calculate semantic impact score
        let semantic_impact_score = self.calculate_semantic_impact(&symbols_affected, &dependencies);

        // Generate recommendations
        let recommended_actions = self.generate_semantic_recommendations(&symbols_affected, &dependencies);

        Ok(SemanticAnalysisResult {
            symbols_affected,
            dependencies,
            semantic_impact_score,
            recommended_actions,
        })
    }

    /// Extract symbols from AST
    fn extract_symbols(
        &self,
        syntax: &syn::File,
        request: &RefactoringRequest,
    ) -> Result<Vec<SymbolReference>, String> {
        let mut symbols = Vec::new();

        for item in &syntax.items {
            match item {
                syn::Item::Fn(fn_item) => {
                    if let Some(symbol_name) = request
                        .options
                        .extra_options
                        .as_ref()
                        .and_then(|m| m.get("symbol_name"))
                        .and_then(|v| v.as_str())
                    {
                        if fn_item.sig.ident == symbol_name {
                            symbols.push(SymbolReference {
                                name:       symbol_name.to_string(),
                                kind:       SymbolKind::Function,
                                location:   CodeLocation {
                                    file_path: request.context.file_path.clone(),
                                    line:      0, // Line tracking not available in syn 2.0
                                    column:    0, // Column tracking not available in syn 2.0
                                    length:    fn_item.sig.ident.to_string().len(),
                                },
                                references: vec![], // Would be populated by usage analysis
                            });
                        }
                    }
                }
                syn::Item::Struct(struct_item) => {
                    symbols.push(SymbolReference {
                        name:       struct_item.ident.to_string(),
                        kind:       SymbolKind::Struct,
                        location:   CodeLocation {
                            file_path: request.context.file_path.clone(),
                            line:      0, // Line tracking not available in syn 2.0
                            column:    0, // Column tracking not available in syn 2.0
                            length:    struct_item.ident.to_string().len(),
                        },
                        references: vec![],
                    });
                }
                // Add more symbol types as needed
                _ => {}
            }
        }

        Ok(symbols)
    }

    /// Analyze dependencies between symbols
    fn analyze_dependencies(&self, syntax: &syn::File, symbols: &[SymbolReference]) -> Result<Vec<Dependency>, String> {
        let mut dependencies = Vec::new();

        // Simple dependency analysis - could be enhanced significantly
        for symbol in symbols {
            // Find references to this symbol in the AST
            for item in &syntax.items {
                if let syn::Item::Fn(fn_item) = item {
                    if self.contains_symbol_reference(&fn_item.block, &symbol.name) {
                        dependencies.push(Dependency {
                            from:            SymbolReference {
                                name:       fn_item.sig.ident.to_string(),
                                kind:       SymbolKind::Function,
                                location:   CodeLocation {
                                    file_path: "".to_string(), // Would be set properly
                                    line:      0,
                                    column:    0,
                                    length:    fn_item.sig.ident.to_string().len(),
                                },
                                references: vec![],
                            },
                            to:              symbol.clone(),
                            dependency_type: DependencyType::Calls,
                        });
                    }
                }
            }
        }

        Ok(dependencies)
    }

    /// Check if a block contains reference to a symbol
    fn contains_symbol_reference(&self, block: &syn::Block, symbol_name: &str) -> bool {
        for stmt in &block.stmts {
            match stmt {
                syn::Stmt::Expr(expr, _) =>
                    if self.expr_contains_symbol(expr, symbol_name) {
                        return true;
                    },
                syn::Stmt::Local(local) =>
                    if let Some(init) = &local.init {
                        if self.expr_contains_symbol(&init.expr, symbol_name) {
                            return true;
                        }
                    },
                _ => {}
            }
        }
        false
    }

    /// Check if expression contains symbol reference
    fn expr_contains_symbol(&self, expr: &syn::Expr, symbol_name: &str) -> bool {
        match expr {
            syn::Expr::Call(call) => {
                if let syn::Expr::Path(path) = &*call.func {
                    if let Some(ident) = path.path.get_ident() {
                        return ident == symbol_name;
                    }
                }
                false
            }
            syn::Expr::MethodCall(method_call) => self.expr_contains_symbol(&method_call.receiver, symbol_name),
            syn::Expr::Path(path) =>
                if let Some(ident) = path.path.get_ident() {
                    ident == symbol_name
                } else {
                    false
                },
            _ => false,
        }
    }

    /// Calculate semantic impact score
    fn calculate_semantic_impact(&self, symbols: &[SymbolReference], dependencies: &[Dependency]) -> f64 {
        let symbol_score = symbols.len() as f64 * 0.3;
        let dependency_score = dependencies.len() as f64 * 0.7;
        (symbol_score + dependency_score).min(1.0)
    }

    /// Generate semantic recommendations
    fn generate_semantic_recommendations(
        &self,
        symbols: &[SymbolReference],
        dependencies: &[Dependency],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if symbols.len() > 1 {
            recommendations.push("Multiple symbols affected - consider reviewing all usages".to_string());
        }

        if dependencies.len() > 5 {
            recommendations.push("High dependency count - extensive testing recommended".to_string());
        }

        if symbols
            .iter()
            .any(|s| matches!(s.kind, SymbolKind::Function))
        {
            recommendations.push("Function refactoring - verify all call sites".to_string());
        }

        recommendations
    }

    /// Execute operation with fallback handling
    async fn execute_operation_with_fallback(
        &self,
        request: &RefactoringRequest,
        context: &ExecutionContext,
    ) -> Result<RefactoringOperationResult, String> {
        // Get the operation
        let operation = self
            .operation_registry
            .get(&request.refactoring_type)
            .ok_or_else(|| format!("Operation not registered: {:?}", request.refactoring_type))?;

        // Execute the operation
        let result = operation
            .execute(
                &convert_request_to_context(request),
                &convert_options_to_refactoring_options(&request.options),
            )
            .await
            .map_err(|e| e.to_string())?;

        Ok(result)
    }

    /// Validate execution results
    async fn validate_execution(&self, result: &RefactoringOperationResult) -> Result<ValidationResult, String> {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Basic validation
        if result.changes.is_empty() && result.success {
            issues.push("Operation succeeded but no changes were made".to_string());
        }

        if !result.warnings.is_empty() {
            recommendations.push("Review warnings from the refactoring operation".to_string());
        }

        // Semantic validation could be added here
        recommendations.push("Run tests to validate refactoring correctness".to_string());

        Ok(ValidationResult {
            passed: issues.is_empty(),
            issues,
            recommendations,
        })
    }

    /// Get engine metrics
    pub async fn get_metrics(&self) -> EngineMetrics {
        self.metrics.lock().await.clone()
    }

    /// Rollback an operation
    pub async fn rollback_operation(&self, backup_session_id: &str) -> Result<(), String> {
        self.backup_manager.rollback(backup_session_id).await
    }
}

/// Result of refactoring execution
#[derive(Debug, Clone)]
pub struct RefactoringExecutionResult {
    pub operation_id:      String,
    pub success:           bool,
    pub execution_time_ms: u64,
    pub semantic_analysis: SemanticAnalysisResult,
    pub safety_analysis:   crate::safety::SafetyAnalysisResult,
    pub impact_analysis:   RefactoringAnalysis,
    pub backup_metadata:   Option<crate::enhanced_backup::BackupMetadata>,
    pub execution_result:  RefactoringOperationResult,
    pub validation_result: ValidationResult,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub passed:          bool,
    pub issues:          Vec<String>,
    pub recommendations: Vec<String>,
}

/// Convert request to context
fn convert_request_to_context(request: &RefactoringRequest) -> RefactoringContext {
    RefactoringContext {
        file_path:        request.context.file_path.clone(),
        cursor_line:      0,
        cursor_character: 0,
        selection:        None,
        symbol_name:      request
            .options
            .extra_options
            .as_ref()
            .and_then(|m| m.get("symbol_name"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        symbol_kind:      None,
    }
}

/// Convert options to refactoring options
fn convert_options_to_refactoring_options(options: &RefactoringOptions) -> RefactoringOptions {
    options.clone()
}

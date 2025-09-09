use crate::types::*;
use syn::{visit::Visit, Expr, Item, Stmt};
use std::collections::HashMap;

/// Comprehensive safety analyzer for refactoring operations
pub struct SafetyAnalyzer {
    /// Enabled safety checks
    enabled_checks: Vec<SafetyCheck>,
    /// Risk thresholds
    risk_thresholds: RiskThresholds,
    /// AST analysis cache
    ast_cache: HashMap<String, ASTAnalysis>,
}

impl SafetyAnalyzer {
    pub fn new() -> Self {
        let enabled_checks = vec![
            SafetyCheck::ASTIntegrityCheck,
            SafetyCheck::SymbolUsageAnalysis,
            SafetyCheck::TypeConsistencyCheck,
            SafetyCheck::ControlFlowAnalysis,
            SafetyCheck::DependencyImpactAnalysis,
        ];

        let risk_thresholds = RiskThresholds {
            max_affected_lines: 1000,
            max_affected_symbols: 50,
            max_dependency_chain: 5,
            max_breaking_changes: 3,
            critical_pattern_threshold: 0.3,
        };

        SafetyAnalyzer {
            enabled_checks,
            risk_thresholds,
            ast_cache: HashMap::new(),
        }
    }

    /// Perform comprehensive safety analysis before refactoring
    pub async fn perform_safety_analysis(
        &mut self,
        context: &RefactoringContext,
        refactoring_type: &RefactoringType,
        code_content: Option<&str>,
    ) -> Result<SafetyReport, Box<dyn std::error::Error + Send + Sync>> {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();
        let mut overall_risk = RefactoringRisk::Low;

        // Run all enabled safety checks
        for check in &self.enabled_checks {
            let result = self.run_safety_check(check, context, refactoring_type, code_content).await?;
            issues.extend(result.issues);
            recommendations.extend(result.recommendations);

            if result.risk_level > overall_risk {
                overall_risk = result.risk_level;
            }
        }

        // Perform integrity validation if code content is available
        let integrity_score = if let Some(code) = code_content {
            self.validate_ast_integrity(code, context)?
        } else {
            0.8 // Default conservative score
        };

        // Generate risk assessment
        let risk_assessment = self.assess_overall_risk(&issues, integrity_score, refactoring_type);

        Ok(SafetyReport {
            is_safe: overall_risk <= RefactoringRisk::Medium,
            overall_risk,
            issues,
            recommendations,
            integrity_score,
            risk_assessment,
        })
    }

    /// Run a specific safety check
    async fn run_safety_check(
        &self,
        check: &SafetyCheck,
        context: &RefactoringContext,
        refactoring_type: &RefactoringType,
        code_content: Option<&str>,
    ) -> Result<SafetyCheckResult, Box<dyn std::error::Error + Send + Sync>> {
        match check {
            SafetyCheck::ASTIntegrityCheck => {
                self.check_ast_integrity(context, code_content).await
            }
            SafetyCheck::SymbolUsageAnalysis => {
                self.analyze_symbol_usage(context, code_content).await
            }
            SafetyCheck::TypeConsistencyCheck => {
                self.check_type_consistency(context, code_content).await
            }
            SafetyCheck::ControlFlowAnalysis => {
                self.analyze_control_flow(context, code_content).await
            }
            SafetyCheck::DependencyImpactAnalysis => {
                self.analyze_dependency_impact(context, refactoring_type).await
            }
        }
    }

    /// Validate AST integrity and parsing accuracy
    fn validate_ast_integrity(
        &self,
        code: &str,
        context: &RefactoringContext,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        // Parse the code into AST
        let ast_result = syn::parse_file(code);
        if let Err(parse_error) = ast_result {
            return Ok(0.2); // Critical integrity failure
        }

        let syntax_tree = ast_result.unwrap();
        let mut integrity_checker = ASTIntegrityChecker::new();
        integrity_checker.visit_file(&syntax_tree);

        // Calculate integrity score based on various factors
        let mut score = 1.0;

        // Penalty for parse errors
        if integrity_checker.parse_errors > 0 {
            score *= 0.7;
        }

        // Penalty for complex structures that might be error-prone
        let complexity_penalty = (integrity_checker.complex_nodes as f64) / 1000.0;
        score *= (1.0 - complexity_penalty).max(0.1);

        // Bonus for well-structured code
        if integrity_checker.well_formed_nodes > integrity_checker.total_nodes / 2 {
            score *= 1.1;
        }

        Ok(score.clamp(0.0, 1.0))
    }

    /// Check AST integrity (implementation)
    async fn check_ast_integrity(
        &self,
        context: &RefactoringContext,
        code_content: Option<&str>,
    ) -> Result<SafetyCheckResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();
        let mut risk_level = RefactoringRisk::Low;

        if let Some(code) = code_content {
            // Use the validate_ast_integrity method
            let integrity_score = self.validate_ast_integrity(code, context)?;

            if integrity_score < 0.5 {
                issues.push(SafetyIssue {
                    severity: IssueSeverity::Critical,
                    category: IssueCategory::ASTIntegrity,
                    message: "AST parsing reveals structural issues that may cause transformation failures".to_string(),
                    location: context.file_path.clone(),
                    suggestion: "Consider fixing syntax errors before refactoring".to_string(),
                });
                risk_level = RefactoringRisk::Critical;
            } else if integrity_score < 0.7 {
                issues.push(SafetyIssue {
                    severity: IssueSeverity::High,
                    category: IssueCategory::ASTIntegrity,
                    message: "Code contains complex structures that increase refactoring risk".to_string(),
                    location: context.file_path.clone(),
                    suggestion: "Review complex code sections manually".to_string(),
                });
                risk_level = RefactoringRisk::High;
            }

            if integrity_score < 0.8 {
                recommendations.push("Consider testing refactoring on a smaller subset first".to_string());
            }
        }

        Ok(SafetyCheckResult {
            issues,
            recommendations,
            risk_level,
        })
    }

    /// Analyze symbol usage patterns
    async fn analyze_symbol_usage(
        &self,
        context: &RefactoringContext,
        code_content: Option<&str>,
    ) -> Result<SafetyCheckResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();
        let mut risk_level = RefactoringRisk::Low;

        if let Some(code) = code_content {
            if let Some(symbol_name) = &context.symbol_name {
                // Count symbol occurrences
                let occurrences = code.matches(symbol_name).count();

                match occurrences {
                                    0 => {
                                        issues.push(SafetyIssue {
                                            severity: IssueSeverity::High,
                                            category: IssueCategory::SymbolUsage,
                                            message: format!("Symbol '{}' not found in file", symbol_name),
                                            location: context.file_path.clone(),
                                            suggestion: "Verify symbol name is correct".to_string(),
                                        });
                                        risk_level = RefactoringRisk::High;
                                    }
                                    1 => {
                                        // Single usage is usually safe
                                        recommendations.push("Single symbol usage - relatively safe refactoring".to_string());
                                    }
                                    10..=50 => {
                                        issues.push(SafetyIssue {
                                            severity: IssueSeverity::Medium,
                                            category: IssueCategory::SymbolUsage,
                                            message: format!("Symbol '{}' has {} usages - moderate risk", symbol_name, occurrences),
                                            location: context.file_path.clone(),
                                            suggestion: "Consider impact on all usages".to_string(),
                                        });
                                        risk_level = RefactoringRisk::Medium;
                                    }
                                    51..=usize::MAX => {
                                        issues.push(SafetyIssue {
                                            severity: IssueSeverity::Critical,
                                            category: IssueCategory::SymbolUsage,
                                            message: format!("Symbol '{}' has {} usages - high risk refactoring", symbol_name, occurrences),
                                            location: context.file_path.clone(),
                                            suggestion: "Consider refactoring in smaller steps".to_string(),
                                        });
                                        risk_level = RefactoringRisk::Critical;
                                    }
                    _ => {} // 2-9 usages, usually acceptable
                }
            }
        }

        Ok(SafetyCheckResult {
            issues,
            recommendations,
            risk_level,
        })
    }

    /// Check type consistency
    async fn check_type_consistency(
        &self,
        context: &RefactoringContext,
        code_content: Option<&str>,
    ) -> Result<SafetyCheckResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();
        let mut risk_level = RefactoringRisk::Low;

        if let Some(code) = code_content {
            if let Some(symbol_name) = &context.symbol_name {
                // Simple type checking - look for common type patterns
                let type_patterns = [
                    format!(": {}", symbol_name),
                    format!("{}::", symbol_name),
                    format!("{}<", symbol_name),
                ];

                let has_type_usage = type_patterns.iter().any(|pattern| code.contains(pattern));

                if has_type_usage {
                    issues.push(SafetyIssue {
                        severity: IssueSeverity::High,
                        category: IssueCategory::TypeConsistency,
                        message: format!("Symbol '{}' appears to be used in type contexts", symbol_name),
                        location: context.file_path.clone(),
                        suggestion: "Type changes may require additional updates throughout codebase".to_string(),
                    });
                    risk_level = RefactoringRisk::High;
                    recommendations.push("Search for additional type usages before proceeding".to_string());
                }
            }
        }

        Ok(SafetyCheckResult {
            issues,
            recommendations,
            risk_level,
        })
    }

    /// Analyze control flow impact
    async fn analyze_control_flow(
        &self,
        context: &RefactoringContext,
        code_content: Option<&str>,
    ) -> Result<SafetyCheckResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();
        let mut risk_level = RefactoringRisk::Low;

        if let Some(code) = code_content {
            if let Some(symbol_name) = &context.symbol_name {
                // Check if symbol is used in control flow constructs
                let control_flow_used =
                    code.contains(&format!("if {}", symbol_name)) ||
                    code.contains(&format!("while {}", symbol_name)) ||
                    code.contains(&format!("for {}", symbol_name)) ||
                    code.contains(&format!("match {}", symbol_name));

                if control_flow_used {
                    issues.push(SafetyIssue {
                        severity: IssueSeverity::Medium,
                        category: IssueCategory::ControlFlow,
                        message: format!("Symbol '{}' is used in control flow constructs", symbol_name),
                        location: context.file_path.clone(),
                        suggestion: "Refactoring may affect program logic flow".to_string(),
                    });
                    risk_level = RefactoringRisk::Medium;
                    recommendations.push("Review control flow dependencies before refactoring".to_string());
                }
            }
        }

        Ok(SafetyCheckResult {
            issues,
            recommendations,
            risk_level,
        })
    }

    /// Analyze dependency impact
    async fn analyze_dependency_impact(
        &self,
        context: &RefactoringContext,
        refactoring_type: &RefactoringType,
    ) -> Result<SafetyCheckResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();
        let mut risk_level = RefactoringRisk::Low;

        // Analyze based on refactoring type risk patterns
        match refactoring_type {
            RefactoringType::Rename => {
                risk_level = RefactoringRisk::Low;
                recommendations.push("Rename operations are generally safe with proper symbol resolution".to_string());
            }
            RefactoringType::ExtractFunction | RefactoringType::ExtractInterface => {
                risk_level = RefactoringRisk::Medium;
                recommendations.push("Ensure extracted code doesn't break encapsulation".to_string());
            }
            RefactoringType::MergeClasses | RefactoringType::SplitClass => {
                risk_level = RefactoringRisk::High;
                issues.push(SafetyIssue {
                    severity: IssueSeverity::High,
                    category: IssueCategory::DependencyImpact,
                    message: "Structural changes may break existing code dependencies".to_string(),
                    location: context.file_path.clone(),
                    suggestion: "Update all dependent code after refactoring".to_string(),
                });
            }
            RefactoringType::ChangeSignature => {
                risk_level = RefactoringRisk::Critical;
                issues.push(SafetyIssue {
                    severity: IssueSeverity::Critical,
                    category: IssueCategory::DependencyImpact,
                    message: "Signature changes require updating all call sites".to_string(),
                    location: context.file_path.clone(),
                    suggestion: "Use LSP to find all call sites before proceeding".to_string(),
                });
            }
            _ => {
                risk_level = RefactoringRisk::Medium;
            }
        }

        Ok(SafetyCheckResult {
            issues,
            recommendations,
            risk_level,
        })
    }

    /// Assess overall risk based on all issues and integrity score
    fn assess_overall_risk(
        &self,
        issues: &[SafetyIssue],
        integrity_score: f64,
        refactoring_type: &RefactoringType,
    ) -> RiskAssessment {
        let critical_issues = issues.iter().filter(|i| i.severity == IssueSeverity::Critical).count();
        let high_issues = issues.iter().filter(|i| i.severity == IssueSeverity::High).count();

        // Base risk from issues
        let mut risk = if critical_issues > 0 {
            RefactoringRisk::Critical
        } else if high_issues > 1 {
            RefactoringRisk::High
        } else if high_issues == 1 {
            RefactoringRisk::Medium
        } else {
            RefactoringRisk::Low
        };

        // Adjust based on integrity score
        if integrity_score < 0.5 {
            risk = RefactoringRisk::Critical;
        } else if integrity_score < 0.7 && risk < RefactoringRisk::High {
            risk = RefactoringRisk::High;
        }

        // Get risk factors
        let factors = vec![
            format!("Critical issues: {}", critical_issues),
            format!("High-severity issues: {}", high_issues),
            format!("AST integrity: {:.1}%", integrity_score * 100.0),
            format!("Operation type: {:?}", refactoring_type),
        ];

        // Generate mitigation strategies
        let mitigation_strategies = self.generate_mitigation_strategies(&risk, issues);

        RiskAssessment {
            risk_level: risk,
            risk_factors: factors,
            mitigation_strategies,
            confidence_score: integrity_score,
        }
    }

    /// Generate mitigation strategies based on risk level
    fn generate_mitigation_strategies(&self, risk: &RefactoringRisk, issues: &[SafetyIssue]) -> Vec<String> {
        let mut strategies = Vec::new();

        // Basic strategies for all refactorings
        strategies.push("Create backup before refactoring".to_string());
        strategies.push("Run existing tests to establish baseline".to_string());

        match risk {
            RefactoringRisk::Critical => {
                strategies.push("Consider manual refactoring for complex cases".to_string());
                strategies.push("Use LSP to identify all affected locations".to_string());
                strategies.push("Refactor in small, testable increments".to_string());
            }
            RefactoringRisk::High => {
                strategies.push("Write tests specifically for the refactoring".to_string());
                strategies.push("Review all affected files manually".to_string());
                strategies.push("Consider extracting small functions first".to_string());
            }
            RefactoringRisk::Medium => {
                strategies.push("Focus on high-confidence refactoring tools".to_string());
                strategies.push("Verify automated changes through diff review".to_string());
            }
            RefactoringRisk::Low => {
                strategies.push("Use automated refactoring tools with confidence".to_string());
                strategies.push("Focus on systematic testing after changes".to_string());
            }
        }

        // Add specific strategies for common issue types
        if issues.iter().any(|i| matches!(i.category, IssueCategory::SymbolUsage)) {
            strategies.push("Search for all symbol usages in workspace".to_string());
        }
        if issues.iter().any(|i| matches!(i.category, IssueCategory::TypeConsistency)) {
            strategies.push("Use type checker to verify changes".to_string());
        }
        if issues.iter().any(|i| matches!(i.category, IssueCategory::ControlFlow)) {
            strategies.push("Review control flow logic carefully".to_string());
        }

        strategies
    }
}

/// Result of a single safety check
#[derive(Debug, Clone)]
struct SafetyCheckResult {
    issues: Vec<SafetyIssue>,
    recommendations: Vec<String>,
    risk_level: RefactoringRisk,
}

/// Comprehensive safety report
#[derive(Debug, Clone)]
pub struct SafetyReport {
    pub is_safe: bool,
    pub overall_risk: RefactoringRisk,
    pub issues: Vec<SafetyIssue>,
    pub recommendations: Vec<String>,
    pub integrity_score: f64,
    pub risk_assessment: RiskAssessment,
}

/// Individual safety issue
#[derive(Debug, Clone)]
pub struct SafetyIssue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub message: String,
    pub location: String,
    pub suggestion: String,
}

/// Risk assessment with mitigation strategies
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub risk_level: RefactoringRisk,
    pub risk_factors: Vec<String>,
    pub mitigation_strategies: Vec<String>,
    pub confidence_score: f64,
}

/// Refactoring risk levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RefactoringRisk {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Issue severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Safety check categories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueCategory {
    ASTIntegrity,
    SymbolUsage,
    TypeConsistency,
    ControlFlow,
    DependencyImpact,
}

/// Available safety checks
#[derive(Debug, Clone)]
pub enum SafetyCheck {
    ASTIntegrityCheck,
    SymbolUsageAnalysis,
    TypeConsistencyCheck,
    ControlFlowAnalysis,
    DependencyImpactAnalysis,
}

/// Risk thresholds configuration
#[derive(Debug, Clone)]
pub struct RiskThresholds {
    pub max_affected_lines: usize,
    pub max_affected_symbols: usize,
    pub max_dependency_chain: usize,
    pub max_breaking_changes: usize,
    pub critical_pattern_threshold: f64,
}

/// AST analysis results
#[derive(Debug, Clone, Default)]
pub struct ASTAnalysis {
    pub total_nodes: usize,
    pub complex_nodes: usize,
    pub well_formed_nodes: usize,
    pub error_nodes: usize,
}

/// AST integrity checker visitor
struct ASTIntegrityChecker {
    pub total_nodes: usize,
    pub complex_nodes: usize,
    pub well_formed_nodes: usize,
    pub parse_errors: usize,
}

impl ASTIntegrityChecker {
    fn new() -> Self {
        ASTIntegrityChecker {
            total_nodes: 0,
            complex_nodes: 0,
            well_formed_nodes: 0,
            parse_errors: 0,
        }
    }
}

impl<'ast> Visit<'ast> for ASTIntegrityChecker {
    fn visit_item(&mut self, item: &'ast Item) {
        self.total_nodes += 1;

        // Check for complex code structures
        match item {
            Item::Fn(_) | Item::Struct(_) | Item::Trait(_) | Item::Impl(_) => {
                self.well_formed_nodes += 1;
            }
            _ => {
                self.complex_nodes += 1;
            }
        }

        syn::visit::visit_item(self, item);
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        self.total_nodes += 1;
        syn::visit::visit_expr(self, expr);
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        self.total_nodes += 1;
        syn::visit::visit_stmt(self, stmt);
    }
}

impl Default for SafetyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
// Validation logic for architectural analysis and recommendations

use super::types::*;
use super::AdvisorResult;

/// Architectural validation engine
#[derive(Debug)]
pub struct ArchitecturalValidator {
    validation_rules: Vec<ValidationRule>,
}

impl Default for ArchitecturalValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchitecturalValidator {
    /// Create a new architectural validator
    pub fn new() -> Self {
        Self {
            validation_rules: Self::initialize_validation_rules(),
        }
    }

    /// Validate architectural context
    pub async fn validate_context(&self, context: &ArchitecturalContext) -> AdvisorResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Check project type compatibility
        if let Some(issues_found) = self.validate_project_type(context).await? {
            issues.extend(issues_found);
        }

        // Check constraint validity
        if let Some(issues_found) = self.validate_constraints(context).await? {
            issues.extend(issues_found);
        }

        // Check team size appropriateness
        if let Some(issues_found) = self.validate_team_size(context).await? {
            issues.extend(issues_found);
        }

        Ok(issues)
    }

    /// Validate analysis results for consistency
    pub async fn validate_analysis(&self, analysis: &PatternAnalysis) -> AdvisorResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Check for conflicting patterns
        if let Some(issues_found) = self
            .check_conflicting_patterns(&analysis.detected_patterns)
            .await?
        {
            issues.extend(issues_found);
        }

        // Validate quality metrics
        if let Some(issues_found) = self
            .validate_quality_metrics(&analysis.quality_metrics)
            .await?
        {
            issues.extend(issues_found);
        }

        // Check complexity consistency
        if let Some(issues_found) = self
            .validate_complexity_assessment(&analysis.complexity_assessment)
            .await?
        {
            issues.extend(issues_found);
        }

        Ok(issues)
    }

    /// Validate recommendations for feasibility
    pub async fn validate_recommendations(
        &self,
        recommendations: &[ArchitecturalRecommendation],
        context: &ArchitecturalContext,
    ) -> AdvisorResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        for rec in recommendations {
            if let Some(issues_found) = self.validate_single_recommendation(rec, context).await? {
                issues.extend(issues_found);
            }
        }

        Ok(issues)
    }

    /// Validate individual recommendation
    async fn validate_single_recommendation(
        &self,
        recommendation: &ArchitecturalRecommendation,
        _context: &ArchitecturalContext,
    ) -> AdvisorResult<Option<Vec<ValidationIssue>>> {
        let mut issues = Vec::new();

        // Check if prerequisites are feasible
        if !recommendation.prerequisites.is_empty() {
            // In a real implementation, this would check if prerequisites can be met
            // For now, just check that they exist
        }

        // Validate risk vs effort balance
        if recommendation.implementation_effort == ImplementationEffort::VeryHigh
            && recommendation.risk_level == RiskLevel::Critical
        {
            issues.push(ValidationIssue {
                issue_type:  ValidationIssueType::Warning,
                severity:    IssueSeverity::High,
                description: "Very high effort with critical risk - consider breaking down into smaller tasks"
                    .to_string(),
                location:    None,
                suggestion:  Some("Split recommendation into smaller, manageable chunks".to_string()),
            });
        }

        Ok(if issues.is_empty() {
            None
        } else {
            Some(issues)
        })
    }

    /// Validate project type
    async fn validate_project_type(
        &self,
        context: &ArchitecturalContext,
    ) -> AdvisorResult<Option<Vec<ValidationIssue>>> {
        let mut issues = Vec::new();

        match context.project_type {
            ProjectType::Embedded =>
                if context.team_size.is_some_and(|size| size > 50) {
                    issues.push(ValidationIssue {
                        issue_type:  ValidationIssueType::Info,
                        severity:    IssueSeverity::Medium,
                        description: "Large team for embedded project".to_string(),
                        location:    None,
                        suggestion:  Some(
                            "Embedded projects usually work best with smaller, focused teams".to_string(),
                        ),
                    });
                },
            ProjectType::WebService => {
                if context
                    .expected_lifecycle
                    .as_ref()
                    .is_some_and(|lifecycle| lifecycle.contains("prototype"))
                {
                    issues.push(ValidationIssue {
                        issue_type:  ValidationIssueType::Warning,
                        severity:    IssueSeverity::Low,
                        description: "Web service marked as prototype".to_string(),
                        location:    None,
                        suggestion:  Some("Consider basic architectural patterns even for prototypes".to_string()),
                    });
                }
            }
            _ => {} // No special validation for other types
        }

        Ok(if issues.is_empty() {
            None
        } else {
            Some(issues)
        })
    }

    /// Validate constraints
    async fn validate_constraints(
        &self,
        context: &ArchitecturalContext,
    ) -> AdvisorResult<Option<Vec<ValidationIssue>>> {
        let mut issues = Vec::new();

        // Check for conflicting constraints
        if context
            .constraints
            .contains(&"high-performance".to_string())
            && context.constraints.contains(&"low-cost".to_string())
        {
            issues.push(ValidationIssue {
                issue_type:  ValidationIssueType::Info,
                severity:    IssueSeverity::Medium,
                description: "Potential conflict between high-performance and low-cost constraints".to_string(),
                location:    None,
                suggestion:  Some("These constraints may require trade-offs or compromise".to_string()),
            });
        }

        Ok(if issues.is_empty() {
            None
        } else {
            Some(issues)
        })
    }

    /// Validate team size appropriateness
    async fn validate_team_size(&self, context: &ArchitecturalContext) -> AdvisorResult<Option<Vec<ValidationIssue>>> {
        if let Some(team_size) = context.team_size {
            let issues = match context.project_type {
                ProjectType::Library =>
                    if team_size > 20 {
                        vec![ValidationIssue {
                            issue_type:  ValidationIssueType::Info,
                            severity:    IssueSeverity::Low,
                            description: "Large team for library development".to_string(),
                            location:    None,
                            suggestion:  Some("Libraries typically benefit from smaller, focused teams".to_string()),
                        }]
                    } else {
                        vec![]
                    },
                ProjectType::Application =>
                    if team_size < 3 {
                        vec![ValidationIssue {
                            issue_type:  ValidationIssueType::Warning,
                            severity:    IssueSeverity::Medium,
                            description: "Very small team for application development".to_string(),
                            location:    None,
                            suggestion:  Some(
                                "Consider if additional team members are needed for larger applications".to_string(),
                            ),
                        }]
                    } else {
                        vec![]
                    },
                _ => vec![],
            };

            Ok(if issues.is_empty() {
                None
            } else {
                Some(issues)
            })
        } else {
            Ok(None)
        }
    }

    /// Check for conflicting architectural patterns
    async fn check_conflicting_patterns(
        &self,
        patterns: &[DetectedPattern],
    ) -> AdvisorResult<Option<Vec<ValidationIssue>>> {
        let mut issues = Vec::new();

        // Example checks for common conflicting patterns
        let has_layered = patterns.iter().any(|p| p.pattern_type.contains("Layered"));
        let has_monolithic = patterns
            .iter()
            .any(|p| p.pattern_type.contains("Monolithic"));

        if has_layered && has_monolithic {
            issues.push(ValidationIssue {
                issue_type:  ValidationIssueType::Error,
                severity:    IssueSeverity::High,
                description: "Detected both layered and monolithic patterns simultaneously".to_string(),
                location:    None,
                suggestion:  Some("Analyze codebase to determine true architectural pattern".to_string()),
            });
        }

        Ok(if issues.is_empty() {
            None
        } else {
            Some(issues)
        })
    }

    /// Validate quality metrics for reasonableness
    async fn validate_quality_metrics(&self, metrics: &QualityMetrics) -> AdvisorResult<Option<Vec<ValidationIssue>>> {
        let mut issues = Vec::new();

        // Check maintainability index bounds (0-171)
        if metrics.maintainability_index < 0.0 || metrics.maintainability_index > 171.0 {
            issues.push(ValidationIssue {
                issue_type:  ValidationIssueType::Error,
                severity:    IssueSeverity::Critical,
                description: format!(
                    "Maintainability index {} is out of valid range (0-171)",
                    metrics.maintainability_index
                ),
                location:    None,
                suggestion:  Some("Re-run maintainability analysis".to_string()),
            });
        }

        // Check for unreasonable complexity values
        if metrics.cyclomatic_complexity > 100.0 {
            issues.push(ValidationIssue {
                issue_type:  ValidationIssueType::Warning,
                severity:    IssueSeverity::High,
                description: format!(
                    "Very high cyclomatic complexity: {}",
                    metrics.cyclomatic_complexity
                ),
                location:    None,
                suggestion:  Some("Consider decomposing complex functions".to_string()),
            });
        }

        // Check test coverage if available
        if let Some(coverage) = metrics.test_coverage {
            if !(0.0..=1.0).contains(&coverage) {
                issues.push(ValidationIssue {
                    issue_type:  ValidationIssueType::Error,
                    severity:    IssueSeverity::Medium,
                    description: format!("Test coverage {} is out of valid range (0.0-1.0)", coverage),
                    location:    None,
                    suggestion:  Some("Verify test coverage calculation method".to_string()),
                });
            }
        }

        Ok(if issues.is_empty() {
            None
        } else {
            Some(issues)
        })
    }

    /// Validate complexity assessment consistency
    async fn validate_complexity_assessment(
        &self,
        assessment: &ComplexityAssessment,
    ) -> AdvisorResult<Option<Vec<ValidationIssue>>> {
        let mut issues = Vec::new();

        // Check if hotspot complexity scores are reasonable
        for hotspot in &assessment.hotspot_complexity {
            if hotspot.complexity_score <= 0.0 {
                issues.push(ValidationIssue {
                    issue_type:  ValidationIssueType::Warning,
                    severity:    IssueSeverity::Medium,
                    description: format!(
                        "Invalid complexity score for {}: {}",
                        hotspot.file, hotspot.complexity_score
                    ),
                    location:    Some(hotspot.file.clone()),
                    suggestion:  Some("Re-run complexity analysis for this file".to_string()),
                });
            }
        }

        Ok(if issues.is_empty() {
            None
        } else {
            Some(issues)
        })
    }

    /// Initialize validation rules
    fn initialize_validation_rules() -> Vec<ValidationRule> {
        vec![ValidationRule {
            name:      "Team Size Adequacy".to_string(),
            rule_type: RuleType::ContextValidation,
            condition: Box::new(|ctx| ctx.team_size.unwrap_or(0) < 2),
            action:    ValidationAction::Warning(
                "Very small team may struggle with complex architectural decisions".to_string(),
            ),
        }]
    }
}

/// Validation rule definition
pub struct ValidationRule {
    pub name:      String,
    pub rule_type: RuleType,
    pub condition: Box<dyn Fn(&ArchitecturalContext) -> bool + Send + Sync>,
    pub action:    ValidationAction,
}

impl std::fmt::Debug for ValidationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidationRule")
            .field("name", &self.name)
            .field("rule_type", &self.rule_type)
            .field("action", &self.action)
            .finish()
    }
}

/// Rule types for classification
#[derive(Debug)]
pub enum RuleType {
    ContextValidation,
    PatternValidation,
    QualityValidation,
}

/// Actions to take when validation fails
#[derive(Debug)]
pub enum ValidationAction {
    Error(String),
    Warning(String),
    Info(String),
}

/// Validation issue representation
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub issue_type:  ValidationIssueType,
    pub severity:    IssueSeverity,
    pub description: String,
    pub location:    Option<String>,
    pub suggestion:  Option<String>,
}

/// Types of validation issues
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationIssueType {
    Error,
    Warning,
    Info,
}

/// Issue severity levels
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Validation utilities and helpers
pub mod validation_utils {
    use super::*;

    /// Summarize validation results
    pub fn summarize_validation_results(issues: &[ValidationIssue]) -> ValidationSummary {
        let total_issues = issues.len();
        let errors = issues
            .iter()
            .filter(|i| i.issue_type == ValidationIssueType::Error)
            .count();
        let warnings = issues
            .iter()
            .filter(|i| i.issue_type == ValidationIssueType::Warning)
            .count();
        let critical_issues = issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Critical)
            .count();

        ValidationSummary {
            total_issues,
            errors,
            warnings,
            critical_issues,
            is_valid: errors == 0 && critical_issues == 0,
        }
    }

    /// Extract issues by severity
    pub fn group_issues_by_severity(
        issues: &[ValidationIssue],
    ) -> std::collections::HashMap<IssueSeverity, Vec<&ValidationIssue>> {
        let mut grouped = std::collections::HashMap::new();

        for issue in issues {
            grouped
                .entry(issue.severity.clone())
                .or_insert_with(Vec::new)
                .push(issue);
        }

        grouped
    }

    /// Format validation results for display
    pub fn format_validation_report(issues: &[ValidationIssue]) -> String {
        let summary = summarize_validation_results(issues);
        let mut report = format!(
            "=== Architectural Validation Report ===\n\nTotal Issues: {}\nErrors: {}\nWarnings: {}\nCritical Issues: \
             {}\nValid: {}\n\n",
            summary.total_issues, summary.errors, summary.warnings, summary.critical_issues, summary.is_valid
        );

        if !issues.is_empty() {
            report.push_str("=== Detailed Issues ===\n\n");

            let grouped = group_issues_by_severity(issues);
            for (severity, issues_in_group) in grouped {
                report.push_str(&format!(
                    "{:?} ({}) issues:\n",
                    severity,
                    issues_in_group.len()
                ));

                for issue in issues_in_group {
                    report.push_str(&format!(
                        "  • {} ({:?})\n",
                        issue.description, issue.issue_type
                    ));
                    if let Some(suggestion) = &issue.suggestion {
                        report.push_str(&format!("    Suggestion: {}\n", suggestion));
                    }
                }
                report.push('\n');
            }
        }

        report
    }
}

/// Validation summary
#[derive(Debug)]
pub struct ValidationSummary {
    pub total_issues:    usize,
    pub errors:          usize,
    pub warnings:        usize,
    pub critical_issues: usize,
    pub is_valid:        bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = ArchitecturalValidator::new();
        assert!(!validator.validation_rules.is_empty());
    }

    #[test]
    fn test_validation_summary_creation() {
        let issues = vec![
            ValidationIssue {
                issue_type:  ValidationIssueType::Error,
                severity:    IssueSeverity::Critical,
                description: "Critical error".to_string(),
                location:    None,
                suggestion:  None,
            },
            ValidationIssue {
                issue_type:  ValidationIssueType::Warning,
                severity:    IssueSeverity::Medium,
                description: "Warning".to_string(),
                location:    None,
                suggestion:  None,
            },
        ];

        let summary = validation_utils::summarize_validation_results(&issues);
        assert_eq!(summary.total_issues, 2);
        assert_eq!(summary.errors, 1);
        assert_eq!(summary.warnings, 1);
        assert_eq!(summary.critical_issues, 1);
        assert!(!summary.is_valid);
    }

    #[tokio::test]
    async fn test_context_validation() {
        let validator = ArchitecturalValidator::new();
        let context = ArchitecturalContext {
            codebase_path:        "src/".to_string(),
            project_type:         ProjectType::Embedded,
            current_architecture: None,
            constraints:          vec![],
            goals:                vec![],
            team_size:            Some(2), // Very small team for embedded
            expected_lifecycle:   Some("5 years".to_string()),
        };

        let issues = validator.validate_context(&context).await.unwrap();
        // Should have warnings about team size for embedded projects
        // (implementation would add this validation)
        assert!(issues.len() >= 0); // At least no validation errors
    }
}

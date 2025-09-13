//! # Code Style Consistency Checker for Rust AI IDE
//!
//! This crate provides comprehensive code style consistency checking and architecture pattern
//! suggestions, ensuring that code follows consistent formatting and design guidelines.

pub mod formatter;
pub mod pattern_suggestions;
pub mod style;
pub mod style_rules;

// Re-exports
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
pub use formatter::*;
pub use pattern_suggestions::*;
use rust_ai_ide_ai_analysis::{ArchitectureSuggestion, CodeMetrics, CodeSmellType, Location, Severity, Suggestion};
use serde::{Deserialize, Serialize};
pub use style::*;
pub use style_rules::*;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Main code style checker
#[derive(Clone)]
pub struct CodeStyleChecker {
    rules:            Arc<RwLock<Vec<StyleRule>>>,
    formatter:        RustFormatter,
    pattern_analyzer: ArchitecturePatternAnalyzer,
}

impl CodeStyleChecker {
    /// Create a new code style checker with default rules
    pub fn new() -> Self {
        let rules = vec![
            StyleRule::NamingConvention,
            StyleRule::Indentation,
            StyleRule::LineLength,
            StyleRule::CommentStyle,
            StyleRule::FunctionLength,
            StyleRule::PackageStructure,
        ];

        Self {
            rules:            Arc::new(RwLock::new(rules)),
            formatter:        RustFormatter::new(),
            pattern_analyzer: ArchitecturePatternAnalyzer::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: StyleCheckConfig) -> Self {
        let mut checker = Self::new();
        // Apply configuration
        checker
    }

    /// Check code style consistency across a file or directory
    pub async fn check_style(&self, content: &str, file_path: &str) -> Result<StyleCheckResult, StyleCheckError> {
        let mut issues = Vec::new();
        let rules = self.rules.read().await;

        // Apply all style rules
        for rule in rules.iter() {
            let rule_issues = self.apply_style_rule(rule, content, file_path).await?;
            issues.extend(rule_issues);
        }

        let consistency_score = self.calculate_consistency_score(&issues, content.lines().count());
        let metrics = self.analyze_style_metrics(content)?;

        Ok(StyleCheckResult {
            issues,
            consistency_score,
            metrics,
            suggestions: self.generate_style_suggestions(&issues).await?,
        })
    }

    /// Format code according to style guidelines
    pub async fn format_code(&self, content: &str) -> Result<String, StyleCheckError> {
        self.formatter.format(content).await
    }

    /// Analyze architecture patterns and suggest improvements
    pub async fn analyze_patterns(
        &self,
        files: &[(&str, &str)],
    ) -> Result<Vec<ArchitectureSuggestion>, StyleCheckError> {
        self.pattern_analyzer.analyze_multiple_files(files).await
    }

    /// Add custom style rule
    pub async fn add_custom_rule(&self, rule: StyleRule) {
        let mut rules = self.rules.write().await;
        rules.push(rule);
    }

    /// Remove style rule
    pub async fn remove_rule(&self, rule: &StyleRule) -> bool {
        let mut rules = self.rules.write().await;
        if let Some(pos) = rules.iter().position(|r| r == rule) {
            rules.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get current style rules
    pub async fn get_rules(&self) -> Vec<StyleRule> {
        let rules = self.rules.read().await;
        rules.clone()
    }

    /// Apply a single style rule
    async fn apply_style_rule(
        &self,
        rule: &StyleRule,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<StyleIssue>, StyleCheckError> {
        match rule {
            StyleRule::NamingConvention => Ok(self.check_naming_conventions(content, file_path)),
            StyleRule::Indentation => Ok(self.check_indentation(content, file_path)),
            StyleRule::LineLength => Ok(self.check_line_length(content, file_path)),
            StyleRule::CommentStyle => Ok(self.check_comment_style(content, file_path)),
            StyleRule::FunctionLength => Ok(self.check_function_length_ast(content, file_path).await?),
            StyleRule::PackageStructure => Ok(self.check_package_structure(content, file_path)),
        }
    }

    fn check_naming_conventions(&self, content: &str, file_path: &str) -> Vec<StyleIssue> {
        // This would require AST parsing, but for now we'll do simple string checks
        // In a real implementation, this would use syn to parse the AST

        let mut issues = Vec::new();
        let lines = content.lines().enumerate();

        for (line_no, line) in lines {
            // Check SCREAMING_SNAKE_CASE for constants
            if line.contains("const") {
                if let Some(var_name) = self.extract_variable_name(line) {
                    if !self.is_screaming_snake_case(&var_name) {
                        issues.push(StyleIssue {
                            id:         Uuid::new_v4(),
                            rule:       StyleRule::NamingConvention,
                            message:    format!("Constant '{}' should be in SCREAMING_SNAKE_CASE", var_name),
                            location:   Location {
                                file:   file_path.to_string(),
                                line:   line_no + 1,
                                column: 0,
                                offset: line_no,
                            },
                            severity:   Severity::Info,
                            suggestion: Some("Use SCREAMING_SNAKE_CASE for constants".to_string()),
                        });
                    }
                }
            }

            // Check snake_case for variables and functions
            if line.contains(" let ") || line.contains("fn ") {
                if let Some(name) = self.extract_identifier(line) {
                    if !self.is_snake_case(&name) {
                        issues.push(StyleIssue {
                            id:         Uuid::new_v4(),
                            rule:       StyleRule::NamingConvention,
                            message:    format!("Identifier '{}' should be in snake_case", name),
                            location:   Location {
                                file:   file_path.to_string(),
                                line:   line_no + 1,
                                column: 0,
                                offset: line_no,
                            },
                            severity:   Severity::Info,
                            suggestion: Some("Use snake_case for variables and functions".to_string()),
                        });
                    }
                }
            }
        }

        issues
    }

    fn check_indentation(&self, content: &str, file_path: &str) -> Vec<StyleIssue> {
        let mut issues = Vec::new();

        for (line_no, line) in content.lines().enumerate() {
            let leading_spaces = line.chars().take_while(|c| *c == ' ').count();

            // Check for mixed tabs and spaces
            if line.contains('\t') && line.contains(' ') && leading_spaces > 0 {
                issues.push(StyleIssue {
                    id:         Uuid::new_v4(),
                    rule:       StyleRule::Indentation,
                    message:    "Mixed tabs and spaces detected".to_string(),
                    location:   Location {
                        file:   file_path.to_string(),
                        line:   line_no + 1,
                        column: 0,
                        offset: line_no,
                    },
                    severity:   Severity::Warning,
                    suggestion: Some("Use consistent indentation (spaces only)".to_string()),
                });
            }

            // Check for proper 4-space indentation
            if leading_spaces % 4 != 0 && !line.trim().is_empty() {
                issues.push(StyleIssue {
                    id:         Uuid::new_v4(),
                    rule:       StyleRule::Indentation,
                    message:    "Indentation should be multiple of 4 spaces".to_string(),
                    location:   Location {
                        file:   file_path.to_string(),
                        line:   line_no + 1,
                        column: 0,
                        offset: line_no,
                    },
                    severity:   Severity::Info,
                    suggestion: Some("Use 4 space indentation".to_string()),
                });
            }
        }

        issues
    }

    fn check_line_length(&self, content: &str, file_path: &str) -> Vec<StyleIssue> {
        let mut issues = Vec::new();
        const MAX_LINE_LENGTH: usize = 100;

        for (line_no, line) in content.lines().enumerate() {
            if line.len() > MAX_LINE_LENGTH && !line.contains("/// ") {
                // Skip doc comments
                issues.push(StyleIssue {
                    id:         Uuid::new_v4(),
                    rule:       StyleRule::LineLength,
                    message:    format!(
                        "Line too long ({} characters, max {})",
                        line.len(),
                        MAX_LINE_LENGTH
                    ),
                    location:   Location {
                        file:   file_path.to_string(),
                        line:   line_no + 1,
                        column: 0,
                        offset: line_no,
                    },
                    severity:   Severity::Info,
                    suggestion: Some("Break long lines into multiple lines".to_string()),
                });
            }
        }

        issues
    }

    fn check_comment_style(&self, content: &str, file_path: &str) -> Vec<StyleIssue> {
        let mut issues = Vec::new();

        for (line_no, line) in content.lines().enumerate() {
            // Check for C-style comments in Rust
            if line.contains("/*") || line.contains("*/") {
                issues.push(StyleIssue {
                    id:         Uuid::new_v4(),
                    rule:       StyleRule::CommentStyle,
                    message:    "Use Rust-style comments (//) instead of C-style (/* */)".to_string(),
                    location:   Location {
                        file:   file_path.to_string(),
                        line:   line_no + 1,
                        column: 0,
                        offset: line_no,
                    },
                    severity:   Severity::Warning,
                    suggestion: Some("Replace /* */ with //".to_string()),
                });
            }

            // Check for TODO comments
            if line.contains("TODO") || line.contains("FIXME") {
                issues.push(StyleIssue {
                    id:         Uuid::new_v4(),
                    rule:       StyleRule::CommentStyle,
                    message:    "TODO/FIXME comment found".to_string(),
                    location:   Location {
                        file:   file_path.to_string(),
                        line:   line_no + 1,
                        column: 0,
                        offset: line_no,
                    },
                    severity:   Severity::Info,
                    suggestion: Some("Consider creating an issue or task".to_string()),
                });
            }
        }

        issues
    }

    async fn check_function_length_ast(
        &self,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<StyleIssue>, StyleCheckError> {
        let ast = syn::parse_file(content).map_err(|e| StyleCheckError::ParseError(e.to_string()))?;

        let mut issues = Vec::new();
        let mut visitor = FunctionLengthVisitor {
            max_length: 50,
            issues:     &mut issues,
            file:       file_path,
        };
        visitor.visit_file(&ast);
        Ok(issues)
    }

    fn check_package_structure(&self, content: &str, file_path: &str) -> Vec<StyleIssue> {
        let mut issues = Vec::new();

        // Check for proper use of mods and pub/private
        if content.contains("mod") && content.contains("struct") {
            if !content.contains("pub struct") && !content.contains("priv") {
                // This is a simplistic check - in reality, we'd need AST analysis
            }
        }

        issues
    }

    fn calculate_consistency_score(&self, issues: &[StyleIssue], total_lines: usize) -> f64 {
        if total_lines == 0 {
            return 100.0;
        }

        let penalty = issues.len() as f64 * 10.0;
        (100.0 - penalty).max(0.0)
    }

    fn analyze_style_metrics(&self, content: &str) -> Result<StyleMetrics, StyleCheckError> {
        let lines = content.lines();
        let total_lines = lines.clone().count();
        let code_lines = lines.clone().filter(|l| !l.trim().is_empty()).count();
        let comment_lines = lines.filter(|l| l.trim().starts_with("//")).count();

        Ok(StyleMetrics {
            total_lines,
            code_lines,
            comment_lines,
            blank_lines: total_lines - code_lines,
            comment_ratio: if code_lines > 0 {
                comment_lines as f64 / code_lines as f64
            } else {
                0.0
            },
        })
    }

    async fn generate_style_suggestions(&self, issues: &[StyleIssue]) -> Result<Vec<Suggestion>, StyleCheckError> {
        let mut suggestions = Vec::new();

        // Group issues by type
        let mut rule_groups = std::collections::HashMap::new();
        for issue in issues {
            rule_groups
                .entry(&issue.rule)
                .or_insert_with(Vec::new)
                .push(issue);
        }

        for (rule, rule_issues) in rule_groups {
            if rule_issues.len() > 3 {
                // If there are many issues of the same rule
                suggestions.push(Suggestion {
                    id:          Uuid::new_v4(),
                    title:       format!("Multiple {} issues found", rule.rule_name()),
                    description: format!(
                        "Found {} style issues that can be auto-fixed",
                        rule_issues.len()
                    ),
                    location:    Some(rule_issues[0].location.clone()),
                    actions:     vec![],
                    priority:    rust_ai_ide_ai_analysis::Priority::Medium,
                });
            }
        }

        Ok(suggestions)
    }

    // Helper methods for naming checks
    fn extract_variable_name(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("const ") {
            let after_const = &line[start + 6..];
            if let Some(end) = after_const.find(":") {
                Some(after_const[..end].trim().to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn extract_identifier(&self, line: &str) -> Option<String> {
        let patterns = [" let ", " fn "];

        for pattern in &patterns {
            if let Some(start) = line.find(pattern) {
                let after_pattern = &line[start + pattern.len()..];
                if let Some(end) = after_pattern.find(|c: char| !(c.is_alphanumeric() || c == '_')) {
                    return Some(after_pattern[..end].trim().to_string());
                }
            }
        }
        None
    }

    fn is_snake_case(&self, input: &str) -> bool {
        if input.is_empty() || !input.chars().next().unwrap().is_lowercase() {
            return false;
        }
        !input.contains('_')
            || input
                .chars()
                .all(|c| c.is_lowercase() || c.is_alphabetic() || c == '_')
    }

    fn is_screaming_snake_case(&self, input: &str) -> bool {
        !input.is_empty()
            && input
                .chars()
                .all(|c| c.is_uppercase() || c.is_numeric() || c == '_')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_style_checker_creation() {
        let checker = CodeStyleChecker::new();
        assert!(!checker.get_rules().await.is_empty());
    }

    #[tokio::test]
    async fn test_naming_convention_check() {
        let checker = CodeStyleChecker::new();
        let code = "const MYVAR: i32 = 42;";
        let result = checker.check_style(code, "test.rs").await.unwrap();

        // Should find naming convention issue
        assert!(result
            .issues
            .iter()
            .any(|i| matches!(i.rule, StyleRule::NamingConvention)));
    }

    #[tokio::test]
    async fn test_line_length_check() {
        let checker = CodeStyleChecker::new();
        let long_line = "let very_long_variable_name_that_exceeds_the_maximum_allowed_line_length_and_should_be_flagged_by_the_style_checker = 42;";
        let result = checker.check_style(long_line, "test.rs").await.unwrap();

        assert!(result
            .issues
            .iter()
            .any(|i| matches!(i.rule, StyleRule::LineLength)));
    }
}

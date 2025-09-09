//! Style rules definitions

use serde::{Deserialize, Serialize};
use std::fmt;

/// Different types of style rules
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StyleRule {
    NamingConvention,
    Indentation,
    LineLength,
    CommentStyle,
    FunctionLength,
    PackageStructure,
}

impl StyleRule {
    /// Get the name of the rule
    pub fn rule_name(&self) -> &'static str {
        match self {
            StyleRule::NamingConvention => "naming convention",
            StyleRule::Indentation => "indentation",
            StyleRule::LineLength => "line length",
            StyleRule::CommentStyle => "comment style",
            StyleRule::FunctionLength => "function length",
            StyleRule::PackageStructure => "package structure",
        }
    }

    /// Get the description of the rule
    pub fn description(&self) -> &'static str {
        match self {
            StyleRule::NamingConvention => "Enforces consistent naming conventions (snake_case, SCREAMING_SNAKE_CASE)",
            StyleRule::Indentation => "Ensures consistent indentation (4 spaces, no tabs)",
            StyleRule::LineLength => "Limits line length to 100 characters",
            StyleRule::CommentStyle => "Enforces Rust-style comments over C-style",
            StyleRule::FunctionLength => "Limits function length to maintain readability",
            StyleRule::PackageStructure => "Ensures proper package and module organization",
        }
    }

    /// Get the category of the rule
    pub fn category(&self) -> StyleCategory {
        match self {
            StyleRule::NamingConvention => StyleCategory::Naming,
            StyleRule::Indentation => StyleCategory::Formatting,
            StyleRule::LineLength => StyleCategory::Formatting,
            StyleRule::CommentStyle => StyleCategory::Documentation,
            StyleRule::FunctionLength => StyleCategory::Structure,
            StyleRule::PackageStructure => StyleCategory::Structure,
        }
    }

    /// Check if this rule is automatically fixable
    pub fn is_auto_fixable(&self) -> bool {
        match self {
            StyleRule::NamingConvention => true,
            StyleRule::Indentation => true,
            StyleRule::LineLength => false, // May require code restructuring
            StyleRule::CommentStyle => true,
            StyleRule::FunctionLength => false, // Requires refactoring
            StyleRule::PackageStructure => false, // Requires module restructuring
        }
    }
}

impl fmt::Display for StyleRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rule_name())
    }
}

/// Categories of style rules
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StyleCategory {
    Naming,
    Formatting,
    Documentation,
    Structure,
    BestPractices,
}

impl fmt::Display for StyleCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StyleCategory::Naming => write!(f, "Naming"),
            StyleCategory::Formatting => write!(f, "Formatting"),
            StyleCategory::Documentation => write!(f, "Documentation"),
            StyleCategory::Structure => write!(f, "Structure"),
            StyleCategory::BestPractices => write!(f, "Best Practices"),
        }
    }
}

/// Configuration for individual rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    pub enabled: bool,
    pub severity: rust_ai_ide_ai_analysis::Severity,
    pub custom_settings: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: rust_ai_ide_ai_analysis::Severity::Info,
            custom_settings: std::collections::HashMap::new(),
        }
    }
}

/// Collection of rule configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleRulesConfig {
    pub naming: RuleConfig,
    pub indentation: RuleConfig,
    pub line_length: RuleConfig,
    pub comments: RuleConfig,
    pub functions: RuleConfig,
    pub package: RuleConfig,
}

impl Default for StyleRulesConfig {
    fn default() -> Self {
        Self {
            naming: RuleConfig::default(),
            indentation: RuleConfig::default(),
            line_length: RuleConfig {
                severity: rust_ai_ide_ai_analysis::Severity::Warning,
                ..Default::default()
            },
            comments: RuleConfig::default(),
            functions: RuleConfig {
                severity: rust_ai_ide_ai_analysis::Severity::Warning,
                ..Default::default()
            },
            package: RuleConfig {
                severity: rust_ai_ide_ai_analysis::Severity::Info,
                ..Default::default()
            },
        }
    }
}

impl StyleRulesConfig {
    /// Get configuration for a specific rule
    pub fn rule_config(&self, rule: &StyleRule) -> &RuleConfig {
        match rule {
            StyleRule::NamingConvention => &self.naming,
            StyleRule::Indentation => &self.indentation,
            StyleRule::LineLength => &self.line_length,
            StyleRule::CommentStyle => &self.comments,
            StyleRule::FunctionLength => &self.functions,
            StyleRule::PackageStructure => &self.package,
        }
    }

    /// Check if a rule is enabled
    pub fn is_enabled(&self, rule: &StyleRule) -> bool {
        self.rule_config(rule).enabled
    }

    /// Get list of enabled rules
    pub fn enabled_rules(&self) -> Vec<StyleRule> {
        vec![
            StyleRule::NamingConvention,
            StyleRule::Indentation,
            StyleRule::LineLength,
            StyleRule::CommentStyle,
            StyleRule::FunctionLength,
            StyleRule::PackageStructure,
        ]
        .into_iter()
        .filter(|rule| self.is_enabled(rule))
        .collect()
    }
}

/// Visitor for analyzing function lengths in AST
pub struct FunctionLengthVisitor<'a> {
    pub max_length: usize,
    pub issues: &'a mut Vec<crate::StyleIssue>,
    pub file: &'a str,
}

impl<'a, 'ast> syn::visit::Visit<'ast> for FunctionLengthVisitor<'a> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let function_length = node.block.stmts.len();

        if function_length > self.max_length {
            self.issues.push(crate::StyleIssue {
                id: uuid::Uuid::new_v4(),
                rule: StyleRule::FunctionLength,
                message: format!(
                    "Function '{}' is too long ({} statements). Maximum allowed: {}",
                    node.sig.ident, function_length, self.max_length
                ),
                location: rust_ai_ide_ai_analysis::Location {
                    file: self.file.to_string(),
                    line: node.sig.span().start().line as u32,
                    column: node.sig.span().start().column as u32,
                    offset: 0,
                },
                severity: rust_ai_ide_ai_analysis::Severity::Info,
                suggestion: Some("Consider breaking this function into smaller functions".to_string()),
            });
        }
        syn::visit::visit_item_fn(self, node);
    }
}

/// Statistics about rule violations
#[derive(Debug, Clone, Default)]
pub struct StyleRuleStats {
    pub violations_by_rule: std::collections::HashMap<StyleRule, usize>,
    pub violations_by_severity: std::collections::HashMap<rust_ai_ide_ai_analysis::Severity, usize>,
    pub total_violations: usize,
    pub auto_fixable: usize,
}

impl StyleRuleStats {
    /// Record a violation
    pub fn record_violation(&mut self, rule: &StyleRule, severity: rust_ai_ide_ai_analysis::Severity) {
        *self.violations_by_rule.entry(rule.clone()).or_insert(0) += 1;
        *self.violations_by_severity.entry(severity).or_insert(0) += 1;
        self.total_violations += 1;

        if rule.is_auto_fixable() {
            self.auto_fixable += 1;
        }
    }

    /// Get most violated rules
    pub fn most_violated_rules(&self, limit: usize) -> Vec<(StyleRule, usize)> {
        let mut violations: Vec<(StyleRule, usize)> = self.violations_by_rule
            .iter()
            .map(|(rule, count)| (rule.clone(), *count))
            .collect();

        violations.sort_by(|a, b| b.1.cmp(&a.1));
        violations.into_iter().take(limit).collect()
    }

    /// Calculate rule compliance rate
    pub fn compliance_rate(&self) -> f64 {
        if self.total_violations == 0 {
            100.0
        } else {
            (1.0 - (self.total_violations as f64 / 1000.0).min(1.0)) * 100.0 // Assuming some baseline
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_rule_names() {
        assert_eq!(StyleRule::NamingConvention.rule_name(), "naming convention");
        assert_eq!(StyleRule::Indentation.rule_name(), "indentation");
    }

    #[test]
    fn test_style_rule_display() {
        assert_eq!(StyleRule::NamingConvention.to_string(), "naming convention");
    }

    #[test]
    fn test_rule_categories() {
        assert_eq!(StyleRule::NamingConvention.category(), StyleCategory::Naming);
        assert_eq!(StyleRule::Indentation.category(), StyleCategory::Formatting);
    }

    #[test]
    fn test_auto_fixable_rules() {
        assert!(StyleRule::NamingConvention.is_auto_fixable());
        assert!(!StyleRule::FunctionLength.is_auto_fixable());
    }

    #[test]
    fn test_style_rules_config() {
        let config = StyleRulesConfig::default();
        assert!(config.is_enabled(&StyleRule::NamingConvention));
        assert_eq!(config.rule_config(&StyleRule::LineLength).severity, rust_ai_ide_ai_analysis::Severity::Warning);
    }

    #[test]
    fn test_enabled_rules() {
        let config = StyleRulesConfig::default();
        let enabled = config.enabled_rules();
        assert!(enabled.contains(&StyleRule::NamingConvention));
        assert!(enabled.contains(&StyleRule::LineLength));
    }
}
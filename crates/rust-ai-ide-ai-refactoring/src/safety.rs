//! Safety analysis for refactoring operations

use std::collections::HashSet;

use crate::types::*;

/// Safety analyzer for refactoring operations
pub struct SafetyAnalyzer {
    /// Risk assessment rules
    risk_rules: Vec<RiskRule>,
}

/// Refactoring risk levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefactoringRisk {
    Low,
    Medium,
    High,
    Critical,
}

/// Risk assessment rule
#[derive(Debug, Clone)]
struct RiskRule {
    condition:   String,
    risk_level:  RefactoringRisk,
    description: String,
}

/// Comprehensive safety analysis result
#[derive(Debug, Clone)]
pub struct SafetyAnalysisResult {
    pub overall_risk:                RefactoringRisk,
    pub risk_score:                  f64,
    pub safety_concerns:             Vec<String>,
    pub recommended_actions:         Vec<String>,
    pub breaking_change_probability: f64,
}

impl SafetyAnalyzer {
    pub fn new() -> Self {
        let risk_rules = vec![
            RiskRule {
                condition:   "public_api_change".to_string(),
                risk_level:  RefactoringRisk::High,
                description: "Changes to public APIs may break external consumers".to_string(),
            },
            RiskRule {
                condition:   "multiple_file_impact".to_string(),
                risk_level:  RefactoringRisk::Medium,
                description: "Refactoring affects multiple files, increasing complexity".to_string(),
            },
            RiskRule {
                condition:   "unsafe_code_involved".to_string(),
                risk_level:  RefactoringRisk::High,
                description: "Unsafe code blocks require careful review".to_string(),
            },
            RiskRule {
                condition:   "macro_usage".to_string(),
                risk_level:  RefactoringRisk::Medium,
                description: "Macros can have complex expansion patterns".to_string(),
            },
            RiskRule {
                condition:   "async_boundary_crossing".to_string(),
                risk_level:  RefactoringRisk::Medium,
                description: "Crossing async boundaries may change execution semantics".to_string(),
            },
        ];

        SafetyAnalyzer { risk_rules }
    }

    /// Comprehensive safety analysis
    pub async fn analyze_safety(&self, context: &RefactoringContext) -> Result<(), String> {
        let analysis_result = self.perform_comprehensive_analysis(context).await?;

        // Log safety concerns
        if !analysis_result.safety_concerns.is_empty() {
            println!("Safety Analysis - Concerns found:");
            for concern in &analysis_result.safety_concerns {
                println!("  - {}", concern);
            }
        }

        // Reject if risk is too high
        if matches!(analysis_result.overall_risk, RefactoringRisk::Critical) {
            return Err(format!(
                "Critical safety risk detected: {}",
                analysis_result.safety_concerns.join(", ")
            ));
        }

        Ok(())
    }

    /// Perform comprehensive safety analysis
    pub async fn perform_comprehensive_analysis(
        &self,
        context: &RefactoringContext,
    ) -> Result<SafetyAnalysisResult, String> {
        let mut safety_concerns = Vec::new();
        let mut recommended_actions = Vec::new();
        let mut risk_factors = Vec::new();

        // Analyze file content for safety concerns
        if let Ok(content) = std::fs::read_to_string(&context.file_path) {
            // Check for unsafe code
            if content.contains("unsafe") {
                safety_concerns.push("File contains unsafe code blocks".to_string());
                recommended_actions.push("Review unsafe code blocks for correctness".to_string());
                risk_factors.push(RefactoringRisk::High);
            }

            // Check for public APIs
            if self.contains_public_apis(&content) {
                safety_concerns.push("Potential changes to public APIs detected".to_string());
                recommended_actions.push("Consider versioning impact on public interfaces".to_string());
                risk_factors.push(RefactoringRisk::High);
            }

            // Check for macro usage
            if content.contains("macro_rules!") || content.contains("derive(") {
                safety_concerns.push("File contains macro definitions or derives".to_string());
                recommended_actions.push("Verify macro expansion patterns".to_string());
                risk_factors.push(RefactoringRisk::Medium);
            }

            // Check for async code
            if content.contains("async") || content.contains("await") {
                safety_concerns.push("File contains async code".to_string());
                recommended_actions.push("Review async execution semantics".to_string());
                risk_factors.push(RefactoringRisk::Medium);
            }

            // Check for complex control flow
            if self.has_complex_control_flow(&content) {
                safety_concerns.push("Complex control flow detected".to_string());
                recommended_actions.push("Consider adding test coverage for edge cases".to_string());
                risk_factors.push(RefactoringRisk::Medium);
            }
        }

        // Check for cross-file dependencies
        if self.has_cross_file_dependencies(context) {
            safety_concerns.push("Cross-file dependencies detected".to_string());
            recommended_actions.push("Verify all dependent files are updated".to_string());
            risk_factors.push(RefactoringRisk::Medium);
        }

        // Calculate overall risk
        let overall_risk = self.calculate_overall_risk(&risk_factors);
        let risk_score = self.calculate_risk_score(&overall_risk);
        let breaking_change_probability = self.estimate_breaking_change_probability(&risk_factors);

        Ok(SafetyAnalysisResult {
            overall_risk,
            risk_score,
            safety_concerns,
            recommended_actions,
            breaking_change_probability,
        })
    }

    /// Check if file contains public APIs
    fn contains_public_apis(&self, content: &str) -> bool {
        // Parse with syn to find public items
        if let Ok(syntax) = syn::parse_file(content) {
            for item in &syntax.items {
                match item {
                    syn::Item::Fn(fn_item) =>
                        if matches!(fn_item.vis, syn::Visibility::Public(_)) {
                            return true;
                        },
                    syn::Item::Struct(struct_item) =>
                        if matches!(struct_item.vis, syn::Visibility::Public(_)) {
                            return true;
                        },
                    syn::Item::Enum(enum_item) =>
                        if matches!(enum_item.vis, syn::Visibility::Public(_)) {
                            return true;
                        },
                    syn::Item::Trait(trait_item) =>
                        if matches!(trait_item.vis, syn::Visibility::Public(_)) {
                            return true;
                        },
                    syn::Item::Impl(_) => {
                        // Impl blocks can expose public methods
                        return true;
                    }
                    _ => {}
                }
            }
        }
        false
    }

    /// Check for complex control flow patterns
    fn has_complex_control_flow(&self, content: &str) -> bool {
        let complex_patterns = [
            "match.*{.*match", // Nested matches
            "if.*{.*if",       // Nested ifs
            "loop.*{.*loop",   // Nested loops
            "while.*{.*while", // Nested whiles
        ];

        complex_patterns
            .iter()
            .any(|pattern| content.contains(pattern))
    }

    /// Check for cross-file dependencies
    fn has_cross_file_dependencies(&self, context: &RefactoringContext) -> bool {
        // Simple heuristic: if symbol name is provided, likely has dependencies
        context.symbol_name.is_some()
    }

    /// Calculate overall risk from multiple factors
    fn calculate_overall_risk(&self, risk_factors: &[RefactoringRisk]) -> RefactoringRisk {
        if risk_factors.is_empty() {
            return RefactoringRisk::Low;
        }

        let mut max_risk = RefactoringRisk::Low;
        for risk in risk_factors {
            match risk {
                RefactoringRisk::Critical => return RefactoringRisk::Critical,
                RefactoringRisk::High => max_risk = RefactoringRisk::High,
                RefactoringRisk::Medium =>
                    if matches!(max_risk, RefactoringRisk::Low) {
                        max_risk = RefactoringRisk::Medium;
                    },
                RefactoringRisk::Low => {}
            }
        }
        max_risk
    }

    /// Calculate numerical risk score
    fn calculate_risk_score(&self, risk: &RefactoringRisk) -> f64 {
        match risk {
            RefactoringRisk::Low => 0.2,
            RefactoringRisk::Medium => 0.5,
            RefactoringRisk::High => 0.8,
            RefactoringRisk::Critical => 1.0,
        }
    }

    /// Estimate probability of breaking changes
    fn estimate_breaking_change_probability(&self, risk_factors: &[RefactoringRisk]) -> f64 {
        if risk_factors.is_empty() {
            return 0.1;
        }

        let high_critical_count = risk_factors
            .iter()
            .filter(|r| matches!(r, RefactoringRisk::High | RefactoringRisk::Critical))
            .count();

        let total_factors = risk_factors.len();
        (high_critical_count as f64) / (total_factors as f64)
    }
}

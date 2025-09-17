use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use syn::visit::Visit;
use syn::{parse_file, File as SynFile, Ident, Path};
use tokio::sync::Mutex;

use crate::ai_suggester::AnalysisContext;
use crate::error::{AnalysisError, AnalysisResult};
use crate::types::{FileDependency, ImpactAssessment, ImpactSeverity, RefactoringSuggestion};

/// Impact assessment component using syn AST analysis
pub struct RefactoringImpactAssessor {
    cache: Arc<Mutex<HashMap<String, ImpactAssessment>>>,
}

impl RefactoringImpactAssessor {
    /// Create a new impact assessor
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Assess the impact of a set of refactoring suggestions
    pub async fn assess_impact(
        &self,
        suggestions: &[RefactoringSuggestion],
        context: &AnalysisContext,
    ) -> AnalysisResult<ImpactAssessment> {
        let mut total_cost = 0.0;
        let mut total_benefit = 0.0;
        let mut all_dependencies = Vec::new();
        let mut risk_score = 0.0;

        for suggestion in suggestions {
            let assessment = self.assess_single_suggestion(suggestion, context).await?;
            total_cost += assessment.cost_benefit_analysis.development_cost;
            total_benefit += assessment.cost_benefit_analysis.maintainability_improvement;
            all_dependencies.extend(assessment.dependency_chain);
            risk_score += assessment.risk_assessment.likelihood_of_failure;
        }

        // Aggregate results
        let overall_impact_score =
            self.calculate_overall_impact_score(suggestions.len(), &all_dependencies);
        let cost_benefit = self.calculate_cost_benefit_analysis(total_cost, total_benefit);
        let risk_assessment = self.calculate_risk_assessment(risk_score, suggestions.len());

        Ok(ImpactAssessment {
            assessment_id: uuid::Uuid::new_v4(),
            suggestion_id: suggestions.first().map(|s| s.id).unwrap_or_default(), // Use first suggestion ID
            overall_impact_score,
            cost_benefit_analysis: cost_benefit,
            dependency_chain: all_dependencies,
            performance_impact: Default::default(), // Would be calculated separately
            risk_assessment,
            timeline_estimate: Default::default(), // Would be calculated separately
        })
    }

    /// Assess impact for a single suggestion
    async fn assess_single_suggestion(
        &self,
        suggestion: &RefactoringSuggestion,
        context: &AnalysisContext,
    ) -> AnalysisResult<ImpactAssessment> {
        // Parse the file to analyze dependencies
        let content = std::fs::read_to_string(&suggestion.target_file).map_err(|e| {
            AnalysisError::DataProcessing {
                stage: format!("Failed to read file {}: {}", suggestion.target_file, e),
            }
        })?;

        let syntax_tree: SynFile =
            parse_file(&content).map_err(|e| AnalysisError::DataProcessing {
                stage: format!("Failed to parse file {}: {}", suggestion.target_file, e),
            })?;

        // Use syn visitor to analyze dependencies
        let mut visitor = DependencyVisitor::new();
        visitor.visit_file(&syntax_tree);

        // Calculate costs and benefits based on suggestion type and dependencies
        let cost_benefit =
            self.calculate_suggestion_cost_benefit(suggestion, &visitor.dependencies);
        let risk_assessment = self.calculate_suggestion_risk(suggestion, &visitor.dependencies);

        // Create dependency chain
        let dependency_chain =
            self.build_dependency_chain(&visitor.dependencies, &suggestion.target_file);

        Ok(ImpactAssessment {
            assessment_id: uuid::Uuid::new_v4(),
            suggestion_id: suggestion.id,
            overall_impact_score: self
                .calculate_single_impact_score(&cost_benefit, &risk_assessment),
            cost_benefit_analysis: cost_benefit,
            dependency_chain,
            performance_impact: Default::default(),
            risk_assessment,
            timeline_estimate: Default::default(),
        })
    }

    /// Calculate cost-benefit analysis for a suggestion
    fn calculate_suggestion_cost_benefit(
        &self,
        suggestion: &RefactoringSuggestion,
        dependencies: &HashSet<String>,
    ) -> crate::types::CostBenefitAnalysis {
        let base_cost = match suggestion.suggestion_type {
            crate::types::RefactoringType::ExtractMethod => 2.0,
            crate::types::RefactoringType::RenameSymbol => 1.0,
            crate::types::RefactoringType::ExtractVariable => 1.5,
            _ => 2.5,
        };

        // Factor in complexity
        let complexity_multiplier = match suggestion.estimated_complexity {
            crate::types::Complexity::Trivial => 0.5,
            crate::types::Complexity::Simple => 1.0,
            crate::types::Complexity::Moderate => 1.5,
            crate::types::Complexity::Complex => 2.0,
            crate::types::Complexity::High => 3.0,
            crate::types::Complexity::VeryHigh => 4.0,
        };

        let development_cost =
            base_cost * complexity_multiplier * (dependencies.len() as f64 + 1.0);

        // Benefits are harder to quantify, use heuristics
        let maintainability_improvement = match suggestion.suggestion_type {
            crate::types::RefactoringType::ExtractMethod => 3.0,
            crate::types::RefactoringType::RenameSymbol => 2.0,
            crate::types::RefactoringType::ExtractVariable => 1.5,
            _ => 2.5,
        };

        crate::types::CostBenefitAnalysis {
            development_cost,
            maintenance_cost: development_cost * 0.3, // Ongoing maintenance
            performance_benefit: 0.0,                 // Not calculated here
            maintainability_improvement,
            breaking_change_risk: if dependencies.is_empty() { 0.1 } else { 0.3 },
            net_benefit_score: maintainability_improvement - development_cost,
        }
    }

    /// Calculate risk assessment for a suggestion
    fn calculate_suggestion_risk(
        &self,
        suggestion: &RefactoringSuggestion,
        dependencies: &HashSet<String>,
    ) -> crate::types::RiskAssessment {
        let base_risk = match suggestion.risk_level {
            crate::types::RiskLevel::Low => 0.2,
            crate::types::RiskLevel::Medium => 0.5,
            crate::types::RiskLevel::High => 0.8,
            crate::types::RiskLevel::Critical => 0.95,
        };

        let dependency_risk = (dependencies.len() as f64) * 0.1;
        let likelihood_of_failure = (base_risk + dependency_risk).min(1.0);

        crate::types::RiskAssessment {
            likelihood_of_failure,
            impact_if_failed: base_risk * 0.8,
            mitigation_strategies: vec![], // Would be populated with actual strategies
            confidence_in_assessment: 0.7, // Moderate confidence
        }
    }

    /// Build dependency chain from analyzed dependencies
    fn build_dependency_chain(
        &self,
        dependencies: &HashSet<String>,
        file_path: &str,
    ) -> Vec<FileDependency> {
        dependencies
            .iter()
            .map(|dep| FileDependency {
                file_path: dep.clone(),
                dependency_type: crate::types::DependencyType::SymbolReference,
                impact_severity: ImpactSeverity::Medium,
                lines_affected: vec![], // Would need more detailed analysis
            })
            .collect()
    }

    /// Calculate overall impact score
    fn calculate_overall_impact_score(
        &self,
        suggestion_count: usize,
        dependencies: &[FileDependency],
    ) -> f64 {
        let base_score = suggestion_count as f64 * 10.0;
        let dependency_penalty = dependencies.len() as f64 * 2.0;
        (base_score - dependency_penalty).max(0.0)
    }

    /// Calculate aggregated cost-benefit analysis
    fn calculate_cost_benefit_analysis(
        &self,
        total_cost: f64,
        total_benefit: f64,
    ) -> crate::types::CostBenefitAnalysis {
        crate::types::CostBenefitAnalysis {
            development_cost: total_cost,
            maintenance_cost: total_cost * 0.3,
            performance_benefit: 0.0,
            maintainability_improvement: total_benefit,
            breaking_change_risk: 0.2,
            net_benefit_score: total_benefit - total_cost,
        }
    }

    /// Calculate aggregated risk assessment
    fn calculate_risk_assessment(
        &self,
        total_risk: f64,
        suggestion_count: usize,
    ) -> crate::types::RiskAssessment {
        let avg_risk = if suggestion_count > 0 {
            total_risk / suggestion_count as f64
        } else {
            0.0
        };

        crate::types::RiskAssessment {
            likelihood_of_failure: avg_risk,
            impact_if_failed: avg_risk * 0.7,
            mitigation_strategies: vec![],
            confidence_in_assessment: 0.6,
        }
    }

    /// Calculate impact score for a single suggestion
    fn calculate_single_impact_score(
        &self,
        cost_benefit: &crate::types::CostBenefitAnalysis,
        risk: &crate::types::RiskAssessment,
    ) -> f64 {
        let benefit_score = cost_benefit.net_benefit_score * 10.0;
        let risk_penalty = risk.likelihood_of_failure * 20.0;
        (benefit_score - risk_penalty).max(0.0)
    }
}

/// Visitor to analyze dependencies using syn 2.x visit trait
struct DependencyVisitor {
    dependencies: HashSet<String>,
    current_file: String,
}

impl DependencyVisitor {
    fn new() -> Self {
        Self {
            dependencies: HashSet::new(),
            current_file: String::new(),
        }
    }
}

impl<'ast> Visit<'ast> for DependencyVisitor {
    fn visit_path(&mut self, node: &'ast Path) {
        // Track paths that might reference external dependencies
        if let Some(segment) = node.segments.first() {
            let ident = segment.ident.to_string();
            // Simple heuristic: if it starts with uppercase or is a common dependency pattern
            if ident.chars().next().unwrap_or(' ').is_uppercase() || ident.contains("::") {
                self.dependencies.insert(ident);
            }
        }
        syn::visit::visit_path(self, node);
    }

    fn visit_ident(&mut self, node: &'ast Ident) {
        // Track identifiers that might be function calls or type references
        let name = node.to_string();
        if name.chars().next().unwrap_or(' ').is_uppercase() {
            // Likely a type reference
            self.dependencies.insert(name);
        }
        syn::visit::visit_ident(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_assessor_creation() {
        let assessor = RefactoringImpactAssessor::new();
        let cache = assessor.cache.lock().await;
        assert!(cache.is_empty());
    }

    #[tokio::test]
    async fn test_dependency_visitor() {
        let code = r#"
            use std::collections::HashMap;
            use some_crate::SomeType;

            fn test() {
                let map: HashMap<String, i32> = HashMap::new();
                let value: SomeType = SomeType::new();
            }
        "#;

        let syntax_tree: SynFile = parse_file(code).unwrap();
        let mut visitor = DependencyVisitor::new();
        visitor.visit_file(&syntax_tree);

        // Should find HashMap and SomeType as dependencies
        assert!(visitor.dependencies.contains("HashMap"));
        assert!(visitor.dependencies.contains("SomeType"));
    }
}

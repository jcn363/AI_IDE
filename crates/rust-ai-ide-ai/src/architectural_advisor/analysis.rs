// Analysis algorithms and metrics calculators for architectural assessment

use std::collections::HashMap;

use super::patterns::*;
use super::types::*;
use super::AdvisorResult;

/// Metrics analyzer for calculating codebase quality metrics
#[derive(Debug)]
pub struct MetricsAnalyzer {
    metric_calculators: HashMap<String, Box<dyn MetricCalculator>>,
}

impl Default for MetricsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsAnalyzer {
    /// Create a new metrics analyzer
    pub fn new() -> Self {
        let mut metric_calculators = HashMap::new();
        metric_calculators.insert(
            "maintainability".to_string(),
            Box::new(MaintainabilityCalculator) as Box<dyn MetricCalculator>,
        );
        metric_calculators.insert(
            "complexity".to_string(),
            Box::new(ComplexityCalculator) as Box<dyn MetricCalculator>,
        );
        metric_calculators.insert(
            "coupling".to_string(),
            Box::new(CouplingCalculator) as Box<dyn MetricCalculator>,
        );

        Self { metric_calculators }
    }

    /// Calculate quality metrics for the given context
    pub async fn calculate_metrics(&self, context: &ArchitecturalContext) -> AdvisorResult<QualityMetrics> {
        let lines_of_code = self.calculate_lines_of_code(&context.codebase_path).await?;
        let cyclomatic_complexity = self
            .calculate_cyclomatic_complexity(&context.codebase_path)
            .await?;
        let halstead_complexity = self
            .calculate_halstead_complexity(&context.codebase_path)
            .await?;
        let maintainability_index = self.calculate_maintainability_index(lines_of_code, cyclomatic_complexity)?;
        let technical_debt_ratio = self
            .calculate_technical_debt_ratio(&context.codebase_path)
            .await?;
        let test_coverage = self.calculate_test_coverage(&context.codebase_path).await;

        Ok(QualityMetrics {
            maintainability_index,
            cyclomatic_complexity,
            halstead_complexity,
            lines_of_code,
            technical_debt_ratio,
            test_coverage,
        })
    }

    /// Assess overall system complexity
    pub async fn assess_complexity(&self, codebase_analysis: &CodebaseAnalysis) -> AdvisorResult<ComplexityAssessment> {
        let overall_complexity = self.calculate_overall_complexity(codebase_analysis);
        let hotspot_complexity = self.find_complexity_hotspots(codebase_analysis);
        let complexity_trends = self.analyze_complexity_trends(codebase_analysis);

        Ok(ComplexityAssessment {
            overall_complexity,
            hotspot_complexity,
            complexity_trends,
        })
    }

    /// Calculate the number of lines of code
    async fn calculate_lines_of_code(&self, _path: &str) -> AdvisorResult<usize> {
        // In a real implementation, this would read and count lines from files
        // For now, return a placeholder value
        Ok(5000)
    }

    /// Calculate cyclomatic complexity
    async fn calculate_cyclomatic_complexity(&self, _path: &str) -> AdvisorResult<f32> {
        // In a real implementation, this would analyze control flow complexity
        Ok(15.0)
    }

    /// Calculate Halstead complexity metrics
    async fn calculate_halstead_complexity(&self, _path: &str) -> AdvisorResult<f32> {
        // In a real implementation, this would calculate Halstead complexity measures
        Ok(25.0)
    }

    /// Calculate technical debt ratio
    async fn calculate_technical_debt_ratio(&self, _path: &str) -> AdvisorResult<f32> {
        // In a real implementation, this would analyze code quality issues
        Ok(0.1)
    }

    /// Calculate test coverage (if available)
    async fn calculate_test_coverage(&self, _path: &str) -> Option<f32> {
        // In a real implementation, this would run test coverage analysis
        Some(0.75)
    }

    /// Calculate maintainability index
    fn calculate_maintainability_index(&self, loc: usize, cc: f32) -> AdvisorResult<f32> {
        // MI = 171 - 5.2 * ln(V) - 0.23 * G - 16.2 * ln(LOC) + 50 * sin(sqrt(2.4 * CC))
        // For simplicity, using a basic calculation
        let loc_log = (loc as f32).ln().max(0.1);
        let cc_factor = (2.4 * cc).sqrt().sin() * 50.0;
        let mi = 171.0 - (5.2 * (loc as f32).ln()) - 0.23 - (16.2 * loc_log) + cc_factor;

        Ok(mi.clamp(0.0, 171.0))
    }

    /// Calculate overall system complexity
    fn calculate_overall_complexity(&self, analysis: &CodebaseAnalysis) -> ComplexityLevel {
        let file_count = analysis.directory_structure.total_files;
        let avg_complexity = if file_count > 0 {
            analysis
                .directory_structure
                .file_types
                .values()
                .sum::<usize>() as f32
                / file_count as f32
        } else {
            0.0
        };

        match avg_complexity {
            0.0..=10.0 => ComplexityLevel::Low,
            10.1..=25.0 => ComplexityLevel::Moderate,
            25.1..=50.0 => ComplexityLevel::High,
            _ => ComplexityLevel::VeryHigh,
        }
    }

    /// Find complexity hotspots in the codebase
    fn find_complexity_hotspots(&self, _analysis: &CodebaseAnalysis) -> Vec<ComplexityHotspot> {
        // In a real implementation, this would identify files with high complexity
        vec![ComplexityHotspot {
            file:             "src/complex_module.rs".to_string(),
            complexity_score: 35.7,
            description:      "High cyclomatic complexity due to nested conditionals".to_string(),
        }]
    }

    /// Analyze complexity trends over time
    fn analyze_complexity_trends(&self, _analysis: &CodebaseAnalysis) -> Vec<ComplexityTrend> {
        // In a real implementation, this would analyze complexity changes over time
        vec![ComplexityTrend {
            period:            "Last 3 months".to_string(),
            complexity_change: 5.2,
            description:       "Moderate increase in complexity due to feature additions".to_string(),
        }]
    }
}

/// Decision engine for architectural decision analysis
#[derive(Debug)]
pub struct DecisionEngine {
    knowledge_base: HashMap<String, DecisionTemplate>,
}

impl Default for DecisionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DecisionEngine {
    /// Create a new decision engine
    pub fn new() -> Self {
        Self {
            knowledge_base: Self::initialize_decision_knowledge(),
        }
    }

    /// Evaluate decision options using weighted criteria
    pub async fn evaluate_decision(
        &self,
        decision: &DecisionOption,
        knowledge: &KnowledgeBase,
    ) -> AdvisorResult<HashMap<String, f32>> {
        let mut scores = HashMap::new();

        for criterion in &decision.criteria {
            // Use decision templates to evaluate each criterion
            if let Some(template) = self.knowledge_base.get(&decision.title) {
                let score = self
                    .evaluate_criterion(criterion, template, decision, knowledge)
                    .await?;
                scores.insert(criterion.name.clone(), score);
            } else {
                // Fallback to basic weighting
                scores.insert(criterion.name.clone(), criterion.weight);
            }
        }

        Ok(scores)
    }

    /// Evaluate a single decision criterion
    async fn evaluate_criterion(
        &self,
        criterion: &DecisionCriterion,
        _template: &DecisionTemplate,
        decision: &DecisionOption,
        _knowledge: &KnowledgeBase,
    ) -> AdvisorResult<f32> {
        // In a real implementation, this would use more sophisticated evaluation logic
        let context_factor = decision.context.len() as f32 / 1000.0;
        let options_factor = decision.alternatives.len() as f32 / 10.0;

        let base_score = criterion.weight;
        let adjusted_score = base_score * (1.0 + context_factor + options_factor);

        Ok(adjusted_score.clamp(0.0, 1.0))
    }

    /// Initialize decision knowledge base
    fn initialize_decision_knowledge() -> HashMap<String, DecisionTemplate> {
        let mut knowledge = HashMap::new();

        knowledge.insert("Database Selection".to_string(), DecisionTemplate {
            name:             "Database Selection Decision Template".to_string(),
            criteria_weights: HashMap::from([
                ("Performance".to_string(), 0.3),
                ("Scalability".to_string(), 0.25),
                ("Maintainability".to_string(), 0.2),
                ("Cost".to_string(), 0.15),
                ("Security".to_string(), 0.1),
            ]),
            risk_factors:     vec![
                "Vendor lock-in".to_string(),
                "Migration complexity".to_string(),
                "Team expertise".to_string(),
            ],
        });

        knowledge
    }
}

/// Decision template for evaluation guidance
#[derive(Debug)]
struct DecisionTemplate {
    name:             String,
    criteria_weights: HashMap<String, f32>,
    risk_factors:     Vec<String>,
}

/// Knowledge base for architectural decisions
#[derive(Debug, Default)]
pub struct KnowledgeBase {
    patterns:    HashMap<String, HashMap<String, f32>>,
    risks:       Vec<String>,
    constraints: Vec<String>,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            patterns:    Self::initialize_patterns(),
            risks:       Self::initialize_risks(),
            constraints: Self::initialize_constraints(),
        }
    }

    /// Get knowledge about a specific pattern
    pub fn get_pattern_knowledge(&self, pattern_name: &str, context: &str) -> Option<f32> {
        self.patterns
            .get(pattern_name)
            .and_then(|context_scores| context_scores.get(context))
            .copied()
    }

    /// Initialize pattern knowledge
    fn initialize_patterns() -> HashMap<String, HashMap<String, f32>> {
        // Would be populated with architectural pattern knowledge
        HashMap::new()
    }

    /// Initialize risk knowledge
    fn initialize_risks() -> Vec<String> {
        vec![
            "Performance bottlenecks".to_string(),
            "Scalability limitations".to_string(),
            "Security vulnerabilities".to_string(),
            "Maintainability issues".to_string(),
            "Testing challenges".to_string(),
        ]
    }

    /// Initialize constraint knowledge
    fn initialize_constraints() -> Vec<String> {
        vec![
            "Budget limitations".to_string(),
            "Time constraints".to_string(),
            "Team expertise".to_string(),
            "Technology requirements".to_string(),
            "Business requirements".to_string(),
        ]
    }
}

/// Trait for metric calculators
trait MetricCalculator: Send + Sync + std::fmt::Debug {
    fn calculate(&self, context: &ArchitecturalContext) -> AdvisorResult<f32>;
    fn name(&self) -> &str;
}

// Implement metric calculators
#[derive(Debug)]
struct MaintainabilityCalculator;

impl MetricCalculator for MaintainabilityCalculator {
    fn calculate(&self, _context: &ArchitecturalContext) -> AdvisorResult<f32> {
        // Placeholder calculation
        Ok(85.0)
    }

    fn name(&self) -> &str {
        "maintainability"
    }
}

#[derive(Debug)]
struct ComplexityCalculator;

impl MetricCalculator for ComplexityCalculator {
    fn calculate(&self, _context: &ArchitecturalContext) -> AdvisorResult<f32> {
        // Placeholder calculation
        Ok(50.0)
    }

    fn name(&self) -> &str {
        "cyclomatic_complexity"
    }
}

#[derive(Debug)]
struct CouplingCalculator;

impl MetricCalculator for CouplingCalculator {
    fn calculate(&self, _context: &ArchitecturalContext) -> AdvisorResult<f32> {
        // Placeholder calculation
        Ok(25.0)
    }

    fn name(&self) -> &str {
        "coupling_index"
    }
}

/// Utility functions for analysis operations
pub mod analysis_utils {
    use super::*;

    /// Calculate stability metric based on coupling
    pub fn calculate_stability(efferent_coupling: usize, afferent_coupling: usize) -> f32 {
        let total_coupling = efferent_coupling + afferent_coupling;
        if total_coupling == 0 {
            1.0 // Stable with no dependencies
        } else {
            afferent_coupling as f32 / total_coupling as f32
        }
    }

    /// Calculate abstractness based on implementation vs interface ratio
    pub fn calculate_abstractness(abstract_classes: usize, total_classes: usize) -> f32 {
        if total_classes == 0 {
            0.0
        } else {
            abstract_classes as f32 / total_classes as f32
        }
    }

    /// Calculate distance from "main sequence" (ideal coupling/abstractness balance)
    pub fn calculate_distance_from_main(abstractness: f32, instability: f32) -> f32 {
        ((abstractness + instability - 1.0).abs()).sqrt()
    }

    /// Analyze dependency depth for potential circular dependencies
    pub fn analyze_dependency_depth(dependencies: &HashMap<String, Vec<String>>, max_depth: usize) -> Vec<String> {
        let mut circular_deps = Vec::new();

        for (module, deps) in dependencies {
            if deps.contains(module) {
                circular_deps.push(format!("{} depends on itself", module));
            } else {
                // Check for longer cycles
                if self::has_cycle(dependencies, module, module, max_depth, HashMap::new()) {
                    circular_deps.push(format!("{} is part of a circular dependency", module));
                }
            }
        }

        circular_deps
    }

    /// Helper function to detect circular dependencies
    fn has_cycle(
        deps: &HashMap<String, Vec<String>>,
        start: &str,
        current: &str,
        depth: usize,
        visited: HashMap<String, usize>,
    ) -> bool {
        if depth == 0 {
            return false; // Stop recursion at max depth
        }

        let mut current_visited = visited.clone();
        *current_visited.entry(current.to_string()).or_insert(0) += 1;

        // If we've seen this node at this depth level, we have a cycle
        if *current_visited.get(current).unwrap_or(&0) > 1 {
            return start == current;
        }

        if let Some(module_deps) = deps.get(current) {
            for dep in module_deps {
                if has_cycle(deps, start, dep, depth - 1, current_visited.clone()) {
                    return true;
                }
            }
        }

        false
    }

    /// Generate analysis summary
    pub fn generate_analysis_summary(analysis: &PatternAnalysis) -> String {
        let pattern_count = analysis.detected_patterns.len();
        let anti_pattern_count = analysis.anti_patterns.len();
        let overall_complexity = analysis.complexity_assessment.overall_complexity.clone();

        format!(
            "Analysis Summary:\n- Detected Patterns: {}\n- Anti-Patterns: {}\n- Overall Complexity: {:?}\n- Quality \
             Score: {:.2}",
            pattern_count,
            anti_pattern_count,
            overall_complexity,
            analysis.quality_metrics.maintainability_index / 171.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_analyzer_creation() {
        let analyzer = MetricsAnalyzer::new();
        assert!(!analyzer.metric_calculators.is_empty());
    }

    #[test]
    fn test_decision_engine_creation() {
        let engine = DecisionEngine::new();
        assert!(!engine.knowledge_base.is_empty());
    }

    #[test]
    fn test_stability_calculation() {
        let stability = analysis_utils::calculate_stability(5, 10);
        assert_eq!(stability, 10.0 / 15.0);
    }

    #[test]
    fn test_abstractness_calculation() {
        let abstractness = analysis_utils::calculate_abstractness(3, 10);
        assert_eq!(abstractness, 0.3);
    }

    #[test]
    fn test_main_sequence_distance() {
        let distance = analysis_utils::calculate_distance_from_main(0.5, 0.5);
        assert_eq!(distance, 0.0); // On the main sequence

        let distance_off = analysis_utils::calculate_distance_from_main(0.8, 0.2);
        assert!(distance_off > 0.0); // Off the main sequence
    }

    #[tokio::test]
    async fn test_knowledge_base_creation() {
        let kb = KnowledgeBase::new();
        assert!(kb.risks.len() > 0);
        assert!(kb.constraints.len() > 0);
    }
}

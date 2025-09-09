use crate::types::*;
use std::collections::HashMap;

/// Enhanced confidence scoring system for refactoring operations
pub struct ConfidenceScorer {
    /// Base confidence scores by operation type
    base_scores: HashMap<RefactoringType, f64>,
    /// Context-specific scoring rules
    context_rules: Vec<ConfidenceRule>,
    /// Historical performance data (operation -> success rate)
    historical_data: HashMap<RefactoringType, SuccessMetrics>,
}

impl ConfidenceScorer {
    pub fn new() -> Self {
        let mut base_scores = HashMap::new();

        // Initialize base confidence scores based on operation complexity and reliability
        base_scores.insert(RefactoringType::Rename, 0.95);
        base_scores.insert(RefactoringType::AddMissingImports, 0.90);
        base_scores.insert(RefactoringType::ExtractVariable, 0.85);
        base_scores.insert(RefactoringType::ExtractFunction, 0.80);
        base_scores.insert(RefactoringType::InlineVariable, 0.75);
        base_scores.insert(RefactoringType::ReplaceConstructor, 0.70);
        base_scores.insert(RefactoringType::ExtractInterface, 0.65);
        base_scores.insert(RefactoringType::MoveClass, 0.60);
        base_scores.insert(RefactoringType::SplitClass, 0.55);
        base_scores.insert(RefactoringType::ChangeSignature, 0.50);
        base_scores.insert(RefactoringType::ConvertToAsync, 0.45);
        base_scores.insert(RefactoringType::MergeClasses, 0.40);

        // Initialize default confidence rules
        let context_rules = Self::create_default_rules();

        ConfidenceScorer {
            base_scores,
            context_rules,
            historical_data: HashMap::new(),
        }
    }

    /// Calculate comprehensive confidence score for a refactoring operation
    pub async fn calculate_confidence(
        &self,
        refactoring_type: &RefactoringType,
        context: &RefactoringContext,
        preflight_analysis: &Option<RefactoringAnalysis>,
    ) -> ConfidenceResult {
        let mut score = self.get_base_score(refactoring_type);
        let mut factors = Vec::new();

        // Apply context-based adjustments
        for rule in &self.context_rules {
            if rule.applies_to(refactoring_type, context) {
                let adjustment = rule.apply(context);
                score = (score * (1.0 + adjustment.delta)).clamp(0.0, 0.95);
                factors.push(adjustment);
            }
        }

        // Apply pre-flight analysis adjustments
        if let Some(analysis) = preflight_analysis {
            let analysis_adjustments = self.calculate_analysis_adjustments(analysis, context);
            for adjustment in analysis_adjustments {
                score = (score * (1.0 + adjustment.delta)).clamp(0.0, 0.95);
                factors.push(adjustment);
            }
        }

        // Apply historical performance adjustments
        if let Some(metrics) = self.historical_data.get(refactoring_type) {
            let historical_adjustment = self.calculate_historical_adjustment(metrics);
            score = (score * (1.0 * historical_adjustment)).clamp(0.05, 0.95);
            factors.push(ConfidenceAdjustment {
                factor: ConfidenceFactor::HistoricalPerformance,
                delta: historical_adjustment,
                reason: format!("Historical success rate: {:.1}%", metrics.success_rate * 100.0),
            });
        }

        // Apply final safety bounds
        score = self.apply_final_bounds(score, refactoring_type, context);

        ConfidenceResult {
            overall_score: score,
            confidence_level: self.score_to_level(score),
            factors: factors.clone(),
            recommendations: self.generate_recommendations(score, &factors, context),
        }
    }

    /// Get base confidence score for operation type
    fn get_base_score(&self, refactoring_type: &RefactoringType) -> f64 {
        self.base_scores.get(refactoring_type).copied().unwrap_or(0.60)
    }

    /// Calculate adjustments based on pre-flight analysis
    fn calculate_analysis_adjustments(&self, analysis: &RefactoringAnalysis, context: &RefactoringContext) -> Vec<ConfidenceAdjustment> {
        let mut adjustments = Vec::new();

        // Impact level adjustment
        let impact_delta = match analysis.potential_impact {
            RefactoringImpact::Low => 0.05,    // Small confidence boost for low risk
            RefactoringImpact::Medium => 0.0,   // No change for medium risk
            RefactoringImpact::High => -0.15,  // Significant penalty for high risk
            RefactoringImpact::Critical => -0.30, // Severe penalty for critical operations
        };

        if impact_delta != 0.0 {
            adjustments.push(ConfidenceAdjustment {
                factor: ConfidenceFactor::ImpactLevel,
                delta: impact_delta,
                reason: format!("Impact level: {:?}", analysis.potential_impact),
            });
        }

        // Safety adjustment
        let safety_delta = if analysis.is_safe { 0.10 } else { -0.25 };
        adjustments.push(ConfidenceAdjustment {
            factor: ConfidenceFactor::SafetyAnalysis,
            delta: safety_delta,
            reason: format!("Safety assessment: {}", if analysis.is_safe { "SAFE" } else { "UNSAFE" }),
        });

        // Symbol complexity adjustment
        if let Some(symbol_name) = &context.symbol_name {
            let complexity_delta = match symbol_name.len() {
                0..=10 => 0.05,  // Simple names = higher confidence
                11..=30 => 0.0,   // Moderate names
                _ => -0.05,      // Complex names = lower confidence
            };
            adjustments.push(ConfidenceAdjustment {
                factor: ConfidenceFactor::SymbolComplexity,
                delta: complexity_delta,
                reason: format!("Symbol complexity based on name length: {}", symbol_name.len()),
            });
        }

        // Affected files count adjustment
        let file_count_delta = match analysis.affected_files.len() {
            1 => 0.05,   // Single file = higher confidence
            2..=3 => 0.0, // Few files = neutral
            4..=10 => -0.10, // Many files = reduced confidence
            _ => -0.20,  // Too many files = significantly reduced confidence
        };

        if file_count_delta != 0.0 {
            adjustments.push(ConfidenceAdjustment {
                factor: ConfidenceFactor::AffectedFilesCount,
                delta: file_count_delta,
                reason: format!("Affects {} files", analysis.affected_files.len()),
            });
        }

        adjustments
    }

    /// Apply historical performance adjustments
    fn calculate_historical_adjustment(&self, metrics: &SuccessMetrics) -> f64 {
        let success_rate = metrics.success_rate;
        let total_executions = metrics.total_executions;

        // Base adjustment from success rate
        let rate_adjustment = (success_rate - 0.5) * 0.3; // Range: -0.15 to +0.15

        // Confidence boost from experience (more executions = more confidence)
        let experience_bonus = if total_executions <= 5 {
            0.0
        } else if total_executions <= 20 {
            0.02
        } else if total_executions <= 100 {
            0.05
        } else {
            0.08
        };

        rate_adjustment + experience_bonus
    }

    /// Apply final safety bounds to confidence score
    fn apply_final_bounds(&self, score: f64, refactoring_type: &RefactoringType, context: &RefactoringContext) -> f64 {
        let min_score = match refactoring_type {
            // High-confidence operations get higher minimum scores
            RefactoringType::Rename | RefactoringType::AddMissingImports => 0.80,
            RefactoringType::ExtractVariable | RefactoringType::InlineVariable => 0.65,
            // Moderate-confidence operations
            RefactoringType::ExtractFunction | RefactoringType::ExtractInterface => 0.50,
            // Low-confidence operations get lower minimums but not too low
            _ => 0.35,
        };

        // Ensure score doesn't go below absolute minimum for any operation
        let absolute_min = 0.20;

        score.max(min_score).max(absolute_min).min(0.95)
    }

    /// Convert numerical score to confidence level
    fn score_to_level(&self, score: f64) -> ConfidenceLevel {
        match score {
            0.80..=0.95 => ConfidenceLevel::VeryHigh,
            0.65..=0.79 => ConfidenceLevel::High,
            0.50..=0.64 => ConfidenceLevel::Medium,
            0.35..=0.49 => ConfidenceLevel::Low,
            _ => ConfidenceLevel::VeryLow,
        }
    }

    /// Generate recommendations based on confidence analysis
    fn generate_recommendations(&self, score: f64, factors: &[ConfidenceAdjustment], context: &RefactoringContext) -> Vec<String> {
        let mut recommendations = Vec::new();

        if score < 0.50 {
            recommendations.push("Consider performing this refactoring manually to ensure correctness.".to_string());
            recommendations.push("Verify all usages of the target symbol before proceeding.".to_string());
        }

        // Check for negative factors
        let negative_factors: Vec<_> = factors.iter().filter(|f| f.delta < 0.0).collect();
        if negative_factors.len() > 2 {
            recommendations.push("Multiple risk factors detected. Consider refactoring in smaller steps.".to_string());
        }

        // Context-specific recommendations
        if let Some(previous_usages) = self.analyze_symbol_usage_complexity(context) {
            if previous_usages > 10 {
                recommendations.push("Symbol has many usages - consider renaming first for clarity.".to_string());
            }
        }

        if factors.iter().any(|f| matches!(f.factor, ConfidenceFactor::ImpactLevel)) {
            recommendations.push("High impact operation - ensure you have proper backups.".to_string());
        }

        recommendations
    }

    /// Analyze symbol usage complexity (placeholder for more advanced analysis)
    fn analyze_symbol_usage_complexity(&self, _context: &RefactoringContext) -> Option<usize> {
        // This would typically query LSP or perform code analysis to count usages
        Some(5) // Placeholder
    }

    /// Create default confidence rules
    fn create_default_rules() -> Vec<ConfidenceRule> {
        vec![
            // Rule: Simple symbol names get higher confidence
            ConfidenceRule {
                name: "SimpleSymbolNames".to_string(),
                conditions: vec![RuleCondition::SymbolNameLengthRange(1, 15)],
                adjustment: 0.05,
            },
            // Rule: AST-supported languages get higher confidence (experimental operations)
            ConfidenceRule {
                name: "ASTSupported".to_string(),
                conditions: vec![RuleCondition::IsASTLanguage(true)],
                adjustment: 0.10,
            },
            // Rule: Local variables get higher confidence
            ConfidenceRule {
                name: "LocalVariables".to_string(),
                conditions: vec![RuleCondition::SymbolKind(SymbolKind::Variable)],
                adjustment: 0.08,
            },
        ]
    }

    /// Update historical performance data
    pub fn update_performance(&mut self, refactoring_type: RefactoringType, was_successful: bool) {
        let metrics = self.historical_data.entry(refactoring_type).or_insert(SuccessMetrics {
            total_executions: 0,
            successful_executions: 0,
            success_rate: 0.0,
        });

        metrics.total_executions += 1;
        if was_successful {
            metrics.successful_executions += 1;
        }
        metrics.success_rate = metrics.successful_executions as f64 / metrics.total_executions as f64;
    }
}

/// Confidence result with detailed breakdown
#[derive(Debug, Clone)]
pub struct ConfidenceResult {
    pub overall_score: f64,
    pub confidence_level: ConfidenceLevel,
    pub factors: Vec<ConfidenceAdjustment>,
    pub recommendations: Vec<String>,
}

/// Individual confidence adjustment
#[derive(Debug, Clone)]
pub struct ConfidenceAdjustment {
    pub factor: ConfidenceFactor,
    pub delta: f64,
    pub reason: String,
}

/// Factors that influence confidence scoring
#[derive(Debug, Clone, PartialEq)]
pub enum ConfidenceFactor {
    ImpactLevel,
    SafetyAnalysis,
    SymbolComplexity,
    AffectedFilesCount,
    HistoricalPerformance,
    SymbolType,
    CodeComplexity,
    ASTSupport,
}

/// Confidence levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfidenceLevel {
    VeryHigh, // 0.80 - 0.95
    High,     // 0.65 - 0.79
    Medium,   // 0.50 - 0.64
    Low,      // 0.35 - 0.49
    VeryLow,  // 0.05 - 0.34
}

/// Historical success metrics
#[derive(Debug, Clone)]
pub struct SuccessMetrics {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub success_rate: f64,
}

/// Confidence adjustment rule
#[derive(Debug, Clone)]
pub struct ConfidenceRule {
    pub name: String,
    pub conditions: Vec<RuleCondition>,
    pub adjustment: f64,
}

impl ConfidenceRule {
    /// Check if rule applies to given context
    pub fn applies_to(&self, refactoring_type: &RefactoringType, context: &RefactoringContext) -> bool {
        for condition in &self.conditions {
            if !condition.matches(refactoring_type, context) {
                return false;
            }
        }
        true
    }

    /// Apply the rule adjustment
    pub fn apply(&self, context: &RefactoringContext) -> ConfidenceAdjustment {
        ConfidenceAdjustment {
            factor: ConfidenceFactor::CodeComplexity, // Generic factor for now
            delta: self.adjustment,
            reason: format!("Applied rule '{}': {}", self.name, if self.adjustment > 0.0 { "confidence boost" } else { "caution applied" }),
        }
    }
}

/// Condition for applying confidence rules
#[derive(Debug, Clone)]
pub enum RuleCondition {
    SymbolNameLengthRange(usize, usize),
    SymbolKind(SymbolKind),
    IsASTLanguage(bool),
    HasSelection(bool),
    ImpactLevel(RefactoringImpact),
}

impl RuleCondition {
    pub fn matches(&self, _refactoring_type: &RefactoringType, context: &RefactoringContext) -> bool {
        match self {
            RuleCondition::SymbolNameLengthRange(min, max) => {
                context.symbol_name.as_ref()
                    .map(|name| name.len() >= *min && name.len() <= *max)
                    .unwrap_or(false)
            }
            RuleCondition::SymbolKind(expected_kind) => {
                context.symbol_kind.as_ref() == Some(expected_kind)
            }
            RuleCondition::IsASTLanguage(expected) => {
                // Check if file extension indicates AST support
                context.file_path.ends_with(".rs") == *expected
            }
            RuleCondition::HasSelection(expected) => {
                context.selection.is_some() == *expected
            }
            RuleCondition::ImpactLevel(_level) => {
                // This would need access to pre-flight analysis
                true
            }
        }
    }
}

impl Default for ConfidenceScorer {
    fn default() -> Self {
        Self::new()
    }
}
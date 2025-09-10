use crate::analysis::RefactoringAnalyzer;
use crate::confidence::{ConfidenceScorer, ConfidenceResult};
use crate::safety::RefactoringRisk;
use crate::types::*;
use std::collections::HashMap;
use async_trait::async_trait;

/// Trait for suggestion engines
#[async_trait]
pub trait SuggestionEngine {
    /// Get suggestions based on refactoring context
    async fn get_suggestions(&self, context: &SuggestionContext) -> Result<Vec<RefactoringSuggestion>, String>;
}

/// Context for suggestion generation
#[derive(Debug, Clone)]
pub struct SuggestionContext {
    pub file_path: String,
    pub symbol_name: Option<String>,
    pub symbol_kind: Option<SymbolKind>,
    pub project_context: std::collections::HashMap<String, String>,
}

/// AI-powered suggestion engine
pub struct AISuggestionEngine {
    base_engine: SuggestionEngineWrapper,
}

/// Wrapper for the old SuggestionEngine to work with the trait
struct SuggestionEngineWrapper {
    inner: SuggestionEngine,
}

#[async_trait]
impl SuggestionEngine for SuggestionEngineWrapper {
    async fn get_suggestions(&self, context: &SuggestionContext) -> Result<Vec<RefactoringSuggestion>, String> {
        // Convert to refactoring context
        let refactoring_context = RefactoringContext {
            file_path: context.file_path.clone(),
            cursor_line: 0, // Default values
            cursor_character: 0,
            selection: None,
            symbol_name: context.symbol_name.clone(),
            symbol_kind: context.symbol_kind.clone(),
        };

        let suggestions = self.inner.generate_suggestions(&refactoring_context, None).await?;
        let suggestions = suggestions.into_iter()
            .map(|s| RefactoringSuggestion {
                operation_type: s.refactoring_type.to_string(),
                confidence_score: s.confidence.overall_score,
                description: s.reasoning,
            })
            .collect();

        Ok(suggestions)
    }
}

/// Implementation for AISuggestionEngine
impl AISuggestionEngine {
    pub fn new() -> Self {
        AISuggestionEngine {
            base_engine: SuggestionEngineWrapper {
                inner: SuggestionEngine::new(),
            },
        }
    }
}

#[async_trait]
impl SuggestionEngine for AISuggestionEngine {
    async fn get_suggestions(&self, context: &SuggestionContext) -> Result<Vec<RefactoringSuggestion>, String> {
        self.base_engine.get_suggestions(context).await
    }
}

/// Context-aware suggestion engine for intelligent refactoring recommendations
pub struct SuggestionEngine {
    analyzer: RefactoringAnalyzer,
    confidence_scorer: ConfidenceScorer,
    suggestion_cache: HashMap<String, Vec<SmartSuggestion>>,
    context_patterns: Vec<ContextPattern>,
}

impl SuggestionEngine {
    pub fn new() -> Self {
        SuggestionEngine {
            analyzer: RefactoringAnalyzer::new(),
            confidence_scorer: ConfidenceScorer::new(),
            suggestion_cache: HashMap::new(),
            context_patterns: Self::create_default_patterns(),
        }
    }

    /// Generate intelligent refactoring suggestions based on context analysis
    pub async fn generate_suggestions(
        &mut self,
        context: &RefactoringContext,
        code_content: Option<&str>,
    ) -> Result<Vec<SmartSuggestion>, Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = self.generate_cache_key(context);

        // Check cache first
        if let Some(cached) = self.suggestion_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let mut suggestions = Vec::new();

        // Analyze context for applicable refactorings
        let applicable_refactorings = self.analyzer.get_applicable_refactorings_parallel(context).await?;

        // Generate smart suggestions with context awareness
        for refactoring_type in applicable_refactorings {
            if let Some(suggestion) = self.create_smart_suggestion(&refactoring_type, context, code_content).await {
                suggestions.push(suggestion);
            }
        }

        // Sort by relevance and confidence
        suggestions.sort_by(|a, b| {
            b.confidence.overall_score.partial_cmp(&a.confidence.overall_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Cache the results
        self.suggestion_cache.insert(cache_key, suggestions.clone());

        Ok(suggestions)
    }

    /// Create a smart suggestion for a specific refactoring type
    async fn create_smart_suggestion(
        &mut self,
        refactoring_type: &RefactoringType,
        context: &RefactoringContext,
        code_content: Option<&str>,
    ) -> Option<SmartSuggestion> {
        let analysis = self.analyzer.analyze_refactoring_cached(refactoring_type, context).await.ok()?;
        let confidence = self.confidence_scorer.calculate_confidence(
            refactoring_type,
            context,
            &Some(analysis.clone()),
        ).await;

        let context_relevance = self.calculate_context_relevance(refactoring_type, context, &analysis);
        let impact_assessment = self.assess_impact(refactoring_type, &analysis);

        // Only suggest if confidence is above threshold
        if confidence.overall_score < 0.4 {
            return None;
        }

        let priority = self.calculate_priority(&confidence, &context_relevance, &impact_assessment);

        Some(SmartSuggestion {
            refactoring_type: refactoring_type.clone(),
            confidence,
            context_relevance,
            impact_assessment,
            priority,
            suggested_context: context.clone(),
            reasoning: self.generate_reasoning(refactoring_type, context, &analysis),
            alternatives: self.suggest_alternatives(refactoring_type, context),
            prerequisites: self.identify_prerequisites(refactoring_type, context, &analysis),
        })
    }

    /// Calculate context relevance score
    fn calculate_context_relevance(
        &self,
        refactoring_type: &RefactoringType,
        context: &RefactoringContext,
        analysis: &RefactoringAnalysis,
    ) -> ContextRelevance {
        let mut score = 0.0;
        let mut factors = Vec::new();

        // Pattern matching against known context patterns
        for pattern in &self.context_patterns {
            if pattern.matches(context) {
                if pattern.target_refactorings.contains(refactoring_type) {
                    score += pattern.relevance_boost;
                    factors.push(format!("Context pattern match: {}", pattern.name));
                }
            }
        }

        // Symbol-based relevance
        if let Some(symbol_name) = &context.symbol_name {
            match refactoring_type {
                RefactoringType::Rename => {
                    // Rename is highly relevant for any symbol
                    if symbol_name.len() > 50 || symbol_name.contains('_') {
                        score += 0.3;
                        factors.push("Symbol naming issues detected".to_string());
                    }
                }
                RefactoringType::ExtractFunction => {
                    // Extract function for long methods/variables
                    if context.symbol_kind == Some(SymbolKind::Function) {
                        score += 0.4;
                        factors.push("Function extraction opportunity".to_string());
                    }
                }
                _ => {}
            }
        }

        // Selection-based relevance
        if context.selection.is_some() {
            match refactoring_type {
                RefactoringType::ExtractFunction | RefactoringType::ExtractVariable => {
                    score += 0.5;
                    factors.push("Code selection available for extraction".to_string());
                }
                _ => {}
            }
        }

        ContextRelevance {
            score: score.min(1.0).max(0.0),
            factors,
        }
    }

    /// Assess impact of the refactoring
    fn assess_impact(
        &self,
        _refactoring_type: &RefactoringType,
        analysis: &RefactoringAnalysis,
    ) -> ImpactAssessment {
        let affected_files = analysis.affected_files.len();
        let breaking_changes = analysis.breaking_changes.len();

        let risk_level = match (affected_files, breaking_changes) {
            (1, 0) => RefactoringRisk::Low,
            (1..=5, 0..=2) => RefactoringRisk::Medium,
            (_, 3..) | (10.., _) => RefactoringRisk::High,
            (_, _) => RefactoringRisk::Critical,
        };

        ImpactAssessment {
            risk_level,
            affected_files_count: affected_files,
            breaking_changes_count: breaking_changes,
            estimated_effort: self.estimate_effort(affected_files, breaking_changes),
        }
    }

    /// Calculate suggestion priority
    fn calculate_priority(
        &self,
        confidence: &ConfidenceResult,
        _relevance: &ContextRelevance,
        impact: &ImpactAssessment,
    ) -> SuggestionPriority {
        // High confidence, low risk = high priority
        if confidence.overall_score >= 0.8 && matches!(impact.risk_level, RefactoringRisk::Low) {
            SuggestionPriority::High
        }
        // Medium confidence, medium risk = medium priority
        else if confidence.overall_score >= 0.6 && matches!(impact.risk_level, RefactoringRisk::Medium) {
            SuggestionPriority::Medium
        }
        // High confidence, high risk = consider carefully
        else if confidence.overall_score >= 0.7 && matches!(impact.risk_level, RefactoringRisk::High) {
            SuggestionPriority::Medium
        }
        // Low confidence or critical risk = low priority
        else {
            SuggestionPriority::Low
        }
    }

    /// Generate reasoning for suggestion
    fn generate_reasoning(
        &self,
        refactoring_type: &RefactoringType,
        context: &RefactoringContext,
        analysis: &RefactoringAnalysis,
    ) -> String {
        let mut reasons = Vec::new();

        reasons.push(format!("This refactoring improves code maintainability"));

        if analysis.confidence_score > 0.8 {
            reasons.push(format!("High confidence ({:.1}%) based on analysis", analysis.confidence_score * 100.0));
        }

        if let Some(symbol_name) = &context.symbol_name {
            reasons.push(format!("Applied to symbol '{}'", symbol_name));
        }

        if context.selection.is_some() {
            reasons.push(format!("Based on selected code range"));
        }

        reasons.join(", ")
    }

    /// Suggest alternative refactorings
    fn suggest_alternatives(
        &self,
        refactoring_type: &RefactoringType,
        _context: &RefactoringContext,
    ) -> Vec<RefactoringType> {
        match refactoring_type {
            RefactoringType::Rename => vec![
                RefactoringType::ExtractVariable,
                RefactoringType::ExtractFunction,
            ],
            RefactoringType::ExtractFunction => vec![
                RefactoringType::Rename,
                RefactoringType::ExtractVariable,
                RefactoringType::ConvertToAsync,
            ],
            RefactoringType::ExtractVariable => vec![
                RefactoringType::Rename,
                RefactoringType::InlineVariable,
            ],
            _ => Vec::new(),
        }
    }

    /// Identify prerequisites for the refactoring
    fn identify_prerequisites(
        &self,
        refactoring_type: &RefactoringType,
        context: &RefactoringContext,
        _analysis: &RefactoringAnalysis,
    ) -> Vec<String> {
        let mut prerequisites = Vec::new();

        prerequisites.push("File must be writable".to_string());

        match refactoring_type {
            RefactoringType::Rename => {
                prerequisites.push("Symbol must be defined in current file".to_string());
                if context.symbol_name.is_none() {
                    prerequisites.push("Symbol name must be provided".to_string());
                }
            }
            RefactoringType::ExtractFunction => {
                if context.selection.is_none() {
                    prerequisites.push("Code must be selected for extraction".to_string());
                }
                prerequisites.push("Selected code must be syntactically valid".to_string());
            }
            RefactoringType::ExtractVariable => {
                if context.selection.is_none() {
                    prerequisites.push("Expression must be selected".to_string());
                }
                prerequisites.push("Selected expression must be side-effect free".to_string());
            }
            _ => {}
        }

        prerequisites
    }

    /// Estimate effort required for refactoring
    fn estimate_effort(&self, affected_files: usize, breaking_changes: usize) -> EffortEstimate {
        match (affected_files, breaking_changes) {
            (1, 0..=1) => EffortEstimate::Low,
            (1..=3, 0..=3) => EffortEstimate::Medium,
            (4..=10, _) | (_, 4..=10) => EffortEstimate::High,
            (_, _) => EffortEstimate::VeryHigh,
        }
    }

    /// Generate cache key for suggestions
    fn generate_cache_key(&self, context: &RefactoringContext) -> String {
        format!(
            "{}_{}_{}_{}",
            context.file_path,
            context.cursor_line,
            context.cursor_character,
            context.symbol_name.as_deref().unwrap_or("none")
        )
    }

    /// Create default context patterns
    fn create_default_patterns() -> Vec<ContextPattern> {
        vec![
            ContextPattern {
                name: "LongFunction".to_string(),
                conditions: vec![
                    PatternCondition::SymbolKind(SymbolKind::Function),
                    PatternCondition::SelectionLongerThan(30),
                ],
                target_refactorings: vec![
                    RefactoringType::ExtractFunction,
                    RefactoringType::ExtractVariable,
                ],
                relevance_boost: 0.6,
            },
            ContextPattern {
                name: "MagicNumber".to_string(),
                conditions: vec![
                    PatternCondition::SelectionShorterThan(10),
                    PatternCondition::SelectionNumeric,
                ],
                target_refactorings: vec![
                    RefactoringType::ExtractVariable,
                ],
                relevance_boost: 0.4,
            },
            ContextPattern {
                name: "ComplexExpression".to_string(),
                conditions: vec![
                    PatternCondition::SelectionLongerThan(20),
                    PatternCondition::SelectionComplex,
                ],
                target_refactorings: vec![
                    RefactoringType::ExtractVariable,
                    RefactoringType::ExtractFunction,
                ],
                relevance_boost: 0.5,
            },
        ]
    }

    /// Clear suggestion cache
    pub fn clear_cache(&mut self) {
        self.suggestion_cache.clear();
    }
}

/// Smart refactoring suggestion with context and reasoning
#[derive(Debug, Clone)]
pub struct SmartSuggestion {
    pub refactoring_type: RefactoringType,
    pub confidence: ConfidenceResult,
    pub context_relevance: ContextRelevance,
    pub impact_assessment: ImpactAssessment,
    pub priority: SuggestionPriority,
    pub suggested_context: RefactoringContext,
    pub reasoning: String,
    pub alternatives: Vec<RefactoringType>,
    pub prerequisites: Vec<String>,
}

/// Context relevance assessment
#[derive(Debug, Clone)]
pub struct ContextRelevance {
    pub score: f64,
    pub factors: Vec<String>,
}

/// Impact assessment
#[derive(Debug, Clone)]
pub struct ImpactAssessment {
    pub risk_level: RefactoringRisk,
    pub affected_files_count: usize,
    pub breaking_changes_count: usize,
    pub estimated_effort: EffortEstimate,
}

/// Suggestion priority
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SuggestionPriority {
    Low = 1,
    Medium = 2,
    High = 3,
}

/// Effort estimate
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffortEstimate {
    Low = 1,
    Medium = 2,
    High = 3,
    VeryHigh = 4,
}

/// Context pattern for intelligent suggestions
#[derive(Debug, Clone)]
pub struct ContextPattern {
    pub name: String,
    pub conditions: Vec<PatternCondition>,
    pub target_refactorings: Vec<RefactoringType>,
    pub relevance_boost: f64,
}

impl ContextPattern {
    pub fn matches(&self, context: &RefactoringContext) -> bool {
        for condition in &self.conditions {
            if !condition.matches(context) {
                return false;
            }
        }
        true
    }
}

/// Pattern condition
#[derive(Debug, Clone)]
pub enum PatternCondition {
    SymbolKind(SymbolKind),
    SelectionLongerThan(usize),
    SelectionShorterThan(usize),
    SelectionNumeric,
    SelectionComplex,
    SymbolNameContains(String),
    FileExtension(String),
}

impl PatternCondition {
    pub fn matches(&self, context: &RefactoringContext) -> bool {
        match self {
            PatternCondition::SymbolKind(expected) => {
                context.symbol_kind == Some(expected.clone())
            }
            PatternCondition::SelectionLongerThan(min_len) => {
                context.selection.as_ref()
                    .map(|sel| sel.end_character - sel.start_character > *min_len as usize)
                    .unwrap_or(false)
            }
            PatternCondition::SelectionShorterThan(max_len) => {
                context.selection.as_ref()
                    .map(|sel| sel.end_character - sel.start_character < *max_len as usize)
                    .unwrap_or(false)
            }
            PatternCondition::SelectionNumeric => {
                // This would need code analysis - simplified for now
                true
            }
            PatternCondition::SelectionComplex => {
                // This would need code analysis - simplified for now
                context.selection.is_some()
            }
            PatternCondition::SymbolNameContains(substring) => {
                context.symbol_name.as_ref()
                    .map(|name| name.contains(substring))
                    .unwrap_or(false)
            }
            PatternCondition::FileExtension(expected) => {
                context.file_path.ends_with(expected)
            }
        }
    }
}

impl Default for SuggestionEngine {
    fn default() -> Self {
        Self::new()
    }
}
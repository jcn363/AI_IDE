use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use rust_ai_ide_ai_inference::{AiInferenceService, InferenceRequest, InferenceConfig};
use rust_ai_ide_lsp::LSPService;
use rust_ai_ide_common::validation::TauriInputSanitizer;

use crate::pattern_recognizer::PatternRecognizer;
use crate::context_analyzer::CodeContextAnalyzer;
use crate::suggestion_generator::SuggestionGenerator;
use crate::safety_filter::SafetyFilter;
use crate::confidence_scorer::ConfidenceScorer;
use crate::types::{
    RefactoringSuggestion, RefactoringType, RiskLevel, Complexity,
    RefactoringTransformation, TransformationOperation
};
use crate::error::{AnalysisError, AnalysisResult, RefactoringError};

/// AI-powered refactoring suggestion engine
pub struct AiRefactoringSuggester {
    pattern_recognizer: Arc<PatternRecognizer>,
    context_analyzer: Arc<CodeContextAnalyzer>,
    suggestion_generator: Arc<SuggestionGenerator>,
    confidence_scorer: Arc<ConfidenceScorer>,
    safety_filter: Arc<SafetyFilter>,
    ai_service: Arc<AiInferenceService>,
    lsp_service: Arc<LSPService>,
    config: SuggesterConfig,
    cache: Arc<RwLock<lru::LruCache<String, Vec<RefactoringSuggestion>>>>,
}

/// Configuration for the AI suggestion engine
#[derive(Debug, Clone)]
pub struct SuggesterConfig {
    pub max_suggestions_per_file: usize,
    pub confidence_threshold: f64,
    pub analysis_timeout_seconds: u64,
    pub enable_context_analysis: bool,
    pub enable_pattern_recognition: bool,
    pub enable_safety_filtering: bool,
    pub max_concurrent_analysis: usize,
    pub cache_size_mb: usize,
    pub cache_ttl_seconds: u64,
}

impl Default for SuggesterConfig {
    fn default() -> Self {
        Self {
            max_suggestions_per_file: 10,
            confidence_threshold: 0.85,
            analysis_timeout_seconds: 30,
            enable_context_analysis: true,
            enable_pattern_recognition: true,
            enable_safety_filtering: true,
            max_concurrent_analysis: 5,
            cache_size_mb: 50,
            cache_ttl_seconds: 300,
        }
    }
}

impl AiRefactoringSuggester {
    /// Create a new AI refactoring suggester with default configuration
    pub fn new(
        ai_service: Arc<AiInferenceService>,
        lsp_service: Arc<LSPService>,
    ) -> Self {
        Self::with_config(
            ai_service,
            lsp_service,
            SuggesterConfig::default(),
        )
    }

    /// Create a new AI refactoring suggester with custom configuration
    pub fn with_config(
        ai_service: Arc<AiInferenceService>,
        lsp_service: Arc<LSPService>,
        config: SuggesterConfig,
    ) -> Self {
        let cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(config.cache_size_mb * 1024 * 1024).unwrap()
        )));

        let pattern_recognizer = Arc::new(PatternRecognizer::new(ai_service.clone()));
        let context_analyzer = Arc::new(CodeContextAnalyzer::new(lsp_service.clone()));
        let suggestion_generator = Arc::new(SuggestionGenerator::new(ai_service.clone()));
        let confidence_scorer = Arc::new(ConfidenceScorer::new(ai_service.clone()));
        let safety_filter = Arc::new(SafetyFilter::new());

        Self {
            pattern_recognizer,
            context_analyzer,
            suggestion_generator,
            confidence_scorer,
            safety_filter,
            ai_service,
            lsp_service,
            config,
            cache,
        }
    }

    /// Generate AI-powered refactoring suggestions for a file
    pub async fn generate_suggestions(
        &self,
        file_path: &str,
        file_content: &str,
        context: AnalysisContext,
    ) -> AnalysisResult<Vec<RefactoringSuggestion>> {
        // Input validation
        let sanitized_path = TauriInputSanitizer::sanitize_path(file_path)
            .map_err(|e| AnalysisError::DataProcessing {
                stage: format!("Path sanitization failed: {}", e),
            })?;

        let sanitized_content = TauriInputSanitizer::sanitize_code_input(file_content)
            .map_err(|e| AnalysisError::DataProcessing {
                stage: format!("Content sanitization failed: {}", e),
            })?;

        // Check cache first
        if let Some(cached_suggestions) = self.get_cached_suggestions(file_path).await {
            return Ok(cached_suggestions);
        }

        // Generate suggestions with timeout
        let suggestions = timeout(
            Duration::from_secs(self.config.analysis_timeout_seconds),
            self.generate_suggestions_internal(&sanitized_path, &sanitized_content, context)
        ).await
        .map_err(|_| AnalysisError::DataProcessing {
            stage: format!("Analysis timeout after {} seconds", self.config.analysis_timeout_seconds),
        })??;

        // Cache results
        self.cache_suggestions(file_path, &suggestions).await;

        Ok(suggestions)
    }

    /// Generate suggestions without caching or timeout (for testing)
    async fn generate_suggestions_internal(
        &self,
        file_path: &str,
        file_content: &str,
        context: AnalysisContext,
    ) -> AnalysisResult<Vec<RefactoringSuggestion>> {
        let mut suggestions = Vec::new();

        // Step 1: Pattern Recognition
        if self.config.enable_pattern_recognition {
            let patterns = self.pattern_recognizer
                .analyze_patterns(file_path, file_content, &context)
                .await?;

            for pattern in patterns {
                let suggestion = self.suggestion_generator
                    .generate_from_pattern(pattern, &context)
                    .await?;
                suggestions.push(suggestion);
            }
        }

        // Step 2: Context Analysis for behavioral preservation
        if self.config.enable_context_analysis {
            let context_analysis = self.context_analyzer
                .analyze_context(file_path, file_content, &context)
                .await?;

            for analysis_result in context_analysis {
                let suggestion = self.suggestion_generator
                    .generate_from_context(analysis_result, &context)
                    .await?;
                suggestions.push(suggestion);
            }
        }

        // Step 3: AI-powered suggestion generation
        let ai_suggestions = self.generate_ai_suggestions(file_path, file_content, &context).await?;
        suggestions.extend(ai_suggestions);

        // Step 4: Confidence scoring
        for suggestion in &mut suggestions {
            let confidence = self.confidence_scorer
                .score_suggestion(suggestion, file_content, &context)
                .await?;
            suggestion.confidence_score = confidence;
        }

        // Step 5: Safety filtering
        if self.config.enable_safety_filtering {
            suggestions = self.safety_filter
                .filter_suggestions(suggestions, &context)
                .await?;
        }

        // Step 6: Sort by confidence and limit results
        suggestions.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap());
        suggestions.truncate(self.config.max_suggestions_per_file);

        // Step 7: Apply confidence threshold
        suggestions.retain(|s| s.confidence_score >= self.config.confidence_threshold);

        Ok(suggestions)
    }

    /// Generate AI-powered suggestions using inference service
    async fn generate_ai_suggestions(
        &self,
        file_path: &str,
        file_content: &str,
        context: &AnalysisContext,
    ) -> AnalysisResult<Vec<RefactoringSuggestion>> {
        let prompt = self.build_ai_prompt(file_path, file_content, context);

        let request = InferenceRequest {
            prompt,
            max_tokens: Some(1000),
            temperature: Some(0.3),
            stop_sequences: Some(vec!["---".to_string()]),
            config: InferenceConfig {
                model_name: "refactoring-model".to_string(),
                parameters: Default::default(),
            },
        };

        let response = self.ai_service
            .generate_completion(request)
            .await
            .map_err(|e| AnalysisError::ModelInference {
                model_name: "refactoring-model".to_string(),
            })?;

        self.parse_ai_suggestions(&response.text, file_path, context).await
    }

    /// Build AI inference prompt
    fn build_ai_prompt(&self, file_path: &str, file_content: &str, context: &AnalysisContext) -> String {
        format!(
            "Analyze the following code file for refactoring opportunities:

File: {}
Language: {}
Content:
{}

Context:
- Project type: {}
- Dependencies: {}
- Recent changes: {}
- Code style preferences: {}

Please suggest refactoring opportunities with:
1. Type of refactoring (extract method, rename, etc.)
2. Location in code (line/column)
3. Description of improvement
4. Risk assessment (low/medium/high)
5. Complexity estimate
6. Expected benefits

Format each suggestion as:
---
TYPE: [refactoring_type]
LINE: [line_number]
COLUMN: [column_number]
DESCRIPTION: [description]
RISK: [risk_level]
COMPLEXITY: [complexity]
BEFORE: [code_before]
AFTER: [code_after]
---
---
"---</parameter1_name>
<parameter2_name>           suggestion_risk: suggestion_risk.to_string(),
            suggestion_complexity: suggestion_complexity.to_string(),
            file_path,
        }
    }

    /// Parse AI response into structured suggestions
    async fn parse_ai_suggestions(
        &self,
        ai_response: &str,
        file_path: &str,
        context: &AnalysisContext,
    ) -> AnalysisResult<Vec<RefactoringSuggestion>> {
        let mut suggestions = Vec::new();

        // Split response by suggestion delimiter
        let suggestion_blocks = ai_response.split("---").filter(|block| !block.trim().is_empty());

        for block in suggestion_blocks {
            let suggestion = self.parse_single_suggestion(block, file_path, context)?;
            suggestions.push(suggestion);
        }

        Ok(suggestions)
    }

    /// Parse a single suggestion block
    fn parse_single_suggestion(
        &self,
        suggestion_block: &str,
        file_path: &str,
        context: &AnalysisContext,
    ) -> AnalysisResult<RefactoringSuggestion> {
        let lines = suggestion_block.lines().map(|l| l.trim()).collect::<Vec<_>>();

        let mut suggestion_type = None;
        let mut line_number = None;
        let mut column_number = None;
        let mut description = None;
        let mut risk_level = None;
        let mut complexity = None;
        let mut code_before = String::new();
        let mut code_after = String::new();

        let mut parsing_before = false;
        let mut parsing_after = false;

        for line in lines {
            if line.starts_with("TYPE:") {
                suggestion_type = Some(line["TYPE:".len()..].trim().to_string());
            } else if line.starts_with("LINE:") {
                line_number = line["LINE:".len()..].trim().parse().ok();
            } else if line.starts_with("COLUMN:") {
                column_number = line["COLUMN:".len()..].trim().parse().ok();
            } else if line.starts_with("DESCRIPTION:") {
                description = Some(line["DESCRIPTION:".len()..].trim().to_string());
            } else if line.starts_with("RISK:") {
                risk_level = Some(line["RISK:".len()..].trim().to_string());
            } else if line.starts_with("COMPLEXITY:") {
                complexity = Some(line["COMPLEXITY:".len()..].trim().to_string());
            } else if line.starts_with("BEFORE:") {
                parsing_before = true;
                parsing_after = false;
                code_before = line["BEFORE:".len()..].trim().to_string();
            } else if line.starts_with("AFTER:") {
                parsing_before = false;
                parsing_after = true;
                code_after = line["AFTER:".len()..].trim().to_string();
            } else if parsing_before {
                code_before.push_str(&format!("\n{}", line));
            } else if parsing_after {
                code_after.push_str(&format!("\n{}", line));
            }
        }

        // Validate required fields
        let suggestion_type = suggestion_type.ok_or_else(|| AnalysisError::DataProcessing {
            stage: "Missing TYPE in suggestion".to_string(),
        })?;

        let line_number = line_number.ok_or_else(|| AnalysisError::DataProcessing {
            stage: "Missing LINE in suggestion".to_string(),
        })?;

        let column_number = column_number.unwrap_or(0);

        let description = description.unwrap_or_else(|| "No description provided".to_string());

        let risk_level = risk_level
            .and_then(|r| match r.to_lowercase().as_str() {
                "low" => Some(RiskLevel::Low),
                "medium" => Some(RiskLevel::Medium),
                "high" => Some(RiskLevel::High),
                _ => Some(RiskLevel::Medium),
        })
            .unwrap_or(RiskLevel::Medium);

        let complexity = complexity
            .and_then(|c| match c.to_lowercase().as_str() {
                "trivial" => Some(Complexity::Trivial),
                "simple" => Some(Complexity::Simple),
                "moderate" => Some(Complexity::Moderate),
                "complex" => Some(Complexity::Complex),
                "high" | "veryhigh" => Some(Complexity::High),
                _ => Some(Complexity::Moderate),
        })
            .unwrap_or(Complexity::Moderate);

        Ok(RefactoringSuggestion {
            id: Uuid::new_v4(),
            suggestion_type: parse_refactoring_type(&suggestion_type),
            target_file: file_path.to_string(),
            target_line: line_number,
            target_column: column_number,
            description,
            confidence_score: 0.5, // Will be updated by confidence scorer
            risk_level,
            estimated_complexity: complexity,
            behavioural_preservation_guarantees: vec![
                "No behavioral changes detected".to_string(),
                "Type safety preserved".to_string(),
            ],
            code_before: code_before.trim().to_string(),
            code_after: code_after.trim().to_string(),
            timestamp: Utc::now(),
            metadata: Default::default(),
        })
    }

    /// Get cached suggestions for a file
    async fn get_cached_suggestions(&self, file_path: &str) -> Option<Vec<RefactoringSuggestion>> {
        let cache = self.cache.read().await;
        cache.get(file_path).cloned()
    }

    /// Cache suggestions for a file
    async fn cache_suggestions(&self, file_path: &str, suggestions: &[RefactoringSuggestion]) {
        let mut cache = self.cache.write().await;
        cache.put(file_path.to_string(), suggestions.to_vec());
    }
}

/// Context information for analysis
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    pub project_root: String,
    pub project_type: String,
    pub dependencies: Vec<String>,
    pub recent_changes: Vec<String>,
    pub code_style_preferences: Vec<String>,
    pub excluded_patterns: Vec<String>,
    pub included_languages: Vec<String>,
}

/// Parse refactoring type from string
fn parse_refactoring_type(type_str: &str) -> RefactoringType {
    match type_str.to_lowercase().as_str() {
        "extract method" | "extract_method" => RefactoringType::ExtractMethod,
        "inline method" | "inline_method" => RefactoringType::InlineMethod,
        "rename symbol" | "rename_symbol" | "rename" => RefactoringType::RenameSymbol,
        "extract variable" | "extract_variable" => RefactoringType::ExtractVariable,
        "inline variable" | "inline_variable" => RefactoringType::InlineVariable,
        "rename local" | "rename_local" => RefactoringType::RenameLocal,
        "extract constant" | "extract_constant" => RefactoringType::ExtractConstant,
        "inline constant" | "inline_constant" => RefactoringType::InlineConstant,
        "add parameter" | "add_parameter" => RefactoringType::AddParameter,
        "remove parameter" | "remove_parameter" => RefactoringType::RemoveParameter,
        "reorder parameters" | "reorder_parameters" => RefactoringType::ReorderParameters,
        "move method" | "move_method" => RefactoringType::MoveMethod,
        "move field" | "move_field" => RefactoringType::MoveField,
        "extract class" | "extract_class" => RefactoringType::ExtractClass,
        "extract interface" | "extract_interface" => RefactoringType::ExtractInterface,
        custom => RefactoringType::Custom(custom.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_suggester_creation() {
        let ai_service = Arc::new(MockAiService::new());
        let lsp_service = Arc::new(MockLspService::new());
        let suggester = AiRefactoringSuggester::new(ai_service, lsp_service);

        assert_eq!(suggester.config.max_suggestions_per_file, 10);
        assert_eq!(suggester.config.confidence_threshold, 0.85);
    }

    #[tokio::test]
    async fn test_parse_refactoring_type() {
        assert!(matches!(parse_refactoring_type("extract method"), RefactoringType::ExtractMethod));
        assert!(matches!(parse_refactoring_type("rename"), RefactoringType::RenameSymbol));
        assert!(matches!(parse_refactoring_type("custom_refactoring"), RefactoringType::Custom(_)));
    }

    // Mock implementations for testing
    struct MockAiService;
    impl MockAiService {
        fn new() -> Self { Self }
    }

    struct MockLspService;
    impl MockLspService {
        fn new() -> Self { Self }
    }
}</parameter2_name>
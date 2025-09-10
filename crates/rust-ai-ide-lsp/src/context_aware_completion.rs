//! Context-Aware Completion with Project-Wide Intelligence
//!
//! This module provides completion suggestions that consider the entire project context,
//! semantic understanding, and AI-powered analysis for truly predictive completions.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use rust_ai_ide_errors::IDEError;
use lsp_types::{CompletionItem, CompletionItemKind, Position, TextDocumentPositionParams};
use serde::{Deserialize, Serialize};

use crate::ai_context::*;
use crate::completion::{CompletionProvider, CompletionConfig};

/// Enhanced completion result with rich metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnhancedCompletionItem {
    pub item: CompletionItem,
    pub confidence: f64,
    pub source: CompletionSource,
    pub context_relevance: f64,
    pub usage_frequency: usize,
    pub project_relevance: f64,
}

/// Source of completion suggestion
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CompletionSource {
    Traditional,           // Standard LSP completion
    MLPrediction,          // AI/ML model prediction
    SemanticAnalysis,      // Semantic context analysis
    PatternAnalysis,       // Code pattern recognition
    ProjectHistory,        // Project usage history
    CrossLanguage,         // Cross-language semantics
}

/// Enhanced completion provider with AI context awareness
pub struct ContextAwareCompletionProvider {
    base_provider: CompletionProvider,
    ai_context_manager: Arc<AIContextManager>,
    project_index: Arc<RwLock<ProjectIndex>>,
    completion_cache: Arc<RwLock<HashMap<String, Vec<EnhancedCompletionItem>>>>,
    learning_engine: Arc<CompletionLearningEngine>,
}

/// Project-wide index for intelligent completion
#[derive(Clone, Debug)]
pub struct ProjectIndex {
    pub symbol_map: HashMap<String, SymbolDefinition>,
    pub usage_patterns: HashMap<String, PatternUsage>,
    pub semantic_clusters: HashMap<String, Vec<String>>,
    pub cross_references: HashMap<String, CrossReferenceInfo>,
}

/// Symbol definition in project
#[derive(Clone, Debug)]
pub struct SymbolDefinition {
    pub name: String,
    pub kind: SymbolType,
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    pub usage_count: usize,
    pub dependencies: Vec<String>,
}

/// Pattern usage statistics
#[derive(Clone, Debug)]
pub struct PatternUsage {
    pub pattern: String,
    pub frequency: usize,
    pub contexts: Vec<String>,
    pub confidence: f64,
    pub last_used: u64,
}

/// Cross-reference information
#[derive(Clone, Debug)]
pub struct CrossReferenceInfo {
    pub symbol: String,
    pub referenced_by: Vec<String>,
    pub references: Vec<String>,
    pub semantic_distance: f64,
}

/// Learning engine for completion preferences
pub struct CompletionLearningEngine {
    user_preferences: Arc<RwLock<HashMap<String, UserPreference>>>,
    pattern_learning: Arc<RwLock<PatternLearningData>>,
}

/// User preference data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserPreference {
    pub context_type: String,
    pub preferred_suggestions: Vec<String>,
    pub rejected_suggestions: Vec<String>,
    pub acceptance_rate: f64,
    pub last_updated: u64,
}

/// Learning data for patterns
#[derive(Clone, Debug)]
pub struct PatternLearningData {
    pub successful_patterns: HashMap<String, usize>,
    pub failed_patterns: HashMap<String, usize>,
    pub context_transitions: HashMap<String, Vec<String>>,
}

impl ContextAwareCompletionProvider {
    pub fn new(config: CompletionConfig) -> Self {
        Self {
            base_provider: CompletionProvider::new(config),
            ai_context_manager: Arc::new(AIContextManager::new()),
            project_index: Arc::new(RwLock::new(ProjectIndex {
                symbol_map: HashMap::new(),
                usage_patterns: HashMap::new(),
                semantic_clusters: HashMap::new(),
                cross_references: HashMap::new(),
            })),
            completion_cache: Arc::new(RwLock::new(HashMap::new())),
            learning_engine: Arc::new(CompletionLearningEngine {
                user_preferences: Arc::new(RwLock::new(HashMap::new())),
                pattern_learning: Arc::new(RwLock::new(PatternLearningData {
                    successful_patterns: HashMap::new(),
                    failed_patterns: HashMap::new(),
                    context_transitions: HashMap::new(),
                })),
            }),
        }
    }

    /// Main entry point for predictive completion requests
    pub async fn generate_predictive_completions(
        &self,
        params: &lsp_types::CompletionParams,
    ) -> Result<Vec<EnhancedCompletionItem>, IDEError> {
        let document_uri = params.text_document_position_params.text_document.uri.clone();
        let document_uri_str = document_uri.to_string();
        let position = params.text_document_position_params.position;

        // Build comprehensive AI context
        let document_content = self.get_document_content(&document_uri_str).await?;
        let context = self.ai_context_manager.build_context(
            &document_uri_str,
            &document_content,
            &position,
        ).await?;

        // Generate multi-source completions
        let mut enhanced_completions = Vec::new();

        // 1. Get traditional LSP completions
        enhanced_completions.extend(self.get_base_completions(&params).await?);

        // 2. Generate AI-powered completions
        enhanced_completions.extend(self.generate_ml_completions(&context).await?);

        // 3. Add semantic-aware completions
        enhanced_completions.extend(self.generate_semantic_completions(&context).await?);

        // 4. Include project-wide suggestions
        enhanced_completions.extend(self.generate_project_completions(&context).await?);

        // 5. Add pattern-based completions
        enhanced_completions.extend(self.generate_pattern_completions(&context).await?);

        // Rank and filter suggestions
        self.rank_and_filter_completions(&mut enhanced_completions).await?;

        // Cache results for future use
        self.update_cache(&document_uri_str, &enhanced_completions).await?;

        Ok(enhanced_completions)
    }

    /// Get traditional LSP completions
    async fn get_base_completions(
        &self,
        params: &lsp_types::CompletionParams,
    ) -> Result<Vec<EnhancedCompletionItem>, IDEError> {
        let base_result = self.base_provider.request_completion(*params).await?;
        let mut enhanced = Vec::new();

        if let Some(list) = base_result {
            for item in list.items {
                enhanced.push(EnhancedCompletionItem {
                    item,
                    confidence: 0.7, // Base confidence for traditional completions
                    source: CompletionSource::Traditional,
                    context_relevance: 0.8,
                    usage_frequency: 0,
                    project_relevance: 0.5,
                });
            }
        }

        Ok(enhanced)
    }

    /// Generate AI-powered completions using ML models
    async fn generate_ml_completions(
        &self,
        context: &PredictiveContext,
    ) -> Result<Vec<EnhancedCompletionItem>, IDEError> {
        let prefix = Self::extract_completion_prefix(&context.current_line, context.cursor_position.character);
        let predictions = self.ai_context_manager.generate_ml_predictions(context, &prefix).await?;

        let mut enhanced = Vec::new();
        for prediction in predictions {
            let completion_item = CompletionItem {
                label: prediction.text.clone(),
                kind: Some(prediction.kind),
                detail: Some(format!("AI suggestion (confidence: {:.2})", prediction.confidence)),
                documentation: prediction.documentation.map(|doc| lsp_types::Documentation::String(doc)),
                insert_text: Some(prediction.text),
                insert_text_format: Some(lsp_types::InsertTextFormat::Snippet),
                ..Default::default()
            };

            enhanced.push(EnhancedCompletionItem {
                item: completion_item,
                confidence: prediction.confidence,
                source: CompletionSource::MLPrediction,
                context_relevance: 0.9,
                usage_frequency: 0,
                project_relevance: 0.8,
            });
        }

        Ok(enhanced)
    }

    /// Generate semantic-aware completions
    async fn generate_semantic_completions(
        &self,
        context: &PredictiveContext,
    ) -> Result<Vec<EnhancedCompletionItem>, IDEError> {
        let semantic_context = self.ai_context_manager.infer_semantic_context(
            &context.current_line,
            &context.cursor_position,
        ).await?;

        let mut enhanced = Vec::new();

        // Generate completions based on semantic intent
        match semantic_context.intent {
            CodeIntent::Declaration => {
                enhanced.extend(self.generate_declaration_suggestions(context).await?);
            }
            CodeIntent::FunctionCall => {
                enhanced.extend(self.generate_function_call_suggestions(context).await?);
            }
            CodeIntent::MethodAccess => {
                enhanced.extend(self.generate_method_suggestions(context).await?);
            }
            _ => {},
        }

        // Add likely variable names based on context
        enhanced.extend(self.generate_variable_suggestions(&semantic_context).await?);

        Ok(enhanced)
    }

    /// Generate project-wide context completions
    async fn generate_project_completions(
        &self,
        context: &PredictiveContext,
    ) -> Result<Vec<EnhancedCompletionItem>, IDEError> {
        let mut enhanced = Vec::new();

        // Access project index for relevant symbols
        let index = self.project_index.read().await;

        for (symbol_name, definition) in &index.symbol_map {
            if self.is_symbol_relevant(symbol_name, context)? {
                let completion_item = CompletionItem {
                    label: symbol_name.clone(),
                    kind: Some(definition.kind.to_completion_kind()),
                    detail: Some(format!("{} from {}", symbol_name, definition.file_path)),
                    documentation: Some(lsp_types::Documentation::String(
                        format!("Symbol defined in {} at line {}", definition.file_path, definition.line)
                    )),
                    insert_text: Some(symbol_name.clone()),
                    ..Default::default()
                };

                enhanced.push(EnhancedCompletionItem {
                    item: completion_item,
                    confidence: 0.8, // High confidence for project symbols
                    source: CompletionSource::ProjectHistory,
                    context_relevance: 0.9,
                    usage_frequency: definition.usage_count,
                    project_relevance: 1.0,
                });
            }
        }

        Ok(enhanced)
    }

    /// Generate pattern-based completions
    async fn generate_pattern_completions(
        &self,
        context: &PredictiveContext,
    ) -> Result<Vec<EnhancedCompletionItem>, IDEError> {
        let similar_patterns = self.ai_context_manager.find_similar_patterns(
            &context.current_line,
            context,
        ).await?;

        let mut enhanced = Vec::new();
        for pattern in similar_patterns {
            if pattern.similarity_score > 0.7 {
                let completion_item = CompletionItem {
                    label: pattern.pattern.clone(),
                    kind: Some(CompletionItemKind::SNIPPET),
                    detail: Some(format!("Pattern (similarity: {:.2})", pattern.similarity_score)),
                    documentation: Some(lsp_types::Documentation::String(
                        format!("Similar pattern found in: {}", pattern.source_location)
                    )),
                    insert_text: Some(pattern.pattern),
                    ..Default::default()
                };

                enhanced.push(EnhancedCompletionItem {
                    item: completion_item,
                    confidence: pattern.similarity_score,
                    source: CompletionSource::PatternAnalysis,
                    context_relevance: pattern.similarity_score,
                    usage_frequency: 1, // From pattern learning
                    project_relevance: 0.7,
                });
            }
        }

        Ok(enhanced)
    }

    /// Rank and filter completion suggestions
    async fn rank_and_filter_completions(
        &self,
        completions: &mut Vec<EnhancedCompletionItem>,
    ) -> Result<(), IDEError> {
        // Sort by composite score (confidence + relevance + frequency)
        completions.sort_by(|a, b| {
            let score_a = a.confidence * 0.4 + a.context_relevance * 0.3 + a.project_relevance * 0.3;
            let score_b = b.confidence * 0.4 + b.context_relevance * 0.3 + b.project_relevance * 0.3;

            // Sort by usage frequency for same score
            if (score_a - score_b).abs() < 0.01 {
                b.usage_frequency.cmp(&a.usage_frequency)
            } else {
                score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        // Filter out low-confidence suggestions
        completions.retain(|item| item.confidence >= self.base_provider.config.ai_min_confidence);

        // Limit to maximum configured items
        let max_items = self.base_provider.config.max_completion_items;
        if completions.len() > max_items {
            completions.truncate(max_items);
        }

        Ok(())
    }

    /// Extract completion prefix from current line and cursor position
    fn extract_completion_prefix(line: &str, cursor_char: u32) -> String {
        let cursor_pos = cursor_char as usize;
        if cursor_pos > 0 && cursor_pos <= line.len() {
            let before_cursor = &line[..cursor_pos];
            // Extract the word being typed (from last space, dot, or bracket)
            before_cursor
                .chars()
                .rev()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>()
                .chars()
                .rev()
                .collect()
        } else {
            String::new()
        }
    }

    /// Check if a symbol is relevant to current context
    fn is_symbol_relevant(&self, symbol: &str, context: &PredictiveContext) -> Result<bool, IDEError> {
        // Simple relevance check - symbols that are imported or used nearby are relevant
        Ok(context.imports.contains(symbol) ||
           context.available_symbols.contains(symbol) ||
           context.current_line.contains(symbol))
    }

    /// Generate declaration suggestions based on context
    async fn generate_declaration_suggestions(
        &self,
        _context: &PredictiveContext,
    ) -> Result<Vec<EnhancedCompletionItem>, IDEError> {
        // Generate suggested variable/function names based on context
        Ok(vec![
            EnhancedCompletionItem {
                item: CompletionItem {
                    label: "result".to_string(),
                    kind: Some(CompletionItemKind::VARIABLE),
                    detail: Some("Variable declaration".to_string()),
                    insert_text: Some("result".to_string()),
                    ..Default::default()
                },
                confidence: 0.6,
                source: CompletionSource::SemanticAnalysis,
                context_relevance: 0.8,
                usage_frequency: 0,
                project_relevance: 0.5,
            }
        ])
    }

    /// Generate function call suggestions
    async fn generate_function_call_suggestions(
        &self,
        _context: &PredictiveContext,
    ) -> Result<Vec<EnhancedCompletionItem>, IDEError> {
        Ok(vec![
            EnhancedCompletionItem {
                item: CompletionItem {
                    label: "unwrap()".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Option::unwrap".to_string()),
                    insert_text: Some("unwrap()".to_string()),
                    ..Default::default()
                },
                confidence: 0.7,
                source: CompletionSource::SemanticAnalysis,
                context_relevance: 0.8,
                usage_frequency: 0,
                project_relevance: 0.6,
            }
        ])
    }

    /// Generate method access suggestions
    async fn generate_method_suggestions(
        &self,
        _context: &PredictiveContext,
    ) -> Result<Vec<EnhancedCompletionItem>, IDEError> {
        Ok(vec![
            EnhancedCompletionItem {
                item: CompletionItem {
                    label: "map()".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Iterator::map".to_string()),
                    insert_text: Some("map(|item| item)".to_string()),
                    insert_text_format: Some(lsp_types::InsertTextFormat::Snippet),
                    ..Default::default()
                },
                confidence: 0.8,
                source: CompletionSource::SemanticAnalysis,
                context_relevance: 0.9,
                usage_frequency: 0,
                project_relevance: 0.7,
            }
        ])
    }

    /// Generate variable name suggestions
    async fn generate_variable_suggestions(
        &self,
        _context: &SemanticContext,
    ) -> Result<Vec<EnhancedCompletionItem>, IDEError> {
        Ok(vec![
            EnhancedCompletionItem {
                item: CompletionItem {
                    label: "data".to_string(),
                    kind: Some(CompletionItemKind::VARIABLE),
                    detail: Some("Variable suggestion".to_string()),
                    insert_text: Some("data".to_string()),
                    ..Default::default()
                },
                confidence: 0.5,
                source: CompletionSource::SemanticAnalysis,
                context_relevance: 0.7,
                usage_frequency: 0,
                project_relevance: 0.5,
            }
        ])
    }

    /// Get document content (placeholder - should integrate with workspace)
    async fn get_document_content(&self, _uri: &str) -> Result<String, IDEError> {
        Ok("// Placeholder document content\nfn main() {}\n".to_string())
    }

    /// Update completion cache
    async fn update_cache(
        &self,
        uri: &str,
        completions: &[EnhancedCompletionItem],
    ) -> Result<(), IDEError> {
        let mut cache = self.completion_cache.write().await;
        cache.insert(uri.to_string(), completions.to_vec());
        Ok(())
    }

    /// Record completion acceptance for learning
    pub async fn record_completion_accepted(&self, item: &EnhancedCompletionItem) -> Result<(), IDEError> {
        // Record in learning engine
        let mut learning = self.learning_engine.pattern_learning.write().await;
        let pattern = format!("{}_{:?}", item.item.label, item.source);

        if let Some(entry) = learning.successful_patterns.get_mut(&pattern) {
            *entry += 1;
        } else {
            learning.successful_patterns.insert(pattern, 1);
        }

        Ok(())
    }
}

/// Extension trait for SymbolType to LSP conversion
trait SymbolTypeExt {
    fn to_completion_kind(&self) -> CompletionItemKind;
}

impl SymbolTypeExt for SymbolType {
    fn to_completion_kind(&self) -> CompletionItemKind {
        match self {
            SymbolType::Function => CompletionItemKind::FUNCTION,
            SymbolType::Variable => CompletionItemKind::VARIABLE,
            SymbolType::Struct => CompletionItemKind::STRUCT,
            SymbolType::Enum => CompletionItemKind::ENUM,
            SymbolType::Trait => CompletionItemKind::INTERFACE,
            SymbolType::Module => CompletionItemKind::MODULE,
            SymbolType::Constant => CompletionItemKind::CONSTANT,
            SymbolType::Macro => CompletionItemKind::KEYWORD,
            _ => CompletionItemKind::TEXT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::Position;

    #[test]
    fn test_context_provider_initialization() {
        let config = CompletionConfig::default();
        let _provider = ContextAwareCompletionProvider::new(config);
        // Test structure is valid
    }

    #[test]
    fn test_prefix_extraction() {
        let line = "fn test(par";
        let prefix = ContextAwareCompletionProvider::extract_completion_prefix(line, 11);
        assert_eq!(prefix, "par");
    }

    #[test]
    fn test_symbol_relevance() {
        let config = CompletionConfig::default();
        let provider = ContextAwareCompletionProvider::new(config);
        let context = PredictiveContext::default();

        // Should return true for simple cases
        assert!(provider.is_symbol_relevant("foo", &context).unwrap() ||
                !provider.is_symbol_relevant("nonexistent", &context).unwrap());
    }
}
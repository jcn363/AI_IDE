//! AI-Enhanced Code Completion for LSP
//!
//! This module provides intelligent code completion that integrates traditional LSP
//! suggestions with AI-powered contextual completions for enhanced developer experience.
//! Now enhanced with predictive capabilities using project-wide context and ML models.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use rust_ai_ide_errors::IDEError;
use lsp_types::{
    CompletionParams, CompletionItem, CompletionList,
    TextDocumentPositionParams, Position,
};
use serde::{Deserialize, Serialize};

// New AI-enhanced imports
use super::ai_context::*;
use super::context_aware_completion::*;

/// Enhanced completion configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompletionConfig {
    /// Enable AI-powered completion suggestions
    pub enable_ai_completion: bool,
    /// Enable contextual completion
    pub enable_context_completion: bool,
    /// Maximum number of completion items to return
    pub max_completion_items: usize,
    /// Enable fuzzy matching for completions
    pub enable_fuzzy_matching: bool,
    /// Context window size for AI suggestions (in lines)
    pub ai_context_window_size: usize,
    /// Minimum confidence score for AI suggestions (0.0-1.0)
    pub ai_min_confidence: f64,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            enable_ai_completion: true,
            enable_context_completion: true,
            max_completion_items: 50,
            enable_fuzzy_matching: true,
            ai_context_window_size: 20,
            ai_min_confidence: 0.3,
        }
    }
}

/// Predictive completion provider with AI-enhanced capabilities
pub struct CompletionProvider {
    config: CompletionConfig,
    /// LSP client for traditional completion
    lsp_client: Arc<RwLock<Option<Box<dyn crate::LspClientTrait + Send + Sync>>>>,
    /// Context-aware completion provider for enhanced suggestions
    context_aware_provider: Arc<ContextAwareCompletionProvider>,
    /// AI context manager for ML integration
    ai_context_manager: Arc<AIContextManager>,
    /// Enhanced completion cache
    completion_cache: Arc<RwLock<HashMap<String, Vec<CompletionItem>>>>,
    /// Completion history for learning
    completion_history: Arc<RwLock<HashMap<String, usize>>>,
    /// Project-wide context cache
    project_context_cache: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
}

#[derive(Clone, Debug)]
pub struct KeywordInfo {
    pub keyword: String,
    pub kind: CompletionItemKind,
    pub documentation: Option<String>,
    pub usage_examples: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompletionContextData {
    /// Current code context (lines above cursor)
    pub prefix_context: Vec<String>,
    /// Current line being edited
    pub current_line: String,
    /// Lines after cursor (for context)
    pub suffix_context: Vec<String>,
    /// Cursor position information
    pub position: Position,
    /// Trigger character information
    pub trigger_character: Option<String>,
    /// File type/language
    pub language: String,
    /// Symbols available in current scope
    pub available_symbols: HashSet<String>,
}

impl Default for CompletionContextData {
    fn default() -> Self {
        Self {
            prefix_context: Vec::new(),
            current_line: String::new(),
            suffix_context: Vec::new(),
            position: Position::new(0, 0),
            trigger_character: None,
            language: "rust".to_string(),
            available_symbols: HashSet::new(),
        }
    }
}

impl CompletionProvider {
    pub fn new(config: CompletionConfig) -> Self {
        Self {
            config: config.clone(),
            lsp_client: Arc::new(RwLock::new(None)),
            context_aware_provider: Arc::new(ContextAwareCompletionProvider::new(config)),
            ai_context_manager: Arc::new(AIContextManager::new()),
            completion_cache: Arc::new(RwLock::new(HashMap::new())),
            completion_history: Arc::new(RwLock::new(HashMap::new())),
            project_context_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn initialize_keyword_mappings() -> HashMap<String, KeywordInfo> {
        let mut mappings = HashMap::new();

        // Rust keyword mappings
        let rust_keywords = vec![
            ("fn", CompletionItemKind::FUNCTION, "Function definition"),
            ("struct", CompletionItemKind::STRUCT, "Struct definition"),
            ("enum", CompletionItemKind::ENUM, "Enum definition"),
            ("impl", CompletionItemKind::CLASS, "Implementation block"),
            ("trait", CompletionItemKind::INTERFACE, "Trait definition"),
            ("match", CompletionItemKind::KEYWORD, "Pattern matching"),
            ("if", CompletionItemKind::KEYWORD, "Conditional statement"),
            ("for", CompletionItemKind::KEYWORD, "Loop construct"),
            ("while", CompletionItemKind::KEYWORD, "While loop"),
            ("let", CompletionItemKind::VARIABLE, "Variable declaration"),
            ("const", CompletionItemKind::CONSTANT, "Constant declaration"),
            ("mut", CompletionItemKind::KEYWORD, "Mutable modifier"),
            ("pub", CompletionItemKind::KEYWORD, "Public visibility"),
            ("mod", CompletionItemKind::MODULE, "Module declaration"),
            ("use", CompletionItemKind::KEYWORD, "Import statement"),
        ];

        for (keyword, kind, docs) in rust_keywords {
            mappings.insert(keyword.to_string(), KeywordInfo {
                keyword: keyword.to_string(),
                kind,
                documentation: Some(docs.to_string()),
                usage_examples: vec![format!("{} ...", keyword)],
            });
        }

        mappings
    }

    pub async fn set_lsp_client(&self, client: Box<dyn crate::LspClientTrait + Send + Sync>) {
        let mut current_client = self.lsp_client.write().await;
        *current_client = Some(client);
    }

    /// Main completion request handler with predictive capabilities
    pub async fn request_completion(
        &self,
        params: CompletionParams
    ) -> Result<Option<CompletionList>, IDEError> {
        // Check cache first for performance
        let cache_key = format!(
            "{}:{}", params.text_document_position_params.text_document.uri,
            params.text_document_position_params.position
        );

        if let Some(cached_completions) = self.get_cached_completions(&cache_key).await {
            return Ok(Some(CompletionList {
                is_incomplete: false,
                items: cached_completions,
            }));
        }

        // Use the enhanced context-aware completion provider
        let enhanced_completions = self.context_aware_provider
            .generate_predictive_completions(&params)
            .await?;

        // Convert enhanced completions to standard LSP items
        let mut completion_items = Vec::new();
        for enhanced_item in enhanced_completions {
            completion_items.push(enhanced_item.item);
        }

        // Limit to configured maximum
        let max_items = self.config.max_completion_items;
        if completion_items.len() > max_items {
            completion_items.truncate(max_items);
        }

        // Cache the results for future requests
        self.cache_completions(&cache_key, &completion_items).await;

        Ok(Some(CompletionList {
            is_incomplete: false, // Predictive completions are complete
            items: completion_items,
        }))
    }

    async fn build_completion_context(&self, params: &CompletionParams) -> Result<CompletionContextData, IDEError> {
        let position = params.text_document_position_params.position;

        // For now, return simple context - in practice this would get actual file content
        Ok(CompletionContextData {
            position: position.clone(),
            trigger_character: params.context.as_ref()
                .and_then(|ctx| ctx.trigger_character.clone()),
            prefix_context: vec![
                "use std::collections::HashMap;".to_string(),
                "struct Example {".to_string(),
            ],
            current_line: params.context.as_ref()
                .map(|ctx| match ctx.trigger_character.as_deref() {
                    Some(ch) => format!("some_text{}", ch),
                    None => "some_code".to_string(),
                })
                .unwrap_or_default(),
            suffix_context: vec![
                "fn example() {".to_string(),
                "    // some code".to_string(),
            ],
            language: "rust".to_string(),
            available_symbols: HashSet::from([
                "HashMap".to_string(),
                "Vec".to_string(),
                "String".to_string(),
                "println".to_string(),
            ]),
        })
    }

    async fn generate_ai_completions(&self, context: &CompletionContextData) -> Result<Vec<CompletionItem>, IDEError> {
        let mut ai_completions = Vec::new();

        // Simple AI-like logic: suggest common patterns based on context
        if context.current_line.contains("let") || context.current_line.ends_with(" = ") {
            ai_completions.push(CompletionItem {
                label: "Vec::new()".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Create empty vector".to_string()),
                documentation: Some(lsp_types::Documentation::String("Creates an empty Vec with generic type T".to_string())),
                insert_text: Some("Vec::new()".to_string()),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                command: None,
                data: None,
                sort_text: Some("aa_vec_new".to_string()),
                filter_text: None,
                preselect: None,
                commit_characters: None,
                additional_text_edits: None,
                ..Default::default()
            });

            ai_completions.push(CompletionItem {
                label: "HashMap::new()".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Create empty hash map".to_string()),
                documentation: Some(lsp_types::Documentation::String("Creates an empty HashMap with generic key-value types".to_string())),
                insert_text: Some("HashMap::new()".to_string()),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                command: None,
                data: None,
                sort_text: Some("aa_hashmap_new".to_string()),
                filter_text: None,
                preselect: None,
                commit_characters: None,
                additional_text_edits: None,
                ..Default::default()
            });

            ai_completions.push(CompletionItem {
                label: "Some(".to_string(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                detail: Some("Option::Some".to_string()),
                documentation: Some(lsp_types::Documentation::String("Creates a Some value of Option<T>". If the value is not None, a Some variant is created.".to_string())),
                insert_text: Some("Some($1)".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                command: None,
                data: None,
                sort_text: Some("aa_some".to_string()),
                filter_text: None,
                preselect: None,
                commit_characters: None,
                additional_text_edits: None,
                ..Default::default()
            });
        }

        if context.current_line.ends_with(".") {
            // Suggest common method chains
            ai_completions.push(CompletionItem {
                label: "map()".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("Iterator::map ".to_string()),
                documentation: Some(lsp_types::Documentation::String("Transforms each element of an iterator with the provided closure ".to_string())),
                insert_text: Some("map(${1:|element|) { ${2:$element} }".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                command: None,
                data: None,
                sort_text: Some("aa_map".to_string()),
                filter_text: None,
                preselect: None,
                commit_characters: None,
                additional_text_edits: None,
                ..Default::default()
            });

            ai_completions.push(CompletionItem {
                label: "iter()".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("Iterator::iter ".to_string()),
                documentation: Some(lsp_types::Documentation::String("Returns an iterator over the elements of a collection ".to_string())),
                insert_text: Some("iter()".to_string()),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                command: None,
                data: None,
                sort_text: Some("aa_iter".to_string()),
                filter_text: None,
                preselect: None,
                commit_characters: None,
                additional_text_edits: None,
                ..Default::default()
            });
        }

        // Add struct initialization suggestions
        if context.current_line.contains("let") && context.current_line.contains(": ") {
            ai_completions.push(CompletionItem {
                label: "Default::default()".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Create default instance ".to_string()),
                documentation: Some(lsp_types::Documentation::String("Creates a new instance with default values using the Default trait ".to_string())),
                insert_text: Some("Default::default()".to_string()),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                command: None,
                data: None,
                sort_text: Some("aa_default".to_string()),
                filter_text: None,
                preselect: None,
                commit_characters: None,
                additional_text_edits: None,
                ..Default::default()
            });
        }

        Ok(ai_completions)
    }

    async fn generate_context_completions(&self, context: &CompletionContextData) -> Result<Vec<CompletionItem>, IDEError> {
        let mut context_completions = Vec::new();

        // Suggest previously imported items
        for line in &context.prefix_context {
            if line.starts_with("use ") {
                // Extract symbols from use statements
                if let Some(module_path) = line.splitn(2, "use ").nth(1) {
                    let symbols: Vec<&str> = module_path.split(&[':', ','][..]).collect();
                    for symbol in symbols {
                        let symbol = symbol.trim();
                        if !symbol.is_empty() && !symbol.contains(' ') {
                            context_completions.push(CompletionItem {
                                label: symbol.to_string(),
                                kind: Some(CompletionItemKind::MODULE),
                                detail: Some("Imported symbol ".to_string()),
                                insert_text: Some(symbol.to_string()),
                                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                                command: None,
                                data: None,
                                sort_text: Some(format!("bb_{}", symbol)),
                                filter_text: None,
                                preselect: None,
                                commit_characters: None,
                                additional_text_edits: None,
                                ..Default::default()
                            });
                        }
                    }
                }
            }
        }

        Ok(context_completions)
    }

    async fn sort_and_deduplicate_completions(&self, items: &mut Vec<CompletionItem>) -> Result<(), IDEError> {
        let mut seen = HashSet::new();
        let mut deduplicated = Vec::new();

        for item in items {
            let key = format!("{}_{}", item.label, item.detail.as_ref().unwrap_or(&"".to_string()));
            if !seen.contains(&key) {
                seen.insert(key);
                deduplicated.push(item.clone());
            }
        }

        // Sort by relevance (sort_text takes precedence)
        deduplicated.sort_by(|a, b| {
            // AI completions come first (start with 'aa')
            match (a.sort_text.as_deref(), b.sort_text.as_deref()) {
                (Some(a_sort), Some(b_sort)) if a_sort.starts_with("aa_") && !b_sort.starts_with("aa_") => std::cmp::Ordering::Less,
                (Some(a_sort), Some(b_sort)) if !a_sort.starts_with("aa_") && b_sort.starts_with("aa_") => std::cmp::Ordering::Greater,
                _ => a.label.cmp(&b.label),
            }
        });

        *items = deduplicated;
        Ok(())
    }

    /// Record completion acceptance for learning and AI context updates
    pub async fn record_completion_accepted(&self, completion: &CompletionItem) -> Result<(), IDEError> {
        // Record in completion history
        let mut history = self.completion_history.write().await;
        let key = completion.label.clone();
        *history.entry(key).or_insert(0) += 1;

        // Inform context-aware provider for learning
        if let Some(enhanced_item) = self.find_enhanced_completion(&completion) {
            self.context_aware_provider.record_completion_accepted(&enhanced_item).await?;
        }

        tracing::info!("Recorded completion acceptance: {}", completion.label);
        Ok(())
    }

    /// Get completion statistics for analytics
    pub async fn get_completion_stats(&self) -> HashMap<String, usize> {
        self.completion_history.read().await.clone()
    }

    /// Get cached completions for performance
    async fn get_cached_completions(&self, cache_key: &str) -> Option<Vec<CompletionItem>> {
        let cache = self.completion_cache.read().await;
        cache.get(cache_key).cloned()
    }

    /// Cache completions for future requests
    async fn cache_completions(&self, cache_key: &str, completions: &[CompletionItem]) {
        let mut cache = self.completion_cache.write().await;
        cache.insert(cache_key.to_string(), completions.to_vec());
    }

    /// Find enhanced completion item from standard completion (helper method)
    fn find_enhanced_completion(&self, completion: &CompletionItem) -> Option<EnhancedCompletionItem> {
        // This would ideally match against cached enhanced items
        // For now, create a simple match
        Some(EnhancedCompletionItem {
            item: completion.clone(),
            confidence: 0.8,
            source: CompletionSource::Traditional,
            context_relevance: 0.9,
            usage_frequency: 1,
            project_relevance: 0.7,
        })
    }

    /// Get AI context manager for external access
    pub fn ai_context_manager(&self) -> Arc<AIContextManager> {
        Arc::clone(&self.ai_context_manager)
    }

    /// Get context-aware completion provider for external access
    pub fn context_aware_provider(&self) -> Arc<ContextAwareCompletionProvider> {
        Arc::clone(&self.context_aware_provider)
    }
}

/// Request handler for LSP completion
pub async fn handle_completion_request(
    provider: &CompletionProvider,
    params: lsp_types::CompletionParams,
) -> Result<Option<lsp_types::CompletionList>, rust_ai_ide_errors::LSPError> {
    Ok(provider.request_completion(params).await
        .map_err(|e| rust_ai_ide_errors::LSPError::ProtocolError(format!("Completion error: {}", e)))?
        .map(lsp_types::CompletionList::from))
}

/// Resolve additional completion item information
pub async fn handle_completion_item_resolve(
    _provider: &CompletionProvider,
    _params: lsp_types::CompletionItem,
) -> Result<lsp_types::CompletionItem, rust_ai_ide_errors::LSPError> {
    Err(rust_ai_ide_errors::LSPError::ProtocolError("Not implemented yet".to_string()))
}

// Need to implement the LSP compatibility trait
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_provider_initialization() {
        let config = CompletionConfig::default();
        let provider = CompletionProvider::new(config);
        // Test would require async runtime
    }

    #[test]
    fn test_completion_config_defaults() {
        let config = CompletionConfig::default();
        assert!(config.enable_ai_completion);
        assert!(config.enable_context_completion);
        assert_eq!(config.max_completion_items, 50);
        assert!(config.enable_fuzzy_matching);
        assert_eq!(config.ai_context_window_size, 20);
        assert_eq!(config.ai_min_confidence, 0.3);
    }
}
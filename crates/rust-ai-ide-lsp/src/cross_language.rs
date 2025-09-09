//! Cross-language capabilities for multi-language LSP support
//!
//! This module implements cross-language symbol search, navigation,
//! and intelligent features that work across multiple programming languages.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

use futures::future::join_all;
use lsp_types::{
    CallHierarchyItem, CallHierarchyPrepareParams, Diagnostic, DocumentSymbol, Location,
    SymbolKind, TextEdit, Uri, WorkspaceEdit, *,
};
use tracing::{debug, info, warn};

use crate::client::LSPError;
use crate::language_router::{LanguageRouter, RequestContext, RoutingResult};
use crate::language_server::{LanguageServerHandle, LanguageServerKind};
use crate::pool::LanguageServerPool;

/// Cross-language symbol reference
#[derive(Debug, Clone)]
pub struct CrossLanguageSymbol {
    /// Symbol name
    pub name: String,
    /// Symbol kind (function, class, variable, etc.)
    pub kind: SymbolKind,
    /// Location in source code
    pub location: Location,
    /// Documentation or definition content
    pub documentation: Option<String>,
    /// References to this symbol
    pub references: Vec<Location>,
    /// Origin language
    pub language: LanguageServerKind,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Cross-language document reference
#[derive(Debug, Clone)]
pub struct CrossLanguageDocument {
    /// Document URI
    pub uri: Uri,
    /// Document language
    pub language: LanguageServerKind,
    /// Document content summary
    pub summary: String,
    /// Document symbols
    pub symbols: Vec<DocumentSymbol>,
    /// Document diagnostics
    pub diagnostics: Vec<Diagnostic>,
}

/// Cross-language search result
#[derive(Debug, Clone)]
pub struct CrossLanguageSearchResult {
    /// Matching symbols
    pub symbols: Vec<CrossLanguageSymbol>,
    /// Matching documents
    pub documents: Vec<CrossLanguageDocument>,
    /// Search query
    pub query: String,
    /// Search statistics
    pub statistics: SearchStatistics,
}

/// Search query configuration
#[derive(Debug, Clone)]
pub struct SearchConfiguration {
    /// Maximum number of results to return
    pub max_results: usize,
    /// Include documents in search (not just symbols)
    pub include_documents: bool,
    /// Search only within workspace
    pub workspace_only: bool,
    /// Include definitions only
    pub definitions_only: bool,
    /// Include references
    pub include_references: bool,
    /// Fuzzy matching enabled
    pub fuzzy_matching: bool,
    /// Search timeout in milliseconds
    pub timeout_ms: u64,
}

/// Search statistics
#[derive(Debug, Clone, Default)]
pub struct SearchStatistics {
    /// Total symbols searched
    pub total_symbols: usize,
    /// Total documents searched
    pub total_documents: usize,
    /// Number of languages involved
    pub languages_involved: usize,
    /// Search duration in milliseconds
    pub search_duration_ms: u64,
    /// Cache hits during search
    pub cache_hits: usize,
    /// Server request count
    pub server_requests: usize,
}

/// Cross-language refactoring operation
#[derive(Debug, Clone)]
pub struct CrossLanguageRefactoring {
    /// Refactoring operation type
    pub operation: CrossLanguageRefactoringOperation,
    /// Symbols affected
    pub affected_symbols: Vec<CrossLanguageSymbol>,
    /// Changes required across files
    pub changes: HashMap<Uri, Vec<TextEdit>>,
    /// Languages involved
    pub languages_affected: HashSet<LanguageServerKind>,
}

/// Type of cross-language refactoring
#[derive(Debug, Clone)]
pub enum CrossLanguageRefactoringOperation {
    /// Rename symbol across multiple languages
    RenameSymbol {
        old_symbol: String,
        new_symbol: String,
        symbol_type: SymbolKind,
    },
    /// Move symbol between files/languages
    MoveSymbol {
        symbol: String,
        from_location: Location,
        to_location: Location,
    },
    /// Extract symbol with multi-language impact
    ExtractSymbol {
        new_symbol: String,
        new_symbol_kind: SymbolKind,
    },
    /// Call hierarchy analysis across languages
    AnalyzeCallHierarchy { symbol: String, location: Location },
}

/// Cross-language capabilities manager
pub struct CrossLanguageManager {
    /// Language router for request routing
    router: Arc<RwLock<LanguageRouter>>,
    /// Language server pool
    pool: Arc<RwLock<LanguageServerPool>>,
    /// Symbol index across all languages
    symbol_index: Arc<RwLock<SymbolIndex>>,
    /// Reference tracking
    reference_tracker: Arc<RwLock<ReferenceTracker>>,
    /// Search configuration
    search_config: SearchConfiguration,
}

impl CrossLanguageManager {
    /// Create a new cross-language manager
    pub fn new(router: Arc<RwLock<LanguageRouter>>, pool: Arc<RwLock<LanguageServerPool>>) -> Self {
        Self {
            router,
            pool,
            symbol_index: Arc::new(RwLock::new(SymbolIndex::new())),
            reference_tracker: Arc::new(RwLock::new(ReferenceTracker::new())),
            search_config: SearchConfiguration::default(),
        }
    }

    /// Perform cross-language symbol search
    pub async fn search_symbols(
        &self,
        query: &str,
        config: Option<SearchConfiguration>,
    ) -> Result<CrossLanguageSearchResult, LSPError> {
        let start_time = std::time::Instant::now();
        let _config = config.unwrap_or_else(|| self.search_config.clone());

        debug!("Performing cross-language symbol search for '{}'", query);

        let mut symbols = Vec::new();
        let mut documents = Vec::new();
        let mut statistics = SearchStatistics::default();

        // Get all available languages
        let pool = self.pool.read().await;
        let available_languages = pool.get_supported_languages();

        // Search each language server for symbols
        let search_tasks = available_languages.iter().map(|language| {
            let pool = self.pool.clone();
            let router = self.router.clone();
            let query = query.to_string();
            let config = _config.clone();

            async move {
                Self::search_symbols_in_language(&pool, &router, &query, language, &config).await
            }
        });

        let search_results = join_all(search_tasks).await;

        // Aggregate results
        for result in search_results {
            match result {
                Ok((lang_symbols, lang_docs, lang_stats)) => {
                    symbols.extend(lang_symbols);
                    documents.extend(lang_docs);
                    statistics.total_symbols += lang_stats.total_symbols;
                    statistics.total_documents += lang_stats.total_documents;
                    statistics.server_requests += lang_stats.server_requests;
                }
                Err(e) => {
                    warn!("Search failed for one language: {}", e);
                }
            }
        }

        statistics.languages_involved = available_languages.len();
        statistics.search_duration_ms = start_time.elapsed().as_millis() as u64;

        // Limit results
        symbols.truncate(_config.max_results);
        documents.truncate(if _config.include_documents {
            _config.max_results / 2
        } else {
            0
        });

        info!(
            "Cross-language search for '{}' completed in {}ms: {} symbols, {} documents from {} languages",
            query,
            statistics.search_duration_ms,
            symbols.len(),
            documents.len(),
            statistics.languages_involved
        );

        Ok(CrossLanguageSearchResult {
            symbols,
            documents,
            query: query.to_string(),
            statistics,
        })
    }

    /// Find symbol references across multiple languages
    pub async fn find_references(
        &self,
        location: &Location,
        config: Option<SearchConfiguration>,
    ) -> Result<Vec<CrossLanguageSymbol>, LSPError> {
        let _config = config.unwrap_or_else(|| self.search_config.clone());

        debug!(
            "Finding cross-language references for location {:?}",
            location
        );

        let pool = self.pool.read().await;
        let available_languages = pool.get_supported_languages();

        // Start with the language of the target location
        let mut result_symbols = Vec::new();
        let mut visited_uris = std::collections::HashSet::new();
        visited_uris.insert(location.uri.clone());

        for language in &available_languages {
            match Self::find_references_in_language(
                &self.pool,
                &self.router,
                location,
                language,
                &visited_uris,
            )
            .await
            {
                Ok(symbols) => {
                    // Filter duplicates
                    for symbol in symbols {
                        if !visited_uris.contains(&symbol.location.uri) {
                            visited_uris.insert(symbol.location.uri.clone());
                            result_symbols.push(symbol);
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to find references in {}: {}",
                        language_name(language),
                        e
                    );
                }
            }
        }

        Ok(result_symbols)
    }

    /// Analyze call hierarchy across languages
    pub async fn analyze_call_hierarchy(
        &self,
        location: &Location,
        config: Option<SearchConfiguration>,
    ) -> Result<Vec<lsp_types::CallHierarchyItem>, LSPError> {
        let _config = config.unwrap_or_else(|| self.search_config.clone());

        debug!(
            "Analyzing cross-language call hierarchy for location {:?}",
            location
        );

        // Route to appropriate language server
        let context = RequestContext {
            method: "textDocument/prepareCallHierarchy".to_string(),
            document_uri: Some(location.uri.clone()),
            position: Some(location.range.start),
            selection: None,
            file_path_hint: None,
            language_hint: None,
            workspace_root: None,
        };

        let RoutingResult { server_handle, .. } =
            self.router.read().await.route_request(&context).await?;

        if let Some(server_handle) = server_handle {
            let server_wrapper = server_handle.read().await;

            if server_wrapper.server.is_initialized() {
                let params = CallHierarchyPrepareParams {
                    text_document_position_params: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier {
                            uri: location.uri.clone(),
                        },
                        position: location.range.start,
                    },
                    work_done_progress_params: Default::default(),
                };

                let params_value = serde_json::to_value(params)
                    .map_err(|e| LSPError::Other(format!("Serialization error: {}", e)))?;
                match server_wrapper
                    .server
                    .send_request("textDocument/prepareCallHierarchy", params_value)
                    .await
                {
                    Ok(response) => {
                        let hierarchy_items: Option<Vec<CallHierarchyItem>> =
                            serde_json::from_value(response).map_err(|e| {
                                LSPError::Other(format!("Deserialization error: {}", e))
                            })?;
                        if let Some(hierarchy_items) = hierarchy_items {
                            // Expand hierarchy to cross-language level
                            let expanded_items = self
                                .expand_call_hierarchy(&hierarchy_items, server_handle.clone())
                                .await?;
                            return Ok(expanded_items);
                        } else {
                            return Ok(vec![]);
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
        }

        Err(LSPError::Other(
            "No suitable language server found for call hierarchy".to_string(),
        ))
    }

    /// Perform cross-language refactoring
    pub async fn perform_cross_language_refactoring(
        &self,
        refactoring: CrossLanguageRefactoring,
    ) -> Result<WorkspaceEdit, LSPError> {
        debug!("Performing cross-language refactoring: {:?}", refactoring);

        let mut workspace_edit = WorkspaceEdit {
            changes: Some(HashMap::new()),
            document_changes: None,
            change_annotations: None,
        };

        // Apply changes for each affected file
        for (uri, edits) in &refactoring.changes {
            let changes = workspace_edit.changes.as_mut().unwrap();
            changes.insert(uri.clone(), edits.clone());
        }

        info!(
            "Cross-language refactoring affected {} files across {} languages",
            refactoring.changes.len(),
            refactoring.languages_affected.len()
        );

        Ok(workspace_edit)
    }

    /// Prepare cross-language refactoring preview
    pub async fn prepare_refactoring_preview(
        &self,
        symbol: &CrossLanguageSymbol,
        operation: &CrossLanguageRefactoringOperation,
    ) -> Result<CrossLanguageRefactoring, LSPError> {
        debug!(
            "Preparing cross-language refactoring preview for symbol '{}' in {:?}",
            symbol.name, symbol.language
        );

        let mut affected_symbols = Vec::new();
        let mut all_changes = HashMap::new();
        let mut affected_languages = HashSet::new();

        // Find all references to prepare changes
        let references = self.find_references(&symbol.location, None).await?;
        affected_symbols.extend(references);

        // Prepare changes for each affected symbol location
        for symbol_ref in &affected_symbols {
            let edits = Self::prepare_symbol_edit(symbol, operation)?;
            all_changes.insert(symbol_ref.location.uri.clone(), vec![edits]);
            affected_languages.insert(symbol_ref.language.clone());
        }

        // Also add the original symbol
        affected_symbols.push(symbol.clone());
        affected_languages.insert(symbol.language.clone());

        Ok(CrossLanguageRefactoring {
            operation: operation.clone(),
            affected_symbols,
            changes: all_changes,
            languages_affected: affected_languages,
        })
    }

    /// Update symbol index for improved cross-language search
    ///
    /// # Arguments
    /// * `uri` - Document URI to update
    /// * `_content` - Optional document content (reserved for future implementation)
    pub async fn update_symbol_index(
        &self,
        uri: &Uri,
        _content: Option<&str>,
    ) -> Result<(), LSPError> {
        debug!("Updating symbol index for URI {}", uri.as_str());

        let context = RequestContext {
            method: "textDocument/documentSymbol".to_string(),
            document_uri: Some(uri.clone()),
            position: None,
            selection: None,
            file_path_hint: None,
            language_hint: None,
            workspace_root: None,
        };

        let RoutingResult {
            server_handle,
            target_language,
            ..
        } = self.router.read().await.route_request(&context).await?;

        if let Some(server_handle) = server_handle {
            let server_wrapper = server_handle.read().await;

            if server_wrapper.server.is_initialized() {
                let params = DocumentSymbolParams {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    work_done_progress_params: Default::default(),
                    partial_result_params: Default::default(),
                };

                let params_value = serde_json::to_value(params)
                    .map_err(|e| LSPError::Other(format!("Serialization error: {}", e)))?;
                if let Ok(response) = server_wrapper
                    .server
                    .send_request("textDocument/documentSymbol", params_value)
                    .await
                {
                    let symbols_response: Option<DocumentSymbolResponse> =
                        serde_json::from_value(response).map_err(|e| {
                            LSPError::Other(format!("Deserialization error: {}", e))
                        })?;
                    if let Some(symbols_response) = symbols_response {
                        // Handle DocumentSymbolResponse - convert to Vec<DocumentSymbol>
                        let symbols = match &symbols_response {
                            DocumentSymbolResponse::Flat(symbols) => {
                                // Convert SymbolInformation to CrossLanguageSymbol format
                                symbols
                                    .iter()
                                    .map(|symbol| CrossLanguageSymbol {
                                        name: symbol.name.clone(),
                                        kind: symbol.kind,
                                        location: symbol.location.clone(),
                                        documentation: Some(
                                            symbol.container_name.clone().unwrap_or_default(),
                                        ),
                                        references: vec![],
                                        language: target_language.clone(),
                                        metadata: HashMap::new(),
                                    })
                                    .collect::<Vec<_>>()
                            }
                            DocumentSymbolResponse::Nested(ref symbols) => {
                                // Flatten the nested symbols properly with recursion and convert
                                let flattened = Self::flatten_nested_symbols(symbols.clone());
                                flattened
                                    .into_iter()
                                    .map(|symbol| CrossLanguageSymbol {
                                        name: symbol.name,
                                        kind: symbol.kind,
                                        location: Location {
                                            uri: uri.clone(),
                                            range: symbol.range,
                                        },
                                        documentation: symbol.detail,
                                        references: vec![],
                                        language: target_language.clone(),
                                        metadata: HashMap::new(),
                                    })
                                    .collect::<Vec<_>>()
                            }
                        };

                        // Update symbol index with original DocumentSymbol format
                        let symbols_for_index = match &symbols_response {
                            DocumentSymbolResponse::Flat(symbols) => {
                                // Convert SymbolInformation to DocumentSymbol for consistency
                                symbols
                                    .iter()
                                    .map(|symbol| lsp_types::DocumentSymbol {
                                        name: symbol.name.clone(),
                                        detail: Some(
                                            symbol.container_name.clone().unwrap_or_default(),
                                        ),
                                        kind: symbol.kind,
                                        range: symbol.location.range,
                                        selection_range: symbol.location.range,
                                        children: None,
                                        deprecated: None, // Deprecated field removed - use tags instead
                                        tags: Some(vec![]),
                                    })
                                    .collect::<Vec<_>>()
                            }
                            DocumentSymbolResponse::Nested(ref nested) => {
                                Self::flatten_nested_symbols(nested.clone())
                            }
                        };

                        let mut index = self.symbol_index.write().await;
                        index.update_document_symbols(uri, &symbols_for_index, &target_language);

                        info!(
                            "Updated symbol index for URI {} with {} symbols",
                            uri.as_str(),
                            symbols_for_index.len()
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Search symbols within a single language
    async fn search_symbols_in_language(
        pool: &Arc<RwLock<LanguageServerPool>>,
        router: &Arc<RwLock<LanguageRouter>>,
        query: &str,
        language: &LanguageServerKind,
        config: &SearchConfiguration,
    ) -> Result<
        (
            Vec<CrossLanguageSymbol>,
            Vec<CrossLanguageDocument>,
            SearchStatistics,
        ),
        LSPError,
    > {
        let _symbols: Vec<CrossLanguageSymbol> = Vec::new();
        let documents = Vec::new();
        let mut statistics = SearchStatistics::default();

        // Get servers for this language
        let pool_guard = pool.read().await;
        let servers = pool_guard.get_servers_for_language(language);
        drop(pool_guard);

        statistics.server_requests = servers.len();

        for server_handle in servers {
            let server_wrapper = server_handle.read().await;

            if !server_wrapper.server.is_initialized() {
                continue;
            }

            // Perform symbol search for this server
            // This would typically involve workspace/symbol or document/symbol requests
            // For now, we'll use a simplified approach

            // TODO: Implement actual symbol search per server
            statistics.total_symbols += 0; // Update with actual count
            statistics.total_documents += 1; // Update with actual document count
        }

        Ok((Vec::new(), documents, statistics))
    }

    /// Find references in a single language
    async fn find_references_in_language(
        pool: &Arc<RwLock<LanguageServerPool>>,
        router: &Arc<RwLock<LanguageRouter>>,
        location: &Location,
        language: &LanguageServerKind,
        visited_uris: &HashSet<Uri>,
    ) -> Result<Vec<CrossLanguageSymbol>, LSPError> {
        let pool_guard = pool.read().await;
        let servers = pool_guard.get_servers_for_language(language);
        drop(pool_guard);

        // TODO: Implement actual reference finding per language
        // For now, return empty vector as this is a placeholder implementation
        Ok(Vec::new())
    }

    /// Expand call hierarchy to include cross-language calls
    async fn expand_call_hierarchy(
        &self,
        items: &Vec<CallHierarchyItem>,
        _server_handle: LanguageServerHandle,
    ) -> Result<Vec<CallHierarchyItem>, LSPError> {
        // TODO: Implement cross-language call hierarchy expansion
        // For now, return the items as-is
        Ok(items.clone())
    }

    /// Flatten nested DocumentSymbols recursively
    fn flatten_nested_symbols(symbols: Vec<DocumentSymbol>) -> Vec<DocumentSymbol> {
        let mut result = Vec::new();

        for symbol in symbols {
            result.push(symbol.clone());

            // Recursively flatten children if they exist
            if let Some(children) = &symbol.children {
                result.extend(Self::flatten_nested_symbols(children.clone()));
            }
        }

        result
    }

    /// Prepare symbol edit for refactoring operation
    fn prepare_symbol_edit(
        symbol: &CrossLanguageSymbol,
        operation: &CrossLanguageRefactoringOperation,
    ) -> Result<TextEdit, LSPError> {
        match operation {
            CrossLanguageRefactoringOperation::RenameSymbol { new_symbol, .. } => Ok(TextEdit {
                range: symbol.location.range,
                new_text: new_symbol.clone(),
            }),
            _ => Err(LSPError::Other(
                "Refactoring operation not implemented".to_string(),
            )),
        }
    }
}

/// Global symbol index for fast cross-language searches
#[derive(Debug, Clone)]
struct SymbolIndex {
    /// URI -> symbols mapping
    uri_symbols: HashMap<Uri, Vec<DocumentSymbol>>,
    /// Language -> symbols mapping
    language_symbols: HashMap<LanguageServerKind, Vec<CrossLanguageSymbol>>,
    /// Name -> symbols mapping for fast lookup
    name_index: HashMap<String, Vec<CrossLanguageSymbol>>,
}

impl SymbolIndex {
    fn new() -> Self {
        Self {
            uri_symbols: HashMap::new(),
            language_symbols: HashMap::new(),
            name_index: HashMap::new(),
        }
    }

    fn update_document_symbols(
        &mut self,
        uri: &Uri,
        symbols: &Vec<DocumentSymbol>,
        language: &LanguageServerKind,
    ) {
        // Convert LSP symbols to cross-language symbols
        let cross_symbols: Vec<CrossLanguageSymbol> = symbols
            .iter()
            .map(|symbol| CrossLanguageSymbol {
                name: symbol.name.clone(),
                kind: symbol.kind,
                location: Location {
                    uri: uri.clone(),
                    range: symbol.range,
                },
                documentation: symbol.detail.clone(),
                references: vec![],
                language: language.clone(),
                metadata: HashMap::new(),
            })
            .collect();

        // Update URI mapping
        self.uri_symbols.insert(uri.clone(), symbols.clone());

        // Update language mapping
        let language_symbols = self.language_symbols.entry(language.clone()).or_default();
        language_symbols.extend(cross_symbols.clone());

        // Update name index for fast search
        for symbol in cross_symbols {
            let name_symbols = self.name_index.entry(symbol.name.clone()).or_default();
            name_symbols.push(symbol);
        }
    }
}

/// Reference tracking for cross-language dependencies
#[derive(Debug, Clone)]
struct ReferenceTracker {
    /// Symbol -> referenced symbols mapping
    symbol_references: HashMap<String, Vec<CrossLanguageSymbol>>,
    /// Symbol dependency graph
    dependency_graph: HashMap<String, HashSet<String>>,
}

impl ReferenceTracker {
    fn new() -> Self {
        Self {
            symbol_references: HashMap::new(),
            dependency_graph: HashMap::new(),
        }
    }
}

/// Helper function to get display name for language
fn language_name(language: &LanguageServerKind) -> String {
    match language {
        LanguageServerKind::Rust => "Rust".to_string(),
        LanguageServerKind::TypeScript => "TypeScript".to_string(),
        LanguageServerKind::JavaScript => "JavaScript".to_string(),
        LanguageServerKind::Python => "Python".to_string(),
        LanguageServerKind::Go => "Go".to_string(),
        LanguageServerKind::Custom(name) => name.clone(),
    }
}

impl Default for SearchConfiguration {
    fn default() -> Self {
        Self {
            max_results: 50,
            include_documents: true,
            workspace_only: true,
            definitions_only: false,
            include_references: true,
            fuzzy_matching: true,
            timeout_ms: 5000,
        }
    }
}

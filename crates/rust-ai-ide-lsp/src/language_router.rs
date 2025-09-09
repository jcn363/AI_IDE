//! Language request router for multi-language LSP support
//!
//! This module provides intelligent routing of LSP requests to the appropriate
//! language servers based on file types, content, and user context.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use lsp_types::*;
use tracing::{debug, info, warn};

use crate::client::LSPError;
use crate::language_detection::LanguageDetector;
use crate::language_server::{LanguageServerHandle, LanguageServerKind, ServerHealth};
use crate::pool::LanguageServerPool;

/// Request routing context
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// The LSP method being requested
    pub method: String,
    /// URI of the document being worked on
    pub document_uri: Option<Uri>,
    /// Position in the document (for position-related requests)
    pub position: Option<Position>,
    /// Range in the document (for range-related requests)
    pub selection: Option<Range>,
    /// File path hint provided by client
    pub file_path_hint: Option<String>,
    /// Language-specific hint provided by client
    pub language_hint: Option<String>,
    /// Workspace root path
    pub workspace_root: Option<String>,
}

/// Routing strategy for language server selection
#[derive(Debug, Clone, PartialEq)]
pub enum RoutingStrategy {
    /// Route based on file extension
    FileExtension,
    /// Route based on content analysis
    ContentAnalysis,
    /// Route based on explicit language hint
    LanguageHint,
    /// Route based on workspace configuration
    WorkspaceConfiguration,
    /// Use intelligent multi-factor analysis
    Intelligent,
}

/// Routing result for a language request
#[derive(Debug, Clone)]
pub struct RoutingResult {
    /// Target language server kind
    pub target_language: LanguageServerKind,
    /// Selected language server handle
    pub server_handle: Option<LanguageServerHandle>,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Routing strategy used
    pub strategy_used: RoutingStrategy,
    /// Number of candidates considered
    pub candidates_evaluated: usize,
    /// Reasoning for the routing decision
    pub reason: String,
}

/// Load balancing strategy for multiple servers of the same language
#[derive(Debug, Clone, PartialEq)]
pub enum LoadBalancingStrategy {
    /// Round-robin between available servers
    RoundRobin,
    /// Least loaded server
    LeastLoaded,
    /// Random server
    Random,
    /// Health-based (prefer healthy servers)
    HealthBased,
}

/// Language request router for intelligent multi-language LSP support
pub struct LanguageRouter {
    /// Language server pool for server management
    pool: Arc<RwLock<LanguageServerPool>>,
    /// Language detector for automatic file type recognition (reserved for future implementation)
    _detector: LanguageDetector,
    /// Request routing strategy
    routing_strategy: RoutingStrategy,
    /// Load balancing strategy for multiple servers
    load_balancing: LoadBalancingStrategy,
    /// Cache for recent routing decisions
    routing_cache: Arc<RwLock<HashMap<String, RoutingResult>>>,
    /// Round-robin state for load balancing
    round_robin_state: Arc<RwLock<HashMap<LanguageServerKind, usize>>>,
    /// Router statistics
    stats: Arc<RwLock<RouterStatistics>>,
}

impl LanguageRouter {
    /// Create a new language router
    pub fn new(pool: Arc<RwLock<LanguageServerPool>>) -> Self {
        Self {
            pool,
            _detector: LanguageDetector::default(),
            routing_strategy: RoutingStrategy::Intelligent,
            load_balancing: LoadBalancingStrategy::LeastLoaded,
            routing_cache: Arc::new(RwLock::new(HashMap::new())),
            round_robin_state: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(RouterStatistics::default())),
        }
    }

    /// Route an LSP request to the appropriate language server
    pub async fn route_request(&self, context: &RequestContext) -> Result<RoutingResult, LSPError> {
        let start_time = std::time::Instant::now();

        debug!(
            "Routing request for method '{}' with URI {:?}",
            context.method, context.document_uri
        );

        // Check routing cache first
        if let Some(uri) = &context.document_uri {
            if let Some(cached_result) = self.routing_cache.read().await.get(&uri.to_string()) {
                let cache_age = start_time.duration_since(
                    std::time::Instant::now() - std::time::Duration::from_secs(300), // 5 min cache
                );
                if cache_age.as_secs() < 300 {
                    // 5 minutes
                    debug!("Using cached routing result for URI {}", uri.as_str());
                    let mut stats = self.stats.write().await;
                    stats.cache_hits += 1;
                    return Ok(cached_result.clone());
                }
            }
        }

        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        drop(stats);

        // Determine target language based on routing strategy
        let target_language = self.determine_target_language(context).await?;
        debug!("Target language determined: {:?}", target_language);

        // Select specific server instance
        let server_handle = self.select_server_instance(&target_language).await?;
        debug!(
            "Server instance selected for language {:?}",
            target_language
        );

        // Create routing result
        let routing_result = RoutingResult {
            target_language: target_language.clone(),
            server_handle,
            confidence: 0.95, // High confidence for initial implementation
            strategy_used: self.routing_strategy.clone(),
            candidates_evaluated: 1, // TODO: Track actual candidates
            reason: format!(
                "Routed to {:?} using {:?} strategy",
                target_language, self.routing_strategy
            ),
        };

        // Cache the result
        if let Some(uri) = &context.document_uri {
            let mut cache = self.routing_cache.write().await;
            cache.insert(uri.to_string(), routing_result.clone());
            // Limit cache size
            if cache.len() > 1000 {
                // Simple LRU eviction - in real implementation, use proper LRU
                let keys_to_remove: Vec<String> =
                    cache.keys().skip(cache.len() - 1000).cloned().collect();
                for key in keys_to_remove {
                    cache.remove(&key);
                }
            }
        }

        let routing_time = start_time.elapsed();
        let mut stats = self.stats.write().await;
        stats.routing_times_ms.push(routing_time.as_millis() as f64);
        if stats.routing_times_ms.len() > 100 {
            stats.routing_times_ms.remove(0); // Keep last 100 measurements
        }

        info!(
            "Routed request for '{}' to {:?} in {:.2}ms",
            context.method,
            target_language,
            routing_time.as_millis()
        );

        Ok(routing_result)
    }

    /// Force refresh of routing cache for a specific URI
    pub async fn refresh_cache(&self, uri: &Uri) {
        let mut cache = self.routing_cache.write().await;
        cache.remove(&uri.to_string());
        debug!("Cleared routing cache for URI {}", uri.as_str());
    }

    /// Clear all cached routing decisions
    pub async fn clear_cache(&self) {
        let mut cache = self.routing_cache.write().await;
        let cache_size = cache.len();
        cache.clear();
        info!("Cleared routing cache ({} entries)", cache_size);
    }

    /// Get router statistics
    pub async fn get_statistics(&self) -> RouterStatistics {
        self.stats.read().await.clone()
    }

    /// Determine target language based on routing context
    async fn determine_target_language(
        &self,
        context: &RequestContext,
    ) -> Result<LanguageServerKind, LSPError> {
        match self.routing_strategy {
            RoutingStrategy::FileExtension => self.route_by_file_extension(context).await,
            RoutingStrategy::LanguageHint => self.route_by_language_hint(context).await,
            RoutingStrategy::ContentAnalysis => self.route_by_content(context).await,
            RoutingStrategy::WorkspaceConfiguration => {
                self.route_by_workspace_config(context).await
            }
            RoutingStrategy::Intelligent => self.route_by_intelligent_analysis(context).await,
        }
    }

    /// Route based on file extension
    async fn route_by_file_extension(
        &self,
        context: &RequestContext,
    ) -> Result<LanguageServerKind, LSPError> {
        if let Some(uri) = &context.document_uri {
            if let Ok(url) = url::Url::parse(uri.as_str()) {
                if let Ok(path) = url.to_file_path() {
                    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                        return Ok(self.extension_to_language(extension));
                    }
                }
            }
        }

        if let Some(path_hint) = &context.file_path_hint {
            let path = Path::new(path_hint);
            if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                return Ok(self.extension_to_language(extension));
            }
        }

        Err(LSPError::Other(
            "Could not determine language from file extension".to_string(),
        ))
    }

    /// Route based on language hint
    async fn route_by_language_hint(
        &self,
        context: &RequestContext,
    ) -> Result<LanguageServerKind, LSPError> {
        if let Some(hint) = &context.language_hint {
            return Ok(self.hint_to_language(hint));
        }

        Err(LSPError::Other("No language hint provided".to_string()))
    }

    /// Route based on content analysis
    async fn route_by_content(
        &self,
        _context: &RequestContext,
    ) -> Result<LanguageServerKind, LSPError> {
        // TODO: Implement content-based routing using LanguageDetector
        // For now, fall back to file extension
        Err(LSPError::Other(
            "Content-based routing not yet implemented".to_string(),
        ))
    }

    /// Route based on workspace configuration
    async fn route_by_workspace_config(
        &self,
        _context: &RequestContext,
    ) -> Result<LanguageServerKind, LSPError> {
        // TODO: Implement workspace-based routing
        // For now, fall back to file extension
        Err(LSPError::Other(
            "Workspace-based routing not yet implemented".to_string(),
        ))
    }

    /// Route using intelligent multi-factor analysis
    async fn route_by_intelligent_analysis(
        &self,
        context: &RequestContext,
    ) -> Result<LanguageServerKind, LSPError> {
        // Use hierarchical fallback strategy:
        // 1. Language hint (highest priority)
        // 2. File extension
        // 3. Content pattern matching
        // 4. Workspace configuration

        // Check language hint first
        if let Some(hint) = &context.language_hint {
            return Ok(self.hint_to_language(hint));
        }

        // Check file extension
        if let Some(uri) = &context.document_uri {
            if let Ok(url) = url::Url::parse(uri.as_str()) {
                if let Ok(path) = url.to_file_path() {
                    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                        return Ok(self.extension_to_language(extension));
                    }
                }
            }
        }

        // Try content analysis for special cases
        if let Ok(lang) = self.route_by_content(context).await {
            return Ok(lang);
        }

        // Try workspace configuration
        if let Ok(lang) = self.route_by_workspace_config(context).await {
            return Ok(lang);
        }

        Err(LSPError::Other(
            "Could not determine target language using intelligent routing".to_string(),
        ))
    }

    /// Select server instance for load balancing
    async fn select_server_instance(
        &self,
        language: &LanguageServerKind,
    ) -> Result<Option<LanguageServerHandle>, LSPError> {
        let pool = self.pool.read().await;
        let servers = pool.get_servers_for_language(language);

        if servers.is_empty() {
            warn!("No servers available for language {:?}", language);
            return Ok(None);
        }

        // Use async filtering since we need to await the read operation
        let mut available_servers = Vec::new();
        for handle in &servers {
            let wrapper = handle.read().await;
            if matches!(wrapper.health_status, ServerHealth::Healthy)
                && wrapper.server.is_initialized()
            {
                available_servers.push(handle.clone());
            }
        }

        if available_servers.is_empty() {
            warn!("No healthy servers available for language {:?}", language);
            return Ok(None);
        }

        let selected_index = self
            .select_server_by_strategy(available_servers.len(), language)
            .await;

        Ok(Some(available_servers[selected_index].clone()))
    }

    /// Select server based on load balancing strategy
    async fn select_server_by_strategy(
        &self,
        available_count: usize,
        language: &LanguageServerKind,
    ) -> usize {
        match self.load_balancing {
            LoadBalancingStrategy::RoundRobin => {
                let mut rr_state = self.round_robin_state.write().await;
                let current_index = rr_state.entry(language.clone()).or_insert(0);
                let selected = *current_index % available_count;
                *current_index = selected + 1;
                selected
            }
            LoadBalancingStrategy::LeastLoaded => {
                // TODO: Implement load-based selection
                0 // For now, select first
            }
            LoadBalancingStrategy::Random => rand::random_range(0..available_count),
            LoadBalancingStrategy::HealthBased => {
                // All servers at this point are already healthy, so use round-robin
                let mut rr_state = self.round_robin_state.write().await;
                let current_index = rr_state.entry(language.clone()).or_insert(0);
                let selected = *current_index % available_count;
                *current_index = selected + 1;
                selected
            }
        }
    }

    /// Convert file extension to language
    fn extension_to_language(&self, extension: &str) -> LanguageServerKind {
        match extension.to_lowercase().as_str() {
            "rs" => LanguageServerKind::Rust,
            "ts" | "tsx" => LanguageServerKind::TypeScript,
            "js" | "jsx" => LanguageServerKind::JavaScript,
            "py" => LanguageServerKind::Python,
            "go" => LanguageServerKind::Go,
            _ => LanguageServerKind::Custom(extension.to_string()),
        }
    }

    /// Convert language hint to LanguageServerKind
    fn hint_to_language(&self, hint: &str) -> LanguageServerKind {
        match hint.to_lowercase().as_str() {
            "rust" => LanguageServerKind::Rust,
            "typescript" | "ts" => LanguageServerKind::TypeScript,
            "javascript" | "js" => LanguageServerKind::JavaScript,
            "python" | "py" => LanguageServerKind::Python,
            "go" | "golang" => LanguageServerKind::Go,
            _ => LanguageServerKind::Custom(hint.to_string()),
        }
    }
}

/// Router performance and usage statistics
#[derive(Debug, Clone, Default)]
pub struct RouterStatistics {
    /// Total number of routing requests processed
    pub total_requests: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Average routing time in milliseconds (last 100 requests)
    pub routing_times_ms: Vec<f64>,
    /// Number of routing errors
    pub routing_errors: u64,
    /// Cache size
    pub cache_size: usize,
}

impl RouterStatistics {
    /// Calculate average routing time
    pub fn average_routing_time(&self) -> Option<f64> {
        if self.routing_times_ms.is_empty() {
            None
        } else {
            Some(self.routing_times_ms.iter().sum::<f64>() / self.routing_times_ms.len() as f64)
        }
    }

    /// Calculate cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_requests as f64
        }
    }

    /// Update cache size
    pub fn update_cache_size(&mut self, size: usize) {
        self.cache_size = size;
    }
}

/// Extension trait for LanguageServerPool to provide router integration
#[async_trait::async_trait]
pub trait LanguageServerPoolRouter {
    /// Get all servers for a specific language
    async fn get_servers_for_language(
        &self,
        language: &LanguageServerKind,
    ) -> Vec<LanguageServerHandle>;
}

#[async_trait::async_trait]
impl LanguageServerPoolRouter for LanguageServerPool {
    async fn get_servers_for_language(
        &self,
        language: &LanguageServerKind,
    ) -> Vec<LanguageServerHandle> {
        self.get_servers_for_language(language)
    }
}

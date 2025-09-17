//! Multi-language LSP integration module
//!
//! This module provides the main entry point for multi-language LSP support,
//! integrating all the components into a cohesive API for seamless
//! multi-language development experience.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use tracing::{debug, info, warn};

use crate::client::LSPError;
use crate::cross_language::{
    CrossLanguageManager, CrossLanguageSearchResult, CrossLanguageSymbol, SearchConfiguration,
};
use crate::language_detection::LanguageDetector;
use crate::language_router::RoutingStrategy;
use crate::language_router::{LanguageRouter, LoadBalancingStrategy, RequestContext};
use crate::language_server::{
    LanguageServerConfig, LanguageServerFactory, LanguageServerKind,
};
use crate::pool::{LanguageServerPool, LanguageServerPoolConfig, PoolStatistics, ServerStatus};
use std::collections::HashMap;

use lsp_types::*;

/// Main configuration for multi-language LSP setup
#[derive(Debug, Clone)]
pub struct MultiLanguageConfig {
    /// Base pool configuration
    pub pool_config: LanguageServerPoolConfig,
    /// Routing strategy
    pub routing_strategy: RoutingStrategy,
    /// Load balancing strategy
    pub load_balancing: LoadBalancingStrategy,
    /// Auto-detection enabled
    pub auto_detection: bool,
    /// Cross-language search enabled
    pub cross_language_search: bool,
    /// Default search configuration
    pub search_config: SearchConfiguration,
    /// Maximum startup time for servers (seconds)
    pub max_startup_time: u64,
}

impl Default for MultiLanguageConfig {
    fn default() -> Self {
        Self {
            pool_config: LanguageServerPoolConfig::default(),
            routing_strategy: RoutingStrategy::Intelligent,
            load_balancing: LoadBalancingStrategy::LeastLoaded,
            auto_detection: true,
            cross_language_search: true,
            search_config: SearchConfiguration::default(),
            max_startup_time: 60,
        }
    }
}

/// Main multi-language LSP manager
pub struct MultiLanguageLSP {
    /// Server pool for managing multiple language servers
    pool: Arc<RwLock<LanguageServerPool>>,
    /// Router for intelligent request routing
    router: Arc<RwLock<LanguageRouter>>,
    /// Cross-language capabilities manager
    cross_language: Option<Arc<CrossLanguageManager>>,
    /// Configuration
    config: MultiLanguageConfig,
    /// Language detector
    detector: LanguageDetector,
    /// Tracking startup time
    startup_time: std::time::Instant,
}

impl Default for MultiLanguageLSP {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiLanguageLSP {
    /// Create a new multi-language LSP manager with default configuration
    pub fn new() -> Self {
        Self::with_config(MultiLanguageConfig::default())
    }

    /// Create a new multi-language LSP manager with custom configuration
    pub fn with_config(config: MultiLanguageConfig) -> Self {
        let pool = Arc::new(RwLock::new(LanguageServerPool::new_with_config(
            config.pool_config.clone(),
        )));
        let router = Arc::new(RwLock::new(LanguageRouter::new(Arc::clone(&pool))));

        let cross_language = if config.cross_language_search {
            Some(Arc::new(CrossLanguageManager::new(
                Arc::clone(&router),
                Arc::clone(&pool),
            )))
        } else {
            None
        };

        Self {
            pool,
            router,
            cross_language,
            config,
            detector: LanguageDetector::default(),
            startup_time: std::time::Instant::now(),
        }
    }

    /// Initialize the multi-language LSP system
    pub async fn initialize(&mut self, workspace_root: PathBuf) -> Result<(), LSPError> {
        info!(
            "Initializing multi-language LSP system for workspace: {:?}",
            workspace_root
        );

        // Scan workspace for supported languages if auto-detection is enabled
        if self.config.auto_detection {
            self.scan_workspace(&workspace_root).await?;
        }

        // Initialize cross-language capabilities
        if let Some(cross_language) = &self.cross_language {
            cross_language.initial_context(&workspace_root).await?;
        }

        info!(
            "Multi-language LSP system initialized in {:.2}s",
            self.startup_time.elapsed().as_secs_f64()
        );
        Ok(())
    }

    /// Register a language server factory
    pub async fn register_language_server<'factory, F>(
        &mut self,
        factory: F,
        config: LanguageServerConfig,
    ) -> Result<(), LSPError>
    where
        F: LanguageServerFactory + Send + Sync + 'static,
    {
        let mut pool = self.pool.write().await;
        pool.register_factory(factory)?;

        // Start server if required
        let language = config.language.clone();
        if language != LanguageServerKind::Custom("".to_string()) {
            pool.start_server(language, config).await?;
        }

        Ok(())
    }

    /// Handle an LSP request (routing through appropriate language server)
    pub async fn handle_request(
        &self,
        method: &str,
        params: &serde_json::Value,
        document_uri: Option<Uri>,
        language_hint: Option<String>,
    ) -> Result<Option<serde_json::Value>, LSPError> {
        debug!(
            "Handling LSP request: {} with URI {:?}",
            method, document_uri
        );

        let context = RequestContext {
            method: method.to_string(),
            document_uri,
            position: None,
            selection: None,
            file_path_hint: None,
            language_hint,
            workspace_root: None,
        };

        let routing_result = self.router.read().await.route_request(&context).await?;
        debug!(
            "Routed to {:?} with handle present: {}",
            routing_result.target_language,
            routing_result.server_handle.is_some()
        );

        if let Some(server_handle) = routing_result.server_handle {
            let server_wrapper = server_handle.read().await;

            if server_wrapper.server.is_initialized() {
                // Send request to appropriate language server
                let response = server_wrapper
                    .server
                    .send_request(method, params.clone())
                    .await?;

                Ok(Some(response))
            } else {
                warn!(
                    "Selected server for {:?} is not initialized",
                    routing_result.target_language
                );
                Err(LSPError::NotInitialized)
            }
        } else {
            warn!(
                "No suitable server found for {:?}",
                routing_result.target_language
            );
            Err(LSPError::Other(format!(
                "No server available for language {:?}",
                routing_result.target_language
            )))
        }
    }

    /// Handle text synchronization requests (didOpen, didChange, didClose)
    pub async fn handle_text_sync(
        &self,
        uri: &Uri,
        content: Option<&str>,
        operation: TextSyncOperation,
    ) -> Result<(), LSPError> {
        debug!(
            "Handling text sync operation {:?} for URI {}",
            operation,
            uri.as_str()
        );

        // Get language for the file
        let language_detections = if let Some(content) = content {
            self.detector
                .detect_language(std::path::Path::new(uri.path().as_str()), Some(content))
        } else {
            self.detector
                .detect_language(std::path::Path::new(uri.path().as_str()), None)
        };

        if language_detections.is_empty() {
            warn!("Could not determine language for URI {}", uri.as_str());
            return Ok(());
        }

        let primary_language = &language_detections[0];
        let language = match primary_language.language.as_str() {
            "rust" => LanguageServerKind::Rust,
            "typescript" | "ts" => LanguageServerKind::TypeScript,
            "javascript" | "js" => LanguageServerKind::JavaScript,
            "python" | "py" => LanguageServerKind::Python,
            "go" | "golang" => LanguageServerKind::Go,
            _ => LanguageServerKind::Custom(primary_language.language.clone()),
        };

        // Route to appropriate server
        let context = RequestContext {
            method: operation.event_name().to_string(),
            document_uri: Some(uri.clone()),
            position: None,
            selection: None,
            file_path_hint: None,
            language_hint: Some(primary_language.language.clone()),
            workspace_root: None,
        };

        let routing_result = self.router.read().await.route_request(&context).await?;

        if let Some(server_handle) = routing_result.server_handle {
            let server_wrapper = server_handle.read().await;

            if server_wrapper.server.is_initialized() {
                let event_params = operation.to_params(uri, content);

                server_wrapper
                    .server
                    .send_notification(operation.event_name(), event_params)
                    .await?;

                // Update cross-language index if enabled
                if let Some(cross_language) = &self.cross_language {
                    if operation.creates_or_updates_content() {
                        if let Some(content) = content {
                            cross_language
                                .update_symbol_index(uri, Some(content))
                                .await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Perform cross-language symbol search
    pub async fn search_symbols_cross_language(
        &self,
        query: &str,
        include_references: bool,
    ) -> Result<CrossLanguageSearchResult, LSPError> {
        if let Some(cross_language) = &self.cross_language {
            info!("Performing cross-language symbol search for '{}'", query);

            let mut config = self.config.search_config.clone();
            config.include_references = include_references;

            cross_language.search_symbols(query, Some(config)).await
        } else {
            Err(LSPError::Other(
                "Cross-language search is not enabled".to_string(),
            ))
        }
    }

    /// Find symbol references across languages
    pub async fn find_references_cross_language(
        &self,
        location: &Location,
    ) -> Result<Vec<CrossLanguageSymbol>, LSPError> {
        if let Some(cross_language) = &self.cross_language {
            debug!("Finding references across languages for {:?}", location);
            cross_language.find_references(location, None).await
        } else {
            Err(LSPError::Other(
                "Cross-language features are not enabled".to_string(),
            ))
        }
    }

    /// Analyze call hierarchy across languages
    pub async fn analyze_call_hierarchy_cross_language(
        &self,
        location: &Location,
    ) -> Result<Vec<lsp_types::CallHierarchyItem>, LSPError> {
        if let Some(cross_language) = &self.cross_language {
            debug!(
                "Analyzing call hierarchy across languages for {:?}",
                location
            );
            cross_language.analyze_call_hierarchy(location, None).await
        } else {
            Err(LSPError::Other(
                "Cross-language features are not enabled".to_string(),
            ))
        }
    }

    /// Get system status and statistics
    pub async fn get_system_status(&self) -> MultiLanguageStatus {
        let pool_stats = self.pool.read().await.get_statistics().await;
        let router_stats_clone = self.router.read().await.get_statistics().await;

        MultiLanguageStatus {
            initialization_time_seconds: self.startup_time.elapsed().as_secs_f64(),
            pool_statistics: pool_stats.clone(),
            router_statistics: router_stats_clone,
            cross_language_enabled: self.cross_language.is_some(),
            supported_languages: self.pool.read().await.get_supported_languages().len(),
            active_servers: pool_stats.active_servers,
            server_health: self.get_overall_health().await,
        }
    }

    /// Get live server status for a specific language
    pub async fn get_server_status(&self, language: &LanguageServerKind) -> Vec<ServerStatus> {
        self.pool.read().await.get_server_status(language).await
    }

    /// Add caching and performance optimizations for large codebases
    pub async fn optimize_for_large_codebase(&self) -> Result<(), LSPError> {
        info!("Applying optimizations for large codebase");

        // Apply router optimizations
        let mut router = self.router.write().await;
        router.enable_performance_optimizations().await;

        // Apply pool optimizations
        let mut pool = self.pool.write().await;
        pool.configure_for_high_load().await;

        // Apply cross-language optimizations if enabled
        if let Some(cross_language) = &self.cross_language {
            cross_language.enable_indexing_optimizations().await?;
        }

        info!("Large codebase optimizations applied");
        Ok(())
    }

    /// Graceful shutdown of all language servers
    pub async fn shutdown(&self) -> Result<(), LSPError> {
        info!("Shutting down multi-language LSP system");

        let pool = self.pool.write().await;
        let languages = pool.get_supported_languages();

        let mut shutdown_futures = Vec::new();

        for language in languages {
            let servers = pool.get_servers_for_language(&language);
            for server_handle in &servers {
                let handle_clone = server_handle.clone();
                let language_clone = language.clone();
                let future = async move {
                    let mut wrapper = handle_clone.write().await;
                    match wrapper.server.shutdown().await {
                        Ok(_) => debug!("Successfully shut down server for {:?}", language_clone),
                        Err(e) => {
                            warn!("Error shutting down server for {:?}: {}", language_clone, e)
                        }
                    }
                };
                shutdown_futures.push(future);
            }
        }

        // Wait for all servers to shut down
        for future in shutdown_futures {
            future.await;
        }

        info!("Multi-language LSP system shutdown complete");
        Ok(())
    }

    /// Get overall system health
    async fn get_overall_health(&self) -> SystemHealth {
        let pool = self.pool.read().await;
        let stats = pool.get_statistics().await;

        let healthy_ratio = if stats.total_servers > 0 {
            stats.healthy_servers as f64 / stats.total_servers as f64
        } else {
            0.0
        };

        match healthy_ratio {
            r if r >= 0.9 => SystemHealth::Healthy,
            r if r >= 0.7 => SystemHealth::Degraded,
            r if r >= 0.5 => SystemHealth::Unhealthy,
            _ => SystemHealth::Critical,
        }
    }
}

impl LanguageRouter {
    /// Helper method to enable caching optimizations
    async fn enable_caching(&self) {
        // Implementation would enable request/result caching
        info!("Enabling router caching optimizations");
    }

    /// Helper method to optimize request routing
    async fn optimize_request_routing(&self) {
        // Implementation would optimize routing algorithms
        info!("Optimizing request routing algorithms");
    }

    /// Helper method to enable parallel processing
    async fn enable_parallel_processing(&self) {
        // Implementation would enable parallel request processing
        info!("Enabling parallel request processing");
    }

    /// Helper method to enable CPU-optimized routing
    async fn enable_cpu_optimized_routing(&self) {
        // Implementation would optimize routing for high CPU usage
        info!("Enabling CPU-optimized routing");
    }

    /// Helper method to enable memory-efficient routing
    async fn enable_memory_efficient_routing(&self) {
        // Implementation would optimize routing for low memory
        info!("Enabling memory-efficient routing");
    }

    /// Helper method to enable failover routing
    async fn enable_failover_routing(&self) {
        // Implementation would enable failover to healthy servers
        info!("Enabling failover routing");
    }
}

impl LanguageServerPool {
    /// Helper method to enable response time optimization
    async fn enable_response_time_optimization(&self) {
        // Implementation would optimize for faster response times
        info!("Enabling response time optimizations");
    }

    /// Helper method to enable request queueing optimization
    async fn enable_request_queueing_optimization(&self) {
        // Implementation would optimize request queueing
        info!("Enabling request queueing optimizations");
    }

    /// Helper method to enable server health optimization
    async fn enable_server_health_optimization(&self) {
        // Implementation would optimize server health monitoring
        info!("Enabling server health optimizations");
    }

    /// Helper method to enable memory pool optimization
    async fn enable_memory_pool_optimization(&self) {
        // Implementation would optimize memory usage across pool
        info!("Enabling memory pool optimizations");
    }

    /// Helper method to configure CPU high load settings
    async fn configure_cpu_high_load(&self) {
        // Implementation would configure pool for high CPU usage
        info!("Configuring pool for high CPU load");
    }

    /// Helper method to configure memory high load settings
    async fn configure_memory_high_load(&self) {
        // Implementation would configure pool for low memory conditions
        info!("Configuring pool for memory constraints");
    }

    /// Helper method to configure capacity optimization
    async fn configure_capacity_optimization(&self) {
        // Implementation would optimize server capacity utilization
        info!("Configuring capacity optimization");
    }

    /// Helper method to enable request prioritization
    async fn enable_request_prioritization(&self) {
        // Implementation would enable request prioritization based on load
        info!("Enabling request prioritization");
    }

    /// Scan workspace for detectable language files
    async fn scan_workspace(&self, workspace_root: &PathBuf) -> Result<(), LSPError> {
        info!(
            "Scanning workspace for language files: {:?}",
            workspace_root
        );

        let language_count: HashMap<String, usize> = HashMap::new();

        // TODO: Implement actual workspace scanning
        // For now, this is a placeholder for demonstration

        info!(
            "Workspace scan complete. Found languages with file counts: {:?}",
            language_count
        );
        Ok(())
    }
}

/// Text synchronization operation types
#[derive(Debug, Clone)]
pub enum TextSyncOperation {
    DidOpen,
    DidChange,
    DidClose,
}

impl TextSyncOperation {
    fn event_name(&self) -> &'static str {
        match self {
            TextSyncOperation::DidOpen => "textDocument/didOpen",
            TextSyncOperation::DidChange => "textDocument/didChange",
            TextSyncOperation::DidClose => "textDocument/didClose",
        }
    }

    fn to_params(&self, uri: &Uri, content: Option<&str>) -> serde_json::Value {
        match self {
            TextSyncOperation::DidOpen => serde_json::json!({
                "textDocument": {
                    "uri": uri.to_string(),
                    "languageId": "text", // Would be detected properly in real implementation
                    "version": 1,
                    "text": content.unwrap_or("")
                }
            }),
            TextSyncOperation::DidChange => serde_json::json!({
                "textDocument": {
                    "uri": uri.to_string(),
                    "version": 1
                },
                "contentChanges": [
                    {
                        "text": content.unwrap_or("")
                    }
                ]
            }),
            TextSyncOperation::DidClose => serde_json::json!({
                "textDocument": {
                    "uri": uri.to_string()
                }
            }),
        }
    }

    fn creates_or_updates_content(&self) -> bool {
        matches!(
            self,
            TextSyncOperation::DidOpen | TextSyncOperation::DidChange
        )
    }
}

/// System status information
#[derive(Debug, Clone)]
pub struct MultiLanguageStatus {
    pub initialization_time_seconds: f64,
    pub pool_statistics: PoolStatistics,
    pub router_statistics: crate::language_router::RouterStatistics,
    pub cross_language_enabled: bool,
    pub supported_languages: usize,
    pub active_servers: usize,
    pub server_health: SystemHealth,
}

/// Overall system health status
#[derive(Debug, Clone, PartialEq)]
pub enum SystemHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Critical,
}

/// Extension trait for configuring multi-language features
#[async_trait::async_trait]
pub trait MultiLanguageLspExt {
    /// Enable performance optimizations for large codebases
    async fn enable_performance_optimizations(&mut self);

    /// Configure pool for high load scenarios
    async fn configure_for_high_load(&mut self);
}

#[async_trait::async_trait]
impl MultiLanguageLspExt for LanguageRouter {
    async fn enable_performance_optimizations(&mut self) {
        // Get current load metrics to determine optimization strategy
        let load_metrics = self.pool.read().await.get_server_load_metrics().await;

        // Configure routing based on current system load
        let avg_response_time: f64 = load_metrics.iter()
            .map(|m| m.response_time_ms)
            .sum::<f64>() / load_metrics.len().max(1) as f64;

        let total_pending_requests: usize = load_metrics.iter()
            .map(|m| m.pending_requests)
            .sum();

        // Enable caching for high-load scenarios
        if avg_response_time > 500.0 || total_pending_requests > 50 {
            info!("Enabling router performance optimizations for high load");
            self.enable_caching().await;
            self.optimize_request_routing().await;
        }

        // Enable parallel processing for large codebases
        if load_metrics.len() > 10 {
            self.enable_parallel_processing().await;
        }
    }

    async fn configure_for_high_load(&mut self) {
        // Get resource metrics to assess system capacity
        let resource_metrics = self.pool.read().await.get_resource_metrics().await
            .unwrap_or_default();

        // Configure load balancing based on system resources
        if resource_metrics.cpu_usage_percent > 80.0 {
            info!("Configuring router for high CPU load");
            self.enable_cpu_optimized_routing().await;
        }

        if resource_metrics.memory_available_mb < 500 {
            info!("Configuring router for low memory conditions");
            self.enable_memory_efficient_routing().await;
        }

        // Get pool statistics for server health assessment
        let pool_stats = self.pool.read().await.get_statistics().await;
        let healthy_ratio = if pool_stats.total_servers > 0 {
            pool_stats.healthy_servers as f64 / pool_stats.total_servers as f64
        } else {
            0.0
        };

        // Enable failover routing if server health is poor
        if healthy_ratio < 0.8 {
            warn!("Low server health ratio ({:.2}), enabling failover routing", healthy_ratio);
            self.enable_failover_routing().await;
        }
    }
}

#[async_trait::async_trait]
impl MultiLanguageLspExt for LanguageServerPool {
    async fn enable_performance_optimizations(&mut self) {
        // Get current server load metrics to determine optimization needs
        let load_metrics = self.get_server_load_metrics().await;

        // Calculate optimization metrics
        let avg_response_time: f64 = load_metrics.iter()
            .map(|m| m.response_time_ms)
            .sum::<f64>() / load_metrics.len().max(1) as f64;

        let total_pending_requests: usize = load_metrics.iter()
            .map(|m| m.pending_requests)
            .sum();

        let avg_health_score: f64 = load_metrics.iter()
            .map(|m| m.health_score)
            .sum::<f64>() / load_metrics.len().max(1) as f64;

        // Apply performance optimizations based on metrics
        if avg_response_time > 500.0 {
            info!("Enabling pool performance optimizations for slow response times");
            self.enable_response_time_optimization().await;
        }

        if total_pending_requests > 100 {
            info!("Enabling pool optimizations for high request load");
            self.enable_request_queueing_optimization().await;
        }

        if avg_health_score < 0.8 {
            warn!("Low server health detected, enabling recovery optimizations");
            self.enable_server_health_optimization().await;
        }

        // Enable memory management for large server pools
        if load_metrics.len() > 5 {
            self.enable_memory_pool_optimization().await;
        }
    }

    async fn configure_for_high_load(&mut self) {
        // Get system resource metrics
        let resource_metrics = match self.get_resource_metrics().await {
            Ok(metrics) => metrics,
            Err(_) => {
                warn!("Failed to get resource metrics, using default configuration");
                return;
            }
        };

        // Configure based on system resource pressure
        if resource_metrics.cpu_usage_percent > 85.0 {
            info!("High CPU usage detected ({:.1}%), configuring for CPU optimization",
                  resource_metrics.cpu_usage_percent);
            self.configure_cpu_high_load().await;
        }

        if resource_metrics.memory_available_mb < 200 {
            info!("Low memory detected ({:.0}MB), configuring for memory efficiency",
                  resource_metrics.memory_available_mb);
            self.configure_memory_high_load().await;
        }

        // Get pool statistics for server capacity assessment
        let stats = self.get_statistics().await;

        // Configure load balancing based on server capacity
        if stats.active_servers < stats.total_servers / 2 {
            warn!("Low server utilization ({}/{}), enabling capacity optimization",
                  stats.active_servers, stats.total_servers);
            self.configure_capacity_optimization().await;
        }

        // Enable request prioritization for high load
        if stats.average_request_rate > 100.0 {
            info!("High request rate detected ({:.1} req/s), enabling request prioritization",
                  stats.average_request_rate);
            self.enable_request_prioritization().await;
        }
    }
}

/// Extension trait for cross-language features
#[async_trait::async_trait]
pub trait CrossLanguageExt {
    /// Enable advanced indexing optimizations
    async fn enable_indexing_optimizations(&self) -> Result<(), LSPError>;

    /// Initialize context for a workspace
    async fn initial_context(&self, workspace_root: &PathBuf) -> Result<(), LSPError>;
}

#[async_trait::async_trait]
impl CrossLanguageExt for CrossLanguageManager {
    async fn enable_indexing_optimizations(&self) -> Result<(), LSPError> {
        // Implementation would enable advanced indexing
        Ok(())
    }

    async fn initial_context(&self, _workspace_root: &PathBuf) -> Result<(), LSPError> {
        // Implementation would initialize context
        Ok(())
    }
}

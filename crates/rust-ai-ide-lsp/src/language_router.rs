//! Language request router for multi-language LSP support
//!
//! This module provides intelligent routing of LSP requests to the appropriate
//! language servers based on file types, content, and user context.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use rand::Rng;

#[cfg(feature = "multi-language-lsp")]
use tree_sitter::*;

use lsp_types::*;
use tracing::{debug, info, warn};

use crate::client::LSPError;
use crate::language_detection::LanguageDetector;
use crate::language_server::{LanguageServerHandle, LanguageServerKind, ServerHealth, ServerMetrics};
use crate::pool::LanguageServerPool;
use crate::{project::ProjectManager, utils};

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

/// Server load information for load balancing
#[derive(Debug, Clone)]
pub struct ServerLoadInfo {
    /// Server handle identifier
    pub handle_id: String,
    /// Current request rate (requests per second)
    pub request_rate: f64,
    /// Average response time (milliseconds)
    pub avg_response_time_ms: f64,
    /// Memory usage (MB)
    pub memory_usage_mb: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Number of active connections
    pub active_connections: usize,
    /// Server health status
    pub health: ServerHealth,
    /// Last updated timestamp
    pub last_updated: std::time::Instant,
}

impl ServerLoadInfo {
    /// Calculate load score (lower is better)
    pub fn load_score(&self) -> f64 {
        // Weighted score combining multiple metrics
        let request_weight = 0.3;
        let response_time_weight = 0.3;
        let memory_weight = 0.2;
        let cpu_weight = 0.2;

        // Normalize metrics (higher values are worse)
        let normalized_request_rate = self.request_rate / 50.0; // Assume 50 req/sec is high
        let normalized_response_time = self.avg_response_time_ms / 1000.0; // Assume 1s is slow
        let normalized_memory = self.memory_usage_mb / 1000.0; // Assume 1GB is high
        let normalized_cpu = self.cpu_usage_percent / 100.0;

        request_weight * normalized_request_rate +
        response_time_weight * normalized_response_time +
        memory_weight * normalized_memory +
        cpu_weight * normalized_cpu
    }

    /// Check if load info is stale
    pub fn is_stale(&self) -> bool {
        self.last_updated.elapsed().as_secs() > 30 // 30 seconds threshold
    }
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

/// Cached routing result with timestamp for TTL tracking
#[derive(Debug, Clone)]
struct CachedRoutingResult {
    result: RoutingResult,
    inserted_at: std::time::Instant,
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
    /// Project manager for workspace analysis
    project_manager: Arc<RwLock<ProjectManager>>,
    /// Language detector for automatic file type recognition
    detector: LanguageDetector,
    /// Request routing strategy
    routing_strategy: RoutingStrategy,
    /// Load balancing strategy for multiple servers
    load_balancing: LoadBalancingStrategy,
    /// Cache for recent routing decisions
    routing_cache: Arc<RwLock<HashMap<String, CachedRoutingResult>>>,
    /// Round-robin state for load balancing
    round_robin_state: Arc<RwLock<HashMap<LanguageServerKind, usize>>>,
    /// Server load tracking for load balancing
    server_loads: Arc<RwLock<HashMap<String, ServerLoadInfo>>>,
    /// Router statistics
    stats: Arc<RwLock<RouterStatistics>>,
}

impl LanguageRouter {
    /// Create a new language router
    pub fn new(pool: Arc<RwLock<LanguageServerPool>>, project_manager: Arc<RwLock<ProjectManager>>) -> Self {
        Self {
            pool,
            project_manager,
            detector: LanguageDetector::default(),
            routing_strategy: RoutingStrategy::Intelligent,
            load_balancing: LoadBalancingStrategy::LeastLoaded,
            routing_cache: Arc::new(RwLock::new(HashMap::new())),
            round_robin_state: Arc::new(RwLock::new(HashMap::new())),
            server_loads: Arc::new(RwLock::new(HashMap::new())),
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
            if let Some(cached_entry) = self.routing_cache.read().await.get(&uri.to_string()) {
                if cached_entry.inserted_at.elapsed().as_secs() < 300 {
                    // 5 minutes TTL
                    debug!("Using cached routing result for URI {}", uri.as_str());
                    let mut stats = self.stats.write().await;
                    stats.cache_hits += 1;
                    return Ok(cached_entry.result.clone());
                } else {
                    // Entry is stale, remove it
                    let mut cache = self.routing_cache.write().await;
                    cache.remove(&uri.to_string());
                    // Update cache size stats
                    let mut stats = self.stats.write().await;
                    stats.update_cache_size(cache.len());
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
        let (server_handle, candidates_evaluated, selected_index) = self.select_server_instance(&target_language).await?;
        debug!(
            "Server instance selected for language {:?}",
            target_language
        );

        // Update server load tracking if we have a server handle
        if let Some(ref handle) = server_handle {
            if let Ok(handle_ref) = handle.try_read() {
                let handle_id = format!("{}_{}", Self::language_to_string(&target_language), selected_index);
                self.update_server_load(handle_id, &handle_ref.metrics, handle_ref.health_status.clone()).await;
            }
        }

        // Calculate confidence based on routing strategy and factors
        let confidence = self.calculate_routing_confidence(&context, &target_language, candidates_evaluated, selected_index).await;

        // Create routing result
        let routing_result = RoutingResult {
            target_language: target_language.clone(),
            server_handle,
            confidence,
            strategy_used: self.routing_strategy.clone(),
            candidates_evaluated,
            reason: format!(
                "Routed to {:?} using {:?} strategy",
                target_language, self.routing_strategy
            ),
        };

        // Cache the result
        if let Some(uri) = &context.document_uri {
            let mut cache = self.routing_cache.write().await;
            cache.insert(uri.to_string(), CachedRoutingResult {
                result: routing_result.clone(),
                inserted_at: std::time::Instant::now(),
            });
            // Limit cache size
            if cache.len() > 1000 {
                // Simple LRU eviction - in real implementation, use proper LRU
                let keys_to_remove: Vec<String> =
                    cache.keys().skip(cache.len() - 1000).cloned().collect();
                for key in keys_to_remove {
                    cache.remove(&key);
                }
            }
            // Update cache size stats
            let mut stats = self.stats.write().await;
            stats.update_cache_size(cache.len());
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

    /// Route based on content analysis using tree-sitter and regex patterns
    async fn route_by_content(
        &self,
        context: &RequestContext,
    ) -> Result<LanguageServerKind, LSPError> {
        if let Some(uri) = &context.document_uri {
            if let Ok(url) = url::Url::parse(uri.as_str()) {
                if let Ok(path) = url.to_file_path() {
                    #[cfg(feature = "multi-language-lsp")]
                    {
                        if let Ok(language) = self.analyze_file_content_tree_sitter(&path).await {
                            return Ok(language);
                        }
                    }
                    if let Ok(language) = self.analyze_file_content_regex(&path).await {
                        return Ok(language);
                    }
                }
            }
        }

        // If content analysis fails, try file path hint
        if let Some(path_hint) = &context.file_path_hint {
            let path = Path::new(path_hint);
            #[cfg(feature = "multi-language-lsp")]
            {
                if let Ok(language) = self.analyze_file_content_tree_sitter(path).await {
                    return Ok(language);
                }
            }
            if let Ok(language) = self.analyze_file_content_regex(path).await {
                return Ok(language);
            }
        }

        Err(LSPError::Other(
            "Could not determine language from content analysis".to_string(),
        ))
    }

    /// Analyze file content to determine language
    async fn analyze_file_content(&self, path: &Path) -> Result<LanguageServerKind, LSPError> {
        // Read first 10 lines or 1024 bytes, whichever comes first
        let mut file = tokio::fs::File::open(path).await
            .map_err(|e| LSPError::Other(format!("Failed to open file: {}", e)))?;

        let mut buffer = [0; 1024];
        let n = tokio::io::AsyncReadExt::read(&mut file, &mut buffer).await
            .map_err(|e| LSPError::Other(format!("Failed to read file: {}", e)))?;

        let content = String::from_utf8_lossy(&buffer[..n]);
        let lines: Vec<&str> = content.lines().take(10).collect();

        // Analyze content patterns
        let language = self.detect_language_from_content(&lines, &content);
        Ok(language)
    }

    /// Analyze file content using tree-sitter for accurate language detection
    #[cfg(feature = "multi-language-lsp")]
    async fn analyze_file_content_tree_sitter(&self, path: &Path) -> Result<LanguageServerKind, LSPError> {
        // Read file content
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| LSPError::Other(format!("Failed to read file: {}", e)))?;

        // Try tree-sitter parsers in order of likelihood
        if let Ok(language) = self.try_tree_sitter_parser(&content, tree_sitter_rust::language(), "rust") {
            return Ok(language);
        }
        if let Ok(language) = self.try_tree_sitter_parser(&content, tree_sitter_typescript::language_typescript(), "typescript") {
            return Ok(language);
        }
        if let Ok(language) = self.try_tree_sitter_parser(&content, tree_sitter_javascript::language(), "javascript") {
            return Ok(language);
        }
        if let Ok(language) = self.try_tree_sitter_parser(&content, tree_sitter_python::language(), "python") {
            return Ok(language);
        }
        if let Ok(language) = self.try_tree_sitter_parser(&content, tree_sitter_go::language(), "go") {
            return Ok(language);
        }

        Err(LSPError::Other("Tree-sitter parsing failed".to_string()))
    }

    /// Fallback regex-based content analysis
    async fn analyze_file_content_regex(&self, path: &Path) -> Result<LanguageServerKind, LSPError> {
        // Read first 10 lines or 1024 bytes, whichever comes first
        let mut file = tokio::fs::File::open(path).await
            .map_err(|e| LSPError::Other(format!("Failed to open file: {}", e)))?;

        let mut buffer = [0; 1024];
        let n = tokio::io::AsyncReadExt::read(&mut file, &mut buffer).await
            .map_err(|e| LSPError::Other(format!("Failed to read file: {}", e)))?;

        let content = String::from_utf8_lossy(&buffer[..n]);
        let lines: Vec<&str> = content.lines().take(10).collect();

        // Analyze content patterns
        let language = self.detect_language_from_content(&lines, &content);
        Ok(language)
    }

    /// Try to parse content with a specific tree-sitter parser
    #[cfg(feature = "multi-language-lsp")]
    fn try_tree_sitter_parser(&self, content: &str, language: tree_sitter::Language, lang_name: &str) -> Result<LanguageServerKind, LSPError> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(language)
            .map_err(|e| LSPError::Other(format!("Failed to set tree-sitter language: {}", e)))?;

        let tree = parser.parse(content, None)
            .ok_or_else(|| LSPError::Other("Tree-sitter parsing failed".to_string()))?;

        // Check if parsing was successful (has a root node)
        if tree.root_node().has_error() {
            return Err(LSPError::Other("Tree-sitter parsing had errors".to_string()));
        }

        // Convert language name to LanguageServerKind
        let kind = match lang_name {
            "rust" => LanguageServerKind::Rust,
            "typescript" => LanguageServerKind::TypeScript,
            "javascript" => LanguageServerKind::JavaScript,
            "python" => LanguageServerKind::Python,
            "go" => LanguageServerKind::Go,
            _ => LanguageServerKind::Custom(lang_name.to_string()),
        };

        Ok(kind)
    }

    /// Detect language from file content
    fn detect_language_from_content(&self, lines: &[&str], content: &str) -> LanguageServerKind {
        // Rust detection
        if lines.iter().any(|line| line.contains("fn ") || line.contains("let ") || line.contains("use "))
            && lines.iter().any(|line| line.contains("mod ") || line.contains("struct ") || line.contains("impl ")) {
            return LanguageServerKind::Rust;
        }

        // TypeScript/JavaScript detection
        if lines.iter().any(|line| line.contains("import ") || line.contains("export "))
            || content.contains("function ") || content.contains("const ") || content.contains("let ") {
            if lines.iter().any(|line| line.contains(": ") || line.contains("interface ") || line.contains("<")) {
                return LanguageServerKind::TypeScript;
            }
            return LanguageServerKind::JavaScript;
        }

        // HTML detection
        if lines.iter().any(|line| line.contains("<html") || line.contains("<div") || line.contains("</")) {
            return LanguageServerKind::Html;
        }

        // CSS detection
        if lines.iter().any(|line| line.contains("{") && line.contains(":")) {
            return LanguageServerKind::Css;
        }

        // SQL detection
        if content.to_lowercase().contains("select ") || content.to_lowercase().contains("insert ")
            || content.to_lowercase().contains("create table") {
            return LanguageServerKind::Sql;
        }

        // Go detection
        if lines.iter().any(|line| line.contains("package ") || line.contains("func ") || line.contains("import (")) {
            return LanguageServerKind::Go;
        }

        // Python detection
        if lines.iter().any(|line| line.contains("import ") || line.contains("def ") || line.contains("class "))
            && !lines.iter().any(|line| line.contains("fn ") || line.contains("let ")) {
            return LanguageServerKind::Python;
        }

        // Default fallback
        LanguageServerKind::Custom("unknown".to_string())
    }

    /// Route based on workspace configuration (reads .vscode/settings.json, rust-project.json)
    async fn route_by_workspace_config(
        &self,
        context: &RequestContext,
    ) -> Result<LanguageServerKind, LSPError> {
        if let Some(workspace_root) = &context.workspace_root {
            // First try reading workspace configuration files
            if let Ok(language) = self.read_workspace_config_files(workspace_root).await {
                return Ok(language);
            }

            // Fall back to project manager analysis
            let project_manager = self.project_manager.read().await;
            if let Some(project) = project_manager.find_project(workspace_root) {
                if let Some(metadata) = &project.metadata {
                    // Analyze dependencies to determine primary language
                    let language = self.analyze_project_dependencies(metadata);
                    if language != LanguageServerKind::Custom("unknown".to_string()) {
                        return Ok(language);
                    }
                }
            }

            // Analyze project structure
            let language = self.analyze_project_structure(workspace_root).await;
            if language != LanguageServerKind::Custom("unknown".to_string()) {
                return Ok(language);
            }
        }

        Err(LSPError::Other(
            "Could not determine language from workspace configuration".to_string(),
        ))
    }

    /// Read workspace configuration files (.vscode/settings.json, rust-project.json)
    async fn read_workspace_config_files(&self, workspace_root: &str) -> Result<LanguageServerKind, LSPError> {
        let root_path = Path::new(workspace_root);

        // Check .vscode/settings.json for language configuration
        let vscode_settings = root_path.join(".vscode").join("settings.json");
        if let Ok(content) = tokio::fs::read_to_string(&vscode_settings).await {
            if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
                // Check for language-specific settings
                if let Some(lang_id) = settings.get("lsp.languageId").and_then(|v| v.as_str()) {
                    return Ok(self.language_id_to_language(lang_id));
                }
                if let Some(extensions) = settings.get("files.associations").and_then(|v| v.as_object()) {
                    if let Some(first_ext) = extensions.keys().next() {
                        return Ok(self.extension_to_language(first_ext.trim_start_matches("*."));
                    }
                }
            }
        }

        // Check rust-project.json for Rust-specific configuration
        let rust_project = root_path.join("rust-project.json");
        if let Ok(content) = tokio::fs::read_to_string(&rust_project).await {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                // Check for Rust project markers
                if config.get("sysroot").is_some() || config.get("crates").is_some() {
                    return Ok(LanguageServerKind::Rust);
                }
            }
        }

        Err(LSPError::Other("No workspace configuration found".to_string()))
    }

    /// Convert language ID string to LanguageServerKind
    fn language_id_to_language(&self, lang_id: &str) -> LanguageServerKind {
        match lang_id.to_lowercase().as_str() {
            "rust" => LanguageServerKind::Rust,
            "typescript" | "typescriptreact" => LanguageServerKind::TypeScript,
            "javascript" | "javascriptreact" => LanguageServerKind::JavaScript,
            "html" => LanguageServerKind::Html,
            "css" | "scss" | "sass" | "less" => LanguageServerKind::Css,
            "sql" => LanguageServerKind::Sql,
            "go" | "golang" => LanguageServerKind::Go,
            "python" => LanguageServerKind::Python,
            _ => LanguageServerKind::Custom(lang_id.to_string()),
        }
    }

    /// Analyze project dependencies to determine language
    fn analyze_project_dependencies(&self, metadata: &crate::project::ProjectMetadata) -> LanguageServerKind {
        // Check for Rust-specific dependencies
        if metadata.dependencies.contains_key("tokio") ||
           metadata.dependencies.contains_key("serde") ||
           metadata.dependencies.contains_key("futures") {
            return LanguageServerKind::Rust;
        }

        // Check for JavaScript/TypeScript framework dependencies
        if metadata.dependencies.contains_key("react") ||
           metadata.dependencies.contains_key("vue") ||
           metadata.dependencies.contains_key("angular") {
            return LanguageServerKind::TypeScript;
        }

        // Check for Python dependencies
        if metadata.dependencies.contains_key("django") ||
           metadata.dependencies.contains_key("flask") ||
           metadata.dependencies.contains_key("requests") {
            return LanguageServerKind::Python;
        }

        // Check for Go dependencies (less common in Cargo.toml)
        if metadata.name.contains("go") || metadata.description.as_ref().map_or(false, |d| d.contains("go")) {
            return LanguageServerKind::Go;
        }

        LanguageServerKind::Custom("unknown".to_string())
    }

    /// Analyze project structure to determine language
    async fn analyze_project_structure(&self, workspace_root: &str) -> LanguageServerKind {
        let root_path = Path::new(workspace_root);

        // Check for common project structure indicators
        let indicators = [
            // Rust
            ("Cargo.toml", LanguageServerKind::Rust),
            ("src/main.rs", LanguageServerKind::Rust),
            ("src/lib.rs", LanguageServerKind::Rust),

            // JavaScript/TypeScript
            ("package.json", LanguageServerKind::JavaScript),
            ("tsconfig.json", LanguageServerKind::TypeScript),
            ("node_modules", LanguageServerKind::JavaScript),
            ("yarn.lock", LanguageServerKind::JavaScript),
            ("package-lock.json", LanguageServerKind::JavaScript),

            // Python
            ("requirements.txt", LanguageServerKind::Python),
            ("setup.py", LanguageServerKind::Python),
            ("pyproject.toml", LanguageServerKind::Python),

            // Go
            ("go.mod", LanguageServerKind::Go),
            ("go.sum", LanguageServerKind::Go),

            // HTML/CSS
            ("index.html", LanguageServerKind::Html),

            // SQL
            ("schema.sql", LanguageServerKind::Sql),
        ];

        for (file_name, language) in &indicators {
            if tokio::fs::metadata(root_path.join(file_name)).await.is_ok() {
                return language.clone();
            }
        }

        // Check for file patterns in subdirectories
        if let Ok(mut entries) = tokio::fs::read_dir(root_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(file_type) = entry.file_type().await {
                    if file_type.is_dir() {
                        let dir_name = entry.file_name().to_string_lossy();
                        match dir_name.as_ref() {
                            "src" => {
                                // Check if src contains .rs files (Rust)
                                if self.has_files_with_extension(&entry.path(), "rs").await {
                                    return LanguageServerKind::Rust;
                                }
                                // Check if src contains .js/.ts files
                                if self.has_files_with_extension(&entry.path(), "js").await ||
                                   self.has_files_with_extension(&entry.path(), "ts").await {
                                    return LanguageServerKind::JavaScript;
                                }
                            }
                            "lib" => {
                                if self.has_files_with_extension(&entry.path(), "rs").await {
                                    return LanguageServerKind::Rust;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        LanguageServerKind::Custom("unknown".to_string())
    }

    /// Check if directory contains files with specific extension
    async fn has_files_with_extension(&self, dir_path: &Path, extension: &str) -> bool {
        if let Ok(mut entries) = tokio::fs::read_dir(dir_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(file_type) = entry.file_type().await {
                    if file_type.is_file() {
                        if let Some(file_ext) = entry.path().extension() {
                            if file_ext == extension {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
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
    ) -> Result<(Option<LanguageServerHandle>, usize, usize), LSPError> {
        let pool = self.pool.read().await;
        let servers = pool.get_servers_for_language(language);

        if servers.is_empty() {
            warn!("No servers available for language {:?}", language);
            return Ok((None, 0));
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
            return Ok((None, servers.len()));
        }

        let selected_index = self
            .select_server_by_strategy(available_servers.len(), language)
            .await;

        Ok((Some(available_servers[selected_index].clone()), available_servers.len(), selected_index))
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
                self.select_least_loaded_server(available_count, language).await
            }
            LoadBalancingStrategy::Random => rand::thread_rng().gen_range(0..available_count),
            LoadBalancingStrategy::HealthBased => {
                // All servers at this point are already healthy, so use least loaded
                self.select_least_loaded_server(available_count, language).await
            }
        }
    }

    /// Select the least loaded server
    async fn select_least_loaded_server(&self, available_count: usize, language: &LanguageServerKind) -> usize {
        let server_loads = self.server_loads.read().await;

        // Find the server with the lowest load score
        let mut best_index = 0;
        let mut best_score = f64::INFINITY;

        for i in 0..available_count {
            let handle_id = format!("{}_{}", Self::language_to_string(language), i);
            let load_score = server_loads.get(&handle_id)
                .map(|load_info| load_info.load_score())
                .unwrap_or(0.5); // Default moderate load if no info

            if load_score < best_score {
                best_score = load_score;
                best_index = i;
            }
        }

        best_index
    }

    /// Update server load information with actual pool metrics
    pub async fn update_server_load(&self, handle_id: String, metrics: &ServerMetrics, health: ServerHealth) {
        let mut server_loads = self.server_loads.write().await;

        // Get actual system resource metrics from pool's resource monitor
        let (memory_mb, cpu_percent) = self.get_actual_resource_usage().await;

        let load_info = server_loads.entry(handle_id.clone()).or_insert_with(|| ServerLoadInfo {
            handle_id: handle_id.clone(),
            request_rate: metrics.requests_per_second,
            avg_response_time_ms: metrics.average_response_time_ms,
            memory_usage_mb: memory_mb,
            cpu_usage_percent: cpu_percent,
            active_connections: 1,
            health,
            last_updated: std::time::Instant::now(),
        });

        // Update metrics
        load_info.request_rate = metrics.requests_per_second;
        load_info.avg_response_time_ms = metrics.average_response_time_ms;
        load_info.memory_usage_mb = memory_mb;
        load_info.cpu_usage_percent = cpu_percent;
        load_info.health = health;
        load_info.last_updated = std::time::Instant::now();

        // Clean up stale entries
        let stale_threshold = std::time::Duration::from_secs(300); // 5 minutes
        server_loads.retain(|_, load_info| {
            load_info.last_updated.elapsed() < stale_threshold
        });
    }

    /// Get actual system resource usage from pool's resource monitor
    async fn get_actual_resource_usage(&self) -> (f64, f64) {
        if let Ok(pool) = self.pool.read().await {
            // Get resource metrics from the pool
            if let Ok(metrics) = pool.get_resource_metrics().await {
                // Return memory used and CPU usage
                (metrics.memory_used_mb, metrics.cpu_usage_percent)
            } else {
                // Fallback if metrics retrieval fails
                (512.0, 15.0)
            }
        } else {
            // Fallback to reasonable defaults if pool access fails
            (512.0, 15.0) // 512MB memory, 15% CPU
        }
    }

    /// Get server load statistics
    pub async fn get_server_loads(&self) -> HashMap<String, ServerLoadInfo> {
        self.server_loads.read().await.clone()
    }

    /// Convert file extension to language
    fn extension_to_language(&self, extension: &str) -> LanguageServerKind {
        match extension.to_lowercase().as_str() {
            "rs" => LanguageServerKind::Rust,
            "ts" | "tsx" => LanguageServerKind::TypeScript,
            "js" | "jsx" => LanguageServerKind::JavaScript,
            "html" | "htm" | "xhtml" => LanguageServerKind::Html,
            "css" | "scss" | "sass" | "less" => LanguageServerKind::Css,
            "sql" | "mysql" | "pgsql" | "postgres" | "psql" => LanguageServerKind::Sql,
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
            "html" | "xhtml" => LanguageServerKind::Html,
            "css" | "scss" | "sass" | "less" => LanguageServerKind::Css,
            "sql" | "mysql" | "postgresql" | "postgres" => LanguageServerKind::Sql,
            "go" | "golang" => LanguageServerKind::Go,
            _ => LanguageServerKind::Custom(hint.to_string()),
        }
    }

    /// Convert LanguageServerKind to string for handle ID generation
    fn language_to_string(language: &LanguageServerKind) -> String {
        match language {
            LanguageServerKind::Rust => "rust".to_string(),
            LanguageServerKind::TypeScript => "typescript".to_string(),
            LanguageServerKind::JavaScript => "javascript".to_string(),
            LanguageServerKind::Html => "html".to_string(),
            LanguageServerKind::Css => "css".to_string(),
            LanguageServerKind::Sql => "sql".to_string(),
            LanguageServerKind::Go => "go".to_string(),
            LanguageServerKind::Custom(name) => name.clone(),
        }
    }

    /// Calculate intelligent routing confidence based on multiple factors
    async fn calculate_routing_confidence(
        &self,
        context: &RequestContext,
        target_language: &LanguageServerKind,
        candidates_evaluated: usize,
    ) -> f64 {
        let mut confidence = 0.5; // Base confidence

        // Factor 1: Language hint match (highest weight)
        if let Some(hint) = &context.language_hint {
            if self.hint_to_language(hint) == *target_language {
                confidence += 0.3;
            }
        }

        // Factor 2: File extension match
        if let Some(uri) = &context.document_uri {
            if let Ok(url) = url::Url::parse(uri.as_str()) {
                if let Ok(path) = url.to_file_path() {
                    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                        if self.extension_to_language(extension) == *target_language {
                            confidence += 0.2;
                        }
                    }
                }
            }
        }

        // Factor 3: Content analysis success (high weight)
        if let Ok(content_lang) = self.route_by_content(context).await {
            if content_lang == *target_language {
                confidence += 0.25;
            } else {
                confidence -= 0.1; // Penalty for mismatch
            }
        }

        // Factor 4: Workspace configuration match
        if let Ok(workspace_lang) = self.route_by_workspace_config(context).await {
            if workspace_lang == *target_language {
                confidence += 0.15;
            }
        }

        // Factor 5: Server availability and load
        if candidates_evaluated > 0 {
            confidence += 0.05; // Bonus for having candidates
            if candidates_evaluated > 1 {
                confidence += 0.05; // Additional bonus for load balancing options
            }
        } else {
            confidence -= 0.2; // Penalty for no available servers
        }

        // Factor 6: Routing strategy effectiveness
        match self.routing_strategy {
            RoutingStrategy::Intelligent => confidence += 0.1,
            RoutingStrategy::ContentAnalysis => confidence += 0.05,
            RoutingStrategy::WorkspaceConfiguration => confidence += 0.05,
            _ => {} // No bonus for simpler strategies
        }

        // Clamp to valid range
        confidence.max(0.0).min(1.0)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;
    use tokio::fs;

    fn create_test_project_manager() -> Arc<RwLock<ProjectManager>> {
        Arc::new(RwLock::new(ProjectManager::new()))
    }

    fn create_test_pool() -> Arc<RwLock<LanguageServerPool>> {
        Arc::new(RwLock::new(LanguageServerPool::new()))
    }

    #[tokio::test]
    async fn test_content_based_routing_rust() {
        let pool = create_test_pool();
        let project_manager = create_test_project_manager();
        let router = LanguageRouter::new(pool, project_manager);

        // Create test Rust content
        let temp_dir = tempdir().unwrap();
        let rust_file = temp_dir.path().join("test.rs");
        let rust_content = r#"
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("key", "value");
    println!("{:?}", map);
}

struct TestStruct {
    field: String,
}

impl TestStruct {
    fn new() -> Self {
        Self {
            field: "test".to_string(),
        }
    }
}
"#;
        fs::write(&rust_file, rust_content).await.unwrap();

        let context = RequestContext {
            method: "textDocument/hover".to_string(),
            document_uri: Some(Uri::from_file_path(&rust_file).unwrap()),
            position: None,
            selection: None,
            file_path_hint: None,
            language_hint: None,
            workspace_root: None,
        };

        let result = router.route_by_content(&context).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LanguageServerKind::Rust);
    }

    #[tokio::test]
    async fn test_content_based_routing_typescript() {
        let pool = create_test_pool();
        let project_manager = create_test_project_manager();
        let router = LanguageRouter::new(pool, project_manager);

        // Create test TypeScript content
        let temp_dir = tempdir().unwrap();
        let ts_file = temp_dir.path().join("test.ts");
        let ts_content = r#"
import React from 'react';

interface Props {
    name: string;
}

const TestComponent: React.FC<Props> = ({ name }) => {
    const [count, setCount] = React.useState(0);

    return (
        <div>
            <h1>Hello, {name}!</h1>
            <button onClick={() => setCount(count + 1)}>
                Count: {count}
            </button>
        </div>
    );
};

export default TestComponent;
"#;
        fs::write(&ts_file, ts_content).await.unwrap();

        let context = RequestContext {
            method: "textDocument/hover".to_string(),
            document_uri: Some(Uri::from_file_path(&ts_file).unwrap()),
            position: None,
            selection: None,
            file_path_hint: None,
            language_hint: None,
            workspace_root: None,
        };

        let result = router.route_by_content(&context).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LanguageServerKind::TypeScript);
    }

    #[tokio::test]
    async fn test_content_based_routing_sql() {
        let pool = create_test_pool();
        let project_manager = create_test_project_manager();
        let router = LanguageRouter::new(pool, project_manager);

        // Create test SQL content
        let temp_dir = tempdir().unwrap();
        let sql_file = temp_dir.path().join("test.sql");
        let sql_content = r#"
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE
);

INSERT INTO users (name, email) VALUES
('John Doe', 'john@example.com'),
('Jane Smith', 'jane@example.com');

SELECT * FROM users WHERE name LIKE '%John%';
"#;
        fs::write(&sql_file, sql_content).await.unwrap();

        let context = RequestContext {
            method: "textDocument/hover".to_string(),
            document_uri: Some(Uri::from_file_path(&sql_file).unwrap()),
            position: None,
            selection: None,
            file_path_hint: None,
            language_hint: None,
            workspace_root: None,
        };

        let result = router.route_by_content(&context).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LanguageServerKind::Sql);
    }

    #[tokio::test]
    async fn test_workspace_based_routing_rust() {
        let pool = create_test_pool();
        let project_manager = create_test_project_manager();

        // Add a Rust project
        let temp_dir = tempdir().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        let toml_content = r#"
[package]
name = "test_project"
version = "0.1.0"
authors = ["Test <test@example.com>"]
description = "A test project"

[dependencies]
tokio = "1.0"
serde = "1.0"
"#;
        fs::write(&cargo_toml, toml_content).await.unwrap();

        {
            let mut pm = project_manager.write().await;
            pm.add_workspace_folder(temp_dir.path().to_path_buf()).await.unwrap();
        }

        let router = LanguageRouter::new(pool, project_manager);

        let context = RequestContext {
            method: "textDocument/hover".to_string(),
            document_uri: None,
            position: None,
            selection: None,
            file_path_hint: None,
            language_hint: None,
            workspace_root: Some(temp_dir.path().to_string_lossy().to_string()),
        };

        let result = router.route_by_workspace_config(&context).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LanguageServerKind::Rust);
    }

    #[tokio::test]
    async fn test_load_based_selection() {
        let pool = create_test_pool();
        let project_manager = create_test_project_manager();
        let mut router = LanguageRouter::new(pool, project_manager);

        // Set load balancing to least loaded
        router.load_balancing = LoadBalancingStrategy::LeastLoaded;

        // Simulate server loads
        let metrics_low = ServerMetrics {
            requests_per_second: 5.0,
            average_response_time_ms: 50.0,
            total_requests: 100,
            error_count: 0,
        };

        let metrics_high = ServerMetrics {
            requests_per_second: 20.0,
            average_response_time_ms: 200.0,
            total_requests: 200,
            error_count: 0,
        };

        router.update_server_load("rust_0".to_string(), &metrics_low, ServerHealth::Healthy).await;
        router.update_server_load("rust_1".to_string(), &metrics_high, ServerHealth::Healthy).await;

        // Test selection with 2 available servers
        let selected = router.select_least_loaded_server(2, &LanguageServerKind::Rust).await;
        assert_eq!(selected, 0); // Should select the less loaded server (index 0)
    }

    #[tokio::test]
    async fn test_server_load_info() {
        let load_info = ServerLoadInfo {
            handle_id: "test_server".to_string(),
            request_rate: 10.0,
            avg_response_time_ms: 100.0,
            memory_usage_mb: 50.0,
            cpu_usage_percent: 25.0,
            active_connections: 5,
            health: ServerHealth::Healthy,
            last_updated: std::time::Instant::now(),
        };

        let load_score = load_info.load_score();
        assert!(load_score >= 0.0 && load_score <= 1.0);

        // Test with high load
        let high_load = ServerLoadInfo {
            handle_id: "high_load_server".to_string(),
            request_rate: 100.0, // Very high
            avg_response_time_ms: 2000.0, // Very slow
            memory_usage_mb: 2000.0, // High memory
            cpu_usage_percent: 90.0, // High CPU
            active_connections: 10,
            health: ServerHealth::Healthy,
            last_updated: std::time::Instant::now(),
        };

        assert!(high_load.load_score() > load_score);
    }

    #[tokio::test]
    async fn test_language_detection_from_content() {
        let pool = create_test_pool();
        let project_manager = create_test_project_manager();
        let router = LanguageRouter::new(pool, project_manager);

        // Test Rust detection
        let rust_lines = vec![
            "use std::collections::HashMap;",
            "fn main() {",
            "    let x = 42;",
            "}",
        ];
        let detected = router.detect_language_from_content(&rust_lines, &rust_lines.join("\n"));
        assert_eq!(detected, LanguageServerKind::Rust);

        // Test Go detection
        let go_lines = vec![
            "package main",
            "import \"fmt\"",
            "func main() {",
            "    fmt.Println(\"Hello\")",
            "}",
        ];
        let detected_go = router.detect_language_from_content(&go_lines, &go_lines.join("\n"));
        assert_eq!(detected_go, LanguageServerKind::Go);

        // Test HTML detection
        let html_lines = vec![
            "<!DOCTYPE html>",
            "<html>",
            "<head><title>Test</title></head>",
            "<body><h1>Hello</h1></body>",
            "</html>",
        ];
        let detected_html = router.detect_language_from_content(&html_lines, &html_lines.join("\n"));
        assert_eq!(detected_html, LanguageServerKind::Html);
    }

    #[tokio::test]
    async fn test_project_dependency_analysis() {
        let pool = create_test_pool();
        let project_manager = create_test_project_manager();
        let router = LanguageRouter::new(pool, project_manager);

        // Test Rust dependencies
        let rust_metadata = crate::project::ProjectMetadata {
            name: "test".to_string(),
            version: Some("1.0.0".to_string()),
            authors: vec![],
            description: None,
            dependencies: {
                let mut deps = HashMap::new();
                deps.insert("tokio".to_string(), "1.0".to_string());
                deps.insert("serde".to_string(), "1.0".to_string());
                deps
            },
        };

        let detected = router.analyze_project_dependencies(&rust_metadata);
        assert_eq!(detected, LanguageServerKind::Rust);

        // Test Python dependencies
        let python_metadata = crate::project::ProjectMetadata {
            name: "test".to_string(),
            version: Some("1.0.0".to_string()),
            authors: vec![],
            description: None,
            dependencies: {
                let mut deps = HashMap::new();
                deps.insert("django".to_string(), "3.0".to_string());
                deps.insert("requests".to_string(), "2.0".to_string());
                deps
            },
        };

        let detected_python = router.analyze_project_dependencies(&python_metadata);
        assert_eq!(detected_python, LanguageServerKind::Python);
    }
}

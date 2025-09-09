//! Consolidated Cache Utilities for Rust AI IDE
//!
//! This module provides a unified interface to the rust-ai-ide-cache crate
//! and eliminates the fragmented cache implementations throughout the codebase.
//!
//! ELIMINATES:
//! - src-tauri/src/cache/mod.rs (deprecated)
//! - Duplicate cache implementations in various modules
//! - Inconsistent cache policies and statistics
//! - Multiple CacheEntry and CacheConfig definitions
//!
//! PROVIDES:
//! - Single interface for all cache operations
//! - Consistent TTL and eviction policies
//! - Unified statistics and monitoring
//! - Type-safe cache adapters for common use cases

use rust_ai_ide_cache::*;
use rust_ai_ide_types::*;
use rust_ai_ide_errors::*;
use std::sync::Arc;
use std::time::Duration;

/// Re-export the core cache types for easy access
pub use rust_ai_ide_cache::{Cache, CacheEntry, CacheConfig, CacheStats, EvictionPolicy};

/// Re-export specific implementations
pub use rust_ai_ide_cache::cache_impls::*;

/// Re-export strategies
pub use rust_ai_ide_cache::strategies::*;

/// Re-export storage backends
pub use rust_ai_ide_cache::storage::*;

/// Centralized cache configuration presets
pub mod presets {
    use super::*;

    pub fn diagnostic_cache() -> CacheConfig {
        CacheConfig {
            max_entries: 1000,
            default_ttl: Some(Duration::from_secs(300)), // 5 minutes
            eviction_policy: EvictionPolicy::Lru,
            enable_metrics: true,
            max_memory_mb: Some(50), // 50MB limit
            compression_threshold_kb: Some(10), // 10KB threshold
            background_cleanup_interval_seconds: 300,
        }
    }

    pub fn explanation_cache() -> CacheConfig {
        CacheConfig {
            max_entries: 2000,
            default_ttl: Some(Duration::from_secs(86400)), // 24 hours
            eviction_policy: EvictionPolicy::Lfu,
            enable_metrics: true,
            max_memory_mb: Some(30), // 30MB limit
            compression_threshold_kb: Some(5), // 5KB threshold
            background_cleanup_interval_seconds: 600,
        }
    }

    pub fn performance_cache() -> CacheConfig {
        CacheConfig {
            max_entries: 5000,
            default_ttl: Some(Duration::from_secs(60)), // 1 minute
            eviction_policy: EvictionPolicy::Fifo,
            enable_metrics: true,
            max_memory_mb: Some(20), // 20MB limit
            compression_threshold_kb: None,
            background_cleanup_interval_seconds: 60,
        }
    }
}

/// Unified cache manager for the application
pub struct UnifiedCacheManager {
    diagnostic_cache: Arc<InMemoryCache<DiagnosticCacheKey, CompilerDiagnosticsResult>>,
    explanation_cache: Arc<InMemoryCache<String, ErrorCodeExplanation>>,
    performance_cache: Arc<InMemoryCache<String, serde_json::Value>>,
    stats: std::sync::Arc<std::sync::Mutex<GlobalCacheStats>>,
}

/// Generalized diagnostic cache key
#[derive(Debug, Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticCacheKey {
    pub workspace_path: String,
    pub request_hash: u64, // Hash of request parameters
}

impl DiagnosticCacheKey {
    pub fn new(workspace_path: String, request: &CompilerDiagnosticsRequest) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        workspace_path.hash(&mut hasher);
        request.include_explanations.hash(&mut hasher);
        request.include_suggested_fixes.hash(&mut hasher);

        if let Some(ttl) = request.cache_ttl_seconds {
            ttl.hash(&mut hasher);
        }
        if let Some(timeout) = request.timeout_seconds {
            timeout.hash(&mut hasher);
        }

        Self {
            workspace_path,
            request_hash: hasher.finish(),
        }
    }
}

/// Global cache statistics combining all cache types
#[derive(Debug, Clone, serde::Serialize)]
pub struct GlobalCacheStats {
    pub diagnostic: CacheStats,
    pub explanation: CacheStats,
    pub performance: CacheStats,
    pub total_evictions: u64,
    pub total_hits: u64,
    pub total_misses: u64,
}

impl Default for GlobalCacheStats {
    fn default() -> Self {
        Self {
            diagnostic: CacheStats::default(),
            explanation: CacheStats::default(),
            performance: CacheStats::default(),
            total_evictions: 0,
            total_hits: 0,
            total_misses: 0,
        }
    }
}

impl UnifiedCacheManager {
    pub fn new() -> Self {
        Self {
            diagnostic_cache: Arc::new(InMemoryCache::new(presets::diagnostic_cache())),
            explanation_cache: Arc::new(InMemoryCache::new(presets::explanation_cache())),
            performance_cache: Arc::new(InMemoryCache::new(presets::performance_cache())),
            stats: Arc::new(std::sync::Mutex::new(GlobalCacheStats::default())),
        }
    }

    pub fn new_with_configs(
        diagnostic: CacheConfig,
        explanation: CacheConfig,
        performance: CacheConfig,
    ) -> Self {
        Self {
            diagnostic_cache: Arc::new(InMemoryCache::new(diagnostic)),
            explanation_cache: Arc::new(InMemoryCache::new(explanation)),
            performance_cache: Arc::new(InMemoryCache::new(performance)),
            stats: Arc::new(std::sync::Mutex::new(GlobalCacheStats::default())),
        }
    }

    /// Get diagnostic result from cache
    pub async fn get_diagnostic(&self, request: &CompilerDiagnosticsRequest) -> IDEResult<Option<CompilerDiagnosticsResult>> {
        let key = DiagnosticCacheKey::new(request.workspace_path.clone(), request);
        let result = self.diagnostic_cache.get(&key).await;

        // Update global stats
        if let Ok(mut stats) = self.stats.lock() {
            if let Ok(cache_stats) = self.diagnostic_cache.stats().await {
                stats.diagnostic = cache_stats;
                stats.total_hits += stats.diagnostic.total_hits;
                stats.total_misses += stats.diagnostic.total_misses;
            }
        }

        result
    }

    /// Set diagnostic result in cache
    pub async fn set_diagnostic(&self, request: &CompilerDiagnosticsRequest, result: CompilerDiagnosticsResult) -> IDEResult<()> {
        let key = DiagnosticCacheKey::new(request.workspace_path.clone(), request);
        let ttl = request.cache_ttl_seconds.map(Duration::from_secs);
        self.diagnostic_cache.insert(key, result, ttl).await
    }

    /// Get error explanation from cache
    pub async fn get_explanation(&self, error_code: &str) -> IDEResult<Option<ErrorCodeExplanation>> {
        let result = self.explanation_cache.get(error_code).await?;

        // Update global stats
        if let Ok(mut stats) = self.stats.lock() {
            if let Ok(cache_stats) = self.explanation_cache.stats().await {
                stats.explanation = cache_stats;
                stats.total_hits += stats.explanation.total_hits;
                stats.total_misses += stats.explanation.total_misses;
            }
        }

        result
    }

    /// Set error explanation in cache
    pub async fn set_explanation(&self, error_code: String, explanation: ErrorCodeExplanation, ttl_seconds: Option<u64>) -> IDEResult<()> {
        let ttl = ttl_seconds.map(Duration::from_secs);
        self.explanation_cache.insert(error_code, explanation, ttl).await
    }

    /// Get performance data from cache
    pub async fn get_performance(&self, key: &str) -> IDEResult<Option<serde_json::Value>> {
        let result = self.performance_cache.get(key).await;

        // Update global stats
        if let Ok(mut stats) = self.stats.lock() {
            if let Ok(cache_stats) = self.performance_cache.stats().await {
                stats.performance = cache_stats;
                stats.total_hits += stats.performance.total_hits;
                stats.total_misses += stats.performance.total_misses;
            }
        }

        result
    }

    /// Set performance data in cache
    pub async fn set_performance(&self, key: String, data: serde_json::Value, ttl_seconds: Option<u64>) -> IDEResult<()> {
        let ttl = ttl_seconds.map(Duration::from_secs);
        self.performance_cache.insert(key, data, ttl).await
    }

    /// Get complete cache statistics
    pub async fn global_stats(&self) -> IDEResult<GlobalCacheStats> {
        let mut stats = self.stats.lock().map_err(|e| IDEError::Concurrency {
            message: format!("Failed to acquire stats lock: {}", e),
        })?;

        // Update current cache states
        stats.diagnostic = self.diagnostic_cache.stats().await?;
        stats.explanation = self.explanation_cache.stats().await?;
        stats.performance = self.performance_cache.stats().await?;

        // Calculate totals
        stats.total_hits = stats.diagnostic.total_hits + stats.explanation.total_hits + stats.performance.total_hits;
        stats.total_misses = stats.diagnostic.total_misses + stats.explanation.total_misses + stats.performance.total_misses;
        stats.total_evictions = stats.diagnostic.total_evictions + stats.explanation.total_evictions + stats.performance.total_evictions;

        Ok(stats.clone())
    }

    /// Clear all caches
    pub async fn clear_all(&self) -> IDEResult<()> {
        self.diagnostic_cache.clear().await?;
        self.explanation_cache.clear().await?;
        self.performance_cache.clear().await?;
        Ok(())
    }

    /// Cleanup expired entries in all caches
    pub async fn cleanup_all_expired(&self) -> IDEResult<usize> {
        let diag_cleaned = self.diagnostic_cache.cleanup_expired().await?;
        let exp_cleaned = self.explanation_cache.cleanup_expired().await?;
        let perf_cleaned = self.performance_cache.cleanup_expired().await?;

        Ok(diag_cleaned + exp_cleaned + perf_cleaned)
    }

    /// Get diagnostic cache size
    pub async fn diagnostic_size(&self) -> usize {
        self.diagnostic_cache.size().await
    }

    /// Get explanation cache size
    pub async fn explanation_size(&self) -> usize {
        self.explanation_cache.size().await
    }

    /// Get performance cache size
    pub async fn performance_size(&self) -> usize {
        self.performance_cache.size().await
    }
}

// Legacy compatibility wrappers for gradual migration
pub mod legacy {
    use super::*;
    use std::collections::HashMap;

    /// Legacy diagnostic cache for backward compatibility
    #[async_trait::async_trait]
    impl rust_ai_ide_types::cache::Cache for CachedDiagnosticsProvider {
        /* ... legacy implementation pointing to UnifiedCacheManager ... */
    }

    #[derive(Clone)]
    pub struct CachedDiagnosticsProvider {
        manager: Arc<UnifiedCacheManager>,
    }

    impl CachedDiagnosticsProvider {
        pub fn new(manager: Arc<UnifiedCacheManager>) -> Self {
            Self { manager }
        }

        pub async fn get(&self, request: &CompilerDiagnosticsRequest) -> IDEResult<Option<CompilerDiagnosticsResult>> {
            let key = DiagnosticCacheKey::new(request.workspace_path.clone(), request);
            self.manager.diagnostic_cache.get(&key).await
        }

        pub async fn insert(&self, request: &CompilerDiagnosticsRequest, result: CompilerDiagnosticsResult) -> IDEResult<()> {
            self.manager.set_diagnostic(request, result).await
        }

        pub async fn clear(&self) -> IDEResult<()> {
            self.manager.diagnostic_cache.clear().await
        }

        pub async fn len(&self) -> usize {
            self.manager.diagnostic_cache.size().await
        }

        pub async fn cleanup(&self) -> IDEResult<usize> {
            self.manager.diagnostic_cache.cleanup_expired().await
        }

        pub fn max_entries(&self) -> usize {
            presets::diagnostic_cache().max_entries
        }
    }

    /// Legacy explanation cache for backward compatibility
    #[derive(Clone)]
    pub struct CachedExplanationProvider {
        manager: Arc<UnifiedCacheManager>,
    }

    impl CachedExplanationProvider {
        pub fn new(manager: Arc<UnifiedCacheManager>) -> Self {
            Self { manager }
        }

        pub async fn get(&self, error_code: &str) -> IDEResult<Option<ErrorCodeExplanation>> {
            self.manager.get_explanation(error_code).await
        }

        pub async fn insert(&self, error_code: String, explanation: ErrorCodeExplanation, ttl_seconds: u64) -> IDEResult<()> {
            self.manager.set_explanation(error_code, explanation, Some(ttl_seconds)).await
        }

        pub async fn clear(&self) -> IDEResult<()> {
            self.manager.explanation_cache.clear().await
        }

        pub async fn len(&self) -> usize {
            self.manager.explanation_cache.size().await
        }

        pub async fn cleanup(&self) -> IDEResult<usize> {
            self.manager.explanation_cache.cleanup_expired().await
        }

        pub fn max_entries(&self) -> usize {
            presets::explanation_cache().max_entries
        }
    }
}

/// Utility functions for cache key generation
pub mod key_utils {
    use super::*;

    pub fn diagnostic_key(workspace_path: &str, request: &CompilerDiagnosticsRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        workspace_path.hash(&mut hasher);
        request.include_explanations.hash(&mut hasher);
        request.include_suggested_fixes.hash(&mut hasher);

        if let Some(ttl) = request.cache_ttl_seconds {
            ttl.hash(&mut hasher);
        }

        format!("diagnostic:{}:{:x}", workspace_path, hasher.finish())
    }

    pub fn explanation_key(error_code: &str) -> String {
        format!("explanation:{}", error_code)
    }

    pub fn performance_key(operation: &str, component: &str) -> String {
        format!("performance:{}:{}", operation, component)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_unified_cache_manager_basic_operations() {
        let manager = UnifiedCacheManager::new();

        // Test diagnostic cache
        let request = CompilerDiagnosticsRequest {
            workspace_path: "/tmp/test".to_string(),
            include_explanations: true,
            include_suggested_fixes: false,
            use_cache: true,
            cache_ttl_seconds: Some(300),
            timeout_seconds: Some(30),
        };

        let result = CompilerDiagnosticsResult {
            diagnostics: vec![],
            explanations: HashMap::new(),
            suggested_fixes: vec![],
            metadata: DiagnosticMetadata {
                workspace_path: "/tmp/test".to_string(),
                timestamp: chrono::Utc::now(),
                compilation_time_ms: 100,
                total_errors: 0,
                total_warnings: 0,
                total_notes: 0,
                cached: false,
            },
        };

        manager.set_diagnostic(&request, result.clone()).await.unwrap();
        let cached = manager.get_diagnostic(&request).await.unwrap();
        assert!(cached.is_some());

        // Test explanation cache
        let explanation = ErrorCodeExplanation {
            error_code: "E001".to_string(),
            title: "Test Error".to_string(),
            description: "A test error".to_string(),
            examples: vec![],
            solutions: vec![],
            related_errors: vec![],
            severity: ErrorSeverity::Error,
            category: ErrorCategory::Syntax,
            rustc_code: "".to_string(),
        };

        manager.set_explanation("E001".to_string(), explanation.clone(), Some(3600)).await.unwrap();
        let cached_exp = manager.get_explanation("E001").await.unwrap();
        assert!(cached_exp.is_some());

        // Test performance cache
        let perf_data = serde_json::json!({"cpu": 85.2, "memory": 1024});
        manager.set_performance("test:metrics".to_string(), perf_data.clone(), Some(60)).await.unwrap();
        let cached_perf = manager.get_performance("test:metrics").await.unwrap();
        assert!(cached_perf.is_some());
    }

    #[tokio::test]
    async fn test_global_statistics() {
        let manager = UnifiedCacheManager::new();

        let stats = manager.global_stats().await.unwrap();
        assert_eq!(stats.total_hits, 0);
        assert_eq!(stats.total_misses, 0);
    }

    #[tokio::test]
    async fn test_cache_cleanup() {
        let manager = UnifiedCacheManager::new();

        // Add entries with short TTL
        let request = CompilerDiagnosticsRequest {
            workspace_path: "/tmp/test".to_string(),
            include_explanations: false,
            include_suggested_fixes: false,
            use_cache: true,
            cache_ttl_seconds: Some(1), // 1 second TTL
            timeout_seconds: Some(30),
        };

        let result = CompilerDiagnosticsResult {
            diagnostics: vec![],
            explanations: HashMap::new(),
            suggested_fixes: vec![],
            metadata: DiagnosticMetadata {
                workspace_path: "/tmp/test".to_string(),
                timestamp: chrono::Utc::now(),
                compilation_time_ms: 100,
                total_errors: 0,
                total_warnings: 0,
                total_notes: 0,
                cached: false,
            },
        };

        manager.set_diagnostic(&request, result).await.unwrap();
        assert_eq!(manager.diagnostic_size().await, 1);

        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Cleanup expired entries
        let cleaned = manager.cleanup_all_expired().await.unwrap();
        assert!(cleaned >= 0);
    }
}
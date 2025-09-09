//! Specialized diagnostic cache implementation
//!
//! This module provides a type-safe wrapper around the unified caching
//! infrastructure specifically designed for diagnostic results.
//!
//! INTEGRATED WITH: Unified cache system from rust-ai-ide-common
//! See crates/rust-ai-ide-common/src/cache.rs for the foundation

use rust_ai_ide_cache::{Cache, InMemoryCache, CacheConfig, CacheStats};
use crate::diagnostics::{
    CompilerDiagnosticsResult,
    CompilerDiagnosticsRequest,
    DiagnosticCacheState,
    DiagnosticCache,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Key for diagnostic cache entries
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct DiagnosticCacheKey {
    pub workspace_path: String,
    pub include_explanations: bool,
    pub include_suggested_fixes: bool,
    pub request_hash: u64, // Hash of other request parameters
}

impl DiagnosticCacheKey {
    pub fn from_request(request: &CompilerDiagnosticsRequest) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        request.workspace_path.hash(&mut hasher);
        request.include_explanations.hash(&mut hasher);
        request.include_suggested_fixes.hash(&mut hasher);
        if let Some(ttl) = request.cache_ttl_seconds {
            ttl.hash(&mut hasher);
        }
        if let Some(timeout) = request.timeout_seconds {
            timeout.hash(&mut hasher);
        }

        Self {
            workspace_path: request.workspace_path.clone(),
            include_explanations: request.include_explanations,
            include_suggested_fixes: request.include_suggested_fixes,
            request_hash: hasher.finish(),
        }
    }
}

/// Diagnostic cache manager
#[derive(Clone)]
pub struct DiagnosticCacheManager {
    cache: Arc<dyn Cache<DiagnosticCacheKey, CompilerDiagnosticsResult>>,
    config: CacheConfig,
}

impl DiagnosticCacheManager {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: Arc::new(InMemoryCache::new(config.clone())),
            config,
        }
    }

    pub fn with_cache<C>(cache: Arc<C>, config: CacheConfig) -> Self
    where
        C: Cache<DiagnosticCacheKey, CompilerDiagnosticsResult> + 'static,
    {
        Self { cache, config }
    }

    /// Get cached diagnostic result
    pub async fn get_cached(&self, request: &CompilerDiagnosticsRequest) -> anyhow::Result<Option<Arc<CompilerDiagnosticsResult>>> {
        let key = DiagnosticCacheKey::from_request(request);
        self.cache.get(&key).await
    }

    /// Cache diagnostic result
    pub async fn set_cached(&self, request: &CompilerDiagnosticsRequest, result: CompilerDiagnosticsResult, ttl: Option<Duration>) -> anyhow::Result<()> {
        let key = DiagnosticCacheKey::from_request(request);
        self.cache.insert(key, result, ttl).await
    }

    /// Invalidate cache for workspace path
    pub async fn invalidate_workspace(&self, workspace_path: &str) -> anyhow::Result<usize> {
        // This is a simplified implementation
        // In practice, you'd need to iterate through the cache and remove matching entries
        // Since the generic cache doesn't expose keys, this would require an extended interface

        // For now, clear the entire cache as a conservative approach
        let size_before = self.cache.size().await;
        self.cache.clear().await?;
        let cleared = size_before - self.cache.size().await;
        Ok(cleared)
    }

    /// Invalidate cache for specific file
    pub async fn invalidate_file(&self, file_path: &str) -> anyhow::Result<usize> {
        // Similar to above, this would require cache inspection
        // For first implementation, use conservative clearing
        let size_before = self.cache.size().await;
        self.cache.clear().await?;
        let cleared = size_before - self.cache.size().await;
        Ok(cleared)
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        self.cache.stats().await
    }

    /// Clear entire cache
    pub async fn clear_cache(&self) -> anyhow::Result<()> {
        self.cache.clear().await
    }

    /// Get cache configuration
    pub fn get_config(&self) -> &CacheConfig {
        &self.config
    }
}

/// Legacy diagnostic cache wrapper for backward compatibility
/// This provides the interface expected by existing diagnostic modules
#[derive(Clone)]
pub struct LegacyDiagnosticCache {
    manager: DiagnosticCacheManager,
}

impl LegacyDiagnosticCache {
    pub fn new() -> Self {
        Self {
            manager: DiagnosticCacheManager::new(CacheConfig::default()),
        }
    }

    pub fn new_with_config(config: CacheConfig) -> Self {
        Self {
            manager: DiagnosticCacheManager::new(config),
        }
    }

    pub async fn size(&self) -> usize {
        self.manager.cache.size().await
    }

    pub async fn clear(&self) -> anyhow::Result<()> {
        self.manager.clear_cache().await
    }

    pub async fn get_stats(&self) -> CacheStats {
        self.manager.get_stats().await
    }

    /// Create a new LLCache (Low Level Cache) - backward compatibility alias
    pub fn new_llcache() -> Self {
        Self {
            manager: DiagnosticCacheManager::new(CacheConfig::default()),
        }
    }

    /// Get by key (string key for legacy compatibility)
    pub async fn get(&self, key: &str) -> anyhow::Result<Option<CompilerDiagnosticsResult>> {
        // This is a simplified mapping - in practice, you'd need proper key conversion
        Err(anyhow::anyhow!("Legacy get method not fully implemented - use DiagnosticCacheManager for full functionality"))
    }

    /// Insert with string key for legacy compatibility
    pub async fn insert(&self, _key: String, result: CompilerDiagnosticsResult) -> anyhow::Result<()> {
        // This is a simplified implementation
        // Use set_cached with proper request key instead
        log::warn!("Legacy insert method - consider using DiagnosticCacheManager.set_cached instead");
        Err(anyhow::anyhow!("Legacy insert method not implemented - use DiagnosticCacheManager.set_cached"))
    }
}

/// Type aliases for backward compatibility
pub type DiagnosticCache = LegacyDiagnosticCache;

/// Create diagnostic cache state (for tauri state management)
pub fn create_diagnostic_cache_state() -> DiagnosticCacheState {
    Arc::new(RwLock::new(LegacyDiagnosticCache::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::*;

    #[tokio::test]
    async fn test_diagnostic_cache_manager_basic() {
        let manager = DiagnosticCacheManager::new(CacheConfig::default());

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

        // Cache the result
        manager.set_cached(&request, result.clone(), None).await.unwrap();

        // Retrieve it
        let cached = manager.get_cached(&request).await.unwrap();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().diagnostics.len(), 0);
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let manager = DiagnosticCacheManager::new(CacheConfig::default());

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

        manager.set_cached(&request, result, None).await.unwrap();
        assert_eq!(manager.get_stats().await.total_entries, 1);

        // Invalidate workspace (currently clears all)
        manager.invalidate_workspace("/tmp/test").await.unwrap();
        assert_eq!(manager.get_stats().await.total_entries, 0);
    }
}
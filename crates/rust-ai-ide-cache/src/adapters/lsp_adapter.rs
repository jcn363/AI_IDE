//! LSP Cache Adapters for Backward Compatibility
//!
//! This module provides adapter traits and implementations that allow existing
//! LSP cache code to work with the unified cache system without breaking changes.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

use crate::lsp_cache::LspCacheConfig;
use crate::LspCacheExt;
use crate::{Cache, IDEResult};

// Re-export key types from LSP module for compatibility
pub use crate::lsp_cache::LspAnalysisCache as UnifiedLspCache;

/// Legacy LSP analysis result format for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAnalysisResult {
    pub result: serde_json::Value,
    pub file_hash: String,
    pub file_size: u64,
    pub file_mtime: std::time::SystemTime,
    pub dependencies: Vec<PathBuf>,
    pub cached_at: chrono::DateTime<chrono::Utc>,
    pub ttl: u64,
}

/// Legacy cache statistics format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyCacheStats {
    pub total_entries: usize,
    pub total_size_bytes: u64,
    pub hits: u64,
    pub misses: u64,
    pub hit_ratio: f64,
    pub invalidated: u64,
    pub uptime_seconds: u64,
}

/// Legacy AnalysisCaching trait for backward compatibility
#[async_trait]
pub trait LegacyAnalysisCaching: Send + Sync {
    async fn store(&self, file_path: &PathBuf, result: serde_json::Value) -> Result<(), String>;
    async fn retrieve(&self, file_path: &PathBuf) -> Result<Option<serde_json::Value>, String>;
    async fn is_valid(&self, file_path: &PathBuf) -> Result<bool, String>;
    async fn invalidate(&self, file_path: &PathBuf) -> Result<(), String>;
    async fn clear(&self) -> Result<(), String>;
    async fn stats(&self) -> Result<LegacyCacheStats, String>;
    fn size(&self) -> usize;
    async fn compact(&self) -> Result<usize, String>;
}

/// Adapter that provides legacy AnalysisCaching interface using unified cache
pub struct LegacyLspCacheAdapter {
    unified_cache: UnifiedLspCache,
    legacy_stats: std::sync::RwLock<LegacyCacheStats>,
}

impl LegacyLspCacheAdapter {
    pub fn new(config: LspCacheConfig) -> Self {
        let unified_cache = UnifiedLspCache::new(config);
        let stats = LegacyCacheStats {
            total_entries: 0,
            total_size_bytes: 0,
            hits: 0,
            misses: 0,
            hit_ratio: 0.0,
            invalidated: 0,
            uptime_seconds: 0,
        };

        Self {
            unified_cache,
            legacy_stats: std::sync::RwLock::new(stats),
        }
    }

    pub fn new_from_unified(cache: UnifiedLspCache) -> Self {
        let stats = LegacyCacheStats {
            total_entries: 0,
            total_size_bytes: 0,
            hits: 0,
            misses: 0,
            hit_ratio: 0.0,
            invalidated: 0,
            uptime_seconds: 0,
        };

        Self {
            unified_cache: cache,
            legacy_stats: std::sync::RwLock::new(stats),
        }
    }

    fn make_key(&self, file_path: &PathBuf) -> String {
        UnifiedLspCache::generate_analysis_key(file_path, "legacy_analysis", None)
    }

    async fn update_stats(&self, is_hit: bool) {
        let mut stats = self.legacy_stats.write().unwrap();
        if is_hit {
            stats.hits += 1;
        } else {
            stats.misses += 1;
        }
        stats.hit_ratio = if stats.hits + stats.misses > 0 {
            stats.hits as f64 / (stats.hits + stats.misses) as f64
        } else {
            0.0
        };
    }

    // Helper method to access cache interface through the trait
    async fn get_cache_size(&self) -> usize {
        self.unified_cache.size().await
    }

    async fn clear_cache(&self) -> IDEResult<()> {
        self.unified_cache.clear().await
    }

    async fn cleanup_expired(&self) -> IDEResult<usize> {
        self.unified_cache.cleanup_expired().await
    }
}

#[async_trait]
impl LegacyAnalysisCaching for LegacyLspCacheAdapter {
    async fn store(&self, file_path: &PathBuf, result: serde_json::Value) -> Result<(), String> {
        let key = self.make_key(file_path);

        // Create legacy-style cached result
        let cached_result = CachedAnalysisResult {
            result: result.clone(),
            file_hash: "legacy".to_string(), // Simplified for backward compatibility
            file_size: 0,
            file_mtime: std::time::SystemTime::now(),
            dependencies: Vec::new(),
            cached_at: chrono::Utc::now(),
            ttl: 1800, // 30 minutes
        };

        // Convert to JSON and store
        let json_result = serde_json::to_value(&cached_result)
            .map_err(|e| format!("Serialization error: {}", e))?;

        // Use unified cache
        self.unified_cache
            .lsp_store_analysis(
                key.clone(),
                json_result,
                "legacy_hash".to_string(),
                Some(Duration::from_secs(1800)),
            )
            .await
            .map_err(|e| format!("Cache error: {:?}", e))?;

        Ok(())
    }

    async fn retrieve(&self, file_path: &PathBuf) -> Result<Option<serde_json::Value>, String> {
        let key = self.make_key(file_path);

        let result = self
            .unified_cache
            .lsp_retrieve_analysis(&key)
            .await
            .map_err(|e| format!("Cache error: {:?}", e))?;

        if let Some(json_result) = result {
            if let Ok(cached) = serde_json::from_value::<CachedAnalysisResult>(json_result.clone())
            {
                self.update_stats(true).await;
                Ok(Some(cached.result))
            } else {
                // Return raw result if not in legacy format
                self.update_stats(true).await;
                Ok(Some(json_result))
            }
        } else {
            self.update_stats(false).await;
            Ok(None)
        }
    }

    async fn is_valid(&self, file_path: &PathBuf) -> Result<bool, String> {
        Ok(self.retrieve(file_path).await?.is_some())
    }

    async fn invalidate(&self, file_path: &PathBuf) -> Result<(), String> {
        let _ = self.unified_cache.invalidate_file(file_path).await;
        let mut stats = self.legacy_stats.write().unwrap();
        stats.invalidated += 1;
        Ok(())
    }

    async fn clear(&self) -> Result<(), String> {
        self.clear_cache()
            .await
            .map_err(|e| format!("Cache error: {:?}", e))?;

        let mut stats = self.legacy_stats.write().unwrap();
        *stats = LegacyCacheStats {
            total_entries: 0,
            total_size_bytes: 0,
            hits: 0,
            misses: 0,
            hit_ratio: 0.0,
            invalidated: 0,
            uptime_seconds: stats.uptime_seconds,
        };

        Ok(())
    }

    async fn stats(&self) -> Result<LegacyCacheStats, String> {
        let unified_stats = self.unified_cache.cache.stats().await;
        let mut legacy_stats = self.legacy_stats.read().unwrap().clone();

        legacy_stats.total_entries = unified_stats.total_entries;
        legacy_stats.total_size_bytes = unified_stats.memory_usage_bytes.unwrap_or(0);
        legacy_stats.uptime_seconds = unified_stats.uptime_seconds;

        Ok(legacy_stats)
    }

    fn size(&self) -> usize {
        // Simplified - would need async access in real implementation
        0
    }

    async fn compact(&self) -> Result<usize, String> {
        let cleaned = self
            .cleanup_expired()
            .await
            .map_err(|e| format!("Cache error: {:?}", e))?;
        Ok(cleaned)
    }
}

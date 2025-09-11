//! Diagnostic caching functionality using unified cache infrastructure

use crate::diagnostics::*;
use rust_ai_ide_cache::{Cache, CacheConfig, InMemoryCache};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Unified diagnostic cache using rust-ai-ide-cache
pub struct DiagnosticCache {
    inner: Arc<InMemoryCache<String, CompilerDiagnosticsResult>>,
}

impl Default for DiagnosticCache {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl DiagnosticCache {
    pub fn new(max_entries: usize) -> Self {
        let config = CacheConfig {
            max_entries: Some(max_entries),
            default_ttl: Some(Duration::from_secs(300)), // 5 minutes
            ..Default::default()
        };
        Self {
            inner: Arc::new(InMemoryCache::new(&config)),
        }
    }

    pub async fn get(&self, key: &str) -> Option<CompilerDiagnosticsResult> {
        if let Ok(Some(result)) = self.inner.get(key).await {
            Some((*result).clone())
        } else {
            None
        }
    }

    pub async fn insert(
        &mut self,
        key: String,
        diagnostic: CompilerDiagnosticsResult,
        ttl_seconds: u64,
    ) {
        let _ = self
            .inner
            .insert(
                key,
                diagnostic,
                Some(tokio::time::Duration::from_secs(ttl_seconds)),
            )
            .await;
    }

    pub async fn len(&self) -> usize {
        self.inner.size().await
    }

    pub async fn clear(&mut self) {
        let _ = self.inner.clear().await;
    }
}

/// Unified explanation cache using rust-ai-ide-cache
pub struct ExplanationCache {
    inner: Arc<InMemoryCache<String, ErrorCodeExplanation>>,
}

impl Default for ExplanationCache {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl ExplanationCache {
    pub fn new(max_entries: usize) -> Self {
        let config = CacheConfig {
            max_entries: Some(max_entries),
            default_ttl: Some(Duration::from_secs(1800)), // 30 minutes
            ..Default::default()
        };
        Self {
            inner: Arc::new(InMemoryCache::new(&config)),
        }
    }

    pub async fn get(&self, error_code: &str) -> Option<ErrorCodeExplanation> {
        if let Ok(Some(result)) = self.inner.get(error_code).await {
            Some((*result).clone())
        } else {
            None
        }
    }

    pub async fn insert(
        &mut self,
        error_code: String,
        explanation: ErrorCodeExplanation,
        ttl_seconds: u64,
    ) {
        let _ = self
            .inner
            .insert(
                error_code,
                explanation,
                Some(tokio::time::Duration::from_secs(ttl_seconds)),
            )
            .await;
    }

    pub async fn len(&self) -> usize {
        self.inner.size().await
    }

    pub async fn clear(&mut self) {
        let _ = self.inner.clear().await;
    }
}

/// Real-time diagnostic stream
#[derive(Debug)]
pub struct DiagnosticStream {
    pub id: String,
    pub workspace_path: String,
    pub is_active: bool,
    pub last_update: SystemTime,
    pub subscribers: Vec<String>, // Frontend connection IDs
}

/// Error code explanation request
#[derive(Debug, serde::Deserialize)]
pub struct ErrorCodeExplanationRequest {
    pub error_code: String,
    pub use_cache: bool,
    pub cache_ttl_seconds: Option<u64>,
}

/// Documentation lookup request
#[derive(Debug, serde::Deserialize)]
pub struct DocumentationLookupRequest {
    pub error_code: Option<String>,
    pub keyword: Option<String>,
    pub context: Option<String>,
}

/// Real-time diagnostics subscription request
#[derive(Debug, serde::Deserialize)]
pub struct DiagnosticStreamRequest {
    pub workspace_path: String,
    pub subscriber_id: String,
    pub auto_refresh_interval_seconds: Option<u64>,
}

// Using unified cache statistics from rust-ai-ide-cache
pub use rust_ai_ide_cache::CacheStats as CacheStatistics;

/// Enhanced diagnostic cache statistics combining multiple caches
#[derive(Debug, serde::Serialize)]
pub struct EnhancedCacheStatistics {
    pub diagnostic_cache: CacheStatistics,
    pub explanation_cache: CacheStatistics,
    pub total_cache_size: usize,
    pub diagnostic_cache_hit_ratio: f32,
    pub explanation_cache_hit_ratio: f32,
}

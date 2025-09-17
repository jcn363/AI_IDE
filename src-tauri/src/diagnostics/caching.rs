//! Diagnostic caching functionality using unified cache infrastructure with collaboration support

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use rust_ai_ide_cache::{Cache, CacheConfig, InMemoryCache};
use tokio::sync::RwLock;

use crate::command_templates::spawn_background_task;
use crate::diagnostics::*;
use crate::errors::IDEError;
use crate::infra::EventBus;

/// Unified diagnostic cache using rust-ai-ide-cache with collaboration support
pub struct DiagnosticCache {
    inner: Arc<RwLock<InMemoryCache<String, CompilerDiagnosticsResult>>>,
    event_bus: Arc<EventBus>,
    collaboration_sessions: Arc<RwLock<HashMap<String, Vec<String>>>>, // session_id -> diagnostic_keys
}

impl Default for DiagnosticCache {
    fn default() -> Self {
        Self::new(1000, Arc::new(EventBus::new()))
    }
}

impl DiagnosticCache {
    pub fn new(max_entries: usize, event_bus: Arc<EventBus>) -> Self {
        let config = CacheConfig {
            max_entries: Some(max_entries),
            default_ttl: Some(Duration::from_secs(300)), // 5 minutes
            ..Default::default()
        };
        Self {
            inner: Arc::new(RwLock::new(InMemoryCache::new(&config))),
            event_bus,
            collaboration_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start background cache maintenance tasks
    pub fn start_background_tasks(&self) -> String {
        let cache_clone = Arc::clone(&self.inner);
        let sessions_clone = Arc::clone(&self.collaboration_sessions);

        spawn_background_task(
            async move {
                let mut interval = tokio::time::interval(Duration::from_secs(60)); // Every minute
                loop {
                    interval.tick().await;
                    // Periodic cleanup and maintenance
                    let mut cache = cache_clone.write().await;
                    let _ = cache.cleanup().await;

                    // Clean up stale collaboration sessions
                    let mut sessions = sessions_clone.write().await;
                    sessions.retain(|_, keys| !keys.is_empty());
                }
            },
            "diagnostic_cache_maintenance",
        )
    }

    pub async fn get(&self, key: &str) -> Result<Option<CompilerDiagnosticsResult>, IDEError> {
        let cache = self.inner.read().await;
        match cache.get(key).await {
            Ok(Some(result)) => Ok(Some((*result).clone())),
            Ok(None) => Ok(None),
            Err(e) => Err(IDEError::Cache(format!("Failed to get from cache: {}", e))),
        }
    }

    pub async fn insert(
        &self,
        key: String,
        diagnostic: CompilerDiagnosticsResult,
        ttl_seconds: u64,
        session_id: Option<String>,
    ) -> Result<(), IDEError> {
        let mut cache = self.inner.write().await;
        let result = cache
            .insert(
                key.clone(),
                diagnostic.clone(),
                Some(tokio::time::Duration::from_secs(ttl_seconds)),
            )
            .await;

        if result.is_ok() {
            // Emit cache update event
            let event_data = serde_json::json!({
                "type": "diagnostic_cache_update",
                "key": key.clone(),
                "session_id": session_id
            });
            let _ = self.event_bus.emit("diagnostics", event_data).await;

            // Track for collaboration if session provided
            if let Some(session) = session_id {
                let mut sessions = self.collaboration_sessions.write().await;
                sessions.entry(session).or_insert_with(Vec::new).push(key);
            }
        }

        result.map_err(|e| IDEError::Cache(format!("Failed to insert into cache: {}", e)))
    }

    pub async fn len(&self) -> Result<usize, IDEError> {
        let cache = self.inner.read().await;
        cache
            .size()
            .await
            .map_err(|e| IDEError::Cache(format!("Failed to get cache size: {}", e)))
    }

    pub async fn clear(&self) -> Result<(), IDEError> {
        let mut cache = self.inner.write().await;
        cache
            .clear()
            .await
            .map_err(|e| IDEError::Cache(format!("Failed to clear cache: {}", e)))?;

        // Clear collaboration sessions
        let mut sessions = self.collaboration_sessions.write().await;
        sessions.clear();

        // Emit clear event
        let event_data = serde_json::json!({
            "type": "diagnostic_cache_cleared"
        });
        let _ = self.event_bus.emit("diagnostics", event_data).await;

        Ok(())
    }

    /// Merge diagnostics from collaboration session
    pub async fn merge_diagnostics(
        &self,
        session_id: &str,
        new_diagnostics: HashMap<String, CompilerDiagnosticsResult>,
        ttl_seconds: u64,
    ) -> Result<(), IDEError> {
        for (key, diagnostic) in new_diagnostics {
            self.insert(key, diagnostic, ttl_seconds, Some(session_id.to_string()))
                .await?;
        }

        // Emit collaboration merge event
        let event_data = serde_json::json!({
            "type": "collaboration_merge",
            "session_id": session_id,
            "count": new_diagnostics.len()
        });
        let _ = self.event_bus.emit("diagnostics", event_data).await;

        Ok(())
    }

    /// Get diagnostic keys for a collaboration session
    pub async fn get_session_diagnostics(&self, session_id: &str) -> Vec<String> {
        let sessions = self.collaboration_sessions.read().await;
        sessions.get(session_id).cloned().unwrap_or_default()
    }

    /// Handle collaboration conflicts (simple last-writer-wins for now)
    pub async fn handle_collaboration_conflict(
        &self,
        key: String,
        local: CompilerDiagnosticsResult,
        remote: CompilerDiagnosticsResult,
        session_id: &str,
    ) -> Result<(), IDEError> {
        // For now, prefer remote (assume it's newer)
        self.insert(key.clone(), remote, 300, Some(session_id.to_string()))
            .await?;

        // Emit conflict resolution event
        let event_data = serde_json::json!({
            "type": "collaboration_conflict_resolved",
            "key": key,
            "session_id": session_id,
            "resolution": "remote_preferred"
        });
        let _ = self.event_bus.emit("diagnostics", event_data).await;

        Ok(())
    }
}

/// Unified explanation cache using rust-ai-ide-cache with collaboration support
pub struct ExplanationCache {
    inner: Arc<RwLock<InMemoryCache<String, ErrorCodeExplanation>>>,
    event_bus: Arc<EventBus>,
}

impl Default for ExplanationCache {
    fn default() -> Self {
        Self::new(1000, Arc::new(EventBus::new()))
    }
}

impl ExplanationCache {
    pub fn new(max_entries: usize, event_bus: Arc<EventBus>) -> Self {
        let config = CacheConfig {
            max_entries: Some(max_entries),
            default_ttl: Some(Duration::from_secs(1800)), // 30 minutes
            ..Default::default()
        };
        Self {
            inner: Arc::new(RwLock::new(InMemoryCache::new(&config))),
            event_bus,
        }
    }

    pub async fn get(&self, error_code: &str) -> Result<Option<ErrorCodeExplanation>, IDEError> {
        let cache = self.inner.read().await;
        match cache.get(error_code).await {
            Ok(Some(result)) => Ok(Some((*result).clone())),
            Ok(None) => Ok(None),
            Err(e) => Err(IDEError::Cache(format!(
                "Failed to get from explanation cache: {}",
                e
            ))),
        }
    }

    pub async fn insert(
        &self,
        error_code: String,
        explanation: ErrorCodeExplanation,
        ttl_seconds: u64,
    ) -> Result<(), IDEError> {
        let mut cache = self.inner.write().await;
        cache
            .insert(
                error_code.clone(),
                explanation,
                Some(tokio::time::Duration::from_secs(ttl_seconds)),
            )
            .await
            .map_err(|e| {
                IDEError::Cache(format!("Failed to insert into explanation cache: {}", e))
            })?;

        // Emit cache update event
        let event_data = serde_json::json!({
            "type": "explanation_cache_update",
            "error_code": error_code
        });
        let _ = self.event_bus.emit("diagnostics", event_data).await;

        Ok(())
    }

    pub async fn len(&self) -> Result<usize, IDEError> {
        let cache = self.inner.read().await;
        cache
            .size()
            .await
            .map_err(|e| IDEError::Cache(format!("Failed to get explanation cache size: {}", e)))
    }

    pub async fn clear(&self) -> Result<(), IDEError> {
        let mut cache = self.inner.write().await;
        cache
            .clear()
            .await
            .map_err(|e| IDEError::Cache(format!("Failed to clear explanation cache: {}", e)))?;

        // Emit clear event
        let event_data = serde_json::json!({
            "type": "explanation_cache_cleared"
        });
        let _ = self.event_bus.emit("diagnostics", event_data).await;

        Ok(())
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

/// Enhanced diagnostic cache statistics combining multiple caches with collaboration metrics
#[derive(Debug, serde::Serialize)]
pub struct EnhancedCacheStatistics {
    pub diagnostic_cache: CacheStatistics,
    pub explanation_cache: CacheStatistics,
    pub total_cache_size: usize,
    pub diagnostic_cache_hit_ratio: f32,
    pub explanation_cache_hit_ratio: f32,
    pub collaboration_sessions_active: usize,
    pub total_collaboration_keys: usize,
    pub collaboration_merge_count: u64,
    pub collaboration_conflict_count: u64,
}

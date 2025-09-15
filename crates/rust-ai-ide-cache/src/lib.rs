//! Unified Caching Infrastructure for Rust AI IDE
//!
//! This crate provides a comprehensive caching solution that consolidates
//! multiple cache implementations found throughout the Rust AI IDE codebase.
//!
//! Instead of having multiple cache types like:
//! - GenericCache, InMemoryCache, DiagnosticCache, LegacyDiagnosticCache
//! - CachedItem, CacheEntry (different names for similar concepts)
//! - Duplicate CacheStatistics across modules
//!
//! This crate provides:
//! - Unified Cache trait with async support
//! - Multiple storage backends (memory, disk, hybrid)
//! - Rich TTL and eviction policies
//! - Serialization/deserialization support
//! - Performance monitoring and metrics
//! - Type-safe key generation
//! - Async operations with tokio
//! - Collaboration features with session management
//! - Shared cache functionality across users
//! - Real-time collaborative cache operations

pub mod adapters;
pub mod cache_impls;
pub mod lsp_cache;
pub mod storage;
pub mod strategies;

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use rust_ai_ide_errors::IDEResult;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;
// Using chrono::DateTime<chrono::Utc> as Timestamp alias
type Timestamp = chrono::DateTime<chrono::Utc>;

/// Re-export commonly used items
pub use cache_impls::*;
pub use storage::*;
pub use strategies::*;

/// Collaboration-related types and identifiers
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct SessionId(pub Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Option<Self> {
        Uuid::parse_str(s).ok().map(Self)
    }

    pub fn as_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct UserId(pub String);

impl UserId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for UserId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Collaboration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationConfig {
    pub enable_session_sharing:  bool,
    pub enable_user_isolation:   bool,
    pub session_ttl:             Option<Duration>,
    pub max_users_per_session:   Option<usize>,
    pub enable_realtime_updates: bool,
    pub shared_cache_prefix:     Option<String>,
}

impl Default for CollaborationConfig {
    fn default() -> Self {
        Self {
            enable_session_sharing:  true,
            enable_user_isolation:   false,
            session_ttl:             Some(Duration::from_secs(3600)), // 1 hour
            max_users_per_session:   Some(10),
            enable_realtime_updates: true,
            shared_cache_prefix:     Some("shared:".to_string()),
        }
    }
}

/// Session metadata for collaborative caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id:     SessionId,
    pub created_at:     Timestamp,
    pub last_activity:  Timestamp,
    pub user_count:     usize,
    pub shared_entries: usize,
    pub owner_id:       Option<UserId>,
}

impl Default for SessionMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            session_id:     SessionId::new(),
            created_at:     now,
            last_activity:  now,
            user_count:     1,
            shared_entries: 0,
            owner_id:       None,
        }
    }
}

/// Unified cache entry with rich metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<V> {
    pub value:         V,
    pub created_at:    Timestamp,
    pub last_accessed: Timestamp,
    pub expires_at:    Option<Timestamp>,
    pub access_count:  u64,
    pub ttl_seconds:   Option<u64>,
    pub metadata:      HashMap<String, String>,
}

impl<V> CacheEntry<V> {
    pub fn new(value: V) -> Self {
        let now = chrono::Utc::now();
        Self::new_with_ttl(value, None, now)
    }

    pub fn new_with_ttl(value: V, ttl: Option<Duration>, created_at: Timestamp) -> Self {
        let expires_at = ttl.map(|t| {
            let duration_ms = t.as_millis() as i64;
            created_at + chrono::Duration::milliseconds(duration_ms)
        });

        Self {
            value,
            created_at,
            last_accessed: created_at,
            expires_at,
            access_count: 0,
            ttl_seconds: ttl.map(|t| t.as_secs()),
            metadata: HashMap::new(),
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() > expires_at
        } else {
            false
        }
    }

    pub fn access(&mut self) {
        self.access_count += 1;
        self.last_accessed = chrono::Utc::now();
    }

    pub fn update_value(&mut self, value: V) {
        self.value = value;
        self.last_accessed = chrono::Utc::now();
    }

    pub fn refresh_ttl(&mut self, ttl: Option<Duration>) {
        let new_expires_at = ttl.map(|t| {
            let duration_ms = t.as_millis() as i64;
            self.last_accessed + chrono::Duration::milliseconds(duration_ms)
        });
        self.expires_at = new_expires_at;
        self.ttl_seconds = ttl.map(|t| t.as_secs());
    }

    pub fn size_hint(&self) -> usize
    where
        V: serde::Serialize,
    {
        // Rough estimate of serialized size
        serde_json::to_string(self).map(|s| s.len()).unwrap_or(0)
    }
}

/// Unified cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_entries: Option<usize>,
    pub default_ttl: Option<Duration>,
    pub eviction_policy: EvictionPolicy,
    pub enable_metrics: bool,
    pub max_memory_mb: Option<usize>,
    pub compression_threshold_kb: Option<usize>,
    pub background_cleanup_interval_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: Some(1000),
            default_ttl: Some(Duration::from_secs(300)), // 5 minutes
            eviction_policy: EvictionPolicy::Lru,
            enable_metrics: true,
            max_memory_mb: None,
            compression_threshold_kb: None,
            background_cleanup_interval_seconds: 300, // 5 minutes
        }
    }
}

/// Eviction policies for cache
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EvictionPolicy {
    /// Least Recently Used - evict oldest accessed
    Lru,
    /// Least Frequently Used - evict least accessed
    Lfu,
    /// First In First Out - evict oldest
    Fifo,
    /// Random eviction
    Random,
    /// Size-based eviction (for memory limits)
    SizeBased,
    /// Adaptive strategy with dynamic weight adjustment
    Adaptive,
    /// Windowed TinyLFU - advanced frequency-based eviction
    #[serde(rename = "w_tiny_lfu")]
    WTinyLFU,
    /// Segmented LRU - hybrid of LRU with multiple segments
    #[serde(rename = "segmented_lru")]
    SegmentedLRU,
    /// Clock algorithm - approximate LRU with O(1) operations
    Clock,
}

/// Unified cache trait that unifies all cache implementations
#[async_trait]
pub trait Cache<K, V>: Send + Sync + 'static
where
    K: Send + Sync + Clone + Hash + Eq + serde::Serialize,
    V: Send + Sync + Clone + serde::Serialize,
{
    /// Get a value from the cache
    async fn get(&self, key: &K) -> IDEResult<Option<V>>;

    /// Insert a value into the cache
    async fn insert(&self, key: K, value: V, ttl: Option<Duration>) -> IDEResult<()>;

    /// Remove a value from the cache
    async fn remove(&self, key: &K) -> IDEResult<Option<V>>;

    /// Clear all entries from the cache
    async fn clear(&self) -> IDEResult<()>;

    /// Get cache size
    async fn size(&self) -> usize;

    /// Check if cache contains key
    async fn contains(&self, key: &K) -> bool;

    /// Get cache statistics
    async fn stats(&self) -> CacheStats;

    /// Force cleanup of expired entries
    async fn cleanup_expired(&self) -> IDEResult<usize>;
}

/// Enhanced cache trait with collaboration support
#[async_trait]
pub trait CollaborativeCache<K, V>: Cache<K, V> + Send + Sync + 'static
where
    K: Send + Sync + Clone + Hash + Eq + serde::Serialize + 'static,
    V: Send + Sync + Clone + serde::Serialize + 'static,
{
    /// Create a new collaborative session
    async fn create_session(&self, owner_id: UserId) -> IDEResult<SessionId>;

    /// Join an existing collaborative session
    async fn join_session(&self, session_id: &SessionId, user_id: UserId) -> IDEResult<()>;

    /// Leave a collaborative session
    async fn leave_session(&self, session_id: &SessionId, user_id: UserId) -> IDEResult<()>;

    /// Get session metadata
    async fn get_session_metadata(&self, session_id: &SessionId) -> IDEResult<Option<SessionMetadata>>;

    /// Get a value from cache within a session context
    async fn get_session(&self, session_id: &SessionId, key: &K) -> IDEResult<Option<V>>;

    /// Insert a value into cache within a session context
    async fn insert_session(&self, session_id: &SessionId, key: K, value: V, ttl: Option<Duration>) -> IDEResult<()>;

    /// Remove a value from cache within a session context
    async fn remove_session(&self, session_id: &SessionId, key: &K) -> IDEResult<Option<V>>;

    /// Get a user-specific value from cache
    async fn get_user(&self, user_id: &UserId, key: &K) -> IDEResult<Option<V>>;

    /// Insert a user-specific value into cache
    async fn insert_user(&self, user_id: &UserId, key: K, value: V, ttl: Option<Duration>) -> IDEResult<()>;

    /// Share a cache entry across a session
    async fn share_entry(&self, session_id: &SessionId, key: &K, from_user: &UserId) -> IDEResult<()>;

    /// Get shared entries for a session
    async fn get_shared_entries(&self, session_id: &SessionId) -> IDEResult<Vec<K>>;

    /// Invalidate session cache when collaboration ends
    async fn invalidate_session(&self, session_id: &SessionId) -> IDEResult<usize>;

    /// Get collaborative cache statistics
    async fn collaborative_stats(&self) -> CollaborativeCacheStats;
}

/// Cache performance metrics and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries:      usize,
    pub total_hits:         u64,
    pub total_misses:       u64,
    pub total_evictions:    u64,
    pub total_sets:         u64,
    pub hit_ratio:          f64,
    pub memory_usage_bytes: Option<u64>,
    pub uptime_seconds:     u64,
    pub created_at:         Timestamp,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            total_entries:      0,
            total_hits:         0,
            total_misses:       0,
            total_evictions:    0,
            total_sets:         0,
            hit_ratio:          0.0,
            memory_usage_bytes: None,
            uptime_seconds:     0,
            created_at:         chrono::Utc::now(),
        }
    }
}

impl CacheStats {
    pub fn record_hit(&mut self) {
        self.total_hits += 1;
        self.update_hit_ratio();
    }

    pub fn record_miss(&mut self) {
        self.total_misses += 1;
        self.update_hit_ratio();
    }

    pub fn record_set(&mut self) {
        self.total_sets += 1;
    }

    pub fn record_eviction(&mut self) {
        self.total_evictions += 1;
    }

    pub fn update_hit_ratio(&mut self) {
        let total = self.total_hits + self.total_misses;
        self.hit_ratio = if total > 0 {
            self.total_hits as f64 / total as f64
        } else {
            0.0
        };
    }
}

/// Collaborative cache performance metrics and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborativeCacheStats {
    /// Base cache stats
    pub base_stats:               CacheStats,
    /// Total number of active sessions
    pub active_sessions:          usize,
    /// Total number of users across all sessions
    pub total_users:              usize,
    /// Total shared cache entries
    pub shared_entries:           usize,
    /// Session creation rate (per minute)
    pub session_creation_rate:    f64,
    /// Average users per session
    pub avg_users_per_session:    f64,
    /// Session hit ratio (cache hits within sessions)
    pub session_hit_ratio:        f64,
    /// User isolation hit ratio
    pub user_isolation_hit_ratio: f64,
    /// Real-time update operations
    pub realtime_updates:         u64,
    /// Created at timestamp
    pub created_at:               Timestamp,
}

impl Default for CollaborativeCacheStats {
    fn default() -> Self {
        Self {
            base_stats:               CacheStats::default(),
            active_sessions:          0,
            total_users:              0,
            shared_entries:           0,
            session_creation_rate:    0.0,
            avg_users_per_session:    0.0,
            session_hit_ratio:        0.0,
            user_isolation_hit_ratio: 0.0,
            realtime_updates:         0,
            created_at:               chrono::Utc::now(),
        }
    }
}

impl CollaborativeCacheStats {
    pub fn update_session_metrics(&mut self) {
        if self.active_sessions > 0 {
            self.avg_users_per_session = self.total_users as f64 / self.active_sessions as f64;
        } else {
            self.avg_users_per_session = 0.0;
        }
    }

    pub fn record_session_created(&mut self) {
        self.active_sessions += 1;
        self.update_session_metrics();
    }

    pub fn record_session_joined(&mut self) {
        self.total_users += 1;
        self.update_session_metrics();
    }

    pub fn record_session_left(&mut self) {
        if self.total_users > 0 {
            self.total_users -= 1;
            self.update_session_metrics();
        }
    }

    pub fn record_realtime_update(&mut self) {
        self.realtime_updates += 1;
    }
}

/// Cache manager for coordinating multiple caches with collaboration support
pub struct CacheManager {
    caches:        HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
    config:        CacheConfig,
    collab_config: CollaborationConfig,
    stats:         RwLock<CacheStats>,
    collab_stats:  RwLock<CollaborativeCacheStats>,
    sessions:      RwLock<HashMap<SessionId, SessionMetadata>>,
    user_sessions: RwLock<HashMap<UserId, SessionId>>,
}

impl CacheManager {
    pub fn new(config: CacheConfig) -> Self {
        let stats = CacheStats {
            created_at: chrono::Utc::now(),
            uptime_seconds: 0,
            ..Default::default()
        };

        let collab_stats = CollaborativeCacheStats {
            base_stats: stats.clone(),
            created_at: chrono::Utc::now(),
            ..Default::default()
        };

        Self {
            caches: HashMap::new(),
            config,
            collab_config: CollaborationConfig::default(),
            stats: RwLock::new(stats),
            collab_stats: RwLock::new(collab_stats),
            sessions: RwLock::new(HashMap::new()),
            user_sessions: RwLock::new(HashMap::new()),
        }
    }

    pub fn new_with_collaboration(config: CacheConfig, collab_config: CollaborationConfig) -> Self {
        let mut manager = Self::new(config);
        manager.collab_config = collab_config;
        manager
    }

    pub fn register_cache<K, V, C>(&mut self, name: &str, cache: C)
    where
        K: Send + Sync + Clone + Hash + Eq + serde::Serialize + 'static,
        V: Send + Sync + Clone + serde::Serialize + 'static,
        C: Cache<K, V> + 'static,
    {
        self.caches.insert(name.to_string(), Box::new(cache));
    }

    pub async fn cleanup_all(&self) -> IDEResult<usize> {
        let total_cleaned = 0;
        // Note: In real implementation, we'd iterate through caches and call cleanup_expired
        // This is simplified for the consolidation example
        Ok(total_cleaned)
    }

    pub async fn global_stats(&self) -> CacheStats {
        let mut stats = self.stats.read().await.clone();
        stats.uptime_seconds = (chrono::Utc::now() - stats.created_at)
            .as_seconds_f64()
            .abs() as u64;
        stats
    }

    /// Create a new collaborative session
    pub async fn create_session(&self, owner_id: UserId) -> IDEResult<SessionId> {
        let session_id = SessionId::new();
        let now = chrono::Utc::now();

        let metadata = SessionMetadata {
            session_id:     session_id.clone(),
            created_at:     now,
            last_activity:  now,
            user_count:     1,
            shared_entries: 0,
            owner_id:       Some(owner_id.clone()),
        };

        let mut sessions = self.sessions.write().await;
        let mut user_sessions = self.user_sessions.write().await;
        let mut collab_stats = self.collab_stats.write().await;

        sessions.insert(session_id.clone(), metadata);
        user_sessions.insert(owner_id, session_id.clone());

        collab_stats.record_session_created();

        Ok(session_id)
    }

    /// Join an existing collaborative session
    pub async fn join_session(&self, session_id: &SessionId, user_id: UserId) -> IDEResult<()> {
        let mut sessions = self.sessions.write().await;
        let mut user_sessions = self.user_sessions.write().await;
        let mut collab_stats = self.collab_stats.write().await;

        if let Some(metadata) = sessions.get_mut(session_id) {
            // Check user limit if configured
            if let Some(max_users) = self.collab_config.max_users_per_session {
                if metadata.user_count >= max_users {
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "Session full").into());
                }
            }

            metadata.user_count += 1;
            metadata.last_activity = chrono::Utc::now();
            user_sessions.insert(user_id, session_id.clone());
            collab_stats.record_session_joined();

            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Session not found").into())
        }
    }

    /// Leave a collaborative session
    pub async fn leave_session(&self, session_id: &SessionId, user_id: UserId) -> IDEResult<()> {
        let mut sessions = self.sessions.write().await;
        let mut user_sessions = self.user_sessions.write().await;
        let mut collab_stats = self.collab_stats.write().await;

        if let Some(metadata) = sessions.get_mut(session_id) {
            if metadata.user_count > 0 {
                metadata.user_count -= 1;
                metadata.last_activity = chrono::Utc::now();
                user_sessions.remove(&user_id);
                collab_stats.record_session_left();

                // Clean up empty sessions
                if metadata.user_count == 0 {
                    sessions.remove(session_id);
                }
            }
        }

        Ok(())
    }

    /// Get session metadata
    pub async fn get_session_metadata(&self, session_id: &SessionId) -> Option<SessionMetadata> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// Get collaborative cache statistics
    pub async fn collaborative_stats(&self) -> CollaborativeCacheStats {
        let mut stats = self.collab_stats.read().await.clone();
        stats.base_stats = self.global_stats().await;
        stats
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> IDEResult<usize> {
        let mut sessions = self.sessions.write().await;
        let mut user_sessions = self.user_sessions.write().await;
        let mut collab_stats = self.collab_stats.write().await;
        let mut cleaned = 0;

        if let Some(session_ttl) = self.collab_config.session_ttl {
            let now = chrono::Utc::now();
            let expired_sessions: Vec<_> = sessions
                .iter()
                .filter_map(|(id, metadata)| {
                    if (now - metadata.last_activity).num_seconds() as u64 > session_ttl.as_secs() {
                        Some(id.clone())
                    } else {
                        None
                    }
                })
                .collect();

            for session_id in expired_sessions {
                if let Some(metadata) = sessions.remove(&session_id) {
                    cleaned += metadata.user_count as usize;
                    collab_stats.total_users = collab_stats.total_users.saturating_sub(metadata.user_count);
                    collab_stats.active_sessions = collab_stats.active_sessions.saturating_sub(1);

                    // Remove user mappings for this session
                    user_sessions.retain(|_, sid| sid != &session_id);
                }
            }
        }

        Ok(cleaned)
    }
}

/// Type-safe cache key generation utilities
pub mod key_utils {

    use serde::Serialize;

    /// Generate a cache key from multiple components
    pub fn generate_key(components: &[&(impl Serialize + ?Sized)]) -> String {
        let mut key = String::new();
        for (i, component) in components.iter().enumerate() {
            if i > 0 {
                key.push(';');
            }
            let json = serde_json::to_string(component).unwrap_or_default();
            key.push_str(&json);
        }
        sha256::digest(key)
    }

    /// Generate a structured cache key
    pub fn structured_key(prefix: &str, data: &(impl Serialize + ?Sized)) -> String {
        format!("{}:{}", prefix, generate_key(&[data]))
    }

    /// Generate a path-based cache key
    pub fn path_key(operation: &str, path: &std::path::Path) -> String {
        format!("{}:{}", operation, path.display())
    }
}

/// Shorthand type aliases for common use cases
pub type StringCache = InMemoryCache<String, String>;
/// TODO: Define CompilerDiagnosticsResult in rust_ai_ide_types
// pub type DiagnosticCache = InMemoryCache<String, rust_ai_ide_types::CompilerDiagnosticsResult>;
/// TODO: Define ErrorCodeExplanation in rust_ai_ide_types
// pub type ExplanationCache = InMemoryCache<String, rust_ai_ide_types::ErrorCodeExplanation>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_entry_lifecycle() {
        let entry = CacheEntry::new("test_value");
        assert!(!entry.is_expired());
        assert_eq!(entry.access_count, 0);

        let mut entry_clone = entry.clone();
        entry_clone.access();
        assert_eq!(entry_clone.access_count, 1);
    }

    #[tokio::test]
    async fn test_entry_with_ttl() {
        let short_ttl = Duration::from_millis(10);
        let entry = CacheEntry::new_with_ttl("ttl_value", Some(short_ttl), chrono::Utc::now());

        assert!(!entry.is_expired());

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::default();

        stats.record_hit();
        stats.record_hit();
        stats.record_miss();

        assert_eq!(stats.total_hits, 2);
        assert_eq!(stats.total_misses, 1);
        assert_eq!(stats.hit_ratio, 2.0 / 3.0);
    }

    #[test]
    fn test_key_generation() {
        let key = key_utils::structured_key("test", &"data");
        assert!(!key.is_empty());

        let multi_key = key_utils::generate_key(&[&"component1", &"component2"]);
        assert!(!multi_key.is_empty());
    }

    #[test]
    fn test_session_id_creation() {
        let session_id = SessionId::new();
        assert!(!session_id.as_string().is_empty());

        let parsed = SessionId::from_string(&session_id.as_string());
        assert_eq!(parsed, Some(session_id));
    }

    #[test]
    fn test_user_id_creation() {
        let user_id = UserId::new("test_user");
        assert_eq!(user_id.as_str(), "test_user");

        let from_string: UserId = "another_user".into();
        assert_eq!(from_string.as_str(), "another_user");
    }

    #[test]
    fn test_collaboration_config_defaults() {
        let config = CollaborationConfig::default();
        assert!(config.enable_session_sharing);
        assert!(!config.enable_user_isolation);
        assert!(config.session_ttl.is_some());
        assert!(config.max_users_per_session.is_some());
        assert!(config.enable_realtime_updates);
    }

    #[test]
    fn test_session_metadata_defaults() {
        let metadata = SessionMetadata::default();
        assert_eq!(metadata.user_count, 1);
        assert_eq!(metadata.shared_entries, 0);
        assert!(metadata.owner_id.is_none());
    }

    #[test]
    fn test_collaborative_cache_stats() {
        let mut stats = CollaborativeCacheStats::default();
        assert_eq!(stats.active_sessions, 0);
        assert_eq!(stats.total_users, 0);

        stats.record_session_created();
        assert_eq!(stats.active_sessions, 1);

        stats.record_session_joined();
        assert_eq!(stats.total_users, 1);
        assert_eq!(stats.avg_users_per_session, 1.0);

        stats.record_session_left();
        assert_eq!(stats.total_users, 0);
        assert_eq!(stats.avg_users_per_session, 0.0);
    }

    #[tokio::test]
    async fn test_cache_manager_collaboration() {
        let manager = CacheManager::new(CacheConfig::default());

        // Create a session
        let owner_id = UserId::new("owner");
        let session_id = manager.create_session(owner_id.clone()).await.unwrap();

        // Check session exists
        let metadata = manager.get_session_metadata(&session_id).await.unwrap();
        assert_eq!(metadata.owner_id, Some(owner_id));
        assert_eq!(metadata.user_count, 1);

        // Join session
        let user_id = UserId::new("user");
        manager
            .join_session(&session_id, user_id.clone())
            .await
            .unwrap();

        // Check updated metadata
        let metadata = manager.get_session_metadata(&session_id).await.unwrap();
        assert_eq!(metadata.user_count, 2);

        // Leave session
        manager.leave_session(&session_id, user_id).await.unwrap();
        let metadata = manager.get_session_metadata(&session_id).await.unwrap();
        assert_eq!(metadata.user_count, 1);

        // Leave with owner
        manager.leave_session(&session_id, owner_id).await.unwrap();

        // Session should be cleaned up
        let metadata = manager.get_session_metadata(&session_id).await;
        assert!(metadata.is_none());
    }
}

// Domain-specific cache trait extensions

/// Extensions for LSP-specific caching patterns
#[async_trait]
pub trait LspCacheExt: Cache<String, serde_json::Value> + Send + Sync + 'static {
    /// Cache LSP analysis results with file validation metadata
    async fn lsp_store_analysis(
        &self,
        file_key: String,
        result: serde_json::Value,
        file_hash: String,
        ttl: Option<Duration>,
    ) -> IDEResult<()> {
        // Extend the base result with LSP metadata
        let mut cache_entry = CacheEntry::new(result.clone());
        cache_entry
            .metadata
            .insert("file_hash".to_string(), file_hash);
        cache_entry
            .metadata
            .insert("cache_type".to_string(), "lsp_analysis".to_string());

        // Use the unified cache insert method
        self.insert(file_key, result, ttl).await
    }

    /// Retrieve LSP analysis result with validation
    async fn lsp_retrieve_analysis(&self, file_key: &String) -> IDEResult<Option<serde_json::Value>> {
        self.get(file_key).await
    }
}

// Auto-implement for any cache that implements the base trait
impl<C> LspCacheExt for C where C: Cache<String, serde_json::Value> + Send + Sync + 'static {}

/// Specialized cache for high-performance AI operations
#[async_trait]
pub trait AiCacheExt: Cache<String, serde_json::Value> + Send + Sync + 'static {
    /// Cache AI computation results with usage metrics
    async fn ai_store_inference(
        &self,
        query_key: String,
        result: serde_json::Value,
        tokens_used: Option<u32>,
        ttl: Option<Duration>,
    ) -> IDEResult<()> {
        let mut cache_entry = CacheEntry::new(result.clone());
        if let Some(tokens) = tokens_used {
            cache_entry
                .metadata
                .insert("tokens_used".to_string(), tokens.to_string());
        }
        cache_entry
            .metadata
            .insert("cache_type".to_string(), "ai_inference".to_string());

        self.insert(query_key, result, ttl).await
    }

    /// Cache similarity computations
    async fn ai_store_similarity(
        &self,
        pattern_key: String,
        similarities: serde_json::Value,
        ttl: Option<Duration>,
    ) -> IDEResult<()> {
        let mut cache_entry = CacheEntry::new(similarities.clone());
        cache_entry
            .metadata
            .insert("cache_type".to_string(), "similarity".to_string());

        self.insert(pattern_key, similarities, ttl).await
    }
}

// Auto-implement for any AI cache
impl<C> AiCacheExt for C where C: Cache<String, serde_json::Value> + Send + Sync + 'static {}

/// Extension trait for collaborative caching patterns
#[async_trait]
pub trait CollaborativeCacheExt<K, V>: Cache<K, V> + Send + Sync + 'static
where
    K: Send + Sync + Clone + Hash + Eq + serde::Serialize + 'static,
    V: Send + Sync + Clone + serde::Serialize + 'static,
{
    /// Store a collaborative cache entry with session scope
    async fn collab_store(&self, session_id: &SessionId, key: K, value: V, ttl: Option<Duration>) -> IDEResult<()> {
        // Add session prefix to key for isolation
        let prefixed_key = format!(
            "session:{}:{}",
            session_id.as_string(),
            serde_json::to_string(&key).unwrap_or_default()
        );
        let hashed_key = sha256::digest(prefixed_key);

        // Store with collaborative metadata
        let mut cache_entry = CacheEntry::new(value);
        cache_entry
            .metadata
            .insert("session_id".to_string(), session_id.as_string());
        cache_entry
            .metadata
            .insert("cache_type".to_string(), "collaborative".to_string());

        // For now, delegate to base implementation
        // In a real implementation, this would use session-aware storage
        self.insert(key, cache_entry.value, ttl).await
    }

    /// Retrieve a collaborative cache entry
    async fn collab_retrieve(&self, session_id: &SessionId, key: &K) -> IDEResult<Option<V>> {
        self.get(key).await
    }

    /// Store user-specific cache entry
    async fn user_store(&self, user_id: &UserId, key: K, value: V, ttl: Option<Duration>) -> IDEResult<()> {
        let prefixed_key = format!(
            "user:{}:{}",
            user_id.as_str(),
            serde_json::to_string(&key).unwrap_or_default()
        );
        let hashed_key = sha256::digest(prefixed_key);

        let mut cache_entry = CacheEntry::new(value);
        cache_entry
            .metadata
            .insert("user_id".to_string(), user_id.as_str().to_string());
        cache_entry
            .metadata
            .insert("cache_type".to_string(), "user_specific".to_string());

        self.insert(key, cache_entry.value, ttl).await
    }

    /// Retrieve user-specific cache entry
    async fn user_retrieve(&self, user_id: &UserId, key: &K) -> IDEResult<Option<V>> {
        self.get(key).await
    }

    /// Share cache entry across session users
    async fn share_entry(&self, session_id: &SessionId, key: &K, shared_key: K) -> IDEResult<()> {
        if let Some(value) = self.get(key).await? {
            let mut cache_entry = CacheEntry::new(value);
            cache_entry
                .metadata
                .insert("session_id".to_string(), session_id.as_string());
            cache_entry
                .metadata
                .insert("shared".to_string(), "true".to_string());
            cache_entry
                .metadata
                .insert("cache_type".to_string(), "shared".to_string());

            // Store with shared key
            self.insert(shared_key, cache_entry.value, None).await
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Entry not found").into())
        }
    }
}

// Auto-implement for any collaborative cache
impl<C, K, V> CollaborativeCacheExt<K, V> for C
where
    C: Cache<K, V> + Send + Sync + 'static,
    K: Send + Sync + Clone + Hash + Eq + serde::Serialize + 'static,
    V: Send + Sync + Clone + serde::Serialize + 'static,
{
}

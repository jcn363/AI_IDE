//! Caching layer for dependency graph operations with compression support

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "compression")]
use bincode;
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn};
#[cfg(feature = "compression")]
use zstd;

use crate::error::*;
use crate::graph::*;

/// Compressed data wrapper for large cache entries
#[cfg(feature = "compression")]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CompressedEntry {
    data:            Vec<u8>,
    original_size:   usize,
    compressed_size: usize,
}

#[cfg(feature = "compression")]
impl CompressedEntry {
    fn compress<T: Serialize>(data: &T) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let serialized = bincode::serialize(data)?;
        let original_size = serialized.len();
        let compressed = zstd::encode_all(&serialized[..], 3)?;
        let compressed_size = compressed.len();

        Ok(Self {
            data: compressed,
            original_size,
            compressed_size,
        })
    }

    fn decompress<T: serde::de::DeserializeOwned>(self) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        let decompressed = zstd::decode_all(&self.data[..])?;
        let result = bincode::deserialize(&decompressed)?;
        Ok(result)
    }

    fn should_compress(size_kb: usize, threshold_kb: usize) -> bool {
        size_kb >= threshold_kb
    }
}

/// Cache entry for package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadataEntry {
    pub name:         String,
    pub version:      String,
    pub metadata:     serde_json::Value,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub source:       String,
}

/// Cache entry for dependency tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyTreeEntry {
    pub root_package: String,
    pub tree:         Vec<serde_json::Value>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub hash:         u64, // Hash of the source manifest for cache validation
}

/// Cache key for dependency resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyResolutionKey {
    pub root_package:        String,
    pub resolution_strategy: String,
    pub constraints:         HashMap<String, String>,
}

impl Hash for DependencyResolutionKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.root_package.hash(state);
        self.resolution_strategy.hash(state);
        // Hash the constraints map in a deterministic order
        let mut keys: Vec<_> = self.constraints.keys().collect();
        keys.sort();
        for key in keys {
            key.hash(state);
            self.constraints[key].hash(state);
        }
    }
}

impl PartialEq for DependencyResolutionKey {
    fn eq(&self, other: &Self) -> bool {
        self.root_package == other.root_package
            && self.resolution_strategy == other.resolution_strategy
            && self.constraints == other.constraints
    }
}

impl Eq for DependencyResolutionKey {}

/// Cache entry for dependency resolution results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyResolutionEntry {
    pub resolved_versions: HashMap<String, String>,
    pub conflicts:         Vec<String>,
    pub last_updated:      chrono::DateTime<chrono::Utc>,
}

/// Main cache structure for dependency graph operations with compression support
#[derive(Clone)]
pub struct GraphCache {
    package_metadata_cache: Cache<String, PackageMetadataEntry>,
    dependency_tree_cache:  Cache<String, DependencyTreeEntry>,
    #[cfg(feature = "compression")]
    compressed_tree_cache:  Option<Cache<String, CompressedEntry>>,
    resolution_cache:       Cache<DependencyResolutionKey, DependencyResolutionEntry>,
    config:                 CacheConfig,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub package_metadata_ttl: Duration,
    pub dependency_tree_ttl:  Duration,
    pub resolution_ttl:       Duration,
    pub max_capacity:         u64,
    pub enable_compression:   bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            package_metadata_ttl: Duration::from_secs(3600), // 1 hour
            dependency_tree_ttl:  Duration::from_secs(1800), // 30 minutes
            resolution_ttl:       Duration::from_secs(600),  // 10 minutes
            max_capacity:         10000,
            enable_compression:   true,
        }
    }
}

impl GraphCache {
    /// Create a new cache instance with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new cache instance with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        let package_metadata_cache = Cache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(config.package_metadata_ttl)
            .build();

        let dependency_tree_cache = Cache::builder()
            .max_capacity(config.max_capacity / 10) // Smaller capacity for trees
            .time_to_live(config.dependency_tree_ttl)
            .build();

        #[cfg(feature = "compression")]
        let compressed_tree_cache = if config.enable_compression {
            Some(
                Cache::builder()
                    .max_capacity(config.max_capacity / 20) // Even smaller for compressed
                    .time_to_live(config.dependency_tree_ttl)
                    .build(),
            )
        } else {
            None
        };

        let resolution_cache = Cache::builder()
            .max_capacity(config.max_capacity / 5) // Medium capacity for resolutions
            .time_to_live(config.resolution_ttl)
            .build();

        Self {
            package_metadata_cache,
            dependency_tree_cache,
            #[cfg(feature = "compression")]
            compressed_tree_cache,
            resolution_cache,
            config,
        }
    }

    /// Get cached package metadata
    pub async fn get_package_metadata(&self, package_name: &str) -> Option<PackageMetadataEntry> {
        self.package_metadata_cache.get(package_name).await
    }

    /// Put package metadata in cache
    pub async fn put_package_metadata(&self, package_name: String, entry: PackageMetadataEntry) {
        info!("Caching package metadata for {}", package_name);
        self.package_metadata_cache
            .insert(package_name, entry)
            .await;
    }

    /// Get cached dependency tree with compression support
    pub async fn get_dependency_tree(&self, root_package: &str) -> Option<DependencyTreeEntry> {
        // First check regular cache
        if let Some(entry) = self.dependency_tree_cache.get(root_package).await {
            return Some(entry);
        }

        // Check compressed cache if available
        #[cfg(feature = "compression")]
        if let Some(compressed_cache) = &self.compressed_tree_cache {
            let compressed_key = format!("compressed:{}", root_package);
            if let Some(compressed_entry) = compressed_cache.get(&compressed_key).await {
                match compressed_entry.decompress::<DependencyTreeEntry>() {
                    Ok(entry) => {
                        info!("Decompressed dependency tree for {}", root_package);
                        return Some(entry);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to decompress dependency tree for {}: {}",
                            root_package, e
                        );
                        // Remove corrupted entry
                        compressed_cache.invalidate(&compressed_key).await;
                    }
                }
            }
        }

        None
    }

    /// Put dependency tree in cache with hash validation and optional compression
    pub async fn put_dependency_tree(&self, root_package: String, mut entry: DependencyTreeEntry) {
        // Generate hash of the tree data for cache validation
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        entry.tree.hash(&mut hasher);
        entry.hash = hasher.finish();

        // Check if compression should be used for large trees
        let tree_size_kb = (serde_json::to_string(&entry.tree).unwrap_or_default().len() / 1024) as usize;

        #[cfg(feature = "compression")]
        if self.config.enable_compression && CompressedEntry::should_compress(tree_size_kb, 50) {
            if let Some(compressed_cache) = &self.compressed_tree_cache {
                match CompressedEntry::compress(&entry) {
                    Ok(compressed) => {
                        info!(
                            "Compressing and caching dependency tree for {} (hash: {}, size: {}KB, ratio: {:.2})",
                            root_package,
                            entry.hash,
                            tree_size_kb,
                            compressed.compression_ratio()
                        );
                        compressed_cache
                            .insert(format!("compressed:{}", root_package), compressed)
                            .await;
                        return;
                    }
                    Err(e) => {
                        warn!(
                            "Failed to compress dependency tree for {}: {}",
                            root_package, e
                        );
                        // Fall back to uncompressed cache
                    }
                }
            }
        }

        info!(
            "Caching dependency tree for {} (hash: {}, size: {}KB)",
            root_package, entry.hash, tree_size_kb
        );
        self.dependency_tree_cache.insert(root_package, entry).await;
    }

    /// Validate if dependency tree is still valid based on hash
    pub async fn validate_dependency_tree(&self, root_package: &str, expected_hash: u64) -> bool {
        if let Some(entry) = self.dependency_tree_cache.get(root_package).await {
            entry.hash == expected_hash
        } else {
            false
        }
    }

    /// Get cached resolution result
    pub async fn get_resolution(&self, key: &DependencyResolutionKey) -> Option<DependencyResolutionEntry> {
        self.resolution_cache.get(key).await
    }

    /// Put resolution result in cache
    pub async fn put_resolution(&self, key: DependencyResolutionKey, entry: DependencyResolutionEntry) {
        info!("Caching resolution for {}", key.root_package);
        self.resolution_cache.insert(key, entry).await;
    }

    /// Invalidate all entries related to a package
    pub async fn invalidate_package(&self, package_name: &str) {
        warn!("Invalidating cache entries for {}", package_name);

        // Remove from package metadata cache
        self.package_metadata_cache.invalidate(package_name).await;

        // Remove from dependency tree cache (as root)
        self.dependency_tree_cache.invalidate(package_name).await;

        // Remove from resolution cache for entries that reference this package
        // Note: This is implemented via cache invalidation by key pattern in Moka v0.12+
        // For now, we'll invalidate all resolution cache entries that might be affected
        // In production, you might want to implement a more sophisticated invalidation strategy
        self.resolution_cache.invalidate_all();
        self.resolution_cache.run_pending_tasks();
    }

    /// Clear all caches including compressed caches
    pub async fn clear_all(&self) {
        warn!("Clearing all caches");
        self.package_metadata_cache.invalidate_all();
        self.dependency_tree_cache.invalidate_all();

        #[cfg(feature = "compression")]
        if let Some(compressed_cache) = &self.compressed_tree_cache {
            compressed_cache.invalidate_all();
        }

        self.resolution_cache.invalidate_all();
        self.resolution_cache.run_pending_tasks();
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        CacheStats {
            package_metadata_entries: self.package_metadata_cache.entry_count(),
            dependency_tree_entries:  self.dependency_tree_cache.entry_count(),
            resolution_entries:       self.resolution_cache.entry_count(),
        }
    }

    /// Set custom TTL for package metadata
    pub fn set_package_metadata_ttl(&mut self, ttl: Duration) {
        self.config.package_metadata_ttl = ttl;
        // Note: In a real implementation, you might need to rebuild the cache
    }

    /// Set custom TTL for dependency trees
    pub fn set_dependency_tree_ttl(&mut self, ttl: Duration) {
        self.config.dependency_tree_ttl = ttl;
    }

    /// Set custom TTL for resolutions
    pub fn set_resolution_ttl(&mut self, ttl: Duration) {
        self.config.resolution_ttl = ttl;
    }
}

/// Thread-safe cached dependency graph service
#[derive(Clone)]
pub struct CachedDependencyGraph {
    graph: Arc<RwLock<DependencyGraph>>,
    cache: Arc<GraphCache>,
}

impl CachedDependencyGraph {
    pub fn new(graph: Arc<RwLock<DependencyGraph>>, cache: Arc<GraphCache>) -> Self {
        Self { graph, cache }
    }

    /// Get package metadata with caching
    pub async fn get_package_metadata_cached(&self, package_name: &str) -> DependencyResult<PackageMetadataEntry> {
        if let Some(metadata) = self.cache.get_package_metadata(package_name).await {
            info!("Cache hit for package metadata: {}", package_name);
            return Ok(metadata);
        }

        info!("Cache miss for package metadata: {}", package_name);
        // In a real implementation, this would fetch from a registry
        Err(DependencyError::NetworkError("Cache miss".to_string()))
    }

    /// Get dependency tree with caching
    pub async fn get_dependency_tree_cached(&self, root_package: &str) -> DependencyResult<DependencyTreeEntry> {
        if let Some(tree) = self.cache.get_dependency_tree(root_package).await {
            info!("Cache hit for dependency tree: {}", root_package);
            return Ok(tree);
        }

        info!("Cache miss for dependency tree: {}", root_package);
        Err(DependencyError::NetworkError("Cache miss".to_string()))
    }

    /// Get resolution with caching
    pub async fn get_resolution_cached(&self, key: &DependencyResolutionKey) -> Option<DependencyResolutionEntry> {
        self.cache.get_resolution(key).await
    }

    /// Cache warm-up operation
    pub async fn warmup_cache(&self) -> DependencyResult<()> {
        info!("Starting cache warm-up");

        let graph = self.graph.read().await;

        // Warm up package metadata for all packages
        for package in graph.get_all_packages() {
            let entry = PackageMetadataEntry {
                name:         package.name.clone(),
                version:      package.version.clone().unwrap_or_default(),
                metadata:     serde_json::json!({
                    "name": package.name,
                    "description": package.description,
                    "license": package.license
                }),
                last_updated: chrono::Utc::now(),
                source:       "crates.io".to_string(),
            };

            self.cache
                .put_package_metadata(package.name.clone(), entry)
                .await;
        }

        info!("Cache warm-up completed");
        Ok(())
    }

    /// Background cache maintenance
    pub async fn run_maintenance(&self) {
        info!("Running cache maintenance");

        // Run pending invalidation tasks
        self.cache.resolution_cache.run_pending_tasks();

        // Log current statistics
        let stats = self.cache.get_stats().await;
        info!("Cache stats: {:?}", stats);

        // Periodic cleanup could be implemented here
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub package_metadata_entries: u64,
    pub dependency_tree_entries:  u64,
    pub resolution_entries:       u64,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache Stats {{ packages: {}, trees: {}, resolutions: {} }}",
            self.package_metadata_entries, self.dependency_tree_entries, self.resolution_entries
        )
    }
}

/// Cache-backed dependency resolver
#[derive(Clone)]
pub struct CachedDependencyResolver {
    resolver: Arc<super::resolver::DependencyResolver>,
    cache:    Arc<GraphCache>,
}

impl CachedDependencyResolver {
    pub fn new(resolver: Arc<super::resolver::DependencyResolver>, cache: Arc<GraphCache>) -> Self {
        Self { resolver, cache }
    }

    /// Resolve dependencies with caching
    pub async fn resolve_with_cache(
        &self,
        key: DependencyResolutionKey,
    ) -> DependencyResult<DependencyResolutionEntry> {
        if let Some(cached_result) = self.cache.get_resolution(&key).await {
            info!("Using cached resolution for {}", key.root_package);
            return Ok(cached_result);
        }

        info!("Computing new resolution for {}", key.root_package);

        // Perform the resolution
        let resolved_versions = self.resolver.resolve_conflicts().await?;
        let conflicts = self
            .resolver
            .graph
            .read()
            .await
            .get_cycles()
            .into_iter()
            .flatten()
            .collect();

        let entry = DependencyResolutionEntry {
            resolved_versions,
            conflicts,
            last_updated: chrono::Utc::now(),
        };

        // Cache the result
        self.cache.put_resolution(key, entry.clone()).await;

        Ok(entry)
    }
}

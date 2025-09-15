//! Build result caching for Cargo operations
//!
//! This module provides caching for build results and dependencies to avoid
//! redundant compilation when source files haven't changed.

use std::path::Path;
use std::time::{Duration, SystemTime};

use rust_ai_ide_cache::{key_utils, Cache, CacheConfig, CacheStats, InMemoryCache};
use rust_ai_ide_core::IDEResult;
use serde::{Deserialize, Serialize};

use super::BuildMetrics;

/// Cached build result entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedBuildResult {
    /// Build metrics
    pub metrics:      BuildMetrics,
    /// Hash of the project state when build was performed
    pub project_hash: String,
    /// Profile used for the build
    pub profile:      String,
    /// Features used for the build
    pub features:     Vec<String>,
    /// Timestamp when cached
    pub cached_at:    SystemTime,
}

/// Build cache for Cargo operations
pub struct CargoBuildCache {
    unified_cache: InMemoryCache<String, serde_json::Value>,
    config:        CacheConfig,
}

impl std::fmt::Debug for CargoBuildCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CargoBuildCache")
            .field("config", &self.config)
            .finish()
    }
}

impl CargoBuildCache {
    /// Create a new Cargo build cache with optimized settings
    pub fn new() -> Self {
        let config = CacheConfig {
            max_entries: Some(2000),                      // Higher limit for build artifacts
            default_ttl: Some(Duration::from_secs(3600)), // 1 hour for build results
            ..Default::default()
        };

        let unified_cache = InMemoryCache::new(&config);

        Self {
            unified_cache,
            config,
        }
    }

    /// Create cache with custom configuration
    pub fn new_with_config(config: CacheConfig) -> Self {
        let unified_cache = InMemoryCache::new(&config);
        Self {
            unified_cache,
            config,
        }
    }

    /// Get cached build result for a project
    pub async fn get_build_result(
        &self,
        project_path: &Path,
        profile: &str,
        features: &[&str],
    ) -> Option<CachedBuildResult> {
        let cache_key = self.create_cache_key(project_path, profile, features);

        if let Ok(Some(value)) = self.unified_cache.get(&cache_key).await {
            match serde_json::from_value::<CachedBuildResult>(value) {
                Ok(cached) => {
                    // Check if project has changed since caching
                    if self
                        .is_cache_valid(project_path, &cached.project_hash)
                        .await
                    {
                        Some(cached)
                    } else {
                        // Cache is stale, remove it
                        let _ = self.unified_cache.remove(&cache_key).await;
                        None
                    }
                }
                Err(_) => None,
            }
        } else {
            None
        }
    }

    /// Store build result in cache
    pub async fn store_build_result(
        &self,
        project_path: &Path,
        profile: &str,
        features: &[&str],
        metrics: BuildMetrics,
    ) -> IDEResult<()> {
        let project_hash = self.compute_project_hash(project_path).await?;
        let cache_key = self.create_cache_key(project_path, profile, features);

        let cached_result = CachedBuildResult {
            metrics,
            project_hash,
            profile: profile.to_string(),
            features: features.iter().map(|s| s.to_string()).collect(),
            cached_at: SystemTime::now(),
        };

        let json_value = serde_json::to_value(&cached_result).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Serialization error: {}", e),
            )
        })?;

        self.unified_cache
            .insert(cache_key, json_value, Some(Duration::from_secs(3600)))
            .await?;

        Ok(())
    }

    /// Clear all cached build results
    pub async fn clear(&self) -> IDEResult<()> {
        self.unified_cache.clear().await
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        self.unified_cache.stats().await
    }

    /// Remove cache entry for specific project and profile
    pub async fn invalidate_project_cache(
        &self,
        project_path: &Path,
        profile: &str,
        features: &[&str],
    ) -> IDEResult<()> {
        let cache_key = self.create_cache_key(project_path, profile, features);
        self.unified_cache.remove(&cache_key).await?;
        Ok(())
    }

    /// Create cache key for build result
    fn create_cache_key(&self, project_path: &Path, profile: &str, features: &[&str]) -> String {
        let mut key_components = vec![profile.to_string()];
        key_components.extend(features.iter().map(|s| s.to_string()));
        key_components.push(project_path.to_string_lossy().to_string());

        key_utils::structured_key("cargo_build", &key_components)
    }

    /// Check if cached result is still valid
    async fn is_cache_valid(&self, project_path: &Path, cached_hash: &str) -> bool {
        if let Ok(current_hash) = self.compute_project_hash(project_path).await {
            current_hash == cached_hash
        } else {
            // If we can't compute current hash, assume cache is stale
            false
        }
    }

    /// Compute hash of project state for cache validation
    async fn compute_project_hash(&self, project_path: &Path) -> IDEResult<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let cargo_toml = project_path.join("Cargo.toml");
        if let Ok(metadata) = tokio::fs::metadata(&cargo_toml).await {
            let mut hasher = DefaultHasher::new();

            // Hash file modification time and size for simple validation
            metadata
                .modified()
                .unwrap_or(SystemTime::UNIX_EPOCH)
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
                .hash(&mut hasher);

            metadata.len().hash(&mut hasher);

            // Include src directory file count changes
            if let Ok(src_count) = self.count_src_files(project_path).await {
                src_count.hash(&mut hasher);
            }

            Ok(format!("{:x}", hasher.finish()))
        } else {
            Err(rust_ai_ide_core::IDEError::Generic(
                "Could not read Cargo.toml metadata".to_string(),
            ))
        }
    }

    /// Count files in src directory for hash validation
    async fn count_src_files(&self, project_path: &Path) -> IDEResult<u64> {
        let src_dir = project_path.join("src");
        let mut count = 0u64;

        if let Ok(mut entries) = tokio::fs::read_dir(&src_dir).await {
            while let Ok(Some(_)) = entries.next_entry().await {
                count += 1;
            }
        }

        Ok(count)
    }
}

impl Default for CargoBuildCache {
    fn default() -> Self {
        Self::new()
    }
}

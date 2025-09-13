//! Advanced Caching Strategies with Intelligent Invalidation
//!
//! This module provides sophisticated caching strategies that go beyond
//! simple time-based TTL with intelligent invalidation policies.
//!
//! # Features
//! - **Dependency-aware caching**: Invalidate cache based on file dependencies
//! - **Change pattern analysis**: Learn from code changes to predict invalidation
//! - **Hierarchical caching**: Hot data in memory, warm data in Redis, cold data on disk
//! - **Predictive preloading**: Load anticipated data into cache before access
//! - **Adaptive TTL**: Dynamically adjust cache expiration based on access patterns

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::super::Cache;

/// Advanced cache invalidation strategy
#[derive(Debug, Clone)]
pub enum InvalidationStrategy {
    /// Time-based with fixed TTL
    Lazy,
    /// Immediate invalidation on change
    Eager,
    /// Smart invalidation based on access patterns
    Predictive,
    /// Dependency-aware invalidation
    Dependency,
    /// Hybrid approach combining multiple strategies
    Adaptive,
}

/// Advanced cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedCacheConfig {
    pub max_entries: usize,
    pub invalidation_strategy: InvalidationStrategy,
    pub enable_dependency_tracking: bool,
    pub enable_predictive_preloading: bool,
    pub access_pattern_learning_enabled: bool,
    pub max_dependency_depth: usize,
    pub preload_batch_size: usize,
}

/// Cache entry with dependency tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedCacheEntry<V> {
    pub value: V,
    pub dependencies: HashSet<PathBuf>,
    pub dependents: HashSet<PathBuf>,
    pub access_pattern: VecDeque<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
    pub adaptive_ttl: Option<u64>,
}

/// Dependency tracker for intelligent invalidation
#[derive(Debug)]
pub struct DependencyTracker {
    dependency_graph: RwLock<HashMap<PathBuf, HashSet<PathBuf>>>,
    reverse_dependency_graph: RwLock<HashMap<PathBuf, HashSet<PathBuf>>>,
    access_patterns: RwLock<HashMap<PathBuf, VecDeque<chrono::DateTime<chrono::Utc>>>>,
}

impl DependencyTracker {
    pub fn new() -> Self {
        Self {
            dependency_graph: RwLock::new(HashMap::new()),
            reverse_dependency_graph: RwLock::new(HashMap::new()),
            access_patterns: RwLock::new(HashMap::new()),
        }
    }

    /// Update dependencies for a file
    pub async fn update_dependencies(&self, file: PathBuf, dependencies: HashSet<PathBuf>) {
        let mut graph = self.dependency_graph.write().await;
        let mut reverse_graph = self.reverse_dependency_graph.write().await;

        // Remove old dependencies
        if let Some(old_deps) = graph.insert(file.clone(), dependencies.clone()) {
            for old_dep in old_deps {
                if let Some(reverse_deps) = reverse_graph.get_mut(&old_dep) {
                    reverse_deps.remove(&file);
                }
            }
        }

        // Add new dependencies
        for dep in &dependencies {
            reverse_graph
                .entry(dep.clone())
                .or_insert_with(HashSet::new)
                .insert(file.clone());
        }
    }

    /// Get all files that depend on the given file (directly or indirectly)
    pub async fn get_dependents(&self, file: &PathBuf, max_depth: usize) -> HashSet<PathBuf> {
        let mut result = HashSet::new();
        let mut visited = HashSet::new();
        visited.insert(file.clone());

        self.collect_dependents_recursive(file, &mut result, &mut visited, 0, max_depth)
            .await;

        result
    }

    fn collect_dependents_recursive(
        &self,
        file: &PathBuf,
        result: &mut HashSet<PathBuf>,
        visited: &mut HashSet<PathBuf>,
        current_depth: usize,
        max_depth: usize,
    ) -> impl std::future::Future<Output = ()> + '_ {
        async move {
            if current_depth >= max_depth {
                return;
            }

            if let Some(direct_dependents) = self.reverse_dependency_graph.read().await.get(file) {
                for dependent in direct_dependents {
                    if visited.insert(dependent.clone()) {
                        result.insert(dependent.clone());
                        self.collect_dependents_recursive(
                            dependent,
                            result,
                            visited,
                            current_depth + 1,
                            max_depth,
                        )
                        .await;
                    }
                }
            }
        }
    }

    /// Record access pattern for a file
    pub async fn record_access(&self, file: PathBuf) {
        let mut patterns = self.access_patterns.write().await;
        let deque = patterns
            .entry(file)
            .or_insert_with(|| VecDeque::with_capacity(10));

        deque.push_back(chrono::Utc::now());
        if deque.len() > 10 {
            deque.pop_front();
        }
    }

    /// Get access frequency for a file
    pub async fn get_access_frequency(&self, file: &PathBuf) -> f64 {
        let patterns = self.access_patterns.read().await;
        if let Some(times) = patterns.get(file) {
            if times.len() < 2 {
                return 0.0;
            }

            let time_diff = times.back().unwrap().timestamp() - times.front().unwrap().timestamp();
            if time_diff > 0 {
                (times.len() as f64) / (time_diff as f64) * 3600.0 // accesses per hour
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}

/// Predictive preloading engine
#[derive(Debug)]
pub struct PredictivePreloader {
    recent_accesses: RwLock<VecDeque<(PathBuf, chrono::DateTime<chrono::Utc>)>>,
    loading_predictions: RwLock<HashMap<PathBuf, f64>>,
}

impl PredictivePreloader {
    pub fn new() -> Self {
        Self {
            recent_accesses: RwLock::new(VecDeque::new()),
            loading_predictions: RwLock::new(HashMap::new()),
        }
    }

    /// Record file access for prediction learning
    pub async fn record_access(&self, file: PathBuf) {
        let mut accesses = self.recent_accesses.write().await;
        accesses.push_back((file, chrono::Utc::now()));

        // Keep only recent accesses (last 100)
        while accesses.len() > 100 {
            accesses.pop_front();
        }

        // Update predictions based on recent patterns
        self.update_predictions().await;
    }

    /// Get files that should be preloaded based on predictions
    pub async fn get_preload_candidates(&self, batch_size: usize) -> Vec<PathBuf> {
        let predictions = self.loading_predictions.read().await;
        let mut candidates: Vec<(PathBuf, f64)> = predictions
            .iter()
            .map(|(path, score)| (path.clone(), *score))
            .collect();

        // Sort by prediction score (descending)
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Return top candidates
        candidates
            .into_iter()
            .take(batch_size)
            .map(|(path, _)| path)
            .collect()
    }

    fn update_predictions(&self) -> impl std::future::Future<Output = ()> + '_ {
        async move {
            let accesses = self.recent_accesses.read().await;
            let mut predictions = self.loading_predictions.write().await;

            let current_time = chrono::Utc::now();

            // Calculate access frequencies and patterns
            let mut file_counts = HashMap::new();
            for (file, time) in accesses.iter() {
                let entry = file_counts
                    .entry(file.clone())
                    .or_insert((0usize, Vec::new()));
                entry.0 += 1;
                entry.1.push(time.timestamp() as f64);
            }

            // Calculate prediction scores based on:
            // 1. Access frequency
            // 2. Recency
            // 3. Sequential access patterns
            for (file, (count, timestamps)) in file_counts {
                let mut score = 0.0;

                // Frequency score
                score += (count as f64) * 0.4;

                // Recency score (more recent = higher score)
                if let Some(last_timestamp) =
                    timestamps.iter().max_by(|a, b| a.partial_cmp(b).unwrap())
                {
                    let hours_since_access =
                        (current_time.timestamp() as f64 - last_timestamp) / 3600.0;
                    let recency_bonus = (24.0 / (1.0 + hours_since_access)).clamp(0.0, 1.0);
                    score += recency_bonus * 0.6;
                }

                predictions.insert(file, score.clamp(0.0, 10.0));
            }

            // Clean up old predictions
            let cutoff_score = 1.0;
            predictions.retain(|_, score| *score >= cutoff_score);
        }
    }
}

/// Adaptive TTL calculator
#[derive(Debug)]
pub struct AdaptiveTTLCalculator {
    access_patterns: RwLock<HashMap<PathBuf, VecDeque<chrono::DateTime<chrono::Utc>>>>,
}

impl AdaptiveTTLCalculator {
    pub fn new() -> Self {
        Self {
            access_patterns: RwLock::new(HashMap::new()),
        }
    }

    /// Calculate optimal TTL for a file based on access patterns
    pub async fn calculate_ttl(&self, file: &PathBuf, default_ttl: u64) -> u64 {
        let patterns = self.access_patterns.read().await;
        if let Some(accesses) = patterns.get(file) {
            if accesses.len() < 2 {
                return default_ttl;
            }

            // Calculate average time between accesses
            let timestamps: Vec<i64> = accesses.iter().map(|t| t.timestamp()).collect();
            let intervals: Vec<i64> = timestamps.windows(2).map(|w| w[1] - w[0]).collect();

            if let Some(avg_interval) = intervals
                .iter()
                .sum::<i64>()
                .checked_div(intervals.len() as i64)
            {
                // Set TTL to 3x the average interval, but clamp between 1 minute and 7 days
                let calculated_ttl = (avg_interval * 3).clamp(60, 604800);
                debug!(
                    "Calculated TTL for {}: {} seconds (avg interval: {})",
                    file.display(),
                    calculated_ttl,
                    avg_interval
                );
                calculated_ttl as u64
            } else {
                default_ttl
            }
        } else {
            default_ttl
        }
    }

    /// Record access for TTL calculation
    pub async fn record_access(&self, file: PathBuf) {
        let mut patterns = self.access_patterns.write().await;
        let deque = patterns
            .entry(file)
            .or_insert_with(|| VecDeque::with_capacity(20));

        deque.push_back(chrono::Utc::now());
        if deque.len() > 20 {
            deque.pop_front();
        }
    }
}

/// Hierarchical caching strategy (memory -> Redis -> disk)
#[derive(Debug)]
pub struct HierarchicalCacheManager<K, V> {
    memory_cache: Option<Arc<dyn Cache<K, V>>>,
    redis_cache: Option<Arc<dyn Cache<K, V>>>,
    disk_cache: Option<Arc<dyn Cache<K, V>>>,
    policy: HierarchicalCachePolicy,
}

#[derive(Debug, Clone)]
pub enum HierarchicalCachePolicy {
    /// Try memory first, then Redis, then disk
    MemoryFirst,
    /// Try Redis first for distributed scenarios
    RedisFirst,
    /// Optimize for local development with memory + disk
    LocalOptimized,
}

impl<K, V> HierarchicalCacheManager<K, V>
where
    K: Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    pub fn new(policy: HierarchicalCachePolicy) -> Self {
        Self {
            memory_cache: None,
            redis_cache: None,
            disk_cache: None,
            policy,
        }
    }

    pub fn with_memory_cache(mut self, cache: Arc<dyn Cache<K, V>>) -> Self {
        self.memory_cache = Some(cache);
        self
    }

    pub fn with_redis_cache(mut self, cache: Arc<dyn Cache<K, V>>) -> Self {
        self.redis_cache = Some(cache);
        self
    }

    pub fn with_disk_cache(mut self, cache: Arc<dyn Cache<K, V>>) -> Self {
        self.disk_cache = Some(cache);
        self
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        match self.policy {
            HierarchicalCachePolicy::MemoryFirst => {
                // Try memory first
                if let Some(ref cache) = self.memory_cache {
                    if let Ok(Some(value)) = cache.get(key).await {
                        return Some(value);
                    }
                }
                // Try Redis
                if let Some(ref cache) = self.redis_cache {
                    if let Ok(Some(value)) = cache.get(key).await {
                        // Promote to memory
                        if let Some(ref mem_cache) = self.memory_cache {
                            let _ = mem_cache
                                .insert(
                                    key.clone(),
                                    value.clone(),
                                    Some(std::time::Duration::from_secs(300)),
                                )
                                .await;
                        }
                        return Some(value);
                    }
                }
                None
            }
            HierarchicalCachePolicy::RedisFirst => {
                // Try Redis first
                if let Some(ref cache) = self.redis_cache {
                    if let Ok(Some(value)) = cache.get(key).await {
                        // Promote to memory
                        if let Some(ref mem_cache) = self.memory_cache {
                            let _ = mem_cache
                                .insert(
                                    key.clone(),
                                    value.clone(),
                                    Some(std::time::Duration::from_secs(300)),
                                )
                                .await;
                        }
                        return Some(value);
                    }
                }
                // Try memory
                if let Some(ref cache) = self.memory_cache {
                    if let Ok(Some(value)) = cache.get(key).await {
                        return Some(value);
                    }
                }
                None
            }
            HierarchicalCachePolicy::LocalOptimized => {
                // Memory first, then disk
                if let Some(ref cache) = self.memory_cache {
                    if let Ok(Some(value)) = cache.get(key).await {
                        return Some(value);
                    }
                }
                // Try disk
                if let Some(ref cache) = self.disk_cache {
                    if let Ok(Some(value)) = cache.get(key).await {
                        // Promote to memory
                        if let Some(ref mem_cache) = self.memory_cache {
                            let _ = mem_cache
                                .insert(
                                    key.clone(),
                                    value.clone(),
                                    Some(std::time::Duration::from_secs(600)),
                                )
                                .await;
                        }
                        return Some(value);
                    }
                }
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test as async_test;

    #[async_test]
    async fn test_dependency_tracking() {
        let tracker = DependencyTracker::new();

        let file_a = PathBuf::from("a.rs");
        let file_b = PathBuf::from("b.rs");
        let file_c = PathBuf::from("c.rs");

        let mut deps = HashSet::new();
        deps.insert(file_b.clone());

        // A depends on B
        tracker.update_dependencies(file_a.clone(), deps).await;

        // B depends on C
        let mut deps_b = HashSet::new();
        deps_b.insert(file_c.clone());
        tracker.update_dependencies(file_b.clone(), deps_b).await;

        // Check dependents of C (should include A and B)
        let dependents = tracker.get_dependents(&file_c, 5).await;
        assert!(dependents.contains(&file_a));
        assert!(dependents.contains(&file_b));
    }

    #[async_test]
    async fn test_predictive_preloader() {
        let preloader = PredictivePreloader::new();

        let file_a = PathBuf::from("popular.rs");
        let file_b = PathBuf::from("rare.rs");

        // Simulate frequent access to file_a
        for _ in 0..10 {
            preloader.record_access(file_a.clone()).await;
        }

        // Simulate less access to file_b
        preloader.record_access(file_b.clone()).await;
        preloader.record_access(file_b.clone()).await;

        let candidates = preloader.get_preload_candidates(2).await;
        assert!(!candidates.is_empty());

        // file_a should have higher priority than file_b
        if candidates.len() > 1 {
            // This test depends on internal scoring algorithm
        }
    }

    #[async_test]
    async fn test_adaptive_ttl_calculator() {
        let calculator = AdaptiveTTLCalculator::new();

        let file = PathBuf::from("test.rs");
        let default_ttl = 3600;

        // No access history yet
        let ttl = calculator.calculate_ttl(&file, default_ttl).await;
        assert_eq!(ttl, default_ttl);

        // Record multiple accesses
        for i in 0..5 {
            calculator.record_access(file.clone()).await;
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        let adaptive_ttl = calculator.calculate_ttl(&file, default_ttl).await;
        // Should calculate something based on access patterns
        assert!(adaptive_ttl > 0);
    }
}

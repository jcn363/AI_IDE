//! Memory pooling for frequently allocated objects

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use crate::{Poolable, LazyLoadingError, LazyResult};

/// Object pool for managing frequently allocated objects
pub struct ObjectPool<T: Poolable + Default> {
    available: VecDeque<Arc<Mutex<T>>>,
    max_size: usize,
    created_count: usize,
}

impl<T: Poolable + Default> ObjectPool<T> {
    /// Create a new object pool with the specified maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            available: VecDeque::new(),
            max_size,
            created_count: 0,
        }
    }

    /// Acquire an object from the pool
    pub async fn acquire(&mut self) -> LazyResult<Arc<Mutex<T>>> {
        if let Some(obj) = self.available.pop_front() {
            Ok(obj)
        } else if self.created_count < self.max_size {
            // Create a new object
            let obj = Arc::new(Mutex::new(T::default()));
            self.created_count += 1;
            Ok(obj)
        } else {
            Err(LazyLoadingError::memory_pool_exhausted(
                std::any::type_name::<T>(),
                1,
                self.available.len(),
            ))
        }
    }

    /// Release an object back to the pool
    pub async fn release(&mut self, obj: Arc<Mutex<T>>) -> LazyResult<()> {
        let mut guard = obj.lock().await;
        guard.reset();

        // Only add back to pool if we haven't exceeded max size
        if self.available.len() + self.created_count <= self.max_size {
            self.available.push_back(obj);
        }

        Ok(())
    }

    /// Get the current size of the pool
    pub fn size(&self) -> usize {
        self.available.len()
    }

    /// Get the total number of created objects
    pub fn created_count(&self) -> usize {
        self.created_count
    }

    /// Clear the pool
    pub async fn clear(&mut self) {
        self.available.clear();
        self.created_count = 0;
    }
}

/// Memory pool manager for coordinating multiple object pools
pub struct MemoryPoolManager {
    analysis_result_pool: Arc<RwLock<ObjectPool<AnalysisResult>>>,
    model_state_pool: Arc<RwLock<ObjectPool<ModelState>>>,
    total_memory_usage: Arc<RwLock<usize>>,
    max_memory_limit: usize,
}

impl MemoryPoolManager {
    /// Create a new memory pool manager
    pub fn new(analysis_pool_size: usize, model_pool_size: usize, max_memory: usize) -> Self {
        Self {
            analysis_result_pool: Arc::new(RwLock::new(ObjectPool::new(analysis_pool_size))),
            model_state_pool: Arc::new(RwLock::new(ObjectPool::new(model_pool_size))),
            total_memory_usage: Arc::new(RwLock::new(0)),
            max_memory_limit: max_memory,
        }
    }

    /// Acquire an analysis result from the pool
    pub async fn acquire_analysis_result(&self) -> LazyResult<Arc<Mutex<AnalysisResult>>> {
        self.check_memory_limit(std::mem::size_of::<AnalysisResult>()).await?;

        let mut pool = self.analysis_result_pool.write().await;
        let result = pool.acquire().await?;

        self.update_memory_usage(std::mem::size_of::<AnalysisResult>()).await;

        Ok(result)
    }

    /// Release an analysis result back to the pool
    pub async fn release_analysis_result(&self, obj: Arc<Mutex<AnalysisResult>>) -> LazyResult<()> {
        let mut pool = self.analysis_result_pool.write().await;
        pool.release(obj).await?;

        self.update_memory_usage(-(std::mem::size_of::<AnalysisResult>() as isize)).await;

        Ok(())
    }

    /// Acquire a model state from the pool
    pub async fn acquire_model_state(&self) -> LazyResult<Arc<Mutex<ModelState>>> {
        self.check_memory_limit(std::mem::size_of::<ModelState>()).await?;

        let mut pool = self.model_state_pool.write().await;
        let result = pool.acquire().await?;

        self.update_memory_usage(std::mem::size_of::<ModelState>()).await;

        Ok(result)
    }

    /// Release a model state back to the pool
    pub async fn release_model_state(&self, obj: Arc<Mutex<ModelState>>) -> LazyResult<()> {
        let mut pool = self.model_state_pool.write().await;
        pool.release(obj).await?;

        self.update_memory_usage(-(std::mem::size_of::<ModelState>() as isize)).await;

        Ok(())
    }

    /// Get current memory usage
    pub async fn get_memory_usage(&self) -> usize {
        *self.total_memory_usage.read().await
    }

    /// Get pool statistics
    pub async fn get_pool_stats(&self) -> PoolStats {
        let analysis_pool = self.analysis_result_pool.read().await;
        let model_pool = self.model_state_pool.read().await;
        let memory_usage = self.get_memory_usage().await;

        PoolStats {
            analysis_pool_size: analysis_pool.size(),
            analysis_pool_created: analysis_pool.created_count(),
            model_pool_size: model_pool.size(),
            model_pool_created: model_pool.created_count(),
            total_memory_usage: memory_usage,
            memory_limit: self.max_memory_limit,
        }
    }

    /// Check if adding an object would exceed memory limits
    async fn check_memory_limit(&self, additional_bytes: usize) -> LazyResult<()> {
        let current_usage = self.get_memory_usage().await;
        if current_usage + additional_bytes > self.max_memory_limit {
            return Err(LazyLoadingError::memory_limit_exceeded(
                current_usage + additional_bytes,
                self.max_memory_limit,
            ));
        }
        Ok(())
    }

    /// Update memory usage counter
    async fn update_memory_usage(&self, delta: isize) {
        let mut usage = self.total_memory_usage.write().await;
        if delta >= 0 {
            *usage += delta as usize;
        } else {
            *usage = usage.saturating_sub(delta.unsigned_abs());
        }
    }

    /// Clear all pools
    pub async fn clear_all_pools(&self) {
        let mut analysis_pool = self.analysis_result_pool.write().await;
        let mut model_pool = self.model_state_pool.write().await;

        analysis_pool.clear().await;
        model_pool.clear().await;

        let mut usage = self.total_memory_usage.write().await;
        *usage = 0;
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub analysis_pool_size: usize,
    pub analysis_pool_created: usize,
    pub model_pool_size: usize,
    pub model_pool_created: usize,
    pub total_memory_usage: usize,
    pub memory_limit: usize,
}

/// Analysis result object for pooling
#[derive(Debug, Default, Clone)]
pub struct AnalysisResult {
    pub file_path: String,
    pub analysis_type: String,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl Poolable for AnalysisResult {
    fn size_bytes(&self) -> usize {
        std::mem::size_of::<Self>() +
        self.file_path.capacity() +
        self.analysis_type.capacity() +
        self.issues.iter().map(|s| s.capacity()).sum::<usize>() +
        self.suggestions.iter().map(|s| s.capacity()).sum::<usize>() +
        self.metadata.iter().map(|(k, v)| k.capacity() + v.capacity()).sum::<usize>()
    }

    fn reset(&mut self) {
        self.file_path.clear();
        self.analysis_type.clear();
        self.issues.clear();
        self.suggestions.clear();
        self.metadata.clear();
    }
}

/// Model state object for pooling
#[derive(Debug, Default, Clone)]
pub struct ModelState {
    pub model_id: String,
    pub model_type: String,
    pub parameters: Vec<f32>,
    pub metadata: std::collections::HashMap<String, String>,
    pub last_used: std::time::Instant,
}

impl Poolable for ModelState {
    fn size_bytes(&self) -> usize {
        std::mem::size_of::<Self>() +
        self.model_id.capacity() +
        self.model_type.capacity() +
        self.parameters.len() * std::mem::size_of::<f32>() +
        self.metadata.iter().map(|(k, v)| k.capacity() + v.capacity()).sum::<usize>()
    }

    fn reset(&mut self) {
        self.model_id.clear();
        self.model_type.clear();
        self.parameters.clear();
        self.metadata.clear();
        self.last_used = std::time::Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_object_pool_basic() {
        let mut pool = ObjectPool::<AnalysisResult>::new(10);

        let obj = pool.acquire().await.unwrap();
        assert_eq!(pool.size(), 0);
        assert_eq!(pool.created_count(), 1);

        pool.release(obj).await.unwrap();
        assert_eq!(pool.size(), 1);
        assert_eq!(pool.created_count(), 1);
    }

    #[tokio::test]
    async fn test_object_pool_max_size() {
        let mut pool = ObjectPool::<AnalysisResult>::new(2);

        let obj1 = pool.acquire().await.unwrap();
        let obj2 = pool.acquire().await.unwrap();

        // Should fail to acquire third object
        let result = pool.acquire().await;
        assert!(matches!(result, Err(LazyLoadingError::MemoryPoolExhausted(_, _, _))));

        pool.release(obj1).await.unwrap();
        pool.release(obj2).await.unwrap();

        assert_eq!(pool.size(), 2);
    }

    #[tokio::test]
    async fn test_memory_pool_manager() {
        let manager = MemoryPoolManager::new(10, 5, 1000000);

        let analysis_obj = manager.acquire_analysis_result().await.unwrap();
        let model_obj = manager.acquire_model_state().await.unwrap();

        let stats = manager.get_pool_stats().await;
        assert_eq!(stats.analysis_pool_created, 1);
        assert_eq!(stats.model_pool_created, 1);
        assert!(stats.total_memory_usage > 0);

        manager.release_analysis_result(analysis_obj).await.unwrap();
        manager.release_model_state(model_obj).await.unwrap();

        let stats_after = manager.get_pool_stats().await;
        assert_eq!(stats_after.analysis_pool_size, 1);
        assert_eq!(stats_after.model_pool_size, 1);
    }

    #[tokio::test]
    async fn test_poolable_reset() {
        let mut result = AnalysisResult {
            file_path: "test.rs".to_string(),
            analysis_type: "syntax".to_string(),
            issues: vec!["error1".to_string()],
            suggestions: vec!["fix1".to_string()],
            metadata: [("key".to_string(), "value".to_string())].into_iter().collect(),
        };

        assert!(!result.file_path.is_empty());
        assert!(!result.issues.is_empty());

        result.reset();

        assert!(result.file_path.is_empty());
        assert!(result.issues.is_empty());
        assert!(result.suggestions.is_empty());
        assert!(result.metadata.is_empty());
    }
}
//! Cache implementations for different storage backends

pub mod distributed_work_stealing;
pub mod hybrid;
pub mod in_memory;
#[cfg(feature = "redis-backend")]
pub mod redis;

// Include implementation details
mod in_memory_impl;

#[cfg(feature = "redis-backend")]
pub use redis::{RedisCache, RedisConfig};

#[cfg(feature = "redis-cluster")]
pub use redis::RedisClusterCache;

// Re-exports for convenience
pub use distributed_work_stealing::{
    DistributedWorkStealingCache, HashPartitioner, Partitioner, PredictivePredictor,
    WorkStealingConfig,
};
pub use hybrid::HybridCache;
pub use in_memory::InMemoryCache;

// Type aliases for common configurations
pub type LocalCache<K, V> = InMemoryCache<K, V>;
pub type HybridInMemoryCache<K, V> = HybridCache<K, V>;

#[cfg(feature = "redis-backend")]
pub type DistributedCache<K, V> = RedisCache<K, V>;

#[cfg(feature = "redis-cluster")]
pub type ClusterCache<K, V> = RedisClusterCache<K, V>;

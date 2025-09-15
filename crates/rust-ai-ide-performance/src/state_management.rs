// Optimized state management crate for high-performance concurrent operations
// Implements lock contention reduction through RwLock, lock-free caching, and efficient
// communication

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use moka::future::Cache;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{timeout, Duration, Instant};

/// OptimizedStateManager replaces Arc<Mutex<T>> with Arc<RwLock<T>> for read-heavy workloads,
/// adding timeout protection and lock contention monitoring
pub struct OptimizedStateManager<T> {
    state:                Arc<RwLock<T>>,
    lock_timeout:         Duration,
    contention_counter:   Arc<AtomicU64>,
    memory_limit_bytes:   Option<u64>,
    current_memory_usage: Arc<AtomicU64>,
}

impl<T> OptimizedStateManager<T> {
    /// Create new state manager with initial state and lock timeout
    pub fn new(initial_state: T, lock_timeout_ms: u64, memory_limit_mb: Option<u64>) -> Self {
        Self {
            state:                Arc::new(RwLock::new(initial_state)),
            lock_timeout:         Duration::from_millis(lock_timeout_ms),
            contention_counter:   Arc::new(AtomicU64::new(0)),
            memory_limit_bytes:   memory_limit_mb.map(|mb| mb * 1024 * 1024),
            current_memory_usage: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Read access with timeout protection
    pub async fn read<F, R>(&self, f: F) -> Result<R, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce(&T) -> R,
    {
        let start = Instant::now();
        let result = timeout(self.lock_timeout, self.state.read()).await;

        match result {
            Ok(guard) => {
                let duration = start.elapsed();
                if duration.as_millis() > 1 {
                    self.contention_counter.fetch_add(1, Ordering::Relaxed);
                }
                Ok(f(&*guard))
            }
            Err(_) => {
                self.contention_counter.fetch_add(1, Ordering::Relaxed);
                Err("Lock timeout on read operation".into())
            }
        }
    }

    /// Write access with timeout protection and memory monitoring
    pub async fn write<F, R>(&self, f: F) -> Result<R, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce(&mut T) -> R,
    {
        let start = Instant::now();
        let result = timeout(self.lock_timeout, self.state.write()).await;

        match result {
            Ok(mut guard) => {
                let duration = start.elapsed();
                if duration.as_millis() > 1 {
                    self.contention_counter.fetch_add(1, Ordering::Relaxed);
                }

                // Check memory limits if configured
                if let Some(limit) = self.memory_limit_bytes {
                    let current = self.current_memory_usage.load(Ordering::Relaxed);
                    if current > limit {
                        return Err("Memory limit exceeded".into());
                    }
                }

                Ok(f(&mut *guard))
            }
            Err(_) => {
                self.contention_counter.fetch_add(1, Ordering::Relaxed);
                Err("Lock timeout on write operation".into())
            }
        }
    }

    /// Get current lock contention count
    pub fn get_contention_count(&self) -> u64 {
        self.contention_counter.load(Ordering::Relaxed)
    }

    /// Update memory usage (call periodically to track)
    pub fn update_memory_usage(&self, bytes: u64) {
        self.current_memory_usage.store(bytes, Ordering::Relaxed);
    }
}

/// LockFreeCache provides high-performance caching with lock-free operations
/// Uses Moka for TTL-based eviction and DashMap for concurrent access
pub struct LockFreeCache<K, V> {
    moka_cache:       Cache<K, V>,
    dashmap_fallback: DashMap<K, V>,
    hit_counter:      Arc<AtomicU64>,
    miss_counter:     Arc<AtomicU64>,
    eviction_counter: Arc<AtomicU64>,
}

impl<K, V> LockFreeCache<K, V>
where
    K: std::hash::Hash + Eq + Send + Sync + Clone + 'static,
    V: Send + Sync + Clone + 'static,
{
    /// Create new cache with capacity and TTL
    pub fn new(max_capacity: u64, ttl_secs: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_secs))
            .eviction_listener(|_key, _value, cause| {
                // Count evictions
                if let moka::notification::RemovalCause::Size = cause {
                    // eviction_counter would need to be passed, simplified here
                }
            })
            .build();

        Self {
            moka_cache:       cache,
            dashmap_fallback: DashMap::new(),
            hit_counter:      Arc::new(AtomicU64::new(0)),
            miss_counter:     Arc::new(AtomicU64::new(0)),
            eviction_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get value from cache
    pub async fn get(&self, key: &K) -> Option<V> {
        if let Some(value) = self.moka_cache.get(key).await {
            self.hit_counter.fetch_add(1, Ordering::Relaxed);
            Some(value)
        } else if let Some(value) = self.dashmap_fallback.get(key) {
            self.hit_counter.fetch_add(1, Ordering::Relaxed);
            Some(value.clone())
        } else {
            self.miss_counter.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// Insert value into cache
    pub async fn insert(&self, key: K, value: V) {
        self.moka_cache.insert(key.clone(), value.clone()).await;
        self.dashmap_fallback.insert(key, value);
    }

    /// Remove value from cache
    pub async fn remove(&self, key: &K) -> Option<V> {
        let moka_result = self.moka_cache.remove(key).await;
        let dashmap_result = self.dashmap_fallback.remove(key);

        moka_result.or_else(|| dashmap_result.map(|(_, v)| v))
    }

    /// Get cache hit rate (>95% target)
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hit_counter.load(Ordering::Relaxed) as f64;
        let misses = self.miss_counter.load(Ordering::Relaxed) as f64;
        let total = hits + misses;
        if total == 0.0 {
            0.0
        } else {
            hits / total
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> (u64, u64, u64) {
        (
            self.hit_counter.load(Ordering::Relaxed),
            self.miss_counter.load(Ordering::Relaxed),
            self.eviction_counter.load(Ordering::Relaxed),
        )
    }
}

/// StateMonitor tracks lock contention and performance metrics with alerting
pub struct StateMonitor {
    lock_times:          DashMap<String, Vec<Duration>>,
    alert_threshold_ms:  u64,
    alerts:              mpsc::UnboundedSender<String>,
    performance_history: DashMap<String, Vec<(DateTime<Utc>, Duration)>>,
}

impl StateMonitor {
    /// Create new monitor with alert threshold
    pub fn new(alert_threshold_ms: u64) -> (Self, mpsc::UnboundedReceiver<String>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self {
                lock_times: DashMap::new(),
                alert_threshold_ms,
                alerts: tx,
                performance_history: DashMap::new(),
            },
            rx,
        )
    }

    /// Record lock acquisition time and trigger alerts if above threshold
    pub fn record_lock_time(&self, key: String, duration: Duration) {
        self.lock_times
            .entry(key.clone())
            .or_insert_with(Vec::new)
            .push(duration);
        self.performance_history
            .entry(key.clone())
            .or_insert_with(Vec::new)
            .push((Utc::now(), duration));

        if duration.as_millis() as u64 > self.alert_threshold_ms {
            let _ = self.alerts.send(format!(
                "Lock contention alert for {}: {}ms (threshold: {}ms)",
                key,
                duration.as_millis(),
                self.alert_threshold_ms
            ));
        }
    }

    /// Get average lock acquisition time for a key
    pub fn get_average_lock_time(&self, key: &str) -> Option<Duration> {
        self.lock_times.get(key).and_then(|times| {
            if times.is_empty() {
                None
            } else {
                let sum: Duration = times.iter().sum();
                Some(sum / times.len() as u32)
            }
        })
    }

    /// Get lock contention percentage (<5% target)
    pub fn get_contention_percentage(&self, total_operations: u64) -> f64 {
        if total_operations == 0 {
            0.0
        } else {
            let contended = self
                .lock_times
                .iter()
                .map(|entry| entry.value().len())
                .sum::<usize>() as u64;
            (contended as f64 / total_operations as f64) * 100.0
        }
    }

    /// Get performance trend analysis
    pub fn get_performance_trend(&self, key: &str, window_minutes: i64) -> Option<f64> {
        let cutoff = Utc::now() - chrono::Duration::minutes(window_minutes);
        self.performance_history.get(key).and_then(|history| {
            let recent: Vec<_> = history.iter().filter(|(time, _)| *time > cutoff).collect();

            if recent.len() < 2 {
                None
            } else {
                let first_avg = recent
                    .iter()
                    .take(recent.len() / 2)
                    .map(|(_, dur)| dur.as_millis() as f64)
                    .sum::<f64>()
                    / (recent.len() / 2) as f64;
                let second_avg = recent
                    .iter()
                    .skip(recent.len() / 2)
                    .map(|(_, dur)| dur.as_millis() as f64)
                    .sum::<f64>()
                    / (recent.len() / 2) as f64;
                Some((second_avg - first_avg) / first_avg * 100.0) // percentage change
            }
        })
    }
}

/// MessageBus provides lock-free communication for complex state coordination
/// Uses channels for pub-sub pattern without locks
pub struct MessageBus<M> {
    subscribers:     DashMap<String, Vec<mpsc::UnboundedSender<M>>>,
    message_counter: Arc<AtomicU64>,
}

impl<M: Clone + Send + Sync + 'static> MessageBus<M> {
    /// Create new message bus
    pub fn new() -> Self {
        Self {
            subscribers:     DashMap::new(),
            message_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Subscribe to a topic, returns receiver channel
    pub fn subscribe(&self, topic: String) -> mpsc::UnboundedReceiver<M> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.subscribers
            .entry(topic)
            .or_insert_with(Vec::new)
            .push(tx);
        rx
    }

    /// Publish message to all subscribers of a topic
    pub fn publish(&self, topic: &str, message: M) {
        self.message_counter.fetch_add(1, Ordering::Relaxed);
        if let Some(subscribers) = self.subscribers.get(topic) {
            for subscriber in subscribers.iter() {
                let _ = subscriber.send(message.clone());
            }
        }
    }

    /// Get message throughput statistics
    pub fn message_count(&self) -> u64 {
        self.message_counter.load(Ordering::Relaxed)
    }

    /// Unsubscribe all for a topic (for cleanup)
    pub fn clear_topic(&self, topic: &str) {
        self.subscribers.remove(topic);
    }
}

/// ConnectionPool provides efficient resource sharing across services
/// Implements connection pooling with timeout protection
pub struct ConnectionPool<T> {
    pool:               Arc<RwLock<Vec<T>>>,
    max_size:           usize,
    timeout:            Duration,
    active_connections: Arc<AtomicU64>,
    total_created:      Arc<AtomicU64>,
}

impl<T> ConnectionPool<T> {
    /// Create new connection pool with max size and timeout
    pub fn new(max_size: usize, timeout_ms: u64) -> Self {
        Self {
            pool: Arc::new(RwLock::new(Vec::with_capacity(max_size))),
            max_size,
            timeout: Duration::from_millis(timeout_ms),
            active_connections: Arc::new(AtomicU64::new(0)),
            total_created: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get connection from pool or create new one
    pub async fn get_connection<F, Fut>(&self, factory: F) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let start = Instant::now();

        // Try to get from pool first
        if let Ok(mut pool) = timeout(self.timeout, self.pool.write()).await {
            if let Some(conn) = pool.pop() {
                return Ok(conn);
            }
        }

        // Check if we can create new connection
        let current_active = self.active_connections.load(Ordering::Relaxed);
        if current_active >= self.max_size as u64 {
            return Err("Connection pool exhausted".into());
        }

        // Create new connection
        self.active_connections.fetch_add(1, Ordering::Relaxed);
        self.total_created.fetch_add(1, Ordering::Relaxed);

        let conn = timeout(self.timeout, factory())
            .await
            .map_err(|_| "Connection creation timeout")?;

        Ok(conn)
    }

    /// Return connection to pool
    pub async fn return_connection(&self, conn: T) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut pool = timeout(self.timeout, self.pool.write())
            .await
            .map_err(|_| "Pool return timeout")?;

        if pool.len() < self.max_size {
            pool.push(conn);
        } else {
            // Pool full, drop connection (decrement active count implicitly via Drop)
            self.active_connections.fetch_sub(1, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Get pool statistics
    pub fn stats(&self) -> (u64, u64, usize) {
        (
            self.active_connections.load(Ordering::Relaxed),
            self.total_created.load(Ordering::Relaxed),
            self.max_size,
        )
    }
}

/// PerformanceMetrics for monitoring
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub lock_acquisition_time_ms: f64,
    pub cache_hit_rate:           f64,
    pub memory_usage_mb:          f64,
    pub active_connections:       u64,
    pub timestamp:                DateTime<Utc>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            lock_acquisition_time_ms: 0.0,
            cache_hit_rate:           0.0,
            memory_usage_mb:          0.0,
            active_connections:       0,
            timestamp:                Utc::now(),
        }
    }
}

/// Integrated performance monitoring system
pub struct PerformanceMonitor {
    state_monitors:  DashMap<String, Arc<StateMonitor>>,
    cache_monitors:  DashMap<String, Arc<LockFreeCache<String, serde_json::Value>>>,
    pool_monitors:   DashMap<String, Arc<ConnectionPool<serde_json::Value>>>,
    metrics_history: Arc<RwLock<Vec<PerformanceMetrics>>>,
    alert_sender:    mpsc::UnboundedSender<String>,
}

impl PerformanceMonitor {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<String>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self {
                state_monitors:  DashMap::new(),
                cache_monitors:  DashMap::new(),
                pool_monitors:   DashMap::new(),
                metrics_history: Arc::new(RwLock::new(Vec::new())),
                alert_sender:    tx,
            },
            rx,
        )
    }

    pub async fn collect_metrics(&self) -> PerformanceMetrics {
        let mut metrics = PerformanceMetrics::new();

        // Aggregate lock times
        let mut total_lock_time = 0.0;
        let mut lock_count = 0;
        for monitor in self.state_monitors.iter() {
            // Simplified aggregation
            if let Some(avg) = monitor.get_average_lock_time("global") {
                total_lock_time += avg.as_millis() as f64;
                lock_count += 1;
            }
        }
        metrics.lock_acquisition_time_ms = if lock_count > 0 {
            total_lock_time / lock_count as f64
        } else {
            0.0
        };

        // Aggregate cache stats
        let mut total_hit_rate = 0.0;
        let mut cache_count = 0;
        for cache in self.cache_monitors.iter() {
            total_hit_rate += cache.hit_rate();
            cache_count += 1;
        }
        metrics.cache_hit_rate = if cache_count > 0 {
            total_hit_rate / cache_count as f64
        } else {
            0.0
        };

        // Aggregate connection stats
        let mut total_connections = 0;
        for pool in self.pool_monitors.iter() {
            let (active, _, _) = pool.stats();
            total_connections += active;
        }
        metrics.active_connections = total_connections;

        // Store in history
        let mut history = self.metrics_history.write().await;
        history.push(metrics.clone());
        if history.len() > 1000 {
            // Keep last 1000 entries
            history.remove(0);
        }

        metrics
    }

    pub async fn get_performance_trend(&self, minutes: i64) -> Option<f64> {
        let history = self.metrics_history.read().await;
        let cutoff = Utc::now() - chrono::Duration::minutes(minutes);
        let recent: Vec<_> = history.iter().filter(|m| m.timestamp > cutoff).collect();

        if recent.len() < 2 {
            None
        } else {
            let mid = recent.len() / 2;
            let first_avg = recent
                .iter()
                .take(mid)
                .map(|m| m.lock_acquisition_time_ms)
                .sum::<f64>()
                / mid as f64;
            let second_avg = recent
                .iter()
                .skip(mid)
                .map(|m| m.lock_acquisition_time_ms)
                .sum::<f64>()
                / (recent.len() - mid) as f64;
            Some((second_avg - first_avg) / first_avg * 100.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::sleep;

    use super::*;

    #[tokio::test]
    async fn test_optimized_state_manager() {
        let manager = OptimizedStateManager::new(42i32, 1000, Some(100));

        // Test read
        let result = manager.read(|x| *x).await.unwrap();
        assert_eq!(result, 42);

        // Test write
        manager.write(|x| *x = 100).await.unwrap();
        let result = manager.read(|x| *x).await.unwrap();
        assert_eq!(result, 100);
    }

    #[tokio::test]
    async fn test_lock_free_cache() {
        let cache = LockFreeCache::new(100, 60);

        cache.insert("key1".to_string(), "value1".to_string()).await;
        let result = cache.get(&"key1".to_string()).await;
        assert_eq!(result, Some("value1".to_string()));

        let hit_rate = cache.hit_rate();
        assert!(hit_rate > 0.0);
    }

    #[tokio::test]
    async fn test_message_bus() {
        let bus = MessageBus::<String>::new();
        let mut rx = bus.subscribe("test_topic".to_string());

        bus.publish("test_topic", "hello".to_string());

        let message = rx.recv().await.unwrap();
        assert_eq!(message, "hello");
    }

    #[tokio::test]
    async fn test_connection_pool() {
        let pool = ConnectionPool::<String>::new(5, 1000);

        let conn = pool
            .get_connection(|| async { "connection".to_string() })
            .await
            .unwrap();
        assert_eq!(conn, "connection");

        pool.return_connection(conn).await.unwrap();
    }

    #[tokio::test]
    async fn test_state_monitor() {
        let (monitor, mut alerts) = StateMonitor::new(10);

        monitor.record_lock_time("test".to_string(), Duration::from_millis(15));

        // Should trigger alert
        let alert = alerts.recv().await.unwrap();
        assert!(alert.contains("contention alert"));
    }
}

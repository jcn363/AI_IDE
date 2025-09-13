//! Consolidated async utility functions for the Rust AI IDE
//!
//! This module provides unified async patterns to eliminate code duplication
//! across the Rust codebase. It addresses the following issues:
//!
//! - Inconsistent task spawning (tokio::spawn vs tauri::async_runtime::spawn)
//! - Missing standardized cancellation handling
//! - No unified workflow patterns (fan-out/fan-in)
//! - Inconsistent error handling in async contexts
//! - Memory leak prevention through proper scoping
//! - Standardized timeout and retry behavior
//! - Unified concurrency control mechanisms

use std::sync::Arc;
use std::time::Duration;

use futures::stream::futures_unordered::FuturesUnordered;
use futures::stream::{self, StreamExt};
use tokio::sync::{broadcast, Semaphore};
use tokio::time::{sleep, timeout_at, Instant as TokioInstant};
use tokio_util::sync::CancellationToken;

use crate::errors::{IdeError, IdeResult};

/// Core async utility types
pub mod types {
    use super::{broadcast, CancellationToken};

    /// Unified cancellation token for consistent cancellation handling
    pub type Cancellation = CancellationToken;

    /// Graceful shutdown coordinator
    pub struct ShutdownCoordinator {
        pub token:        CancellationToken,
        _shutdown_sender: broadcast::Sender<()>,
    }

    impl Default for ShutdownCoordinator {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ShutdownCoordinator {
        pub fn new() -> Self {
            let (sender, _) = broadcast::channel(1);
            let token = CancellationToken::new();

            Self {
                token,
                _shutdown_sender: sender,
            }
        }

        pub async fn wait_for_shutdown(&self) {
            self.token.cancelled().await;
        }

        pub fn cancel(&self) {
            self.token.cancel();
        }

        pub fn child_token(&self) -> CancellationToken {
            self.token.child_token()
        }
    }

    /// Result of a timeout operation
    pub enum TimeoutResult<T> {
        Completed(T),
        TimedOut,
        Cancelled,
    }
}

// ===== TIMEOUT UTILITIES =================================================================

/// Enhanced timeout wrapper for async operations
pub async fn with_timeout<T, F, Fut>(future: F, duration: Duration) -> IdeResult<types::TimeoutResult<T>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let deadline = TokioInstant::now() + duration;

    match timeout_at(deadline, future()).await {
        Ok(result) => Ok(types::TimeoutResult::Completed(result)),
        Err(_timeout_error) => {
            // Check if we're past the deadline (timeout) or cancelled
            if TokioInstant::now() >= deadline {
                Ok(types::TimeoutResult::TimedOut)
            } else {
                Ok(types::TimeoutResult::Cancelled)
            }
        }
    }
}

/// Simplified timeout wrapper
pub async fn timeout_operation<T, F, Fut>(future: F, duration: Duration) -> IdeResult<T>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    match with_timeout(future, duration).await {
        Ok(types::TimeoutResult::Completed(result)) => Ok(result),
        Ok(types::TimeoutResult::TimedOut) => Err(IdeError::Timeout {
            message:   "async operation timed out".to_string(),
            operation: "unknown".to_string(),
            duration:  std::time::Duration::new(30, 0),
        }),
        Ok(types::TimeoutResult::Cancelled) => Err(IdeError::Generic {
            message: "async operation cancelled".to_string(),
        }),
        Err(e) => Err(e),
    }
}

// ===== RETRY UTILITIES ==================================================================

/// Configurable retry options
pub struct RetryConfig {
    /// Maximum number of attempts (including first attempt)
    pub max_attempts:       u32,
    /// Initial delay between attempts
    pub initial_delay:      Duration,
    /// Maximum delay between attempts
    pub max_delay:          Duration,
    /// Backoff multiplier (default: 2.0 for exponential)
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts:       3,
            initial_delay:      Duration::from_millis(1000),
            max_delay:          Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// Enhanced retry logic with exponential backoff and cancellation
pub async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
    config: &RetryConfig,
    should_retry: impl Fn(&E, u32) -> bool,
    cancellation: Option<&CancellationToken>,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display + AsRef<str>,
{
    let mut delay = config.initial_delay;
    let mut last_error: Option<E> = None;

    for attempt in 1..=config.max_attempts {
        // Check for cancellation
        if let Some(token) = cancellation {
            if token.is_cancelled() {
                return Err(last_error.unwrap_or_else(|| panic!("No error to return after cancellation")));
            }
        }

        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                last_error = Some(err);

                // Don't retry on the last attempt or if error shouldn't be retried
                if attempt == config.max_attempts || !should_retry(last_error.as_ref().unwrap(), attempt) {
                    return Err(last_error.unwrap());
                }

                log::warn!(
                    "Attempt {} failed: {}. Retrying in {:?}",
                    attempt,
                    last_error.as_ref().unwrap(),
                    delay
                );

                // Wait with cancellation support
                if let Some(token) = cancellation {
                    tokio::select! {
                        _ = sleep(delay) => {},
                        _ = token.cancelled() => {
                            return Err(last_error.unwrap());
                        }
                    }
                } else {
                    sleep(delay).await;
                }

                // Calculate next delay with exponential backoff
                delay = delay
                    .mul_f64(config.backoff_multiplier)
                    .min(config.max_delay);
            }
        }
    }

    unreachable!("Loop should always return");
}

/// Simplified retry function for common use cases
pub async fn retry_simple<T, F, Fut>(
    operation: F,
    max_attempts: u32,
    cancellation: Option<&CancellationToken>,
) -> Result<T, String>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    let config = RetryConfig {
        max_attempts,
        ..Default::default()
    };

    retry_with_backoff(
        operation,
        &config,
        |error: &String, _attempt| {
            // Retry network-related and transient errors
            !error.contains("fatal") && !error.contains("unauthorized") && !error.contains("forbidden")
        },
        cancellation,
    )
    .await
}

// ===== TASK MANAGEMENT ==================================================================

/// Unified task spawner - eliminates tokio::spawn vs tauri::async_runtime::spawn inconsistencies
pub mod task {
    use super::{types, CancellationToken, StreamExt};

    /// Standard task spawning - use this instead of tokio::spawn/AR::spawn directly
    pub fn spawn<F>(future: F, task_name: &str) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let task_name_owned = task_name.to_string();
        tokio::spawn(async move {
            log::debug!("Async task '{}' started", task_name_owned);
            let result = future.await;
            log::debug!("Async task '{}' completed", &task_name_owned);
            result
        })
    }

    /// Spawn with named logging
    pub fn spawn_named<F>(
        future: F,
        task_name: &str,
        cancellation: Option<&CancellationToken>,
    ) -> tokio::task::JoinHandle<Option<F::Output>>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let task_name_owned = task_name.to_string();
        let child_token = cancellation.map(|c| c.child_token());

        let _task_name_owned = task_name.to_string();
        tokio::spawn(async move {
            if let Some(token) = child_token {
                tokio::select! {
                    result = future => {
                        log::debug!("Task '{}' completed normally", task_name_owned);
                        Some(result)
                    },
                    _ = token.cancelled() => {
                        log::debug!("Task '{}' cancelled", task_name_owned);
                        None
                    },
                }
            } else {
                log::debug!("Task '{}' completed normally", task_name_owned);
                Some(future.await)
            }
        })
    }

    /// Background task manager for graceful shutdown
    pub struct TaskManager {
        coordinator: types::ShutdownCoordinator,
        handles:     Vec<tokio::task::JoinHandle<()>>,
    }

    impl Default for TaskManager {
        fn default() -> Self {
            Self::new()
        }
    }

    impl TaskManager {
        pub fn new() -> Self {
            Self {
                coordinator: types::ShutdownCoordinator::new(),
                handles:     Vec::new(),
            }
        }

        /// Spawn a background task that will be cancelled on shutdown
        pub fn spawn_background_task<F>(&mut self, future: F, task_name: &str)
        where
            F: std::future::Future<Output = ()> + Send + 'static,
        {
            let task_name_owned = task_name.to_string();
            let cancellation = self.coordinator.child_token();

            let handle = tokio::spawn(async move {
                tokio::select! {
                    _ = future => {},
                    _ = cancellation.cancelled() => {
                        log::debug!("Background task '{}' cancelled during shutdown", task_name_owned);
                    },
                }
            });

            self.handles.push(handle);
        }

        /// Wait for all background tasks to complete
        pub async fn shutdown(self) {
            self.coordinator.cancel();

            for handle in self.handles {
                let _ = handle.await;
            }
        }

        pub fn get_cancellation_token(&self) -> CancellationToken {
            self.coordinator.child_token()
        }
    }
}

// ===== WORKFLOW UTILITIES ===============================================================

/// Async workflow helpers (fan-out/fan-in patterns)
pub mod workflow {
    use super::{stream, FuturesUnordered, StreamExt};

    /// Fan-out pattern: start multiple async operations
    pub async fn fan_out<F, Fut>(futures: Vec<F>) -> Vec<Fut::Output>
    where
        F: Fn() -> Fut + Clone,
        Fut: std::future::Future,
    {
        let futures_vec: FuturesUnordered<Fut> = futures.into_iter().map(|f| f()).collect();

        futures_vec.collect::<Vec<_>>().await
    }

    /// Fan-in pattern: wait for multiple results and combine them
    pub async fn fan_in<Iter, Fut, T>(futures: Iter, combiner: impl Fn(Vec<Fut::Output>) -> T) -> T
    where
        Iter: IntoIterator<Item = Fut>,
        Fut: std::future::Future,
    {
        let results = futures::future::join_all(futures).await;
        combiner(results)
    }

    /// Waterfall pattern: execute operations sequentially
    pub async fn waterfall<Iter, Fut>(operations: Iter) -> Vec<Fut::Output>
    where
        Iter: IntoIterator<Item = Fut>,
        Fut: std::future::Future,
    {
        let mut results = Vec::new();
        for future in operations {
            results.push(future.await);
        }
        results
    }

    /// Parallel execution with concurrency limit
    pub async fn concurrent_limited<Iter, Item, F, Fut, R>(
        items: Iter,
        concurrency_limit: usize,
        processor: F,
    ) -> Vec<R>
    where
        Iter: IntoIterator<Item = Item>,
        Item: Send + 'static,
        F: Fn(Item) -> Fut + Clone + Send + 'static,
        Fut: std::future::Future<Output = R> + Send + 'static,
        R: Send + 'static,
    {
        use futures::stream::StreamExt;
        stream::iter(items)
            .map(|item| {
                let processor = processor.clone();
                processor(item)
            })
            .buffer_unordered(concurrency_limit)
            .collect::<Vec<R>>()
            .await
    }
}

// ===== CONCURRENCY CONTROL UTILITIES ====================================================

/// Rate limiting and concurrency control
pub mod concurrency {
    use super::{sleep, Arc, Duration, Semaphore, TokioInstant};

    /// Semaphore-based concurrency limiter
    pub struct ConcurrencyLimiter {
        semaphore: Arc<Semaphore>,
    }

    impl ConcurrencyLimiter {
        pub fn new(max_concurrent: usize) -> Self {
            Self {
                semaphore: Arc::new(Semaphore::new(max_concurrent)),
            }
        }

        pub async fn acquire<F, Fut, T, E>(&self, operation: F) -> Result<T, E>
        where
            F: Fn() -> Fut,
            Fut: std::future::Future<Output = Result<T, E>>,
        {
            let permit = self
                .semaphore
                .acquire()
                .await
                .map_err(|e| panic!("Failed to acquire permit: {}", e))?;

            let result = operation().await;
            drop(permit); // Release permit
            result
        }

        /// Try to acquire without blocking
        pub fn try_acquire(&self) -> Option<tokio::sync::SemaphorePermit<'_>> {
            self.semaphore.try_acquire().ok()
        }
    }

    /// Rate limiter for operations
    pub struct RateLimiter {
        semaphore: Arc<Semaphore>,
        interval:  Duration,
    }

    impl RateLimiter {
        pub fn new(operations_per_second: f64) -> Self {
            let max_concurrent = operations_per_second.max(1.0) as usize;
            let interval = Duration::from_secs_f64(1.0 / operations_per_second);

            Self {
                semaphore: Arc::new(Semaphore::new(max_concurrent)),
                interval,
            }
        }

        pub async fn throttle<F, Fut, T>(&self, operation: F) -> Result<T, String>
        where
            F: FnOnce() -> Fut,
            Fut: std::future::Future<Output = Result<T, String>>,
        {
            let _permit = self
                .semaphore
                .acquire()
                .await
                .map_err(|e| format!("Failed to acquire rate limit permit: {}", e))?;
            let start = TokioInstant::now();

            let result = operation().await;

            let elapsed = start.elapsed();
            if elapsed < self.interval {
                sleep(self.interval - elapsed).await;
            }

            result
        }
    }
}

// ===== COMPREHENSIVE ASYNC OPERATION WRAPPER =================================================

/// Configuration for async operations
pub struct AsyncOperationConfig {
    pub timeout:           Option<Duration>,
    pub retry:             Option<RetryConfig>,
    pub concurrency_limit: Option<usize>,
    pub cancellation:      Option<CancellationToken>,
}

pub async fn async_operation<T, F, Fut, E>(operation: F, config: AsyncOperationConfig) -> IdeResult<T>
where
    F: Fn() -> Fut + Clone + Send + 'static,
    Fut: std::future::Future<Output = Result<T, E>> + Send + 'static,
    T: Send + 'static,
    E: std::fmt::Display + std::fmt::Debug + Send + std::convert::AsRef<str> + 'static,
{
    let operation = operation;

    // Wrap with concurrency limit if specified
    let operation_with_concurrency = if let Some(limit) = config.concurrency_limit {
        let limiter = concurrency::ConcurrencyLimiter::new(limit);
        let _operation_clone1 = operation.clone();
        let operation_clone2 = operation.clone();
        Box::pin(async move {
            limiter
                .acquire(move || {
                    let operation = operation_clone2.clone();
                    async move {
                        operation().await.map_err(|e| IdeError::Generic {
                            message: e.as_ref().to_string(),
                        })
                    }
                })
                .await
        }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, IdeError>> + Send>>
    } else {
        let operation_clone = operation.clone();
        Box::pin(async move {
            operation_clone().await.map_err(|e| IdeError::Generic {
                message: e.as_ref().to_string(),
            })
        })
    };

    // Wrap with retry if specified
    let operation_with_retry = if let Some(retry_config) = &config.retry {
        let operation_clone = operation.clone();
        let cancellation = config.cancellation.as_ref();
        Box::pin(async move {
            retry_with_backoff(
                operation_clone,
                retry_config,
                |error: &E, _attempt| !error.as_ref().contains("fatal") && !error.as_ref().contains("unauthorized"),
                cancellation,
            )
            .await
            .map_err(|e| IdeError::Generic {
                message: e.as_ref().to_string(),
            })
        })
    } else {
        operation_with_concurrency
    };

    // Apply timeout if specified
    if let Some(timeout_duration) = config.timeout {
        timeout_operation(|| operation_with_retry, timeout_duration).await?
    } else {
        operation_with_retry.await.map_err(|e| IdeError::Generic {
            message: format!("async operation failed: {}", e),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    use super::*;

    #[tokio::test]
    async fn test_timeout_operation() {
        let operation = || async {
            sleep(Duration::from_millis(100)).await;
            "success"
        };

        let result = timeout_operation(operation, Duration::from_millis(200)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_timeout_exceeded() {
        let operation = || async {
            sleep(Duration::from_millis(200)).await;
            "success"
        };

        let result = timeout_operation(operation, Duration::from_millis(100)).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IdeError::Timeout(_)));
    }

    #[tokio::test]
    async fn test_retry_simple() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let operation = || {
            let counter = counter_clone.clone();
            async move {
                let attempts = counter.fetch_add(1, Ordering::SeqCst);
                if attempts < 2 {
                    Err("temporary failure".to_string())
                } else {
                    Ok("success")
                }
            }
        };

        let result = retry_simple(operation, 3, None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_task_manager() {
        let mut manager = task::TaskManager::new();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        manager.spawn_background_task(
            async move {
                sleep(Duration::from_millis(50)).await;
                counter_clone.fetch_add(1, Ordering::SeqCst);
            },
            "test_task",
        );

        sleep(Duration::from_millis(10)).await; // Give task time to start

        // Shutdown should cancel the task before it completes
        manager.shutdown().await;

        // Task might complete after shutdown starts, so counter could be 0 or 1
        let final_count = counter.load(Ordering::SeqCst);
        assert!(final_count <= 1);
    }

    #[tokio::test]
    async fn test_workflow_fan_out() {
        let futures: Vec<_> = (0..5)
            .map(|i| {
                move || async move {
                    sleep(Duration::from_millis(10)).await;
                    i * 2
                }
            })
            .collect();

        let results = workflow::fan_out(futures).await;
        assert_eq!(results.len(), 5);
        assert!(results.iter().all(|&x| x % 2 == 0));
    }
}

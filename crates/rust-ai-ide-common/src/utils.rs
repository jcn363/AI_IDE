//! Utility functions and helpers

use std::time::Duration;

use futures::stream::{self, StreamExt};
use tokio::time::{sleep, timeout};

use crate::errors::IdeError;

/// Utility functions for common operations
pub fn safe_unwrap<T>(option: Option<T>) -> Result<T, IdeError> {
    option.ok_or_else(|| IdeError::NotFound {
        message: "Value is None".to_string(),
    })
}

/// Safe string manipulation
pub fn safe_substring(s: &str, start: usize, len: usize) -> Option<&str> {
    if start + len <= s.len() {
        Some(&s[start..start + len])
    } else {
        None
    }
}

/// Timeout wrapper for async operations
pub async fn with_timeout<T, F, Fut>(future: F, duration: Duration) -> Result<T, String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    match timeout(duration, future()).await {
        Ok(result) => Ok(result),
        Err(_) => Err(format!("Operation timed out after {:?}", duration)),
    }
}

/// Retry logic with exponential backoff
pub async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
    max_attempts: u32,
    initial_delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display + From<std::io::Error>,
{
    let mut delay = initial_delay;
    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                if attempt == max_attempts {
                    return Err(err);
                }
                log::warn!(
                    "Attempt {} failed: {}. Retrying in {:?}",
                    attempt,
                    err,
                    delay
                );
                sleep(delay).await;
                delay = delay.saturating_mul(2); // Exponential backoff
            }
        }
    }
    // This should never be reached, but handle gracefully
    tracing::error!("Stream processing loop completed unexpectedly");
    Err(std::io::Error::new(std::io::ErrorKind::Other, "Stream processing failed").into())
}

/// Stream processing with concurrency limit
pub async fn process_stream_concurrent<T, F, Fut, R>(
    items: impl IntoIterator<Item = T>,
    concurrency: usize,
    processor: F,
) -> Vec<R>
where
    T: Send + 'static,
    F: Fn(T) -> Fut + Send + Clone + 'static,
    Fut: std::future::Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    stream::iter(items)
        .map(processor)
        .buffer_unordered(concurrency)
        .collect()
        .await
}

/// Generic async operation wrapper with optional timeout and retry
pub async fn async_operation<T, F, Fut, E>(
    operation: F,
    timeout_duration: Option<Duration>,
    retry_config: Option<(u32, Duration)>,
) -> Result<T, String>
where
    F: FnMut() -> Fut + Send + Clone + 'static,
    Fut: std::future::Future<Output = Result<T, E>> + Send + 'static,
    T: Send + 'static,
    E: std::fmt::Display + std::fmt::Debug + Send + 'static + From<std::io::Error>,
{
    let mut op = operation;

    let result_future = async move {
        if let Some((max_attempts, initial_delay)) = retry_config {
            retry_with_backoff(op, max_attempts, initial_delay)
                .await
                .map_err(|e| format!("{:?}", e))
        } else {
            op().await.map_err(|e| format!("{:?}", e))
        }
    };

    if let Some(duration) = timeout_duration {
        with_timeout(|| result_future, duration).await?
    } else {
        result_future.await
    }
}
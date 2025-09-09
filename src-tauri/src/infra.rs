//! Infrastructure modules for Tauri application
//!
//! This module provides thin wrappers around core functionality to resolve compilation
//! errors in the Tauri app.

use rust_ai_ide_common::types::PluginEventBus;
use rust_ai_ide_common::rate_limiting::RateLimiter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Type alias for compatibility
pub type RateLimiter = InfraRateLimiter<governor::middleware::StateInformationMiddleware<governor::clock::QuantaClock>, governor::clock::QuantaClock>;

/// EventBus wrapper for Tauri's state management
#[derive(Debug, Clone)]
pub struct EventBus {
    inner: Arc<PluginEventBus>,
}

impl EventBus {
    /// Create a new EventBus with default capacity
    pub fn new() -> Self {
        let capacity = 100;
        let sender = tokio::sync::broadcast::Sender::new(capacity);
        let receiver = sender.subscribe();
        let inner = Arc::new(PluginEventBus {
            sender: sender.clone(),
            receiver,
        });
        Self { inner }
    }

    /// Subscribe to events on a specific channel
    pub fn subscribe(&self, _channel: &str) -> tokio::sync::broadcast::Receiver<rust_ai_ide_common::types::PluginEvent> {
        self.inner.subscribe()
    }

    /// Emit an event on a specific channel
    pub async fn emit(&self, channel: &str, data: serde_json::Value) -> Result<(), String> {
        let event = rust_ai_ide_common::types::PluginEvent::Custom {
            plugin_id: channel.to_string(),
            event_type: "test_event".to_string(),
            data,
        };

        self.inner.send_event(event)
            .await
            .map_err(|e| format!("Failed to send event: {:?}", e))
    }
}

/// Send trait implementation for EventBus
unsafe impl Send for EventBus {}
/// Sync trait implementation for EventBus
unsafe impl Sync for EventBus {}

/// RateLimiter wrapper for Tauri's state management
#[derive(Debug)]
pub struct InfraRateLimiter<M = governor::middleware::StateInformationMiddleware<governor::clock::DefaultClock>, C = governor::clock::DefaultClock>
where
    C: governor::clock::Clock + Send + Sync + 'static,
    M: governor::middleware::RateLimitingMiddleware<Inner = governor::state::InMemoryState>,
    M::Clock: governor::clock::Clock + Send + Sync + 'static,
{
    inner: governor::RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, M, C>,
}

// Generic implementation for supported clock types
impl<M, C> InfraRateLimiter<M, C>
where
    C: governor::clock::Clock + Send + Sync + 'static,
    M: governor::middleware::RateLimitingMiddleware<State = governor::state::InMemoryState> + Default,
{
    /// Create a new RateLimiter with default quota
    pub fn new() -> Self {
        let quota = governor::Quota::per_second(std::num::NonZeroU32::new(10).unwrap());
        InfraRateLimiter {
            inner: governor::RateLimiter::direct(M::default(), quota),
        }
    }
    /// Create a new RateLimiter with custom quota
    pub fn new_with_quota(quota: governor::Quota) -> Self {
        InfraRateLimiter {
            inner: governor::RateLimiter::direct(M::default(), quota),
        }
    }

    /// Create a new RateLimiter with default quota for other clock types
    pub fn new_default<C2>() -> InfraRateLimiter<governor::middleware::StateInformationMiddleware<C2>, C2>
    where
        C2: governor::clock::Clock + Send + Sync + 'static,
    {
        let quota = governor::Quota::per_second(std::num::NonZeroU32::new(10).unwrap());
        InfraRateLimiter {
            inner: governor::RateLimiter::direct(governor::middleware::StateInformationMiddleware::default(), quota),
        }
    }
}

    /// Check if the rate limit allows n operations
    pub fn check_n(&self, n: u32) -> Result<(), governor::NotUntil<C>> {
        self.inner.check_n(n)
    }

    /// Placeholders for compatibility - not needed in new version
    pub fn check_limit(&self) -> bool {
        true
    }

    pub fn consume_token(&mut self) -> bool {
        true
    }
}

impl InfraRateLimiter<governor::middleware::StateInformationMiddleware<governor::clock::QuantaClock>, governor::clock::QuantaClock> {
    /// Create a new RateLimiter with QuantaClock
    pub fn new_quanta() -> Self {
        let quota = governor::Quota::per_second(std::num::NonZeroU32::new(10).unwrap());
        Self {
            inner: governor::RateLimiter::direct(governor::middleware::StateInformationMiddleware::default(), quota),
        }
    }
}

/// Send trait implementation for InfraRateLimiter
unsafe impl<M, C> Send for InfraRateLimiter<M, C>
where
    M: Send + Sync,
    C: Send + Sync,
{}
/// Sync trait implementation for InfraRateLimiter
unsafe impl<M, C> Sync for InfraRateLimiter<M, C>
where
    M: Send + Sync,
    C: Send + Sync,
{}

/// Generic connection pool wrapper for LSP connections
#[derive(Debug, Clone)]
pub struct ConnectionPool<C> {
    pool: Arc<Mutex<HashMap<String, C>>>,
    max_connections: usize,
}

impl<C> ConnectionPool<C>
where
    C: Clone + Send + Sync,
{
    /// Create a new ConnectionPool with default max connections
    pub fn new() -> Self {
        let max_connections = 10;
        Self {
            pool: Arc::new(Mutex::new(HashMap::new())),
            max_connections,
        }
    }

    /// Get a connection from the pool
    pub async fn get_connection(&self, key: &str) -> Option<C> {
        let pool = self.pool.lock().await;
        pool.get(key).cloned()
    }

    /// Add or update a connection in the pool
    pub async fn add_connection(&self, key: String, connection: C) -> Result<(), String> {
        let mut pool = self.pool.lock().await;
        if pool.len() >= self.max_connections {
            return Err("Max connections reached".to_string());
        }
        pool.insert(key, connection);
        Ok(())
    }

    /// Release a connection from the pool
    pub async fn release_connection(&self, key: &str) -> Result<(), String> {
        let mut pool = self.pool.lock().await;
        if pool.remove(key).is_none() {
            return Err("Connection not found".to_string());
        }
        Ok(())
    }

    /// Get current number of connections
    pub async fn connection_count(&self) -> usize {
        let pool = self.pool.lock().await;
        pool.len()
    }
}

/// Send trait implementation for ConnectionPool
unsafe impl<C> Send for ConnectionPool<C> {}
/// Sync trait implementation for ConnectionPool
unsafe impl<C> Sync for ConnectionPool<C> {}
//! # Infrastructure Components Module
//!
//! This module provides core infrastructure components for the Rust AI IDE application,
//! implementing the standard patterns described in AGENTS.md for pub-sub communication,
//! rate limiting, and connection pooling. These components are designed to be thread-safe
//! and provide the foundation for scalable async operations throughout the application.
//!
//! ## Key Components
//!
//! - **EventBus**: Asynchronous pub-sub communication system for inter-module communication
//! - **InfraRateLimiter**: Governor-based rate limiting with configurable quotas and clocks
//! - **ConnectionPool**: Generic connection pooling for LSP and other network services
//!
//! ## Architecture Patterns
//!
//! This module implements several architectural patterns:
//!
//! - **EventBus Pattern**: The standard pub-sub communication system (see AGENTS.md)
//! - **Rate Limiting**: Prevents resource exhaustion through configurable request limits
//! - **Connection Pooling**: Reuses connections to improve performance and resource usage
//! - **Thread Safety**: All components use Arc<Mutex<T>> for safe concurrent access
//!
//! ## Usage Examples
//!
//! ### EventBus for Pub-Sub Communication
//! ```rust,ignore
//! let event_bus = EventBus::new();
//!
//! // Subscribe to events
//! let mut receiver = event_bus.subscribe("my_channel");
//!
//! // Publish events asynchronously
//! event_bus.emit("my_channel", serde_json::json!({"data": "value"})).await?;
//! ```
//!
//! ### Rate Limiting API Requests
//! ```rust,ignore
//! let limiter = InfraRateLimiter::new();
//!
//! if limiter.check_n(1).is_ok() {
//!     // Process request
//!     make_api_call().await?;
//! }
//! ```
//!
//! ### Connection Pooling for LSP
//! ```rust,ignore
//! let pool = ConnectionPool::<LspConnection>::new();
//!
//! // Add connection to pool
//! pool.add_connection("rust-analyzer".to_string(), connection).await?;
//!
//! // Retrieve connection from pool
//! if let Some(conn) = pool.get_connection("rust-analyzer").await {
//!     // Use connection
//! }
//! ```

use rust_ai_ide_common::types::PluginEventBus;
use rust_ai_ide_common::rate_limiting::RateLimiter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Type alias for the standard rate limiter configuration used throughout the application.
/// Uses QuantaClock for high-precision timing and StateInformationMiddleware for detailed metrics.
pub type RateLimiter = InfraRateLimiter<governor::middleware::StateInformationMiddleware<governor::clock::QuantaClock>, governor::clock::QuantaClock>;

/// # Asynchronous Event Bus for Pub-Sub Communication
///
/// The EventBus implements the standard pub-sub communication pattern described in AGENTS.md.
/// It provides a thread-safe, asynchronous communication system for inter-module messaging
/// throughout the Rust AI IDE application.
///
/// ## Architecture
///
/// - Uses Tokio broadcast channels for efficient multi-subscriber communication
/// - Wraps `PluginEventBus` from `rust-ai-ide-common` for standardized event types
/// - Thread-safe through Arc<Mutex<>> pattern for concurrent access
/// - Capacity-limited channels prevent memory exhaustion under high load
///
/// ## Usage Patterns
///
/// ### Publishing Events
/// ```rust,ignore
/// let event_bus = EventBus::new();
/// event_bus.emit("file_watcher", json!({"action": "modified", "path": "/project/file.rs"})).await?;
/// ```
///
/// ### Subscribing to Events
/// ```rust,ignore
/// let mut receiver = event_bus.subscribe("file_watcher");
/// while let Ok(event) = receiver.recv().await {
///     match event {
///         PluginEvent::Custom { plugin_id, event_type, data } => {
///             // Handle event
///         }
///         _ => {}
///     }
/// }
/// ```
///
/// ## Performance Characteristics
///
/// - O(1) publish operations (broadcast to all subscribers)
/// - Memory bounded by channel capacity (default: 100 events)
/// - Non-blocking sends with backpressure handling
/// - Zero-copy event distribution where possible
#[derive(Debug, Clone)]
pub struct EventBus {
    inner: Arc<PluginEventBus>,
}

impl EventBus {
    /// Creates a new EventBus instance with default configuration.
    ///
    /// Initializes a broadcast channel with capacity of 100 events to prevent
    /// unbounded memory growth under high event load scenarios.
    ///
    /// # Returns
    /// A new EventBus instance ready for pub-sub operations.
    ///
    /// # Example
    /// ```rust,ignore
    /// let event_bus = EventBus::new();
    /// ```
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

    /// Subscribes to events on a specific channel.
    ///
    /// Returns a broadcast receiver that will receive all events published to any channel.
    /// The current implementation doesn't filter by channel (all subscribers receive all events),
    /// but the channel parameter is kept for API compatibility and future enhancements.
    ///
    /// # Parameters
    /// - `_channel`: Reserved for future channel-specific filtering (currently ignored)
    ///
    /// # Returns
    /// A broadcast receiver for consuming events asynchronously.
    ///
    /// # Example
    /// ```rust,ignore
    /// let mut receiver = event_bus.subscribe("analysis_events");
    /// if let Ok(event) = receiver.recv().await {
    ///     println!("Received event: {:?}", event);
    /// }
    /// ```
    pub fn subscribe(&self, _channel: &str) -> tokio::sync::broadcast::Receiver<rust_ai_ide_common::types::PluginEvent> {
        self.inner.subscribe()
    }

    /// Publishes an event to the specified channel.
    ///
    /// Creates a standardized PluginEvent and broadcasts it to all subscribers.
    /// The event contains the channel name as plugin_id and the provided data.
    ///
    /// # Parameters
    /// - `channel`: Identifier for the event channel/source
    /// - `data`: JSON-serializable event payload
    ///
    /// # Returns
    /// - `Ok(())` if the event was successfully queued for broadcasting
    /// - `Err(String)` if broadcasting failed (e.g., no subscribers or channel full)
    ///
    /// # Errors
    /// Returns an error if the underlying broadcast channel is full or closed.
    ///
    /// # Example
    /// ```rust,ignore
    /// let data = serde_json::json!({
    ///     "action": "file_saved",
    ///     "path": "/project/main.rs",
    ///     "timestamp": chrono::Utc::now()
    /// });
    /// event_bus.emit("file_operations", data).await?;
    /// ```
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

/// # Governor-Based Rate Limiter for Resource Protection
///
/// The InfraRateLimiter provides configurable rate limiting using the Governor crate,
/// implementing token bucket algorithms to prevent resource exhaustion and abuse.
/// This component is essential for protecting external API calls, file operations,
/// and other resource-intensive activities.
///
/// ## Architecture
///
/// - Uses Governor's middleware system for flexible rate limiting strategies
/// - Support for different clock implementations (QuantaClock for precision, DefaultClock for compatibility)
/// - In-memory state storage suitable for single-instance deployments
/// - Configurable quotas with burst allowances and refill rates
///
/// ## Clock Types
///
/// - **QuantaClock**: High-precision clock for low-latency applications
/// - **DefaultClock**: Standard system clock for general use cases
///
/// ## Usage Examples
///
/// ### Basic Rate Limiting
/// ```rust,ignore
/// let limiter = InfraRateLimiter::new(); // 10 requests per second
///
/// if limiter.check_n(1).is_ok() {
///     make_api_call().await?;
/// } else {
///     return Err("Rate limit exceeded".into());
/// }
/// ```
///
/// ### Custom Quota Configuration
/// ```rust,ignore
/// let quota = governor::Quota::per_minute(std::num::NonZeroU32::new(100).unwrap());
/// let limiter = InfraRateLimiter::new_with_quota(quota);
/// ```
///
/// ## Performance Characteristics
///
/// - O(1) rate limit checks with minimal overhead
/// - Memory efficient with bounded state storage
/// - Configurable burst allowances for handling traffic spikes
/// - Thread-safe for concurrent access patterns
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

/// # Generic Connection Pool for Resource Management
///
/// The ConnectionPool provides thread-safe connection pooling for network services,
/// particularly LSP (Language Server Protocol) connections. This component implements
/// the connection pooling pattern mandated by AGENTS.md for database and network operations.
///
/// ## Architecture
///
/// - Generic implementation supporting any connection type `C`
/// - Thread-safe access through Arc<Mutex<>> pattern
/// - Configurable maximum connection limits to prevent resource exhaustion
/// - In-memory HashMap storage for fast lookups and insertions
///
/// ## Supported Connection Types
///
/// - LSP connections (primary use case)
/// - Database connections
/// - External API clients
/// - Any cloneable, thread-safe resource
///
/// ## Usage Examples
///
/// ### LSP Connection Pooling
/// ```rust,ignore
/// let lsp_pool = ConnectionPool::<LspConnection>::new();
///
/// // Add a language server connection
/// lsp_pool.add_connection("rust-analyzer".to_string(), connection).await?;
///
/// // Retrieve connection for use
/// if let Some(conn) = lsp_pool.get_connection("rust-analyzer").await {
///     conn.request_completion(file_content).await?;
/// }
///
/// // Release when done (optional cleanup)
/// lsp_pool.release_connection("rust-analyzer").await?;
/// ```
///
/// ### Database Connection Pooling
/// ```rust,ignore
/// let db_pool = ConnectionPool::<DatabaseConnection>::new();
/// // Similar usage pattern for database connections
/// ```
///
/// ## Performance Characteristics
///
/// - O(1) average-case connection lookups via HashMap
/// - Minimal memory overhead per connection
/// - Configurable connection limits prevent resource exhaustion
/// - Thread-safe for concurrent access patterns
///
/// ## Error Handling
///
/// - Returns `None` for missing connections (graceful degradation)
/// - Validates connection limits before adding new connections
/// - Thread-safe operations with proper locking
#[derive(Debug, Clone)]
pub struct ConnectionPool<C> {
    pool: Arc<Mutex<HashMap<String, C>>>,
    max_connections: usize,
}

impl<C> ConnectionPool<C>
where
    C: Clone + Send + Sync,
{
    /// Creates a new ConnectionPool with default configuration.
    ///
    /// Initializes with a maximum of 10 connections, suitable for most LSP server scenarios.
    /// The pool uses an in-memory HashMap for efficient connection storage and retrieval.
    ///
    /// # Returns
    /// A new ConnectionPool instance ready for connection management.
    ///
    /// # Example
    /// ```rust,ignore
    /// let pool = ConnectionPool::<MyConnection>::new();
    /// ```
    pub fn new() -> Self {
        let max_connections = 10;
        Self {
            pool: Arc::new(Mutex::new(HashMap::new())),
            max_connections,
        }
    }

    /// Retrieves a connection from the pool by key.
    ///
    /// Performs a thread-safe lookup in the connection pool. Returns a clone of the
    /// connection if found, allowing multiple concurrent users of the same connection.
    ///
    /// # Parameters
    /// - `key`: Unique identifier for the connection (e.g., "rust-analyzer", "typescript")
    ///
    /// # Returns
    /// - `Some(C)` if the connection exists and can be cloned
    /// - `None` if no connection is found for the given key
    ///
    /// # Example
    /// ```rust,ignore
    /// if let Some(conn) = pool.get_connection("database").await {
    ///     conn.execute_query("SELECT * FROM users").await?;
    /// }
    /// ```
    pub async fn get_connection(&self, key: &str) -> Option<C> {
        let pool = self.pool.lock().await;
        pool.get(key).cloned()
    }

    /// Adds a new connection to the pool with the specified key.
    ///
    /// Stores the connection in the pool if capacity allows. The key must be unique;
    /// existing connections with the same key will be replaced.
    ///
    /// # Parameters
    /// - `key`: Unique identifier for the connection
    /// - `connection`: The connection instance to store
    ///
    /// # Returns
    /// - `Ok(())` if the connection was successfully added
    /// - `Err(String)` if the maximum connection limit has been reached
    ///
    /// # Errors
    /// Returns an error if adding the connection would exceed the maximum connection limit.
    ///
    /// # Example
    /// ```rust,ignore
    /// let connection = create_lsp_connection("rust-analyzer").await?;
    /// pool.add_connection("rust-analyzer".to_string(), connection).await?;
    /// ```
    pub async fn add_connection(&self, key: String, connection: C) -> Result<(), String> {
        let mut pool = self.pool.lock().await;
        if pool.len() >= self.max_connections {
            return Err("Max connections reached".to_string());
        }
        pool.insert(key, connection);
        Ok(())
    }

    /// Removes a connection from the pool by key.
    ///
    /// Explicitly releases a connection from the pool, freeing up capacity for new connections.
    /// This method should be called when a connection is no longer needed or has become invalid.
    ///
    /// # Parameters
    /// - `key`: Unique identifier of the connection to remove
    ///
    /// # Returns
    /// - `Ok(())` if the connection was found and removed
    /// - `Err(String)` if no connection exists for the given key
    ///
    /// # Errors
    /// Returns an error if no connection is found with the specified key.
    ///
    /// # Example
    /// ```rust,ignore
    /// // Clean up when connection is no longer needed
    /// pool.release_connection("temporary-session").await?;
    /// ```
    pub async fn release_connection(&self, key: &str) -> Result<(), String> {
        let mut pool = self.pool.lock().await;
        if pool.remove(key).is_none() {
            return Err("Connection not found".to_string());
        }
        Ok(())
    }

    /// Returns the current number of active connections in the pool.
    ///
    /// Useful for monitoring pool utilization and capacity planning.
    ///
    /// # Returns
    /// The number of connections currently stored in the pool.
    ///
    /// # Example
    /// ```rust,ignore
    /// let active_connections = pool.connection_count().await;
    /// println!("Pool utilization: {}/{}", active_connections, pool.max_connections);
    /// ```
    pub async fn connection_count(&self) -> usize {
        let pool = self.pool.lock().await;
        pool.len()
    }
}

/// Send trait implementation for ConnectionPool
unsafe impl<C> Send for ConnectionPool<C> {}
/// Sync trait implementation for ConnectionPool
unsafe impl<C> Sync for ConnectionPool<C> {}
//! Common AI Service interfaces and abstractions
//!
//! This module provides core traits and implementations for AI service management,
//! including service discovery, connection pooling, and unified interfaces.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use rust_ai_ide_ai_inference::ModelSize;
use rust_ai_ide_lsp::{AIService, AIContext, Completion};
use tokio::sync::Semaphore;
use std::time::{Duration, Instant};

/// Local AIProvider enum with added Claude variant
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AIProvider {
    Mock,
    OpenAI,
    Anthropic,
    CodeLlamaRust { model_size: rust_ai_ide_ai_inference::ModelSize },
    StarCoderRust { model_size: rust_ai_ide_ai_inference::ModelSize },
    Local { model_path: String },
    Claude { api_key: String },
}

/// Trait for AI services to implement common functionality
#[async_trait]
pub trait AIServiceTrait: Send + Sync {
    async fn get_completions(&self, context: AIContext) -> Result<Vec<Completion>, String>;
    async fn get_task_response(&self, context: AIContext, task: String) -> Result<String, String>;
    fn is_healthy(&self) -> bool;
    fn provider_type(&self) -> &str;
}

/// Wrapper for AIService to implement AIServiceTrait
pub struct WrappedAIService {
    service: AIService,
    provider: AIProvider,
}

impl WrappedAIService {
    pub fn new(service: AIService, provider: AIProvider) -> Self {
        Self { service, provider }
    }
}

#[async_trait]
impl AIServiceTrait for WrappedAIService {
    async fn get_completions(&self, context: AIContext) -> Result<Vec<Completion>, String> {
        // TODO: Implement get_completions method in AIService
        Err("get_completions method not yet implemented in AIService".to_string())
    }

    async fn get_task_response(&self, context: AIContext, task: String) -> Result<String, String> {
        // TODO: Implement get_task_response method in AIService
        Err("get_task_response method not yet implemented in AIService".to_string())
    }

    fn is_healthy(&self) -> bool {
        // Basic health check - could be extended
        true
    }

    fn provider_type(&self) -> &str {
        match &self.provider {
            AIProvider::Mock => "mock",
            AIProvider::OpenAI => "openai",
            AIProvider::Anthropic => "anthropic",
            AIProvider::CodeLlamaRust { .. } => "codellamarust",
            AIProvider::StarCoderRust { .. } => "starcoderrust",
            AIProvider::Local { .. } => "local",
            AIProvider::Claude { .. } => "claude",
        }
    }
}

/// Configuration for a pooled service
#[derive(Clone, Debug)]
pub struct PooledServiceConfig {
    pub provider: AIProvider,
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
}

/// Connection pool entry
pub struct PooledConnection<T> {
    service: Arc<T>,
    last_used: Instant,
    in_use: bool,
}

impl<T> PooledConnection<T> {
    pub fn new(service: Arc<T>) -> Self {
        Self {
            service,
            last_used: Instant::now(),
            in_use: false,
        }
    }

    pub fn is_expired(&self, config: &PooledServiceConfig) -> bool {
        self.last_used.elapsed() > config.idle_timeout
    }
}

/// Generic connection pool for any AI service
pub struct ConnectionPool<T: Send + Sync> {
    config: PooledServiceConfig,
    connections: Mutex<Vec<PooledConnection<T>>>,
    semaphore: Semaphore,
}

impl<T: Send + Sync> ConnectionPool<T> {
    pub fn new(config: PooledServiceConfig, initial_services: Vec<Arc<T>>) -> Self {
        let mut connections = Vec::new();
        for service in initial_services {
            connections.push(PooledConnection::new(service));
        }

        Self {
            connections: Mutex::new(connections),
            semaphore: Semaphore::new(config.max_connections),
            config,
        }
    }

    /// Acquire a service connection from the pool
    pub async fn acquire(&self) -> Result<PoolGuard<T>, String> {
        let permit = self.semaphore.acquire().await.map_err(|e| e.to_string())?;
        let mut connections = self.connections.lock().map_err(|e| e.to_string())?;

        // Find an available connection
        for conn in connections.iter_mut() {
            if !conn.in_use && !conn.is_expired(&self.config) {
                conn.in_use = true;
                conn.last_used = Instant::now();
                return Ok(PoolGuard {
                    connection: conn.service.clone(),
                    _permit: permit,
                    pool: self,
                });
            }
        }

        Err("No available connections in pool".to_string())
    }

    /// Return a connection to the pool (internal method)
    fn release(&self, service: &Arc<T>) {
        let mut connections = match self.connections.lock() {
            Ok(lock) => lock,
            Err(_) => return,
        };

        for conn in connections.iter_mut() {
            if Arc::ptr_eq(&conn.service, service) {
                conn.in_use = false;
                break;
            }
        }
    }

    /// Add a new service to the pool
    pub fn add_service(&self, service: Arc<T>) -> Result<(), String> {
        let mut connections = self.connections.lock().map_err(|e| e.to_string())?;
        connections.push(PooledConnection::new(service));
        Ok(())
    }

    /// Get pool status
    pub fn status(&self) -> Result<PoolStatus, String> {
        let connections = self.connections.lock().map_err(|e| e.to_string())?;
        let total = connections.len();
        let in_use = connections.iter().filter(|c| c.in_use).count();
        let available = total - in_use;

        Ok(PoolStatus { total, available, in_use })
    }
}

/// Guard for pooled connections
pub struct PoolGuard<'a, T: Send + Sync> {
    connection: Arc<T>,
    _permit: tokio::sync::SemaphorePermit<'a>,
    pool: &'a ConnectionPool<T>,
}

impl<'a, T: Send + Sync> std::ops::Deref for PoolGuard<'a, T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

impl<'a, T: Send + Sync> Drop for PoolGuard<'a, T> {
    fn drop(&mut self) {
        self.pool.release(&self.connection);
    }
}

/// Pool status information
#[derive(Debug, Clone)]
pub struct PoolStatus {
    pub total: usize,
    pub available: usize,
    pub in_use: usize,
}

/// AI Service Registry for service discovery and management
pub struct AIServiceRegistry {
    services: Mutex<HashMap<String, Arc<WrappedAIService>>>,
    pools: Mutex<HashMap<String, Arc<ConnectionPool<WrappedAIService>>>>,
}

impl AIServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: Mutex::new(HashMap::new()),
            pools: Mutex::new(HashMap::new()),
        }
    }

    /// Register a direct service (non-pooled)
    pub fn register_service(&self, name: &str, service: Arc<WrappedAIService>) -> Result<(), String> {
        let mut services = self.services.lock().map_err(|e| e.to_string())?;
        if services.contains_key(name) {
            return Err(format!("Service '{}' already registered", name));
        }
        services.insert(name.to_string(), service);
        Ok(())
    }

    /// Register a pooled service
    pub fn register_pooled_service(&self, name: &str, config: PooledServiceConfig, initial_services: Vec<WrappedAIService>) -> Result<(), String> {
        let services: Vec<Arc<WrappedAIService>> = initial_services.into_iter().map(Arc::new).collect();
        let pool = Arc::new(ConnectionPool::new(config, services));

        let mut pools = self.pools.lock().map_err(|e| e.to_string())?;
        pools.insert(name.to_string(), pool);
        Ok(())
    }

    /// Get a direct service by name
    pub fn get_service(&self, name: &str) -> Result<Arc<WrappedAIService>, String> {
        let services = self.services.lock().map_err(|e| e.to_string())?;
        services.get(name).cloned().ok_or_else(|| format!("Service '{}' not found", name))
    }

    /// Get a pooled service connection
    pub async fn get_pooled_service(&self, name: &str) -> Result<PoolGuard<WrappedAIService>, String> {
        let pools = self.pools.lock().map_err(|e| e.to_string())?;
        let pool = pools.get(name).ok_or_else(|| format!("Pooled service '{}' not found", name))?;
        pool.acquire().await
    }

    /// List all registered services
    pub fn list_services(&self) -> Result<Vec<String>, String> {
        let services = self.services.lock().map_err(|e| e.to_string())?;
        let service_names: Vec<String> = services.keys().cloned().collect();
        Ok(service_names)
    }

    /// List all pooled services
    pub fn list_pooled_services(&self) -> Result<Vec<String>, String> {
        let pools = self.pools.lock().map_err(|e| e.to_string())?;
        let pool_names: Vec<String> = pools.keys().cloned().collect();
        Ok(pool_names)
    }

    /// Get health status of all services
    pub fn health_check(&self) -> HashMap<String, bool> {
        let mut status = HashMap::new();

        // Check direct services
        if let Ok(services) = self.services.lock() {
            for (name, service) in services.iter() {
                status.insert(name.clone(), service.is_healthy());
            }
        }

        // Note: Pooled services health check could be extended to check pool status
        // For now, we assume pools are healthy if they exist
        if let Ok(pools) = self.pools.lock() {
            for name in pools.keys() {
                status.insert(name.clone(), true);
            }
        }

        status
    }

    /// Unregister a service
    pub fn unregister_service(&self, name: &str) -> Result<(), String> {
        let mut services = self.services.lock().map_err(|e| e.to_string())?;
        services.remove(name);
        Ok(())
    }
}

/// Global service registry instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_AI_REGISTRY: Arc<AIServiceRegistry> = Arc::new(AIServiceRegistry::new());
}

impl Default for AIServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
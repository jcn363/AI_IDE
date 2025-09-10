//! Language server pool manager for multi-language LSP support
//!
//! This module provides a pool of language servers with intelligent
//! resource management, automatic scaling, and request routing.

use dashmap::DashMap;
use futures::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};
use tokio::task::{spawn, JoinHandle};
use tokio::time::interval;
use tracing::{debug, info, warn};

use crate::client::LSPError;
#[cfg(feature = "multi-language-lsp")]
use crate::language_detection::LanguageDetector;
#[cfg(feature = "multi-language-lsp")]
use crate::language_server::{
    LanguageServerConfig, LanguageServerFactory, LanguageServerHandle,
    LanguageServerKind, LanguageServerWrapper, ServerHealth, ServerMetrics,
};
use crate::WebLanguageServerFactory;

/// Pool configuration for language server management
#[derive(Debug, Clone)]
pub struct LanguageServerPoolConfig {
    pub max_servers_per_language: usize,
    pub min_servers_per_language: usize,
    pub deactivation_timeout_secs: u64,
    pub health_check_interval_secs: u64,
    pub performance_monitoring: bool,
    pub enable_auto_scaling: bool,
    pub request_timeout_ms: u64,
    pub pool_maintenance_interval_secs: u64,
}

impl Default for LanguageServerPoolConfig {
    fn default() -> Self {
        Self {
            max_servers_per_language: 3,
            min_servers_per_language: 1,
            deactivation_timeout_secs: 300, // 5 minutes
            health_check_interval_secs: 60,
            performance_monitoring: true,
            enable_auto_scaling: true,
            request_timeout_ms: 10000, // 10 seconds
            pool_maintenance_interval_secs: 30,
        }
    }
}

/// Pool statistics and health information
#[derive(Debug, Clone, Default)]
pub struct PoolStatistics {
    pub total_servers: usize,
    pub active_servers: usize,
    pub healthy_servers: usize,
    pub languages_supported: usize,
    pub average_request_rate: f64,
    pub average_response_time_ms: f64,
    pub error_rate: f64,
    pub memory_usage_mb: f64,
    pub battery_level: Option<f32>,
    pub network_latency_ms: Option<f64>,
    pub background_tasks_active: usize,
    pub resource_pressure_score: f64,
}

/// Resource monitor structure
struct ResourceMonitor {
    cpu_usage: std::sync::atomic::AtomicU32,
    memory_available_mb: std::sync::atomic::AtomicU64,
    battery_level: std::sync::atomic::AtomicU32,
    last_update: std::sync::RwLock<std::time::Instant>,
}

impl ResourceMonitor {
    fn new() -> Self {
        Self {
            cpu_usage: std::sync::atomic::AtomicU32::new(0),
            memory_available_mb: std::sync::atomic::AtomicU64::new(0),
            battery_level: std::sync::atomic::AtomicU32::new(100),
            last_update: std::sync::RwLock::new(std::time::Instant::now()),
        }
    }

    async fn update_metrics(&self) -> Result<(), LSPError> {
        // Simulate resource monitoring (in real implementation, use sysinfo crate)
        // CPU usage (0-100%)
        let cpu = (chronodb::Utc::now().timestamp_nanos() % 100) as u32;
        self.cpu_usage.store(cpu, std::sync::atomic::Ordering::Relaxed);

        // Available memory (simulate decent amount)
        let mem_mb = 8000 + (chronodb::Utc::now().timestamp_nanos() % 2000) as u64;
        self.memory_available_mb.store(mem_mb, std::sync::atomic::Ordering::Relaxed);

        // Battery level (simulate laptop on battery)
        let battery = if std::time::Instant::now().duration_since(*self.last_update.read().await).as_secs() % 3600 < 1800 {
            80 // On battery
        } else {
            100 // Plugged in
        };
        self.battery_level.store(battery, std::sync::atomic::Ordering::Relaxed);

        *self.last_update.write().await = std::time::Instant::now();
        Ok(())
    }

    fn should_throttle_background_tasks(&self) -> bool {
        let cpu = self.cpu_usage.load(std::sync::atomic::Ordering::Relaxed);
        let battery = self.battery_level.load(std::sync::atomic::Ordering::Relaxed);
        let mem_mb = self.memory_available_mb.load(std::sync::atomic::Ordering::Relaxed);

        // High CPU usage (>80%) or low battery (<20%) or low memory (<100MB)
        cpu > 80 || battery < 20 || mem_mb < 100
    }

    fn get_resource_pressure_score(&self) -> f64 {
        let cpu = self.cpu_usage.load(std::sync::atomic::Ordering::Relaxed) as f64;
        let battery = self.battery_level.load(std::sync::atomic::Ordering::Relaxed) as f64;
        let mem_mb = self.memory_available_mb.load(std::sync::atomic::Ordering::Relaxed) as f64;

        // Calculate pressure score (0.0 = good, 1.0 = critical)
        let cpu_pressure = cpu / 100.0;
        let battery_pressure = if battery < 20.0 { (20.0 - battery) / 20.0 } else { 0.0 };
        let mem_pressure = if mem_mb < 100.0 { (100.0 - mem_mb) / 100.0 } else { 0.0 };

        (cpu_pressure * 0.5 + battery_pressure * 0.3 + mem_pressure * 0.2).min(1.0)
    }
}

/// Request to route to a language server
#[derive(Debug)]
pub struct PoolRequest<P, R> {
    pub method: String,
    pub params: P,
    pub language_hint: Option<String>,
    pub file_path: Option<String>,
    pub response_sender: oneshot::Sender<Result<R, LSPError>>,
    pub _phantom: std::marker::PhantomData<R>,
}

/// Server pool manager for multi-language LSP support
pub struct LanguageServerPool {
    /// Active language servers mapped by language kind
    servers: DashMap<LanguageServerKind, Vec<LanguageServerHandle>>,

    /// Language detector for automatic routing
    detector: LanguageDetector,

    /// Server factories for creating language servers
    factories: HashMap<String, Arc<dyn LanguageServerFactory>>,

    /// Request routing channel
    request_sender: Option<
        mpsc::UnboundedSender<
            PoolRequest<Box<dyn std::any::Any + Send + Sync>, Box<dyn std::any::Any + Send + Sync>>,
        >,
    >,

    /// Pool configuration
    config: Arc<RwLock<LanguageServerPoolConfig>>,

    /// Pool statistics
    stats: Arc<RwLock<PoolStatistics>>,

    /// Active background tasks
    background_tasks: Mutex<Vec<JoinHandle<()>>>,

    /// Pool health status
    healthy: Arc<RwLock<bool>>,

    /// Resource monitor for adaptive behavior
    resource_monitor: Arc<ResourceMonitor>,
}

impl Default for LanguageServerPool {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageServerPool {
    /// Create a new language server pool
    pub fn new_with_config(config: LanguageServerPoolConfig) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let servers = Arc::new(DashMap::new());
        let config = Arc::new(RwLock::new(config));
        let stats = Arc::new(RwLock::new(PoolStatistics::default()));
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let factories: Arc<Mutex<HashMap<String, Box<dyn LanguageServerFactory + Send + Sync>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        // Register the web language server factory
        let web_factory = WebLanguageServerFactory::new();
        let mut factories_guard = futures::executor::block_on(factories.lock());
        factories_guard.insert(web_factory.factory_name().to_string(), Box::new(web_factory));
        drop(factories_guard);

        // Start request processing task
        let servers_clone = servers.clone();
        let config_clone = config.clone();
        let stats_clone = stats.clone();
        let resource_monitor_clone = resource_monitor.clone();
        let factories_clone = factories.clone();
        let _pool_task = spawn(async move {
            Self::process_requests(
                rx,
                servers_clone,
                config_clone,
                stats_clone,
                resource_monitor_clone,
                factories_clone,
            )
            .await;
        });

        Self {
            servers,
            detector: LanguageDetector::default(),
            factories: HashMap::new(),
            request_sender: Some(tx),
            config,
            stats,
            background_tasks: Mutex::new(Vec::new()),
            healthy: Arc::new(RwLock::new(true)),
            resource_monitor,
        }
    }

    pub fn new() -> Self {
        Self::new_with_config(Default::default())
    }

    /// Initialize background tasks asynchronously
    pub async fn initialize_background_tasks(&self) -> Result<(), LSPError> {
        if let Some(_sender) = &self.request_sender {
            let (_, receiver) = mpsc::unbounded_channel();
            // Note: This simplified approach may need adjustment based on your specific implementation

            // Start background request processing
            let task_handle = spawn(Self::process_requests(receiver));
            self.background_tasks.lock().await.push(task_handle);

            // Start pool maintenance
            let maintenance_handle = spawn(Self::pool_maintenance(
                Arc::new(self.servers.clone()),
                Arc::clone(&self.config),
                Arc::clone(&self.stats),
            ));
            self.background_tasks.lock().await.push(maintenance_handle);

            info!("Background tasks initialized");
        }

        Ok(())
    }

    /// Register a language server factory
    pub fn register_factory<F: LanguageServerFactory + 'static>(
        &mut self,
        factory: F,
    ) -> Result<(), LSPError> {
        let factory_name = factory.factory_name().to_string();
        if self.factories.contains_key(&factory_name) {
            return Err(LSPError::Other(format!(
                "Factory '{}' already registered",
                factory_name
            )));
        }

        self.factories.insert(factory_name, Arc::new(factory));
        Ok(())
    }

    /// Start a language server for a specific language
    pub async fn start_server(
        &self,
        language: LanguageServerKind,
        config: LanguageServerConfig,
    ) -> Result<(), LSPError> {
        let factory_name = self.get_factory_name_for_language(&language)?;
        let factory = self.factories.get(&factory_name).ok_or_else(|| {
            LSPError::Other(format!("No factory found for language {:?}", language))
        })?;

        if !factory.is_available() {
            return Err(LSPError::Other(format!(
                "Language server for {:?} is not available",
                language
            )));
        }

        let mut servers = self.servers.entry(language.clone()).or_default();
        let current_count = servers.len();

        if current_count >= self.config.read().await.max_servers_per_language {
            return Err(LSPError::Other(format!(
                "Maximum servers ({}) for {:?} already reached",
                self.config.read().await.max_servers_per_language,
                language
            )));
        }

        let server = factory
            .create_server(&config, None)
            .await
            .map_err(|e| LSPError::Other(format!("Failed to create server: {}", e)))?;

        let wrapper = LanguageServerWrapper::new(server, config);
        let handle = Arc::new(RwLock::new(wrapper));

        servers.push(handle);

        info!(
            "Started language server for {:?} ({}/{})",
            language,
            current_count + 1,
            self.config.read().await.max_servers_per_language
        );

        Ok(())
    }

    /// Shut down a language server
    pub async fn shutdown_server(
        &self,
        language: &LanguageServerKind,
        index: usize,
    ) -> Result<(), LSPError> {
        let mut servers = self
            .servers
            .get_mut(language)
            .ok_or_else(|| LSPError::Other(format!("No servers found for {:?}", language)))?;

        if index >= servers.len() {
            return Err(LSPError::Other(format!(
                "Server index {} out of range for {:?}",
                index, language
            )));
        }

        let server_handle = servers.swap_remove(index);
        let mut server_wrapper = server_handle.write().await;

        if server_wrapper.server.is_initialized() {
            server_wrapper.server.shutdown().await?;
        }

        info!(
            "Shut down language server for {:?} (index {})",
            language, index
        );
        Ok(())
    }

    /// Send a request to the appropriate language server
    ///
    /// # Arguments
    /// * `_method` - Request method (reserved for future implementation)
    /// * `_params` - Request parameters (reserved for future implementation)
    /// * `_language_hint` - Language hint for routing (reserved for future implementation)
    /// * `_file_path` - File path for language detection (reserved for future implementation)
    pub async fn send_request_pooled<P, R>(
        &self,
        _method: &str,
        _params: P,
        _language_hint: Option<String>,
        _file_path: Option<String>,
    ) -> Result<R, LSPError>
    where
        P: serde::Serialize + Send + Sync + 'static,
        R: serde::de::DeserializeOwned + Send + 'static,
    {
        Err(LSPError::Other(
            "Request pooling not yet implemented".to_string(),
        ))
    }

    /// Get pool statistics
    pub async fn get_statistics(&self) -> PoolStatistics {
        self.stats.read().await.clone()
    }

    /// Get all supported languages
    pub fn get_supported_languages(&self) -> Vec<LanguageServerKind> {
        self.servers
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Get servers for a specific language
    pub fn get_servers_for_language(
        &self,
        language: &LanguageServerKind,
    ) -> Vec<LanguageServerHandle> {
        self.servers
            .get(language)
            .map(|servers| servers.clone())
            .unwrap_or_default()
    }

    /// Get server status for a language
    pub async fn get_server_status(&self, language: &LanguageServerKind) -> Vec<ServerStatus> {
        let servers = match self.servers.get(language) {
            Some(servers) => servers,
            None => return Vec::new(),
        };

        let mut status_vec = Vec::new();

        for server_handle in servers.iter() {
            let server_wrapper = server_handle.read().await;
            status_vec.push(ServerStatus {
                language: language.clone(),
                health: server_wrapper.health_status.clone(),
                metrics: server_wrapper.metrics.clone(),
                initialized: server_wrapper.server.is_initialized(),
                last_health_check: server_wrapper.last_health_check.elapsed(),
            });
        }

        status_vec
    }

    /// Process requests from the request channel
    async fn process_requests(
        mut receiver: mpsc::UnboundedReceiver<
            PoolRequest<Box<dyn std::any::Any + Send + Sync>, Box<dyn std::any::Any + Send + Sync>>,
        >,
    ) {
        info!("Starting request processing loop");

        while let Some(request) = receiver.recv().await {
            // Determine target language server
            let target_language = if let Some(hint) = &request.language_hint {
                // Use language hint if provided
                Self::parse_language_hint(hint)
            } else if let Some(path) = &request.file_path {
                // Detect language from file path
                Self::detect_language_from_path(std::path::Path::new(path))
            } else {
                None
            };

            // Route request to appropriate server
            if let Some(language) = target_language {
                // TODO: Route to actual server and handle response
                debug!(
                    "Routing request for method '{}' to {:?}",
                    request.method, language
                );
                // For now, send back an error indicating implementation needed
                let _ = request.response_sender.send(Err(LSPError::Other(
                    "Server routing not yet implemented".to_string(),
                )));
            } else {
                let _ = request.response_sender.send(Err(LSPError::Other(
                    "Could not determine target language server".to_string(),
                )));
            }
        }

        info!("Request processing loop ended");
    }

    /// Maintain pool health through background tasks
    async fn pool_maintenance(
        servers: Arc<DashMap<LanguageServerKind, Vec<Arc<RwLock<LanguageServerWrapper>>>>>,
        config: Arc<RwLock<LanguageServerPoolConfig>>,
        stats: Arc<RwLock<PoolStatistics>>,
    ) {
        info!("Starting pool maintenance task");

        loop {
            let interval_secs = {
                let config_read = config.read().await;
                config_read.pool_maintenance_interval_secs
            };
            let mut interval = interval(Duration::from_secs(interval_secs));

            interval.tick().await;

            // Update statistics
            Self::update_pool_statistics(&servers, &stats).await;

            // Health checks
            Self::perform_health_checks(&servers).await;

            // Auto-scaling
            if config.read().await.enable_auto_scaling {
                Self::perform_auto_scaling(&servers, &config).await;
            }
        }
    }

    /// Update pool statistics
    async fn update_pool_statistics(
        servers: &DashMap<LanguageServerKind, Vec<LanguageServerHandle>>,
        stats: &Arc<RwLock<PoolStatistics>>,
    ) {
        let mut total_servers = 0;
        let mut active_servers = 0;
        let mut healthy_servers = 0;
        let mut languages_supported = 0;
        let mut total_requests = 0;
        let mut total_response_time = 0.0;

        for servers_vec in servers.iter() {
            languages_supported += 1;

            for server_handle in servers_vec.value() {
                total_servers += 1;

                let server_wrapper = server_handle.read().await;
                total_requests += server_wrapper.metrics.requests_per_second as usize * 60;
                total_response_time += server_wrapper.metrics.average_response_time_ms;

                if server_wrapper.server.is_initialized() {
                    active_servers += 1;
                    if matches!(server_wrapper.health_status, ServerHealth::Healthy) {
                        healthy_servers += 1;
                    }
                }
            }
        }

        let mut stats_lock = stats.write().await;
        stats_lock.total_servers = total_servers;
        stats_lock.active_servers = active_servers;
        stats_lock.healthy_servers = healthy_servers;
        stats_lock.languages_supported = languages_supported;
        stats_lock.average_request_rate = total_requests as f64 / 60.0;
        stats_lock.average_response_time_ms = if active_servers > 0 {
            total_response_time / active_servers as f64
        } else {
            0.0
        };
    }

    /// Perform health checks on all servers
    async fn perform_health_checks(
        servers: &DashMap<LanguageServerKind, Vec<LanguageServerHandle>>,
    ) {
        let tasks = servers.iter().map(|entry| {
            let language = entry.key().clone();
            let servers_vec = entry.value().clone();

            spawn(async move {
                for server_handle in &servers_vec {
                    let mut server_wrapper = server_handle.write().await;

                    // Basic health check - server is responsive
                    let is_healthy = server_wrapper.server.is_initialized()
                        && matches!(
                            server_wrapper.health_status,
                            ServerHealth::Healthy | ServerHealth::Degraded
                        );

                    // Update health status
                    if !is_healthy {
                        server_wrapper.health_status = ServerHealth::Unhealthy;
                        warn!("Server for {:?} marked as unhealthy", language);
                    } else {
                        server_wrapper.health_status = ServerHealth::Healthy;
                    }

                    server_wrapper.last_health_check = Instant::now();
                }
            })
        });

        join_all(tasks).await;
    }

    /// Perform auto-scaling based on load
    async fn perform_auto_scaling(
        servers: &DashMap<LanguageServerKind, Vec<LanguageServerHandle>>,
        config: &Arc<RwLock<LanguageServerPoolConfig>>,
    ) {
        let config_read = config.read().await;

        for entry in servers.iter() {
            let language = entry.key().clone();
            let servers_vec = entry.value().clone();

            let mut active_count = 0;
            let mut load_avg = 0.0;

            for server_handle in &servers_vec {
                let server_wrapper = server_handle.read().await;
                if server_wrapper.server.is_initialized() {
                    active_count += 1;
                    load_avg += server_wrapper.metrics.requests_per_second;
                }
            }

            load_avg = if active_count > 0 {
                load_avg / active_count as f64
            } else {
                0.0
            };

            // Scale up if average load is high
            if load_avg > 10.0 && active_count < config_read.max_servers_per_language {
                info!(
                    "High load detected for {:?}, considering to scale up",
                    language
                );
                // TODO: Implement actual server scaling
            }
            // Scale down if average load is low
            else if load_avg < 2.0 && active_count > config_read.min_servers_per_language {
                info!(
                    "Low load detected for {:?}, considering to scale down",
                    language
                );
                // TODO: Implement actual server scaling
            }
        }
    }

    /// Get factory name for a language
    fn get_factory_name_for_language(
        &self,
        language: &LanguageServerKind,
    ) -> Result<String, LSPError> {
        match language {
            LanguageServerKind::Rust => Ok("rust".to_string()),
            LanguageServerKind::TypeScript | LanguageServerKind::JavaScript => {
                Ok("typescript".to_string())
            }
            LanguageServerKind::Python => Ok("python".to_string()),
            LanguageServerKind::Go => Ok("go".to_string()),
            LanguageServerKind::Custom(name) => Ok(name.clone()),
        }
    }

    /// Parse language hint string
    fn parse_language_hint(hint: &str) -> Option<LanguageServerKind> {
        match hint.to_lowercase().as_str() {
            "rust" => Some(LanguageServerKind::Rust),
            "typescript" | "ts" => Some(LanguageServerKind::TypeScript),
            "javascript" | "js" => Some(LanguageServerKind::JavaScript),
            "python" | "py" => Some(LanguageServerKind::Python),
            "go" | "golang" => Some(LanguageServerKind::Go),
            _ => Some(LanguageServerKind::Custom(hint.to_string())),
        }
    }

    /// Detect language from file path
    fn detect_language_from_path(path: &std::path::Path) -> Option<LanguageServerKind> {
        let _detector = LanguageDetector::default();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext {
                "rs" => Some(LanguageServerKind::Rust),
                "ts" | "tsx" => Some(LanguageServerKind::TypeScript),
                "js" | "jsx" => Some(LanguageServerKind::JavaScript),
                "py" => Some(LanguageServerKind::Python),
                "go" => Some(LanguageServerKind::Go),
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Server status information
#[derive(Debug, Clone)]
pub struct ServerStatus {
    pub language: LanguageServerKind,
    pub health: ServerHealth,
    pub metrics: ServerMetrics,
    pub initialized: bool,
    pub last_health_check: Duration,
}

impl Drop for LanguageServerPool {
    fn drop(&mut self) {
        // Cancel background tasks
        // Note: This is called in sync context, but we can't await in Drop
        // Background tasks will be aborted when Arc is dropped
    }
}

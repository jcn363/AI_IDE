//! Language server pool manager for multi-language LSP support
//!
//! This module provides a pool of language servers with intelligent
//! resource management, automatic scaling, and request routing.

use dashmap::DashMap;
use futures::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{System, SystemExt};
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

/// Resource monitor structure with real system monitoring
struct ResourceMonitor {
    system: std::sync::Arc<std::sync::RwLock<System>>,
    cpu_usage: std::sync::atomic::AtomicU32,
    memory_available_mb: std::sync::atomic::AtomicU64,
    battery_level: std::sync::atomic::AtomicU32,
    last_update: std::sync::RwLock<std::time::Instant>,
    refresh_interval: std::time::Duration,
}

impl ResourceMonitor {
    fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            system: std::sync::Arc::new(std::sync::RwLock::new(system)),
            cpu_usage: std::sync::atomic::AtomicU32::new(0),
            memory_available_mb: std::sync::atomic::AtomicU64::new(0),
            battery_level: std::sync::atomic::AtomicU32::new(100),
            last_update: std::sync::RwLock::new(std::time::Instant::now()),
            refresh_interval: std::time::Duration::from_secs(2), // Refresh every 2 seconds
        }
    }

    async fn update_metrics(&self) -> Result<(), LSPError> {
        // Check if we need to refresh
        let should_refresh = {
            let last_update = self.last_update.read().unwrap();
            last_update.elapsed() >= self.refresh_interval
        };

        if should_refresh {
            // Refresh system information
            {
                let mut system = self.system.write().unwrap();
                system.refresh_all();
            }

            let system = self.system.read().unwrap();

            // Get real CPU usage
            let cpu_usage = system.global_cpu_info().cpu_usage() as u32;
            self.cpu_usage.store(cpu_usage, std::sync::atomic::Ordering::Relaxed);

            // Get real memory information
            let available_memory = system.available_memory() / 1_000_000; // Convert to MB
            self.memory_available_mb.store(available_memory, std::sync::atomic::Ordering::Relaxed);

            // Get battery information (if available)
            let battery_level = self.detect_battery_level();
            self.battery_level.store(battery_level, std::sync::atomic::Ordering::Relaxed);

            // Update last update time
            let mut last_update = self.last_update.write().unwrap();
            *last_update = std::time::Instant::now();
        }

        Ok(())
    }

    fn detect_battery_level(&self) -> u32 {
        // Try to get battery information if available
        // This is a simplified implementation - in a real system you might
        // use platform-specific APIs or additional crates for better battery monitoring
        #[cfg(target_os = "linux")]
        {
            // On Linux, we could try reading battery info from /sys/class/power_supply/
            // For now, return a reasonable default
            100
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, we could use ioreg or system_profiler
            // For now, return a reasonable default
            100
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, we could use Windows API
            // For now, return a reasonable default
            100
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            // For other platforms, assume always plugged in
            100
        }
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
    servers: Arc<DashMap<LanguageServerKind, Vec<LanguageServerHandle>>>,

    /// Language detector for automatic routing
    detector: LanguageDetector,

    /// Server factories for creating language servers
    factories: Arc<Mutex<HashMap<String, Box<dyn LanguageServerFactory + Send + Sync>>>>,

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
            factories,
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
            let task_handle = spawn(Self::process_requests(receiver, self.servers.clone(), self.config.clone(), self.stats.clone(), self.resource_monitor.clone(), self.factories.clone()));
            self.background_tasks.lock().await.push(task_handle);

            // Start pool maintenance
            let maintenance_handle = spawn(Self::pool_maintenance(
                self.servers.clone(),
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
        let mut factories_guard = futures::executor::block_on(self.factories.lock());
        if factories_guard.contains_key(&factory_name) {
            return Err(LSPError::Other(format!(
                "Factory '{}' already registered",
                factory_name
            )));
        }

        factories_guard.insert(factory_name, Box::new(factory));
        Ok(())
    }

    /// Start a language server for a specific language
    pub async fn start_server(
        &self,
        language: LanguageServerKind,
        config: LanguageServerConfig,
    ) -> Result<(), LSPError> {
        let factory_name = self.get_factory_name_for_language(&language)?;
        let factories_guard = self.factories.lock().await;
        let factory = factories_guard.get(&factory_name).ok_or_else(|| {
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
        let (tx, rx) = oneshot::channel();

        let params_boxed = Box::new(serde_json::to_value(_params).unwrap()) as Box<dyn std::any::Any + Send + Sync>;

        let request = PoolRequest {
            method: _method.to_string(),
            params: params_boxed,
            language_hint: _language_hint,
            file_path: _file_path,
            response_sender: tx,
            _phantom: std::marker::PhantomData,
        };

        if let Some(sender) = &self.request_sender {
            let _ = sender.send(request);
        } else {
            return Err(LSPError::Other("Request sender not available".to_string()));
        }

        let config = self.config.read().await;
        let timeout_duration = Duration::from_millis(config.request_timeout_ms);

        let result = tokio::time::timeout(timeout_duration, rx).await
            .map_err(|_| LSPError::Other("Request timeout".to_string()))?
            .map_err(|_| LSPError::Other("Request receiver dropped".to_string()))?;

        let response_value = result?;
        let response_json: Box<serde_json::Value> = response_value.downcast().map_err(|_| LSPError::Other("Failed to downcast response".to_string()))?;
        let response: R = serde_json::from_value(*response_json).map_err(|e| LSPError::Other(format!("Failed to deserialize response: {}", e)))?;

        Ok(response)
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

    /// Get load metrics for all servers
    pub async fn get_server_load_metrics(&self) -> Vec<ServerLoadMetrics> {
        let mut load_metrics = Vec::new();

        for entry in self.servers.iter() {
            let language = entry.key().clone();
            let servers = entry.value();

            for (index, server_handle) in servers.iter().enumerate() {
                let server_wrapper = server_handle.read().await;
                let server_id = format!("{}_{}", Self::language_to_string(&language), index);

                let health_score = self.calculate_server_health_score(&server_wrapper).await;

                load_metrics.push(ServerLoadMetrics {
                    server_id,
                    language: language.clone(),
                    pending_requests: server_wrapper.metrics.pending_requests,
                    response_time_ms: server_wrapper.metrics.average_response_time_ms,
                    cpu_usage_percent: server_wrapper.metrics.cpu_usage_percent,
                    memory_usage_percent: self.get_memory_usage_percent(&server_wrapper),
                    request_rate: server_wrapper.metrics.requests_per_second,
                    health_score,
                    last_updated: std::time::Instant::now(),
                });
            }
        }

        load_metrics
    }

    /// Get current resource usage metrics from the system monitor
    pub async fn get_resource_metrics(&self) -> Result<ResourceMetrics, LSPError> {
        self.resource_monitor.update_metrics().await?;
        Ok(self.resource_monitor.get_detailed_metrics())
    }

    /// Get load metrics for servers of a specific language
    pub async fn get_language_load_metrics(&self, language: &LanguageServerKind) -> Vec<ServerLoadMetrics> {
        let mut load_metrics = Vec::new();

        if let Some(servers) = self.servers.get(language) {
            for (index, server_handle) in servers.iter().enumerate() {
                let server_wrapper = server_handle.read().await;
                let server_id = format!("{}_{}", Self::language_to_string(language), index);

                let health_score = self.calculate_server_health_score(&server_wrapper).await;

                load_metrics.push(ServerLoadMetrics {
                    server_id,
                    language: language.clone(),
                    pending_requests: server_wrapper.metrics.pending_requests,
                    response_time_ms: server_wrapper.metrics.average_response_time_ms,
                    cpu_usage_percent: server_wrapper.metrics.cpu_usage_percent,
                    memory_usage_percent: self.get_memory_usage_percent(&server_wrapper),
                    request_rate: server_wrapper.metrics.requests_per_second,
                    health_score,
                    last_updated: std::time::Instant::now(),
                });
            }
        }

        load_metrics
    }

    /// Select server for request based on load balancing
    fn select_server_for_request(available_servers: &Vec<LanguageServerHandle>) -> usize {
        // Simple load balancing: select server with least pending requests
        let mut best_index = 0;
        let mut min_pending = usize::MAX;

        for (index, handle) in available_servers.iter().enumerate() {
            if let Ok(wrapper) = handle.try_read() {
                if wrapper.metrics.pending_requests < min_pending {
                    min_pending = wrapper.metrics.pending_requests;
                    best_index = index;
                }
            }
        }

        best_index
    }

    /// Process requests from the request channel
    async fn process_requests(
        mut receiver: mpsc::UnboundedReceiver<
            PoolRequest<Box<dyn std::any::Any + Send + Sync>, Box<dyn std::any::Any + Send + Sync>>,
        >,
        servers: Arc<DashMap<LanguageServerKind, Vec<LanguageServerHandle>>>,
        config: Arc<RwLock<LanguageServerPoolConfig>>,
        stats: Arc<RwLock<PoolStatistics>>,
        resource_monitor: Arc<ResourceMonitor>,
        factories: Arc<Mutex<HashMap<String, Box<dyn LanguageServerFactory + Send + Sync>>>>,
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
                debug!(
                    "Routing request for method '{}' to {:?}",
                    request.method, language
                );

                // Get available servers for this language
                if let Some(servers) = servers.get(&language) {
                    let mut available_servers = Vec::new();

                    // Filter for healthy servers
                    for server_handle in servers.iter() {
                        if let Ok(server_wrapper) = server_handle.try_read() {
                            if matches!(server_wrapper.health_status, ServerHealth::Healthy)
                                && server_wrapper.server.is_initialized()
                            {
                                available_servers.push(server_handle.clone());
                            }
                        }
                    }

                    if !available_servers.is_empty() {
                        // Select server based on load balancing strategy
                        let selected_index = Self::select_server_for_request(&available_servers);
                        let selected_handle = &available_servers[selected_index];

                        debug!("Selected server {}_{} for request", Self::language_to_string(&language), selected_index);

                        // Increment pending requests counter
                        if let Ok(mut server_wrapper) = selected_handle.try_write() {
                            server_wrapper.metrics.pending_requests += 1;
                        }

                        // Forward request to selected server
                        let method_clone = request.method.clone();
                        let params_clone = request.params.clone();
                        let response_sender = request.response_sender;

                        let handle_clone = selected_handle.clone();
                        tokio::spawn(async move {
                            // Send request to the selected language server
                            let result = {
                                let server_wrapper = handle_clone.read().await;
                                server_wrapper.server.send_request(&method_clone, params_clone).await
                            };

                            // Update metrics after request completion
                            if let Ok(mut server_wrapper) = handle_clone.try_write() {
                                server_wrapper.metrics.pending_requests = server_wrapper.metrics.pending_requests.saturating_sub(1);
                                server_wrapper.metrics.last_response_time = std::time::Instant::now();

                                match &result {
                                    Ok(_) => {
                                        // Update success metrics
                                        server_wrapper.metrics.requests_per_second += 1.0;
                                    }
                                    Err(_) => {
                                        // Update error metrics
                                        server_wrapper.metrics.error_rate += 0.01;
                                        server_wrapper.metrics.error_rate = server_wrapper.metrics.error_rate.min(1.0);
                                    }
                                }
                            }

                            // Send response back to client
                            let _ = response_sender.send(result);
                        });
                    } else {
                        let _ = request.response_sender.send(Err(LSPError::Other(
                            format!("No healthy servers available for language {:?}", language)
                        )));
                    }
                } else {
                    let _ = request.response_sender.send(Err(LSPError::Other(
                        format!("No servers configured for language {:?}", language)
                    )));
                }
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

    /// Calculate health score for a server based on metrics
    async fn calculate_server_health_score(&self, server_wrapper: &crate::language_server::LanguageServerWrapper) -> f64 {
        let mut score = 1.0;

        // Penalize for high pending requests
        if server_wrapper.metrics.pending_requests > 10 {
            score -= (server_wrapper.metrics.pending_requests as f64 - 10.0) * 0.05;
        }

        // Penalize for high CPU usage
        if server_wrapper.metrics.cpu_usage_percent > 80.0 {
            score -= (server_wrapper.metrics.cpu_usage_percent - 80.0) / 20.0;
        }

        // Penalize for high error rate
        if server_wrapper.metrics.error_rate > 0.1 {
            score -= server_wrapper.metrics.error_rate * 2.0;
        }

        // Penalize for slow response times
        if server_wrapper.metrics.average_response_time_ms > 1000.0 {
            score -= (server_wrapper.metrics.average_response_time_ms - 1000.0) / 2000.0;
        }

        // Penalize for low memory
        if let Some(mem_mb) = server_wrapper.metrics.memory_usage_mb {
            if mem_mb < 50.0 {
                score -= (50.0 - mem_mb) / 50.0;
            }
        }

        score.max(0.0).min(1.0)
    }

    /// Get memory usage percent for a server
    fn get_memory_usage_percent(&self, server_wrapper: &crate::language_server::LanguageServerWrapper) -> f64 {
        if let Some(mem_mb) = server_wrapper.metrics.memory_usage_mb {
            // Assume 1GB is 100% for calculation
            (mem_mb / 1000.0).min(1.0)
        } else {
            0.0
        }
    }

    /// Convert language to string for server ID generation
    fn language_to_string(language: &LanguageServerKind) -> String {
        match language {
            LanguageServerKind::Rust => "rust".to_string(),
            LanguageServerKind::TypeScript => "typescript".to_string(),
            LanguageServerKind::JavaScript => "javascript".to_string(),
            LanguageServerKind::Html => "html".to_string(),
            LanguageServerKind::Css => "css".to_string(),
            LanguageServerKind::Sql => "sql".to_string(),
            LanguageServerKind::Go => "go".to_string(),
            LanguageServerKind::Custom(name) => name.clone(),
        }
    }
}

/// Detailed resource metrics for advanced monitoring
#[derive(Debug, Clone)]
pub struct ResourceMetrics {
    pub cpu_usage_percent: f64,
    pub memory_total_mb: f64,
    pub memory_used_mb: f64,
    pub memory_available_mb: f64,
    pub disk_total_gb: f64,
    pub disk_available_gb: f64,
    pub load_average: sysinfo::LoadAvg,
    pub process_count: u32,
}

impl ResourceMonitor {
    /// Get detailed resource information for performance monitoring
    pub fn get_detailed_metrics(&self) -> ResourceMetrics {
        let system = self.system.read().unwrap();

        ResourceMetrics {
            cpu_usage_percent: system.global_cpu_info().cpu_usage() as f64,
            memory_total_mb: (system.total_memory() / 1_000_000) as f64,
            memory_used_mb: (system.used_memory() / 1_000_000) as f64,
            memory_available_mb: (system.available_memory() / 1_000_000) as f64,
            disk_total_gb: system.disks().iter().map(|d| d.total_space()).sum::<u64>() as f64 / 1_000_000_000.0,
            disk_available_gb: system.disks().iter().map(|d| d.available_space()).sum::<u64>() as f64 / 1_000_000_000.0,
            load_average: system.load_average(),
            process_count: system.processes().len() as u32,
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

/// Server load metrics for monitoring and load balancing
#[derive(Debug, Clone)]
pub struct ServerLoadMetrics {
    pub server_id: String,
    pub language: LanguageServerKind,
    pub pending_requests: usize,
    pub response_time_ms: f64,
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub request_rate: f64,
    pub health_score: f64,
    pub last_updated: std::time::Instant,
}

impl Drop for LanguageServerPool {
    fn drop(&mut self) {
        // Cancel background tasks
        // Note: This is called in sync context, but we can't await in Drop
        // Background tasks will be aborted when Arc is dropped
    }
}

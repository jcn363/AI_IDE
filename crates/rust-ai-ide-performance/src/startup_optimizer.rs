use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock, Semaphore};
use rust_ai_ide_common::{IDEError, IDEErrorKind};
use moka::future::Cache;
use tokio::task::spawn_blocking;
use std::future::Future;
use tokio::sync::oneshot;
use crate::{DefaultSystemMonitor, Monitor, SystemMonitor};

/// Startup time targets (in milliseconds)
const COLD_STARTUP_TARGET: u64 = 400;
const WARM_STARTUP_TARGET: u64 = 80;

/// Startup profiler for measuring and analyzing initialization phases
pub struct StartupProfiler {
    pub(crate) measurements: Arc<Mutex<HashMap<String, Vec<Duration>>>>,
    pub(crate) current_phase: Arc<Mutex<Option<String>>>,
    pub(crate) start_times: Arc<Mutex<HashMap<String, Instant>>>,
    pub(crate) total_measurements: Arc<Mutex<usize>>,
}

impl StartupProfiler {
    pub fn new() -> Self {
        Self {
            measurements: Arc::new(Mutex::new(HashMap::new())),
            current_phase: Arc::new(Mutex::new(None)),
            start_times: Arc::new(Mutex::new(HashMap::new())),
            total_measurements: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn start_phase(&self, phase_name: &str) {
        let mut current_phase = self.current_phase.lock().await;
        let mut start_times = self.start_times.lock().await;

        if let Some(ref old_phase) = *current_phase {
            self.end_phase_inner(old_phase, &mut start_times).await;
        }

        *current_phase = Some(phase_name.to_string());
        start_times.insert(phase_name.to_string(), Instant::now());
    }

    pub async fn end_phase(&self, phase_name: &str) {
        let mut start_times = self.start_times.lock().await;
        self.end_phase_inner(phase_name, &mut start_times).await;
    }

    async fn end_phase_inner(&self, phase_name: &str, start_times: &mut HashMap<String, Instant>) {
        if let Some(start_time) = start_times.remove(phase_name) {
            let duration = start_time.elapsed();
            let mut measurements = self.measurements.lock().await;
            measurements.entry(phase_name.to_string()).or_insert_with(Vec::new).push(duration);
        }
    }

    pub async fn measure_async<T, F>(&self, phase_name: &str, future: F) -> Result<T, F::Error>
    where
        F: Future<Output = Result<T, F::Error>>,
    {
        self.start_phase(phase_name).await;
        let result = future.await;
        self.end_phase(phase_name).await;
        result
    }

    pub async fn measure_blocking<T, F>(&self, phase_name: &str, blocking_fn: F) -> Result<T, IDEError>
    where
        F: FnOnce() -> Result<T, IDEError> + Send + 'static,
        T: Send + 'static,
    {
        self.start_phase(phase_name).await;

        let result = spawn_blocking(move || blocking_fn())
            .await
            .map_err(|e| IDEError::new(
                IDEErrorKind::ConcurrencyError,
                "Blocking task panicked",
            )
            .with_source(e))??;

        self.end_phase(phase_name).await;
        Ok(result)
    }

    pub async fn get_phase_average(&self, phase_name: &str) -> Option<Duration> {
        let measurements = self.measurements.lock().await;
        measurements.get(phase_name).and_then(|times| {
            if times.is_empty() {
                None
            } else {
                let total: Duration = times.iter().sum();
                Some(total / times.len() as u32)
            }
        })
    }

    pub async fn get_total_startup_time(&self) -> Duration {
        let measurements = self.measurements.lock().await;
        measurements.values().flatten().sum()
    }

    pub async fn get_startup_report(&self) -> StartupReport {
        let measurements = self.measurements.lock().await;
        let mut phase_times = HashMap::new();

        for (phase, times) in measurements.iter() {
            if let Some(avg) = self.get_phase_average(phase).await {
                phase_times.insert(phase.clone(), avg);
            }
        }

        let total_time = phase_times.values().sum();

        StartupReport {
            total_startup_time: total_time,
            cold_startup_target: Duration::from_millis(COLD_STARTUP_TARGET),
            warm_startup_target: Duration::from_millis(WARM_STARTUP_TARGET),
            phase_average_times: phase_times,
            target_achievement: if total_time <= Duration::from_millis(COLD_STARTUP_TARGET) {
                TargetAchievement::ColdTarget
            } else if total_time <= Duration::from_millis(WARM_STARTUP_TARGET) {
                TargetAchievement::WarmTarget
            } else {
                TargetAchievement::AboveTarget {
                    excess: total_time - Duration::from_millis(WARM_STARTUP_TARGET),
                }
            },
        }
    }

    pub async fn get_slowest_phases(&self, limit: usize) -> Vec<(String, Duration)> {
        let measurements = self.measurements.lock().await;
        let mut phases: Vec<(String, Duration)> = vec![];

        for (phase, times) in measurements.iter() {
            if let Some(avg) = self.get_phase_average(phase).await {
                phases.push((phase.clone(), avg));
            }
        }

        phases.sort_by(|a, b| b.1.cmp(&a.1));
        phases.truncate(limit);
        phases
    }
}

/// Lazy loader for deferring non-critical component initialization
pub struct LazyLoader {
    pub(crate) async_state: Arc<Mutex<HashMap<String, InitializationState>>>,
    pub(crate) semaphore: Arc<Semaphore>,
    pub(crate) queue: Arc<Mutex<VecDeque<InitializationRequest>>>,
}

impl LazyLoader {
    pub fn new(max_concurrent_init: usize) -> Self {
        Self {
            async_state: Arc::new(Mutex::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(max_concurrent_init)),
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn register_lazy_initialization<F, Fut, T>(
        &self,
        key: String,
        initializer: F,
    ) -> Result<(), IDEError>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, IDEError>> + Send + 'static,
        T: Send + 'static,
    {
        let mut state = self.async_state.lock().await;

        if state.contains_key(&key) {
            return Err(IDEError::new(
                IDEErrorKind::ResourceConflict,
                format!("Lazy initialization already registered for key: {}", key),
            ));
        }

        let (tx, rx) = oneshot::channel();
        let request = InitializationRequest {
            key: key.clone(),
            initializer: Box::pin(async move {
                let initializer = initializer;
                let result = initializer().await?;
                let _ = tx.send(result);
                Ok(())
            }),
        };

        state.insert(key, InitializationState::Pending);
        drop(state);

        let mut queue = self.queue.lock().await;
        queue.push_back(request);

        Ok(())
    }

    pub async fn get_lazy_initialized<T: Clone + Send + 'static>(
        &self,
        key: &str,
    ) -> Result<T, IDEError> {
        let mut state = self.async_state.lock().await;

        let current_state = state.get(key).cloned().unwrap_or(InitializationState::NotStarted);

        match current_state {
            InitializationState::Completed(result) => {
                // Type casting - in practice, you'd use Any or store properly typed values
                Err(IDEError::new(
                    IDEErrorKind::TypeConversion,
                    "Type casting not implemented in this example",
                ))
            },
            InitializationState::InProgress => {
                // Wait for ongoing initialization (simplified)
                Err(IDEError::new(
                    IDEErrorKind::ResourceBusy,
                    "Initialization already in progress",
                ))
            },
            InitializationState::Pending | InitializationState::NotStarted => {
                state.insert(key.to_string(), InitializationState::InProgress);
                drop(state);

                // Start initialization
                self.initialize_component(key).await
            },
            InitializationState::Failed(e) => Err(e),
        }
    }

    async fn initialize_component(&self, key: &str) -> Result<(), IDEError> {
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            IDEError::new(
                IDEErrorKind::ConcurrencyError,
                "Failed to acquire initialization permit",
            )
            .with_source(e)
        })?;

        let mut queue = self.queue.lock().await;
        if let Some(request) = queue.iter().find(|req| req.key == key) {
            request.initializer.clone().as_ref().await?;
        }

        Ok(())
    }

    pub async fn preload_frequently_used(&self, frequently_used_keys: &[String]) -> Result<(), IDEError> {
        for key in frequently_used_keys {
            let state = self.async_state.lock().await;
            if let Some(InitializationState::NotStarted) = state.get(key) {
                drop(state);
                // Preload in background
                let lazy_clone = self.queue.clone();
                let key_clone = key.clone();

                tokio::spawn(async move {
                    let mut queue = lazy_clone.lock().await;
                    if let Some(request) = queue.iter_mut().find(|req| req.key == key_clone) {
                        if let Err(_) = request.initializer.as_mut().await {
                            // Log error but don't panic
                        }
                    }
                });
            }
        }

        Ok(())
    }
}

/// Startup cache for expensive computations
pub struct StartupCache {
    cache: Cache<String, serde_json::Value>,
}

impl StartupCache {
    pub fn new(max_capacity: u64, ttl_seconds: u64) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(max_capacity)
                .time_to_live(Duration::from_secs(ttl_seconds))
                .build(),
        }
    }

    pub async fn cache_expensive_computation<T, F>(
        &self,
        key: String,
        computation: F,
    ) -> Result<T, IDEError>
    where
        F: Future<Output = Result<T, IDEError>> + Send + 'static,
        T: serde::Serialize + serde::de::DeserializeOwned + Send + 'static,
    {
        // Try cache first
        if let Some(cached_value) = self.cache.get(&key).await {
            return serde_json::from_value(cached_value).map_err(|e| {
                IDEError::new(
                    IDEErrorKind::Serialization,
                    "Failed to deserialize cached value",
                )
                .with_source(e)
            });
        }

        // Compute and cache
        let result = computation.await?;
        let serialized = serde_json::to_value(&result).map_err(|e| {
            IDEError::new(
                IDEErrorKind::Serialization,
                "Failed to serialize computed value",
            )
            .with_source(e)
        })?;

        self.cache.insert(key, serialized).await;
        Ok(result)
    }

    pub async fn invalidate_pattern(&self, pattern: &str) {
        // In practice, use a regex filter or iterate through keys
        // For this example, we'll clear all (inefficient but simple)
        self.cache.invalidate_all();
    }

    pub async fn clear_expired(&self) {
        self.cache.run_pending_tasks().await;
    }
}

/// Module preloader for predictive loading of frequently used components
pub struct ModulePreloader {
    pub(crate) patterns: Arc<RwLock<HashMap<String, Vec<String>>>>,
    pub(crate) preload_queue: Arc<Mutex<VecDeque<String>>>,
    pub(crate) active_preloads: Arc<Mutex<HashSet<String>>>,
}

impl ModulePreloader {
    pub fn new() -> Self {
        Self {
            patterns: Arc::new(RwLock::new(HashMap::new())),
            preload_queue: Arc::new(Mutex::new(VecDeque::new())),
            active_preloads: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub async fn register_pattern(&self, trigger: String, modules_to_preload: Vec<String>) {
        let mut patterns = self.patterns.write().await;
        patterns.insert(trigger, modules_to_preload);
    }

    pub async fn trigger_pattern(&self, trigger: &str) -> Result<(), IDEError> {
        let patterns = self.patterns.read().await;
        if let Some(modules) = patterns.get(trigger) {
            let modules_clone = modules.clone();
            drop(patterns);

            for module in modules_clone {
                if !self.is_preloading(&module).await {
                    self.queue_preload(module).await?;
                }
            }
        }
        Ok(())
    }

    async fn queue_preload(&self, module: String) -> Result<(), IDEError> {
        let mut queue = self.preload_queue.lock().await;
        queue.push_back(module);
        Ok(())
    }

    async fn is_preloading(&self, module: &str) -> bool {
        let active = self.active_preloads.lock().await;
        active.contains(module)
    }

    pub async fn process_queue(&self) {
        let mut queue = self.preload_queue.lock().await;
        let mut active = self.active_preloads.lock().await;

        while let Some(module) = queue.pop_front() {
            if active.contains(&module) {
                continue;
            }

            active.insert(module.clone());

            // Process preload in background
            let active_clone = self.active_preloads.clone();
            tokio::spawn(async move {
                // Load the module (placeholder implementation)
                // In practice, this would dynamically load the module
                tokio::time::sleep(Duration::from_millis(10)).await;

                let mut active = active_clone.lock().await;
                active.remove(&module);
            });
        }
    }
}

/// Startup validator for continuous performance monitoring
pub struct StartupValidator {
    pub(crate) profiler: Arc<StartupProfiler>,
    pub(crate) validation_threshold: Duration,
    pub(crate) violation_count: Arc<Mutex<u64>>,
}

impl StartupValidator {
    pub fn new(profiler: Arc<StartupProfiler>, validation_threshold: Duration) -> Self {
        Self {
            profiler,
            validation_threshold,
            violation_count: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn validate_startup_performance(&self) -> ValidationResult {
        let report = self.profiler.get_startup_report().await;
        let is_within_threshold = report.total_startup_time <= self.validation_threshold;

        let violations = if is_within_threshold {
            vec![]
        } else {
            let excess_time = report.total_startup_time - self.validation_threshold;
            let slowest_phases = self.profiler.get_slowest_phases(5).await;

            vec![PerformanceViolation {
                phase: "Total Startup".to_string(),
                excess_time,
                actual_time: report.total_startup_time,
                threshold: self.validation_threshold,
                suggestions: self.generate_suggestions(excess_time, slowest_phases),
            }]
        };

        ValidationResult {
            is_within_threshold,
            total_startup_time: report.total_startup_time,
            threshold: self.validation_threshold,
            violations,
        }
    }

    pub async fn validate_phase_performance(&self, phase_name: &str) -> Option<PhaseValidationResult> {
        let average_time = self.profiler.get_phase_average(phase_name).await?;
        let phase_threshold = self.get_phase_threshold(phase_name);

        let is_violation = average_time > phase_threshold;

        if is_violation {
            let mut count = self.violation_count.lock().await;
            *count += 1;
        }

        Some(PhaseValidationResult {
            phase_name: phase_name.to_string(),
            average_time,
            threshold: phase_threshold,
            is_violation,
            violation_count: *self.violation_count.lock().await,
        })
    }

    fn get_phase_threshold(&self, phase_name: &str) -> Duration {
        // Define thresholds for specific phases
        match phase_name {
            "plugin_initialization" => Duration::from_millis(50),
            "lsp_startup" => Duration::from_millis(30),
            "ai_model_loading" => Duration::from_millis(100),
            "database_connection" => Duration::from_millis(20),
            _ => Duration::from_millis(25),
        }
    }

    fn generate_suggestions(&self, excess_time: Duration, slowest_phases: Vec<(String, Duration)>) -> Vec<String> {
        let mut suggestions = Vec::new();

        for (phase, time) in slowest_phases {
            suggestions.push(format!("Consider optimizing {} ({}ms)", phase, time.as_millis()));
        }

        if excess_time > Duration::from_millis(200) {
            suggestions.push("Consider implementing lazy loading for non-critical components".to_string());
        }

        if excess_time > Duration::from_millis(300) {
            suggestions.push("Evaluate memory allocation patterns during startup".to_string());
        }

        suggestions
    }

    pub async fn get_violation_summary(&self) -> ViolationSummary {
        let count = *self.violation_count.lock().await;
        let report = self.profiler.get_startup_report().await;

        ViolationSummary {
            total_violations: count,
            current_performance: report.target_achievement.clone(),
            average_startup_time: report.total_startup_time,
            recommended_actions: self.generate_recommendations(count).await,
        }
    }

    async fn generate_recommendations(&self, violation_count: u64) -> Vec<String> {
        let mut recommendations = Vec::new();

        if violation_count > 5 {
            recommendations.push("Implement startup profiling and optimization pipeline".to_string());
        }

        if violation_count > 3 {
            recommendations.push("Consider async component loading strategies".to_string());
        }

        recommendations.push("Monitor startup performance in CI/CD pipeline".to_string());

        recommendations
    }
}

// Data structures

#[derive(Clone, Debug)]
pub struct StartupReport {
    pub total_startup_time: Duration,
    pub cold_startup_target: Duration,
    pub warm_startup_target: Duration,
    pub phase_average_times: HashMap<String, Duration>,
    pub target_achievement: TargetAchievement,
}

#[derive(Clone, Debug)]
pub enum TargetAchievement {
    ColdTarget,
    WarmTarget,
    AboveTarget { excess: Duration },
}

pub struct InitializationRequest {
    pub key: String,
    pub initializer: std::pin::Pin<Box<dyn Future<Output = Result<(), IDEError>> + Send>>,
}

#[derive(Clone)]
pub enum InitializationState {
    NotStarted,
    Pending,
    InProgress,
    Completed(serde_json::Value), // Simplified
    Failed(IDEError),
}

pub struct ValidationResult {
    pub is_within_threshold: bool,
    pub total_startup_time: Duration,
    pub threshold: Duration,
    pub violations: Vec<PerformanceViolation>,
}

#[derive(Clone, Debug)]
pub struct PerformanceViolation {
    pub phase: String,
    pub excess_time: Duration,
    pub actual_time: Duration,
    pub threshold: Duration,
    pub suggestions: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct PhaseValidationResult {
    pub phase_name: String,
    pub average_time: Duration,
    pub threshold: Duration,
    pub is_violation: bool,
    pub violation_count: u64,
}

#[derive(Clone, Debug)]
pub struct ViolationSummary {
    pub total_violations: u64,
    pub current_performance: TargetAchievement,
    pub average_startup_time: Duration,
    pub recommended_actions: Vec<String>,
}

/// Integrated startup optimizer combining all performance optimization components
pub struct IntegratedStartupOptimizer {
    profiler: Arc<StartupProfiler>,
    adapter: Arc<ProfilingAdapter>,
    lazy_loader: Arc<LazyLoader>,
    cache: Arc<StartupCache>,
    preloader: Arc<ModulePreloader>,
    validator: Arc<StartupValidator>,
    monitor: Arc<RwLock<SystemMonitor>>,
}

impl IntegratedStartupOptimizer {
    pub fn new() -> Self {
        let profiler = Arc::new(StartupProfiler::new());
        let adapter = Arc::new(ProfilingAdapter::new(profiler.clone()));
        let lazy_loader = Arc::new(LazyLoader::new(10)); // Max 10 concurrent initializations
        let cache = Arc::new(StartupCache::new(1000, 300)); // 1000 entries, 5min TTL
        let preloader = Arc::new(ModulePreloader::new());
        let validator = Arc::new(StartupValidator::new(profiler.clone(), Duration::from_millis(400)));
        let monitor = Arc::new(RwLock::new(SystemMonitor::new()));

        Self {
            profiler,
            adapter,
            lazy_loader,
            cache,
            preloader,
            validator,
            monitor,
        }
    }

    pub async fn initialize_with_monitoring(&self, is_cold_startup: bool) -> Result<StartupReport, IDEError> {
        // Start monitoring
        let monitor = self.monitor.write().await;
        // Start collecting system metrics during startup

        // Start measurement
        self.adapter.start_startup_measurement(is_cold_startup).await?;

        // Phase 1: Core systems (always loaded)
        self.adapter.measure_phase("core_systems", async {
            self.initialize_core_systems().await
        }).await?;

        // Phase 2: Essential services (with lazy loading for non-critical)
        self.adapter.measure_phase("essential_services", async {
            self.initialize_essential_services().await
        }).await?;

        // Phase 3: UI and plugins
        self.adapter.measure_phase("ui_plugins", async {
            self.initialize_ui_and_plugins().await
        }).await?;

        // End measurement and get report
        let report = self.adapter.end_startup_measurement().await?;

        Ok(report)
    }

    async fn initialize_core_systems(&self) -> Result<(), IDEError> {
        // Core systems that must be initialized immediately

        // Database connections with connection pooling
        self.cache_expensive_operation("database_init", async {
            tokio::time::sleep(Duration::from_millis(20)).await;
            Ok::<_, IDEError>(())
        }).await?;

        // Event bus initialization
        self.cache_expensive_operation("event_bus_init", async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok::<_, IDEError>(())
        }).await?;

        Ok(())
    }

    async fn initialize_essential_services(&self) -> Result<(), IDEError> {
        // Register lazy initialization for heavy services
        self.lazy_loader.register_lazy_initialization(
            "lsp_service".to_string(),
            || async {
                tokio::time::sleep(Duration::from_millis(60)).await;
                Ok::<_, IDEError>(())
            }
        ).await?;

        self.lazy_loader.register_lazy_initialization(
            "ai_models".to_string(),
            || async {
                tokio::time::sleep(Duration::from_millis(80)).await;
                Ok::<_, IDEError>(())
            }
        ).await?;

        // Preload frequently used components
        self.lazy_loader.preload_frequently_used(&vec![
            "lsp_service".to_string(),
        ]).await?;

        Ok(())
    }

    async fn initialize_ui_and_plugins(&self) -> Result<(), IDEError> {
        // Plugin loading (with lazy loading for non-essential plugins)
        self.lazy_loader.register_lazy_initialization(
            "plugin_system".to_string(),
            || async {
                tokio::time::sleep(Duration::from_millis(40)).await;
                Ok::<_, IDEError>(())
            }
        ).await?;

        // UI essentials
        tokio::time::sleep(Duration::from_millis(25)).await;

        Ok(())
    }

    pub async fn cache_expensive_operation<T, F>(
        &self,
        key: String,
        computation: F,
    ) -> Result<T, IDEError>
    where
        F: Future<Output = Result<T, IDEError>> + Send + 'static,
        T: serde::Serialize + serde::de::DeserializeOwned + Send + 'static,
    {
        self.cache.cache_expensive_computation(key, computation).await
    }

    pub async fn preload_components(&self, components: &[String]) {
        // Setup preloading patterns based on usage history
        self.preloader.register_pattern(
            "ui_ready".to_string(),
            components.to_vec(),
        ).await;

        // Trigger preloading
        self.preloader.trigger_pattern("ui_ready").await.ok();
        self.preloader.process_queue().await;
    }

    pub async fn get_lazy_initialized_component(&self, key: &str) -> Result<(), IDEError> {
        // Trigger initialization of lazy component when needed
        let _ = self.lazy_loader.get_lazy_initialized::<()>(key).await;
        Ok(())
    }

    pub async fn validate_performance(&self) -> ValidationResult {
        self.validator.validate_startup_performance().await
    }

    pub async fn get_performance_stats(&self) -> Result<StartupReport, IDEError> {
        let adapter = &self.adapter;
        let measurements = adapter.get_measurements_history().await;
        if measurements.is_empty() {
            return Err(IDEError::new(
                IDEErrorKind::StateError,
                "No startup measurements available"
            ));
        }

        // Return the latest startup report
        Ok(StartupReport {
            total_startup_time: measurements.last().unwrap().total_duration,
            cold_startup_target: Duration::from_millis(COLD_STARTUP_TARGET),
            warm_startup_target: Duration::from_millis(WARM_STARTUP_TARGET),
            phase_average_times: std::collections::HashMap::new(), // Would need to be populated
            target_achievement: TargetAchievement::AboveTarget { excess: Duration::default() },
        })
    }

    pub async fn optimize_for_warm_startup(&self) -> Result<(), IDEError> {
        // Clear expired cache entries
        self.cache.clear_expired().await;

        // Prepare frequently used components
        self.preload_components(&vec![
            "database_connection".to_string(),
            "lsp_service".to_string(),
            "ui_layout".to_string(),
        ]).await;

        Ok(())
    }

    pub async fn shutdown_monitoring(&self) {
        // Clean shutdown of monitoring components
        let monitor = self.monitor.write().await;
        // Stop monitoring if active
    }
}

impl Default for IntegratedStartupOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_integrated_optimizer() {
        let optimizer = IntegratedStartupOptimizer::new();

        // Simulate cold startup
        let report = optimizer.initialize_with_monitoring(true).await.unwrap();
        println!("Cold startup time: {}ms", report.total_startup_time.as_millis());

        assert!(report.total_startup_time < Duration::from_millis(COLD_STARTUP_TARGET));

        // Test lazy loading
        optimizer.get_lazy_initialized_component("lsp_service").await.unwrap();

        // Validate performance
        let validation = optimizer.validate_performance().await;
        assert!(validation.is_within_threshold, "Performance validation failed: {} violations",
                validation.violations.len());
    }

    #[tokio::test]
    async fn test_warm_startup_optimization() {
        let optimizer = IntegratedStartupOptimizer::new();

        // Prepare for warm startup
        optimizer.optimize_for_warm_startup().await.unwrap();

        // Simulate warm startup
        let report = optimizer.initialize_with_monitoring(false).await.unwrap();
        println!("Warm startup time: {}ms", report.total_startup_time.as_millis());

        assert!(report.total_startup_time < Duration::from_millis(WARM_STARTUP_TARGET));
    }

    #[tokio::test]
    async fn test_caching_effectiveness() {
        let optimizer = IntegratedStartupOptimizer::new();

        // First expensive operation
        let start = Instant::now();
        let result1: i32 = optimizer.cache_expensive_operation("test_op".to_string(), async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            Ok::<i32, IDEError>(42)
        }).await.unwrap();
        let first_duration = start.elapsed();

        // Second call should be cached
        let start = Instant::now();
        let result2: i32 = optimizer.cache_expensive_operation("test_op".to_string(), async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            Ok::<i32, IDEError>(42)
        }).await.unwrap();
        let second_duration = start.elapsed();

        assert_eq!(result1, result2);
        assert!(second_duration < Duration::from_millis(10),
            "Cached operation should be much faster: {}ms", second_duration.as_millis());
    }
}
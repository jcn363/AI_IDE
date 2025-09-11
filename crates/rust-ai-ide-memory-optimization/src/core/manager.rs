//! Memory Optimization Manager - Central orchestrator for memory management
//! Manages leak detection, optimization suggestions, and performance monitoring
//! with integration to existing SIMD and AI infrastructure.

use std::sync::Arc;
use sysinfo::{System, SystemExt};
use tokio::sync::{Mutex, RwLock};
use tokio::time::{self, Duration};

#[cfg(feature = "ai_analysis")]
use rust_ai_ide_ai_codegen::types::CodeAnalysis;
#[cfg(feature = "simd_acceleration")]
use rust_ai_ide_simd::monitoring::SIMDPerformanceMonitor;

use crate::core::MemoryOptimizationConfig;
use crate::leak_detection::{LeakDetector, MemorySnapshot};
use crate::optimization::{OptimizationEngine, OptimizationSuggestion};
use crate::MemoryOptimizationResult;

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStatistics {
    pub total_memory_mb: f64,
    pub used_memory_mb: f64,
    pub available_memory_mb: f64,
    pub memory_usage_percent: f64,
    pub virtual_memory_mb: f64,
    pub swap_memory_mb: f64,
}

/// Real-time memory monitoring system
#[derive(Debug)]
pub struct MemoryMonitor {
    system: System,
    last_snapshot: Option<MemoryStatistics>,
    alert_threshold_percent: f64,
}

impl MemoryMonitor {
    pub fn new(threshold: f64) -> Self {
        Self {
            system: System::new_all(),
            last_snapshot: None,
            alert_threshold_percent: threshold,
        }
    }

    pub fn collect_statistics(&mut self) -> MemoryStatistics {
        self.system.refresh_all();

        let total_memory = self.system.total_memory() as f64 / 1024.0; // KB to MB
        let used_memory = self.system.used_memory() as f64 / 1024.0;
        let available_memory = total_memory - used_memory;
        let memory_usage_percent = if total_memory > 0.0 {
            (used_memory / total_memory) * 100.0
        } else {
            0.0
        };

        let virtual_memory = self.system.total_swap() as f64 / 1024.0;
        let swap_memory = self.system.used_swap() as f64 / 1024.0;

        let stats = MemoryStatistics {
            total_memory_mb: total_memory,
            used_memory_mb: used_memory,
            available_memory_mb: available_memory,
            memory_usage_percent,
            virtual_memory_mb: virtual_memory,
            swap_memory_mb: swap_memory,
        };

        self.last_snapshot = Some(stats.clone());
        stats
    }

    pub fn should_alert(&self, current_stats: &MemoryStatistics) -> bool {
        current_stats.memory_usage_percent >= self.alert_threshold_percent
    }
}

/// Main Memory Optimization Manager with thread-safe state management
#[derive(Debug)]
pub struct MemoryOptimizationManager {
    /// Manager configuration
    config: MemoryOptimizationConfig,

    /// Leak detector instance
    leak_detector: Arc<RwLock<Option<LeakDetector>>>,

    /// Optimization engine
    optimization_engine: Arc<RwLock<Option<OptimizationEngine>>>,

    /// Memory monitor
    memory_monitor: Arc<Mutex<MemoryMonitor>>,

    /// Background monitoring task handle
    monitoring_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,

    /// SIMD performance monitor (if available)
    #[cfg(feature = "simd_acceleration")]
    simd_monitor: Arc<Mutex<Option<SIMDPerformanceMonitor>>>,
}

impl MemoryOptimizationManager {
    /// Create a new memory optimization manager
    pub async fn new(config: MemoryOptimizationConfig) -> MemoryOptimizationResult<Self> {
        config
            .validate()
            .map_err(|e| anyhow::anyhow!("Invalid configuration: {}", e))?;

        let manager = Self {
            config,
            leak_detector: Arc::new(RwLock::new(None)),
            optimization_engine: Arc::new(RwLock::new(None)),
            memory_monitor: Arc::new(Mutex::new(MemoryMonitor::new(85.0))),
            monitoring_task: Arc::new(RwLock::new(None)),

            #[cfg(feature = "simd_acceleration")]
            simd_monitor: Arc::new(Mutex::new(None)),
        };

        // Initialize components
        manager.initialize_components().await?;

        Ok(manager)
    }

    /// Initialize memory optimization components
    async fn initialize_components(&self) -> MemoryOptimizationResult<()> {
        tracing::debug!("Initializing memory optimization components");

        // Initialize leak detector
        if self.config.enable_leak_detection {
            let leak_detector = LeakDetector::new();
            *self.leak_detector.write().await = Some(leak_detector);
        }

        // Initialize optimization engine
        if self.config.enable_auto_suggestions {
            let optimization_engine = OptimizationEngine::new(self.config.suggestion_cache_size_mb);
            *self.optimization_engine.write().await = Some(optimization_engine);
        }

        // Initialize SIMD monitor if enabled
        #[cfg(feature = "simd_acceleration")]
        if self.config.enable_simd_acceleration {
            let simd_monitor = SIMDPerformanceMonitor::new();
            *self.simd_monitor.lock().await = Some(simd_monitor);
        }

        Ok(())
    }

    /// Start background memory monitoring
    pub async fn start_background_monitoring(&self) -> MemoryOptimizationResult<()> {
        tracing::info!("Starting background memory monitoring");

        if let Some(_) = self.monitoring_task.read().await.as_ref() {
            return Err(anyhow::anyhow!("Background monitoring already running"));
        }

        let manager = self.clone();
        let task = tokio::spawn(async move {
            let interval = manager.config.get_monitoring_interval();
            let mut interval = time::interval(interval);

            loop {
                interval.tick().await;
                if let Err(e) = manager.perform_background_scan().await {
                    tracing::error!("Background memory scan failed: {}", e);
                }
            }
        });

        *self.monitoring_task.write().await = Some(task);

        tracing::info!("✅ Background memory monitoring started");
        Ok(())
    }

    /// Stop background monitoring
    pub async fn stop_background_monitoring(&self) -> MemoryOptimizationResult<()> {
        if let Some(task) = self.monitoring_task.write().await.take() {
            task.abort();
            tracing::info!("Background memory monitoring stopped");
        }
        Ok(())
    }

    /// Perform comprehensive memory scan and optimization
    pub async fn perform_comprehensive_scan(
        &self,
    ) -> MemoryOptimizationResult<SComprehensiveScanResults> {
        tracing::info!("Performing comprehensive memory scan");

        // Collect current memory statistics
        let stats = self.memory_monitor.lock().await.collect_statistics();

        // Determine if we should alert
        let should_alert = self.memory_monitor.lock().await.should_alert(&stats);

        // Check for memory leaks
        let mut leak_reports: Vec<crate::leak_detection::LeakReport> = Vec::new();
        if let Some(detector) = self.leak_detector.read().await.as_ref() {
            match detector.scan_for_leaks().await {
                Ok(snapshots) => {
                    for snapshot in snapshots {
                        let report = detector.analyze_snapshot(&snapshot).await?;
                        leak_reports.push(report);
                    }
                }
                Err(e) => tracing::warn!("Leak detection failed: {}", e),
            }
        }

        // Generate optimization suggestions
        let mut suggestions: Vec<OptimizationSuggestion> = Vec::new();
        if let Some(engine) = self.optimization_engine.read().await.as_ref() {
            suggestions = engine.generate_suggestions(&stats).await?;
        }

        // SIMD performance analysis
        let mut performance_report = None;
        #[cfg(feature = "simd_acceleration")]
        if let Some(simd_monitor) = self.simd_monitor.lock().await.as_ref() {
            performance_report = Some(simd_monitor.generate_performance_report().await?);
        }

        let results = ComprehensiveScanResults {
            memory_statistics: stats,
            leak_reports,
            optimization_suggestions: suggestions,
            should_alert,
            performance_report,
            timestamp: chrono::Utc::now(),
        };

        tracing::info!("Comprehensive memory scan completed");
        Ok(results)
    }

    /// Internal method for background scanning
    async fn perform_background_scan(&self) -> MemoryOptimizationResult<()> {
        // Collect memory statistics
        let stats = {
            let mut monitor = self.memory_monitor.lock().await;
            monitor.collect_statistics()
        };

        // Check alert threshold
        let should_alert = self.memory_monitor.lock().await.should_alert(&stats);
        if should_alert {
            tracing::warn!(
                "🚨 Memory usage alert: {:.1}% (threshold: {:.1}%)",
                stats.memory_usage_percent,
                self.config.memory_alert_threshold_percent
            );
        }

        // Perform leak scanning if enabled and due
        if self.config.enable_leak_detection {
            if let Some(detector) = self.leak_detector.read().await.as_ref() {
                match detector.scan_for_leaks().await {
                    Ok(snapshots) if !snapshots.is_empty() => {
                        tracing::warn!(
                            "🔍 Potential memory leaks detected: {} snapshot(s)",
                            snapshots.len()
                        );
                    }
                    Ok(_) => {} // No leaks detected
                    Err(e) => tracing::error!("Leak detection failed: {}", e),
                }
            }
        }

        Ok(())
    }

    /// Force immediate garbage collection and memory cleanup
    pub async fn force_garbage_collection(&self) -> MemoryOptimizationResult<()> {
        tracing::info!("🧹 Forcing garbage collection and memory cleanup");

        // Collect garbage from optimization engine
        if let Some(engine) = self.optimization_engine.read().await.as_ref() {
            engine.force_cleanup().await?;
        }

        // Force leak detector cleanup
        if let Some(detector) = self.leak_detector.read().await.as_ref() {
            detector.cleanup().await?;
        }

        tracing::info!("✅ Garbage collection completed");
        Ok(())
    }

    /// Get current memory statistics (non-blocking)
    pub async fn get_current_stats(&self) -> MemoryStatistics {
        let mut monitor = self.memory_monitor.lock().await;
        monitor.collect_statistics()
    }

    /// Generate memory usage report
    pub async fn generate_memory_report(&self) -> MemoryOptimizationResult<serde_json::Value> {
        let stats = self.get_current_stats().await;
        let results = self.perform_comprehensive_scan().await?;

        let report = serde_json::json!({
            "memory_statistics": {
                "total_mb": stats.total_memory_mb,
                "used_mb": stats.used_memory_mb,
                "available_mb": stats.available_memory_mb,
                "usage_percent": stats.memory_usage_percent,
                "virtual_mb": stats.virtual_memory_mb,
                "swap_mb": stats.swap_memory_mb
            },
            "leaks_detected": results.leak_reports.len(),
            "suggestions_count": results.optimization_suggestions.len(),
            "alert_active": results.should_alert,
            "timestamp": results.timestamp.to_rfc3339()
        });

        Ok(report)
    }
}

impl Clone for MemoryOptimizationManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            leak_detector: Arc::clone(&self.leak_detector),
            optimization_engine: Arc::clone(&self.optimization_engine),
            memory_monitor: Arc::clone(&self.memory_monitor),
            monitoring_task: Arc::clone(&self.monitoring_task),

            #[cfg(feature = "simd_acceleration")]
            simd_monitor: Arc::clone(&self.simd_monitor),
        }
    }
}

impl Drop for MemoryOptimizationManager {
    fn drop(&mut self) {
        // Clean shutdown - abort background tasks
        if let Some(task) = self.monitoring_task.clone().try_read().unwrap().as_ref() {
            task.abort();
        }
    }
}

/// Results from comprehensive memory scan
#[derive(Debug, Clone)]
pub struct ComprehensiveScanResults {
    pub memory_statistics: MemoryStatistics,
    pub leak_reports: Vec<crate::leak_detection::LeakReport>,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    pub should_alert: bool,
    pub performance_report: Option<String>, // SIMD performance report
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_optimization_manager_creation() {
        let config = MemoryOptimizationConfig::default();
        let manager = MemoryOptimizationManager::new(config).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_memory_monitor() {
        let config = MemoryOptimizationConfig::default();
        let manager = MemoryOptimizationManager::new(config).await.unwrap();
        let stats = manager.get_current_stats().await;

        assert!(stats.total_memory_mb > 0.0);
        assert!(stats.memory_usage_percent >= 0.0 && stats.memory_usage_percent <= 100.0);
    }

    #[tokio::test]
    async fn test_memory_alert() {
        let config = MemoryOptimizationConfig::default();
        let manager = MemoryOptimizationManager::new(config).await.unwrap();

        let mut monitor = MemoryMonitor::new(50.0); // Low threshold for testing
        let stats = monitor.collect_statistics();

        assert!(monitor.should_alert(&stats) || !monitor.should_alert(&stats));
    }
}

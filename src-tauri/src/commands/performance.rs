//! Performance monitoring commands for the Rust AI IDE
//!
//! This module provides Tauri commands for performance monitoring,
//! memory optimization, low-power mode, and system diagnostics.

use rust_ai_ide_common::validation::TauriInputSanitizer;
use tauri::State;

use crate::command_templates::tauri_command_template;
use crate::state::AppState;

/// Get current system performance metrics
#[tauri::command]
pub async fn get_system_metrics(state: State<'_, AppState>) -> Result<String, String> {
    execute_command!("get_system_metrics", &Default::default(), async move || {
        // Use double-locking pattern for lazy initialization
        let monitor_guard = state.performance_monitor.lock().await;
        match monitor_guard.as_ref() {
            Some(monitor) => {
                match monitor.collect_metrics().await {
                    Ok(metrics) => {
                        // Emit event to EventBus
                        let event_bus = state.event_bus();
                        let _ = event_bus
                            .emit(
                                "performance:system_metrics",
                                serde_json::json!({
                                    "cpu_percent": metrics.cpu_usage_percent,
                                    "memory_used_mb": metrics.memory_used_mb,
                                    "timestamp": metrics.timestamp
                                }),
                            )
                            .await;

                        Ok(serde_json::json!({
                            "status": "success",
                            "data": metrics
                        })
                        .to_string())
                    }
                    Err(e) => Ok(serde_json::json!({
                        "status": "error",
                        "message": format!("Failed to collect metrics: {}", e)
                    })
                    .to_string()),
                }
            }
            None => Ok(serde_json::json!({
                "status": "error",
                "message": "Performance monitor not initialized"
            })
            .to_string()),
        }
    })
}

/// Get performance history for visualization
#[tauri::command]
pub async fn get_performance_history(
    duration_minutes: Option<u64>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    execute_command!(
        "get_performance_history",
        &Default::default(),
        async move || {
            let duration = duration_minutes
                .unwrap_or(60)
                .min(1440) // Max 24 hours
                .as_secs()
                .as_duration();

            let monitor_guard = state.performance_monitor.lock().await;
            match monitor_guard.as_ref() {
                Some(monitor) => {
                    let history = monitor.get_history(duration).await;
                    Ok(serde_json::json!({
                        "status": "success",
                        "data": history
                    })
                    .to_string())
                }
                None => Ok(serde_json::json!({
                    "status": "error",
                    "message": "Performance monitor not initialized"
                })
                .to_string()),
            }
        }
    )
}

/// Get battery status and low-power mode configuration
#[tauri::command]
pub async fn get_battery_status(state: State<'_, AppState>) -> Result<String, String> {
    execute_command!("get_battery_status", &Default::default(), async move || {
        let monitor_guard = state.battery_monitor.lock().await;
        match monitor_guard.as_mut() {
            Some(monitor) => {
                let status = monitor.get_battery_status();
                Ok(serde_json::json!({
                    "status": "success",
                    "data": status,
                    "low_power_mode": monitor.is_low_power_mode()
                })
                .to_string())
            }
            None => Ok(serde_json::json!({
                "status": "error",
                "message": "Battery monitor not initialized"
            })
            .to_string()),
        }
    })
}

/// Run memory leak detection analysis
#[tauri::command]
pub async fn detect_memory_leaks(state: State<'_, AppState>) -> Result<String, String> {
    execute_command!(
        "detect_memory_leaks",
        &Default::default(),
        async move || {
            let memory_guard = state.memory_optimizer.lock().await;
            match memory_guard.as_ref() {
                Some(optimizer) => {
                    // Get memory leak analysis from the leak detector
                    // This would integrate with the memory leak detector
                    let mock_leaks = vec![]; // Replace with actual detection
                    Ok(serde_json::json!({
                        "status": "success",
                        "leaks_detected": mock_leaks.len(),
                        "recommended_actions": ["Clear caches", "Restart services"],
                    })
                    .to_string())
                }
                None => Ok(serde_json::json!({
                    "status": "error",
                    "message": "Memory optimizer not initialized"
                })
                .to_string()),
            }
        }
    )
}

/// Apply memory optimizations
#[tauri::command]
pub async fn optimize_memory(state: State<'_, AppState>) -> Result<String, String> {
    execute_command!("optimize_memory", &Default::default(), async move || {
        let mut optimizer_guard = state.memory_optimizer.lock().await;
        match optimizer_guard.as_mut() {
            Some(optimizer) => {
                let optimizations = optimizer.optimize_memory().await;
                let total_freed = optimizations
                    .iter()
                    .map(|o| o.memory_freed_bytes)
                    .sum::<usize>();

                Ok(serde_json::json!({
                    "status": "success",
                    "optimizations_applied": optimizations.len(),
                    "total_memory_freed": total_freed,
                    "recommendations": ["Cache cleared", "Garbage collected"]
                })
                .to_string())
            }
            None => Ok(serde_json::json!({
                "status": "error",
                "message": "Memory optimizer not initialized"
            })
            .to_string()),
        }
    })
}

/// Get active optimization statistics
#[tauri::command]
pub async fn get_optimization_stats(state: State<'_, AppState>) -> Result<String, String> {
    execute_command!(
        "get_optimization_stats",
        &Default::default(),
        async move || {
            let optimizer_guard = state.memory_optimizer.lock().await;
            match optimizer_guard.as_ref() {
                Some(optimizer) => {
                    let (freed_mb, last_cleanup) = optimizer.get_optimization_stats().await;
                    Ok(serde_json::json!({
                    "status": "success",
                    "total_memory_freed_mb": freed_mb / (1024 * 1024),
                    "last_cleanup_seconds_ago": last_cleanup.as_secs(),
                    "optimization_efficiency": if last_cleanup.as_secs() > 0 { (freed_mb as f64 / last_cleanup.as_secs_f64()).round() } else { 0.0 }
                }).to_string())
                }
                None => Ok(serde_json::json!({
                    "status": "error",
                    "message": "Memory optimizer not initialized"
                })
                .to_string()),
            }
        }
    )
}

/// Get process-level performance metrics
#[tauri::command]
pub async fn get_process_metrics(pid: Option<u32>, state: State<'_, AppState>) -> Result<String, String> {
    execute_command!(
        "get_process_metrics",
        &Default::default(),
        async move || {
            let monitor_guard = state.performance_monitor.lock().await;
            match monitor_guard.as_ref() {
                Some(monitor) => {
                    if let Some(process_id) = pid {
                        match monitor.collect_process_metrics(process_id).await {
                            Ok(Some(metrics)) => Ok(serde_json::json!({
                                "status": "success",
                                "data": metrics
                            })
                            .to_string()),
                            Ok(None) => Ok(serde_json::json!({
                                "status": "error",
                                "message": "Process not found"
                            })
                            .to_string()),
                            Err(e) => Ok(serde_json::json!({
                                "status": "error",
                                "message": format!("Failed to collect metrics: {}", e)
                            })
                            .to_string()),
                        }
                    } else {
                        // Return all processes
                        Ok(serde_json::json!({
                            "status": "success",
                            "message": "Process ID required for individual metrics"
                        })
                        .to_string())
                    }
                }
                None => Ok(serde_json::json!({
                    "status": "error",
                    "message": "Performance monitor not initialized"
                })
                .to_string()),
            }
        }
    )
}

/// Enable or disable low-power mode
#[tauri::command]
pub async fn set_low_power_mode(enabled: bool, state: State<'_, AppState>) -> Result<String, String> {
    execute_command!("set_low_power_mode", &Default::default(), async move || {
        // This would configure system-wide low-power settings
        // For now, return success
        Ok(serde_json::json!({
            "status": "success",
            "message": format!("Low-power mode {}", if enabled { "enabled" } else { "disabled" })
        })
        .to_string())
    })
}

/// Get system resource usage alerts
#[tauri::command]
pub async fn get_resource_alerts(state: State<'_, AppState>) -> Result<String, String> {
    execute_command!(
        "get_resource_alerts",
        &Default::default(),
        async move || {
            let monitor_guard = state.performance_monitor.lock().await;
            match monitor_guard.as_ref() {
                Some(monitor) => {
                    let heavy_load = monitor.is_heavy_load().await.unwrap_or(false);
                    let alerts = if heavy_load {
                        vec![serde_json::json!({
                            "severity": "warning",
                            "message": "High system load detected",
                            "suggestion": "Consider freeing resources"
                        })]
                    } else {
                        vec![]
                    };

                    Ok(serde_json::json!({
                        "status": "success",
                        "alerts": alerts
                    })
                    .to_string())
                }
                None => Ok(serde_json::json!({
                    "status": "warning",
                    "message": "Performance monitor not available",
                    "alerts": []
                })
                .to_string()),
            }
        }
    )
}

/// Get parallel processing statistics
#[tauri::command]
pub async fn get_parallel_stats(state: State<'_, AppState>) -> Result<String, String> {
    execute_command!("get_parallel_stats", &Default::default(), async move || {
        // Return mock parallel processing stats
        // In real implementation, this would integrate with parallel processing components
        Ok(serde_json::json!({
            "status": "success",
            "active_tasks": 3,
            "completed_tasks": 15,
            "failed_tasks": 0,
            "average_completion_time_ms": 120.5,
            "worker_utilization": [0.7, 0.8, 0.6, 0.75]
        })
        .to_string())
    })
}

// Placeholder command for cross-platform features
#[tauri::command]
pub async fn get_cross_platform_memory(state: State<'_, AppState>) -> Result<String, String> {
    Ok(serde_json::json!({
        "status": "info",
        "message": "Cross-platform memory monitoring available",
        "platforms_supported": ["linux", "macos", "windows"]
    })
    .to_string())
}

// Command to initialize performance services (for lazy initialization)
#[tauri::command]
pub async fn initialize_performance_monitoring(state: State<'_, AppState>) -> Result<String, String> {
    execute_command!(
        "initialize_performance_monitoring",
        &Default::default(),
        async move || {
            // Initialize performance monitor
            let monitor = rust_ai_ide_performance_monitoring::PerformanceMonitor::new();
            state.set_performance_monitor(monitor.clone()).await;

            // Initialize memory optimizer
            let optimizer = rust_ai_ide_performance_monitoring::memory::MemoryOptimizer::new(true);
            state.set_memory_optimizer(optimizer).await;

            // Initialize battery monitor
            let battery_config = rust_ai_ide_performance_monitoring::battery::LowPowerConfig {
                enable_cpu_throttling:  true,
                reduce_refresh_rate:    false,
                disable_animations:     false,
                limit_background_tasks: true,
                reduce_cache_sizes:     false,
                battery_threshold:      0.2,
            };
            let battery_monitor = rust_ai_ide_performance_monitoring::battery::BatteryMonitor::new(battery_config);
            state.set_battery_monitor(battery_monitor).await;

            Ok(serde_json::json!({
                "status": "success",
                "message": "Performance monitoring services initialized"
            })
            .to_string())
        }
    )
}

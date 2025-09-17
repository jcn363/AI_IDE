//! Performance guard for defragmentation operations

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use crate::InfraResult;

/// Guards defragmentation operations to prevent performance degradation
#[derive(Debug)]
pub struct PerformanceGuard {
    /// Guard configuration
    config: GuardConfig,

    /// Current operation state
    state: Arc<RwLock<GuardState>>,

    /// Performance metrics
    metrics: Arc<RwLock<PerformanceMetrics>>,
}

#[derive(Debug, Clone)]
pub struct GuardConfig {
    /// Maximum CPU usage allowed during defragmentation (0.0-1.0)
    pub max_cpu_usage: f64,

    /// Maximum memory pressure allowed (0.0-1.0)
    pub max_memory_pressure: f64,

    /// Maximum duration for a single defragmentation cycle
    pub max_cycle_duration: Duration,

    /// Minimum time between operations
    pub cooldown_period: Duration,

    /// Enable adaptive throttling
    pub adaptive_throttling: bool,

    /// CPU usage threshold for throttling (0.0-1.0)
    pub cpu_throttle_threshold: f64,

    /// Memory pressure threshold for throttling (0.0-1.0)
    pub memory_throttle_threshold: f64,

    /// Maximum consecutive failures before disabling
    pub max_consecutive_failures: usize,
}

#[derive(Debug)]
struct GuardState {
    /// Whether defragmentation is currently allowed
    allowed: bool,

    /// Last operation timestamp
    last_operation: Option<Instant>,

    /// Current operation start time
    current_operation_start: Option<Instant>,

    /// Consecutive failure count
    consecutive_failures: usize,

    /// Current throttling factor (1.0 = normal, < 1.0 = throttled)
    throttling_factor: f64,

    /// Performance monitoring enabled
    monitoring_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Total operations performed
    pub total_operations: usize,

    /// Successful operations
    pub successful_operations: usize,

    /// Failed operations
    pub failed_operations: usize,

    /// Average operation duration
    pub average_duration: Duration,

    /// Peak CPU usage during operations
    pub peak_cpu_usage: f64,

    /// Peak memory pressure during operations
    pub peak_memory_pressure: f64,

    /// Total time spent in operations
    pub total_operation_time: Duration,

    /// Last operation timestamp
    pub last_operation_time: Option<Instant>,
}

#[derive(Debug, Clone)]
pub enum GuardDecision {
    /// Operation is allowed to proceed
    Allow {
        /// Throttling factor to apply (1.0 = full speed)
        throttling_factor: f64,
    },

    /// Operation should be delayed
    Delay {
        /// Suggested delay duration
        delay: Duration,
    },

    /// Operation should be cancelled due to performance constraints
    Cancel {
        /// Reason for cancellation
        reason: String,
    },
}

impl Default for GuardConfig {
    fn default() -> Self {
        Self {
            max_cpu_usage: 0.8, // 80%
            max_memory_pressure: 0.9, // 90%
            max_cycle_duration: Duration::from_secs(30),
            cooldown_period: Duration::from_secs(10),
            adaptive_throttling: true,
            cpu_throttle_threshold: 0.7, // 70%
            memory_throttle_threshold: 0.8, // 80%
            max_consecutive_failures: 3,
        }
    }
}

impl Default for GuardState {
    fn default() -> Self {
        Self {
            allowed: true,
            last_operation: None,
            current_operation_start: None,
            consecutive_failures: 0,
            throttling_factor: 1.0,
            monitoring_enabled: true,
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            average_duration: Duration::from_secs(0),
            peak_cpu_usage: 0.0,
            peak_memory_pressure: 0.0,
            total_operation_time: Duration::from_secs(0),
            last_operation_time: None,
        }
    }
}

impl PerformanceGuard {
    /// Create a new performance guard
    pub fn new(config: GuardConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(GuardState::default())),
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
        }
    }

    /// Check if an operation should be allowed to proceed
    pub async fn check_operation(&self, current_cpu: f64, current_memory_pressure: f64) -> GuardDecision {
        let mut state = self.state.write().await;

        // Check cooldown period
        if let Some(last_op) = state.last_operation {
            let time_since_last = last_op.elapsed();
            if time_since_last < self.config.cooldown_period {
                let remaining = self.config.cooldown_period - time_since_last;
                return GuardDecision::Delay { delay: remaining };
            }
        }

        // Check CPU usage
        if current_cpu > self.config.max_cpu_usage {
            return GuardDecision::Cancel {
                reason: format!("CPU usage too high: {:.1}% > {:.1}%", current_cpu * 100.0, self.config.max_cpu_usage * 100.0)
            };
        }

        // Check memory pressure
        if current_memory_pressure > self.config.max_memory_pressure {
            return GuardDecision::Cancel {
                reason: format!("Memory pressure too high: {:.1}% > {:.1}%", current_memory_pressure * 100.0, self.config.max_memory_pressure * 100.0)
            };
        }

        // Check consecutive failures
        if state.consecutive_failures >= self.config.max_consecutive_failures {
            return GuardDecision::Cancel {
                reason: format!("Too many consecutive failures: {}", state.consecutive_failures)
            };
        }

        // Calculate throttling factor
        let throttling_factor = if self.config.adaptive_throttling {
            self.calculate_throttling_factor(current_cpu, current_memory_pressure).await
        } else {
            1.0
        };

        state.current_operation_start = Some(Instant::now());

        GuardDecision::Allow { throttling_factor }
    }

    /// Mark the start of an operation
    pub async fn start_operation(&self) {
        let mut state = self.state.write().await;
        state.current_operation_start = Some(Instant::now());
    }

    /// Mark the completion of an operation
    pub async fn complete_operation(&self, success: bool, cpu_usage: f64, memory_pressure: f64) {
        let mut state = self.state.write().await;
        let mut metrics = self.metrics.write().await;

        let operation_duration = if let Some(start) = state.current_operation_start {
            start.elapsed()
        } else {
            Duration::from_secs(0)
        };

        // Update state
        state.last_operation = Some(Instant::now());
        state.current_operation_start = None;

        if success {
            state.consecutive_failures = 0;
        } else {
            state.consecutive_failures += 1;
        }

        // Update metrics
        metrics.total_operations += 1;
        metrics.last_operation_time = Some(Instant::now());

        if success {
            metrics.successful_operations += 1;
        } else {
            metrics.failed_operations += 1;
        }

        metrics.total_operation_time += operation_duration;
        metrics.average_duration = metrics.total_operation_time / metrics.total_operations as u32;

        metrics.peak_cpu_usage = metrics.peak_cpu_usage.max(cpu_usage);
        metrics.peak_memory_pressure = metrics.peak_memory_pressure.max(memory_pressure);

        // Adaptive throttling adjustment
        if self.config.adaptive_throttling {
            self.adjust_throttling_factor(&mut state, success, operation_duration).await;
        }
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }

    /// Check if the guard is in a healthy state
    pub async fn is_healthy(&self) -> bool {
        let state = self.state.read().await;
        let metrics = self.metrics.read().await;

        let success_rate = if metrics.total_operations > 0 {
            metrics.successful_operations as f64 / metrics.total_operations as f64
        } else {
            1.0
        };

        state.allowed &&
        state.consecutive_failures < self.config.max_consecutive_failures &&
        success_rate > 0.5 // At least 50% success rate
    }

    /// Reset failure counters (for recovery)
    pub async fn reset_failures(&self) {
        let mut state = self.state.write().await;
        state.consecutive_failures = 0;
    }

    /// Enable or disable the guard
    pub async fn set_enabled(&self, enabled: bool) {
        let mut state = self.state.write().await;
        state.allowed = enabled;
    }

    /// Calculate adaptive throttling factor
    async fn calculate_throttling_factor(&self, cpu: f64, memory: f64) -> f64 {
        let state = self.state.read().await;

        let cpu_factor = if cpu > self.config.cpu_throttle_threshold {
            (self.config.max_cpu_usage - cpu) / (self.config.max_cpu_usage - self.config.cpu_throttle_threshold)
        } else {
            1.0
        };

        let memory_factor = if memory > self.config.memory_throttle_threshold {
            (self.config.max_memory_pressure - memory) / (self.config.max_memory_pressure - self.config.memory_throttle_threshold)
        } else {
            1.0
        };

        let combined_factor = (cpu_factor + memory_factor) / 2.0;
        let throttled_factor = combined_factor.max(0.1).min(1.0); // Min 10% speed

        (state.throttling_factor + throttled_factor) / 2.0 // Smooth transitions
    }

    /// Adjust throttling factor based on operation results
    async fn adjust_throttling_factor(&self, state: &mut GuardState, success: bool, duration: Duration) {
        let target_duration = self.config.max_cycle_duration;

        if success {
            if duration < target_duration / 2 {
                // Operation was fast, can increase speed slightly
                state.throttling_factor = (state.throttling_factor * 1.1).min(1.0);
            } else if duration > target_duration {
                // Operation was slow, reduce speed
                state.throttling_factor *= 0.9;
            }
        } else {
            // Failed operation, reduce speed significantly
            state.throttling_factor *= 0.7;
        }

        // Ensure throttling factor stays within reasonable bounds
        state.throttling_factor = state.throttling_factor.max(0.1).min(1.0);
    }

    /// Export guard status for monitoring
    pub async fn export_status(&self) -> serde_json::Value {
        let state = self.state.read().await;
        let metrics = self.metrics.read().await;

        let success_rate = if metrics.total_operations > 0 {
            metrics.successful_operations as f64 / metrics.total_operations as f64
        } else {
            0.0
        };

        serde_json::json!({
            "enabled": state.allowed,
            "healthy": self.is_healthy().await,
            "consecutive_failures": state.consecutive_failures,
            "throttling_factor": state.throttling_factor,
            "metrics": {
                "total_operations": metrics.total_operations,
                "success_rate": success_rate,
                "average_duration_seconds": metrics.average_duration.as_secs_f64(),
                "peak_cpu_usage": metrics.peak_cpu_usage,
                "peak_memory_pressure": metrics.peak_memory_pressure
            },
            "config": {
                "max_cpu_usage": self.config.max_cpu_usage,
                "max_memory_pressure": self.config.max_memory_pressure,
                "adaptive_throttling": self.config.adaptive_throttling
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_guard_basic_functionality() {
        let config = GuardConfig::default();
        let guard = PerformanceGuard::new(config);

        // Test initial state
        assert!(guard.is_healthy().await);

        // Test operation start/complete
        guard.start_operation().await;
        guard.complete_operation(true, 0.5, 0.5).await;

        let metrics = guard.get_metrics().await;
        assert_eq!(metrics.total_operations, 1);
        assert_eq!(metrics.successful_operations, 1);
    }

    #[tokio::test]
    async fn test_guard_cpu_limit() {
        let config = GuardConfig {
            max_cpu_usage: 0.7,
            ..Default::default()
        };
        let guard = PerformanceGuard::new(config);

        let decision = guard.check_operation(0.8, 0.5).await;
        match decision {
            GuardDecision::Cancel { reason } => {
                assert!(reason.contains("CPU usage too high"));
            }
            _ => panic!("Expected operation to be cancelled due to high CPU usage"),
        }
    }

    #[tokio::test]
    async fn test_guard_adaptive_throttling() {
        let config = GuardConfig {
            adaptive_throttling: true,
            cpu_throttle_threshold: 0.6,
            ..Default::default()
        };
        let guard = PerformanceGuard::new(config);

        // High CPU should trigger throttling
        let decision = guard.check_operation(0.7, 0.5).await;
        match decision {
            GuardDecision::Allow { throttling_factor } => {
                assert!(throttling_factor < 1.0, "Expected throttling factor < 1.0, got {}", throttling_factor);
            }
            _ => panic!("Expected operation to be allowed with throttling"),
        }
    }
}
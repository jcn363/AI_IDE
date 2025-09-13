use std::sync::Arc;
use std::time::{Duration, Instant};

use rust_ai_ide_common::{IDEError, IDEErrorKind};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};

use crate::{StartupProfiler, StartupReport};

/// Configuration for profiling adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingConfiguration {
    pub cold_startup_target:        Duration,
    pub warm_startup_target:        Duration,
    pub profiling_enabled:          bool,
    pub measurements_history_limit: usize,
}

impl Default for ProfilingConfiguration {
    fn default() -> Self {
        Self {
            cold_startup_target:        Duration::from_millis(400),
            warm_startup_target:        Duration::from_millis(80),
            profiling_enabled:          true,
            measurements_history_limit: 100,
        }
    }
}

/// Startup measurement storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupMeasurement {
    pub timestamp:       u64,
    pub is_cold_startup: bool,
    pub total_duration:  Duration,
    pub phase_durations: std::collections::HashMap<String, Duration>,
    pub metadata:        std::collections::HashMap<String, String>,
}

/// Profiling adapter for startup time measurement and monitoring
pub struct ProfilingAdapter {
    profiler:     Arc<StartupProfiler>,
    config:       Arc<RwLock<ProfilingConfiguration>>,
    measurements: Arc<Mutex<Vec<StartupMeasurement>>>,
}

impl ProfilingAdapter {
    pub fn new(profiler: Arc<StartupProfiler>) -> Self {
        Self {
            profiler,
            config: Arc::new(RwLock::new(ProfilingConfiguration::default())),
            measurements: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn start_startup_measurement(&self, is_cold_startup: bool) -> Result<(), IDEError> {
        let config = self.config.read().await;
        if !config.profiling_enabled {
            return Ok(());
        }

        // Clear previous measurements
        let mut measurements = self.measurements.lock().await;
        measurements.clear();

        // Start overall profiling
        self.profiler.start_phase("startup_total").await;

        // Add initial metadata
        let mut initial_measurement = StartupMeasurement {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            is_cold_startup,
            total_duration: Duration::default(),
            phase_durations: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
        };

        initial_measurement.metadata.insert(
            "startup_type".to_string(),
            if is_cold_startup { "cold" } else { "warm" }.to_string(),
        );
        measurements.push(initial_measurement);

        Ok(())
    }

    pub async fn end_startup_measurement(&self) -> Result<StartupReport, IDEError> {
        let config = self.config.read().await;

        // End overall profiling
        self.profiler.end_phase("startup_total").await;

        // Get the startup report
        let report = self.profiler.get_startup_report().await;

        // Store measurement in history
        let mut measurements = self.measurements.lock().await;
        if let Some(current_measurement) = measurements.last_mut() {
            current_measurement.total_duration = report.total_startup_time;
            current_measurement.phase_durations = report.phase_average_times.clone();

            // Add performance metadata
            current_measurement.metadata.insert(
                "performance_status".to_string(),
                if report.total_startup_time <= config.cold_startup_target {
                    "within_cold_target"
                } else if report.total_startup_time <= config.warm_startup_target {
                    "within_warm_target"
                } else {
                    "above_targets"
                }
                .to_string(),
            );
        }

        // Maintain history limit
        let mut measurements_vec = Vec::new();
        std::mem::swap(&mut *measurements, &mut measurements_vec);
        let keep_count = config
            .measurements_history_limit
            .min(measurements_vec.len());
        let mut new_measurements = measurements_vec
            .into_iter()
            .rev()
            .take(keep_count)
            .collect::<Vec<_>>();
        new_measurements.reverse();
        std::mem::swap(&mut *measurements, &mut new_measurements);

        Ok(report)
    }

    pub async fn measure_phase<F, Fut>(&self, phase_name: &str, future: F) -> Result<F::Output, F::Error>
    where
        F: Future<Output: Result<T, E>>,
        Fut: Future<Output = Result<T, E>>,
    {
        self.profiler.start_phase(phase_name).await;
        let result = future.await;
        self.profiler.end_phase(phase_name).await;
        result
    }

    pub async fn measure_blocking<F, T>(&self, phase_name: &str, blocking_fn: F) -> Result<T, IDEError>
    where
        F: FnOnce() -> Result<T, IDEError> + Send + 'static,
        T: Send + 'static,
    {
        self.profiler
            .measure_blocking(phase_name, blocking_fn)
            .await
    }

    pub async fn get_current_startup_stats(&self) -> Result<StartupStats, IDEError> {
        let config = self.config.read().await;
        let measurements = self.measurements.lock().await;

        if measurements.is_empty() {
            return Err(IDEError::new(
                IDEErrorKind::StateError,
                "No startup measurements available",
            ));
        }

        let recent_measurements = measurements.iter().rev().take(10).collect::<Vec<_>>();

        let cold_startup_times = recent_measurements
            .iter()
            .filter(|m| m.is_cold_startup)
            .map(|m| m.total_duration)
            .collect::<Vec<_>>();

        let warm_startup_times = recent_measurements
            .iter()
            .filter(|m| !m.is_cold_startup)
            .map(|m| m.total_duration)
            .collect::<Vec<_>>();

        let avg_cold_startup = if !cold_startup_times.is_empty() {
            cold_startup_times.iter().sum::<Duration>() / cold_startup_times.len() as u32
        } else {
            Duration::default()
        };

        let avg_warm_startup = if !warm_startup_times.is_empty() {
            warm_startup_times.iter().sum::<Duration>() / warm_startup_times.len() as u32
        } else {
            Duration::default()
        };

        Ok(StartupStats {
            target_cold_startup:       config.cold_startup_target,
            target_warm_startup:       config.warm_startup_target,
            average_cold_startup:      avg_cold_startup,
            average_warm_startup:      avg_warm_startup,
            recent_measurements_count: recent_measurements.len(),
            cold_startup_target_met:   avg_cold_startup <= config.cold_startup_target,
            warm_startup_target_met:   avg_warm_startup <= config.warm_startup_target,
        })
    }

    pub async fn update_configuration(&self, new_config: ProfilingConfiguration) -> Result<(), IDEError> {
        let mut config = self.config.write().await;
        *config = new_config;
        Ok(())
    }

    pub async fn get_configuration(&self) -> ProfilingConfiguration {
        self.config.read().await.clone()
    }

    pub async fn get_measurements_history(&self) -> Vec<StartupMeasurement> {
        self.measurements.lock().await.clone()
    }

    pub async fn clear_history(&self) {
        let mut measurements = self.measurements.lock().await;
        measurements.clear();
    }
}

/// Statistics for startup performance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupStats {
    pub target_cold_startup:       Duration,
    pub target_warm_startup:       Duration,
    pub average_cold_startup:      Duration,
    pub average_warm_startup:      Duration,
    pub recent_measurements_count: usize,
    pub cold_startup_target_met:   bool,
    pub warm_startup_target_met:   bool,
}

impl StartupStats {
    pub fn is_within_targets(&self) -> bool {
        self.cold_startup_target_met && self.warm_startup_target_met
    }

    pub fn get_performance_summary(&self) -> String {
        format!(
            "Startup Performance: Cold: {}ms (target: {}ms, met: {}), Warm: {}ms (target: {}ms, met: {}), \
             Measurements: {}",
            self.average_cold_startup.as_millis(),
            self.target_cold_startup.as_millis(),
            self.cold_startup_target_met,
            self.average_warm_startup.as_millis(),
            self.target_warm_startup.as_millis(),
            self.warm_startup_target_met,
            self.recent_measurements_count
        )
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_profiling_adapter_startup_measurement() {
        let profiler = Arc::new(StartupProfiler::new());
        let adapter = ProfilingAdapter::new(profiler);

        // Test configuration
        let config = ProfilingConfiguration {
            cold_startup_target:        Duration::from_millis(400),
            warm_startup_target:        Duration::from_millis(80),
            profiling_enabled:          true,
            measurements_history_limit: 10,
        };

        adapter.update_configuration(config).await.unwrap();

        // Test cold startup measurement
        adapter.start_startup_measurement(true).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        let report = adapter.end_startup_measurement().await.unwrap();

        assert!(report.total_startup_time >= Duration::from_millis(50));

        // Test stats
        let stats = adapter.get_current_startup_stats().await.unwrap();
        assert_eq!(stats.target_cold_startup, Duration::from_millis(400));
        assert_eq!(stats.target_warm_startup, Duration::from_millis(80));
    }

    #[tokio::test]
    async fn test_phase_measurement() {
        let profiler = Arc::new(StartupProfiler::new());
        let adapter = ProfilingAdapter::new(profiler);

        // Start overall measurement
        adapter.start_startup_measurement(false).await.unwrap();

        // Measure a phase
        let result = adapter
            .measure_phase("test_phase", async {
                tokio::time::sleep(Duration::from_millis(30)).await;
                Ok::<_, IDEError>(42)
            })
            .await
            .unwrap();

        adapter.end_startup_measurement().await.unwrap();

        assert_eq!(result, 42);

        // Check that phase was measured
        let stats = adapter.get_current_startup_stats().await.unwrap();
        assert!(stats.average_warm_startup >= Duration::from_millis(30));
    }
}

//! GUI integration components for memory optimization
//! Provides Tauri commands and React-friendly data structures
//! for real-time memory monitoring and analysis visualization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Memory visualization data for the React dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryVisualizationData {
    /// Current memory usage in MB
    pub current_usage_mb: f64,

    /// Memory usage trend over time (last 60 data points)
    pub usage_trend: Vec<f64>,

    /// Memory usage percentage (0-100)
    pub usage_percentage: f64,

    /// Memory leak indicators (count of suspected leaks)
    pub leak_indicators: u32,

    /// Active memory optimizations applied
    pub active_optimizations: Vec<String>,

    /// Optimization suggestions available
    pub suggestions_count: u32,

    /// Memory pool utilization percentages
    pub pool_utilization: HashMap<String, f64>,

    /// Timestamp of last update
    pub last_updated: String,

    /// System memory information
    pub system_memory: SystemMemoryInfo,
}

/// System memory information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMemoryInfo {
    pub total_memory_gb: f64,
    pub available_memory_gb: f64,
    pub used_memory_gb: f64,
    pub memory_pressure: String, // "low", "medium", "high", "critical"
}

/// Memory alert level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryAlertLevel {
    Normal,
    Warning,
    Critical,
}

/// Memory dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDashboardConfig {
    /// Enable real-time updates
    pub enable_real_time: bool,

    /// Update interval in seconds
    pub update_interval_seconds: u32,

    /// Memory threshold percentages for alerts
    pub memory_thresholds: MemoryThresholds,

    /// Auto-optimization settings
    pub auto_optimization: AutoOptimizationSettings,
}

/// Memory threshold configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryThresholds {
    pub warning_threshold: f64,  // percentage
    pub critical_threshold: f64, // percentage
    pub leak_alert_count: u32,   // number of suspected leaks
}

/// Auto-optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoOptimizationSettings {
    pub enable_auto_apply: bool,
    pub aggressive_mode: bool,
    pub optimization_period_minutes: u32,
}

/// Memory dashboard state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDashboardState {
    pub config: MemoryDashboardConfig,
    pub current_data: MemoryVisualizationData,
    pub alerts: Vec<MemoryAlert>,
    pub history: Vec<MemoryHistoryPoint>,
    pub optimizations_applied: Vec<OptimizationEvent>,
}

/// Memory alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAlert {
    pub id: String,
    pub level: MemoryAlertLevel,
    pub message: String,
    pub timestamp: String,
    pub resolved: bool,
}

/// Memory history data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHistoryPoint {
    pub timestamp: String,
    pub memory_usage_mb: f64,
    pub leak_count: u32,
}

/// Optimization application event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationEvent {
    pub id: String,
    pub optimization_type: String,
    pub applied_at: String,
    pub memory_improvement_mb: f64,
    pub success: bool,
}

/// Memory dashboard implementation
pub struct MemoryDashboard {
    config: MemoryDashboardConfig,
    state: tokio::sync::RwLock<MemoryDashboardState>,
}

impl Default for MemoryDashboardConfig {
    fn default() -> Self {
        Self {
            enable_real_time: true,
            update_interval_seconds: 30,
            memory_thresholds: MemoryThresholds {
                warning_threshold: 75.0,
                critical_threshold: 90.0,
                leak_alert_count: 5,
            },
            auto_optimization: AutoOptimizationSettings {
                enable_auto_apply: false,
                aggressive_mode: false,
                optimization_period_minutes: 60,
            },
        }
    }
}

impl Default for MemoryVisualizationData {
    fn default() -> Self {
        Self {
            current_usage_mb: 0.0,
            usage_trend: Vec::new(),
            usage_percentage: 0.0,
            leak_indicators: 0,
            active_optimizations: Vec::new(),
            suggestions_count: 0,
            pool_utilization: HashMap::new(),
            last_updated: chrono::Utc::now().to_rfc3339(),
            system_memory: SystemMemoryInfo {
                total_memory_gb: 0.0,
                available_memory_gb: 0.0,
                used_memory_gb: 0.0,
                memory_pressure: "unknown".to_string(),
            },
        }
    }
}

impl MemoryDashboard {
    /// Create a new memory dashboard
    pub fn new(config: Option<MemoryDashboardConfig>) -> Self {
        let config = config.unwrap_or_default();
        let state = MemoryDashboardState {
            config: config.clone(),
            current_data: MemoryVisualizationData::default(),
            alerts: Vec::new(),
            history: Vec::new(),
            optimizations_applied: Vec::new(),
        };

        Self {
            config,
            state: tokio::sync::RwLock::new(state),
        }
    }

    /// Update dashboard with new memory data
    pub async fn update_data(
        &self,
        new_data: MemoryVisualizationData,
    ) -> crate::MemoryOptimizationResult<()> {
        let mut state = self.state.write().await;
        state.current_data = new_data;
        state.history.push(MemoryHistoryPoint {
            timestamp: new_data.last_updated.clone(),
            memory_usage_mb: new_data.current_usage_mb,
            leak_count: new_data.leak_indicators,
        });

        // Keep only last 100 history points
        if state.history.len() > 100 {
            state.history.remove(0);
        }

        // Generate alerts based on current data
        self.check_alerts(&mut state).await;

        Ok(())
    }

    /// Get current dashboard state
    pub async fn get_state(&self) -> MemoryDashboardState {
        self.state.read().await.clone()
    }

    /// Add a new alert
    pub async fn add_alert(&self, level: MemoryAlertLevel, message: String) {
        let mut state = self.state.write().await;
        let alert = MemoryAlert {
            id: uuid::Uuid::new_v4().to_string(),
            level,
            message,
            timestamp: chrono::Utc::now().to_rfc3339(),
            resolved: false,
        };
        state.alerts.push(alert);
    }

    /// Resolve an alert
    pub async fn resolve_alert(&self, alert_id: &str) {
        let mut state = self.state.write().await;
        if let Some(alert) = state.alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.resolved = true;
        }
    }

    /// Get current visualization data (React-compatible)
    pub async fn get_visualization_data(&self) -> MemoryVisualizationData {
        self.state.read().await.current_data.clone()
    }

    /// Configure dashboard settings
    pub async fn update_config(&self, new_config: MemoryDashboardConfig) {
        let mut state = self.state.write().await;
        state.config = new_config.clone();
        self.config = new_config;
    }

    /// Check and generate memory alerts
    async fn check_alerts(&self, state: &mut MemoryDashboardState) {
        let data = &state.current_data;

        // Memory usage alerts
        if data.usage_percentage >= self.config.memory_thresholds.critical_threshold {
            let alert = MemoryAlert {
                id: uuid::Uuid::new_v4().to_string(),
                level: MemoryAlertLevel::Critical,
                message: format!("Critical memory usage: {:.1}%", data.usage_percentage),
                timestamp: chrono::Utc::now().to_rfc3339(),
                resolved: false,
            };
            state.alerts.push(alert);
        } else if data.usage_percentage >= self.config.memory_thresholds.warning_threshold {
            let alert = MemoryAlert {
                id: uuid::Uuid::new_v4().to_string(),
                level: MemoryAlertLevel::Warning,
                message: format!("High memory usage: {:.1}%", data.usage_percentage),
                timestamp: chrono::Utc::now().to_rfc3339(),
                resolved: false,
            };
            state.alerts.push(alert);
        }

        // Leak alerts
        if data.leak_indicators >= self.config.memory_thresholds.leak_alert_count {
            let alert = MemoryAlert {
                id: uuid::Uuid::new_v4().to_string(),
                level: MemoryAlertLevel::Warning,
                message: format!("Potential memory leaks detected: {}", data.leak_indicators),
                timestamp: chrono::Utc::now().to_rfc3339(),
                resolved: false,
            };
            state.alerts.push(alert);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_dashboard_creation() {
        let dashboard = MemoryDashboard::new(None);
        let config = MemoryDashboardConfig::default();
        assert_eq!(dashboard.config.enable_real_time, config.enable_real_time);
    }

    #[tokio::test]
    async fn test_memory_dashboard_update() {
        let dashboard = MemoryDashboard::new(None);
        let test_data = MemoryVisualizationData {
            current_usage_mb: 1024.0,
            usage_percentage: 75.0,
            ..Default::default()
        };

        let result = dashboard.update_data(test_data).await;
        assert!(result.is_ok());

        let state = dashboard.get_state().await;
        assert_eq!(state.current_data.current_usage_mb, 1024.0);
    }
}

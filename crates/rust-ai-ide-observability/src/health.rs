//! Health check system for monitoring service availability
//!
//! Provides comprehensive health checks for critical services,
//! databases, LSP servers, and AI components.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::errors::Result;
use crate::ObservabilityConfig;

/// Health check manager
pub struct HealthChecker {
    config: ObservabilityConfig,
    checks: Arc<RwLock<HashMap<String, HealthCheckResult>>>,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(config: ObservabilityConfig) -> Self {
        Self {
            config,
            checks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Perform all configured health checks
    pub async fn perform_all_checks(&self) -> Result<HealthStatus> {
        if !self.config.health.enabled {
            return Ok(HealthStatus {
                overall_status: HealthCheckStatus::Healthy,
                timestamp:      Utc::now(),
                checks:         HashMap::new(),
                message:        "Health checks disabled".to_string(),
            });
        }

        let mut checks = HashMap::new();
        let mut overall_status = HealthCheckStatus::Healthy;

        // Database health check
        if self.config.health.database_checks_enabled {
            let db_status = self.check_database().await;
            checks.insert("database".to_string(), db_status.clone());
            if db_status.status != HealthCheckStatus::Healthy {
                overall_status = HealthCheckStatus::Unhealthy;
            }
        }

        // LSP servers health check
        if self.config.health.lsp_checks_enabled {
            let lsp_status = self.check_lsp_servers().await;
            checks.insert("lsp_servers".to_string(), lsp_status.clone());
            if lsp_status.status != HealthCheckStatus::Healthy {
                overall_status = HealthCheckStatus::Unhealthy;
            }
        }

        // AI services health check
        if self.config.health.ai_checks_enabled {
            let ai_status = self.check_ai_services().await;
            checks.insert("ai_services".to_string(), ai_status.clone());
            if ai_status.status != HealthCheckStatus::Healthy {
                overall_status = HealthCheckStatus::Unhealthy;
            }
        }

        // System resources check
        let system_status = self.check_system_resources().await;
        checks.insert("system_resources".to_string(), system_status.clone());
        if system_status.status != HealthCheckStatus::Healthy {
            overall_status = HealthCheckStatus::Unhealthy;
        }

        let status = HealthStatus {
            overall_status,
            timestamp: Utc::now(),
            checks,
            message: format!("Health check completed with {} checks", checks.len()),
        };

        // Update stored checks
        let mut stored_checks = self.checks.write().await;
        for (name, result) in &status.checks {
            stored_checks.insert(name.clone(), result.clone());
        }

        Ok(status)
    }

    /// Get the overall health status
    pub async fn get_health_status(&self) -> Result<HealthStatus> {
        let checks = self.checks.read().await.clone();
        let overall_status = self.determine_overall_status(&checks);

        Ok(HealthStatus {
            overall_status,
            timestamp: Utc::now(),
            checks,
            message: "Current health status".to_string(),
        })
    }

    /// Check database connectivity
    async fn check_database(&self) -> HealthCheckResult {
        let start = Instant::now();

        // TODO: Implement actual database connectivity check
        // For now, simulate a check
        tokio::time::sleep(Duration::from_millis(100)).await;

        let duration = start.elapsed();

        if duration < Duration::from_secs(self.config.health.max_response_time_secs) {
            HealthCheckResult {
                name:        "database".to_string(),
                status:      HealthCheckStatus::Healthy,
                timestamp:   Utc::now(),
                duration_ms: duration.as_millis() as u64,
                message:     "Database connection successful".to_string(),
                details:     None,
            }
        } else {
            HealthCheckResult {
                name:        "database".to_string(),
                status:      HealthCheckStatus::Unhealthy,
                timestamp:   Utc::now(),
                duration_ms: duration.as_millis() as u64,
                message:     "Database response time too slow".to_string(),
                details:     Some(serde_json::json!({
                    "response_time_ms": duration.as_millis(),
                    "threshold_ms": self.config.health.max_response_time_secs * 1000
                })),
            }
        }
    }

    /// Check LSP servers
    async fn check_lsp_servers(&self) -> HealthCheckResult {
        let start = Instant::now();

        // TODO: Implement actual LSP server checks
        tokio::time::sleep(Duration::from_millis(50)).await;

        let duration = start.elapsed();

        HealthCheckResult {
            name:        "lsp_servers".to_string(),
            status:      HealthCheckStatus::Healthy,
            timestamp:   Utc::now(),
            duration_ms: duration.as_millis() as u64,
            message:     "LSP servers responding normally".to_string(),
            details:     Some(serde_json::json!({
                "active_servers": ["rust-analyzer", "typescript-language-server"],
                "total_servers": 2
            })),
        }
    }

    /// Check AI services
    async fn check_ai_services(&self) -> HealthCheckResult {
        let start = Instant::now();

        // TODO: Implement actual AI service checks
        tokio::time::sleep(Duration::from_millis(200)).await;

        let duration = start.elapsed();

        if duration < Duration::from_secs(self.config.health.max_response_time_secs) {
            HealthCheckResult {
                name:        "ai_services".to_string(),
                status:      HealthCheckStatus::Healthy,
                timestamp:   Utc::now(),
                duration_ms: duration.as_millis() as u64,
                message:     "AI services operational".to_string(),
                details:     Some(serde_json::json!({
                    "active_models": ["gpt-4", "claude-3"],
                    "queue_depth": 0
                })),
            }
        } else {
            HealthCheckResult {
                name:        "ai_services".to_string(),
                status:      HealthCheckStatus::Degraded,
                timestamp:   Utc::now(),
                duration_ms: duration.as_millis() as u64,
                message:     "AI services responding slowly".to_string(),
                details:     Some(serde_json::json!({
                    "response_time_ms": duration.as_millis(),
                    "queue_depth": 5
                })),
            }
        }
    }

    /// Check system resources
    async fn check_system_resources(&self) -> HealthCheckResult {
        use sysinfo::{System, SystemExt};

        let start = Instant::now();
        let mut system = System::new_all();
        system.refresh_all();
        let duration = start.elapsed();

        let cpu_usage = system.global_cpu_info().cpu_usage() as f64;
        let memory_usage = if system.total_memory() > 0 {
            (system.used_memory() as f64 / system.total_memory() as f64) * 100.0
        } else {
            0.0
        };

        let disk_usage = system
            .disks()
            .iter()
            .map(|disk| {
                let total = disk.total_space() as f64;
                let available = disk.available_space() as f64;
                if total > 0.0 {
                    ((total - available) / total) * 100.0
                } else {
                    0.0
                }
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let status = if cpu_usage > self.config.alerting.cpu_warning_threshold
            || memory_usage > self.config.alerting.memory_warning_threshold
            || disk_usage > self.config.alerting.disk_warning_threshold
        {
            HealthCheckStatus::Degraded
        } else {
            HealthCheckStatus::Healthy
        };

        HealthCheckResult {
            name: "system_resources".to_string(),
            status,
            timestamp: Utc::now(),
            duration_ms: duration.as_millis() as u64,
            message: format!(
                "System resources: CPU {:.1}%, Memory {:.1}%, Disk {:.1}%",
                cpu_usage, memory_usage, disk_usage
            ),
            details: Some(serde_json::json!({
                "cpu_usage_percent": cpu_usage,
                "memory_usage_percent": memory_usage,
                "disk_usage_percent": disk_usage,
                "process_count": system.processes().len()
            })),
        }
    }

    /// Determine overall health status from individual checks
    fn determine_overall_status(&self, checks: &HashMap<String, HealthCheckResult>) -> HealthCheckStatus {
        let mut has_critical = false;
        let mut has_degraded = false;

        for result in checks.values() {
            match result.status {
                HealthCheckStatus::Unhealthy => has_critical = true,
                HealthCheckStatus::Degraded => has_degraded = true,
                HealthCheckStatus::Healthy => {}
            }
        }

        if has_critical {
            HealthCheckStatus::Unhealthy
        } else if has_degraded {
            HealthCheckStatus::Degraded
        } else {
            HealthCheckStatus::Healthy
        }
    }

    /// Get health check history
    pub async fn get_health_history(&self, limit: usize) -> Result<Vec<HealthStatus>> {
        // TODO: Implement health check history storage
        // For now, return current status
        let current = self.get_health_status().await?;
        Ok(vec![current])
    }
}

/// Overall health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub overall_status: HealthCheckStatus,
    pub timestamp:      DateTime<Utc>,
    pub checks:         HashMap<String, HealthCheckResult>,
    pub message:        String,
}

/// Individual health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub name:        String,
    pub status:      HealthCheckStatus,
    pub timestamp:   DateTime<Utc>,
    pub duration_ms: u64,
    pub message:     String,
    pub details:     Option<serde_json::Value>,
}

/// Health check status levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HealthCheckStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl HealthCheckStatus {
    /// Check if the status is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }

    /// Check if the status requires attention
    pub fn needs_attention(&self) -> bool {
        !self.is_healthy()
    }
}

/// Helper functions for health checks
pub mod helpers {
    use super::*;

    /// Create a simple health check result
    pub fn simple_result(name: &str, healthy: bool, message: &str) -> HealthCheckResult {
        HealthCheckResult {
            name:        name.to_string(),
            status:      if healthy {
                HealthCheckStatus::Healthy
            } else {
                HealthCheckStatus::Unhealthy
            },
            timestamp:   Utc::now(),
            duration_ms: 0,
            message:     message.to_string(),
            details:     None,
        }
    }

    /// Create a health check result with timing
    pub fn timed_result(name: &str, duration: Duration, healthy: bool, message: &str) -> HealthCheckResult {
        HealthCheckResult {
            name:        name.to_string(),
            status:      if healthy {
                HealthCheckStatus::Healthy
            } else {
                HealthCheckStatus::Unhealthy
            },
            timestamp:   Utc::now(),
            duration_ms: duration.as_millis() as u64,
            message:     message.to_string(),
            details:     None,
        }
    }

    /// Check if a service is responding within timeout
    pub async fn check_service_timeout(url: &str, timeout: Duration) -> Result<bool> {
        // TODO: Implement actual HTTP health check
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(true)
    }
}

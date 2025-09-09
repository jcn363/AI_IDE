//! # Resource Monitor Module
//!
//! System resource monitoring functionality for the model loader.

use crate::resource_types::{ModelSize, Quantization, BYTES_PER_GB};
use std::sync::Arc;
use sysinfo::System;
use tokio::sync::RwLock;
use tracing::debug;

/// System resource monitor
#[derive(Debug, Clone)]
pub struct SystemMonitor {
    system: Arc<RwLock<System>>,
}

impl SystemMonitor {
    /// Create a new system monitor
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            system: Arc::new(RwLock::new(system)),
        }
    }

    /// Get current system memory information
    pub async fn get_memory_info(&self) -> (u64, u64) {
        let system = self.system.read().await;
        (system.used_memory(), system.total_memory())
    }

    /// Get current memory usage as a percentage
    pub async fn get_memory_usage_percentage(&self) -> f64 {
        let (used, total) = self.get_memory_info().await;
        if total == 0 {
            0.0
        } else {
            (used as f64 / total as f64) * 100.0
        }
    }

    /// Check if system has sufficient memory for a new model
    pub async fn has_sufficient_memory(&self, required_memory: u64) -> bool {
        let (used, total) = self.get_memory_info().await;
        let available = total - used;

        debug!(
            "Memory check - Required: {} bytes, Available: {} bytes",
            required_memory, available
        );
        available > required_memory
    }

    /// Calculate memory requirement for a model using centralized constants
    pub fn estimate_memory_requirement(
        model_size: ModelSize,
        quantization: Option<Quantization>,
    ) -> u64 {
        let base_memory_mb = match model_size {
            ModelSize::Small => 500.0,       // 500MB
            ModelSize::Medium => 1000.0,     // 1GB
            ModelSize::Large => 2000.0,      // 2GB
            ModelSize::ExtraLarge => 4000.0, // 4GB
        };

        let multiplier = match quantization.unwrap_or(Quantization::FP32) {
            Quantization::FP32 => 1.0,
            Quantization::FP16 => 0.5,
            Quantization::INT8 => 0.25,
            Quantization::INT4 => 0.125,
        };

        (base_memory_mb * multiplier) as u64 * 1024 * 1024 // Convert MB to bytes
    }

    /// Refresh system information
    pub async fn refresh(&self) {
        let mut system = self.system.write().await;
        system.refresh_all();
    }

    /// Get detailed system resource information
    pub async fn get_resource_summary(&self) -> ResourceSummary {
        let (used, total) = self.get_memory_info().await;
        let percentage = self.get_memory_usage_percentage().await;

        ResourceSummary {
            used_memory_bytes: used,
            total_memory_bytes: total,
            memory_percentage: percentage,
        }
    }
}

/// Summary of system resources
#[derive(Debug, Clone)]
pub struct ResourceSummary {
    pub used_memory_bytes: u64,
    pub total_memory_bytes: u64,
    pub memory_percentage: f64,
}

impl ResourceSummary {
    /// Check if memory usage is above a threshold
    pub fn is_above_threshold(&self, threshold_gb: f64) -> bool {
        let threshold_bytes = (threshold_gb * BYTES_PER_GB) as u64;
        self.used_memory_bytes > threshold_bytes
    }

    /// Get memory pressure level
    pub fn pressure_level(&self) -> MemoryPressure {
        match self.memory_percentage {
            p if p >= 90.0 => MemoryPressure::Critical,
            p if p >= 75.0 => MemoryPressure::High,
            p if p >= 60.0 => MemoryPressure::Moderate,
            _ => MemoryPressure::Low,
        }
    }
}

/// Memory pressure levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryPressure {
    Low,
    Moderate,
    High,
    Critical,
}

impl MemoryPressure {
    /// Check if unloading should be considered
    pub fn should_consider_unloading(&self) -> bool {
        matches!(self, MemoryPressure::High | MemoryPressure::Critical)
    }

    /// Get urgency factor for unloading (higher = more urgent)
    pub fn urgency_factor(&self) -> f32 {
        match self {
            MemoryPressure::Low => 0.0,
            MemoryPressure::Moderate => 0.5,
            MemoryPressure::High => 0.8,
            MemoryPressure::Critical => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pressure_levels() {
        let summary = ResourceSummary {
            used_memory_bytes: 0,
            total_memory_bytes: 1024 * 1024 * 1024, // 1GB
            memory_percentage: 95.0,
        };

        assert_eq!(summary.pressure_level(), MemoryPressure::Critical);
        assert!(summary.pressure_level().should_consider_unloading());
    }

    #[test]
    fn test_memory_estimation() {
        let small_fp32 =
            SystemMonitor::estimate_memory_requirement(ModelSize::Small, Some(Quantization::FP32));
        let small_int8 =
            SystemMonitor::estimate_memory_requirement(ModelSize::Small, Some(Quantization::INT8));

        assert_eq!(small_fp32, 1024 * 1024 * 500); // 500MB
        assert_eq!(small_int8, (1024 * 1024 * 500) / 4); // 125MB
    }
}

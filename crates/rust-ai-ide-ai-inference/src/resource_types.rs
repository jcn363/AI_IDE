//! # Resource Types Module
//!
//! Core types for resource management and monitoring in the model loader system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Model size enumeration for memory estimation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

/// Quantization options for model optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Quantization {
    FP32,
    FP16,
    INT8,
    INT4,
}

/// Model types available in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelType {
    CodeLlama,
    StarCoder,
}

/// Resource usage information for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_usage_bytes: u64,
    pub last_accessed:      DateTime<Utc>,
    pub access_count:       u64,
    pub load_timestamp:     DateTime<Utc>,
}

/// Unloading policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnloadingPolicy {
    /// Least Recently Used - unload models not accessed recently
    LRU { max_age_hours: u32 },
    /// Memory threshold - unload models when total memory exceeds threshold
    MemoryThreshold { max_memory_gb: f64 },
    /// Time-based - unload models older than specified time
    TimeBased { max_age_hours: u32 },
    /// Hybrid - combination of LRU and memory threshold
    Hybrid {
        max_age_hours: u32,
        max_memory_gb: f64,
    },
}

impl UnloadingPolicy {
    /// Get the memory threshold if applicable
    pub fn memory_threshold(&self) -> Option<f64> {
        match self {
            UnloadingPolicy::MemoryThreshold { max_memory_gb } => Some(*max_memory_gb),
            UnloadingPolicy::Hybrid { max_memory_gb, .. } => Some(*max_memory_gb),
            _ => None,
        }
    }

    /// Get the time threshold if applicable
    pub fn time_threshold_hours(&self) -> Option<u32> {
        match self {
            UnloadingPolicy::LRU { max_age_hours } => Some(*max_age_hours),
            UnloadingPolicy::TimeBased { max_age_hours } => Some(*max_age_hours),
            UnloadingPolicy::Hybrid { max_age_hours, .. } => Some(*max_age_hours),
            _ => None,
        }
    }

    /// Check if this policy considers memory constraints
    pub fn considers_memory(&self) -> bool {
        matches!(
            self,
            UnloadingPolicy::MemoryThreshold { .. } | UnloadingPolicy::Hybrid { .. }
        )
    }

    /// Check if this policy considers access time
    pub fn considers_access_time(&self) -> bool {
        matches!(
            self,
            UnloadingPolicy::LRU { .. } | UnloadingPolicy::Hybrid { .. }
        )
    }

    /// Check if this policy considers load time
    pub fn considers_load_time(&self) -> bool {
        matches!(self, UnloadingPolicy::TimeBased { .. })
    }
}

/// # Memory Conversion Constants
///
/// Centralized constants to avoid duplication of memory conversion calculations.
/// Number of bytes in one gigabyte
pub const BYTES_PER_GB: f64 = 1024.0 * 1024.0 * 1024.0;

/// Number of bytes in one megabyte
pub const BYTES_PER_MB: f64 = 1024.0 * 1024.0;

/// Number of kilobytes in one gigabyte (for cleaner calculations)
pub const KB_PER_GB: u64 = 1024 * 1024;

/// Memory pressure thresholds (percentage)
pub mod memory_thresholds {
    pub const CRITICAL_THRESHOLD: f64 = 90.0;
    pub const HIGH_THRESHOLD: f64 = 75.0;
    pub const MODERATE_THRESHOLD: f64 = 60.0;
}

/// Default timeout values (seconds)
pub mod timeouts {
    pub const DEFAULT_LOAD_TIMEOUT_SECS: u64 = 300; // 5 minutes
    pub const DEFAULT_UNLOAD_TIMEOUT_SECS: u64 = 60; // 1 minute
}

/// Quality factors for memory calculations
pub mod memory_quality {
    pub const FP32_QUALITY_FACTOR: f64 = 1.0;
    pub const FP16_QUALITY_FACTOR: f64 = 0.5;
    pub const INT8_QUALITY_FACTOR: f64 = 0.25;
    pub const INT4_QUALITY_FACTOR: f64 = 0.125;
}

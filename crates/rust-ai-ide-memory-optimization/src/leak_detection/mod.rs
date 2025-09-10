//! Memory leak detection system with SIMD acceleration
//! Provides advanced leak detection, snapshot analysis, and cross-crate monitoring
//! using parallel processing and AI-powered analysis.

mod detector;
mod snapshot;
mod analysis;

pub use detector::{LeakDetector, LeakDetectorConfig};
pub use snapshot::{MemorySnapshot, AllocationInfo, ReferenceInfo};
pub use analysis::{LeakReport, LeakType, LeakSeverity, ReferenceCycle};

/// Memory leak detection result
#[derive(Debug, Clone)]
pub enum LeakDetectionResult {
    /// No leaks detected
    Clean,
    /// Potential leaks found
    LeaksDetected(Vec<LeakReport>),
    /// Detection failed with error
    Error(String),
}
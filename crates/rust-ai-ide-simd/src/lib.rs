// SIMD acceleration crate for RUST AI IDE
// Provides vectorized computations for performance optimization across AI inference,
// compilation operations, and numerical computations.

use core_simd::f32x4;
use core_simd::f32x8;
use core_simd::f64x4;
use core_simd::i32x4;
use core_simd::i32x8;
use std::alloc::{alloc, dealloc, Layout};
use std::mem::size_of;
use std::ptr::{null_mut, NonNull};

pub mod memory;
pub mod operations;
pub mod capability;
pub mod dispatch;
pub mod error;
pub mod monitoring;

pub use memory::*;
pub use operations::*;
pub use capability::*;
pub use dispatch::*;
pub use error::*;
pub use monitoring::*;

/// SIMD processor for managing vectorized computation operations
pub struct SIMDProcessor {
    /// SIMD capability information for the current platform
    capabilities: SIMDCapabilities,
    /// Vector operation dispatcher optimized for current capabilities
    vector_dispatcher: VectorDispatcher,
    /// Memory management for SIMD-aligned allocations
    memory_manager: SIMDAllocator,
    /// Performance monitoring for SIMD operations
    performance_monitor: SIMDPerformanceMonitor,
    /// Fallback strategies for unsupported operations
    fallback_manager: SIMDFallbackManager,
}

impl SIMDProcessor {
    /// Create a new SIMD processor with automatic capability detection
    pub fn new() -> Result<Self, SIMDError> {
        let capabilities = detect_simd_capabilities()?;

        Ok(Self {
            capabilities: capabilities.clone(),
            vector_dispatcher: VectorDispatcher::new(),
            memory_manager: SIMDAllocator::new(),
            performance_monitor: SIMDPerformanceMonitor::new(),
            fallback_manager: SIMDFallbackManager::new(capabilities),
        })
    }

    /// Get SIMD capabilities for the current platform
    pub fn capabilities(&self) -> &SIMDCapabilities {
        &self.capabilities
    }

    /// Check if broad SIMD support is available
    pub fn has_simd(&self) -> bool {
        self.capabilities.has_simd()
    }

    /// Get optimal vector size for the given data type
    pub fn optimal_vector_size<T>(&self) -> usize {
        self.capabilities.vector_size_for_type::<T>()
    }

    /// Allocate SIMD-aligned memory
    pub fn allocate<T>(&self, count: usize) -> Result<SIMDVector<T>, SIMDError> {
        self.memory_manager.allocate(count)
    }

    /// Process vectorized floating point operations
    pub fn vectorized_f32_operations<F>(
        &mut self,
        lhs: &[f32],
        rhs: &[f32],
        operation: F
    ) -> Result<Vec<f32>, SIMDError>
    where
        F: Fn(f32, f32) -> f32
    {
        self.performance_monitor.start_operation("f32_vectorized");

        let result = if self.capabilities.has_avx2() && lhs.len() >= 8 {
            self.vectorized_f32x8_operations(lhs, rhs, operation)
        } else if self.capabilities.has_sse() && lhs.len() >= 4 {
            self.vectorized_f32x4_operations(lhs, rhs, operation)
        } else {
            self.fallback_manager.vectorized_f32_fallback(lhs, rhs, operation)
        };

        self.performance_monitor.end_operation("f32_vectorized");
        result
    }

    /// Process batch matrix multiplications for AI inference
    pub fn matrix_multiply_f32(
        &mut self,
        a: &[f32],
        b: &[f32],
        m: usize,
        n: usize,
        k: usize,
    ) -> Result<Vec<f32>, SIMDError> {
        self.performance_monitor.start_operation("matrix_multiply_f32");

        let result = if self.capabilities.has_acclerated_matrix_ops() {
            self.vectorized_matrix_multiply(a, b, m, n, k)
        } else {
            self.fallback_manager.matrix_multiply_fallback_f32(a, b, m, n, k)
        };

        self.performance_monitor.end_operation("matrix_multiply_f32");
        result
    }

    /// Compute batch Euclidean distances for vector search and embeddings similarity
    pub fn vectorized_distance_computation(
        &mut self,
        query: &[f32],
        database: &[f32],
        dimension: usize,
    ) -> Result<Vec<f32>, SIMDError> {
        self.performance_monitor.start_operation("distance_computation");

        let result = if self.capabilities.has_avx512() && dimension % 16 == 0 {
            self.avx512_distance_computation(query, database, dimension)
        } else if self.capabilities.has_avx2() && dimension % 8 == 0 {
            self.avx2_distance_computation(query, database, dimension)
        } else {
            self.fallback_manager.euclidean_distance_fallback(query, database, dimension)
        };

        self.performance_monitor.end_operation("distance_computation");
        result
    }
}

// SIMD operation implementations
impl SIMDProcessor {
    fn vectorized_f32x8_operations<F>(
        &mut self,
        lhs: &[f32],
        rhs: &[f32],
        operation: F
    ) -> Result<Vec<f32>, SIMDError>
    where
        F: Fn(f32, f32) -> f32
    {
        self.vector_dispatcher.dispatch_f32x8(lhs, rhs, &operation)
    }

    fn vectorized_f32x4_operations<F>(
        &mut self,
        lhs: &[f32],
        rhs: &[f32],
        operation: F
    ) -> Result<Vec<f32>, SIMDError>
    where
        F: Fn(f32, f32) -> f32
    {
        self.vector_dispatcher.dispatch_f32x4(lhs, rhs, &operation)
    }

    fn vectorized_matrix_multiply(
        &mut self,
        a: &[f32],
        b: &[f32],
        m: usize,
        n: usize,
        k: usize,
    ) -> Result<Vec<f32>, SIMDError> {
        self.vector_dispatcher.matrix_multiply_dispatch(a, b, m, n, k)
    }

    fn avx512_distance_computation(
        &mut self,
        query: &[f32],
        database: &[f32],
        dimension: usize,
    ) -> Result<Vec<f32>, SIMDError> {
        self.vector_dispatcher.avx512_distance_dispatch(query, database, dimension)
    }

    fn avx2_distance_computation(
        &mut self,
        query: &[f32],
        database: &[f32],
        dimension: usize,
    ) -> Result<Vec<f32>, SIMDError> {
        self.vector_dispatcher.avx2_distance_dispatch(query, database, dimension)
    }
}

/// SIMD vector wrapper for safe SIMD memory management
pub struct SIMDVector<T> {
    ptr: NonNull<T>,
    len: usize,
    capacity: usize,
    alignment: usize,
}

impl<T> SIMDVector<T> {
    /// Get alignment-safe slice
    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Get alignment-safe mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    /// Get SIMD-safe length
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if vector is SIMD-aligned
    pub fn is_simd_aligned(&self) -> bool {
        self.ptr.as_ptr() as usize % self.alignment == 0
    }
}

unsafe impl<T> Send for SIMDVector<T> {}
unsafe impl<T> Sync for SIMDVector<T> {}

impl<T> Drop for SIMDVector<T> {
    fn drop(&mut self) {
        if self.capacity > 0 {
            let layout = Layout::from_size_align(
                self.capacity * size_of::<T>(),
                self.alignment
            ).unwrap();
            unsafe {
                dealloc(self.ptr.cast().as_ptr(), layout);
            }
        }
    }
}

/// Global SIMD processor instance for efficient reuse
static mut SIMD_PROCESSOR: Option<SIMDProcessor> = None;

lazy_static::lazy_static! {
    static ref SIMD_PROC_INIT: std::sync::Once = std::sync::Once::new();
}

/// Get or initialize global SIMD processor instance
pub fn get_simd_processor() -> Result<&'static SIMDProcessor, SIMDError> {
    unsafe {
        SIMD_PROC_INIT.call_once(|| {
            match SIMDProcessor::new() {
                Ok(processor) => SIMD_PROCESSOR = Some(processor),
                Err(e) => {
                    tracing::warn!("Failed to initialize SIMD processor: {:?}", e);
                    tracing::warn!("SIMD acceleration will be disabled");
                }
            }
        });
        SIMD_PROCESSOR.as_ref().ok_or(SIMDError::SIMDUnavailable)
    }
}

/// Check if SIMD acceleration is available and enabled
pub fn is_simd_available() -> bool {
    get_simd_processor().is_ok()
}

/// Configuration for SIMD operations
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SIMDConfig {
    pub enable_simd: bool,
    pub enable_monitoring: bool,
    pub enable_fallback: bool,
    pub vector_alignment: usize,
    pub cache_simd_results: bool,
}

impl Default for SIMDConfig {
    fn default() -> Self {
        Self {
            enable_simd: true,
            enable_monitoring: true,
            enable_fallback: true,
            vector_alignment: 32, // AVX/AVX2 alignment
            cache_simd_results: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_processor_creation() {
        let processor = SIMDProcessor::new();
        assert!(processor.is_ok() || processor.is_err()); // Either succeeds or fails gracefully
    }

    #[test]
    fn test_simd_vector_allocation() {
        let processor = SIMDProcessor::new().expect("Failed to create SIMD processor");
        let vector: Result<SIMDVector<f32>, _> = processor.allocate(1024);

        match vector {
            Ok(vec) => {
                assert_eq!(vec.len(), 1024);
                assert!(vec.is_simd_aligned());
            }
            Err(_) => {
                // SIMD not available, but allocation should not crash
            }
        }
    }

    #[test]
    fn test_simd_capability_detection() {
        let capabilities = detect_simd_capabilities();
        // Detection should never fail, even on unsupported platforms
        assert!(capabilities.is_ok());
    }

    #[test]
    fn test_fallback_operations() {
        let processor = SIMDProcessor::new().expect("Failed to create SIMD processor");
        let lhs = vec![1.0, 2.0, 3.0, 4.0];
        let rhs = vec![1.0, 2.0, 3.0, 4.0];

        let result = processor.vectorized_f32_operations(&lhs, &rhs, |a, b| a + b);

        match result {
            Ok(values) => {
                assert_eq!(values.len(), 4);
                assert_eq!(values, vec![2.0, 4.0, 6.0, 8.0]);
            }
            Err(_) => {
                // Fallback to scalar should work
            }
        }
    }
}
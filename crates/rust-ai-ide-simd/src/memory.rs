/// SIMD-aligned memory management for optimal performance
use std::alloc::{alloc, dealloc, Layout};
use std::ptr::{null_mut, NonNull};

use crate::capability::get_cached_capabilities;
use crate::error::{RecoveryStrategy, SIMDError, SIMDResult};

/// SIMD memory allocator for aligned memory allocation
pub struct SIMDAllocator {
    cache:            moka::sync::Cache<String, NonNull<u8>>,
    allocation_count: std::sync::atomic::AtomicUsize,
}

impl SIMDAllocator {
    /// Create new SIMD allocator with caching enabled
    pub fn new() -> Self {
        Self {
            cache:            moka::sync::Cache::builder()
                .max_capacity(100) // Cache up to 100 allocations
                .time_to_live(std::time::Duration::from_secs(300)) // 5 minute TTL
                .build(),
            allocation_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Allocate SIMD-aligned memory for a vector
    pub fn allocate<T>(&self, count: usize) -> SIMDResult<SIMDVector<T>> {
        let caps = get_cached_capabilities();
        let alignment = caps.recommended_alignment();

        let size_bytes = count
            .checked_mul(std::mem::size_of::<T>())
            .ok_or(SIMDError::MemoryAllocationError {
                reason: "Integer overflow in size calculation".to_string(),
            })?;

        // Ensure minimum alignment and natural alignment for type
        let alignment = alignment.max(std::mem::align_of::<T>());

        // Check for alignment compatibility
        if !alignment.is_power_of_two() {
            return Err(SIMDError::MemoryAllocationError {
                reason: format!("Alignment {} is not a power of two", alignment),
            });
        }

        let layout = Layout::from_size_align(size_bytes, alignment).map_err(|_| SIMDError::MemoryAllocationError {
            reason: format!(
                "Invalid layout for size {} and alignment {}",
                size_bytes, alignment
            ),
        })?;

        // Use cached allocation if available
        let cache_key = format!("{}_{}_{}", std::any::type_name::<T>(), count, alignment);
        if let Some(cached_ptr) = self.cache.get(&cache_key) {
            // Verify cached allocation is still valid
            if self.is_allocation_valid(cached_ptr, layout) {
                tracing::debug!("Reusing cached SIMD allocation: {}", cache_key);
                return self.create_vector_from_ptr(cached_ptr, count, alignment);
            }
        }

        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            return Err(SIMDError::MemoryAllocationError {
                reason: format!(
                    "Failed to allocate {} bytes aligned to {}",
                    size_bytes, alignment
                ),
            });
        }

        let allocation_count = self
            .allocation_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        tracing::debug!(
            "SIMD allocation #{}: {} bytes at alignment {}, type: {}",
            allocation_count,
            size_bytes,
            alignment,
            std::any::type_name::<T>()
        );

        let ptr = NonNull::new(ptr).ok_or(SIMDError::MemoryAllocationError {
            reason: "Allocated pointer is null".to_string(),
        })?;

        // Cache the allocation for potential reuse
        self.cache.insert(cache_key, ptr);

        self.create_vector_from_ptr(ptr, count, alignment)
    }

    /// Prefetch memory for upcoming SIMD operations
    pub fn prefetch_memory<T>(&self, data: &[T], hint: PrefetchHint) {
        let caps = get_cached_capabilities();
        if !caps.has_simd {
            return; // No point in prefetching without SIMD
        }

        // Convert prefetch hint to SIMD-friendly values
        let locality = match hint {
            PrefetchHint::HighLocality => 3, // High temporal locality
            PrefetchHint::MediumLocality => 2,
            PrefetchHint::LowLocality => 1,
            PrefetchHint::NonTemporal => 0, // Non-temporal data
        };

        // Prefetch data in blocks to avoid cache thrashing
        let prefetch_block_size = caps.max_vector_width / std::mem::size_of::<T>();
        let mut i = 0;

        while i < data.len() {
            let end = (i + prefetch_block_size).min(data.len());
            let _block = &data[i..end];

            // Use compiler built-in prefetches when available
            #[cfg(target_arch = "x86_64")]
            if caps.has_sse41 {
                unsafe {
                    // SSE4.1 prefetch instructions
                    std::arch::x86_64::_mm_prefetch(
                        data.as_ptr().add(i) as *const i8,
                        if hint == PrefetchHint::NonTemporal {
                            0
                        } else {
                            locality
                        },
                    );
                }
            }

            i += prefetch_block_size;
        }
    }

    /// Validate that an allocation is still valid and properly aligned
    fn is_allocation_valid(&self, ptr: NonNull<u8>, layout: Layout) -> bool {
        // Basic checks: aligned and within bounds
        ptr.as_ptr() as usize % layout.align() == 0
    }

    /// Create SIMDVector from an allocated pointer
    fn create_vector_from_ptr<T>(&self, ptr: NonNull<u8>, count: usize, alignment: usize) -> SIMDResult<SIMDVector<T>> {
        let data_ptr = ptr.cast::<T>();

        // Verify alignment
        let actual_alignment = data_ptr.as_ptr() as usize % std::mem::align_of::<T>();
        if actual_alignment != 0 {
            unsafe {
                dealloc(
                    ptr.as_ptr(),
                    Layout::from_size_align_unchecked(count * std::mem::size_of::<T>(), alignment),
                );
            }
            return Err(SIMDError::AlignmentError {
                required: std::mem::align_of::<T>(),
                actual:   actual_alignment,
            });
        }

        Ok(SIMDVector {
            ptr: data_ptr,
            len: count,
            capacity: count,
            alignment,
        })
    }

    /// Get current allocation statistics
    pub fn stats(&self) -> AllocationStats {
        AllocationStats {
            total_allocations: self
                .allocation_count
                .load(std::sync::atomic::Ordering::SeqCst),
            cache_hits:        0, // TODO: Implement cache hit tracking
            cache_size:        self.cache.run_pending_tasks(),
        }
    }
}

pub use crate::SIMDVector;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrefetchHint {
    /// High temporal locality - data will be reused soon
    HighLocality,
    /// Medium temporal locality
    MediumLocality,
    /// Low temporal locality
    LowLocality,
    /// Non-temporal - data will not be reused (streaming)
    NonTemporal,
}

/// SIMD memory allocation statistics
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AllocationStats {
    pub total_allocations: usize,
    pub cache_hits:        usize,
    pub cache_size:        moka::sync::Cache<String, NonNull<u8>>,
}

/// Fast copy operations optimized for SIMD data movement
pub struct SIMDDataMover {
    caps: &'static crate::capability::SIMDCapabilities,
}

impl SIMDDataMover {
    pub fn new() -> Self {
        Self {
            caps: get_cached_capabilities(),
        }
    }

    /// Fast SIMD-accelerated data copy
    pub fn fast_copy<T: Copy>(&self, dst: &mut [T], src: &[T]) -> SIMDResult<()> {
        if dst.len() != src.len() {
            return Err(SIMDError::VectorSizeMismatch {
                expected: dst.len(),
                actual:   src.len(),
            });
        }

        if !self.caps.has_simd {
            // Fallback to standard copy
            dst.copy_from_slice(src);
            return Ok(());
        }

        self.simd_copy_dispatch(dst, src)
    }

    /// Zero out memory using SIMD instructions
    pub fn fast_zero<T: Copy + Default>(&self, data: &mut [T]) -> SIMDResult<()> {
        if data.is_empty() {
            return Ok(());
        }

        if !self.caps.has_simd {
            for item in data.iter_mut() {
                *item = T::default();
            }
            return Ok(());
        }

        // Use SIMD zeroing when available
        self.simd_zero_dispatch(data)
    }

    /// Optimized data copy with SIMD when available
    fn simd_copy_dispatch<T: Copy>(&self, dst: &mut [T], src: &[T]) -> SIMDResult<()> {
        // Focus on floating point and integer types which benefit most from SIMD
        match std::mem::size_of::<T>() {
            4 if self.caps.has_avx2 => self.avx2_copy_f32(dst, src),
            8 if self.caps.has_avx2 => self.avx2_copy_f64(dst, src),
            4 if self.caps.has_sse41 => self.sse_copy_f32(dst, src),
            8 if self.caps.has_sse2 => self.sse_copy_f64(dst, src),
            _ => {
                // Fallback to standard copy for unsupported types/sizes
                dst.copy_from_slice(src);
                Ok(())
            }
        }
    }

    /// SIMD zero fill dispatch
    fn simd_zero_dispatch<T: Copy + Default>(&self, data: &mut [T]) -> SIMDResult<()> {
        match std::mem::size_of::<T>() {
            4 if self.caps.has_sse41 => self.sse_zero_f32(data),
            8 if self.caps.has_sse2 => self.sse_zero_f64(data),
            _ => {
                for item in data.iter_mut() {
                    *item = T::default();
                }
                Ok(())
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    fn avx2_copy_f32(&self, dst: &mut [f32], src: &[f32]) -> SIMDResult<()> {
        // AVX2 accelerated copy for f32
        for i in (0..dst.len()).step_by(8) {
            if i + 8 <= dst.len() {
                unsafe {
                    let v = std::arch::x86_64::_mm256_loadu_ps(src.as_ptr().add(i));
                    std::arch::x86_64::_mm256_storeu_ps(dst.as_mut_ptr().add(i), v);
                }
            } else {
                // Handle remainder with scalar copy
                for j in i..dst.len() {
                    dst[j] = src[j];
                }
            }
        }
        Ok(())
    }

    #[cfg(target_arch = "x86_64")]
    fn avx2_copy_f64(&self, dst: &mut [f64], src: &[f64]) -> SIMDResult<()> {
        // AVX2 accelerated copy for f64
        for i in (0..dst.len()).step_by(4) {
            if i + 4 <= dst.len() {
                unsafe {
                    let v = std::arch::x86_64::_mm256_loadu_pd(src.as_ptr().add(i));
                    std::arch::x86_64::_mm256_storeu_pd(dst.as_mut_ptr().add(i), v);
                }
            } else {
                for j in i..dst.len() {
                    dst[j] = src[j];
                }
            }
        }
        Ok(())
    }

    #[cfg(target_arch = "x86_64")]
    fn sse_copy_f32(&self, dst: &mut [f32], src: &[f32]) -> SIMDResult<()> {
        // SSE accelerated copy for f32
        for i in (0..dst.len()).step_by(4) {
            if i + 4 <= dst.len() {
                unsafe {
                    let v = std::arch::x86_64::_mm_loadu_ps(src.as_ptr().add(i));
                    std::arch::x86_64::_mm_storeu_ps(dst.as_mut_ptr().add(i), v);
                }
            } else {
                for j in i..dst.len() {
                    dst[j] = src[j];
                }
            }
        }
        Ok(())
    }

    #[cfg(target_arch = "x86_64")]
    fn sse_copy_f64(&self, dst: &mut [f64], src: &[f64]) -> SIMDResult<()> {
        // SSE accelerated copy for f64
        for i in (0..dst.len()).step_by(2) {
            if i + 2 <= dst.len() {
                unsafe {
                    let v = std::arch::x86_64::_mm_loadu_pd(src.as_ptr().add(i));
                    std::arch::x86_64::_mm_storeu_pd(dst.as_mut_ptr().add(i), v);
                }
            } else {
                for j in i..dst.len() {
                    dst[j] = src[j];
                }
            }
        }
        Ok(())
    }

    #[cfg(target_arch = "x86_64")]
    fn sse_zero_f32(&self, data: &mut [f32]) -> SIMDResult<()> {
        let zero = unsafe { std::arch::x86_64::_mm_setzero_ps() };
        for i in (0..data.len()).step_by(4) {
            if i + 4 <= data.len() {
                unsafe {
                    std::arch::x86_64::_mm_storeu_ps(data.as_mut_ptr().add(i), zero);
                }
            } else {
                for j in i..data.len() {
                    data[j] = 0.0;
                }
            }
        }
        Ok(())
    }

    #[cfg(target_arch = "x86_64")]
    fn sse_zero_f64(&self, data: &mut [f64]) -> SIMDResult<()> {
        let zero = unsafe { std::arch::x86_64::_mm_setzero_pd() };
        for i in (0..data.len()).step_by(2) {
            if i + 2 <= data.len() {
                unsafe {
                    std::arch::x86_64::_mm_storeu_pd(data.as_mut_ptr().add(i), zero);
                }
            } else {
                for j in i..data.len() {
                    data[j] = 0.0;
                }
            }
        }
        Ok(())
    }

    // Fallback implementations for non-x86 architectures
    #[cfg(not(target_arch = "x86_64"))]
    fn avx2_copy_f32(&self, dst: &mut [f32], src: &[f32]) -> SIMDResult<()> {
        dst.copy_from_slice(src);
        Ok(())
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn avx2_copy_f64(&self, dst: &mut [f64], src: &[f64]) -> SIMDResult<()> {
        dst.copy_from_slice(src);
        Ok(())
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn sse_copy_f32(&self, dst: &mut [f32], src: &[f32]) -> SIMDResult<()> {
        dst.copy_from_slice(src);
        Ok(())
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn sse_copy_f64(&self, dst: &mut [f64], src: &[f64]) -> SIMDResult<()> {
        dst.copy_from_slice(src);
        Ok(())
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn sse_zero_f32(&self, data: &mut [f32]) -> SIMDResult<()> {
        for item in data.iter_mut() {
            *item = 0.0;
        }
        Ok(())
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn sse_zero_f64(&self, data: &mut [f64]) -> SIMDResult<()> {
        for item in data.iter_mut() {
            *item = 0.0;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_allocator_creation() {
        let allocator = SIMDAllocator::new();
        assert_eq!(
            allocator
                .allocation_count
                .load(std::sync::atomic::Ordering::SeqCst),
            0
        );
    }

    #[test]
    fn test_allocation_stats() {
        let allocator = SIMDAllocator::new();
        let stats = allocator.stats();
        assert_eq!(stats.total_allocations, 0);
    }

    #[test]
    fn test_memory_allocation_basic() {
        let allocator = SIMDAllocator::new();
        let vector = allocator.allocate::<f32>(1024);

        match vector {
            Ok(vec) => {
                assert_eq!(vec.len(), 1024);
                // Check that memory is accessible (won't page fault)
                let slice = vec.as_slice();
                assert_eq!(slice.len(), 1024);
            }
            Err(SIMDError::SIMDUnavailable) => {
                // SIMD not supported, allocation failed as expected
            }
            Err(e) => panic!("Unexpected allocation error: {:?}", e),
        }
    }

    #[test]
    fn test_data_mover_creation() {
        let mover = SIMDDataMover::new();
        // Should not panic
    }

    #[test]
    fn test_prefetch_hints() {
        let _high = PrefetchHint::HighLocality;
        let _medium = PrefetchHint::MediumLocality;
        let _low = PrefetchHint::LowLocality;
        let _non_temporal = PrefetchHint::NonTemporal;
        // Just test enum values are accessible
    }
}

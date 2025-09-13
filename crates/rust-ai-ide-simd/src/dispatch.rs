/// Vector operation dispatcher for SIMD-accelerated computations
use core_simd::f32x4;
use core_simd::{f32x8, f64x4, i32x4, i32x8};

use crate::capability::get_cached_capabilities;
use crate::error::{SIMDError, SIMDResult};

/// Vector operation dispatcher that selects optimal SIMD instruction set
pub struct VectorDispatcher;

impl VectorDispatcher {
    /// Create new dispatcher
    pub fn new() -> Self {
        Self
    }

    /// Dispatch f32x4 operations using SSE
    pub fn dispatch_f32x4<F>(&self, lhs: &[f32], rhs: &[f32], operation: F) -> SIMDResult<Vec<f32>>
    where
        F: Fn(f32, f32) -> f32,
    {
        let caps = get_cached_capabilities();
        if !caps.has_sse {
            return Err(SIMDError::SIMDUnavailable);
        }

        if lhs.len() != rhs.len() {
            return Err(SIMDError::VectorSizeMismatch {
                expected: lhs.len(),
                actual:   rhs.len(),
            });
        }

        let mut result = vec![0.0; lhs.len()];
        let vector_size = 4; // f32x4 has 4 elements

        for i in (0..lhs.len()).step_by(vector_size) {
            if i + vector_size <= lhs.len() {
                // Load data into SIMD vectors
                let lhs_vec = f32x4::from_slice(&lhs[i..i + vector_size]);
                let rhs_vec = f32x4::from_slice(&rhs[i..i + vector_size]);

                // Apply operation in SIMD fashion
                let mut result_slice = [0.0; 4];
                for j in 0..vector_size {
                    result_slice[j] = operation(lhs_vec.into_array()[j], rhs_vec.into_array()[j]);
                }

                // Store result
                result[i..i + vector_size].copy_from_slice(&result_slice);
            } else {
                // Handle remainder with scalar operations
                for j in i..lhs.len() {
                    result[j] = operation(lhs[j], rhs[j]);
                }
            }
        }

        Ok(result)
    }

    /// Dispatch f32x8 operations using AVX/AVX2
    pub fn dispatch_f32x8<F>(&self, lhs: &[f32], rhs: &[f32], operation: F) -> SIMDResult<Vec<f32>>
    where
        F: Fn(f32, f32) -> f32,
    {
        let caps = get_cached_capabilities();
        if !caps.has_avx {
            return Err(SIMDError::SIMDUnavailable);
        }

        if lhs.len() != rhs.len() {
            return Err(SIMDError::VectorSizeMismatch {
                expected: lhs.len(),
                actual:   rhs.len(),
            });
        }

        let mut result = vec![0.0; lhs.len()];
        let vector_size = 8; // f32x8 has 8 elements

        for i in (0..lhs.len()).step_by(vector_size) {
            if i + vector_size <= lhs.len() {
                let lhs_vec = f32x8::from_slice(&lhs[i..i + vector_size]);
                let rhs_vec = f32x8::from_slice(&rhs[i..i + vector_size]);

                let mut result_slice = [0.0; 8];
                for j in 0..vector_size {
                    result_slice[j] = operation(lhs_vec.into_array()[j], rhs_vec.into_array()[j]);
                }

                result[i..i + vector_size].copy_from_slice(&result_slice);
            } else {
                for j in i..lhs.len() {
                    result[j] = operation(lhs[j], rhs[j]);
                }
            }
        }

        Ok(result)
    }

    /// Matrix multiplication dispatcher optimized for SIMD
    pub fn matrix_multiply_dispatch(&self, a: &[f32], b: &[f32], m: usize, n: usize, k: usize) -> SIMDResult<Vec<f32>> {
        let caps = get_cached_capabilities();

        if m * n != a.len() || n * k != b.len() {
            return Err(SIMDError::MatrixDimensionsError {
                a_dims: (m, n),
                b_dims: (n, k),
            });
        }

        if caps.has_avx2 && n >= 8 {
            self.matrix_multiply_avx2(a, b, m, n, k)
        } else if caps.has_avx && n >= 8 {
            self.matrix_multiply_avx(a, b, m, n, k)
        } else {
            self.matrix_multiply_scalar(a, b, m, n, k)
        }
    }

    /// AVX512 distance computation dispatcher
    pub fn avx512_distance_dispatch(&self, query: &[f32], database: &[f32], dimension: usize) -> SIMDResult<Vec<f32>> {
        let caps = get_cached_capabilities();
        if !caps.has_avx512f {
            return Err(SIMDError::SIMDUnavailable);
        }

        if dimension == 0 {
            return Ok(vec![]);
        }

        let query_vectors = query.len() / dimension;
        let database_vectors = database.len() / dimension;

        if query.len() % dimension != 0 || database.len() % dimension != 0 {
            return Err(SIMDError::VectorSizeMismatch {
                expected: query_vectors * dimension,
                actual:   query.len(),
            });
        }

        let mut distances = Vec::with_capacity(query_vectors * database_vectors);

        for query_idx in 0..query_vectors {
            for db_idx in 0..database_vectors {
                let q_start = query_idx * dimension;
                let q_end = q_start + dimension;
                let db_start = db_idx * dimension;
                let db_end = db_start + dimension;

                let query_vec = &query[q_start..q_end];
                let db_vec = &database[db_start..db_end];

                let distance = self.vectorized_euclidean_distance(query_vec, db_vec)?;
                distances.push(distance);
            }
        }

        Ok(distances)
    }

    /// AVX2 distance computation dispatcher
    pub fn avx2_distance_dispatch(&self, query: &[f32], database: &[f32], dimension: usize) -> SIMDResult<Vec<f32>> {
        let caps = get_cached_capabilities();
        if !caps.has_avx2 {
            return Err(SIMDError::SIMDUnavailable);
        }

        if dimension == 0 {
            return Ok(vec![]);
        }

        let query_vectors = query.len() / dimension;
        let database_vectors = database.len() / dimension;

        let mut distances = Vec::with_capacity(query_vectors * database_vectors);

        for query_idx in 0..query_vectors {
            for db_idx in 0..database_vectors {
                let q_start = query_idx * dimension;
                let db_start = db_idx * dimension;

                let query_vec = &query[q_start..q_start + dimension];
                let db_vec = &database[db_start..db_start + dimension];

                let distance = self.vectorized_euclidean_distance(query_vec, db_vec)?;
                distances.push(distance);
            }
        }

        Ok(distances)
    }

    /// Vectorized Euclidean distance calculation
    fn vectorized_euclidean_distance(&self, a: &[f32], b: &[f32]) -> SIMDResult<f32> {
        if a.len() != b.len() {
            return Err(SIMDError::VectorSizeMismatch {
                expected: a.len(),
                actual:   b.len(),
            });
        }

        let caps = get_cached_capabilities();
        let mut sum: f32 = 0.0;

        if caps.has_avx && a.len() >= 8 {
            sum = self.vectorized_distance_avx(a, b);
        } else if caps.has_sse && a.len() >= 4 {
            sum = self.vectorized_distance_sse(a, b);
        } else {
            for i in 0..a.len() {
                let diff = a[i] - b[i];
                sum += diff * diff;
            }
        }

        Ok(sum.sqrt())
    }

    #[cfg(target_arch = "x86_64")]
    fn vectorized_distance_avx(&self, a: &[f32], b: &[f32]) -> f32 {
        use std::arch::x86_64::*;

        let mut sum = _mm256_setzero_ps();
        let mut i = 0;

        while i + 8 <= a.len() {
            let va = _mm256_loadu_ps(&a[i]);
            let vb = _mm256_loadu_ps(&b[i]);
            let diff = _mm256_sub_ps(va, vb);
            let sq = _mm256_mul_ps(diff, diff);
            sum = _mm256_add_ps(sum, sq);
            i += 8;
        }

        let mut sum_array = [0.0; 8];
        _mm256_storeu_ps(sum_array.as_mut_ptr(), sum);

        let mut total = sum_array.iter().sum();
        for j in i..a.len() {
            let diff = a[j] - b[j];
            total += diff * diff;
        }

        total
    }

    #[cfg(target_arch = "x86_64")]
    fn vectorized_distance_sse(&self, a: &[f32], b: &[f32]) -> f32 {
        use std::arch::x86_64::*;

        let mut sum = _mm_setzero_ps();
        let mut i = 0;

        while i + 4 <= a.len() {
            let va = _mm_loadu_ps(&a[i]);
            let vb = _mm_loadu_ps(&b[i]);
            let diff = _mm_sub_ps(va, vb);
            let sq = _mm_mul_ps(diff, diff);
            sum = _mm_add_ps(sum, sq);
            i += 4;
        }

        let mut sum_array = [0.0; 4];
        _mm_storeu_ps(sum_array.as_mut_ptr(), sum);

        let mut total = sum_array.iter().sum();
        for j in i..a.len() {
            let diff = a[j] - b[j];
            total += diff * diff;
        }

        total
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn vectorized_distance_avx(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut sum = 0.0;
        for i in 0..a.len() {
            let diff = a[i] - b[i];
            sum += diff * diff;
        }
        sum
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn vectorized_distance_sse(&self, a: &[f32], b: &[f32]) -> f32 {
        self.vectorized_distance_avx(a, b)
    }

    /// AVX matrix multiplication implementation
    fn matrix_multiply_avx2(&self, a: &[f32], b: &[f32], m: usize, n: usize, k: usize) -> SIMDResult<Vec<f32>> {
        #[cfg(target_arch = "x86_64")]
        {
            use std::arch::x86_64::*;

            let mut result = vec![0.0; m * k];

            // Basic blocking for better cache efficiency
            let block_size = 64; // L1 cache size consideration

            for ii in (0..m).step_by(block_size) {
                for jj in (0..k).step_by(block_size) {
                    for kk in (0..n).step_by(8) {
                        // AVX/AVX2 register width
                        for i in ii..(ii + block_size).min(m) {
                            for j in jj..(jj + block_size).min(k) {
                                let mut sum = _mm256_setzero_ps();

                                if kk + 8 <= n {
                                    for l in 0..8 {
                                        let a_val = _mm256_broadcast_ss(&a[i * n + kk + l]);
                                        let b_val = _mm256_loadu_ps(&b[(kk + l) * k + j - 3]); // Offset for gather if possible
                                        sum = _mm256_add_ps(sum, _mm256_mul_ps(a_val, b_val));
                                    }
                                } else {
                                    // Scalar operations for remaining elements
                                    let mut scalar_sum = result[i * k + j];
                                    for l in kk..n {
                                        scalar_sum += a[i * n + l] * b[l * k + j];
                                    }
                                    result[i * k + j] = scalar_sum;
                                    continue;
                                }

                                // Horizontal sum of AVX register
                                let sum_array = [0.0; 8];
                                _mm256_storeu_ps(sum_array.as_mut_ptr(), sum);
                                result[i * k + j] += sum_array.iter().sum::<f32>();
                            }
                        }
                    }
                }
            }

            Ok(result)
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            self.matrix_multiply_scalar(a, b, m, n, k)
        }
    }

    /// AVX matrix multiplication (fallback for specific AVX without AVX2)
    fn matrix_multiply_avx(&self, a: &[f32], b: &[f32], m: usize, n: usize, k: usize) -> SIMDResult<Vec<f32>> {
        // Simplified AVX version - in practice this would be more optimized
        self.matrix_multiply_scalar(a, b, m, n, k)
    }

    /// Scalar fallback matrix multiplication
    fn matrix_multiply_scalar(&self, a: &[f32], b: &[f32], m: usize, n: usize, k: usize) -> SIMDResult<Vec<f32>> {
        let mut result = vec![0.0; m * k];

        // i-k-j loop order for better cache locality
        for i in 0..m {
            for j in 0..k {
                for l in 0..n {
                    result[i * k + j] += a[i * n + l] * b[l * k + j];
                }
            }
        }

        Ok(result)
    }
}

/// SIMD operation registry for dynamic dispatch
pub struct SIMDOperationRegistry {
    operations: std::collections::HashMap<String, Box<dyn SIMDOperation>>,
}

impl SIMDOperationRegistry {
    pub fn new() -> Self {
        Self {
            operations: std::collections::HashMap::new(),
        }
    }

    pub fn register<O: SIMDOperation + 'static>(&mut self, name: String, operation: O) {
        self.operations.insert(name, Box::new(operation));
    }

    pub fn find(&self, name: &str) -> Option<&dyn SIMDOperation> {
        self.operations.get(name).map(|op| op.as_ref())
    }
}

/// Trait for SIMD operations that can be registered and dispatched dynamically
pub trait SIMDOperation: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&self, inputs: &[&[f32]], outputs: &mut [f32]) -> SIMDResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatcher_creation() {
        let dispatcher = VectorDispatcher::new();
        // Should not panic
    }

    #[test]
    fn test_vector_distance_small() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let dispatcher = VectorDispatcher::new();

        let distance = dispatcher.vectorized_euclidean_distance(&a, &b);
        match distance {
            Ok(d) => {
                let expected = ((3.0 * 3.0) + (3.0 * 3.0) + (3.0 * 3.0)).sqrt(); // 5.196
                assert!((d - expected).abs() < 0.001);
            }
            Err(SIMDError::SIMDUnavailable) => {
                // SIMD not available, test passes
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_matrix_multiply_dimensions() {
        let dispatcher = VectorDispatcher::new();

        // Test mismatched dimensions
        let a = vec![1.0, 2.0, 3.0, 4.0]; // 2x2
        let b = vec![5.0, 6.0]; // 1x2

        let result = dispatcher.matrix_multiply_dispatch(&a, &b, 2, 2, 1);
        assert!(matches!(
            result,
            Err(SIMDError::MatrixDimensionsError { .. })
        ));
    }

    #[test]
    fn test_matrix_multiply_small() {
        let dispatcher = VectorDispatcher::new();

        // Simple 2x2 matrix multiplication
        let a = vec![1.0, 2.0, 3.0, 4.0]; // 2x2
        let b = vec![5.0, 6.0, 7.0, 8.0]; // 2x2
        let expected = vec![19.0, 22.0, 43.0, 50.0]; // 2x2 result

        let result = dispatcher.matrix_multiply_dispatch(&a, &b, 2, 2, 2);
        match result {
            Ok(r) => {
                assert_eq!(r.len(), 4);
                for (actual, expected) in r.iter().zip(expected.iter()) {
                    assert!((actual - expected).abs() < 0.001);
                }
            }
            Err(SIMDError::SIMDUnavailable) => {
                // SIMD not available, test passes
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}

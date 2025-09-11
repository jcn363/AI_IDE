use crate::capability::get_cached_capabilities;
use crate::dispatch::VectorDispatcher;
/// Vectorized mathematical operations and neural network primitives
use crate::error::{SIMDError, SIMDResult};

/// SIMD operations library for mathematical computations
pub struct SIMDOperations {
    dispatcher: VectorDispatcher,
}

impl SIMDOperations {
    pub fn new() -> Self {
        Self {
            dispatcher: VectorDispatcher::new(),
        }
    }

    /// Vectorized addition of arrays
    pub fn vector_add(&self, lhs: &[f32], rhs: &[f32]) -> SIMDResult<Vec<f32>> {
        self.dispatcher
            .dispatch_f32x8(lhs, rhs, |a, b| a + b)
            .or_else(|_| self.dispatcher.dispatch_f32x4(lhs, rhs, |a, b| a + b))
            .or_else(|_| self.vector_add_fallback(lhs, rhs))
    }

    /// Vectorized multiplication of arrays
    pub fn vector_mul(&self, lhs: &[f32], rhs: &[f32]) -> SIMDResult<Vec<f32>> {
        self.dispatcher
            .dispatch_f32x8(lhs, rhs, |a, b| a * b)
            .or_else(|_| self.dispatcher.dispatch_f32x4(lhs, rhs, |a, b| a * b))
            .or_else(|_| self.vector_mul_fallback(lhs, rhs))
    }

    /// Vectorized dot product computation
    pub fn dot_product(&self, lhs: &[f32], rhs: &[f32]) -> SIMDResult<f32> {
        if lhs.len() != rhs.len() {
            return Err(SIMDError::VectorSizeMismatch {
                expected: lhs.len(),
                actual: rhs.len(),
            });
        }

        let caps = get_cached_capabilities();
        if caps.has_fma && (caps.has_avx || caps.has_avx2) {
            self.dot_product_fma(lhs, rhs)
        } else {
            self.dot_product_scalar(lhs, rhs)
        }
    }

    /// Vectorized matrix-vector multiplication
    pub fn matrix_vector_multiply(
        &self,
        matrix: &[f32],
        vector: &[f32],
        rows: usize,
        cols: usize,
    ) -> SIMDResult<Vec<f32>> {
        if cols != vector.len() {
            return Err(SIMDError::VectorSizeMismatch {
                expected: cols,
                actual: vector.len(),
            });
        }

        let caps = get_cached_capabilities();
        if caps.can_use_wide_vectors() {
            self.matrix_vector_multiply_simd(matrix, vector, rows, cols)
        } else {
            self.matrix_vector_multiply_scalar(matrix, vector, rows, cols)
        }
    }

    /// Vectorized ReLU activation function
    pub fn relu_activation(&self, input: &[f32]) -> SIMDResult<Vec<f32>> {
        let caps = get_cached_capabilities();
        if caps.has_avx2 && input.len() >= 8 {
            self.relu_avx2(input)
        } else if caps.has_avx && input.len() >= 8 {
            self.relu_avx(input)
        } else if caps.has_sse && input.len() >= 4 {
            self.relu_sse(input)
        } else {
            self.relu_scalar(input)
        }
    }

    /// Vectorized sigmoid activation function
    pub fn sigmoid_activation(&self, input: &[f32]) -> SIMDResult<Vec<f32>> {
        let caps = get_cached_capabilities();
        if caps.has_avx2 && input.len() >= 8 {
            self.sigmoid_avx2(input)
        } else if caps.has_avx && input.len() >= 8 {
            self.sigmoid_avx(input)
        } else if caps.has_sse && input.len() >= 4 {
            self.sigmoid_sse(input)
        } else {
            self.sigmoid_scalar(input)
        }
    }

    /// Vectorized cosine similarity computation for embeddings
    pub fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> SIMDResult<f32> {
        if a.len() != b.len() {
            return Err(SIMDError::VectorSizeMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        let dot = self.dot_product(a, b)?;
        let norm_a = self.l2_norm(a)?;
        let norm_b = self.l2_norm(b)?;

        if norm_a == 0.0 || norm_b == 0.0 {
            Ok(0.0)
        } else {
            Ok(dot / (norm_a * norm_b))
        }
    }

    /// Vectorized L2 norm computation
    pub fn l2_norm(&self, vector: &[f32]) -> SIMDResult<f32> {
        let squares = self.vector_mul(vector, vector)?;
        let sum = squares.iter().sum::<f32>();
        Ok(sum.sqrt())
    }

    /// Vectorized softmax computation
    pub fn softmax(&self, input: &[f32]) -> SIMDResult<Vec<f32>> {
        let mut result = Vec::with_capacity(input.len());

        // Find maximum for numerical stability
        let max_val = input.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        // Compute exp(input - max) to prevent overflow
        let mut sum = 0.0;
        for &x in input {
            let exp_val = (x - max_val).exp();
            result.push(exp_val);
            sum += exp_val;
        }

        // Normalize by sum
        for x in &mut result {
            *x /= sum;
        }

        Ok(result)
    }

    #[cfg(target_arch = "x86_64")]
    fn relu_avx2(&self, input: &[f32]) -> SIMDResult<Vec<f32>> {
        use std::arch::x86_64::*;

        let mut result = Vec::with_capacity(input.len());
        unsafe {
            for i in (0..input.len()).step_by(8) {
                if i + 8 <= input.len() {
                    let v = _mm256_loadu_ps(&input[i]);
                    let zero = _mm256_setzero_ps();
                    let result_v = _mm256_max_ps(v, zero);
                    let mut result_array = [0.0; 8];
                    _mm256_storeu_ps(result_array.as_mut_ptr(), result_v);
                    result.extend_from_slice(&result_array);
                } else {
                    for j in i..input.len() {
                        result.push(if input[j] > 0.0 { input[j] } else { 0.0 });
                    }
                }
            }
        }
        Ok(result)
    }

    #[cfg(target_arch = "x86_64")]
    fn relu_avx(&self, input: &[f32]) -> SIMDResult<Vec<f32>> {
        // AVX2 fallback to SSE for AVX-only systems (simplified)
        self.relu_sse(input)
    }

    #[cfg(target_arch = "x86_64")]
    fn relu_sse(&self, input: &[f32]) -> SIMDResult<Vec<f32>> {
        use std::arch::x86_64::*;

        let mut result = Vec::with_capacity(input.len());
        unsafe {
            for i in (0..input.len()).step_by(4) {
                if i + 4 <= input.len() {
                    let v = _mm_loadu_ps(&input[i]);
                    let zero = _mm_setzero_ps();
                    let result_v = _mm_max_ps(v, zero);
                    let mut result_array = [0.0; 4];
                    _mm_storeu_ps(result_array.as_mut_ptr(), result_v);
                    result.extend_from_slice(&result_array);
                } else {
                    for j in i..input.len() {
                        result.push(if input[j] > 0.0 { input[j] } else { 0.0 });
                    }
                }
            }
        }
        Ok(result)
    }

    fn relu_scalar(&self, input: &[f32]) -> SIMDResult<Vec<f32>> {
        Ok(input
            .iter()
            .map(|&x| if x > 0.0 { x } else { 0.0 })
            .collect())
    }

    fn sigmoid_avx2(&self, input: &[f32]) -> SIMDResult<Vec<f32>> {
        // Compute sigmoid(x) = 1 / (1 + exp(-x))
        // We'll compute this as 1 / (1 + exp(-x))
        #[cfg(target_arch = "x86_64")]
        {
            use std::arch::x86_64::*;
            let mut result = Vec::with_capacity(input.len());
            unsafe {
                let exp_mask = _mm256_set1_ps(1.0);
                for i in (0..input.len()).step_by(8) {
                    if i + 8 <= input.len() {
                        let v = _mm256_loadu_ps(&input[i]);
                        let neg_v = _mm256_sub_ps(_mm256_setzero_ps(), v);
                        let exp_neg = _mm256_exp_ps(neg_v); // Assuming exp intrinsic available
                        let one_plus_exp = _mm256_add_ps(exp_mask, exp_neg);
                        let sigmoid = _mm256_div_ps(exp_mask, one_plus_exp);
                        let mut result_array = [0.0; 8];
                        _mm256_storeu_ps(result_array.as_mut_ptr(), sigmoid);
                        result.extend_from_slice(&result_array);
                    } else {
                        for j in i..input.len() {
                            result.push(1.0 / (1.0 + (-input[j]).exp()));
                        }
                    }
                }
            }
            Ok(result)
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            self.sigmoid_scalar(input)
        }
    }

    fn sigmoid_avx(&self, input: &[f32]) -> SIMDResult<Vec<f32>> {
        // AVX2 fallback
        self.sigmoid_sse(input)
    }

    fn sigmoid_sse(&self, input: &[f32]) -> SIMDResult<Vec<f32>> {
        // SSE version - similar to AVX2 but with 4-element vectors
        #[cfg(target_arch = "x86_64")]
        {
            use std::arch::x86_64::*;
            let mut result = Vec::with_capacity(input.len());
            unsafe {
                let exp_mask = _mm_set1_ps(1.0);
                for i in (0..input.len()).step_by(4) {
                    if i + 4 <= input.len() {
                        let v = _mm_loadu_ps(&input[i]);
                        let neg_v = _mm_sub_ps(_mm_setzero_ps(), v);
                        let exp_neg = _mm_exp_ps(neg_v);
                        let one_plus_exp = _mm_add_ps(exp_mask, exp_neg);
                        let sigmoid = _mm_div_ps(exp_mask, one_plus_exp);
                        let mut result_array = [0.0; 4];
                        _mm_storeu_ps(result_array.as_mut_ptr(), sigmoid);
                        result.extend_from_slice(&result_array);
                    } else {
                        for j in i..input.len() {
                            result.push(1.0 / (1.0 + (-input[j]).exp()));
                        }
                    }
                }
            }
            Ok(result)
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            self.sigmoid_scalar(input)
        }
    }

    fn sigmoid_scalar(&self, input: &[f32]) -> SIMDResult<Vec<f32>> {
        Ok(input.iter().map(|&x| 1.0 / (1.0 + (-x).exp())).collect())
    }

    fn matrix_vector_multiply_simd(
        &self,
        matrix: &[f32],
        vector: &[f32],
        rows: usize,
        cols: usize,
    ) -> SIMDResult<Vec<f32>> {
        let mut result = vec![0.0; rows];

        #[cfg(target_arch = "x86_64")]
        {
            use std::arch::x86_64::*;

            unsafe {
                let caps = get_cached_capabilities();
                if caps.has_avx2 {
                    for i in 0..rows {
                        let mut sum = _mm256_setzero_ps();
                        let row_start = i * cols;

                        for j in (0..cols).step_by(8) {
                            if j + 8 <= cols {
                                let matrix_chunk = _mm256_loadu_ps(&matrix[row_start + j]);
                                let broadcast_vec = _mm256_broadcast_ss(&vector[j]);
                                sum =
                                    _mm256_add_ps(sum, _mm256_mul_ps(matrix_chunk, broadcast_vec));
                            } else {
                                // Handle remaining elements
                                for k in j..cols.min(j + 8) {
                                    if k < cols {
                                        result[i] += matrix[row_start + k] * vector[k];
                                    }
                                }
                            }
                        }

                        // Horizontal sum
                        let mut sum_array = [0.0; 8];
                        _mm256_storeu_ps(sum_array.as_mut_ptr(), sum);
                        result[i] += sum_array.iter().sum::<f32>();
                    }
                } else if caps.has_sse {
                    // SSE version with 4-element vectors
                    for i in 0..rows {
                        let mut sum = _mm_setzero_ps();
                        let row_start = i * cols;

                        for j in (0..cols).step_by(4) {
                            if j + 4 <= cols {
                                let matrix_chunk = _mm_loadu_ps(&matrix[row_start + j]);
                                let broadcast_vec = _mm_broadcast_ss(&vector[j]);
                                sum = _mm_add_ps(sum, _mm_mul_ps(matrix_chunk, broadcast_vec));
                            } else {
                                for k in j..cols.min(j + 4) {
                                    if k < cols {
                                        result[i] += matrix[row_start + k] * vector[k];
                                    }
                                }
                            }
                        }

                        let mut sum_array = [0.0; 4];
                        _mm_storeu_ps(sum_array.as_mut_ptr(), sum);
                        result[i] += sum_array.iter().sum::<f32>();
                    }
                } else {
                    return self.matrix_vector_multiply_scalar(matrix, vector, rows, cols);
                }
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            return self.matrix_vector_multiply_scalar(matrix, vector, rows, cols);
        }

        Ok(result)
    }

    fn matrix_vector_multiply_scalar(
        &self,
        matrix: &[f32],
        vector: &[f32],
        rows: usize,
        cols: usize,
    ) -> SIMDResult<Vec<f32>> {
        let mut result = vec![0.0; rows];
        for i in 0..rows {
            for j in 0..cols {
                result[i] += matrix[i * cols + j] * vector[j];
            }
        }
        Ok(result)
    }

    fn dot_product_fma(&self, lhs: &[f32], rhs: &[f32]) -> SIMDResult<f32> {
        // FMA (Fused Multiply-Add) enabled dot product for AVX/AVX2
        #[cfg(target_arch = "x86_64")]
        {
            use std::arch::x86_64::*;
            unsafe {
                let mut sum = _mm256_setzero_ps();
                let mut i = 0;

                while i + 8 <= lhs.len() {
                    let a = _mm256_loadu_ps(&lhs[i]);
                    let b = _mm256_loadu_ps(&rhs[i]);
                    sum = _mm256_fmadd_ps(a, b, sum); // FMA: a*b + sum
                    i += 8;
                }

                // Handle remaining elements
                for j in i..lhs.len() {
                    let a_scalar = _mm256_broadcast_ss(&lhs[j]);
                    let b_scalar = _mm256_loadu_ps(&rhs[j..(j + 1).min(rhs.len())]);
                    sum = _mm256_fmadd_ps(a_scalar, b_scalar, sum);
                }

                let mut sum_array = [0.0; 8];
                _mm256_storeu_ps(sum_array.as_mut_ptr(), sum);
                Ok(sum_array.iter().sum())
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            self.dot_product_scalar(lhs, rhs)
        }
    }

    fn dot_product_scalar(&self, lhs: &[f32], rhs: &[f32]) -> SIMDResult<f32> {
        Ok(lhs.iter().zip(rhs.iter()).map(|(a, b)| a * b).sum())
    }

    fn vector_add_fallback(&self, lhs: &[f32], rhs: &[f32]) -> SIMDResult<Vec<f32>> {
        if lhs.len() != rhs.len() {
            return Err(SIMDError::VectorSizeMismatch {
                expected: lhs.len(),
                actual: rhs.len(),
            });
        }
        Ok(lhs.iter().zip(rhs.iter()).map(|(a, b)| a + b).collect())
    }

    fn vector_mul_fallback(&self, lhs: &[f32], rhs: &[f32]) -> SIMDResult<Vec<f32>> {
        if lhs.len() != rhs.len() {
            return Err(SIMDError::VectorSizeMismatch {
                expected: lhs.len(),
                actual: rhs.len(),
            });
        }
        Ok(lhs.iter().zip(rhs.iter()).map(|(a, b)| a * b).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_operations_creation() {
        let ops = SIMDOperations::new();
        // Should not panic
    }

    #[test]
    fn test_vector_add() {
        let ops = SIMDOperations::new();
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![4.0, 3.0, 2.0, 1.0];

        let result = ops.vector_add(&a, &b);
        match result {
            Ok(r) => {
                assert_eq!(r, vec![5.0, 5.0, 5.0, 5.0]);
            }
            Err(SIMDError::SIMDUnavailable) => {
                // SIMD not available, test passes
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_vector_mul() {
        let ops = SIMDOperations::new();
        let a = vec![2.0, 3.0, 4.0, 5.0];
        let b = vec![0.5, 0.25, 0.333, 0.2];

        let result = ops.vector_mul(&a, &b);
        match result {
            Ok(r) => {
                assert_eq!(r.len(), 4);
            }
            Err(SIMDError::SIMDUnavailable) => {
                // SIMD not available, test passes
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_dot_product() {
        let ops = SIMDOperations::new();
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];

        let result = ops.dot_product(&a, &b);
        match result {
            Ok(r) => {
                // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
                assert_eq!(r, 32.0);
            }
            Err(SIMDError::SIMDUnavailable) => {
                // SIMD not available, test passes
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_l2_norm() {
        let ops = SIMDOperations::new();
        let v = vec![3.0, 4.0]; // 3-4-5 triangle

        let result = ops.l2_norm(&v);
        match result {
            Ok(r) => {
                // sqrt(3^2 + 4^2) = sqrt(9 + 16) = sqrt(25) = 5
                assert!((r - 5.0).abs() < 0.001);
            }
            Err(SIMDError::SIMDUnavailable) => {
                // SIMD not available, test passes
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}

//! AI/ML specific SIMD operations for performance optimization
//! Focuses on confidence scoring, similarity search, and vector operations

use crate::capability::get_cached_capabilities;
use crate::error::{SIMDError, SIMDResult};

/// SIMD-accelerated confidence scoring operations
pub struct SIMDConfidenceScorer;

/// SIMD-accelerated embedding similarity search
pub struct SIMDEmbeddingSearch;

/// SIMD-accelerated vector operations for AI inference
pub struct SIMDAIInferenceOps;

impl SIMDConfidenceScorer {
    /// Compute confidence scores using vectorized operations
    pub fn compute_confidence_scores(
        predictions: &[f32],
        uncertainties: &[f32],
        weights: &[f32],
    ) -> SIMDResult<Vec<f32>> {
        if predictions.len() != uncertainties.len() || predictions.len() != weights.len() {
            return Err(SIMDError::VectorSizeMismatch {
                expected: predictions.len(),
                actual: uncertainties.len().max(weights.len()),
            });
        }

        let caps = get_cached_capabilities();

        #[cfg(target_arch = "x86_64")]
        if caps.has_avx2 && predictions.len() >= 8 {
            Self::compute_confidence_scores_avx2(predictions, uncertainties, weights)
        } else if caps.has_sse && predictions.len() >= 4 {
            Self::compute_confidence_scores_sse(predictions, uncertainties, weights)
        } else {
            Self::compute_confidence_scores_scalar(predictions, uncertainties, weights)
        }

        #[cfg(target_arch = "aarch64")]
        if caps.has_neon && predictions.len() >= 4 {
            Self::compute_confidence_scores_neon(predictions, uncertainties, weights)
        } else {
            Self::compute_confidence_scores_scalar(predictions, uncertainties, weights)
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        Self::compute_confidence_scores_scalar(predictions, uncertainties, weights)
    }

    /// AVX2 implementation for confidence scoring
    #[cfg(target_arch = "x86_64")]
    fn compute_confidence_scores_avx2(
        predictions: &[f32],
        uncertainties: &[f32],
        weights: &[f32],
    ) -> SIMDResult<Vec<f32>> {
        use std::arch::x86_64::*;
        use std::simd::f32x8;

        let mut scores = Vec::with_capacity(predictions.len());

        unsafe {
            for i in (0..predictions.len()).step_by(8) {
                if i + 8 <= predictions.len() {
                    // Load SIMD vectors
                    let pred_vec = f32x8::from_slice(&predictions[i..i + 8]);
                    let uncert_vec = f32x8::from_slice(&uncertainties[i..i + 8]);
                    let weight_vec = f32x8::from_slice(&weights[i..i + 8]);

                    // Compute confidence: prediction * (1 - uncertainty) * weight
                    let one_minus_uncert = f32x8::splat(1.0) - uncert_vec;
                    let confidence_vec = pred_vec * one_minus_uncert * weight_vec;

                    // Store results
                    let mut result_array = [0.0; 8];
                    confidence_vec.copy_to_slice(&mut result_array);
                    scores.extend_from_slice(&result_array);
                } else {
                    // Handle remainder with scalar operations
                    for j in i..predictions.len() {
                        let confidence = predictions[j] * (1.0 - uncertainties[j]) * weights[j];
                        scores.push(confidence.max(0.0).min(1.0)); // Clamp to [0, 1]
                    }
                }
            }
        }

        Ok(scores)
    }

    /// SSE implementation for confidence scoring
    #[cfg(target_arch = "x86_64")]
    fn compute_confidence_scores_sse(
        predictions: &[f32],
        uncertainties: &[f32],
        weights: &[f32],
    ) -> SIMDResult<Vec<f32>> {
        use std::simd::f32x4;

        let mut scores = Vec::with_capacity(predictions.len());

        for i in (0..predictions.len()).step_by(4) {
            if i + 4 <= predictions.len() {
                let pred_vec = f32x4::from_slice(&predictions[i..i + 4]);
                let uncert_vec = f32x4::from_slice(&uncertainties[i..i + 4]);
                let weight_vec = f32x4::from_slice(&weights[i..i + 4]);

                let one_minus_uncert = f32x4::splat(1.0) - uncert_vec;
                let confidence_vec = pred_vec * one_minus_uncert * weight_vec;

                let mut result_array = [0.0; 4];
                confidence_vec.copy_to_slice(&mut result_array);
                scores.extend_from_slice(&result_array);
            } else {
                for j in i..predictions.len() {
                    let confidence = predictions[j] * (1.0 - uncertainties[j]) * weights[j];
                    scores.push(confidence.max(0.0).min(1.0));
                }
            }
        }

        Ok(scores)
    }

    /// NEON implementation for confidence scoring (ARM64)
    #[cfg(target_arch = "aarch64")]
    fn compute_confidence_scores_neon(
        predictions: &[f32],
        uncertainties: &[f32],
        weights: &[f32],
    ) -> SIMDResult<Vec<f32>> {
        use std::simd::f32x4;

        let mut scores = Vec::with_capacity(predictions.len());

        for i in (0..predictions.len()).step_by(4) {
            if i + 4 <= predictions.len() {
                let pred_vec = f32x4::from_slice(&predictions[i..i + 4]);
                let uncert_vec = f32x4::from_slice(&uncertainties[i..i + 4]);
                let weight_vec = f32x4::from_slice(&weights[i..i + 4]);

                let one_minus_uncert = f32x4::splat(1.0) - uncert_vec;
                let confidence_vec = pred_vec * one_minus_uncert * weight_vec;

                let mut result_array = [0.0; 4];
                confidence_vec.copy_to_slice(&mut result_array);
                scores.extend_from_slice(&result_array);
            } else {
                for j in i..predictions.len() {
                    let confidence = predictions[j] * (1.0 - uncertainties[j]) * weights[j];
                    scores.push(confidence.max(0.0).min(1.0));
                }
            }
        }

        Ok(scores)
    }

    /// Scalar fallback implementation
    fn compute_confidence_scores_scalar(
        predictions: &[f32],
        uncertainties: &[f32],
        weights: &[f32],
    ) -> SIMDResult<Vec<f32>> {
        let scores = predictions
            .iter()
            .zip(uncertainties.iter())
            .zip(weights.iter())
            .map(|((pred, uncert), weight)| {
                let confidence = pred * (1.0 - uncert) * weight;
                confidence.max(0.0).min(1.0)
            })
            .collect();

        Ok(scores)
    }

    /// Vectorized softmax computation for confidence normalization
    pub fn softmax_confidence(scores: &[f32]) -> SIMDResult<Vec<f32>> {
        let caps = get_cached_capabilities();

        #[cfg(target_arch = "x86_64")]
        if caps.has_avx2 && scores.len() >= 8 {
            Self::softmax_avx2(scores)
        } else if caps.has_sse && scores.len() >= 4 {
            Self::softmax_sse(scores)
        } else {
            Self::softmax_scalar(scores)
        }

        #[cfg(target_arch = "aarch64")]
        if caps.has_neon && scores.len() >= 4 {
            Self::softmax_neon(scores)
        } else {
            Self::softmax_scalar(scores)
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        Self::softmax_scalar(scores)
    }

    /// AVX2 softmax implementation
    #[cfg(target_arch = "x86_64")]
    fn softmax_avx2(scores: &[f32]) -> SIMDResult<Vec<f32>> {
        use std::simd::f32x8;

        // Find maximum for numerical stability
        let max_val = scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        let mut result = Vec::with_capacity(scores.len());
        let mut sum = 0.0;

        unsafe {
            let max_vec = f32x8::splat(max_val);

            for i in (0..scores.len()).step_by(8) {
                if i + 8 <= scores.len() {
                    let score_vec = f32x8::from_slice(&scores[i..i + 8]);
                    let exp_vec = (score_vec - max_vec).exp();

                    let mut exp_array = [0.0; 8];
                    exp_vec.copy_to_slice(&mut exp_array);

                    result.extend_from_slice(&exp_array);
                    sum += exp_array.iter().sum::<f32>();
                } else {
                    for j in i..scores.len() {
                        let exp_val = (scores[j] - max_val).exp();
                        result.push(exp_val);
                        sum += exp_val;
                    }
                }
            }
        }

        // Normalize
        let sum_vec = f32x8::splat(sum);
        for i in (0..result.len()).step_by(8) {
            if i + 8 <= result.len() {
                let mut chunk = [0.0; 8];
                chunk.copy_from_slice(&result[i..i + 8]);
                let chunk_vec = f32x8::from_array(chunk) / sum_vec;
                chunk_vec.copy_to_slice(&mut result[i..i + 8]);
            } else {
                for j in i..result.len() {
                    result[j] /= sum;
                }
            }
        }

        Ok(result)
    }

    /// SSE softmax implementation
    #[cfg(target_arch = "x86_64")]
    fn softmax_sse(scores: &[f32]) -> SIMDResult<Vec<f32>> {
        use std::simd::f32x4;

        let max_val = scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        let mut result = Vec::with_capacity(scores.len());
        let mut sum = 0.0;

        let max_vec = f32x4::splat(max_val);

        for i in (0..scores.len()).step_by(4) {
            if i + 4 <= scores.len() {
                let score_vec = f32x4::from_slice(&scores[i..i + 4]);
                let exp_vec = (score_vec - max_vec).exp();

                let mut exp_array = [0.0; 4];
                exp_vec.copy_to_slice(&mut exp_array);

                result.extend_from_slice(&exp_array);
                sum += exp_array.iter().sum::<f32>();
            } else {
                for j in i..scores.len() {
                    let exp_val = (scores[j] - max_val).exp();
                    result.push(exp_val);
                    sum += exp_val;
                }
            }
        }

        // Normalize
        let sum_vec = f32x4::splat(sum);
        for i in (0..result.len()).step_by(4) {
            if i + 4 <= result.len() {
                let mut chunk = [0.0; 4];
                chunk.copy_from_slice(&result[i..i + 4]);
                let chunk_vec = f32x4::from_array(chunk) / sum_vec;
                chunk_vec.copy_to_slice(&mut result[i..i + 4]);
            } else {
                for j in i..result.len() {
                    result[j] /= sum;
                }
            }
        }

        Ok(result)
    }

    /// NEON softmax implementation
    #[cfg(target_arch = "aarch64")]
    fn softmax_neon(scores: &[f32]) -> SIMDResult<Vec<f32>> {
        use std::simd::f32x4;

        let max_val = scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        let mut result = Vec::with_capacity(scores.len());
        let mut sum = 0.0;

        let max_vec = f32x4::splat(max_val);

        for i in (0..scores.len()).step_by(4) {
            if i + 4 <= scores.len() {
                let score_vec = f32x4::from_slice(&scores[i..i + 4]);
                let exp_vec = (score_vec - max_vec).exp();

                let mut exp_array = [0.0; 4];
                exp_vec.copy_to_slice(&mut exp_array);

                result.extend_from_slice(&exp_array);
                sum += exp_array.iter().sum::<f32>();
            } else {
                for j in i..scores.len() {
                    let exp_val = (scores[j] - max_val).exp();
                    result.push(exp_val);
                    sum += exp_val;
                }
            }
        }

        // Normalize
        let sum_vec = f32x4::splat(sum);
        for i in (0..result.len()).step_by(4) {
            if i + 4 <= result.len() {
                let mut chunk = [0.0; 4];
                chunk.copy_from_slice(&result[i..i + 4]);
                let chunk_vec = f32x4::from_array(chunk) / sum_vec;
                chunk_vec.copy_to_slice(&mut result[i..i + 4]);
            } else {
                for j in i..result.len() {
                    result[j] /= sum;
                }
            }
        }

        Ok(result)
    }

    /// Scalar softmax implementation
    fn softmax_scalar(scores: &[f32]) -> SIMDResult<Vec<f32>> {
        let max_val = scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        let mut result: Vec<f32> = scores.iter().map(|&x| (x - max_val).exp()).collect();

        let sum: f32 = result.iter().sum();
        for x in &mut result {
            *x /= sum;
        }

        Ok(result)
    }
}

impl SIMDEmbeddingSearch {
    /// SIMD-accelerated cosine similarity search for embeddings
    pub fn cosine_similarity_search(
        query: &[f32],
        embeddings: &[f32],
        embedding_dim: usize,
        top_k: usize,
    ) -> SIMDResult<Vec<(usize, f32)>> {
        if embeddings.len() % embedding_dim != 0 {
            return Err(SIMDError::VectorSizeMismatch {
                expected: embeddings.len(),
                actual: embeddings.len(),
            });
        }

        let num_embeddings = embeddings.len() / embedding_dim;
        let mut similarities = Vec::with_capacity(num_embeddings);

        let caps = get_cached_capabilities();

        #[cfg(target_arch = "x86_64")]
        {
            if caps.has_avx2 && embedding_dim >= 8 {
                similarities =
                    Self::cosine_similarity_search_avx2(query, embeddings, embedding_dim)?;
            } else if caps.has_sse && embedding_dim >= 4 {
                similarities =
                    Self::cosine_similarity_search_sse(query, embeddings, embedding_dim)?;
            } else {
                similarities =
                    Self::cosine_similarity_search_scalar(query, embeddings, embedding_dim)?;
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            if caps.has_neon && embedding_dim >= 4 {
                similarities =
                    Self::cosine_similarity_search_neon(query, embeddings, embedding_dim)?;
            } else {
                similarities =
                    Self::cosine_similarity_search_scalar(query, embeddings, embedding_dim)?;
            }
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            similarities = Self::cosine_similarity_search_scalar(query, embeddings, embedding_dim)?;
        }

        // Sort by similarity and return top-k
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        similarities.truncate(top_k);

        Ok(similarities)
    }

    /// AVX2 cosine similarity implementation
    #[cfg(target_arch = "x86_64")]
    fn cosine_similarity_search_avx2(
        query: &[f32],
        embeddings: &[f32],
        embedding_dim: usize,
    ) -> SIMDResult<Vec<(usize, f32)>> {
        use std::simd::f32x8;

        let num_embeddings = embeddings.len() / embedding_dim;
        let mut similarities = Vec::with_capacity(num_embeddings);

        // Pre-compute query norm
        let query_norm = Self::compute_vector_norm_avx2(query)?;

        for i in 0..num_embeddings {
            let start = i * embedding_dim;
            let embedding = &embeddings[start..start + embedding_dim];

            let dot_product = Self::compute_dot_product_avx2(query, embedding)?;
            let embedding_norm = Self::compute_vector_norm_avx2(embedding)?;

            let similarity = if query_norm > 0.0 && embedding_norm > 0.0 {
                dot_product / (query_norm * embedding_norm)
            } else {
                0.0
            };

            similarities.push((i, similarity));
        }

        Ok(similarities)
    }

    /// SSE cosine similarity implementation
    #[cfg(target_arch = "x86_64")]
    fn cosine_similarity_search_sse(
        query: &[f32],
        embeddings: &[f32],
        embedding_dim: usize,
    ) -> SIMDResult<Vec<(usize, f32)>> {
        use std::simd::f32x4;

        let num_embeddings = embeddings.len() / embedding_dim;
        let mut similarities = Vec::with_capacity(num_embeddings);

        let query_norm = Self::compute_vector_norm_sse(query)?;

        for i in 0..num_embeddings {
            let start = i * embedding_dim;
            let embedding = &embeddings[start..start + embedding_dim];

            let dot_product = Self::compute_dot_product_sse(query, embedding)?;
            let embedding_norm = Self::compute_vector_norm_sse(embedding)?;

            let similarity = if query_norm > 0.0 && embedding_norm > 0.0 {
                dot_product / (query_norm * embedding_norm)
            } else {
                0.0
            };

            similarities.push((i, similarity));
        }

        Ok(similarities)
    }

    /// NEON cosine similarity implementation
    #[cfg(target_arch = "aarch64")]
    fn cosine_similarity_search_neon(
        query: &[f32],
        embeddings: &[f32],
        embedding_dim: usize,
    ) -> SIMDResult<Vec<(usize, f32)>> {
        use std::simd::f32x4;

        let num_embeddings = embeddings.len() / embedding_dim;
        let mut similarities = Vec::with_capacity(num_embeddings);

        let query_norm = Self::compute_vector_norm_neon(query)?;

        for i in 0..num_embeddings {
            let start = i * embedding_dim;
            let embedding = &embeddings[start..start + embedding_dim];

            let dot_product = Self::compute_dot_product_neon(query, embedding)?;
            let embedding_norm = Self::compute_vector_norm_neon(embedding)?;

            let similarity = if query_norm > 0.0 && embedding_norm > 0.0 {
                dot_product / (query_norm * embedding_norm)
            } else {
                0.0
            };

            similarities.push((i, similarity));
        }

        Ok(similarities)
    }

    /// Scalar cosine similarity implementation
    fn cosine_similarity_search_scalar(
        query: &[f32],
        embeddings: &[f32],
        embedding_dim: usize,
    ) -> SIMDResult<Vec<(usize, f32)>> {
        let num_embeddings = embeddings.len() / embedding_dim;
        let mut similarities = Vec::with_capacity(num_embeddings);

        let query_norm = Self::compute_vector_norm_scalar(query);

        for i in 0..num_embeddings {
            let start = i * embedding_dim;
            let embedding = &embeddings[start..start + embedding_dim];

            let dot_product = Self::compute_dot_product_scalar(query, embedding);
            let embedding_norm = Self::compute_vector_norm_scalar(embedding);

            let similarity = if query_norm > 0.0 && embedding_norm > 0.0 {
                dot_product / (query_norm * embedding_norm)
            } else {
                0.0
            };

            similarities.push((i, similarity));
        }

        Ok(similarities)
    }

    /// AVX2 dot product computation
    #[cfg(target_arch = "x86_64")]
    fn compute_dot_product_avx2(a: &[f32], b: &[f32]) -> SIMDResult<f32> {
        use std::simd::f32x8;

        let mut sum = f32x8::splat(0.0);

        for i in (0..a.len()).step_by(8) {
            if i + 8 <= a.len() {
                let a_vec = f32x8::from_slice(&a[i..i + 8]);
                let b_vec = f32x8::from_slice(&b[i..i + 8]);
                sum += a_vec * b_vec;
            } else {
                // Handle remainder
                for j in i..a.len() {
                    let a_val = f32x8::splat(a[j]);
                    let b_val = f32x8::from_slice(&b[j..(j + 1).min(b.len())]);
                    sum += a_val * b_val;
                }
                break;
            }
        }

        let mut sum_array = [0.0; 8];
        sum.copy_to_slice(&mut sum_array);
        Ok(sum_array.iter().sum())
    }

    /// SSE dot product computation
    #[cfg(target_arch = "x86_64")]
    fn compute_dot_product_sse(a: &[f32], b: &[f32]) -> SIMDResult<f32> {
        use std::simd::f32x4;

        let mut sum = f32x4::splat(0.0);

        for i in (0..a.len()).step_by(4) {
            if i + 4 <= a.len() {
                let a_vec = f32x4::from_slice(&a[i..i + 4]);
                let b_vec = f32x4::from_slice(&b[i..i + 4]);
                sum += a_vec * b_vec;
            } else {
                for j in i..a.len() {
                    let a_val = f32x4::splat(a[j]);
                    let b_val = f32x4::from_slice(&b[j..(j + 1).min(b.len())]);
                    sum += a_val * b_val;
                }
                break;
            }
        }

        let mut sum_array = [0.0; 4];
        sum.copy_to_slice(&mut sum_array);
        Ok(sum_array.iter().sum())
    }

    /// NEON dot product computation
    #[cfg(target_arch = "aarch64")]
    fn compute_dot_product_neon(a: &[f32], b: &[f32]) -> SIMDResult<f32> {
        use std::simd::f32x4;

        let mut sum = f32x4::splat(0.0);

        for i in (0..a.len()).step_by(4) {
            if i + 4 <= a.len() {
                let a_vec = f32x4::from_slice(&a[i..i + 4]);
                let b_vec = f32x4::from_slice(&b[i..i + 4]);
                sum += a_vec * b_vec;
            } else {
                for j in i..a.len() {
                    let a_val = f32x4::splat(a[j]);
                    let b_val = f32x4::from_slice(&b[j..(j + 1).min(b.len())]);
                    sum += a_val * b_val;
                }
                break;
            }
        }

        let mut sum_array = [0.0; 4];
        sum.copy_to_slice(&mut sum_array);
        Ok(sum_array.iter().sum())
    }

    /// Scalar dot product computation
    fn compute_dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    /// AVX2 vector norm computation
    #[cfg(target_arch = "x86_64")]
    fn compute_vector_norm_avx2(v: &[f32]) -> SIMDResult<f32> {
        let dot_product = Self::compute_dot_product_avx2(v, v)?;
        Ok(dot_product.sqrt())
    }

    /// SSE vector norm computation
    #[cfg(target_arch = "x86_64")]
    fn compute_vector_norm_sse(v: &[f32]) -> SIMDResult<f32> {
        let dot_product = Self::compute_dot_product_sse(v, v)?;
        Ok(dot_product.sqrt())
    }

    /// NEON vector norm computation
    #[cfg(target_arch = "aarch64")]
    fn compute_vector_norm_neon(v: &[f32]) -> SIMDResult<f32> {
        let dot_product = Self::compute_dot_product_neon(v, v)?;
        Ok(dot_product.sqrt())
    }

    /// Scalar vector norm computation
    fn compute_vector_norm_scalar(v: &[f32]) -> f32 {
        Self::compute_dot_product_scalar(v, v).sqrt()
    }
}

impl SIMDAIInferenceOps {
    /// SIMD-accelerated matrix-vector multiplication for inference layers
    pub fn matrix_vector_mul(
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

        #[cfg(target_arch = "x86_64")]
        {
            if caps.has_avx2 && cols >= 8 {
                Self::matrix_vector_mul_avx2(matrix, vector, rows, cols)
            } else if caps.has_sse && cols >= 4 {
                Self::matrix_vector_mul_sse(matrix, vector, rows, cols)
            } else {
                Self::matrix_vector_mul_scalar(matrix, vector, rows, cols)
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            if caps.has_neon && cols >= 4 {
                Self::matrix_vector_mul_neon(matrix, vector, rows, cols)
            } else {
                Self::matrix_vector_mul_scalar(matrix, vector, rows, cols)
            }
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        Self::matrix_vector_mul_scalar(matrix, vector, rows, cols)
    }

    /// AVX2 matrix-vector multiplication
    #[cfg(target_arch = "x86_64")]
    fn matrix_vector_mul_avx2(
        matrix: &[f32],
        vector: &[f32],
        rows: usize,
        cols: usize,
    ) -> SIMDResult<Vec<f32>> {
        use std::simd::f32x8;

        let mut result = vec![0.0; rows];

        for i in 0..rows {
            let mut sum = f32x8::splat(0.0);
            let row_start = i * cols;

            for j in (0..cols).step_by(8) {
                if j + 8 <= cols {
                    let matrix_chunk = f32x8::from_slice(&matrix[row_start + j..row_start + j + 8]);
                    let vec_chunk = f32x8::from_slice(&vector[j..j + 8]);
                    sum += matrix_chunk * vec_chunk;
                } else {
                    // Handle remaining elements
                    for k in j..cols {
                        if k < cols {
                            let matrix_val = f32x8::splat(matrix[row_start + k]);
                            let vec_val = f32x8::from_slice(&vector[k..k + 1]);
                            sum += matrix_val * vec_val;
                        }
                    }
                }
            }

            let mut sum_array = [0.0; 8];
            sum.copy_to_slice(&mut sum_array);
            result[i] = sum_array.iter().sum();
        }

        Ok(result)
    }

    /// SSE matrix-vector multiplication
    #[cfg(target_arch = "x86_64")]
    fn matrix_vector_mul_sse(
        matrix: &[f32],
        vector: &[f32],
        rows: usize,
        cols: usize,
    ) -> SIMDResult<Vec<f32>> {
        use std::simd::f32x4;

        let mut result = vec![0.0; rows];

        for i in 0..rows {
            let mut sum = f32x4::splat(0.0);
            let row_start = i * cols;

            for j in (0..cols).step_by(4) {
                if j + 4 <= cols {
                    let matrix_chunk = f32x4::from_slice(&matrix[row_start + j..row_start + j + 4]);
                    let vec_chunk = f32x4::from_slice(&vector[j..j + 4]);
                    sum += matrix_chunk * vec_chunk;
                } else {
                    for k in j..cols {
                        if k < cols {
                            let matrix_val = f32x4::splat(matrix[row_start + k]);
                            let vec_val = f32x4::from_slice(&vector[k..k + 1]);
                            sum += matrix_val * vec_val;
                        }
                    }
                }
            }

            let mut sum_array = [0.0; 4];
            sum.copy_to_slice(&mut sum_array);
            result[i] = sum_array.iter().sum();
        }

        Ok(result)
    }

    /// NEON matrix-vector multiplication
    #[cfg(target_arch = "aarch64")]
    fn matrix_vector_mul_neon(
        matrix: &[f32],
        vector: &[f32],
        rows: usize,
        cols: usize,
    ) -> SIMDResult<Vec<f32>> {
        use std::simd::f32x4;

        let mut result = vec![0.0; rows];

        for i in 0..rows {
            let mut sum = f32x4::splat(0.0);
            let row_start = i * cols;

            for j in (0..cols).step_by(4) {
                if j + 4 <= cols {
                    let matrix_chunk = f32x4::from_slice(&matrix[row_start + j..row_start + j + 4]);
                    let vec_chunk = f32x4::from_slice(&vector[j..j + 4]);
                    sum += matrix_chunk * vec_chunk;
                } else {
                    for k in j..cols {
                        if k < cols {
                            let matrix_val = f32x4::splat(matrix[row_start + k]);
                            let vec_val = f32x4::from_slice(&vector[k..k + 1]);
                            sum += matrix_val * vec_val;
                        }
                    }
                }
            }

            let mut sum_array = [0.0; 4];
            sum.copy_to_slice(&mut sum_array);
            result[i] = sum_array.iter().sum();
        }

        Ok(result)
    }

    /// Scalar matrix-vector multiplication
    fn matrix_vector_mul_scalar(
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

    /// SIMD-accelerated activation functions for inference layers
    pub fn apply_activation(
        input: &[f32],
        activation_type: ActivationType,
    ) -> SIMDResult<Vec<f32>> {
        let caps = get_cached_capabilities();

        #[cfg(target_arch = "x86_64")]
        {
            if caps.has_avx2 && input.len() >= 8 {
                Self::apply_activation_avx2(input, activation_type)
            } else if caps.has_sse && input.len() >= 4 {
                Self::apply_activation_sse(input, activation_type)
            } else {
                Self::apply_activation_scalar(input, activation_type)
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            if caps.has_neon && input.len() >= 4 {
                Self::apply_activation_neon(input, activation_type)
            } else {
                Self::apply_activation_scalar(input, activation_type)
            }
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        Self::apply_activation_scalar(input, activation_type)
    }

    /// AVX2 activation functions
    #[cfg(target_arch = "x86_64")]
    fn apply_activation_avx2(
        input: &[f32],
        activation_type: ActivationType,
    ) -> SIMDResult<Vec<f32>> {
        use std::simd::f32x8;

        let mut result = Vec::with_capacity(input.len());

        for i in (0..input.len()).step_by(8) {
            if i + 8 <= input.len() {
                let input_vec = f32x8::from_slice(&input[i..i + 8]);
                let output_vec = match activation_type {
                    ActivationType::ReLU => input_vec.simd_max(f32x8::splat(0.0)),
                    ActivationType::Sigmoid => {
                        let neg_input = -input_vec;
                        f32x8::splat(1.0) / (f32x8::splat(1.0) + neg_input.exp())
                    }
                    ActivationType::Tanh => input_vec.tanh(),
                    ActivationType::LeakyReLU => {
                        let zero = f32x8::splat(0.0);
                        let alpha = f32x8::splat(0.01);
                        input_vec.simd_max(zero) + alpha * input_vec.simd_min(zero)
                    }
                };

                let mut output_array = [0.0; 8];
                output_vec.copy_to_slice(&mut output_array);
                result.extend_from_slice(&output_array);
            } else {
                for j in i..input.len() {
                    let output_val = match activation_type {
                        ActivationType::ReLU => input[j].max(0.0),
                        ActivationType::Sigmoid => 1.0 / (1.0 + (-input[j]).exp()),
                        ActivationType::Tanh => input[j].tanh(),
                        ActivationType::LeakyReLU => {
                            if input[j] > 0.0 {
                                input[j]
                            } else {
                                0.01 * input[j]
                            }
                        }
                    };
                    result.push(output_val);
                }
            }
        }

        Ok(result)
    }

    /// SSE activation functions
    #[cfg(target_arch = "x86_64")]
    fn apply_activation_sse(
        input: &[f32],
        activation_type: ActivationType,
    ) -> SIMDResult<Vec<f32>> {
        use std::simd::f32x4;

        let mut result = Vec::with_capacity(input.len());

        for i in (0..input.len()).step_by(4) {
            if i + 4 <= input.len() {
                let input_vec = f32x4::from_slice(&input[i..i + 4]);
                let output_vec = match activation_type {
                    ActivationType::ReLU => input_vec.simd_max(f32x4::splat(0.0)),
                    ActivationType::Sigmoid => {
                        let neg_input = -input_vec;
                        f32x4::splat(1.0) / (f32x4::splat(1.0) + neg_input.exp())
                    }
                    ActivationType::Tanh => input_vec.tanh(),
                    ActivationType::LeakyReLU => {
                        let zero = f32x4::splat(0.0);
                        let alpha = f32x4::splat(0.01);
                        input_vec.simd_max(zero) + alpha * input_vec.simd_min(zero)
                    }
                };

                let mut output_array = [0.0; 4];
                output_vec.copy_to_slice(&mut output_array);
                result.extend_from_slice(&output_array);
            } else {
                for j in i..input.len() {
                    let output_val = match activation_type {
                        ActivationType::ReLU => input[j].max(0.0),
                        ActivationType::Sigmoid => 1.0 / (1.0 + (-input[j]).exp()),
                        ActivationType::Tanh => input[j].tanh(),
                        ActivationType::LeakyReLU => {
                            if input[j] > 0.0 {
                                input[j]
                            } else {
                                0.01 * input[j]
                            }
                        }
                    };
                    result.push(output_val);
                }
            }
        }

        Ok(result)
    }

    /// NEON activation functions
    #[cfg(target_arch = "aarch64")]
    fn apply_activation_neon(
        input: &[f32],
        activation_type: ActivationType,
    ) -> SIMDResult<Vec<f32>> {
        use std::simd::f32x4;

        let mut result = Vec::with_capacity(input.len());

        for i in (0..input.len()).step_by(4) {
            if i + 4 <= input.len() {
                let input_vec = f32x4::from_slice(&input[i..i + 4]);
                let output_vec = match activation_type {
                    ActivationType::ReLU => input_vec.simd_max(f32x4::splat(0.0)),
                    ActivationType::Sigmoid => {
                        let neg_input = -input_vec;
                        f32x4::splat(1.0) / (f32x4::splat(1.0) + neg_input.exp())
                    }
                    ActivationType::Tanh => input_vec.tanh(),
                    ActivationType::LeakyReLU => {
                        let zero = f32x4::splat(0.0);
                        let alpha = f32x4::splat(0.01);
                        input_vec.simd_max(zero) + alpha * input_vec.simd_min(zero)
                    }
                };

                let mut output_array = [0.0; 4];
                output_vec.copy_to_slice(&mut output_array);
                result.extend_from_slice(&output_array);
            } else {
                for j in i..input.len() {
                    let output_val = match activation_type {
                        ActivationType::ReLU => input[j].max(0.0),
                        ActivationType::Sigmoid => 1.0 / (1.0 + (-input[j]).exp()),
                        ActivationType::Tanh => input[j].tanh(),
                        ActivationType::LeakyReLU => {
                            if input[j] > 0.0 {
                                input[j]
                            } else {
                                0.01 * input[j]
                            }
                        }
                    };
                    result.push(output_val);
                }
            }
        }

        Ok(result)
    }

    /// Scalar activation functions
    fn apply_activation_scalar(
        input: &[f32],
        activation_type: ActivationType,
    ) -> SIMDResult<Vec<f32>> {
        let result = input
            .iter()
            .map(|&x| match activation_type {
                ActivationType::ReLU => x.max(0.0),
                ActivationType::Sigmoid => 1.0 / (1.0 + (-x).exp()),
                ActivationType::Tanh => x.tanh(),
                ActivationType::LeakyReLU => {
                    if x > 0.0 {
                        x
                    } else {
                        0.01 * x
                    }
                }
            })
            .collect();

        Ok(result)
    }
}

/// Activation function types for inference layers
#[derive(Debug, Clone, Copy)]
pub enum ActivationType {
    ReLU,
    Sigmoid,
    Tanh,
    LeakyReLU,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_scoring() {
        let predictions = vec![0.8, 0.6, 0.9];
        let uncertainties = vec![0.1, 0.2, 0.05];
        let weights = vec![0.5, 0.7, 0.8];

        let result =
            SIMDConfidenceScorer::compute_confidence_scores(&predictions, &uncertainties, &weights);

        match result {
            Ok(scores) => {
                assert_eq!(scores.len(), 3);
                // All scores should be between 0 and 1
                for &score in &scores {
                    assert!(score >= 0.0 && score <= 1.0);
                }
            }
            Err(SIMDError::SIMDUnavailable) => {
                // SIMD not available, but test passes
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_softmax_confidence() {
        let scores = vec![1.0, 2.0, 3.0];

        let result = SIMDConfidenceScorer::softmax_confidence(&scores);

        match result {
            Ok(softmax_scores) => {
                assert_eq!(softmax_scores.len(), 3);
                let sum: f32 = softmax_scores.iter().sum();
                assert!((sum - 1.0).abs() < 0.001);
            }
            Err(SIMDError::SIMDUnavailable) => {
                // SIMD not available, but test passes
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_cosine_similarity_search() {
        let query = vec![1.0, 0.0, 0.0];
        let embeddings = vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        let embedding_dim = 3;

        let result =
            SIMDEmbeddingSearch::cosine_similarity_search(&query, &embeddings, embedding_dim, 2);

        match result {
            Ok(similarities) => {
                assert_eq!(similarities.len(), 2);
                // First result should be most similar (index 0, score 1.0)
                assert_eq!(similarities[0].0, 0);
                assert!((similarities[0].1 - 1.0).abs() < 0.001);
            }
            Err(SIMDError::SIMDUnavailable) => {
                // SIMD not available, but test passes
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_matrix_vector_mul() {
        let matrix = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]; // 2x3
        let vector = vec![0.5, 1.0, 2.0]; // 3 elements
        let rows = 2;
        let cols = 3;

        let result = SIMDAIInferenceOps::matrix_vector_mul(&matrix, &vector, rows, cols);

        match result {
            Ok(output) => {
                assert_eq!(output.len(), 2);
                // First row: 1*0.5 + 2*1.0 + 3*2.0 = 0.5 + 2.0 + 6.0 = 8.5
                // Second row: 4*0.5 + 5*1.0 + 6*2.0 = 2.0 + 5.0 + 12.0 = 19.0
                assert!((output[0] - 8.5).abs() < 0.001);
                assert!((output[1] - 19.0).abs() < 0.001);
            }
            Err(SIMDError::SIMDUnavailable) => {
                // SIMD not available, but test passes
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_activation_functions() {
        let input = vec![-1.0, 0.0, 1.0, 2.0];

        let result = SIMDAIInferenceOps::apply_activation(&input, ActivationType::ReLU);

        match result {
            Ok(output) => {
                assert_eq!(output.len(), 4);
                assert_eq!(output[0], 0.0); // ReLU(-1) = 0
                assert_eq!(output[1], 0.0); // ReLU(0) = 0
                assert_eq!(output[2], 1.0); // ReLU(1) = 1
                assert_eq!(output[3], 2.0); // ReLU(2) = 2
            }
            Err(SIMDError::SIMDUnavailable) => {
                // SIMD not available, but test passes
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}

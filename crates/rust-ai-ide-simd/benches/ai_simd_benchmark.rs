//! Comprehensive benchmarks for AI/ML SIMD operations
//! Tests performance improvements for confidence scoring, similarity search, and vector operations

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_ai_ide_simd::ai_operations::{SIMDConfidenceScorer, SIMDEmbeddingSearch, SIMDAIInferenceOps, ActivationType};

// Benchmark for confidence scoring operations
fn bench_confidence_scoring(c: &mut Criterion) {
    let predictions = vec![0.8, 0.6, 0.9, 0.7, 0.85, 0.75, 0.65, 0.95, 0.55, 0.88];
    let uncertainties = vec![0.1, 0.2, 0.05, 0.15, 0.08, 0.12, 0.18, 0.03, 0.22, 0.07];
    let weights = vec![0.5, 0.7, 0.8, 0.6, 0.9, 0.4, 0.75, 0.85, 0.65, 0.55];

    c.bench_function("confidence_scoring_simd", |b| {
        b.iter(|| {
            let _result = SIMDConfidenceScorer::compute_confidence_scores(
                black_box(&predictions),
                black_box(&uncertainties),
                black_box(&weights),
            );
        });
    });
}

// Benchmark for softmax operations
fn bench_softmax(c: &mut Criterion) {
    let scores = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    c.bench_function("softmax_simd", |b| {
        b.iter(|| {
            let _result = SIMDConfidenceScorer::softmax_confidence(black_box(&scores));
        });
    });
}

// Benchmark for cosine similarity search
fn bench_cosine_similarity(c: &mut Criterion) {
    // Create test embeddings (10 embeddings of dimension 128)
    let mut embeddings = Vec::new();
    for i in 0..10 {
        for j in 0..128 {
            embeddings.push((i as f32 * 0.1 + j as f32 * 0.01).sin());
        }
    }

    let query = vec![0.5; 128];

    c.bench_function("cosine_similarity_search_simd", |b| {
        b.iter(|| {
            let _result = SIMDEmbeddingSearch::cosine_similarity_search(
                black_box(&query),
                black_box(&embeddings),
                128,
                5,
            );
        });
    });
}

// Benchmark for matrix-vector multiplication
fn bench_matrix_vector_mul(c: &mut Criterion) {
    // 64x128 matrix
    let matrix: Vec<f32> = (0..8192).map(|i| (i as f32 * 0.001).sin()).collect();
    let vector = vec![0.1; 128];
    let rows = 64;
    let cols = 128;

    c.bench_function("matrix_vector_mul_simd", |b| {
        b.iter(|| {
            let _result = SIMDAIInferenceOps::matrix_vector_mul(
                black_box(&matrix),
                black_box(&vector),
                rows,
                cols,
            );
        });
    });
}

// Benchmark for activation functions
fn bench_activation_functions(c: &mut Criterion) {
    let input: Vec<f32> = (0..256).map(|i| (i as f32 * 0.1 - 12.8)).collect();

    let mut group = c.benchmark_group("activation_functions");

    group.bench_function("relu_simd", |b| {
        b.iter(|| {
            let _result = SIMDAIInferenceOps::apply_activation(
                black_box(&input),
                ActivationType::ReLU,
            );
        });
    });

    group.bench_function("sigmoid_simd", |b| {
        b.iter(|| {
            let _result = SIMDAIInferenceOps::apply_activation(
                black_box(&input),
                ActivationType::Sigmoid,
            );
        });
    });

    group.bench_function("tanh_simd", |b| {
        b.iter(|| {
            let _result = SIMDAIInferenceOps::apply_activation(
                black_box(&input),
                ActivationType::Tanh,
            );
        });
    });

    group.bench_function("leaky_relu_simd", |b| {
        b.iter(|| {
            let _result = SIMDAIInferenceOps::apply_activation(
                black_box(&input),
                ActivationType::LeakyReLU,
            );
        });
    });

    group.finish();
}

// Benchmark for large-scale operations (stress test)
fn bench_large_scale_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_scale");

    // Large confidence scoring benchmark (1000 elements)
    let large_predictions: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.001).sin()).collect();
    let large_uncertainties: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.002).cos().abs()).collect();
    let large_weights: Vec<f32> = vec![1.0; 1000];

    group.bench_function("large_confidence_scoring", |b| {
        b.iter(|| {
            let _result = SIMDConfidenceScorer::compute_confidence_scores(
                black_box(&large_predictions),
                black_box(&large_uncertainties),
                black_box(&large_weights),
            );
        });
    });

    // Large embedding similarity benchmark
    let mut large_embeddings = Vec::new();
    for i in 0..100 {
        for j in 0..512 {
            large_embeddings.push((i as f32 * 0.01 + j as f32 * 0.001).sin());
        }
    }
    let large_query = vec![0.5; 512];

    group.bench_function("large_embedding_search", |b| {
        b.iter(|| {
            let _result = SIMDEmbeddingSearch::cosine_similarity_search(
                black_box(&large_query),
                black_box(&large_embeddings),
                512,
                10,
            );
        });
    });

    group.finish();
}

// Performance comparison benchmarks
fn bench_performance_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_comparison");

    // Test data
    let predictions: Vec<f32> = (0..100).map(|i| (i as f32 * 0.01).sin()).collect();
    let uncertainties: Vec<f32> = (0..100).map(|i| (i as f32 * 0.02).cos().abs()).collect();
    let weights: Vec<f32> = vec![1.0; 100];

    group.bench_function("confidence_scoring_scalar_baseline", |b| {
        b.iter(|| {
            let _result: Vec<f32> = predictions.iter()
                .zip(uncertainties.iter())
                .zip(weights.iter())
                .map(|((pred, uncert), weight)| {
                    let confidence = pred * (1.0 - uncert) * weight;
                    confidence.max(0.0).min(1.0)
                })
                .collect();
        });
    });

    group.bench_function("confidence_scoring_simd", |b| {
        b.iter(|| {
            let _result = SIMDConfidenceScorer::compute_confidence_scores(
                black_box(&predictions),
                black_box(&uncertainties),
                black_box(&weights),
            );
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_confidence_scoring,
    bench_softmax,
    bench_cosine_similarity,
    bench_matrix_vector_mul,
    bench_activation_functions,
    bench_large_scale_operations,
    bench_performance_comparison
);
criterion_main!(benches);
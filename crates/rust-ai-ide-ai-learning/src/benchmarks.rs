//! Performance Benchmarks for Modular Learning System
//!
//! This module provides comprehensive benchmarking for the learning system
//! to ensure performance requirements are met and regressions are caught.

use std::hint::black_box;
use std::path::Path;
use std::time::{Duration, Instant};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::Rng;
use tempfile::TempDir;
use tokio::runtime::Runtime;

use super::database::LearningDatabase;
use super::models::{
    ChangeScope, ChangeTemplate, ChangeType, FixTemplate, LearnedPattern, LearningPreferences, PatternSimilarity,
};
use super::similarity::SimilarityCalculator;
use super::system::LearningSystem;
use super::types::{AIResult, PrivacyMode};

/// Configuration for benchmark datasets
struct BenchmarkConfig {
    small_dataset_size:        usize,
    medium_dataset_size:       usize,
    large_dataset_size:        usize,
    similarity_search_queries: Vec<String>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            small_dataset_size:        100,
            medium_dataset_size:       1_000,
            large_dataset_size:        10_000,
            similarity_search_queries: vec![
                "borrow checker error mutable".to_string(),
                "unused variable compilation".to_string(),
                "trait implementation missing".to_string(),
                "type mismatch expected found".to_string(),
                "ownership moved value".to_string(),
                "lifetime validity check".to_string(),
                "pattern matching exhaustive".to_string(),
                "async function await missing".to_string(),
            ],
        }
    }
}

/// Benchmark group for LearningSystem operations
fn learning_system_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("learning_system");
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();

    // Small dataset benchmarks
    group.bench_with_input(
        BenchmarkId::new("store_small_dataset", config.small_dataset_size),
        &config.small_dataset_size,
        |b, &size| {
            b.to_async(&rt).iter(|| async {
                let (system, temp_dir) = setup_benchmark_system().await;
                let patterns = generate_test_patterns(size, temp_dir.path());

                for pattern in patterns {
                    black_box(system.store_pattern(&pattern).await.unwrap());
                }
            });
        },
    );

    // Medium dataset benchmarks
    group.bench_with_input(
        BenchmarkId::new("store_medium_dataset", config.medium_dataset_size),
        &config.medium_dataset_size,
        |b, &size| {
            b.to_async(&rt).iter(|| async {
                let (system, temp_dir) = setup_benchmark_system().await;
                let patterns = generate_test_patterns(size, temp_dir.path());

                for pattern in patterns {
                    black_box(system.store_pattern(&pattern).await.unwrap());
                }
            });
        },
    );

    // Similarity search benchmarks
    for query in &config.similarity_search_queries {
        group.bench_with_input(
            BenchmarkId::new("similarity_search", query),
            query,
            |b, query| {
                b.to_async(&rt).iter_batched(
                    || {
                        rt.block_on(async {
                            let (system, _) = setup_benchmark_system_with_data().await;
                            (system, query.to_string())
                        })
                    },
                    |(system, query)| async move {
                        let results = system.find_similar_patterns(&query).await.unwrap();
                        black_box(results);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

/// Benchmark group for database operations
fn database_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("learning_database");
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Database CRUD operations
    group.bench_function("database_store_single", |b| {
        b.to_async(&rt).iter_batched(
            || {
                rt.block_on(async {
                    let (system, _) = setup_benchmark_system().await;
                    let pattern = generate_test_pattern(0, std::path::Path::new("."));
                    (system, pattern)
                })
            },
            |(system, pattern)| async move {
                system.store_pattern(&pattern).await.unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("database_retrieve_single", |b| {
        b.to_async(&rt).iter_batched(
            || {
                rt.block_on(async {
                    let (system, _) = setup_benchmark_system_with_data().await;
                    let patterns = system.get_all_patterns_with_limit(1).await.unwrap();
                    let pattern_id = patterns.first().unwrap().id.clone();
                    (system, pattern_id)
                })
            },
            |(system, id)| async move {
                let _ = system.get_pattern(&id).await.unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("database_bulk_retrieve", |b| {
        b.to_async(&rt).iter_batched(
            || {
                rt.block_on(async {
                    let (system, _) = setup_benchmark_system_with_data().await;
                    system
                })
            },
            |system| async move {
                let results = system.get_all_patterns_with_limit(100).await.unwrap();
                black_box(results);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark group for similarity calculations
fn similarity_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("similarity_algorithm");
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Text similarity benchmark
    group.bench_function("text_similarity_calculation", |b| {
        b.to_async(&rt).iter_batched(
            || {
                (
                    "unused variable x in function".to_string(),
                    "unused variable y in function".to_string(),
                )
            },
            |(text1, text2)| async move {
                let similarity = SimilarityCalculator::calculate_text_similarity(&text1, &text2);
                black_box(similarity);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Structural similarity benchmark
    group.bench_function("structural_similarity_calculation", |b| {
        b.to_async(&rt).iter_batched(
            || {
                let context = "fn test() { let x = 1; }";
                let patterns = vec![
                    "fn function() { let x = 1; }".to_string(),
                    "fn other() { let y = 2; }".to_string(),
                ];
                (context.to_string(), patterns)
            },
            |(context, patterns)| async move {
                let similarity = SimilarityCalculator::calculate_structure_similarity(&context, &patterns);
                black_box(similarity);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Full system similarity search
    group.bench_function("system_similarity_search", |b| {
        b.to_async(&rt).iter_batched(
            || {
                rt.block_on(async {
                    let (system, _) = setup_benchmark_system_with_data().await;
                    let query = "unused variable".to_string();
                    (system, query)
                })
            },
            |(system, query)| async move {
                let _ = system.find_similar_patterns(&query).await.unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark group for memory and caching performance
fn memory_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_performance");
    let rt = Runtime::new().unwrap();

    // Benchmark cache hit performance
    group.bench_function("cache_hit_performance", |b| {
        b.to_async(&rt).iter_batched(
            || {
                rt.block_on(async {
                    let (system, _) = setup_benchmark_system_with_data().await;
                    // Prime the cache with initial queries
                    let _ = system.find_similar_patterns("borrow checker").await;
                    ("borrow checker error mutable".to_string(), system)
                })
            },
            |(query, system)| async move {
                // This should be a cache hit
                let results = system.find_similar_patterns(&query).await.unwrap();
                black_box(results);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Benchmark cache miss performance
    group.bench_function("cache_miss_performance", |b| {
        b.to_async(&rt).iter_batched(
            || {
                rt.block_on(async {
                    let (system, _) = setup_benchmark_system_with_data().await;
                    system.invalidate_cache("*"); // Clear cache
                    system
                })
            },
            |system| async move {
                let query = format!("unique_query_{}", rand::random::<u32>());
                let results = system.find_similar_patterns(&query).await.unwrap();
                black_box(results);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Benchmark memory usage with growing dataset
    group.bench_function("memory_usage_growth", |b| {
        b.to_async(&rt).iter_batched(
            || {
                rt.block_on(async {
                    let (system, _) = setup_benchmark_system().await;
                    system
                })
            },
            |system| async move {
                let pattern_count = 1000;
                let test_patterns = generate_test_patterns(pattern_count, std::path::Path::new("."));

                for (i, pattern) in test_patterns.into_iter().enumerate() {
                    system.store_pattern(&pattern).await.unwrap();

                    // Periodic similarity searches to build cache
                    if i % 100 == 0 {
                        let query = format!("test context {}", i);
                        let results = system.find_similar_patterns(&query).await.unwrap();
                        black_box(results);
                    }
                }

                black_box(system);
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.finish();
}

/// Benchmark group for concurrent operations
fn concurrent_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    let rt = Runtime::new().unwrap();

    // Concurrent reads
    group.bench_function("concurrent_reads", |b| {
        b.to_async(&rt).iter_batched(
            || {
                rt.block_on(async {
                    let (system, _) = setup_benchmark_system_with_data().await;
                    let system = std::sync::Arc::new(system);
                    let mut handles = Vec::new();

                    for _ in 0..10 {
                        let system_clone = std::sync::Arc::clone(&system);
                        let handle = tokio::spawn(async move {
                            let results = system_clone
                                .find_similar_patterns("borrow checker")
                                .await
                                .unwrap();
                            black_box(results);
                        });
                        handles.push(handle);
                    }

                    (system, handles)
                })
            },
            |(system, handles)| async move {
                for handle in handles {
                    handle.await.unwrap();
                }
                black_box(system);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Concurrent writes
    group.bench_function("concurrent_writes", |b| {
        b.to_async(&rt).iter_batched(
            || {
                rt.block_on(async {
                    let (system, _) = setup_benchmark_system().await;
                    let system = std::sync::Arc::new(system);
                    let mut pattern_batches = Vec::new();

                    for i in 0..10 {
                        let start_idx = i * 100;
                        let patterns = generate_test_patterns_with_offset(100, start_idx);
                        pattern_batches.push(patterns);
                    }

                    (system, pattern_batches)
                })
            },
            |(system, pattern_batches)| async move {
                let mut handles = Vec::new();

                for patterns in pattern_batches {
                    let system_clone = std::sync::Arc::clone(&system);
                    let handle = tokio::spawn(async move {
                        for pattern in patterns {
                            system_clone.store_pattern(&pattern).await.unwrap();
                            black_box(&pattern);
                        }
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    handle.await.unwrap();
                }

                black_box(system);
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.finish();
}

/// Helper functions for benchmark setup
async fn setup_benchmark_system() -> (LearningSystem, TempDir) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_learning.db");

    // Create a learning system with test database
    let system = LearningSystem::new_with_path(Some(db_path))
        .await
        .expect("Failed to create learning system");

    (system, temp_dir)
}

async fn setup_benchmark_system_with_data() -> (LearningSystem, TempDir) {
    let (system, temp_dir) = setup_benchmark_system().await;

    // Configure for benchmarking
    let bench_prefs = LearningPreferences {
        enable_learning:          true,
        privacy_mode:             PrivacyMode::OptIn,
        confidence_threshold:     0.5, // Lower threshold for benchmarks
        max_patterns_per_type:    1000,
        enable_community_sharing: false,
        use_community_patterns:   false,
        auto_apply_threshold:     0.8,
    };

    system
        .update_preferences(bench_prefs)
        .await
        .expect("Failed to configure benchmark preferences");

    // Pre-populate with test data
    let patterns = generate_test_patterns(500, Path::new(""));
    for pattern in patterns {
        system
            .store_pattern(&pattern)
            .await
            .expect("Failed to populate benchmark data");
    }

    (system, temp_dir)
}

fn generate_test_patterns(count: usize, base_path: &Path) -> Vec<LearnedPattern> {
    (0..count)
        .map(|i| generate_test_pattern(i, base_path))
        .collect()
}

fn generate_test_patterns_with_offset(count: usize, offset: usize) -> Vec<LearnedPattern> {
    (0..count)
        .map(|i| generate_test_pattern(i + offset, Path::new("")))
        .collect()
}

fn generate_test_pattern(index: usize, base_path: &std::path::Path) -> LearnedPattern {
    use chrono::Utc;

    let error_patterns = [
        "borrow checker error mutable",
        "unused variable compilation",
        "trait implementation missing",
        "type mismatch expected found",
        "ownership moved value",
        "lifetime validity check",
        "compilation error syntax",
        "pattern matching exhaustive",
        "async function await missing",
    ];

    let contexts = [
        "fn test_function_{}() {{\n    let var_{} = {};\n}}",
        "impl Trait for {} {{\n    fn method_{}(&self) -> {} {{\n        todo!()\n    }}\n}}",
        "struct Test{} {{\n    field_{}: {},\n}}",
        "enum Variant{} {{\n    Case{}(String),\n}}",
    ];

    let index = index % error_patterns.len();
    let error_pattern = error_patterns[index];
    let context_template = contexts[index % contexts.len()];

    LearnedPattern {
        id:               format!("bench_pattern_{}", index),
        description:      format!("Benchmark pattern {}", index),
        error_pattern:    error_pattern.to_string(),
        error_code:       Some(format!("E0{:03}", (index % 1000))),
        context_patterns: vec![
            format!("fn benchmark_{}() {{", index),
            format!("{}", error_pattern),
            format!("let var_{} = {};", index, index * 2),
        ],
        fix_template:     FixTemplate {
            description_template: format!("Fix for benchmark {}", index),
            change_templates:     vec![ChangeTemplate {
                match_pattern:       format!("let var_{} = {};", index, index * 2),
                replacement_pattern: format!("let _var_{} = {};", index, index * 2),
                change_type:         ChangeType::Replace,
                scope:               ChangeScope::Local,
            }],
            variables:            std::collections::HashMap::new(),
            conditions:           vec![],
            warnings:             vec![],
        },
        confidence:       0.6 + (index % 40) as f32 / 100.0, // 0.6 - 1.0 range
        success_count:    (index % 20 + 1) as u32,
        attempt_count:    (index % 30 + 2) as u32,
        created_at:       Utc::now(),
        updated_at:       Utc::now(),
        context_hash:     format!("bench_hash_{}", index),
        tags:             vec![
            error_pattern.split(' ').next().unwrap().to_string(),
            format!("group_{}", index / 10),
        ],
        contributor_id:   None,
    }
}

// Manual extension trait for benchmark-specific methods
#[async_trait::async_trait]
pub trait LearningSystemBenchExt: Send + Sync {
    async fn store_pattern(&self, pattern: &LearnedPattern) -> AIResult<()>;
    async fn get_pattern(&self, id: &str) -> AIResult<LearnedPattern>;
    async fn get_all_patterns_with_limit(&self, limit: usize) -> AIResult<Vec<LearnedPattern>>;
    async fn find_similar_patterns(&self, query: &str) -> AIResult<Vec<PatternSimilarity>>;
}

#[async_trait::async_trait]
impl LearningSystemBenchExt for LearningSystem {
    async fn store_pattern(&self, pattern: &LearnedPattern) -> AIResult<()> {
        // Store the pattern using the database
        self.database.store_pattern(pattern).await
    }

    async fn get_pattern(&self, id: &str) -> AIResult<LearnedPattern> {
        // Get a pattern by ID from the database
        self.database
            .get_pattern_by_id(id)
            .await?
            .ok_or_else(|| super::types::LearningError::PatternNotFoundError(id.to_string()).into())
    }

    async fn get_all_patterns_with_limit(&self, limit: usize) -> AIResult<Vec<LearnedPattern>> {
        // Get all patterns and limit the results
        let patterns = self.database.get_all_patterns().await?;
        Ok(patterns.into_iter().take(limit).collect())
    }

    async fn find_similar_patterns(&self, query: &str) -> AIResult<Vec<PatternSimilarity>> {
        // Use the system's built-in similarity search
        self.find_similar_patterns(query).await
    }
}

// ... rest of the code remains the same ...
criterion_group!(
    benches,
    learning_system_benchmarks,
    database_benchmarks,
    similarity_benchmarks,
    memory_benchmarks,
    concurrent_benchmarks
);

criterion_main!(benches);

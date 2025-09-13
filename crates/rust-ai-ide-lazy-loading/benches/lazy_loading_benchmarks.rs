//! Performance benchmarks for lazy loading infrastructure

use std::sync::Arc;
use std::time::{Duration, Instant};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_ai_ide_lazy_loading::*;
use tokio::runtime::Runtime;

/// Benchmark lazy loading performance
fn bench_lazy_loading_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("lazy_component_registration", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = LazyLoadingConfig::default();
                let loader = LazyLoader::new(config);

                let component = SimpleLazyComponent::new("bench_component", || async {
                    // Simulate component initialization
                    tokio::time::sleep(Duration::from_millis(1)).await;
                    Ok(Arc::new(42) as Arc<dyn std::any::Any + Send + Sync>)
                });

                black_box(
                    loader
                        .register_component(Box::new(component))
                        .await
                        .unwrap(),
                );
            });
        });
    });

    c.bench_function("lazy_component_loading", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = LazyLoadingConfig::default();
                let loader = LazyLoader::new(config.clone());

                let component = SimpleLazyComponent::new("bench_load_component", || async {
                    // Simulate heavier component initialization
                    tokio::time::sleep(Duration::from_millis(5)).await;
                    Ok(Arc::new(vec![1, 2, 3, 4, 5]) as Arc<dyn std::any::Any + Send + Sync>)
                });

                loader
                    .register_component(Box::new(component))
                    .await
                    .unwrap();

                let start = Instant::now();
                let _result = loader
                    .get_component::<Vec<i32>>("bench_load_component")
                    .await
                    .unwrap();
                let duration = start.elapsed();

                black_box(duration);
            });
        });
    });

    c.bench_function("concurrent_lazy_loading", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = LazyLoadingConfig {
                    max_concurrent_loads: 10,
                    ..Default::default()
                };
                let loader = Arc::new(LazyLoader::new(config));

                let mut tasks = Vec::new();

                for i in 0..10 {
                    let loader_clone = loader.clone();
                    let task = tokio::spawn(async move {
                        let component = SimpleLazyComponent::new(&format!("concurrent_component_{}", i), || async {
                            tokio::time::sleep(Duration::from_millis(2)).await;
                            Ok(Arc::new(i) as Arc<dyn std::any::Any + Send + Sync>)
                        });

                        loader_clone
                            .register_component(Box::new(component))
                            .await
                            .unwrap();
                        loader_clone
                            .get_component::<i32>(&format!("concurrent_component_{}", i))
                            .await
                            .unwrap()
                    });
                    tasks.push(task);
                }

                for task in tasks {
                    black_box(task.await.unwrap());
                }
            });
        });
    });

    c.bench_function("memory_pool_acquisition", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut pool = ObjectPool::<AnalysisResult>::new(100);

                let result = pool.acquire().await.unwrap();
                black_box(result);
            });
        });
    });

    c.bench_function("memory_pool_release", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut pool = ObjectPool::<AnalysisResult>::new(100);

                let obj = pool.acquire().await.unwrap();
                black_box(pool.release(obj).await.unwrap());
            });
        });
    });

    c.bench_function("memory_pool_manager_operations", |b| {
        b.iter(|| {
            rt.block_on(async {
                let manager = MemoryPoolManager::new(100, 50, 50 * 1024 * 1024);

                let analysis_obj = manager.acquire_analysis_result().await.unwrap();
                let model_obj = manager.acquire_model_state().await.unwrap();

                black_box(manager.release_analysis_result(analysis_obj).await.unwrap());
                black_box(manager.release_model_state(model_obj).await.unwrap());
            });
        });
    });

    c.bench_function("performance_monitoring_overhead", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Initialize performance monitor
                PerformanceMonitor::init().await.unwrap();
                let monitor = PerformanceMonitor::global().unwrap();

                // Record various metrics
                monitor
                    .record_component_load("test_component", Duration::from_millis(10))
                    .await;
                monitor.record_memory_usage(1024, Some("test_pool")).await;

                let pool_stats = crate::memory_pool::PoolStats {
                    analysis_pool_size:    10,
                    analysis_pool_created: 10,
                    model_pool_size:       5,
                    model_pool_created:    5,
                    total_memory_usage:    2048,
                    memory_limit:          1024 * 1024,
                };

                monitor.record_pool_stats(pool_stats).await;

                black_box(monitor.generate_performance_report().await);
            });
        });
    });
}

/// Benchmark startup time improvements
fn bench_startup_time_improvements(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("lazy_vs_eager_startup", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Simulate lazy loading startup (only core components)
                let start = Instant::now();
                let config = LazyLoadingConfig::default();
                let _loader = LazyLoader::new(config);
                let lazy_startup_time = start.elapsed();

                // Simulate eager loading startup (all components)
                let start = Instant::now();
                tokio::time::sleep(Duration::from_millis(50)).await; // Simulate loading all components
                let eager_startup_time = start.elapsed();

                black_box((lazy_startup_time, eager_startup_time));
            });
        });
    });

    c.bench_function("component_loading_on_demand", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = LazyLoadingConfig::default();
                let loader = LazyLoader::new(config);

                // Register multiple components
                for i in 0..20 {
                    let component = SimpleLazyComponent::new(&format!("demand_component_{}", i), || async {
                        tokio::time::sleep(Duration::from_millis(1)).await;
                        Ok(Arc::new(i) as Arc<dyn std::any::Any + Send + Sync>)
                    });
                    loader
                        .register_component(Box::new(component))
                        .await
                        .unwrap();
                }

                // Only load first component (on-demand)
                let start = Instant::now();
                let _result = loader
                    .get_component::<i32>("demand_component_0")
                    .await
                    .unwrap();
                let load_time = start.elapsed();

                black_box(load_time);
            });
        });
    });
}

/// Benchmark memory usage improvements
fn bench_memory_usage_improvements(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("pooled_vs_allocated_objects", |b| {
        b.iter(|| {
            rt.block_on(async {
                let manager = MemoryPoolManager::new(1000, 100, 100 * 1024 * 1024);

                let mut pooled_objects = Vec::new();
                let mut allocated_objects = Vec::new();

                // Acquire pooled objects
                for _ in 0..100 {
                    let obj = manager.acquire_analysis_result().await.unwrap();
                    pooled_objects.push(obj);
                }

                // Create allocated objects
                for _ in 0..100 {
                    allocated_objects.push(Arc::new(tokio::sync::Mutex::new(AnalysisResult::default())));
                }

                // Release pooled objects
                for obj in pooled_objects {
                    manager.release_analysis_result(obj).await.unwrap();
                }

                black_box((pooled_objects, allocated_objects));
            });
        });
    });

    c.bench_function("memory_pool_efficiency", |b| {
        b.iter(|| {
            rt.block_on(async {
                let manager = MemoryPoolManager::new(1000, 100, 100 * 1024 * 1024);

                let mut objects = Vec::new();

                // Simulate repeated acquire/release pattern
                for _ in 0..10 {
                    for _ in 0..100 {
                        let obj = manager.acquire_analysis_result().await.unwrap();
                        objects.push(obj);
                    }

                    for obj in objects.drain(..) {
                        manager.release_analysis_result(obj).await.unwrap();
                    }
                }

                let stats = manager.get_pool_stats().await;
                black_box(stats);
            });
        });
    });
}

criterion_group!(
    benches,
    bench_lazy_loading_performance,
    bench_startup_time_improvements,
    bench_memory_usage_improvements
);
criterion_main!(benches);

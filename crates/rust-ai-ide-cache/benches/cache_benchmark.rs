use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_ai_ide_cache::{Cache, CacheConfig, InMemoryCache};
use tokio::runtime::Builder as RuntimeBuilder;

fn cache_operations_benchmark(c: &mut Criterion) {
    let rt = RuntimeBuilder::new_current_thread().build().unwrap();
    let config = CacheConfig {
        max_entries: Some(10000),
        default_ttl: None,
        enable_metrics: false, // Disable metrics for pure performance measurement
        ..Default::default()
    };

    rt.block_on(async {
        let cache: InMemoryCache<String, String> = InMemoryCache::new(&config);

        c.bench_function("cache_insert_small", |b| {
            b.iter(|| {
                let key = format!("key_{}", black_box(0));
                let value = format!("value_{}", black_box(42));
                rt.block_on(async {
                    cache.insert(key, value, None).await.unwrap();
                });
            });
        });

        c.bench_function("cache_get_hit", |b| {
            let key = "test_key".to_string();
            let value = "test_value".to_string();
            rt.block_on(async {
                cache.insert(key.clone(), value, None).await.unwrap();
            });

            b.iter(|| {
                rt.block_on(async {
                    let _result = cache.get(&key).await.unwrap();
                });
            });
        });

        c.bench_function("cache_get_miss", |b| {
            b.iter(|| {
                let key = format!("nonexistent_{}", black_box(0));
                rt.block_on(async {
                    let _result = cache.get(&key).await.unwrap();
                });
            });
        });

        c.bench_function("cache_cleanup_expired", |b| {
            // Setup: insert many expired entries
            rt.block_on(async {
                for i in 0..1000 {
                    let key = format!("expired_key_{}", i);
                    let value = format!("expired_value_{}", i);
                    cache
                        .insert(key, value, Some(Duration::from_nanos(1)))
                        .await
                        .unwrap();
                }
            });

            c.bench_function("cache_cleanup_expired_1000", |b| {
                b.iter(|| {
                    rt.block_on(async {
                        let _count = cache.cleanup_expired().await.unwrap();
                    });
                });
            });
        });
    });
}

fn cache_large_operations_benchmark(c: &mut Criterion) {
    let rt = RuntimeBuilder::new_current_thread().build().unwrap();
    let config = CacheConfig {
        max_entries: Some(50000),
        default_ttl: None,
        enable_metrics: false,
        ..Default::default()
    };

    c.bench_function("cache_bulk_insert_1000", |b| {
        b.iter(|| {
            let cache: InMemoryCache<String, String> = InMemoryCache::new(&config);
            rt.block_on(async {
                for i in 0..1000 {
                    let key = format!("key_{}", i);
                    let value = format!("value_with_some_content_{}", i);
                    cache.insert(key, value, None).await.unwrap();
                }
                black_box(cache);
            });
        });
    });

    let cache: InMemoryCache<String, String> = InMemoryCache::new(&config);
    rt.block_on(async {
        for i in 0..10000 {
            let key = format!("key_{}", i);
            let value = format!("value_with_some_content_{}", i);
            cache.insert(key, value, None).await.unwrap();
        }
    });

    c.bench_function("cache_bulk_get_10000", |b| {
        b.iter(|| {
            rt.block_on(async {
                for i in 0..10000 {
                    let key = format!("key_{}", i);
                    let _result = cache.get(&key).await.unwrap();
                }
            });
        });
    });
}

criterion_group!(
    benches,
    cache_operations_benchmark,
    cache_large_operations_benchmark
);
criterion_main!(benches);

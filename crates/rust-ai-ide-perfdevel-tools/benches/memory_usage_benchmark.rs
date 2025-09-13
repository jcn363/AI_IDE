use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_ai_ide_perfdevel_tools::*;
use std::time::Duration;

fn memory_usage_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    // Test memory allocation with different sizes
    let allocation_sizes = [
        ("small", 1_000),      // 1 KB
        ("medium", 1_000_000), // 1 MB
        ("large", 10_000_000), // 10 MB
    ];

    for (size_name, bytes) in &allocation_sizes {
        group.throughput(criterion::Throughput::Bytes(*bytes as u64));

        group.bench_with_input(
            BenchmarkId::new("allocate_memory", size_name),
            bytes,
            |b, &size| {
                b.iter(|| {
                    // Allocate a vector of the specified size
                    let _vec: Vec<u8> = vec![0; size];
                });
            },
        );
    }

    // Test memory usage with different numbers of small allocations
    let allocation_counts = [100, 1_000, 10_000];

    for &count in &allocation_counts {
        group.bench_with_input(
            BenchmarkId::new("multiple_allocations", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    let mut vecs = Vec::with_capacity(count);
                    for i in 0..count {
                        // Allocate small vectors
                        vecs.push(vec![i as u8; 100]);
                    }
                    vecs
                });
            },
        );
    }

    // Test string memory usage
    group.bench_function("string_allocations", |b| {
        b.iter(|| {
            let mut s = String::new();
            for i in 0..1000 {
                s.push_str(&i.to_string());
            }
            s
        });
    });

    // Test memory fragmentation
    group.bench_function("memory_fragmentation", |b| {
        b.iter(|| {
            let mut vecs = Vec::with_capacity(1000);
            // Allocate and deallocate memory in a way that could cause fragmentation
            for i in 0..1000 {
                let size = if i % 2 == 0 { 100 } else { 1000 };
                let mut v = Vec::with_capacity(size);
                for j in 0..size {
                    v.push(j as u8);
                }
                if i % 3 != 0 {
                    // Keep some allocations
                    vecs.push(v);
                }
            }
            vecs
        });
    });

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(10));
    targets = memory_usage_benchmark
);
criterion_main!(benches);

use criterion::{criterion_group, criterion_main, Criterion};
use rust_ai_ide_simd::*;

fn memory_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_operations");

    // Benchmark memory allocation and deallocation
    group.bench_function("allocate_large_buffer", |b| {
        b.iter(|| {
            let _buffer = vec![0u8; 1024 * 1024]; // 1MB buffer
        });
    });

    // Benchmark memory copy operations
    let src = vec![1u8; 1024 * 1024];
    group.bench_function("copy_large_buffer", |b| {
        b.iter(|| {
            let mut dst = vec![0u8; src.len()];
            dst.copy_from_slice(&src);
            dst
        });
    });

    group.finish();
}

criterion_group!(benches, memory_benchmark);
criterion_main!(benches);

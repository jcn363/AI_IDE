use std::{mem, ptr};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use packed_simd::f32x8; // Using f32x8 for 256-bit SIMD (8 f32 lanes)

// Helper function to generate test data
fn generate_test_data(size: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let mut a = Vec::with_capacity(size);
    let mut b = Vec::with_capacity(size);
    let mut result = vec![0.0; size];

    for i in 0..size {
        a.push(i as f32);
        b.push((i * 2) as f32);
    }

    (a, b, result)
}

// SIMD-accelerated vector addition using packed_simd
fn simd_vector_add(a: &[f32], b: &[f32], result: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), result.len());

    let len = a.len();
    let mut i = 0;

    // Process 8 floats at a time (256-bit SIMD registers)
    while i + 8 <= len {
        // Load 8 f32 values into SIMD vectors
        let a_simd = f32x8::from_slice_unaligned(&a[i..i + 8]);
        let b_simd = f32x8::from_slice_unaligned(&b[i..i + 8]);

        // Perform SIMD addition
        let sum = a_simd + b_simd;

        // Store the result
        sum.write_to_slice_unaligned(&mut result[i..i + 8]);
        i += 8;
    }

    // Process remaining elements
    while i < len {
        result[i] = a[i] + b[i];
        i += 1;
    }
}

// Regular vector addition for comparison
fn scalar_vector_add(a: &[f32], b: &[f32], result: &mut [f32]) {
    for i in 0..a.len() {
        result[i] = a[i] + b[i];
    }
}

fn simd_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("SIMD Operations");

    // Test different vector sizes
    let sizes = [
        ("small", 1_000),     // 1K elements
        ("medium", 10_000),   // 10K elements
        ("large", 1_000_000), // 1M elements
    ];

    for (size_name, size) in &sizes {
        let (a, b, mut result) = generate_test_data(*size);

        // Benchmark SIMD vector addition
        group.bench_with_input(
            BenchmarkId::new("SIMD Vector Add", size_name),
            size,
            |b, _| {
                b.iter(|| unsafe {
                    simd_vector_add(&a, &b, &mut result);
                    black_box(&mut result);
                });
            },
        );

        // Benchmark scalar vector addition for comparison
        group.bench_with_input(
            BenchmarkId::new("Scalar Vector Add", size_name),
            size,
            |b, _| {
                b.iter(|| {
                    scalar_vector_add(&a, &b, &mut result);
                    black_box(&mut result);
                });
            },
        );
    }

    // Benchmark matrix multiplication
    group.bench_function("SIMD Matrix Multiplication", |b| {
        // 64x64 matrices
        const N: usize = 64;
        let a = vec![1.0f32; N * N];
        let b = vec![2.0f32; N * N];
        let mut result = vec![0.0f32; N * N];

        b.iter(|| {
            // Simple matrix multiplication (can be optimized further with SIMD)
            for i in 0..N {
                for k in 0..N {
                    let a_ik = a[i * N + k];
                    for j in 0..N {
                        result[i * N + j] += a_ik * b[k * N + j];
                    }
                }
            }
            black_box(&mut result);
        });
    });

    // Benchmark memory operations
    group.bench_function("SIMD Memory Copy", |b| {
        let src = vec![1.0f32; 1024 * 1024]; // 1MB of data
        let mut dst = vec![0.0f32; 1024 * 1024];

        b.iter(|| {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    src.as_ptr(),
                    dst.as_mut_ptr(),
                    src.len() / mem::size_of::<f32>(),
                );
            }
            black_box(&mut dst);
        });
    });

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(10));
    targets = simd_benchmark
);
criterion_main!(benches);

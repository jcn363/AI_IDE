use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use rust_ai_ide_perfdevel_tools::*;
use std::time::Duration;

fn completion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("completion");
    
    // Test with different input sizes
    let input_sizes = [100, 1000, 10000];
    
    for &size in &input_sizes {
        group.throughput(criterion::Throughput::Elements(size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("process_completion", size), 
            &size, 
            |b, &size| {
                // Generate test data
                let test_data: Vec<_> = (0..size)
                    .map(|i| format!("test_data_{}", i))
                    .collect();
                
                b.iter(|| {
                    // Simulate completion processing
                    let _result: Vec<_> = test_data
                        .iter()
                        .map(|s| s.to_uppercase())
                        .collect();
                });
            }
        );
    }
    
    // Benchmark with different levels of concurrency
    let concurrency_levels = [1, 4, 8, 16];
    
    for &level in &concurrency_levels {
        group.bench_with_input(
            BenchmarkId::new("concurrent_completion", level), 
            &level, 
            |b, &level| {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(level)
                    .build()
                    .unwrap();
                
                b.iter(|| {
                    rt.block_on(async {
                        let handles: Vec<_> = (0..level)
                            .map(|i| {
                                tokio::spawn(async move {
                                    // Simulate some async work
                                    tokio::time::sleep(Duration::from_micros(100)).await;
                                    i * 2
                                })
                            })
                            .collect();
                        
                        let results: Vec<_> = futures::future::join_all(handles).await;
                        results.into_iter().map(Result::unwrap).sum::<i32>()
                    });
                });
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(10));
    targets = completion_benchmark
);
criterion_main!(benches);

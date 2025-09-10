use criterion::{criterion_group, criterion_main, Criterion};
use rust_ai_ide_quality_dashboard::*;
use std::time::Duration;

fn metric_collection_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("metric_collection");
    
    // Benchmark collecting metrics at different rates
    for &batch_size in &[10, 100, 1000] {
        group.bench_function(
            format!("collect_{}_metrics_per_batch", batch_size), 
            |b| {
                let collector = MetricCollector::new();
                let metrics: Vec<_> = (0..batch_size)
                    .map(|i| Metric {
                        timestamp: chrono::Utc::now(),
                        value: i as f64,
                        name: format!("metric_{}", i % 10),
                        tags: vec![format!("tag_{}", i % 5)],
                    })
                    .collect();
                
                b.iter(|| {
                    collector.collect_batch(metrics.clone());
                });
            }
        );
    }
    
    // Benchmark query performance with different time ranges
    let time_ranges = [
        ("1h", Duration::from_secs(3600)),
        ("1d", Duration::from_secs(86400)),
        ("7d", Duration::from_secs(604800)),
    ];
    
    for (name, duration) in time_ranges {
        group.bench_function(
            format!("query_metrics_{}", name), 
            |b| {
                let collector = MetricCollector::new();
                let end_time = chrono::Utc::now();
                let start_time = end_time - chrono::Duration::from_std(duration).unwrap();
                
                b.iter(|| {
                    collector.query_metrics(start_time, end_time, None);
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
    targets = metric_collection_benchmark
);
criterion_main!(benches);

use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use rust_ai_ide_quality_dashboard::*;

fn dashboard_performance_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("dashboard_performance");

    // Benchmark dashboard rendering with different data sizes
    for &data_size in &[100, 1000, 10000] {
        group.bench_function(format!("render_with_{}_data_points", data_size), |b| {
            b.iter(|| {
                // Simulate data collection
                let metrics: Vec<_> = (0..data_size)
                    .map(|i| Metric {
                        timestamp: chrono::Utc::now(),
                        value:     i as f64,
                        name:      format!("metric_{}", i % 10),
                        tags:      vec![format!("tag_{}", i % 5)],
                    })
                    .collect();

                // Simulate dashboard rendering
                let dashboard = Dashboard::new(metrics);
                dashboard.render()
            });
        });
    }

    // Benchmark dashboard updates
    group.bench_function("update_dashboard", |b| {
        let mut dashboard = Dashboard::new(Vec::new());
        b.iter(|| {
            let metric = Metric {
                timestamp: chrono::Utc::now(),
                value:     42.0,
                name:      "sample_metric".to_string(),
                tags:      vec!["test".to_string()],
            };
            dashboard.update(metric);
        });
    });

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(10));
    targets = dashboard_performance_benchmark
);
criterion_main!(benches);

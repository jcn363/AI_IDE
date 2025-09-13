use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_ai_ide_perfdevel_tools::*;
use std::time::Duration;

fn lsp_server_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("lsp_server");

    // Benchmark document analysis with different file sizes
    let document_sizes = [
        ("small", 100),   // ~100 lines
        ("medium", 1000), // ~1,000 lines
        ("large", 10000), // ~10,000 lines
    ];

    for (size_name, line_count) in &document_sizes {
        group.throughput(criterion::Throughput::Elements(*line_count as u64));

        group.bench_with_input(
            BenchmarkId::new("analyze_document", size_name),
            line_count,
            |b, &lines| {
                // Generate a test document with the specified number of lines
                let content: String = (0..lines)
                    .map(|i| format!("fn test_{}() {{ /* Some Rust code */ }}\n", i))
                    .collect();

                b.iter(|| {
                    // Simulate document analysis
                    let _tokens: Vec<_> = content
                        .lines()
                        .flat_map(|line| line.split_whitespace())
                        .collect();
                });
            },
        );
    }

    // Benchmark code completion at different positions
    let completion_positions = [
        ("start", 0.1),  // Near start of file
        ("middle", 0.5), // Middle of file
        ("end", 0.9),    // Near end of file
    ];

    for (pos_name, ratio) in &completion_positions {
        group.bench_with_input(
            BenchmarkId::new("code_completion", pos_name),
            ratio,
            |b, &ratio| {
                // Create a document with 1000 lines
                let lines: Vec<String> = (0..1000)
                    .map(|i| format!("fn test_{}() {{ /* Some Rust code */ }}", i))
                    .collect();

                // Calculate position based on ratio
                let line = (lines.len() as f64 * ratio).round() as usize;

                b.iter(|| {
                    // Simulate code completion
                    let _suggestions: Vec<_> = lines
                        .iter()
                        .filter(|l| l.contains(&format!("test_{}", line % 10)))
                        .take(10)
                        .collect();
                });
            },
        );
    }

    // Benchmark diagnostics
    group.bench_function("run_diagnostics", |b| {
        let content: String = (0..1000)
            .map(|i| {
                format!(
                    "fn test_{}() {{ /* Some Rust code with potential issues */ }}\n",
                    i
                )
            })
            .collect();

        b.iter(|| {
            // Simulate running diagnostics
            let _issues: Vec<_> = content
                .lines()
                .enumerate()
                .filter(|(_, line)| line.contains("potential"))
                .map(|(i, _)| format!("Issue found on line {}", i + 1))
                .collect();
        });
    });

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(10));
    targets = lsp_server_benchmark
);
criterion_main!(benches);

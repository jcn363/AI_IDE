use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_ai_ide_advanced_refactoring::*;

// Helper function to generate a mock codebase
fn generate_mock_codebase(size: usize) -> Codebase {
    let mut modules = Vec::with_capacity(size);

    for i in 0..size {
        let complexity = i % 10 + 1; // 1-10 complexity
        let dependencies = (i % 5) + 1; // 1-5 dependencies

        let mut deps = Vec::with_capacity(dependencies);
        for d in 0..dependencies {
            deps.push(format!("module_{}", (i + d) % size));
        }

        modules.push(CodeModule {
            name: format!("module_{}", i),
            path: format!("/path/to/module_{}.rs", i),
            complexity,
            dependencies: deps,
            size: 100 + (i % 1000), // 100-1099 lines
        });
    }

    Codebase { modules }
}

fn refactoring_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("refactoring_suggestions");

    // Test with different codebase sizes
    let codebase_sizes = [
        ("small", 10),   // 10 modules
        ("medium", 100), // 100 modules
        ("large", 1000), // 1000 modules
    ];

    for (size_name, size) in &codebase_sizes {
        let codebase = generate_mock_codebase(*size);

        // Benchmark generating refactoring suggestions
        group.bench_with_input(
            BenchmarkId::new("generate_suggestions", size_name),
            &codebase,
            |b, codebase| {
                let analyzer = RefactoringAnalyzer::new();
                b.iter(|| {
                    let suggestions = analyzer.analyze_codebase(&codebase);
                    black_box(suggestions);
                });
            },
        );

        // Benchmark applying refactoring
        group.bench_with_input(
            BenchmarkId::new("apply_refactoring", size_name),
            &codebase,
            |b, codebase| {
                let refactoring = Refactoring {
                    name:       "extract_method".to_string(),
                    target:     "module_0".to_string(),
                    parameters: vec!["function1".to_string()],
                };

                b.iter(|| {
                    let mut codebase_clone = codebase.clone();
                    let result = codebase_clone.apply_refactoring(&refactoring);
                    black_box(result);
                });
            },
        );
    }

    // Benchmark complex refactoring scenarios
    group.bench_function("complex_refactoring_scenario", |b| {
        let codebase = generate_mock_codebase(100);
        let analyzer = RefactoringAnalyzer::new();

        b.iter(|| {
            // 1. Get suggestions
            let suggestions = analyzer.analyze_codebase(&codebase);

            // 2. Apply first suggestion
            if let Some(first_suggestion) = suggestions.first() {
                let mut codebase_clone = codebase.clone();
                let result = codebase_clone.apply_refactoring(&first_suggestion);
                black_box(result);

                // 3. Get new suggestions after refactoring
                let new_suggestions = analyzer.analyze_codebase(&codebase_clone);
                black_box(new_suggestions);
            }
        });
    });

    group.finish();
}

// Mock implementations for the benchmark
#[derive(Clone)]
struct CodeModule {
    name:         String,
    path:         String,
    complexity:   u32,
    dependencies: Vec<String>,
    size:         usize,
}

#[derive(Clone)]
struct Codebase {
    modules: Vec<CodeModule>,
}

impl Codebase {
    fn apply_refactoring(&mut self, _refactoring: &Refactoring) -> bool {
        // Simulate applying a refactoring
        true
    }
}

struct RefactoringAnalyzer;

impl RefactoringAnalyzer {
    fn new() -> Self {
        RefactoringAnalyzer
    }

    fn analyze_codebase(&self, codebase: &Codebase) -> Vec<Refactoring> {
        // Simulate analysis - in a real implementation, this would be more complex
        codebase
            .modules
            .iter()
            .filter(|m| m.complexity > 7) // Suggest refactoring for complex modules
            .map(|m| Refactoring {
                name: "simplify_module".to_string(),
                target: m.name.clone(),
                parameters: vec![format!("complexity_{}", m.complexity)],
            })
            .collect()
    }
}

struct Refactoring {
    name:       String,
    target:     String,
    parameters: Vec<String>,
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(10));
    targets = refactoring_benchmark
);
criterion_main!(benches);

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use rust_ai_ide_advanced_refactoring::*;
use std::time::Duration;

fn impact_assessment_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("impact_assessment");
    
    // Test with different codebase sizes (in lines of code)
    let codebase_sizes = [
        ("small", 1_000),     // 1K LOC
        ("medium", 10_000),   // 10K LOC
        ("large", 100_000),   // 100K LOC
    ];
    
    for (size_name, loc) in &codebase_sizes {
        group.throughput(criterion::Throughput::Elements(*loc as u64));
        
        group.bench_with_input(
            BenchmarkId::new("assess_impact", size_name),
            loc,
            |b, &loc| {
                // Simulate a codebase with the specified size
                let codebase = generate_mock_codebase(loc);
                
                b.iter(|| {
                    // Simulate impact assessment
                    let impact = ImpactAssessment {
                        affected_modules: codebase.modules.len() / 10, // 10% of modules affected
                        estimated_effort: loc / 100, // 1% of total LOC needs changes
                        complexity: if *loc > 50_000 { "high" } else { "medium" }.to_string(),
                        risk_level: if *loc > 50_000 { "high" } else { "medium" }.to_string(),
                        dependencies: codebase.dependencies.len(),
                    };
                    impact
                });
            },
        );
    }
    
    // Test with different change scopes
    let change_scopes = [
        ("local", 0.01),    // 1% of codebase
        ("module", 0.1),    // 10% of codebase
        ("cross_cutting", 0.5), // 50% of codebase
    ];
    
    for (scope_name, scope_factor) in &change_scopes {
        group.bench_with_input(
            BenchmarkId::new("assess_change_scope", scope_name),
            scope_factor,
            |b, &scope_factor| {
                let codebase_size = 10_000; // Fixed size for this test
                let codebase = generate_mock_codebase(codebase_size);
                
                b.iter(|| {
                    let affected = (codebase.modules.len() as f64 * scope_factor).ceil() as usize;
                    let impact = ImpactAssessment {
                        affected_modules: affected,
                        estimated_effort: (codebase_size as f64 * scope_factor * 0.1) as usize, // 10% of scope
                        complexity: if *scope_factor > 0.3 { "high" } else { "medium" }.to_string(),
                        risk_level: if *scope_factor > 0.3 { "high" } else { "medium" }.to_string(),
                        dependencies: (codebase.dependencies.len() as f64 * *scope_factor).ceil() as usize,
                    };
                    impact
                });
            },
        );
    }
    
    // Test dependency analysis
    group.bench_function("analyze_dependencies", |b| {
        let codebase = generate_mock_codebase(50_000);
        
        b.iter(|| {
            // Simulate dependency analysis
            let mut impact = ImpactAssessment {
                affected_modules: 0,
                estimated_effort: 0,
                complexity: "medium".to_string(),
                risk_level: "medium".to_string(),
                dependencies: 0,
            };
            
            // Analyze dependencies (simplified)
            impact.dependencies = codebase.dependencies
                .iter()
                .filter(|d| d.impact_level > 5) // Arbitrary threshold
                .count();
                
            impact
        });
    });
    
    group.finish();
}

// Helper structs and functions for the benchmark
struct MockCodebase {
    modules: Vec<MockModule>,
    dependencies: Vec<MockDependency>,
}

struct MockModule {
    loc: usize,
    complexity: usize,
}

struct MockDependency {
    impact_level: u8, // 1-10 scale
}

fn generate_mock_codebase(loc: usize) -> MockCodebase {
    // Simple mock codebase generation
    let module_count = (loc as f64 / 100.0).ceil() as usize; // ~100 LOC per module
    let dep_count = (module_count as f64 * 1.5).ceil() as usize; // 1.5x modules for dependencies
    
    let modules = (0..module_count)
        .map(|_| MockModule {
            loc: 100, // Average 100 LOC per module
            complexity: 5, // Medium complexity
        })
        .collect();
    
    let dependencies = (0..dep_count)
        .map(|i| MockDependency {
            impact_level: (i % 10 + 1) as u8, // 1-10
        })
        .collect();
    
    MockCodebase {
        modules,
        dependencies,
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(10));
    targets = impact_assessment_benchmark
);
criterion_main!(benches);

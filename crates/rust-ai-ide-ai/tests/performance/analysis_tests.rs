//! Performance analysis tests for the Rust AI IDE

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_ai_ide_ai::analysis::architectural::{
    CircularDependencyAnalyzer, DependencyInversionAnalyzer, InterfaceSegregationAnalyzer, LayerViolationDetector,
};
use rust_ai_ide_ai::analysis::{AnalysisRegistry, AnalysisType};
use rust_ai_ide_ai::test_helpers::create_test_ast;

/// Performance test for the circular dependency analyzer
fn benchmark_circular_dependency_analyzer(c: &mut Criterion) {
    let code = r#"
        mod a { pub fn a() { b::b(); } }
        mod b { pub fn b() { c::c(); } }
        mod c { pub fn c() { a::a(); } }
    "#;

    let ast = create_test_ast(code);
    let analyzer = CircularDependencyAnalyzer::default();

    c.bench_function("circular_dependency_analyzer", |b| {
        b.iter(|| {
            let mut findings = Vec::new();
            analyzer.analyze(black_box(&ast), black_box(code), black_box("test.rs"));
            black_box(findings);
        })
    });
}

/// Performance test for the layer violation detector
fn benchmark_layer_violation_detector(c: &mut Criterion) {
    let code = r#"
        mod domain {
            use super::infrastructure::Database;
            pub struct Service { db: Database }
        }
        mod infrastructure {
            use super::domain::Service;
            pub struct Database;
        }
    "#;

    let ast = create_test_ast(code);
    let detector = LayerViolationDetector::default();

    c.bench_function("layer_violation_detector", |b| {
        b.iter(|| {
            let mut findings = Vec::new();
            detector.analyze(black_box(&ast), black_box(code), black_box("test.rs"));
            black_box(findings);
        })
    });
}

/// Performance test for the full analysis pipeline
fn benchmark_full_analysis(c: &mut Criterion) {
    let code = r#"
        mod domain {
            pub trait Repository {}
            pub struct Service<T: Repository> { repo: T }
        }
        mod infrastructure {
            use super::domain::Repository;
            pub struct DbRepository;
            impl Repository for DbRepository {}
        }
        mod application {
            use super::domain::Service;
            use super::infrastructure::DbRepository;
            pub fn run() { let _service = Service { repo: DbRepository }; }
        }
    "#;

    let mut registry = AnalysisRegistry::default();
    registry.register_architectural_analyzer(CircularDependencyAnalyzer::default());
    registry.register_architectural_analyzer(LayerViolationDetector::default());
    registry.register_architectural_analyzer(InterfaceSegregationAnalyzer::default());
    registry.register_architectural_analyzer(DependencyInversionAnalyzer::default());

    c.bench_function("full_analysis_pipeline", |b| {
        b.iter(|| {
            let result = registry.analyze_code(black_box(code), black_box("test.rs"));
            black_box(result);
        })
    });
}

criterion_group!(
    benches,
    benchmark_circular_dependency_analyzer,
    benchmark_layer_violation_detector,
    benchmark_full_analysis
);
criterion_main!(benches);

//! Benchmarks for performance-critical paths in the analysis pipeline

use criterion::{criterion_group, criterion_main, Criterion};
use rust_ai_ide_ai::{
    analysis::{
        architectural::{
            CircularDependencyAnalyzer, DependencyInversionAnalyzer, InterfaceSegregationAnalyzer,
            LayerViolationDetector,
        },
        metrics::{
            CognitiveComplexityCalculator, CyclomaticComplexityCalculator, HalsteadMetrics,
            MaintainabilityIndex, MetricsAnalyzer, SourceLinesOfCode,
        },
        security::{HardcodedSecretsDetector, InsecureCryptoDetector, SqlInjectionDetector},
        AnalysisPipeline, AnalysisPipelineBuilder, AnalysisResult, AnalysisType,
    },
    test_helpers::create_test_ast,
};
use std::hint::black_box;
use std::path::Path;

/// Benchmark the analysis of a small code snippet
fn benchmark_small_code_analysis(c: &mut Criterion) {
    let code = r#"
        mod a {
            use super::b;
            pub fn a() { b::b(); }
        }
        
        mod b {
            use super::a;
            pub fn b() { a::a(); }
        }
        
        fn main() {
            a::a();
        }
    "#;

    let pipeline = AnalysisPipelineBuilder::new()
        .with_architectural_analyzer(CircularDependencyAnalyzer::default())
        .build();

    c.bench_function("small_code_analysis", |b| {
        b.iter(|| {
            let result = pipeline.analyze_code(black_box(code), "benchmark.rs");
            black_box(result);
        })
    });
}

/// Benchmark the analysis of a medium-sized codebase
fn benchmark_medium_codebase_analysis(c: &mut Criterion) {
    // Generate a medium-sized codebase with multiple modules
    let mut code = String::new();

    // Add some modules
    for i in 0..10 {
        code.push_str(&format!("mod module{} {{\n", i));

        // Add some functions to each module
        for j in 0..5 {
            code.push_str(&format!("    pub fn func{}() {{}}\n", j));
        }

        // Add some cross-module calls
        if i > 0 {
            code.push_str(&format!(
                "    pub fn call_prev() {{ module{}::func0(); }}\n",
                i - 1
            ));
        }

        code.push_str("}\n\n");
    }

    // Add a main function that uses all modules
    code.push_str("fn main() {\n");
    for i in 0..10 {
        code.push_str(&format!("    module{}::func0();\n", i));
    }
    code.push_str("}\n");

    let pipeline = AnalysisPipelineBuilder::new()
        .with_architectural_analyzer(CircularDependencyAnalyzer::default())
        .with_architectural_analyzer(DependencyInversionAnalyzer::default())
        .build();

    c.bench_function("medium_codebase_analysis", |b| {
        b.iter(|| {
            let result = pipeline.analyze_code(black_box(&code), "benchmark.rs");
            black_box(result);
        })
    });
}

/// Benchmark the performance of the circular dependency analyzer
fn benchmark_circular_dependency_analyzer(c: &mut Criterion) {
    // Create a deep dependency chain
    let mut code = String::new();

    // Add modules with linear dependencies
    for i in 0..20 {
        code.push_str(&format!("mod module{} {{\n", i));

        if i > 0 {
            code.push_str(&format!("    use super::module{};\n", i - 1));
            code.push_str("    pub fn func() {\n        // Some code\n");
            code.push_str(&format!("        module{}::func();\n", i - 1));
            code.push_str("    }\n");
        } else {
            code.push_str("    pub fn func() {}\n");
        }

        code.push_str("}\n\n");
    }

    // Add a circular dependency
    code.push_str("mod module_circular_a {\n    use super::module_circular_b;\n    pub fn a() { module_circular_b::b(); }\n}\n\n");
    code.push_str("mod module_circular_b {\n    use super::module_circular_a;\n    pub fn b() { module_circular_a::a(); }\n}\n\n");

    // Add main function
    code.push_str("fn main() {\n    module19::func();\n    module_circular_a::a();\n}\n");

    let analyzer = CircularDependencyAnalyzer::default();
    let ast = create_test_ast(&code);

    c.bench_function("circular_dependency_analyzer", |b| {
        b.iter(|| {
            let result = analyzer.analyze(black_box(&ast), black_box(&code), "benchmark.rs");
            black_box(result);
        })
    });
}

/// Benchmark the performance of the metrics analyzer
fn benchmark_metrics_analyzer(c: &mut Criterion) {
    // Generate a complex function for metrics analysis
    let mut code = String::new();

    // Add a complex function
    code.push_str("fn complex_function(x: i32) -> i32 {\n");
    code.push_str("    let mut result = 0;\n\n");

    // Add nested control structures
    code.push_str("    if x > 0 {\n");
    code.push_str("        for i in 0..x {\n");
    code.push_str("            match i % 3 {\n");
    code.push_str("                0 => result += i,\n");
    code.push_str("                1 => result -= i * 2,\n");
    code.push_str("                _ => result *= i,\n");
    code.push_str("            }\n");
    code.push_str("        }\n");
    code.push_str("    } else {\n");
    code.push_str("        let mut i = x;\n");
    code.push_str("        while i < 0 {\n");
    code.push_str("            result += i;\n");
    code.push_str("            i += 1;\n");
    code.push_str("        }\n");
    code.push_str("    }\n\n");

    // Add more complexity
    code.push_str("    let closure = |a: i32, b: i32| -> i32 {\n");
    code.push_str("        let sum = a + b;\n");
    code.push_str("        sum * sum\n");
    code.push_str("    };\n\n");

    code.push_str("    result = closure(result, x);\n");
    code.push_str("    result\n");
    code.push_str("}\n");

    // Add a trait and implementation
    code.push_str("trait MyTrait {\n");
    code.push_str("    fn method(&self) -> i32;\n");
    code.push_str("}\n\n");

    code.push_str("struct MyStruct { value: i32 }\n\n");

    code.push_str("impl MyTrait for MyStruct {\n");
    code.push_str("    fn method(&self) -> i32 {\n");
    code.push_str("        self.value * 2\n");
    code.push_str("    }\n");
    code.push_str("}\n");

    let analyzer = MetricsAnalyzer::default();
    let ast = create_test_ast(&code);

    c.bench_function("metrics_analyzer", |b| {
        b.iter(|| {
            let result = analyzer.analyze(black_box(&ast), black_box(&code), "benchmark.rs");
            black_box(result);
        })
    });
}

/// Benchmark the performance of the security analyzers
fn benchmark_security_analyzers(c: &mut Criterion) {
    // Generate code with various security issues
    let code = r#"
        // Hardcoded secret
        const API_KEY: &str = "sk_test_1234567890abcdef12345678";
        
        // Insecure crypto
        fn hash_password(password: &str) -> String {
            use md5;
            let hash = md5::compute(password.as_bytes());
            format!("{:x}", hash)
        }
        
        // SQL injection
        fn get_user(pool: &sqlx::PgPool, user_id: &str) -> Result<(), sqlx::Error> {
            let query = format!("SELECT * FROM users WHERE id = {}", user_id);
            sqlx::query(&query).fetch_one(pool).await?;
            Ok(())
        }
        
        // Hardcoded credentials
        struct DatabaseConfig {
            username: &'static str,
            password: &'static str,
        }
        
        impl DatabaseConfig {
            fn new() -> Self {
                Self {
                    username: "admin",
                    password: "s3cr3tP@ssw0rd",
                }
            }
        }
    "#;

    let mut group = c.benchmark_group("security_analyzers");

    // Benchmark each security analyzer separately
    group.bench_function("hardcoded_secrets_detector", |b| {
        let analyzer = HardcodedSecretsDetector::default();
        let ast = create_test_ast(code);

        b.iter(|| {
            let result = analyzer.analyze(black_box(&ast), black_box(code), "benchmark.rs");
            black_box(result);
        })
    });

    group.bench_function("insecure_crypto_detector", |b| {
        let analyzer = InsecureCryptoDetector::default();
        let ast = create_test_ast(code);

        b.iter(|| {
            let result = analyzer.analyze(black_box(&ast), black_box(code), "benchmark.rs");
            black_box(result);
        })
    });

    group.bench_function("sql_injection_detector", |b| {
        let analyzer = SqlInjectionDetector::default();
        let ast = create_test_ast(code);

        b.iter(|| {
            let result = analyzer.analyze(black_box(&ast), black_box(code), "benchmark.rs");
            black_box(result);
        })
    });

    group.finish();
}

/// Benchmark the performance of the full analysis pipeline
fn benchmark_full_pipeline(c: &mut Criterion) {
    // Create a pipeline with all analyzers
    let pipeline = AnalysisPipelineBuilder::new()
        // Architectural analyzers
        .with_architectural_analyzer(CircularDependencyAnalyzer::default())
        .with_architectural_analyzer(DependencyInversionAnalyzer::default())
        .with_architectural_analyzer(InterfaceSegregationAnalyzer::default())
        .with_architectural_analyzer(LayerViolationDetector::default())
        // Security analyzers
        .with_security_analyzer(InsecureCryptoDetector::default())
        .with_security_analyzer(HardcodedSecretsDetector::default())
        .with_security_analyzer(SqlInjectionDetector::default())
        // Metrics analyzers
        .with_metrics_analyzer(MetricsAnalyzer::default())
        .build();

    // Test code with various patterns
    let code = r#"
        // Circular dependency
        mod a { pub fn a() { b::b(); } }
        mod b { pub fn b() { a::a(); } }
        
        // Layer violation
        mod domain {
            use super::infrastructure::Database;
            pub struct Service { db: Database }
        }
        
        mod infrastructure {
            use super::domain::Service;
            pub struct Database;
        }
        
        // Insecure crypto
        fn hash_password(pwd: &str) -> String {
            use md5;
            let hash = md5::compute(pwd.as_bytes());
            format!("{:x}", hash)
        }
        
        // Hardcoded secret
        const API_KEY: &str = "sk_test_1234567890";
        
        // SQL injection
        fn get_user(pool: &sqlx::PgPool, user_id: &str) -> Result<(), sqlx::Error> {
            let query = format!("SELECT * FROM users WHERE id = {}", user_id);
            sqlx::query(&query).fetch_one(pool).await?;
            Ok(())
        }
        
        // High complexity function
        fn complex_function(x: i32) -> i32 {
            let mut result = 0;
            for i in 0..x {
                if i % 2 == 0 {
                    result += i;
                } else if i % 3 == 0 {
                    result -= i;
                } else {
                    result *= i;
                }
            }
            result
        }
    "#;

    c.bench_function("full_pipeline_analysis", |b| {
        b.iter(|| {
            let result = pipeline.analyze_code(black_box(code), "benchmark.rs");
            black_box(result);
        })
    });
}

criterion_group!(
    benches,
    benchmark_small_code_analysis,
    benchmark_medium_codebase_analysis,
    benchmark_circular_dependency_analyzer,
    benchmark_metrics_analyzer,
    benchmark_security_analyzers,
    benchmark_full_pipeline,
);

criterion_main!(benches);

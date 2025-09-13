//! End-to-end integration tests for the Rust AI IDE analysis engine

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
        AnalysisPipeline, AnalysisPipelineBuilder, AnalysisResult, AnalysisType, Severity,
    },
    test_helpers::*,
};
use std::path::Path;

/// Test a complete analysis pipeline with all analyzers
#[test]
fn test_complete_analysis_pipeline() {
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

    // Test code with various issues
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

    // Analyze the code
    let result = pipeline.analyze_code(code, "test.rs").unwrap();

    // Verify we found all expected issues
    let mut found_issues = std::collections::HashSet::new();

    for finding in &result.findings {
        match finding.analysis_type {
            AnalysisType::CircularDependency => {
                found_issues.insert("circular_dependency");
            }
            AnalysisType::LayerViolation => {
                found_issues.insert("layer_violation");
            }
            AnalysisType::InsecureCrypto => {
                found_issues.insert("insecure_crypto");
            }
            AnalysisType::HardcodedSecret => {
                found_issues.insert("hardcoded_secret");
            }
            AnalysisType::SqlInjection => {
                found_issues.insert("sql_injection");
            }
            AnalysisType::CodeMetrics => {
                if finding.message.contains("high complexity") {
                    found_issues.insert("high_complexity");
                }
            }
            _ => {}
        }
    }

    // Check that all expected issues were found
    let expected_issues = [
        "circular_dependency",
        "layer_violation",
        "insecure_crypto",
        "hardcoded_secret",
        "sql_injection",
        "high_complexity",
    ];

    for issue in &expected_issues {
        assert!(
            found_issues.contains(*issue),
            "Expected to find issue: {}",
            issue
        );
    }
}

/// Test the analysis of a complete Rust project
#[test]
fn test_project_analysis() {
    // Create a temporary directory for the test project
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let project_dir = temp_dir.path();

    // Create a simple Rust project structure
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    // Create Cargo.toml
    std::fs::write(
        project_dir.join("Cargo.toml"),
        r#"
        [package]
        name = "test_project"
        version = "0.1.0"
        edition = "2021"

        [dependencies]
        sqlx = { version = "0.6", features = ["postgres", "runtime-tokio"] }
        md-5 = "0.10"
        "#,
    )
    .unwrap();

    // Create main.rs
    std::fs::write(
        src_dir.join("main.rs"),
        r#"
        mod domain;
        mod infrastructure;

        use domain::service::Service;
        use infrastructure::database::Database;

        #[tokio::main]
        async fn main() {
            let db = Database::new();
            let service = Service::new(db);

            // This would be insecure in a real application
            let _hash = service.hash_password("password123");

            // This would be a SQL injection vulnerability
            let _user = service.get_user("1 OR 1=1").await.unwrap();
        }
        "#,
    )
    .unwrap();

    // Create domain/mod.rs
    std::fs::write(
        src_dir.join("domain.rs"),
        r#"
        // Domain layer

        // This is a layer violation - domain should not depend on infrastructure
        use crate::infrastructure::database::Database;

        pub struct Service {
            db: Database,
        }

        impl Service {
            pub fn new(db: Database) -> Self {
                Self { db }
            }

            pub fn hash_password(&self, password: &str) -> String {
                // Insecure hashing
                let hash = md5::compute(password.as_bytes());
                format!("{:x}", hash)
            }

            pub async fn get_user(&self, user_id: &str) -> Result<(), sqlx::Error> {
                // SQL injection vulnerability
                let query = format!("SELECT * FROM users WHERE id = {}", user_id);
                sqlx::query(&query).fetch_one(&self.db.pool).await?;
                Ok(())
            }
        }
        "#,
    )
    .unwrap();

    // Create infrastructure/mod.rs
    std::fs::write(
        src_dir.join("infrastructure.rs"),
        r#"
        // Infrastructure layer

        use sqlx::PgPool;

        pub struct Database {
            pub pool: PgPool,
        }

        impl Database {
            pub fn new() -> Self {
                // In a real app, we would connect to a database
                // This is just for testing
                Self {
                    pool: PgPool::connect_lazy("postgres://user:pass@localhost/db").unwrap(),
                }
            }
        }
        "#,
    )
    .unwrap();

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

    // Analyze the project
    let results = pipeline
        .analyze_project(project_dir.to_str().unwrap())
        .unwrap();

    // Verify we analyzed all source files
    let expected_files = ["src/main.rs", "src/domain.rs", "src/infrastructure.rs"];
    for file in &expected_files {
        assert!(
            results.contains_key(*file),
            "Expected to analyze file: {}",
            file
        );
    }

    // Check for specific issues in domain.rs
    if let Some(domain_result) = results.get("src/domain.rs") {
        // Should find layer violation (domain depending on infrastructure)
        assert_finding!(
            domain_result,
            AnalysisType::LayerViolation,
            Severity::Error,
            "Domain layer depends on infrastructure layer"
        );

        // Should find insecure crypto (MD5 hashing)
        assert_finding!(
            domain_result,
            AnalysisType::InsecureCrypto,
            Severity::High,
            "Insecure hashing function 'md5::compute' detected"
        );

        // Should find SQL injection
        assert_finding!(
            domain_result,
            AnalysisType::SqlInjection,
            Severity::Critical,
            "Potential SQL injection vulnerability"
        );
    } else {
        panic!("Failed to analyze domain.rs");
    }

    // Clean up
    temp_dir.close().expect("Failed to clean up temp dir");
}

/// Test the analysis of a real-world Rust crate
#[test]
fn test_real_world_crate_analysis() {
    // Skip this test in CI as it requires network access
    if std::env::var("CI").is_ok() {
        return;
    }

    // Clone a small, well-known Rust crate for testing
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let crate_dir = temp_dir.path().join("serde_json");

    // Clone serde_json (a small, well-maintained crate)
    let status = std::process::Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "https://github.com/serde-rs/json.git",
        ])
        .arg(&crate_dir)
        .status()
        .expect("Failed to clone serde_json");

    if !status.success() {
        eprintln!("Warning: Failed to clone serde_json for testing");
        return;
    }

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

    // Analyze the crate
    let results = pipeline
        .analyze_project(crate_dir.to_str().unwrap())
        .unwrap();

    // Verify we analyzed some files
    assert!(!results.is_empty(), "Should analyze at least one file");

    // Check for specific files we expect to find
    let expected_files = ["src/lib.rs", "src/de.rs", "src/ser.rs", "src/value/mod.rs"];

    let mut found_any = false;
    for file in &expected_files {
        if results.contains_key(&format!("src/{}", file)) || results.contains_key(file) {
            found_any = true;
            break;
        }
    }

    assert!(found_any, "Should analyze expected source files");

    // Clean up
    temp_dir.close().expect("Failed to clean up temp dir");
}

/// Test the analysis of a file with various Rust language features
#[test]
fn test_language_feature_analysis() {
    let code = r#"
        #![feature(specialization)]
        #![allow(dead_code)]

        use std::fmt::Debug;

        // Generic function
        fn print_debug<T: Debug>(value: &T) {
            println!("{:?}", value);
        }

        // Trait with default implementation
        trait Greet {
            fn greet(&self) {
                println!("Hello!");
            }
        }

        // Struct implementing the trait
        struct Person {
            name: String,
            age: u32,
        }

        impl Greet for Person {
            fn greet(&self) {
                println!("Hello, my name is {} and I'm {} years old.", self.name, self.age);
            }
        }

        // Async function
        async fn fetch_data(url: &str) -> Result<String, reqwest::Error> {
            reqwest::get(url).await?.text().await
        }

        // Macro
        macro_rules! my_vec {
            ($($x:expr),*) => {
                {
                    let mut temp_vec = Vec::new();
                    $(temp_vec.push($x);)*
                    temp_vec
                }
            };
        }

        // Test module
        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn test_greet() {
                let person = Person {
                    name: "Alice".to_string(),
                    age: 30,
                };
                person.greet();
            }
        }

        // Main function
        fn main() {
            let numbers = my_vec![1, 2, 3];
            print_debug(&numbers);

            let person = Person {
                name: "Bob".to_string(),
                age: 25,
            };
            person.greet();

            // Spawn a task
            let handle = std::thread::spawn(move || {
                println!("Hello from another thread!");
            });

            handle.join().unwrap();

            // Use unsafe for FFI (just for testing)
            unsafe {
                let x = 42;
                let ptr = &x as *const i32;
                println!("Pointer value: {:?}", *ptr);
            }
        }
    "#;

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

    // Analyze the code
    let result = pipeline.analyze_code(code, "language_features.rs").unwrap();

    // The code should be valid Rust with no errors
    assert_success(&result);

    // Should find some metrics
    assert!(
        result
            .findings
            .iter()
            .any(|f| f.analysis_type == AnalysisType::CodeMetrics),
        "Should generate code metrics"
    );

    // Should not find any security issues
    assert!(
        !result.findings.iter().any(|f| matches!(
            f.analysis_type,
            AnalysisType::InsecureCrypto
                | AnalysisType::HardcodedSecret
                | AnalysisType::SqlInjection
        )),
        "Should not find security issues in valid code"
    );
}

/// Test the analysis of a file with potential performance issues
#[test]
fn test_performance_analysis() {
    let code = r#"
        use std::collections::HashMap;

        // Inefficient string concatenation in a loop
        fn build_string_inefficient(items: &[&str]) -> String {
            let mut result = String::new();
            for item in items {
                result = result + item; // Creates a new String in each iteration
            }
            result
        }

        // More efficient string building
        fn build_string_efficient(items: &[&str]) -> String {
            let mut result = String::with_capacity(items.iter().map(|s| s.len()).sum());
            for item in items {
                result.push_str(item);
            }
            result
        }

        // Inefficient vector growth
        fn collect_numbers_inefficient(n: usize) -> Vec<usize> {
            let mut result = Vec::new(); // No initial capacity
            for i in 0..n {
                result.push(i);
            }
            result
        }

        // More efficient vector collection
        fn collect_numbers_efficient(n: usize) -> Vec<usize> {
            let mut result = Vec::with_capacity(n); // Pre-allocate capacity
            for i in 0..n {
                result.push(i);
            }
            result
        }

        // Inefficient hash map usage
        fn count_chars_inefficient(s: &str) -> HashMap<char, usize> {
            let mut map = HashMap::new();
            for c in s.chars() {
                // Uses multiple lookups
                let count = map.entry(c).or_insert(0);
                *count += 1;
            }
            map
        }

        // More efficient hash map usage
        fn count_chars_efficient(s: &str) -> HashMap<char, usize> {
            let mut map = HashMap::with_capacity(s.len()); // Over-allocate to avoid resizing
            for c in s.chars() {
                // Single lookup using the entry API
                *map.entry(c).or_insert(0) += 1;
            }
            map
        }

        // Unnecessary clone
        fn process_string(s: String) -> String {
            let s2 = s.clone(); // Unnecessary clone
            s2.to_uppercase()
        }

        // More efficient version
        fn process_string_efficient(s: String) -> String {
            // Take ownership directly
            s.to_uppercase()
        }
    "#;

    // Create a pipeline with performance-focused analyzers
    let pipeline = AnalysisPipelineBuilder::new()
        .with_metrics_analyzer(MetricsAnalyzer::default())
        .build();

    // Analyze the code
    let result = pipeline.analyze_code(code, "performance.rs").unwrap();

    // Should find some performance-related findings
    let performance_findings: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.message.contains("performance") || f.message.contains("inefficient"))
        .collect();

    assert!(
        !performance_findings.is_empty(),
        "Should find performance-related issues"
    );

    // Should find the inefficient string concatenation
    assert!(
        performance_findings
            .iter()
            .any(|f| f.message.contains("string concatenation in a loop")),
        "Should find inefficient string concatenation"
    );

    // Should find the inefficient vector growth
    assert!(
        performance_findings
            .iter()
            .any(|f| f.message.contains("vector growth")),
        "Should find inefficient vector growth"
    );
}

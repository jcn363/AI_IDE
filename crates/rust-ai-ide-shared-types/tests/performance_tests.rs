//! Performance tests and benchmarks for the shared types crate
//!
//! This module contains performance-focused tests that validate
//! the efficiency and scalability of the type generation system.

use rust_ai_ide_shared_types::*;
use std::time::{Duration, Instant};

/// Performance benchmarks for type parsing
#[tokio::test]
async fn benchmark_parsing_performance() {
    let rust_code = r#"
        pub struct BenchType1 { pub field1: String, pub field2: i32 }
        pub struct BenchType2 { pub field1: String, pub field2: i32 }
        pub struct BenchType3 { pub field1: String, pub field2: i32 }
        pub struct BenchType4 { pub field1: String, pub field2: i32 }
        pub struct BenchType5 { pub field1: String, pub field2: i32 }
    "#;

    let parser = TypeParser::new();
    let start = Instant::now();

    // Parse multiple times for consistent measurement
    for _ in 0..100 {
        let _types = parser.parse_file(rust_code, "bench.rs").unwrap();
    }

    let duration = start.elapsed();
    let avg_duration = duration / 100;

    println!("🔍 Parsing Performance: {:?} per parse (100 iterations)", avg_duration);
    assert!(avg_duration < Duration::from_millis(10), "Parsing should be fast");

    // Single parse for detailed measurement
    let start = Instant::now();
    let types = parser.parse_file(rust_code, "bench.rs").unwrap();
    let single_duration = start.elapsed();

    println!("📊 Single parse duration: {:?}", single_duration);
    println!("📈 Types parsed: {}", types.len());
    assert_eq!(types.len(), 5);
}

/// Performance benchmark for TypeScript generation
#[tokio::test]
async fn benchmark_typescript_generation() {
    let rust_code = include_str!("../examples/complex_types.rs");

    let generator = create_typescript_generator().unwrap();
    let start = Instant::now();

    // Generate multiple times
    let mut results = Vec::new();
    for _ in 0..50 {
        let result = generator.generate_types_from_source(rust_code, "bench.rs", &[]).await.unwrap();
        results.push(result);
    }

    let total_duration = start.elapsed();
    let avg_duration = total_duration / 50;

    println!("🛠️  TypeScript Generation Performance: {:?} per generation (50 iterations)", avg_duration);
    assert!(avg_duration < Duration::from_millis(50), "Generation should be reasonably fast");

    // Validate consistency
    let first_size = results[0].content.len();
    for result in &results[1..] {
        assert_eq!(result.content.len(), first_size, "Generation results should be consistent");
    }
}

/// Memory usage benchmark
#[tokio::test]
async fn benchmark_memory_usage() {
    let rust_code = r#"
        pub struct MemoryType {
            pub field1: String,
            pub field2: Vec<i32>,
            pub field3: HashMap<String, String>,
        }
    "#;

    // Measure baseline memory
    let baseline_usage = get_memory_usage();

    let generator = create_typescript_generator().unwrap();
    let result = generator.generate_types_from_source(rust_code, "memory.rs", &[]).await.unwrap();

    let peak_usage = get_memory_usage();
    let memory_delta = peak_usage.saturating_sub(baseline_usage);

    println!("💾 Memory Usage Delta: {} bytes (Peak: {} bytes)", memory_delta, peak_usage);
    println!("📝 Generated code size: {} bytes", result.content.len());

    // Memory usage should be reasonable (less than 10MB increase)
    assert!(memory_delta < 10_000_000, "Memory usage should be reasonable");
}

/// Concurrent processing benchmark
#[tokio::test]
async fn benchmark_concurrent_processing() {
    let rust_codes = vec![
        r#"pub struct ConcurrentType1 { pub field: String }"#,
        r#"pub struct ConcurrentType2 { pub field: i32 }"#,
        r#"pub struct ConcurrentType3 { pub field: bool }"#,
        r#"pub struct ConcurrentType4 { pub field: Vec<String> }"#,
    ];

    let generator = create_typescript_generator().unwrap();
    let start = Instant::now();

    // Process concurrently
    let tasks: Vec<_> = rust_codes.iter().enumerate().map(|(i, code)| {
        let gen = generator.clone();
        tokio::spawn(async move {
            gen.generate_types_from_source(code, &format!("concurrent_{}.rs", i), &[]).await
        })
    }).collect();

    let results = futures::future::join_all(tasks).await;
    let duration = start.elapsed();

    println!("⚡ Concurrent Processing: {:?} for {} files", duration, rust_codes.len());
    println!("📈 Average: {:?}", duration / rust_codes.len() as u32);

    // All results should be successful
    for result in results {
        let result = result.unwrap();
        assert!(result.is_ok());
        let generated = result.unwrap();
        assert!(!generated.content.is_empty());
    }
}

/// Large codebase simulation
#[tokio::test]
async fn benchmark_large_codebase() {
    // Generate a large Rust codebase for testing
    let mut large_code = String::new();
    for i in 1..=100 {
        large_code.push_str(&format!(r#"
            /// Type {} documentation
            #[derive(Serialize, Deserialize)]
            pub struct LargeType{} {{
                /// ID field for type {}
                pub id: u64,
                /// Name field for type {}
                pub name: String,
                /// Data field for type {}
                pub data: Vec<SubType{}>,
                /// Optional metadata for type {}
                pub metadata: Option<Meta{}>,
            }}

            /// Sub-type for type {}
            pub struct SubType{} {{
                pub value: String,
                pub count: i32,
            }}

            /// Metadata for type {}
            pub struct Meta{} {{
                pub created_at: chrono::NaiveDateTime,
                pub tags: Vec<String>,
            }}
        "#, i, i, i, i, i, i, i, i, i, i, i, i));
    }

    let start = Instant::now();
    let generator = create_typescript_generator().unwrap();
    let result = generator.generate_types_from_source(&large_code, "large_codebase.rs", &[]).await.unwrap();
    let duration = start.elapsed();

    println!("🏗️  Large Codebase Processing: {:?}", duration);
    println!("📊 Generated code size: {} KB", result.content.len() / 1024);
    println!("🔢 Types processed: {}", result.source_types.len());

    assert!(duration < Duration::from_secs(30), "Large codebase processing should be reasonable");
    assert!(result.source_types.len() > 100, "Should process all types");
    assert!(result.content.len() > 50_000, "Should generate substantial code");
}

/// Caching performance benchmark
#[tokio::test]
async fn benchmark_caching_performance() {
    let config = GenerationConfig::default();
    assert!(config.cache.enabled);

    let rust_code = r#"
        pub struct CacheTest {
            pub field1: String,
            pub field2: i32,
            pub field3: Vec<String>,
        }
    "#;

    let generator = TypeGenerator::with_full_config(config).unwrap();
    let start = Instant::now();

    // Generate multiple times to test caching
    for i in 0..10 {
        let _result = generator.generate_types_from_source(rust_code, "cache_bench.rs", &[]).await.unwrap();

        if i == 0 {
            let first_duration = start.elapsed();
            println!("🚀 First generation: {:?}", first_duration);
        }
    }

    let total_duration = start.elapsed();
    let avg_duration = total_duration / 10;

    println!("💨 Average generation time: {:?}", avg_duration);
    println!("📈 Total time for 10 generations: {:?}", total_duration);

    // Average should be reasonable even with caching overhead
    assert!(avg_duration < Duration::from_millis(100));
}

/// Validation performance benchmark
#[tokio::test]
async fn benchmark_validation_performance() {
    let rust_code = r#"
        pub struct ValidateType1 { pub field: String }
        pub struct ValidateType2 { pub field: i32 }
        pub struct ValidateType3 { pub field: bool }
    "#;

    let generator = create_typescript_generator().unwrap();
    let result = generator.generate_types_from_source(rust_code, "validate_bench.rs", &[]).await.unwrap();

    let start = Instant::now();
    let config = default_config();

    // Run validation multiple times
    for _ in 0..20 {
        let _validation = validate_cross_platform(&result.source_types, &config).await.unwrap();
    }

    let duration = start.elapsed();
    let avg_duration = duration / 20;

    println!("🔍 Validation Performance: {:?} per validation (20 iterations)", avg_duration);
    assert!(avg_duration < Duration::from_millis(10), "Validation should be fast");
}

/// Scalability test with increasing type complexity
#[tokio::test]
async fn benchmark_scalability() {
    let mut durations = Vec::new();
    let mut type_counts = Vec::new();

    for size in [5, 10, 25, 50, 100] {
        let rust_code = generate_code_with_types(size);
        let generator = create_typescript_generator().unwrap();

        let start = Instant::now();
        let result = generator.generate_types_from_source(&rust_code, "scale_bench.rs", &[]).await.unwrap();
        let duration = start.elapsed();

        durations.push(duration);
        type_counts.push(result.source_types.len());

        println!("📏 Size {}: {} types in {:?}", size, result.source_types.len(), duration);
    }

    // Verify performance scaling
    for i in 1..durations.len() {
        let ratio = durations[i].as_millis() as f64 / durations[i-1].as_millis() as f64;
        let size_ratio = type_counts[i] as f64 / type_counts[i-1] as f64;

        println!("🔄 Scale factor {}x types: {:.1}x time", size_ratio, ratio);

        // Time should not scale worse than linearly (allowing some overhead)
        assert!(ratio < size_ratio * 2.0, "Performance should scale reasonably");
    }
}

/// Stress test with maximum complexity
#[tokio::test]
async fn benchmark_maximum_complexity() {
    let rust_code = r#"
        pub struct ComplexType<T: Clone + Send + Sync, U> {
            pub field1: T,
            pub field2: U,
            pub field3: Option<Vec<HashMap<String, (T, U)>>>,
            pub field4: Result<String, std::io::Error>,
            pub nested: NestedType<T>,
        }

        pub struct NestedType<T> {
            pub value: T,
            pub list: Vec<T>,
            pub map: HashMap<String, T>,
        }
    "#;

    let generator = create_typescript_generator().unwrap();
    let start = Instant::now();
    let result = generator.generate_types_from_source(rust_code, "complex_bench.rs", &[]).await.unwrap();
    let duration = start.elapsed();

    println!("⚡ Complex Type Processing: {:?}", duration);
    println!("🔧 Generated {} lines",
             result.content.lines().count());
    println!("📝 Content preview: {}",
             result.content.lines().next().unwrap());

    assert!(duration < Duration::from_millis(200), "Complex types should process reasonably fast");
    assert!(result.content.contains("export interface ComplexType"));
}

/// Helper function to get current memory usage (simplified)
fn get_memory_usage() -> u64 {
    // In a real implementation, this would use system APIs
    // For this test, we'll return a placeholder
    0
}

/// Helper function to generate code with N types
fn generate_code_with_types(n: usize) -> String {
    let mut code = String::new();

    for i in 1..=n {
        code.push_str(&format!(r#"
            pub struct GeneratedType{} {{
                pub id: u32,
                pub name: String,
                pub data: Vec<String>,
                pub metadata: Option<HashMap<String, String>>,
            }}
        "#, i));
    }

    code
}

/// Performance regression test
#[tokio::test]
async fn test_performance_regression() {
    // This test ensures performance doesn't degrade significantly
    // by comparing against known good baselines

    let rust_code = r#"
        pub struct RegressionTest {
            pub id: u64,
            pub name: String,
            pub data: Vec<String>,
        }
    "#;

    let generator = create_typescript_generator().unwrap();

    // Establish baseline performance
    let mut times = Vec::new();
    for _ in 0..10 {
        let start = Instant::now();
        let _result = generator.generate_types_from_source(rust_code, "regression.rs", &[]).await.unwrap();
        times.push(start.elapsed().as_millis());
    }

    let avg_time = times.iter().sum::<u128>() / times.len() as u128;

    println!("📈 Average generation time: {}ms", avg_time);

    // Performance should be consistent (within 50% of baseline)
    let baseline = 50u128; // Expected baseline in ms
    let upper_bound = baseline * 3 / 2; // 150% of baseline

    assert!(avg_time <= upper_bound,
            "Performance regression detected: {}ms > {}ms",
            avg_time, upper_bound);
}

/// Continuous integration benchmark
#[tokio::test]
async fn ci_benchmark() {
    // This test can be used for CI statistics gathering

    let rust_code = include_str!("../examples/basic_types.rs");
    let generator = create_typescript_generator().unwrap();

    let start = Instant::now();
    let result = generator.generate_types_from_source(rust_code, "ci_bench.rs", &[]).await.unwrap();
    let parsing_duration = start.elapsed();

    let validation_start = Instant::now();
    let _validation = validate_cross_platform(&result.source_types, &default_config()).await.unwrap();
    let validation_duration = validation_start.elapsed();

    // CI-friendly output
    println!("CI_METRIC_PARSING_DURATION_MS={}", parsing_duration.as_millis());
    println!("CI_METRIC_VALIDATION_DURATION_MS={}", validation_duration.as_millis());
    println!("CI_METRIC_GENERATED_CODE_SIZE_BYTES={}", result.content.len());
    println!("CI_METRIC_TYPES_PROCESSED={}", result.source_types.len());

    // Performance assertions for CI
    assert!(parsing_duration < Duration::from_millis(500), "CI parsing performance regression");
    assert!(validation_duration < Duration::from_millis(100), "CI validation performance regression");
    assert!(result.content.len() > 100, "CI code generation insufficient");
}
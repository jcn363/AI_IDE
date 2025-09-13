//! Tests for code quality metrics analysis

use rust_ai_ide_ai::{
    analysis::{
        metrics::{
            CognitiveComplexityCalculator, CyclomaticComplexityCalculator, HalsteadMetrics,
            MaintainabilityIndex, MetricsAnalyzer, MetricsCollector, SourceLinesOfCode,
        },
        AnalysisRegistry, AnalysisResult, AnalysisType, Severity,
    },
    test_helpers::*,
};

/// Test cyclomatic complexity calculation
#[test]
fn test_cyclomatic_complexity() {
    let code = r#"
        fn simple_function(x: i32) -> i32 { // +1
            if x > 0 {                     // +1
                return 42;
            } else if x < 0 {              // +1
                return -1;
            }

            match x {                      // +1 (for the match)
                0 => 0,                    // +1 (for the first arm)
                1 => 1,                    // +1 (for the second arm)
                _ => x * 2,                // +1 (for the wildcard arm)
            }
        }

        fn another_function() {            // +1
            for i in 0..10 {               // +1 (for the loop)
                if i % 2 == 0 {            // +1
                    println!("Even");
                } else {                    // +1
                    println!("Odd");
                }
            }
        }
    "#;

    let ast = create_test_ast(code);
    let mut collector = MetricsCollector::default();
    collector.visit_file(&ast);

    // Get metrics for the first function
    let simple_func_metrics = collector
        .get_function_metrics("simple_function")
        .expect("Function metrics not found");

    // simple_function should have a cyclomatic complexity of 6
    // (1 base + 1 if + 1 else if + 1 match + 3 match arms)
    assert_eq!(
        simple_func_metrics.cyclomatic_complexity, 6,
        "Incorrect cyclomatic complexity for simple_function"
    );

    // another_function should have a cyclomatic complexity of 4
    // (1 base + 1 for loop + 1 if + 1 else)
    let another_func_metrics = collector
        .get_function_metrics("another_function")
        .expect("Function metrics not found");

    assert_eq!(
        another_func_metrics.cyclomatic_complexity, 4,
        "Incorrect cyclomatic complexity for another_function"
    );
}

/// Test cognitive complexity calculation
#[test]
fn test_cognitive_complexity() {
    let code = r#"
        fn complex_function(x: i32, y: i32) -> i32 { // +1
            let mut result = 0;

            if x > 0 {                               // +1 (if)
                result += 1;

                if y > 0 {                           // +2 (nested if)
                    result += 1;
                }
            }

            match x {                                // +1 (match)
                0 => result = 0,                     // +1 (match arm)
                1 => {                               // +1 (match arm)
                    result = 1;
                    if y > 0 { result += 1; }        // +2 (nested if in match arm)
                }
                _ => result = x,                     // +1 (match arm)
            }

            for i in 0..x {                         // +1 (for loop)
                result += i;

                while result > 10 {                  // +2 (nested while)
                    result -= 1;
                }
            }

            result
        }
    "#;

    let ast = create_test_ast(code);
    let mut collector = MetricsCollector::default();
    collector.visit_file(&ast);

    let metrics = collector
        .get_function_metrics("complex_function")
        .expect("Function metrics not found");

    // Expected cognitive complexity: 1 (base) + 1 (if) + 2 (nested if) + 1 (match) + 3 (match arms) + 1 (for) + 2 (nested while) = 11
    assert_eq!(
        metrics.cognitive_complexity, 11,
        "Incorrect cognitive complexity calculation"
    );
}

/// Test Halstead metrics calculation
#[test]
fn test_halstead_metrics() {
    let code = r#"
        fn calculate(x: i32, y: i32) -> i32 { // 2 distinct operators (fn, ->), 3 operands (calculate, x, y)
            let a = x + y;                    // 2 operators (=, +), 2 operands (x, y)
            let b = a * 2;                    // 2 operators (=, *), 2 operands (a, 2)
            if b > 10 {                       // 1 operator (>), 2 operands (b, 10)
                return b - 5;                 // 2 operators (return, -), 2 operands (b, 5)
            }
            b                                 // 0 operators, 1 operand (b)
        }
    "#;

    let ast = create_test_ast(code);
    let mut collector = MetricsCollector::default();
    collector.visit_file(&ast);

    let metrics = collector
        .get_function_metrics("calculate")
        .expect("Function metrics not found");

    // Operators: fn, ->, =, +, =, *, >, return, -
    // Operands: calculate, x, y, a, b, 2, 10, 5, b
    let halstead = &metrics.halstead_metrics;

    assert_eq!(
        halstead.unique_operators, 5,
        "Incorrect number of unique operators"
    );
    assert_eq!(
        halstead.unique_operands, 6,
        "Incorrect number of unique operands"
    );
    assert_eq!(halstead.total_operators, 9, "Incorrect total operators");
    assert_eq!(halstead.total_operands, 10, "Incorrect total operands");

    // Program length = total operators + total operands
    assert_eq!(halstead.program_length(), 19, "Incorrect program length");

    // Program vocabulary = unique operators + unique operands
    assert_eq!(
        halstead.program_vocabulary(),
        11,
        "Incorrect program vocabulary"
    );

    // Volume = program_length * log2(program_vocabulary)
    assert!(
        (halstead.volume() - 66.0).abs() < 0.1, // 19 * log2(11) ≈ 66.0
        "Incorrect volume calculation"
    );
}

/// Test maintainability index calculation
#[test]
fn test_maintainability_index() {
    // This function has moderate complexity
    let code = r#"
        fn calculate(x: i32, y: i32) -> i32 {
            let mut result = 0;

            if x > 0 {
                result += x;

                if y > 0 {
                    result += y;
                }
            }

            result
        }
    "#;

    let ast = create_test_ast(code);
    let mut collector = MetricsCollector::default();
    collector.visit_file(&ast);

    let metrics = collector
        .get_function_metrics("calculate")
        .expect("Function metrics not found");

    // MI = 171 - 5.2 * ln(HV) - 0.23 * CC - 16.2 * ln(LOC)
    // Where HV is Halstead Volume, CC is Cyclomatic Complexity, and LOC is Lines of Code
    // The exact value depends on the implementation details, but it should be in a reasonable range
    assert!(
        metrics.maintainability_index > 50.0 && metrics.maintainability_index < 120.0,
        "Maintainability index {} is outside expected range",
        metrics.maintainability_index
    );
}

/// Test SLOC (Source Lines of Code) calculation
#[test]
fn test_source_lines_of_code() {
    let code = r#"/* This is a comment */

// Another comment

fn empty_function() {
    // Does nothing
}

fn function_with_body() {
    let x = 42;
    let y = x * 2;

    if y > 10 {
        println!("Y is greater than 10");
    }
}

struct TestStruct {
    field1: i32,
    field2: String,
}

impl TestStruct {
    fn new() -> Self {
        Self {
            field1: 0,
            field2: String::new(),
        }
    }
}"#;

    let ast = create_test_ast(code);
    let sloc = SourceLinesOfCode::calculate(&ast);

    // Count non-empty, non-comment lines
    assert_eq!(sloc.physical, 20, "Incorrect physical SLOC");
    // Count lines containing actual code (excluding comments and whitespace)
    assert!(
        sloc.logical >= 10 && sloc.logical <= 15,
        "Incorrect logical SLOC"
    );
}

/// Test the metrics analyzer integration
#[test]
fn test_metrics_analyzer_integration() {
    let code = r#"
        fn complex_function(x: i32) -> i32 {
            let mut result = 0;

            for i in 0..x {
                if i % 2 == 0 {
                    result += i;
                } else {
                    result -= i;
                }
            }

            result
        }
    "#;

    let mut registry = AnalysisRegistry::default();
    registry.register_metrics_analyzer(MetricsAnalyzer::default());

    let result = registry.analyze_code(code, "metrics_test.rs").unwrap();

    // Should generate metrics findings
    assert!(
        result
            .findings
            .iter()
            .any(|f| f.analysis_type == AnalysisType::CodeMetrics),
        "Expected to find code metrics in the analysis results"
    );

    // Check for specific metrics in the findings
    let metrics_findings: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.analysis_type == AnalysisType::CodeMetrics)
        .collect();

    assert!(!metrics_findings.is_empty(), "No code metrics findings");

    // Check that we have metrics for the function
    let func_metrics = metrics_findings
        .iter()
        .find(|f| f.message.contains("complex_function"));

    assert!(
        func_metrics.is_some(),
        "No metrics found for complex_function"
    );

    // The function should have a cyclomatic complexity > 1
    assert!(
        func_metrics
            .unwrap()
            .message
            .contains("Cyclomatic Complexity: "),
        "Cyclomatic complexity not found in metrics"
    );
}

/// Test that metrics analyzer flags high complexity functions
#[test]
fn test_high_complexity_detection() {
    let code = r#"
        // This function has high cyclomatic complexity
        fn very_complex_function(x: i32) -> i32 {
            let mut result = 0;

            if x > 0 { result += 1; }
            if x > 1 { result += 1; }
            if x > 2 { result += 1; }
            if x > 3 { result += 1; }
            if x > 4 { result += 1; }
            if x > 5 { result += 1; }
            if x > 6 { result += 1; }
            if x > 7 { result += 1; }
            if x > 8 { result += 1; }
            if x > 9 { result += 1; }

            result
        }
    "#;

    let mut registry = AnalysisRegistry::default();
    registry.register_metrics_analyzer(MetricsAnalyzer::default());

    let result = registry
        .analyze_code(code, "high_complexity_test.rs")
        .unwrap();

    // Should flag the high complexity function
    assert_finding!(
        &result,
        AnalysisType::CodeMetrics,
        Severity::Warning,
        "High cyclomatic complexity detected"
    );

    assert_finding!(
        &result,
        AnalysisType::CodeMetrics,
        Severity::Warning,
        "very_complex_function"
    );
}

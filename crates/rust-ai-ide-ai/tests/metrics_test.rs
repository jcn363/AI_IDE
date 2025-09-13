use rust_ai_ide_ai::analysis::metrics::{CodeMetrics, MetricsCalculator};
use syn::parse_quote;

#[test]
fn test_metrics_calculation() {
    let code = r#"
    // This is a test function
    fn test_function(x: i32) -> i32 {
        if x > 0 {
            x * 2
        } else {
            0
        }
    }

    #[test]
    fn test_test_function() {
        assert_eq!(test_function(2), 4);
        assert_eq!(test_function(-1), 0);
    }
    "#;

    let ast = syn::parse_file(code).unwrap();
    let calculator = MetricsCalculator::new();
    let metrics = calculator.calculate(&ast, code, "test.rs");

    // Basic metrics should be calculated
    assert!(
        metrics.cyclomatic_complexity > 0,
        "Cyclomatic complexity should be > 0"
    );
    assert!(metrics.lines_of_code > 0, "Lines of code should be > 0");
    assert!(metrics.comment_ratio > 0.0, "Comment ratio should be > 0");

    // Maintainability index should be in a reasonable range (0-100)
    assert!(
        metrics.maintainability_index > 0.0 && metrics.maintainability_index <= 100.0,
        "Maintainability index should be between 0 and 100"
    );

    // Technical debt ratio should be >= 0
    assert!(
        metrics.technical_debt_ratio >= 0.0,
        "Technical debt ratio should be >= 0"
    );

    // Security and performance scores should be between 0 and 1
    assert!(
        metrics.security_score >= 0.0 && metrics.security_score <= 1.0,
        "Security score should be between 0 and 1"
    );

    assert!(
        metrics.performance_score >= 0.0 && metrics.performance_score <= 1.0,
        "Performance score should be between 0 and 1"
    );

    // Duplication ratio should be between 0 and 100
    assert!(
        metrics.duplication_ratio >= 0.0 && metrics.duplication_ratio <= 100.0,
        "Duplication ratio should be between 0 and 100"
    );
}

#[test]
fn test_empty_file_metrics() {
    let code = "";
    let ast = syn::parse_file(code).unwrap();
    let calculator = MetricsCalculator::new();
    let metrics = calculator.calculate(&ast, code, "empty.rs");

    // Empty file should have 0 for most metrics
    assert_eq!(metrics.cyclomatic_complexity, 0);
    assert_eq!(metrics.lines_of_code, 0);
    assert_eq!(metrics.comment_ratio, 0.0);
    assert_eq!(metrics.duplication_ratio, 0.0);
}

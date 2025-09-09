# Test Organization

This directory contains all the tests for the Rust AI IDE analysis engine. The tests are organized by category and follow Rust's testing conventions.

## Directory Structure

```text
tests/
├── architectural/             # Tests for architectural analysis
│   ├── mod.rs                # Main module file
│   ├── circular_dependencies.rs
│   ├── dependency_inversion.rs
│   ├── interface_segregation.rs
│   ├── layer_violations.rs
│   └── helpers.rs             # Test helpers specific to architectural tests
│
├── security/                  # Tests for security analysis
│   └── analysis_tests.rs      # Security vulnerability detection tests
│
├── quality/                   # Tests for code quality metrics
│   └── metrics_tests.rs       # Code metrics and quality analysis tests
│
├── performance/               # Performance benchmarks
│   └── analysis_tests.rs      # Performance tests for analyzers
│
├── integration/               # Integration and end-to-end tests
│   └── end_to_end_tests.rs    # Tests for complete analysis pipeline
│
└── test_helpers/             # Global test utilities and helpers
    └── mod.rs                # Re-export of test helpers
```

## Test Categories

### 1. Architectural Tests

Tests for architectural rule validations:

- Circular dependency detection
- Layer violations
- Dependency inversion principle
- Interface segregation principle

### 2. Security Tests

Tests for security vulnerability detection:

- Insecure cryptographic functions
- Hardcoded secrets
- SQL injection vulnerabilities
- Other security anti-patterns

### 3. Quality Metrics Tests

Tests for code quality metrics:

- Cyclomatic complexity
- Cognitive complexity
- Halstead metrics
- Maintainability index
- Source lines of code (SLOC)

### 4. Performance Tests

Benchmarks for analysis performance:

- Individual analyzer performance
- End-to-end analysis pipeline
- Memory usage
- Throughput

### 5. Integration Tests

End-to-end tests that verify the complete analysis pipeline:

- Full project analysis
- Multiple analyzers working together
- Real-world code patterns
- Error handling and edge cases

## Writing Tests

### Test Naming Conventions

- **Test modules**: `*_tests.rs` or `mod.rs` in test directories
- **Test functions**:
  - `test_*` for unit and integration tests
  - `bench_*` for benchmarks
  - `should_*` for behavior-driven test names (e.g., `should_detect_circular_dependency`)
- **Test files**: Group related functionality together (e.g., `circular_dependencies.rs`)
- **Benchmark files**: Place in the `benches/` directory with `_benchmark.rs` suffix

### Test Structure

A typical test module should follow this structure:

```rust
//! Tests for [feature being tested]

use super::*;
use crate::test_helpers::*;

#[test]
fn test_feature_scenario() {
    // 1. Setup test data and environment
    let input = "test input";
    
    // 2. Execute the code being tested
    let result = analyze_code(input, "test.rs").unwrap();
    
    // 3. Verify the results
    assert_success(&result);
    assert_finding!(
        &result, 
        AnalysisType::Feature, 
        Severity::Warning, 
        "Expected message"
    );
}
```

### Test Helpers

Use the test helpers in `test_helpers/mod.rs` for common test functionality:

```rust
use crate::test_helpers::*;

#[test]
fn test_example() {
    // Create a test AST from code
    let ast = create_test_ast("fn example() {}");
    
    // Run analysis
    let result = analyze_ast(&ast, "test.rs").unwrap();
    
    // Verify results
    assert_success(&result);
    assert_finding!(
        &result, 
        AnalysisType::Example, 
        Severity::Warning, 
        "Expected message"
    );
}
```

### Assertion Macros and Helpers

Use these assertion macros for better test failure messages:

- `assert_success(result)` - Asserts the analysis completed without errors
- `assert_failure(result, expected_error)` - Asserts the analysis failed with a specific error
- `assert_finding!(result, type, severity, message)` - Asserts a specific finding exists
- `assert_no_finding!(result, type)` - Asserts no findings of a specific type exist
- `assert_finding_count(result, count)` - Asserts the total number of findings

### Testing Best Practices

1. **Isolation**: Each test should be independent and not rely on the state from other tests.
2. **Readability**: Write clear, descriptive test names and include comments explaining the test's purpose.
3. **Coverage**: Test both positive and negative cases, including edge cases and error conditions.
4. **Performance**: Keep tests fast by using small, focused test cases and avoiding unnecessary I/O.
5. **Maintainability**: Use helper functions to reduce code duplication and improve test clarity.

## Example Tests

### Unit Test Example

```rust
#[test]
fn test_detects_circular_dependency() {
    // Arrange
    let code = r#"
        mod a { pub fn a() { b::b(); } }
        mod b { pub fn b() { a::a(); } }
    "#;
    
    // Act
    let result = analyze_code(code, "test.rs").unwrap();
    
    // Assert
    assert_finding!(
        &result,
        AnalysisType::CircularDependency,
        Severity::Warning,
        "Circular dependency detected between modules 'a' and 'b'"
    );
}
```

### Integration Test Example

```rust
#[test]
fn test_end_to_end_analysis() {
    // Create a temporary directory for the test project
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Set up test files
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    
    // Create a simple Rust project
    std::fs::write(
        src_dir.join("lib.rs"),
        r#"
            pub mod a;
            pub mod b;
            
            pub fn main() {
                a::a();
            }
        "#,
    ).unwrap();
    
    // Create module files...
    
    // Run the analysis
    let pipeline = create_analysis_pipeline();
    let results = pipeline.analyze_project(temp_dir.path()).unwrap();
    
    // Verify the results
    assert!(!results.is_empty());
    // More assertions...
}
```

### Benchmark Example

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_analyzer(c: &mut Criterion) {
    let code = "fn example() {}";
    
    c.bench_function("analyze_small_function", |b| {
        b.iter(|| {
            let _ = analyze_code(black_box(code), "benchmark.rs");
        })
    });
}

criterion_group!(benches, benchmark_analyzer);
criterion_main!(benches);
```

## Running Tests

Run all tests:

```bash
cargo test
```

Run specific test categories:

```bash
# Run only architectural tests
cargo test --test architectural

# Run only security tests
cargo test --test security

# Run only metrics tests
cargo test --test metrics
```

Run benchmarks (requires `--release` flag):

```bash
cargo bench --features="benchmarks"
```

## Test Fixtures

For larger test cases, use the `test_fixtures` directory at the workspace root:

```
test-fixtures/
├── projects/          # Complete test projects
├── files/             # Individual test files
└── expected/          # Expected outputs
```

Use the `test_file_path()` helper to reference test fixtures:

```rust
let path = test_file_path("projects/example", "src/main.rs");
```

## Best Practices

1. **Isolation**: Each test should be independent and not rely on shared state
2. **Descriptive Names**: Use clear, descriptive test names
3. **Minimal Fixtures**: Use the smallest possible test cases
4. **Assert Precisely**: Test one thing per test case
5. **Document Edge Cases**: Add comments for non-obvious test cases
6. **Performance**: Be mindful of test execution time

## Debugging Tests

For debugging, use the standard Rust test output flags:

```bash
# Show output from passing tests
cargo test -- --nocapture

# Run a specific test
cargo test test_name -- --nocapture

# Run with detailed output
cargo test -- --nocapture --test-threads=1
```

## Coverage

To generate test coverage reports:

```bash
# Install cargo-tarpaulin if needed
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --ignore-tests
```

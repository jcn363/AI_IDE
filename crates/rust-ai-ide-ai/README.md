# Rust AI IDE - Analysis Engine

[![Crates.io](https://img.shields.io/crates/v/rust-ai-ide-ai)](https://crates.io/crates/rust-ai-ide-ai)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://github.com/yourusername/rust-ai-ide/actions/workflows/rust.yml/badge.svg)](https://github.com/yourusername/rust-ai-ide/actions)

Advanced static analysis for Rust codebases, powering the Rust AI IDE's intelligent code insights and suggestions.

## Features

- **Code Quality Analysis**
  - Detect code smells and anti-patterns
  - Enforce coding standards
  - Identify complex or duplicated code

- **Performance Analysis**
  - Find potential performance bottlenecks
  - Suggest optimizations
  - Memory usage analysis

- **Security Analysis**
  - Vulnerability detection
  - Security anti-patterns
  - Safe/unsafe code analysis

- **Architectural Analysis**
  - Circular dependency detection
  - Layer violation detection
  - Interface segregation analysis
  - Dependency inversion analysis

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-ai-ide-ai = { version = "0.1.0", features = ["full"] }
```

Basic example:

```rust
use rust_ai_ide_ai::{analyze_code, AnalysisConfig};
use syn::parse_file;

let config = AnalysisConfig::default()
    .with_security(true)
    .with_performance(true);

let code = r#"
    fn main() {
        let x = 5;
        println!("{}", x);
    }
"#;

let ast = syn::parse_file(code).unwrap();
let results = analyze_code(&ast, "example.rs", &config);

for finding in results {
    println!("{}: {}", finding.severity, finding.message);
}
```

## Configuration

Create a `.rust-ai-ide.toml` in your project root:

```toml
[analysis]
# Enable/disable analyzers
security = true
performance = true
architecture = true
style = true

# Customize rules
[analysis.rules]
"complexity.max_function_length" = 30
"security.allow_unsafe" = false

[architecture]
# Define module layers and allowed dependencies
[[architecture.layers]]
name = "domain"
dependencies = []

[[architecture.layers]]
name = "infrastructure"
dependencies = ["domain"]
```

## API Reference

### Core Types

- `AnalysisFinding`: Result of analysis with details about the issue
- `AnalysisConfig`: Configuration for the analysis process
- `Severity`: Issue severity level (Error, Warning, Info, Hint)
- `AnalysisCategory`: Type of analysis performed

### Main Functions

- `analyze_code(ast: &syn::File, path: &str, config: &AnalysisConfig) -> Vec<AnalysisFinding>`
  - Main entry point for code analysis

- `analyze_security(ast: &syn::File, config: &SecurityConfig) -> Vec<AnalysisFinding>`
  - Run security-specific analysis

- `analyze_performance(ast: &syn::File) -> Vec<AnalysisFinding>`
  - Run performance-specific analysis

- `analyze_architecture(ast: &syn::File, config: &ArchitectureConfig) -> Vec<AnalysisFinding>`
  - Run architecture-specific analysis

## Extending

### Adding New Analyzers

1. Implement the `Analyzer` trait:

```rust
pub trait Analyzer {
    fn analyze(&self, ast: &syn::File) -> Vec<AnalysisFinding>;
    fn name(&self) -> &'static str;
}
```

2. Register your analyzer in the main analysis pipeline.

## Benchmarks

Run benchmarks with:

```bash
cargo bench
```

Current performance:
- Small file (<100 LOC): <10ms
- Medium project (~10k LOC): ~500ms
- Large project (100k+ LOC): ~5s (with caching)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for your changes
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

## License

MIT

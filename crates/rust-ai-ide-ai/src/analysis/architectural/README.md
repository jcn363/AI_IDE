# Architectural Analysis Module

This module provides advanced static analysis for Rust code to detect architectural issues, code smells, and potential design problems. It's designed to help maintain clean architecture and good design practices.

## Features

- **Cyclomatic Complexity Analysis**: Identifies overly complex methods and implementations
- **Dependency Inversion**: Detects violations of the Dependency Inversion Principle
- **Interface Segregation**: Ensures interfaces are small and focused
- **Module Size Validation**: Warns about modules that are too large
- **Public API Surface Analysis**: Helps maintain a clean and focused public API
- **Layer Dependency Validation**: Ensures architectural layer dependencies are respected

## Usage

```rust
use rust_ai_ide_ai::analysis::architectural::ArchitecturalAnalyzer;

// Create a new analyzer with default settings
let analyzer = ArchitecturalAnalyzer::with_all_checks()
    .with_max_cyclomatic_complexity(10)  // Set max complexity threshold
    .with_max_trait_methods(5)          // Set max methods per trait
    .with_max_module_size(1000)          // Set max lines per module
    .with_max_public_items(20);          // Set max public items per module

// Analyze a Rust file
let ast = syn::parse_file(r#"
    pub struct Example {
        value: i32,
    }

    impl Example {
        pub fn new() -> Self {
            Self { value: 42 }
        }
    }
"#).unwrap();

let findings = analyzer.analyze(&ast, "example.rs", "");
```

## Configuration

The analyzer is highly configurable. Here are the main configuration options:

- `with_circular_dep_checking`: Enable/disable circular dependency detection
- `with_dependency_inversion_checking`: Enable/disable dependency inversion checks
- `with_interface_segregation_checking`: Enable/disable interface segregation checks
- `with_max_cyclomatic_complexity`: Set maximum allowed cyclomatic complexity
- `with_max_trait_methods`: Set maximum methods per trait/implementation
- `with_max_module_size`: Set maximum lines per module
- `with_max_public_items`: Set maximum public items per module
- `with_layer_dependency_enforcement`: Enable/disable layer dependency validation
- `with_allowed_layer`: Add an allowed architectural layer
- `with_allowed_concrete_dependency`: Allow specific concrete dependencies

## Architecture

The module is organized into several components:

- **Visitors**: Traverse the AST to collect metrics and detect issues
  - `ComplexityVisitor`: Analyzes code complexity
  - `DependencyVisitor`: Checks dependency inversion violations
  - `InterfaceVisitor`: Validates interface segregation

- **Graph**: Manages dependency relationships between modules and items
  - `DependencyGraph`: Tracks dependencies and detects cycles
  - `DependencyNode`: Represents a node in the dependency graph

## Adding New Checks

To add a new architectural check:

1. Create a new visitor in the `visitors` module
2. Implement the `ArchitecturalVisitor` trait
3. Add configuration options to `ArchitecturalAnalyzer`
4. Update the main `analyze` method to use your visitor

## Testing

Run the tests with:

```bash
cargo test -p rust-ai-ide-ai --test architectural
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

# Rust AI IDE - Architectural Analysis Module

## Overview

The Architectural Analysis module provides advanced static analysis capabilities to identify and help resolve architectural issues in Rust codebases. It's designed to help developers maintain clean, maintainable, and efficient code by detecting potential problems early in the development cycle.

## Features

### 1. Circular Dependency Detection

- Identifies circular dependencies between modules
- Calculates cycle complexity metrics
- Provides actionable suggestions for breaking cycles
- Generates visual dependency graphs (DOT format)
- Detects potential cycles before they become problems

### 2. Layer Violation Detection

- Enforces architectural layer boundaries
- Configurable layer definitions
- Custom rule support for layer access patterns
- Visual feedback on layer violations

### 3. Interface Segregation Analysis

- Identifies large interfaces that could be split
- Suggests more focused, cohesive interfaces
- Analyzes trait cohesion
- Provides metrics on interface usage

### 4. Dependency Inversion Analysis

- Detects violations of the Dependency Inversion Principle
- Identifies high-level modules depending on low-level details
- Suggests abstraction points
- Analyzes constructor and field dependencies

## Usage

### Running Analysis

```rust
let analyzer = ArchitecturalAnalyzer::new();
let findings = analyzer.analyze(&ast, file_path);
```

### Configuration

Create a `.rust-ai-ide.toml` file in your project root:

```toml
[architectural]
# Enable/disable specific analyzers
enable_circular_detection = true
enable_layer_violations = true

[layer_rules]
# Define allowed layer dependencies
"domain" = []
"infrastructure" = ["domain"]
"application" = ["domain"]
"presentation" = ["application"]
```

## Output Format

Analysis results are returned as a vector of `AnalysisFinding`:

```rust
pub struct AnalysisFinding {
    pub message: String,
    pub severity: Severity,
    pub category: AnalysisCategory,
    pub range: Range,
    pub suggestion: Option<String>,
    pub confidence: f32,
    pub rule_id: String,
}
```

## Integration

The module integrates with the main IDE through:

1. **Code Editor**
   - Inline annotations for issues
   - Quick-fix suggestions
   - CodeLens integration for architectural metrics

2. **Project View**
   - Module dependency visualization
   - Architecture health indicators
   - Hotspot identification

3. **Command Palette**
   - Run architectural analysis
   - Generate architecture documentation
   - Export dependency graphs

## Performance

- Incremental analysis for large codebases
- Parallel processing of independent modules
- Configurable analysis depth
- Caching of intermediate results

## Best Practices

1. **Start Small**
   - Begin with critical modules
   - Gradually expand analysis scope
   - Focus on high-severity issues first

2. **Customize Rules**
   - Adjust sensitivity thresholds
   - Define project-specific patterns
   - Create custom rules for domain concepts

3. **Continuous Integration**
   - Add architectural checks to CI/CD
   - Set up quality gates
   - Track metrics over time

## Troubleshooting

### Common Issues

1. **False Positives**
   - Adjust confidence thresholds
   - Add exceptions for specific patterns
   - Update layer definitions

2. **Performance Problems**
   - Increase analysis depth gradually
   - Exclude third-party code
   - Use caching for large projects

## Contributing

### Adding New Analyzers

1. Implement the `Analyzer` trait
2. Register in `ArchitecturalAnalyzer`
3. Add tests
4. Update documentation

### Running Tests

```bash
cargo test -p rust-ai-ide-ai --test architectural
```

## License

MIT

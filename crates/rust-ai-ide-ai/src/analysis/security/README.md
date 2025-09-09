# Security Analyzer Module

This module provides comprehensive security analysis for Rust code, detecting various security vulnerabilities and anti-patterns.

## Features

- **Command Injection Detection**: Identifies potential command injection vulnerabilities in code that uses `std::process::Command`
- **TOCTOU (Time-of-Check Time-of-Use) Detection**: Finds race conditions in file operations
- **Cryptographic Issues**: Detects weak cryptographic algorithms and insecure key management
- **Input Validation**: Identifies missing or insufficient input validation
- **Concurrency Issues**: Detects data races and other concurrency-related security problems
- **Memory Safety**: Identifies potential memory safety violations and undefined behavior

## Usage

### Basic Usage

```rust
use rust_ai_ide_ai::analysis::{
    AnalysisPreferences,
    security::SecurityAnalyzer,
    Analyzer,
};
use syn::parse_file;

let code = r#"
    use std::process::Command;
    
    pub fn vulnerable(user_input: &str) {
        let _ = Command::new("echo").arg(user_input).output();
    }
"#;

let analyzer = SecurityAnalyzer::new()?;
let ast = parse_file(code)?;
let findings = analyzer.analyze(&ast, code, "example.rs")?;

// Process findings
for finding in findings {
    println!("Found security issue: {:?}", finding);
}
```

### Integration with Analysis Registry

The security analyzer is automatically registered with the default `AnalysisRegistry`:

```rust
use rust_ai_ide_ai::analysis::{
    AnalysisRegistry,
    AnalysisPreferences,
};

let registry = AnalysisRegistry::new();
let prefs = AnalysisPreferences {
    enable_security_analysis: true,
    ..Default::default()
};

let code = "/* your code here */";
let ast = parse_file(code)?;
let (findings, _) = registry.analyze_all(&ast, code, "example.rs", &prefs);
```

## Configuration

The security analyzer can be configured using `AnalysisPreferences`:

```rust
let mut prefs = AnalysisPreferences::default();
prefs.enable_security_analysis = true; // Enable/disable security analysis
prefs.confidence_threshold = 0.7; // Set confidence threshold (0.0 to 1.0)
```

## Adding Custom Security Rules

You can extend the security analyzer by implementing your own analyzers that implement the `Analyzer` trait:

```rust
use rust_ai_ide_ai::analysis::{
    Analyzer,
    AnalysisFinding,
    AnalysisPreferences,
};
use syn::File;

pub struct CustomSecurityAnalyzer;

impl Analyzer for CustomSecurityAnalyzer {
    type Finding = AnalysisFinding;
    
    fn analyze(&self, ast: &File, code: &str, file_path: &str) -> anyhow::Result<Vec<Self::Finding>> {
        // Your analysis logic here
        Ok(Vec::new())
    }
    
    fn name(&self) -> &'static str {
        "custom_security_analyzer"
    }
    
    fn description(&self) -> &'static str {
        "Custom security analyzer"
    }
}
```

## Testing

Run the security analyzer tests:

```bash
cargo test -p rust-ai-ide-ai --test security
```

## License

This project is licensed under the same license as the main project.

use rust_ai_ide_ai::analysis::{
    AnalysisCategory, AnalysisConfig, AnalysisFinding, AnalysisPreferences, AnalysisRegistry,
    Range, Severity,
};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_incremental_analysis() -> anyhow::Result<()> {
    // Create a temporary directory for our test
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("test.rs");

    // Create a simple Rust file
    let mut file = File::create(&file_path)?;
    writeln!(
        file,
        r#"
        // This is a test file
        fn main() {{
            println!("Hello, world!");
        }}
        "#
    )?;

    // Create a test analyzer that always returns a finding
    struct TestAnalyzer;
    impl rust_ai_ide_ai::analysis::Analyzer for TestAnalyzer {
        type Finding = AnalysisFinding;

        fn analyze(
            &self,
            _ast: &syn::File,
            _code: &str,
            file_path: &str,
        ) -> anyhow::Result<Vec<Self::Finding>> {
            Ok(vec![AnalysisFinding {
                message: "Test finding".to_string(),
                severity: Severity::Info,
                category: AnalysisCategory::CodeSmell,
                location: Range::new(1, 1, 1, 10),
                file: file_path.to_string(),
                suggestion: Some("Test suggestion".to_string()),
                confidence: 1.0,
                rule_id: "test-rule".to_string(),
            }])
        }

        fn name(&self) -> &'static str {
            "test-analyzer"
        }

        fn category(&self) -> AnalysisCategory {
            AnalysisCategory::CodeSmell
        }
    }

    // Set up analysis registry with test analyzer
    let config = AnalysisConfig {
        incremental_analysis: true,
        cache_enabled: true,
        ..Default::default()
    };

    let mut registry = AnalysisRegistry::new()?;
    registry.register_architectural_analyzer(TestAnalyzer);
    registry.update_config(config.clone())?;

    let prefs = AnalysisPreferences::default();

    // First analysis - should process the file
    let results = registry.analyze_directory(temp_dir.path().to_str().unwrap(), &prefs)?;
    assert!(!results.is_empty(), "First analysis should find issues");

    // Second analysis - should use cached results and not re-analyze
    let results = registry.analyze_directory(temp_dir.path().to_str().unwrap(), &prefs)?;
    assert!(
        results.is_empty(),
        "Second analysis should find no issues (cached)"
    );

    // Modify the file
    std::thread::sleep(std::time::Duration::from_secs(1)); // Ensure mtime changes
    let mut file = File::create(&file_path)?;
    writeln!(
        file,
        r#"
        // This is a modified test file
        fn main() {{
            println!("Hello, modified world!");
        }}
        "#
    )?;

    // Third analysis - should detect changes and re-analyze
    let results = registry.analyze_directory(temp_dir.path().to_str().unwrap(), &prefs)?;
    assert!(
        !results.is_empty(),
        "Third analysis should find issues after modification"
    );

    Ok(())
}

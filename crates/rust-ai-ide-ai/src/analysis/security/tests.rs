use super::*;
use crate::analysis::{AnalysisPreferences, CodeLocation};
use syn::parse_quote;

fn create_test_ast() -> syn::File {
    parse_quote! {
        use std::process::Command;

        pub fn vulnerable_function(user_input: &str) {
            // This is a vulnerable function that could lead to command injection
            let _ = Command::new("echo")
                .arg(user_input)  // Potential command injection
                .output();
        }

        pub fn safe_function() {
            // This is a safe function with no vulnerabilities
            println!("Hello, world!");
        }
    }
}

#[test]
fn test_security_analyzer_detects_command_injection() {
    let code = r#"
        use std::process::Command;

        pub fn vulnerable(user_input: &str) {
            let _ = Command::new("echo").arg(user_input).output();
        }
    "#;

    let analyzer = SecurityAnalyzer::new().unwrap();
    let ast = syn::parse_file(code).unwrap();
    let findings = analyzer.analyze(&ast, code, "test.rs").unwrap();

    // We should find at least one security issue
    assert!(!findings.is_empty(), "Expected to find security issues");

    // Check that we found the command injection issue
    let has_command_injection = findings
        .iter()
        .any(|finding| finding.issue_type == SecurityIssueType::CommandInjection);

    assert!(
        has_command_injection,
        "Expected to find command injection issue"
    );
}

#[test]
fn test_security_analyzer_with_safe_code() {
    let code = r#"
        pub fn safe_function() {
            println!("This is safe code");
        }
    "#;

    let analyzer = SecurityAnalyzer::new().unwrap();
    let ast = syn::parse_file(code).unwrap();
    let findings = analyzer.analyze(&ast, code, "safe.rs").unwrap();

    // Safe code should not trigger any security issues
    assert!(
        findings.is_empty(),
        "Expected no security issues in safe code"
    );
}

#[test]
fn test_security_analyzer_disabled() {
    let analyzer = SecurityAnalyzer::new().unwrap();
    let mut prefs = AnalysisPreferences::default();
    prefs.enable_security_analysis = false;

    assert!(
        !analyzer.is_enabled(&prefs),
        "Security analyzer should be disabled"
    );
}

#[test]
fn test_security_analyzer_enabled() {
    let analyzer = SecurityAnalyzer::new().unwrap();
    let mut prefs = AnalysisPreferences::default();
    prefs.enable_security_analysis = true;

    assert!(
        analyzer.is_enabled(&prefs),
        "Security analyzer should be enabled"
    );
}

#[test]
fn test_security_analyzer_integration() {
    // This test verifies that the security analyzer works with the analysis registry
    use crate::analysis::{AnalysisPreferences, AnalysisRegistry};

    let registry = AnalysisRegistry::new();
    let prefs = AnalysisPreferences {
        enable_security_analysis: true,
        ..Default::default()
    };

    let code = r#"
        use std::process::Command;

        pub fn vulnerable(user_input: &str) {
            let _ = Command::new("echo").arg(user_input).output();
        }
    "#;

    let ast = syn::parse_file(code).unwrap();
    let (findings, _) = registry.analyze_all(&ast, code, "test_integration.rs", &prefs);

    // We should find at least one security issue
    assert!(!findings.is_empty(), "Expected to find security issues");

    // Check that we found the command injection issue
    let has_security_issue = findings
        .iter()
        .any(|finding| finding.category == AnalysisCategory::Security);

    assert!(
        has_security_issue,
        "Expected to find security issues in analysis results"
    );
}

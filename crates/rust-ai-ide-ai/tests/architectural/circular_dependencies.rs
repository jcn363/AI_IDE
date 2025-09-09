//! Tests for circular dependency detection

use super::*;
use crate::analysis::{
    architectural::CircularDependencyAnalyzer,
    AnalysisType, Severity,
};

#[test]
fn test_no_circular_dependencies() {
    let code = r#"
        mod a {
            pub fn a() {}
        }
        
        mod b {
            use super::a;
            pub fn b() { a::a(); }
        }
    "#;
    
    let registry = create_test_registry();
    let result = registry.analyze_code(code, "test.rs").unwrap();
    
    assert_success(&result);
    assert!(
        !has_findings(&result, AnalysisType::CircularDependency, Severity::Warning),
        "Expected no circular dependency findings"
    );
}

#[test]
fn test_direct_circular_dependency() {
    let code = r#"
        mod a {
            use super::b;
            pub fn a() { b::b(); }
        }
        
        mod b {
            use super::a;
            pub fn b() { a::a(); }
        }
    "#;
    
    let registry = create_test_registry();
    let result = registry.analyze_code(code, "test.rs").unwrap();
    
    assert_finding!(
        &result,
        AnalysisType::CircularDependency,
        Severity::Warning,
        "Circular dependency detected between modules"
    );
}

#[test]
fn test_indirect_circular_dependency() {
    let code = r#"
        mod a {
            use super::b;
            pub fn a() { b::b(); }
        }
        
        mod b {
            use super::c;
            pub fn b() { c::c(); }
        }
        
        mod c {
            use super::a;
            pub fn c() { a::a(); }
        }
    "#;
    
    let registry = create_test_registry();
    let result = registry.analyze_code(code, "test.rs").unwrap();
    
    assert_finding!(
        &result,
        AnalysisType::CircularDependency,
        Severity::Warning,
        "Circular dependency detected between modules"
    );
}

#[test]
fn test_self_referential_module() {
    let code = r#"
        mod a {
            use super::a;
            pub fn a() { a::a(); }
        }
    "#;
    
    let registry = create_test_registry();
    let result = registry.analyze_code(code, "test.rs").unwrap();
    
    assert_finding!(
        &result,
        AnalysisType::CircularDependency,
        Severity::Warning,
        "Module has a self-reference"
    );
}

#[test]
fn test_circular_dependency_with_depth() {
    let code = r#"
        mod a { pub fn a() { b::b(); } }
        mod b { pub fn b() { c::c(); } }
        mod c { pub fn c() { d::d(); } }
        mod d { pub fn d() { a::a(); } }
    "#;
    
    let registry = create_test_registry();
    let result = registry.analyze_code(code, "test.rs").unwrap();
    
    assert_finding!(
        &result,
        AnalysisType::CircularDependency,
        Severity::Warning,
        "Circular dependency detected between modules"
    );
}

// Helper function to check for specific findings
fn has_findings(result: &AnalysisResult, analysis_type: AnalysisType, severity: Severity) -> bool {
    result.findings.iter().any(|f| f.analysis_type == analysis_type && f.severity == severity)
}

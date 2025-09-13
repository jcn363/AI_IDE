//! Comprehensive tests for enhanced AI analysis capabilities
//!
//! This module tests all aspects of the AI-enhanced code analysis system including:
//! - Code smell detection and style analysis
//! - Error resolution with pattern matching
//! - Learning system database operations
//! - Code generation capabilities
//! - End-to-end integration workflows
//! - Performance testing with large codebases

use crate::analysis::*;
use crate::code_generation::*;
use crate::error_resolution::*;
use crate::learning::*;
use crate::*;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;

/// Test fixtures and sample code for analysis
mod fixtures {
    pub const SIMPLE_FUNCTION: &str = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;

    pub const LONG_METHOD: &str = r#"
fn very_long_function() -> i32 {
    let mut result = 0;
    for i in 0..100 {
        if i % 2 == 0 {
            result += i;
        } else {
            result -= i;
        }
        if i % 10 == 0 {
            println!("Processing: {}", i);
        }
        if i % 20 == 0 {
            println!("Checkpoint: {}", i);
        }
        if i % 30 == 0 {
            println!("Major checkpoint: {}", i);
        }
        // Many more lines to make it long...
        let temp = i * 2;
        let temp2 = temp + 1;
        let temp3 = temp2 * 3;
        result += temp3;
        println!("Temp calculations: {} {} {}", temp, temp2, temp3);
        if temp3 > 100 {
            println!("Large temp3: {}", temp3);
        }
        if temp3 > 200 {
            println!("Very large temp3: {}", temp3);
        }
        if temp3 > 300 {
            println!("Extremely large temp3: {}", temp3);
        }
        // Continue with more repetitive code...
        for j in 0..10 {
            let nested_calc = j * i;
            if nested_calc > 50 {
                println!("Nested calc: {}", nested_calc);
            }
        }
        // More lines to exceed the threshold...
        let final_calc = result + i;
        if final_calc > 1000 {
            println!("Final calc exceeded 1000: {}", final_calc);
        }
        result = final_calc;
    }
    result
}
"#;

    pub const GOD_OBJECT: &str = r#"
struct UserManager {
    users: Vec<User>,
    database: Database,
    cache: Cache,
    logger: Logger,
}

impl UserManager {
    fn new() -> Self { todo!() }
    fn create_user(&mut self) { todo!() }
    fn delete_user(&mut self) { todo!() }
    fn update_user(&mut self) { todo!() }
    fn find_user(&self) { todo!() }
    fn list_users(&self) { todo!() }
    fn authenticate_user(&self) { todo!() }
    fn authorize_user(&self) { todo!() }
    fn send_email(&self) { todo!() }
    fn send_sms(&self) { todo!() }
    fn log_activity(&self) { todo!() }
    fn backup_data(&self) { todo!() }
    fn restore_data(&self) { todo!() }
    fn validate_input(&self) { todo!() }
    fn sanitize_input(&self) { todo!() }
    fn encrypt_data(&self) { todo!() }
    fn decrypt_data(&self) { todo!() }
    fn compress_data(&self) { todo!() }
    fn decompress_data(&self) { todo!() }
    fn generate_report(&self) { todo!() }
    fn export_data(&self) { todo!() }
    fn import_data(&self) { todo!() }
    fn schedule_task(&self) { todo!() }
    fn cancel_task(&self) { todo!() }
    fn monitor_system(&self) { todo!() }
}
"#;

    pub const SECURITY_ISSUES: &str = r#"
fn insecure_function() {
    let password = "hardcoded_password123";
    let api_key = "sk-1234567890abcdef";
    let query = format!("SELECT * FROM users WHERE id = {}", user_input);

    unsafe {
        let ptr = std::ptr::null_mut();
        *ptr = 42;
    }
}
"#;

    pub const PERFORMANCE_ISSUES: &str = r#"
fn inefficient_function(data: &[String]) -> String {
    let mut result = String::new();
    for item in data {
        result = result + item; // Inefficient string concatenation
        let cloned = item.clone(); // Unnecessary clone
        println!("{}", cloned.to_string()); // Unnecessary to_string
    }

    // Nested loops
    for i in 0..data.len() {
        for j in 0..data.len() {
            if data[i] == data[j] {
                println!("Found duplicate");
            }
        }
    }

    result
}
"#;

    pub const STYLE_VIOLATIONS: &str = r#"
fn BadFunctionName() { } // Should be snake_case

struct badStructName { } // Should be PascalCase

enum badEnumName { } // Should be PascalCase

const badConstant: i32 = 42; // Should be SCREAMING_SNAKE_CASE

fn function_with_many_params(
    param1: i32,
    param2: String,
    param3: bool,
    param4: f64,
    param5: Vec<i32>,
    param6: HashMap<String, String>
) {
    // Data clump - many related parameters
}
"#;

    pub const ARCHITECTURE_ISSUES: &str = r#"
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::net::TcpStream;
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;
use serde_json;
use reqwest;
use tokio;
use async_std;

// High coupling - many external dependencies

struct TightlyCoupledStruct {
    file_handler: File,
    network_handler: TcpStream,
    data_processor: DataProcessor,
    cache: HashMap<String, String>,
}

impl TightlyCoupledStruct {
    fn process_everything(&mut self) {
        // Violates single responsibility
        self.read_file();
        self.process_network();
        self.update_cache();
        self.send_notifications();
    }

    fn read_file(&mut self) { }
    fn process_network(&mut self) { }
    fn update_cache(&mut self) { }
    fn send_notifications(&mut self) { }
}
"#;

    pub const FEATURE_ENVY: &str = r#"
fn calculate_user_score(user: &User) -> f64 {
    let base_score = user.get_base_score();
    let bonus = user.get_bonus_points();
    let penalty = user.get_penalty_points();
    let multiplier = user.get_multiplier();
    let level = user.get_level();
    let experience = user.get_experience();

    // This function is overly dependent on User's internals
    (base_score + bonus - penalty) * multiplier * level as f64 + experience
}
"#;

    pub const ASYNC_BLOCKING: &str = r#"
async fn problematic_async_function() {
    let mut file = std::fs::File::open("data.txt").unwrap(); // Blocking in async
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap(); // Blocking in async

    std::thread::sleep(std::time::Duration::from_secs(1)); // Blocking in async

    println!("{}", contents);
}
"#;

    pub const COMPILATION_ERRORS: &str = r#"
fn function_with_errors() {
    let x: i32 = "not a number"; // Type mismatch
    let y = undefined_variable; // Undefined variable
    let z = x.some_method(); // Method doesn't exist

    if x = 5 { // Assignment instead of comparison
        println!("x is 5");
    }

    let vec = Vec::new();
    vec.push(1); // Cannot borrow as mutable
}
"#;

    pub const LARGE_CODEBASE: &str = r#"
// Simulated large codebase for performance testing
mod module1 {
    pub struct LargeStruct1 {
        field1: String,
        field2: i32,
        field3: Vec<String>,
    }

    impl LargeStruct1 {
        pub fn method1(&self) -> String { self.field1.clone() }
        pub fn method2(&self) -> i32 { self.field2 }
        pub fn method3(&self) -> &Vec<String> { &self.field3 }
    }
}

mod module2 {
    pub struct LargeStruct2 {
        data: std::collections::HashMap<String, i32>,
    }

    impl LargeStruct2 {
        pub fn new() -> Self {
            Self { data: std::collections::HashMap::new() }
        }

        pub fn insert(&mut self, key: String, value: i32) {
            self.data.insert(key, value);
        }

        pub fn get(&self, key: &str) -> Option<&i32> {
            self.data.get(key)
        }
    }
}

// Repeat similar patterns for performance testing...
"#;
}

/// Helper functions for testing
mod test_helpers {
    use super::*;

    pub fn create_test_context(code: &str) -> AIContext {
        AIContext {
            current_code: code.to_string(),
            file_name: Some("test.rs".to_string()),
            cursor_position: Some((1, 1)),
            selection: None,
            project_context: HashMap::new(),
            dependencies: vec![],
            workspace_structure: HashMap::new(),
            analysis_preferences: AnalysisPreferences::default(),
        }
    }

    pub fn create_test_preferences() -> AnalysisPreferences {
        AnalysisPreferences {
            enable_code_smells: true,
            enable_security: true,
            enable_performance: true,
            enable_code_style: true,
            enable_architecture: true,
            enable_learning: true,
            confidence_threshold: 0.5,
            timeout_seconds: 30,
            include_explanations: true,
            include_examples: true,
            privacy_mode: PrivacyMode::OptIn,
        }
    }

    pub async fn create_temp_db() -> Result<(TempDir, PathBuf)> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        Ok((temp_dir, db_path))
    }

    pub fn assert_finding_exists(findings: &[AnalysisFinding], rule_id: &str) {
        assert!(
            findings.iter().any(|f| f.rule_id == rule_id),
            "Expected finding with rule_id '{}' not found. Available: {:?}",
            rule_id,
            findings.iter().map(|f| &f.rule_id).collect::<Vec<_>>()
        );
    }

    pub fn count_findings_by_category(
        findings: &[AnalysisFinding],
        category: AnalysisCategory,
    ) -> usize {
        findings.iter().filter(|f| f.category == category).count()
    }
}

/// Tests for code smell detection
#[cfg(test)]
mod code_smell_tests {
    use super::*;
    use test_helpers::*;

    #[test]
    fn test_long_method_detection() {
        let analyzer = CodeAnalyzer::new();
        let result = analyzer
            .analyze_code(fixtures::LONG_METHOD, "test.rs")
            .unwrap();

        assert_finding_exists(&result.findings, "long_method");
        assert!(count_findings_by_category(&result.findings, AnalysisCategory::CodeSmell) > 0);
    }

    #[test]
    fn test_god_object_detection() {
        let analyzer = CodeAnalyzer::new();
        let result = analyzer
            .analyze_code(fixtures::GOD_OBJECT, "test.rs")
            .unwrap();

        assert_finding_exists(&result.findings, "god_object");

        let god_object_findings: Vec<_> = result
            .findings
            .iter()
            .filter(|f| f.rule_id == "god_object")
            .collect();

        assert!(!god_object_findings.is_empty());
        assert!(god_object_findings[0].message.contains("UserManager"));
    }

    #[test]
    fn test_feature_envy_detection() {
        let analyzer = CodeAnalyzer::new();
        let result = analyzer
            .analyze_code(fixtures::FEATURE_ENVY, "test.rs")
            .unwrap();

        assert_finding_exists(&result.findings, "feature_envy");

        let feature_envy_findings: Vec<_> = result
            .findings
            .iter()
            .filter(|f| f.rule_id == "feature_envy")
            .collect();

        assert!(!feature_envy_findings.is_empty());
        assert!(feature_envy_findings[0].confidence > 0.5);
    }

    #[test]
    fn test_data_clump_detection() {
        let analyzer = CodeAnalyzer::new();
        let result = analyzer
            .analyze_code(fixtures::STYLE_VIOLATIONS, "test.rs")
            .unwrap();

        assert_finding_exists(&result.findings, "data_clump");
    }

    #[test]
    fn test_magic_number_detection() {
        let code = r#"
fn calculate_area(radius: f64) -> f64 {
    3.14159 * radius * radius // Magic number
}
"#;
        let analyzer = CodeAnalyzer::new();
        let result = analyzer.analyze_code(code, "test.rs").unwrap();

        // Magic numbers might be detected depending on implementation
        let magic_findings: Vec<_> = result
            .findings
            .iter()
            .filter(|f| f.rule_id == "magic_number")
            .collect();

        // Should detect the magic number 3.14159
        assert!(!magic_findings.is_empty());
    }

    #[test]
    fn test_duplicate_code_detection() {
        let code = r#"
fn function1() {
    println!("Hello");
    println!("World");
    println!("Test");
}

fn function2() {
    println!("Hello");
    println!("World");
    println!("Test");
}
"#;
        let analyzer = CodeAnalyzer::new();
        let result = analyzer.analyze_code(code, "test.rs").unwrap();

        assert_finding_exists(&result.findings, "duplicate_code");
    }

    #[test]
    fn test_complex_conditional_detection() {
        let code = r#"
fn complex_function(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 20 {
                if x > 30 {
                    return x * 4;
                }
                return x * 3;
            }
            return x * 2;
        }
        return x;
    }
    0
}
"#;
        let analyzer = CodeAnalyzer::new();
        let result = analyzer.analyze_code(code, "test.rs").unwrap();

        assert_finding_exists(&result.findings, "complex_conditional");
    }
}

/// Tests for security analysis
#[cfg(test)]
mod security_tests {
    use super::*;
    use test_helpers::*;

    #[test]
    fn test_hardcoded_password_detection() {
        let analyzer = CodeAnalyzer::new();
        let result = analyzer
            .analyze_code(fixtures::SECURITY_ISSUES, "test.rs")
            .unwrap();

        let security_findings: Vec<_> = result
            .findings
            .iter()
            .filter(|f| f.category == AnalysisCategory::Security)
            .collect();

        assert!(!security_findings.is_empty());
        assert!(security_findings
            .iter()
            .any(|f| f.rule_id == "hardcoded_password"));
    }

    #[test]
    fn test_hardcoded_api_key_detection() {
        let analyzer = CodeAnalyzer::new();
        let result = analyzer
            .analyze_code(fixtures::SECURITY_ISSUES, "test.rs")
            .unwrap();

        let security_findings: Vec<_> = result
            .findings
            .iter()
            .filter(|f| f.category == AnalysisCategory::Security)
            .collect();

        assert!(security_findings
            .iter()
            .any(|f| f.rule_id == "hardcoded_api_key"));
    }

    #[test]
    fn test_sql_injection_risk_detection() {
        let analyzer = CodeAnalyzer::new();
        let result = analyzer
            .analyze_code(fixtures::SECURITY_ISSUES, "test.rs")
            .unwrap();

        let security_findings: Vec<_> = result
            .findings
            .iter()
            .filter(|f| f.category == AnalysisCategory::Security)
            .collect();

        assert!(security_findings
            .iter()
            .any(|f| f.rule_id == "sql_injection_risk"));
    }

    #[test]
    fn test_unsafe_block_detection() {
        let analyzer = CodeAnalyzer::new();
        let result = analyzer
            .analyze_code(fixtures::SECURITY_ISSUES, "test.rs")
            .unwrap();

        let security_findings: Vec<_> = result
            .findings
            .iter()
            .filter(|f| f.category == AnalysisCategory::Security)
            .collect();

        assert!(security_findings
            .iter()
            .any(|f| f.rule_id == "unsafe_block"));
    }

    #[test]
    fn test_security_confidence_scores() {
        let analyzer = CodeAnalyzer::new();
        let result = analyzer
            .analyze_code(fixtures::SECURITY_ISSUES, "test.rs")
            .unwrap();

        let security_findings: Vec<_> = result
            .findings
            .iter()
            .filter(|f| f.category == AnalysisCategory::Security)
            .collect();

        for finding in security_findings {
            assert!(
                finding.confidence >= 0.5,
                "Security finding confidence too low: {}",
                finding.confidence
            );
            assert!(
                finding.confidence <= 1.0,
                "Security finding confidence too high: {}",
                finding.confidence
            );
        }
    }
}

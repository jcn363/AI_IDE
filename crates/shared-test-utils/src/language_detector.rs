//! Language detection and test framework mapping utilities
//!
//! This module provides language detection based on file extensions and
//! test framework mappings for different programming languages.

use std::collections::HashMap;
use std::path::Path;
use super::test_config::ProgrammingLanguage;

/// Test framework information
#[derive(Debug, Clone)]
pub struct TestFrameworkInfo {
    pub language: ProgrammingLanguage,
    pub test_frameworks: Vec<String>,
    pub file_extensions: Vec<String>,
    pub preferred_framework: String,
}

/// Language autodetection service
#[derive(Debug)]
pub struct LanguageDetector {
    framework_map: HashMap<String, TestFrameworkInfo>,
}

impl LanguageDetector {
    /// Create a new language detector with default mappings
    pub fn new() -> Self {
        let mut framework_map = HashMap::new();

        // Add language mappings...
        Self::add_language_mapping(&mut framework_map, "rs", ProgrammingLanguage::Rust, vec!["cargo-test"], "cargo-test");
        Self::add_language_mapping(&mut framework_map, "ts", ProgrammingLanguage::TypeScript, vec!["jest", "mocha", "vitest"], "jest");
        Self::add_language_mapping(&mut framework_map, "js", ProgrammingLanguage::JavaScript, vec!["jest", "mocha", "jasmine"], "jest");
        Self::add_language_mapping(&mut framework_map, "py", ProgrammingLanguage::Python, vec!["pytest", "unittest"], "pytest");
        Self::add_language_mapping(&mut framework_map, "java", ProgrammingLanguage::Java, vec!["junit", "testng"], "junit");
        Self::add_language_mapping(&mut framework_map, "cs", ProgrammingLanguage::CSharp, vec!["nunit", "xunit", "mstest"], "xunit");
        Self::add_language_mapping(&mut framework_map, "go", ProgrammingLanguage::Go, vec!["testing"], "testing");
        Self::add_language_mapping(&mut framework_map, "cpp", ProgrammingLanguage::Cpp, vec!["googletest", "catch2"], "googletest");
        Self::add_language_mapping(&mut framework_map, "c", ProgrammingLanguage::C, vec!["cmocka", "criterion"], "cmocka");

        Self { framework_map }
    }

    fn add_language_mapping(
        map: &mut HashMap<String, TestFrameworkInfo>,
        extension: &str,
        language: ProgrammingLanguage,
        frameworks: Vec<&str>,
        preferred: &str,
    ) {
        map.insert(
            extension.to_string(),
            TestFrameworkInfo {
                language,
                test_frameworks: frameworks.into_iter().map(|s| s.to_string()).collect(),
                file_extensions: vec![extension.to_string()],
                preferred_framework: preferred.to_string(),
            }
        );
    }

    /// Detect language and framework from file path
    pub fn detect_language(&self, file_path: &str) -> (ProgrammingLanguage, String) {
        let extension = self.get_file_extension(file_path);

        if let Some(info) = self.framework_map.get(&extension) {
            (info.language.clone(), info.preferred_framework.clone())
        } else {
            (ProgrammingLanguage::Unknown, "unknown".to_string())
        }
    }

    /// Get available test frameworks for a language
    pub fn get_test_frameworks(&self, language: &ProgrammingLanguage) -> Vec<String> {
        for info in self.framework_map.values() {
            if info.language == *language {
                return info.test_frameworks.clone();
            }
        }
        vec![]
    }

    /// Extract file extension from path
    fn get_file_extension(&self, file_path: &str) -> String {
        Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase()
    }
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility method for extracting file extension (standalone function)
pub fn get_file_extension(file_path: &str) -> String {
    Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase()
}
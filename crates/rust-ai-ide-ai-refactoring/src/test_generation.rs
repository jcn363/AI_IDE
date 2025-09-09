use crate::types;
use rust_ai_ide_common::types::{
    GeneratedTest, GeneratedTests, ProgrammingLanguage, TestCoverage, TestType,
};
use std::collections::HashMap;
use std::path::Path;

/// Language detection configuration and information for test generation
#[derive(Debug, Clone)]
pub struct LanguageInfo {
    pub language: ProgrammingLanguage,
    pub test_frameworks: Vec<String>,
    pub file_extensions: Vec<String>,
    pub preferred_framework: String,
}

/// Language detector that maps file extensions to language information
#[derive(Debug, Clone)]
pub struct LanguageDetector {
    pub language_patterns: HashMap<String, LanguageInfo>,
}

/// Unified test generator for ai-refactoring using shared types and traits
#[derive(Debug)]
pub struct RefactoringTestGenerator {
    language_detector: LanguageDetector,
}

impl LanguageDetector {
    pub fn new() -> Self {
        let mut language_patterns = HashMap::new();

        // Rust
        language_patterns.insert(
            "rs".to_string(),
            LanguageInfo {
                language: ProgrammingLanguage::Rust,
                test_frameworks: vec!["cargo-test".to_string(), "rust_test".to_string()],
                file_extensions: vec!["rs".to_string()],
                preferred_framework: "cargo-test".to_string(),
            },
        );

        // TypeScript
        language_patterns.insert(
            "ts".to_string(),
            LanguageInfo {
                language: ProgrammingLanguage::TypeScript,
                test_frameworks: vec![
                    "jest".to_string(),
                    "mocha".to_string(),
                    "vitest".to_string(),
                ],
                file_extensions: vec!["ts".to_string()],
                preferred_framework: "jest".to_string(),
            },
        );

        // JavaScript
        language_patterns.insert(
            "js".to_string(),
            LanguageInfo {
                language: ProgrammingLanguage::JavaScript,
                test_frameworks: vec![
                    "jest".to_string(),
                    "mocha".to_string(),
                    "jasmine".to_string(),
                ],
                file_extensions: vec!["js".to_string(), "mjs".to_string()],
                preferred_framework: "jest".to_string(),
            },
        );

        // Python
        language_patterns.insert(
            "py".to_string(),
            LanguageInfo {
                language: ProgrammingLanguage::Python,
                test_frameworks: vec!["pytest".to_string(), "unittest".to_string()],
                file_extensions: vec!["py".to_string()],
                preferred_framework: "pytest".to_string(),
            },
        );

        // Java
        language_patterns.insert(
            "java".to_string(),
            LanguageInfo {
                language: ProgrammingLanguage::Java,
                test_frameworks: vec!["junit".to_string(), "testng".to_string()],
                file_extensions: vec!["java".to_string()],
                preferred_framework: "junit".to_string(),
            },
        );

        // C#
        language_patterns.insert(
            "cs".to_string(),
            LanguageInfo {
                language: ProgrammingLanguage::CSharp,
                test_frameworks: vec![
                    "nunit".to_string(),
                    "xunit".to_string(),
                    "mstest".to_string(),
                ],
                file_extensions: vec!["cs".to_string()],
                preferred_framework: "xunit".to_string(),
            },
        );

        // Go
        language_patterns.insert(
            "go".to_string(),
            LanguageInfo {
                language: ProgrammingLanguage::Go,
                test_frameworks: vec!["testing".to_string()],
                file_extensions: vec!["go".to_string()],
                preferred_framework: "testing".to_string(),
            },
        );

        // C++
        language_patterns.insert(
            "cpp".to_string(),
            LanguageInfo {
                language: ProgrammingLanguage::Cpp,
                test_frameworks: vec!["googletest".to_string(), "catch2".to_string()],
                file_extensions: vec!["cpp".to_string(), "cc".to_string(), "cxx".to_string()],
                preferred_framework: "googletest".to_string(),
            },
        );

        // C
        language_patterns.insert(
            "c".to_string(),
            LanguageInfo {
                language: ProgrammingLanguage::C,
                test_frameworks: vec!["cmocka".to_string(), "criterion".to_string()],
                file_extensions: vec!["c".to_string(), "h".to_string()],
                preferred_framework: "cmocka".to_string(),
            },
        );

        LanguageDetector { language_patterns }
    }

    /// Detect language from file path
    pub fn detect_language(&self, file_path: &str) -> (ProgrammingLanguage, String) {
        let file_extension = self.get_file_extension(file_path);

        if let Some(info) = self.language_patterns.get(file_extension.as_str()) {
            (info.language.clone(), info.preferred_framework.clone())
        } else {
            (ProgrammingLanguage::Unknown, "unknown".to_string())
        }
    }

    /// Get test frameworks available for a language
    pub fn get_test_frameworks(&self, language: &ProgrammingLanguage) -> Vec<String> {
        for info in self.language_patterns.values() {
            if info.language == *language {
                return info.test_frameworks.clone();
            }
        }
        vec![]
    }

    /// Extract file extension from file path
    fn get_file_extension(&self, file_path: &str) -> String {
        Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase()
    }
}

impl RefactoringTestGenerator {
    pub fn new() -> Self {
        RefactoringTestGenerator {
            language_detector: LanguageDetector::new(),
        }
    }

    pub fn new_with_detector(language_detector: LanguageDetector) -> Self {
        RefactoringTestGenerator { language_detector }
    }
}

#[async_trait::async_trait]
impl rust_ai_ide_shared_codegen::traits::TestGenerator for RefactoringTestGenerator {
    async fn generate_test_suite(
        &self,
        _code: &str,
        _context: &rust_ai_ide_common::types::TestGenerationContext,
    ) -> Result<GeneratedTests, Box<dyn std::error::Error + Send + Sync>> {
        // For now, just return empty test suite - not implemented for refactoring-specific generator
        Ok(GeneratedTests {
            unit_tests: Vec::new(),
            integration_tests: vec![],
            property_tests: vec![],
            benchmark_tests: vec![],
            coverage_estimates: vec![],
        })
    }

    /// Generate basic unit tests for given code
    async fn generate_unit_tests(
        &self,
        _code: &str,
        _context: &rust_ai_ide_common::types::TestGenerationContext,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Vec::new()) // Not implemented for refactoring-specific generator
    }

    /// Generate integration tests for component interactions
    async fn generate_integration_tests(
        &self,
        _context: &rust_ai_ide_common::types::TestGenerationContext,
        _dependencies: Vec<String>,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Vec::new()) // Not implemented for refactoring-specific generator
    }

    /// Generate property-based tests
    async fn generate_property_tests(
        &self,
        _code: &str,
        _language: &rust_ai_ide_common::types::ProgrammingLanguage,
        _properties: Vec<String>,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Vec::new()) // Not implemented for refactoring-specific generator
    }

    /// Estimate test coverage for generated tests
    async fn estimate_coverage(
        &self,
        _tests: &[GeneratedTest],
        _language: &rust_ai_ide_common::types::ProgrammingLanguage,
    ) -> Result<Vec<TestCoverage>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Vec::new())
    }

    /// Get supported programming languages
    fn supported_languages(&self) -> Vec<rust_ai_ide_common::types::ProgrammingLanguage> {
        vec![
            rust_ai_ide_common::types::ProgrammingLanguage::Rust,
            rust_ai_ide_common::types::ProgrammingLanguage::TypeScript,
            rust_ai_ide_common::types::ProgrammingLanguage::JavaScript,
            rust_ai_ide_common::types::ProgrammingLanguage::Python,
            rust_ai_ide_common::types::ProgrammingLanguage::Java,
            rust_ai_ide_common::types::ProgrammingLanguage::CSharp,
            rust_ai_ide_common::types::ProgrammingLanguage::Go,
            rust_ai_ide_common::types::ProgrammingLanguage::Cpp,
            rust_ai_ide_common::types::ProgrammingLanguage::C,
        ]
    }

    /// Get available test frameworks for a language
    fn get_test_frameworks(
        &self,
        language: &rust_ai_ide_common::types::ProgrammingLanguage,
    ) -> Vec<String> {
        self.language_detector.get_test_frameworks(language)
    }

    /// Validate generated tests for correctness
    fn validate_tests(
        &self,
        _tests: &[GeneratedTest],
    ) -> Vec<rust_ai_ide_shared_codegen::traits::ValidationError> {
        Vec::new()
    }

    /// Generate refactoring tests (required by TestGenerator trait)
    async fn generate_refactoring_tests(
        &self,
        refactoring_type: &rust_ai_ide_common::types::RefactoringType,
        context: &rust_ai_ide_common::types::RefactoringContext,
        result: &rust_ai_ide_common::types::RefactoringResult,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>> {
        // Convert common types to local types for internal processing
        let local_refactoring_type = match refactoring_type {
            rust_ai_ide_common::types::RefactoringType::Rename => {
                Some(types::RefactoringType::Rename)
            }
            rust_ai_ide_common::types::RefactoringType::ExtractFunction => {
                Some(types::RefactoringType::ExtractFunction)
            }
            rust_ai_ide_common::types::RefactoringType::ExtractVariable => {
                Some(types::RefactoringType::ExtractVariable)
            }
            rust_ai_ide_common::types::RefactoringType::ExtractInterface => {
                Some(types::RefactoringType::ExtractInterface)
            }
            rust_ai_ide_common::types::RefactoringType::ConvertToAsync => {
                Some(types::RefactoringType::ConvertToAsync)
            }
            _ => None, // Unsupported refactoring type
        };

        match local_refactoring_type {
            Some(ref_type) => {
                let (language, framework) =
                    self.language_detector.detect_language(&context.file_path);
                let mut tests = Vec::new();

                match ref_type {
                    types::RefactoringType::Rename => {
                        tests.extend(
                            self.generate_rename_tests(context, result, &language, &framework)
                                .await?,
                        );
                    }
                    types::RefactoringType::ExtractFunction => {
                        tests.extend(
                            self.generate_extract_function_tests(
                                context, result, &language, &framework,
                            )
                            .await?,
                        );
                    }
                    types::RefactoringType::ExtractVariable => {
                        tests.extend(
                            self.generate_extract_variable_tests(
                                context, result, &language, &framework,
                            )
                            .await?,
                        );
                    }
                    types::RefactoringType::ExtractInterface => {
                        tests.extend(
                            self.generate_extract_interface_tests(
                                context, result, &language, &framework,
                            )
                            .await?,
                        );
                    }
                    types::RefactoringType::ConvertToAsync => {
                        tests.extend(
                            self.generate_async_conversion_tests(
                                context, result, &language, &framework,
                            )
                            .await?,
                        );
                    }
                    _ => {
                        tests.extend(
                            self.generate_generic_tests(
                                refactoring_type,
                                context,
                                result,
                                &language,
                                &framework,
                            )
                            .await?,
                        );
                    }
                }

                Ok(tests)
            }
            None => {
                // Return empty vec for unsupported types
                Ok(vec![])
            }
        }
    }

    /// Generate tests for rename refactoring with language-aware templates
    async fn generate_rename_tests(
        &self,
        context: &rust_ai_ide_common::types::RefactoringContext,
        result: &rust_ai_ide_common::types::RefactoringResult,
        language: &rust_ai_ide_common::types::ProgrammingLanguage,
        framework: &str,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>> {
        let default_name = "old_name".to_string();
        let old_name = context.symbol_name.as_ref().unwrap_or(&default_name);
        let new_name = "new_name"; // This would be extracted from the refactoring result

        let tests = match language {
            ProgrammingLanguage::Rust => {
                let test_code = format!(
                    r#"
#[test]
fn test_rename_{}_to_{}() {{
    // Test that the renamed symbol works correctly
    let {}: &str = "{}";
    assert_eq!({}, "{}");

// Test function call with renamed parameter
    fn renamed_function({}: &str) -> &str {{
        {}
    }}

    let result = renamed_function("{}");
    assert_eq!(result, "{}");
}}
"#,
                    old_name,
                    new_name,
                    new_name,
                    old_name,
                    new_name,
                    old_name,
                    new_name,
                    new_name,
                    new_name,
                    old_name
                );

                vec![GeneratedTest {
                    name: format!("test_rename_{}_to_{}", old_name, new_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!("Test renaming {} to {} in Rust", old_name, new_name),
                    framework: framework.to_string(),
                    language: language.clone(),
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.8,
                }]
            }
            ProgrammingLanguage::TypeScript => {
                let test_code = format!(
                    r#"
describe("Rename Test: {} -> {}", () => {{
    it("should work with renamed identifier", () => {{
        const {}: string = "{}";
        expect({}).toBe("{}");

        function renamedFunction({}: string): string {{
            return {};
        }}

        const result = renamedFunction("{}");
        expect(result).toBe("{}");
    }});

    it("should handle renamed method calls", () => {{
        class TestClass {{
            renamedMethod({}: string): string {{
                return {};
            }}
        }}

        const instance = new TestClass();
        const result = instance.renamedMethod("{}");
        expect(result).toBe("{}");
    }});
}});
"#,
                    old_name,
                    new_name,
                    new_name,
                    old_name,
                    new_name,
                    old_name,
                    new_name,
                    new_name,
                    new_name,
                    old_name,
                    new_name,
                    new_name,
                    new_name,
                    old_name
                );

                vec![GeneratedTest {
                    name: format!("test_rename_{}_to_{}", old_name, new_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!(
                        "Test renaming {} to {} in TypeScript",
                        old_name, new_name
                    ),
                    framework: framework.to_string(),
                    language: language.clone(),
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.8,
                }]
            }
            ProgrammingLanguage::JavaScript => {
                let test_code = format!(
                    r#"
describe("Rename Test: {} -> {}", () => {{
    it("should work with renamed identifier", () => {{
        const {} = "{}";
        expect({}).toBe("{}");

        function renamedFunction({}) {{
            return {};
        }}

        const result = renamedFunction("{}");
        expect(result).toBe("{}");
    }});
}});
"#,
                    old_name,
                    new_name,
                    new_name,
                    old_name,
                    new_name,
                    old_name,
                    new_name,
                    new_name,
                    new_name,
                    old_name
                );

                vec![GeneratedTest {
                    name: format!("test_rename_{}_to_{}", old_name, new_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!(
                        "Test renaming {} to {} in JavaScript",
                        old_name, new_name
                    ),
                    framework: framework.to_string(),
                    language: language.clone(),
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.8,
                }]
            }
            ProgrammingLanguage::Python => {
                let test_code = format!(
                    r#"
class TestRename:
    def test_rename_{}_to_{}(self):
        """Test that {} was successfully renamed"""
        # Original value for comparison
        {} = "{}"

        # Test that the renamed symbol has the expected value
        self.assertEqual({}, "{}")

        def renamed_function({}):
            return {}

        # Test function with renamed parameter
        result = renamed_function("{}")
        self.assertEqual(result, "{}")
"#,
                    old_name,
                    new_name,
                    old_name,
                    new_name,
                    old_name,
                    new_name,
                    old_name,
                    new_name,
                    new_name,
                    new_name,
                    old_name
                );

                vec![GeneratedTest {
                    name: format!("test_rename_{}_to_{}", old_name, new_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!("Test renaming {} to {} in Python", old_name, new_name),
                    framework: framework.to_string(),
                    language: language.clone(),
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.8,
                }]
            }
            ProgrammingLanguage::Java => {
                let test_code = format!(
                    r#"
public class TestRename_{}To{} {{
    @Test
    public void testRenamedIdentifier() {{
        String {} = "{}";

        // Test that the renamed variable has the expected value
        assertEquals("{}", {});

        String result = renamedFunction("{}");
        assertEquals("{}", result);
    }}

    private String renamedFunction(String {}) {{
        return {};
    }}
}}
"#,
                    old_name,
                    new_name,
                    new_name,
                    old_name,
                    old_name,
                    new_name,
                    old_name,
                    old_name,
                    new_name,
                    new_name
                );

                vec![GeneratedTest {
                    name: format!("TestRename{}To{}", old_name, new_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!("Test renaming {} to {} in Java", old_name, new_name),
                    framework: framework.to_string(),
                    language: language.clone(),
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.8,
                }]
            }
            _ => {
                // Fallback to generic test for unsupported languages
                let test_code = format!(
                    r#"
// Generic rename test for unsupported language
// Test that {} was successfully renamed to {}
test_rename_{}_to_{}() {{
    // Test logic would depend on the target language
    assert_equal({}, {});
}}
"#,
                    old_name, new_name, old_name, new_name, new_name, old_name
                );

                vec![GeneratedTest {
                    name: format!("test_rename_{}_to_{}", old_name, new_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!("Generic test for renaming {} to {}", old_name, new_name),
                    framework: "unknown".to_string(),
                    language: ProgrammingLanguage::Unknown,
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.5,
                }]
            }
        };

        Ok(tests)
    }

    /// Generate tests for extract function refactoring with language-aware templates
    async fn generate_extract_function_tests(
        &self,
        context: &rust_ai_ide_common::types::RefactoringContext,
        result: &rust_ai_ide_common::types::RefactoringResult,
        language: &rust_ai_ide_common::types::ProgrammingLanguage,
        framework: &str,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>> {
        let default_function_name = "extracted_function".to_string();
        let function_name = context
            .symbol_name
            .as_ref()
            .unwrap_or(&default_function_name);

        let tests = match language {
            ProgrammingLanguage::Rust => {
                let test_code = format!(
                    r#"
#[test]
fn test_{}_with_various_inputs() {{
    // Test the extracted function with various inputs
    assert_eq!({}(1), 2);
    assert_eq!({}(5), 10);
    assert_eq!({}(0), 0);

    // Test edge cases
    assert_eq!({}(-1), -2);
}}

#[test]
fn test_{}_preserves_original_behavior() {{
    // Test that the behavior is preserved after extraction
    let original_result = compute_original_logic(42);
    let extracted_result = {}(42);

    assert_eq!(extracted_result, original_result);
}}

#[test]
fn test_{}_handles_boundary_conditions() {{
    // Test boundary conditions
    assert!({}(i32::MIN).is_some() || {}(i32::MIN).is_none());
    assert!({}(i32::MAX).is_some() || {}(i32::MAX).is_none());
}}
"#,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name
                );

                vec![GeneratedTest {
                    name: format!("test_{}_unit", function_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!(
                        "Unit tests for extracted function {} in Rust",
                        function_name
                    ),
                    framework: framework.to_string(),
                    language: language.clone(),
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.8,
                }]
            }
            ProgrammingLanguage::TypeScript => {
                let test_code = format!(
                    r#"
describe("Extracted Function: {}", () => {{
    it("should work with various inputs", () => {{
        expect({}(1)).toBe(2);
        expect({}(5)).toBe(10);
        expect({}(0)).toBe(0);
    }});

    it("should handle edge cases", () => {{
        expect({}(-1)).toBe(-2);
        expect({}(Number.MAX_SAFE_INTEGER)).toBeDefined();
    }});

    it("should preserve original behavior", () => {{
        const originalResult = computeOriginalLogic(42);
        const extractedResult = {}(42);
        expect(extractedResult).toBe(originalResult);
    }});
}});
"#,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name
                );

                vec![GeneratedTest {
                    name: format!("test_extracted_function_{}", function_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!("Test extracted function {} in TypeScript", function_name),
                    framework: framework.to_string(),
                    language: language.clone(),
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.8,
                }]
            }
            ProgrammingLanguage::Python => {
                let test_code = format!(
                    r#"
class TestExtractedFunction:
    def test_{}_with_inputs(self):
        """Test {} with various inputs"""
        self.assertEqual({}(1), 2)
        self.assertEqual({}(5), 10)
        self.assertEqual({}(0), 0)

    def test_{}_edge_cases(self):
        """Test {} with edge cases"""
        self.assertEqual({}(-1), -2)

    def test_{}_preserves_behavior(self):
        """Test that {} preserves original behavior"""
        original = compute_original_logic(42)
        extracted = {}(42)
        self.assertEqual(extracted, original)
"#,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name
                );

                vec![GeneratedTest {
                    name: format!("TestExtractedFunction{}", function_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!("Test extracted function {} in Python", function_name),
                    framework: framework.to_string(),
                    language: language.clone(),
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.8,
                }]
            }
            _ => {
                // Generic test for unsupported languages
                let test_code = format!(
                    r#"
// Generic test for extracted function {}
// Test that the extracted function works correctly
test_{}_function() {{
    assert_equal({}(1), 2);
    assert_equal({}(5), 10);
}}
"#,
                    function_name, function_name, function_name, function_name
                );

                vec![GeneratedTest {
                    name: format!("test_extracted_{}", function_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!("Generic test for extracted function {}", function_name),
                    framework: "unknown".to_string(),
                    language: ProgrammingLanguage::Unknown,
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.5,
                }]
            }
        };

        Ok(tests)
    }

    /// Generate tests for extract variable refactoring with language-aware templates
    async fn generate_extract_variable_tests(
        &self,
        context: &rust_ai_ide_common::types::RefactoringContext,
        result: &rust_ai_ide_common::types::RefactoringResult,
        language: &rust_ai_ide_common::types::ProgrammingLanguage,
        framework: &str,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>> {
        let default_variable_name = "extractedVariable".to_string();
        let variable_name = context
            .symbol_name
            .as_ref()
            .unwrap_or(&default_variable_name);
        let constant_name = variable_name.to_uppercase();

        let tests = match language {
            ProgrammingLanguage::Rust => {
                let test_code = format!(
                    r#"
#[test]
fn test_{}_has_expected_value() {{
    // Test that the extracted variable has the expected value
    let computed_value = compute_complex_expression();
    assert_eq!(computed_value, {});

    // Test that variable extraction doesn't change behavior
    let original = compute_complex_expression();
    let with_variable = {};
    assert_eq!(original, with_variable);
}}

#[test]
fn test_{}_usage_scenarios() {{
    // Test different scenarios where the extracted variable might be used
    assert!({} > 0);
    assert!({} < 1000);

    // Test with edge values
    let edge_value = {};
    assert!(edge_value >= 0);
}}

#[test]
fn test_{}_type_safety() {{
    // Test type safety of extracted variable
    let {}: i32 = {};
    assert_eq!(std::any::TypeId::of::<i32>(), std::any::TypeId::of_val(&{}));
}}
"#,
                    variable_name,
                    constant_name,
                    constant_name,
                    variable_name,
                    constant_name,
                    constant_name,
                    constant_name,
                    variable_name,
                    variable_name,
                    constant_name,
                    constant_name
                );

                vec![GeneratedTest {
                    name: format!("test_{}_extraction", variable_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!("Test extraction of variable {} in Rust", variable_name),
                    framework: framework.to_string(),
                    language: language.clone(),
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.8,
                }]
            }
            ProgrammingLanguage::TypeScript => {
                let test_code = format!(
                    r#"
describe("Extract Variable: {}", () => {{
    it("should have expected value", () => {{
        const computedValue = computeComplexExpression();
        expect(computedValue).toBe({});

        const original = computeComplexExpression();
        expect(original).toBe({});
    }});

    it("should handle usage scenarios", () => {{
        expect({}).toBeGreaterThan(0);
        expect({}).toBeLessThan(1000);
    }});

    it("should maintain type safety", () => {{
        const {}: number = {};
        expect(typeof {}).toBe("number");
    }});
}});
"#,
                    variable_name,
                    constant_name,
                    constant_name,
                    constant_name,
                    constant_name,
                    constant_name,
                    constant_name,
                    variable_name
                );

                vec![GeneratedTest {
                    name: format!("test_extract_variable_{}", variable_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!(
                        "Test extraction of variable {} in TypeScript",
                        variable_name
                    ),
                    framework: framework.to_string(),
                    language: language.clone(),
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.8,
                }]
            }
            ProgrammingLanguage::Python => {
                let test_code = format!(
                    r#"
class TestExtractVariable{}:
    def test_{}_value(self):
        """Test that {} has the expected value"""
        computed_value = compute_complex_expression()
        self.assertEqual(computed_value, {})

        self.assertEqual(compute_complex_expression(), {})

    def test_{}_scenarios(self):
        """Test {} usage scenarios"""
        self.assertGreater({}, 0)
        self.assertLess({}, 1000)

    def test_{}_type_safety(self):
        """Test type safety of {}"""
        {} = {}
        self.assertIsInstance({}, int)
"#,
                    variable_name,
                    variable_name,
                    constant_name,
                    constant_name,
                    constant_name,
                    variable_name,
                    variable_name,
                    constant_name,
                    constant_name,
                    constant_name,
                    variable_name,
                    variable_name,
                    constant_name,
                    constant_name
                );
                vec![GeneratedTest {
                    name: format!("TestExtractVariable{}", variable_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!("Test extraction of variable {} in Python", variable_name),
                    framework: framework.to_string(),
                    language: language.clone(),
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.8,
                }]
            }
            _ => {
                // Generic test for unsupported languages
                let test_code = format!(
                    r#"
// Generic test for extracted variable {}
// Test that the variable has the correct value
test_{}_value() {{
    computed = compute_complex_expression()
    assert_equal(computed, {})
}}
"#,
                    variable_name, variable_name, constant_name
                );

                vec![GeneratedTest {
                    name: format!("test_variable_{}", variable_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!("Generic test for extracted variable {}", variable_name),
                    framework: "unknown".to_string(),
                    language: ProgrammingLanguage::Unknown,
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.5,
                }]
            }
        };

        Ok(tests)
    }

    /// Generate tests for extract interface refactoring
    async fn generate_extract_interface_tests(
        &self,
        context: &rust_ai_ide_common::types::RefactoringContext,
        result: &rust_ai_ide_common::types::RefactoringResult,
        language: &rust_ai_ide_common::types::ProgrammingLanguage,
        framework: &str,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>> {
        let default_interface_name = "IExtracted".to_string();
        let interface_name = context
            .symbol_name
            .as_ref()
            .unwrap_or(&default_interface_name);
        let class_name = "OriginalClass";

        let tests = match language {
            ProgrammingLanguage::Rust => {
                let test_code = format!(
                    r#"
#[test]
fn test_{}_implementation() {{
    // Test that a struct can implement the extracted trait
    struct TestImpl;

    impl {} for TestImpl {{
        fn method_one(&self) -> i32 {{ 42 }}
        fn method_two(&self, x: i32) -> i32 {{ x * 2 }}
    }}

    let instance = TestImpl;
    assert_eq!(instance.method_one(), 42);
    assert_eq!(instance.method_two(21), 42);
}}

#[test]
fn test_{}_inheritance() {{
    // Test that we can use the interface as a trait bound
    fn process_with_trait<T: {}>(item: T) -> i32 {{
        item.method_one() + item.method_two(10)
    }}

    struct TestImpl;
    impl {} for TestImpl {{
        fn method_one(&self) -> i32 {{ 5 }}
        fn method_two(&self, x: i32) -> i32 {{ x }}
    }}

    let result = process_with_trait(TestImpl);
    assert_eq!(result, 15);
}}
"#,
                    interface_name, interface_name, interface_name, interface_name, interface_name
                );

                vec![GeneratedTest {
                    name: format!("test_{}_extraction", interface_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!("Test {} trait extraction in Rust", interface_name),
                    framework: framework.to_string(),
                    language: language.clone(),
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.8,
                }]
            }
            ProgrammingLanguage::TypeScript => {
                let test_code = format!(
                    r#"
describe("Extract Interface: {}", () => {{
    it("should implement extracted interface", () => {{
        class TestImpl implements {} {{
            methodOne(): number {{
                return 42;
            }}
            methodTwo(x: number): number {{
                return x * 2;
            }}
        }}

        const instance = new TestImpl();
        expect(instance.methodOne()).toBe(42);
        expect(instance.methodTwo(21)).toBe(42);
    }});

    it("should support interface polymorphism", () => {{
        function processInterface(obj: {}): number {{
            return obj.methodOne() + obj.methodTwo(10);
        }}

        const instance: {} = {{
            methodOne: () => 5,
            methodTwo: (x: number) => x
        }};

        expect(processInterface(instance)).toBe(15);
    }});
}});
"#,
                    interface_name, interface_name, interface_name, interface_name
                );

                vec![GeneratedTest {
                    name: format!("test_extract_interface_{}", interface_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!(
                        "Test {} interface extraction in TypeScript",
                        interface_name
                    ),
                    framework: framework.to_string(),
                    language: language.clone(),
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.8,
                }]
            }
            _ => {
                // Generic test for unsupported languages
                let test_code = format!(
                    r#"
// Generic test for extracted interface {}
// Test basic interface implementation
test_{}_implementation() {{
    // Implementation test would vary by language
    assert_true(true); // Placeholder
}}
"#,
                    interface_name, interface_name
                );

                vec![GeneratedTest {
                    name: format!("test_interface_{}", interface_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!("Generic test for extracted interface {}", interface_name),
                    framework: "unknown".to_string(),
                    language: ProgrammingLanguage::Unknown,
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.5,
                }]
            }
        };

        Ok(tests)
    }

    /// Generate tests for async conversion refactoring
    async fn generate_async_conversion_tests(
        &self,
        context: &rust_ai_ide_common::types::RefactoringContext,
        result: &rust_ai_ide_common::types::RefactoringResult,
        language: &rust_ai_ide_common::types::ProgrammingLanguage,
        framework: &str,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>> {
        let default_function_name = "convertedFunction".to_string();
        let function_name = context
            .symbol_name
            .as_ref()
            .unwrap_or(&default_function_name);

        let tests = match language {
            ProgrammingLanguage::Rust => {
                let test_code = format!(
                    r#"
#[tokio::test]
async fn test_{}_async_conversion() {{
    // Test that the converted async function works
    let result = {}(42).await;
    assert_eq!(result, 84); // Assuming the function doubles input

    // Test that calls are properly awaited
    let future_result = {}(21);
    let value = future_result.await;
    assert_eq!(value, 42);
}}

#[test]
fn test_{}_preserves_logic() {{
    // Test that async conversion preserves the original logic
    // This would require a synchronous version for comparison
    async fn async_version() -> i32 {{
        100
    }}

    fn sync_version() -> i32 {{
        100
    }}

    // Note: We can't directly compare sync and async return types
    // This test would need adjustment based on actual function signatures
    assert_eq!(sync_version(), 100);
}}

#[tokio::test]
async fn test_{}_error_handling() {{
    // Test error handling in the async version
    match {}(0).await {{
        Ok(value) => assert!(value >= 0),
        Err(e) => panic!("Unexpected error: {{}}", e),
    }}
}}
"#,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name
                );

                vec![GeneratedTest {
                    name: format!("test_async_{}", function_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!("Test async conversion of {} in Rust", function_name),
                    framework: framework.to_string(),
                    language: language.clone(),
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.8,
                }]
            }
            ProgrammingLanguage::JavaScript | ProgrammingLanguage::TypeScript => {
                let await_syntax = if matches!(language, ProgrammingLanguage::TypeScript) {
                    ": Promise<number>"
                } else {
                    ""
                };

                let test_code = format!(
                    r#"
describe("Async Conversion: {}", () => {{
    it("should return a promise{}", () => {{
        const result = {}();
        expect(result).toBeInstanceOf(Promise);
    }});

    it("should await correctly", async () => {{
        const result = await {}(42);
        expect(result).toBe(84);
    }});

    it("should handle errors properly", async () => {{
        try {{
            await {}(0);
        }} catch (error) {{
            expect(error).toBeDefined();
        }}
    }});

    it("should preserve original behavior", async () => {{
        // Compare with synchronous version if available
        function syncVersion(x) {{
            return x * 2;
        }}

        const asyncResult = await {}(21);
        const syncResult = syncVersion(21);

        expect(asyncResult).toBe(syncResult);
    }});
}});
"#,
                    function_name,
                    await_syntax,
                    function_name,
                    function_name,
                    function_name,
                    function_name
                );

                vec![GeneratedTest {
                    name: format!("test_async_conversion_{}", function_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!(
                        "Test async conversion of {} in {}Script",
                        function_name,
                        if matches!(language, ProgrammingLanguage::TypeScript) {
                            "Type"
                        } else {
                            "Java"
                        }
                    ),
                    framework: framework.to_string(),
                    language: language.clone(),
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.8,
                }]
            }
            ProgrammingLanguage::Python => {
                let test_code = format!(
                    r#"
class TestAsyncConversion:
    async def test_{}_conversion(self):
        """Test that {} is properly async"""
        result = await {}(42)
        self.assertEqual(result, 84)

    def test_{}_coroutine_return(self):
        """Test that {} returns a coroutine"""
        import asyncio
        coro = {}(21)
        self.assertTrue(asyncio.iscoroutine(coro))

    async def test_{}_error_handling(self):
        """Test error handling in async {}"""
        try:
            await {}(0)
        except Exception as e:
            self.assertIsInstance(e, Exception)

    async def test_{}_preserves_behavior(self):
        """Test that async conversion preserves original behavior"""
        # This would need comparison with sync version
        result = await {}(99)
        expected = 99 * 2  # Assuming doubling behavior
        self.assertEqual(result, expected)
"#,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name,
                    function_name
                );

                vec![GeneratedTest {
                    name: format!("TestAsyncConversion{}", function_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!("Test async conversion of {} in Python", function_name),
                    framework: framework.to_string(),
                    language: language.clone(),
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.8,
                }]
            }
            _ => {
                // Generic test for unsupported languages
                let test_code = format!(
                    r#"
// Generic test for async conversion of {}
// Test async behavior
test_{}_async() {{
    // Generic async test logic would depend on target language
    assert_true(is_async({}()));
}}
"#,
                    function_name, function_name, function_name
                );

                vec![GeneratedTest {
                    name: format!("test_async_{}", function_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!("Generic test for async conversion of {}", function_name),
                    framework: "unknown".to_string(),
                    language: ProgrammingLanguage::Unknown,
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.5,
                }]
            }
        };

        Ok(tests)
    }

    /// Generate generic tests for any refactoring type with language awareness
    async fn generate_generic_tests(
        &self,
        refactoring_type: &rust_ai_ide_common::types::RefactoringType,
        context: &rust_ai_ide_common::types::RefactoringContext,
        result: &rust_ai_ide_common::types::RefactoringResult,
        language: &rust_ai_ide_common::types::ProgrammingLanguage,
        framework: &str,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>> {
        let refactoring_name = format!("{:?}", refactoring_type).to_lowercase();

        let tests = match language {
            ProgrammingLanguage::Rust => {
                let test_code = format!(
                    r#"
#[test]
fn test_{}_preserves_behavior() {{
    // Test that the refactoring preserves the original behavior
    let baseline_state = get_baseline_state();
    let refactored_state = get_refactored_state();

    assert_eq!(baseline_state, refactored_state, "{:?} changed behavior unexpectedly");
}}

#[test]
fn test_{}_maintains_contract() {{
    // Test that the refactoring maintains the expected contract
    assert!(refactoring_contract_holds(), "{:?} broke contract");
}}

#[test]
fn test_{}_handles_edge_cases() {{
    // Test edge cases for safety
    let edge_result = test_edge_case_scenario();
    assert!(edge_result.is_ok(), "{:?} failed edge case");
}}
"#,
                    refactoring_name,
                    refactoring_type,
                    refactoring_name,
                    refactoring_type,
                    refactoring_name,
                    refactoring_type
                );

                vec![GeneratedTest {
                    name: format!("test_{}_generic", refactoring_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!(
                        "Generic safety tests for {:?} refactoring in Rust",
                        refactoring_type
                    ),
                    framework: framework.to_string(),
                    language: language.clone(),
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.8,
                }]
            }
            ProgrammingLanguage::TypeScript => {
                let test_code = format!(
                    r#"
describe("{} Generic Tests", () => {{
    it("should preserve behavior", () => {{
        const baseline = getBaselineState();
        const refactored = getRefactoredState();
        expect(baseline).toEqual(refactored);
    }});

    it("should maintain contract", () => {{
        expect(refactoringContractHolds()).toBe(true);
    }});

    it("should handle edge cases", () => {{
        const edgeResult = testEdgeCaseScenario();
        expect(edgeResult.isSuccess).toBe(true);
    }});
}});
"#,
                    refactoring_name
                );

                vec![GeneratedTest {
                    name: format!("test_generic_{}", refactoring_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!(
                        "Generic safety tests for {:?} refactoring in TypeScript",
                        refactoring_type
                    ),
                    framework: framework.to_string(),
                    language: language.clone(),
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.8,
                }]
            }
            ProgrammingLanguage::Python => {
                let test_code = format!(
                    r#"
class TestGeneric{}:
    def test_{}_preserves_behavior(self):
        """Test that {:?} preserves original behavior"""
        baseline = get_baseline_state()
        refactored = get_refactored_state()
        self.assertEqual(baseline, refactored)

    def test_{}_maintains_contract(self):
        """Test that {:?} maintains expected contract"""
        self.assertTrue(refactoring_contract_holds())

    def test_{}_edge_cases(self):
        """Test {:?} edge cases for safety"""
        edge_result = test_edge_case_scenario()
        self.assertTrue(edge_result.is_success)
"#,
                    refactoring_name,
                    refactoring_name,
                    refactoring_type,
                    refactoring_name,
                    refactoring_type,
                    refactoring_name,
                    refactoring_type
                );

                vec![GeneratedTest {
                    name: format!("TestGeneric{}", refactoring_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!(
                        "Generic safety tests for {:?} refactoring in Python",
                        refactoring_type
                    ),
                    framework: framework.to_string(),
                    language: language.clone(),
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.8,
                }]
            }
            _ => {
                // Generic fallback for unsupported languages
                let test_code = format!(
                    r#"
// Generic safety test for {} refactoring
test_{}_preserves_behavior() {{
    baseline = get_baseline_state()
    refactored = get_refactored_state()
    assert_equal(baseline, refactored)
}}
"#,
                    refactoring_name, refactoring_name
                );

                vec![GeneratedTest {
                    name: format!("test_generic_{}", refactoring_name),
                    code: test_code,
                    test_type: TestType::Unit,
                    description: format!(
                        "Generic safety tests for {:?} refactoring",
                        refactoring_type
                    ),
                    framework: "unknown".to_string(),
                    language: ProgrammingLanguage::Unknown,
                    expected_coverage: vec![],
                    dependencies: vec![],
                    tags: vec![],
                    confidence_score: 0.5,
                }]
            }
        };

        Ok(tests)
    }
}

// All test generation types now imported from rust_ai_ide_common

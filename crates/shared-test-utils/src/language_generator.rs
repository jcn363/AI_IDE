use crate::error::TestError;
use crate::test_generation::*;
use async_trait::async_trait;
use std::collections::HashMap;

/// Trait for language-specific test generation
/// Consolidates language-specific testing capabilities
#[async_trait]
pub trait LanguageTestGenerator: Send + Sync {
    /// Get the programming language this generator supports
    fn supported_language(&self) -> ProgrammingLanguage;

    /// Generate unit tests for the given code
    async fn generate_unit_tests(
        &self,
        code: &str,
        context: &TestGenerationContext,
    ) -> Result<Vec<GeneratedTest>, TestError>;

    /// Generate integration tests for the given code
    async fn generate_integration_tests(
        &self,
        code: &str,
        context: &TestGenerationContext,
    ) -> Result<Vec<GeneratedTest>, TestError>;

    /// Generate property-based tests
    async fn generate_property_tests(
        &self,
        code: &str,
        context: &TestGenerationContext,
    ) -> Result<Vec<GeneratedTest>, TestError>;

    /// Generate performance benchmarks
    async fn generate_benchmarks(
        &self,
        code: &str,
        context: &TestGenerationContext,
    ) -> Result<Vec<GeneratedTest>, TestError>;

    /// Generate tests for function extraction refactoring
    async fn generate_extract_function_tests(
        &self,
        context: &RefactoringContext,
        result: &RefactoringResult,
    ) -> Result<Vec<GeneratedTest>, TestError>;

    /// Generate tests for variable extraction refactoring
    async fn generate_extract_variable_tests(
        &self,
        context: &RefactoringContext,
        result: &RefactoringResult,
    ) -> Result<Vec<GeneratedTest>, TestError>;

    /// Generate tests for rename refactoring
    async fn generate_rename_tests(
        &self,
        context: &RefactoringContext,
        result: &RefactoringResult,
    ) -> Result<Vec<GeneratedTest>, TestError>;

    /// Generate tests for async conversion refactoring
    async fn generate_async_conversion_tests(
        &self,
        context: &RefactoringContext,
        result: &RefactoringResult,
    ) -> Result<Vec<GeneratedTest>, TestError>;

    /// Generate tests for interface extraction
    async fn generate_extract_interface_tests(
        &self,
        context: &RefactoringContext,
        result: &RefactoringResult,
    ) -> Result<Vec<GeneratedTest>, TestError>;

    /// Get framework-specific test templates
    fn get_test_templates(&self) -> HashMap<TestFramework, TestTemplateSet> {
        HashMap::new()
    }

    /// Validate that the generator can work with the given code
    async fn validate_code(&self, code: &str) -> Result<(), TestError>;

    /// Estimate test coverage for generated tests
    async fn estimate_coverage(
        &self,
        tests: &[GeneratedTest],
        code: &str,
    ) -> Result<Vec<TestCoverage>, TestError>;
}

/// Test framework enumeration
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum TestFramework {
    CargoTest,
    Jest,
    JUnit,
    NUnit,
    Pytest,
    XUnit,
    Vitest,
    Mocha,
    Custom(String),
}

/// Test template set for a framework
#[derive(Debug, Clone)]
pub struct TestTemplateSet {
    pub unit_test_template: String,
    pub integration_test_template: String,
    pub property_test_template: Option<String>,
    pub benchmark_template: Option<String>,
    pub assert_true_template: String,
    pub assert_equal_template: String,
    pub assert_not_null_template: String,
}

/// Factory for creating language generators
#[async_trait]
pub trait LanguageGeneratorFactory {
    async fn create_generator(
        &self,
        language: &ProgrammingLanguage,
    ) -> Result<Box<dyn LanguageTestGenerator>, TestError>;
}

/// Registry for available language generators
pub struct LanguageGeneratorRegistry {
    generators: HashMap<ProgrammingLanguage, Box<dyn LanguageGeneratorFactory>>,
}

impl LanguageGeneratorRegistry {
    pub fn new() -> Self {
        Self {
            generators: HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, language: ProgrammingLanguage, factory: F)
    where
        F: LanguageGeneratorFactory + 'static,
    {
        self.generators.insert(language, Box::new(factory));
    }

    pub async fn create_generator(
        &self,
        language: &ProgrammingLanguage,
    ) -> Result<Box<dyn LanguageTestGenerator>, TestError> {
        let factory = self.generators.get(language).ok_or_else(|| {
            use crate::error::ValidationError;
            TestError::Validation(ValidationError::InvalidTestSetup {
                message: format!(
                    "No generator factory registered for language: {:?}",
                    language
                ),
            })
        })?;

        factory.create_generator(language).await
    }

    pub fn supported_languages(&self) -> Vec<ProgrammingLanguage> {
        self.generators.keys().cloned().collect()
    }
}

impl Default for LanguageGeneratorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Composite generator that delegates to language-specific generators
pub struct UnifiedLanguageTestGenerator {
    registry: LanguageGeneratorRegistry,
}

impl UnifiedLanguageTestGenerator {
    pub fn new(registry: LanguageGeneratorRegistry) -> Self {
        Self { registry }
    }

    pub async fn get_generator(
        &self,
        language: &ProgrammingLanguage,
    ) -> Result<Box<dyn LanguageTestGenerator>, TestError> {
        self.registry.create_generator(language).await
    }

    /// Generate tests using appropriate language generator
    pub async fn generate_tests(
        &self,
        code: &str,
        context: &TestGenerationContext,
        language: &ProgrammingLanguage,
        test_type: TestType,
    ) -> Result<Vec<GeneratedTest>, TestError> {
        let generator = self.get_generator(language).await?;

        match test_type {
            TestType::Unit => generator.generate_unit_tests(code, context).await,
            TestType::Integration => generator.generate_integration_tests(code, context).await,
            TestType::Property => {
                if generator
                    .get_test_templates()
                    .values()
                    .any(|t| t.property_test_template.is_some())
                {
                    generator.generate_property_tests(code, context).await
                } else {
                    Ok(vec![])
                }
            }
            TestType::Benchmark => {
                if generator
                    .get_test_templates()
                    .values()
                    .any(|t| t.benchmark_template.is_some())
                {
                    generator.generate_benchmarks(code, context).await
                } else {
                    Ok(vec![])
                }
            }
            TestType::Fuzz | TestType::E2e => {
                // Fuzz and E2E not implemented yet
                Ok(vec![])
            }
        }
    }
}

/// Helper macro for implementing LanguageTestGenerator
#[macro_export]
macro_rules! language_generator_impl {
    ($struct_name:ident, $language:expr) => {
        #[async_trait]
        impl LanguageTestGenerator for $struct_name {
            fn supported_language(&self) -> ProgrammingLanguage {
                $language
            }

            async fn generate_unit_tests(
                &self,
                _code: &str,
                _context: &TestGenerationContext,
            ) -> Result<Vec<GeneratedTest>, TestError> {
                Ok(vec![]) // Placeholder implementation
            }

            async fn generate_integration_tests(
                &self,
                _code: &str,
                _context: &TestGenerationContext,
            ) -> Result<Vec<GeneratedTest>, TestError> {
                Ok(vec![]) // Placeholder implementation
            }

            async fn generate_property_tests(
                &self,
                _code: &str,
                _context: &TestGenerationContext,
            ) -> Result<Vec<GeneratedTest>, TestError> {
                Ok(vec![]) // Placeholder implementation
            }

            async fn generate_benchmarks(
                &self,
                _code: &str,
                _context: &TestGenerationContext,
            ) -> Result<Vec<GeneratedTest>, TestError> {
                Ok(vec![]) // Placeholder implementation
            }

            async fn generate_extract_function_tests(
                &self,
                _context: &RefactoringContext,
                _result: &RefactoringResult,
            ) -> Result<Vec<GeneratedTest>, TestError> {
                Ok(vec![]) // Placeholder implementation
            }

            async fn generate_extract_variable_tests(
                &self,
                _context: &RefactoringContext,
                _result: &RefactoringResult,
            ) -> Result<Vec<GeneratedTest>, TestError> {
                Ok(vec![]) // Placeholder implementation
            }

            async fn generate_rename_tests(
                &self,
                _context: &RefactoringContext,
                _result: &RefactoringResult,
            ) -> Result<Vec<GeneratedTest>, TestError> {
                Ok(vec![]) // Placeholder implementation
            }

            async fn generate_async_conversion_tests(
                &self,
                _context: &RefactoringContext,
                _result: &RefactoringResult,
            ) -> Result<Vec<GeneratedTest>, TestError> {
                Ok(vec![]) // Placeholder implementation
            }

            async fn generate_extract_interface_tests(
                &self,
                _context: &RefactoringContext,
                _result: &RefactoringResult,
            ) -> Result<Vec<GeneratedTest>, TestError> {
                Ok(vec![]) // Placeholder implementation
            }

            async fn validate_code(&self, _code: &str) -> Result<(), TestError> {
                Ok(()) // Default implementation always passes
            }

            async fn estimate_coverage(
                &self,
                _tests: &[GeneratedTest],
                _code: &str,
            ) -> Result<Vec<TestCoverage>, TestError> {
                Ok(vec![]) // Default empty implementation
            }
        }
    };
}

/// Default language generators
pub mod generators {
    use super::*;
    // Removed unused import - test_generation types are accessed via crate::test_generation path

    pub struct RustTestGenerator;

    pub struct TypeScriptTestGenerator;

    pub struct PythonTestGenerator;

    pub struct JavaTestGenerator;

    language_generator_impl!(RustTestGenerator, ProgrammingLanguage::Rust);
    language_generator_impl!(TypeScriptTestGenerator, ProgrammingLanguage::TypeScript);
    language_generator_impl!(PythonTestGenerator, ProgrammingLanguage::Python);
    language_generator_impl!(JavaTestGenerator, ProgrammingLanguage::Java);

    // Factory implementations for each language
    pub struct RustGeneratorFactory;

    #[async_trait]
    impl LanguageGeneratorFactory for RustGeneratorFactory {
        async fn create_generator(
            &self,
            _language: &ProgrammingLanguage,
        ) -> Result<Box<dyn LanguageTestGenerator>, TestError> {
            Ok(Box::new(RustTestGenerator))
        }
    }

    pub struct TypeScriptGeneratorFactory;

    #[async_trait]
    impl LanguageGeneratorFactory for TypeScriptGeneratorFactory {
        async fn create_generator(
            &self,
            _language: &ProgrammingLanguage,
        ) -> Result<Box<dyn LanguageTestGenerator>, TestError> {
            Ok(Box::new(TypeScriptTestGenerator))
        }
    }

    pub struct PythonGeneratorFactory;

    #[async_trait]
    impl LanguageGeneratorFactory for PythonGeneratorFactory {
        async fn create_generator(
            &self,
            _language: &ProgrammingLanguage,
        ) -> Result<Box<dyn LanguageTestGenerator>, TestError> {
            Ok(Box::new(PythonTestGenerator))
        }
    }

    pub struct JavaGeneratorFactory;

    #[async_trait]
    impl LanguageGeneratorFactory for JavaGeneratorFactory {
        async fn create_generator(
            &self,
            _language: &ProgrammingLanguage,
        ) -> Result<Box<dyn LanguageTestGenerator>, TestError> {
            Ok(Box::new(JavaTestGenerator))
        }
    }
}

// Removed duplicate HashMap import - already imported at top of file (line 3)

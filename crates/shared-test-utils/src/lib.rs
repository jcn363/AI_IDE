//! # Shared Test Utilities
//!
//! A comprehensive testing utilities library for the Rust AI IDE workspace, providing:
//! - Common assertions and advanced validation patterns
//! - Enhanced async testing helpers with concurrency testing
//! - Database testing fixtures (SQLite, PostgreSQL)
//! - Cache testing utilities and mock implementations
//! - LSP (Language Server Protocol) testing fixtures
//! - Unified mock framework with common patterns
//! - CI/CD integration helpers (coverage, benchmarking, reporting)
//! - Tauri command testing framework
//! - File system testing abstractions
//! - Test fixtures and factories

pub mod assertions;
pub mod async_utils;
pub mod builder;
pub mod command_tests;
pub mod compatibility;
pub mod data_generator;
pub mod debugger_integration;
pub mod error;
pub mod filesystem;
pub mod fixtures;
pub mod harness;
pub mod integration;
pub mod macros;
pub mod setup;
pub mod test_generation;
pub mod validation;

// New enhanced testing modules
pub mod cache;
#[cfg(feature = "ci")]
pub mod ci;
pub mod database;
pub mod lsp;
pub mod mock;

// Re-export commonly used types
// Re-export migrated utilities
pub use assertions::*;
pub use async_utils::{run_concurrent, timeout_task, with_timeout, AsyncContext};
// Enhanced async testing utilities
pub use async_utils::{
    AsyncScheduler, AsyncStressTester, AsyncTestHelper, AsyncTestHooks, ConcurrencyTester, DeadlockDetector,
    TokioTestUtils,
};
pub use builder::TestFixtureBuilder as GenericTestFixtureBuilder; // Generic builder
// Cache testing utilities
pub use cache::{CacheFixture, CacheFixtures, CachePerformanceTester, MockCache, MultiLevelCacheFixture};
// CI/CD integration utilities
#[cfg(feature = "ci")]
pub use ci::{BenchmarkRunner, CICommandRunner, CIEnvironment, CIPipeline, CoverageAnalyzer, TestReporter};
pub use command_tests::CommandTestRunner;
pub use compatibility::*; // Export compatibility layer
pub use data_generator::*;
// Database testing utilities
#[cfg(feature = "database")]
pub use database::{DatabaseConfig, DatabaseFixture, DatabaseFixtures, DatabaseTransaction, MockDatabase};
pub use error::{TestError, ValidationError};
pub use filesystem::TempWorkspace;
pub use fixtures::TestFixtureBuilder;
pub use harness::{TestHarness, TestHarnessFactory, TestHarnessMetadata, TestHarnessSuite, TestResult}; /* Export specific harness types */
pub use integration::IntegrationContext;
// LSP testing utilities
pub use lsp::{LSPAssertions, LSPFixture, LSPFixtures, LSPMessageBuilder, MockLSPClient, MockLSPServer};
// Mock framework utilities
pub use mock::{FileSystemMock, GenericMock, HttpMock, MockBehaviors, MockFactory, MockPresets, MockScenario};
pub use setup::*;
pub use test_generation::{
    GeneratedTest, GeneratedTests, LanguageDetector, LanguageGeneratorFactory, LanguageGeneratorRegistry,
    LanguageTestGenerator, ProgrammingLanguage, RefactoringContext, RefactoringResult, RefactoringType, TestFramework,
    TestGenerationConfig, TestGenerationContext, TestTemplateSet, TestType, UnifiedLanguageTestGenerator,
    UnifiedTestGenerator,
};
pub use validation::ValidationUtils;

// Usage Pattern: Basic Test Fixture Creation
// ```ignore
// use shared_test_utils::*;
//
// let workspace = TempWorkspace::new().unwrap();
// let fixture = TestFixtureBuilder::new()
//     .with_file("test.rs", "fn test() { assert!(true); }")
//     .with_directory("src")
//     .build(&workspace).unwrap();
// ```
//
// Usage Pattern: Async Testing with Timeouts
// ```ignore
// use shared_test_utils::*;
//
// let result = with_timeout(async {
//     tokio::time::sleep(Duration::from_millis(100)).await;
//     "completed"
// }, Duration::from_millis(200)).await;
// assert!(result.is_ok());
// ```
//
// Usage Pattern: Test Harness Implementation
// ```ignore
// use shared_test_utils::*;
// use async_trait::async_trait;
//
// #[async_trait]
// impl TestHarness for MyTest {
//     type Context = MyData;
//     type Input = String;
//     type Output = bool;
//
//     async fn setup(&self, _input: Self::Input) -> Result<Self::Context, TestError> {
//         Ok(MyData::new())
//     }
//
//     async fn execute(&mut self, _context: Self::Context) -> Result<Self::Output, TestError> {
//         Ok(true)
//     }
//
//     async fn validate(&self, _context: Self::Context, _output: Self::Output) -> Result<TestResult, TestError> {
//         Ok(TestResult {
//             passed: true,
//             message: "Test passed".to_string(),
//             details: None,
//             duration: Duration::from_millis(50)
//         })
//     }
// }
// ```
//
// Usage Pattern: Language Detection and Test Generation
// ```ignore
// use shared_test_utils::*;
//
// let detector = LanguageDetector::default();
// let code = "fn main() { println!(\"Hello\"); }";
// let language = detector.detect_language(code);
// assert_eq!(language, Some(ProgrammingLanguage::Rust));
// ```

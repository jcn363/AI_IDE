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

pub mod error;
pub mod validation;
pub mod filesystem;
pub mod async_utils;
pub mod fixtures;
pub mod builder;
pub mod harness;
pub mod compatibility;
pub mod debugger_integration;
pub mod macros;
pub mod command_tests;
pub mod integration;
pub mod test_generation;
pub mod data_generator;
pub mod setup;
pub mod assertions;

// New enhanced testing modules
pub mod database;
pub mod cache;
pub mod lsp;
pub mod mock;
#[cfg(feature = "ci")]
pub mod ci;

// Re-export commonly used types
pub use error::{TestError, ValidationError};
pub use validation::ValidationUtils;
pub use filesystem::TempWorkspace;
pub use async_utils::{with_timeout, timeout_task, run_concurrent, AsyncContext};
pub use fixtures::TestFixtureBuilder;
pub use builder::TestFixtureBuilder as GenericTestFixtureBuilder; // Generic builder
pub use harness::{TestHarness, TestResult, TestHarnessMetadata, TestHarnessFactory, TestHarnessSuite}; // Export specific harness types
pub use compatibility::*; // Export compatibility layer
pub use command_tests::CommandTestRunner;
pub use integration::IntegrationContext;
pub use test_generation::{
    UnifiedTestGenerator, TestGenerationContext, RefactoringContext, RefactoringResult,
    ProgrammingLanguage, TestType, RefactoringType, GeneratedTest, GeneratedTests,
    LanguageDetector, TestGenerationConfig,
    LanguageTestGenerator, LanguageGeneratorRegistry, UnifiedLanguageTestGenerator,
    TestFramework, TestTemplateSet, LanguageGeneratorFactory
};

// Enhanced async testing utilities
pub use async_utils::{
    ConcurrencyTester, AsyncScheduler, AsyncStressTester, AsyncTestHooks, DeadlockDetector,
    TokioTestUtils, AsyncTestHelper,
};

// Database testing utilities
#[cfg(feature = "database")]
pub use database::{DatabaseFixture, DatabaseConfig, MockDatabase, DatabaseTransaction, DatabaseFixtures};

// Cache testing utilities
pub use cache::{MockCache, CacheFixture, CacheFixtures, CachePerformanceTester, MultiLevelCacheFixture};

// LSP testing utilities
pub use lsp::{MockLSPServer, MockLSPClient, LSPFixture, LSPFixtures, LSPMessageBuilder, LSPAssertions};

// Mock framework utilities
pub use mock::{GenericMock, MockBehaviors, MockFactory, MockScenario, MockPresets, HttpMock, FileSystemMock};

// CI/CD integration utilities
#[cfg(feature = "ci")]
pub use ci::{CIEnvironment, CoverageAnalyzer, BenchmarkRunner, TestReporter, CIPipeline, CICommandRunner};

// Re-export migrated utilities
pub use data_generator::*;
pub use setup::*;
pub use assertions::*;

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
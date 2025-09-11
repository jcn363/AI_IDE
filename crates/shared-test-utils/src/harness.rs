use crate::error::TestError;
use async_trait::async_trait;
use std::fmt::Debug;

/// Core trait for unified test harness
/// Follows setup-execute-validate async pattern
#[async_trait]
pub trait TestHarness: Send + Sync {
    /// Test context type that this harness manages
    type Context: Send + Sync + Debug + Default + Clone;

    /// Input type for the test execution
    type Input: Send + Sync + Debug + Clone;

    /// Output type from the test execution
    type Output: Send + Sync + Debug;

    /// Setup phase: Prepare test context and fixtures
    /// Returns prepared context for the test
    async fn setup(&self, input: Self::Input) -> Result<Self::Context, TestError>;

    /// Execute phase: Run the actual test logic
    /// Returns output that can be validated
    async fn execute(&mut self, context: Self::Context) -> Result<Self::Output, TestError>;

    /// Validate phase: Assert expected behavior and state
    /// Returns validation results or errors
    async fn validate(
        &self,
        context: Self::Context,
        output: Self::Output,
    ) -> Result<TestResult, TestError>;

    /// Cleanup phase: tear down test resources (optional)
    async fn cleanup(&self, _context: Self::Context) -> Result<(), TestError> {
        Ok(()) // Default implementation does nothing
    }

    /// Complete test flow: setup -> execute -> validate -> cleanup
    async fn run_test(&mut self, input: Self::Input) -> Result<TestResult, TestError> {
        let context = self.setup(input).await?;
        let output = self.execute(context.clone()).await?;
        let result = self.validate(context.clone(), output).await?;

        // Run cleanup regardless of validation result
        let cleanup_result = self.cleanup(context).await;
        match cleanup_result {
            Ok(_) => Ok(result),
            Err(e) => Err(TestError::Cleanup(e.to_string())),
        }
    }

    /// Get harness metadata for diagnostics
    fn metadata(&self) -> TestHarnessMetadata {
        TestHarnessMetadata {
            name: std::any::type_name::<Self>().to_string(),
            version: "1.0.0".to_string(),
            description: "Unified test harness implementation".to_string(),
        }
    }
}

/// Test result returned by validation phase
#[derive(Debug, Clone)]
pub struct TestResult {
    pub passed: bool,
    pub message: String,
    pub details: Option<TestDetails>,
    pub duration: std::time::Duration,
}

/// Detailed test information
#[derive(Debug, Clone)]
pub struct TestDetails {
    pub assertions_made: Vec<String>,
    pub expected_vs_actual: Option<(String, String)>,
    pub additional_data: std::collections::HashMap<String, String>,
}

/// Metadata for test harness identification and diagnostics
#[derive(Debug, Clone)]
pub struct TestHarnessMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
}

/// Factory for creating test harness instances
#[async_trait]
pub trait TestHarnessFactory {
    async fn create_harness(
        &self,
    ) -> Result<Box<dyn TestHarness<Context = (), Input = (), Output = ()> + Send + Sync>, TestError>;
}

/// Utility for composing multiple test harnesses
pub struct TestHarnessSuite<T: TestHarness> {
    harnesses: Vec<T>,
}

impl<T: TestHarness> TestHarnessSuite<T> {
    pub fn new() -> Self {
        Self {
            harnesses: Vec::new(),
        }
    }

    pub fn add_harness(mut self, harness: T) -> Self {
        self.harnesses.push(harness);
        self
    }

    /// Run all harnesses in the suite
    pub async fn run_all(
        &mut self,
        inputs: Vec<T::Input>,
    ) -> Result<Vec<Result<TestResult, TestError>>, TestError>
    where
        T: Clone,
    {
        if inputs.len() != self.harnesses.len() {
            return Err(TestError::Validation(
                crate::ValidationError::invalid_setup(format!(
                    "Input count ({}) doesn't match harness count ({})",
                    inputs.len(),
                    self.harnesses.len()
                )),
            ));
        }

        let mut results = Vec::new();
        for (harness, input) in self.harnesses.iter_mut().zip(inputs.iter()) {
            let mut harness = harness.clone();
            let result = harness.run_test(input.clone()).await;
            results.push(result);
        }
        Ok(results)
    }
}

impl<T: TestHarness> Default for TestHarnessSuite<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for validating test harness configurations
#[async_trait]
pub trait TestHarnessValidator {
    async fn validate_harness(&self, input: &[u8]) -> Result<(), TestError>;
}

impl Default for TestResult {
    fn default() -> Self {
        Self {
            passed: false,
            message: "Test not executed".to_string(),
            details: None,
            duration: std::time::Duration::from_nanos(0),
        }
    }
}

/// Helper macro for creating test harness implementations
#[macro_export]
macro_rules! test_harness {
    ($struct_name:ident, $context_type:ty) => {
        impl TestHarness for $struct_name {
            type Context = $context_type;
        }
    };
    ($struct_name:ident, $context_type:ty, $input_type:ty, $output_type:ty) => {
        impl TestHarness for $struct_name {
            type Context = $context_type;
            type Input = $input_type;
            type Output = $output_type;
        }
    };
}

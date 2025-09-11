use async_trait::async_trait;
use crate::types::*;

/// Core trait for all refactoring operations
#[async_trait]
pub trait RefactoringOperation {
    /// Execute the refactoring operation
    async fn execute(
        &self,
        context: &RefactoringContext,
        options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>>;

    /// Analyze the refactoring operation before execution
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>>;

    /// Check if this operation is applicable in the given context
    async fn is_applicable(
        &self,
        context: &RefactoringContext,
        options: Option<&RefactoringOptions>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;

    /// Get the type of refactoring this operation implements
    fn refactoring_type(&self) -> RefactoringType;

    /// Get a user-friendly name for this operation
    fn name(&self) -> &str;

    /// Get a description of this operation
    fn description(&self) -> &str;

    /// Check if experimental features are enabled
    fn is_experimental_enabled(&self, options: &RefactoringOptions) -> bool {
        options
            .extra_options
            .as_ref()
            .and_then(|opts| opts.get("experimental"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }
}
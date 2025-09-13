//! Extensible Traits for AI Components
//! ====================================
//!
//! This module defines the core traits that provide extensibility and plugin
//! interfaces for AI components throughout the Rust AI IDE.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::ai::{AIContext, AIProvider, AnalysisConfig, AnalysisResult, ComponentStatus, ModelConfig, ModelState};
use super::analysis::AnalysisTarget;
use super::config::AppConfig;
use super::error::IDEResult;

/// Component lifecycle trait
#[async_trait]
pub trait ComponentLifecycle: Send + Sync {
    /// Initialize the component
    async fn initialize(&mut self, config: &AppConfig) -> IDEResult<()>;

    /// Shutdown the component gracefully
    async fn shutdown(&mut self) -> IDEResult<()>;

    /// Get component status
    fn status(&self) -> ComponentStatus;

    /// Get component metadata
    fn metadata(&self) -> ComponentMetadata;

    /// Health check
    async fn health_check(&self) -> IDEResult<ComponentHealth>;
}

/// Component metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    pub name:           String,
    pub version:        String,
    pub description:    String,
    pub category:       ComponentCategory,
    pub authors:        Vec<String>,
    pub repository_url: Option<String>,
    pub license:        Option<String>,
    pub capabilities:   Vec<String>,
    pub dependencies:   Vec<ComponentDependency>,
    pub metadata:       HashMap<String, serde_json::Value>,
}

/// Component categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentCategory {
    AIProvider,
    Analyzer,
    CodeGenerator,
    RefactoringTool,
    Debugger,
    LSPClient,
    UIComponent,
    Service,
    Utility,
    Plugin,
}

/// Component dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDependency {
    pub name:               String,
    pub version_constraint: String,
    pub optional:           bool,
    pub description:        Option<String>,
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status:     HealthStatus,
    pub details:    String,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub metrics:    HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// AI provider trait for different model providers
#[async_trait]
pub trait AIProviderTrait: ComponentLifecycle {
    /// Provider identifier
    fn provider_id(&self) -> &str;

    /// Check if this provider supports the given model
    fn supports_model(&self, model_name: &str) -> bool;

    /// Load a model with the given configuration
    async fn load_model(&self, config: ModelConfig) -> IDEResult<ModelHandle>;

    /// Unload a model
    async fn unload_model(&self, handle: &ModelHandle) -> IDEResult<()>;

    /// Get model information
    async fn get_model_info(&self, handle: &ModelHandle) -> IDEResult<ModelState>;

    /// Perform inference with the model
    async fn inference(
        &self,
        handle: &ModelHandle,
        context: AIContext,
        options: InferenceOptions,
    ) -> IDEResult<InferenceResult>;
}

/// Model handle for referencing loaded models
#[derive(Debug, Clone)]
pub struct ModelHandle(Arc<ModelHandleInner>);

#[derive(Debug)]
struct ModelHandleInner {
    id:         String,
    provider:   AIProvider,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl ModelHandle {
    pub fn new(id: String, provider: AIProvider) -> Self {
        Self(Arc::new(ModelHandleInner {
            id,
            provider,
            created_at: chrono::Utc::now(),
        }))
    }

    pub fn id(&self) -> &str {
        &self.0.id
    }

    pub fn provider(&self) -> &AIProvider {
        &self.0.provider
    }

    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.0.created_at
    }
}

/// Inference options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceOptions {
    /// Temperature for generation (0.0-2.0)
    pub temperature:    Option<f32>,
    /// Top-P sampling parameter
    pub top_p:          Option<f32>,
    /// Top-K sampling parameter
    pub top_k:          Option<u32>,
    /// Maximum tokens to generate
    pub max_tokens:     Option<u32>,
    /// Stop sequences
    pub stop_sequences: Vec<String>,
    /// Additional provider-specific options
    pub extra_options:  HashMap<String, serde_json::Value>,
}

impl Default for InferenceOptions {
    fn default() -> Self {
        Self {
            temperature:    Some(0.7),
            top_p:          Some(0.9),
            top_k:          Some(40),
            max_tokens:     Some(2048),
            stop_sequences: Vec::new(),
            extra_options:  HashMap::new(),
        }
    }
}

/// Inference result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    /// Generated text
    pub text:          String,
    /// Finish reason
    pub finish_reason: FinishReason,
    /// Token usage information
    pub usage:         Option<TokenUsage>,
    /// Generation metadata
    pub metadata:      HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinishReason {
    Stop,
    Length,
    Cancelled,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Tokens in the prompt
    pub prompt_tokens:     u32,
    /// Tokens in the completion
    pub completion_tokens: u32,
    /// Total tokens
    pub total_tokens:      u32,
}

/// Analysis trait for extensible analysis capabilities
#[async_trait]
pub trait AnalysisTrait: ComponentLifecycle {
    /// Analyze a target
    async fn analyze(&self, target: AnalysisTarget) -> IDEResult<Vec<AnalysisResult>>;

    /// Configure the analyzer
    async fn configure(&mut self, config: AnalysisConfig) -> IDEResult<()>;

    /// Get supported analysis categories
    fn supported_categories(&self) -> Vec<String>;
}

/// Code generation trait
#[async_trait]
pub trait CodeGenerationTrait: ComponentLifecycle {
    /// Generate code based on specification
    async fn generate(&self, spec: CodeGenSpec, context: AIContext) -> IDEResult<CodeGenResult>;

    /// Validate generated code
    async fn validate(&self, code: String, language: &str) -> IDEResult<ValidationResult>;

    /// Complete partial code
    async fn complete(&self, partial: String, context: AIContext) -> IDEResult<Vec<String>>;
}

/// Code generation specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGenSpec {
    pub description:   String,
    pub requirements:  Vec<String>,
    pub constraints:   Vec<String>,
    pub language:      String,
    pub framework:     Option<String>,
    pub template_path: Option<String>,
    pub metadata:      HashMap<String, serde_json::Value>,
}

/// Code generation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGenResult {
    pub code:         String,
    pub explanation:  Option<String>,
    pub tests:        Vec<String>,
    pub dependencies: Vec<String>,
    pub metadata:     HashMap<String, serde_json::Value>,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid:  bool,
    pub issues: Vec<String>,
    pub score:  Option<f64>,
}

/// Refactoring trait for code transformation
#[async_trait]
pub trait RefactoringTrait: ComponentLifecycle {
    /// Analyze refactoring possibilities
    async fn analyze_opportunities(&self, target: AnalysisTarget) -> IDEResult<Vec<RefactoringOpportunity>>;

    /// Execute a refactoring
    async fn execute_refactoring(&self, opportunity: RefactoringOpportunity) -> IDEResult<RefactoringResult>;

    /// Preview refactoring changes
    async fn preview_refactoring(&self, opportunity: RefactoringOpportunity) -> IDEResult<RefactoringPreview>;
}

/// Refactoring opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringOpportunity {
    pub id:              String,
    pub description:     String,
    pub impact:          RefactoringImpact,
    pub confidence:      f32,
    pub affected_files:  Vec<String>,
    pub preview_changes: Vec<CodeChange>,
}

/// Refactoring impact levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RefactoringImpact {
    Low,
    Medium,
    High,
    Breaking,
}

/// Refactoring result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringResult {
    pub success:       bool,
    pub changes:       Vec<CodeChange>,
    pub errors:        Vec<String>,
    pub rollback_info: Option<String>,
}

/// Refactoring preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringPreview {
    pub changes:           Vec<CodeChange>,
    pub warnings:          Vec<String>,
    pub impact_assessment: String,
}

/// Code change representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub file_path:   String,
    pub old_range:   super::ai::Range,
    pub new_content: String,
    pub change_type: ChangeType,
    pub description: Option<String>,
}

/// Types of code changes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Add,
    Remove,
    Modify,
    Replace,
}

/// Plugin trait for extensibility
#[async_trait]
pub trait PluginTrait: ComponentLifecycle {
    /// Handle plugin events
    async fn handle_event(&self, event: PluginEvent) -> IDEResult<()>;

    /// Execute plugin commands
    async fn execute_command(&self, command: PluginCommand) -> IDEResult<serde_json::Value>;

    /// Get plugin configuration schema
    fn config_schema(&self) -> Option<serde_json::Value>;
}

/// Plugin event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEvent {
    pub id:         String,
    pub event_type: String,
    pub payload:    serde_json::Value,
    pub timestamp:  chrono::DateTime<chrono::Utc>,
    pub source:     String,
}

/// Plugin command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    pub command_id: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub timeout:    Option<std::time::Duration>,
}

/// Component registry trait for managing components
#[async_trait]
pub trait ComponentRegistry: Send + Sync {
    /// Register a component
    async fn register_component(&self, component: Box<dyn ComponentLifecycle>) -> IDEResult<String>;

    /// Unregister a component
    async fn unregister_component(&self, id: &str) -> IDEResult<()>;

    /// Get a component by ID
    async fn get_component(&self, id: &str) -> IDEResult<Box<dyn ComponentLifecycle>>;

    /// List all registered components
    async fn list_components(&self) -> IDEResult<Vec<ComponentInfo>>;
}

/// Component information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub id:       String,
    pub name:     String,
    pub version:  String,
    pub category: ComponentCategory,
    pub status:   ComponentStatus,
    pub enabled:  bool,
}

/// Service coordination trait for multi-component workflows
#[async_trait]
pub trait ServiceCoordinatorTrait: Send + Sync {
    /// Start a coordinated analysis workflow
    async fn start_analysis_workflow(
        &self,
        targets: Vec<AnalysisTarget>,
        config: AnalysisConfig,
    ) -> IDEResult<WorkflowHandle>;

    /// Monitor workflow progress
    async fn workflow_status(&self, handle: &WorkflowHandle) -> IDEResult<WorkflowStatus>;

    /// Cancel a workflow
    async fn cancel_workflow(&self, handle: &WorkflowHandle) -> IDEResult<()>;
}

/// Workflow handle
#[derive(Debug, Clone)]
pub struct WorkflowHandle {
    pub id:         String,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

/// Workflow status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStatus {
    pub status:       WorkflowState,
    pub progress:     f32,
    pub current_step: Option<String>,
    pub errors:       Vec<String>,
    pub results:      HashMap<String, AnalysisResult>,
    pub metadata:     HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowState {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

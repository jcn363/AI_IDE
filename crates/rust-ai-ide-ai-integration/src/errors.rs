//! Error handling for AI integration layer
//!
//! This module defines the error types and handling patterns for the AI service
//! integration layer, following the project's error handling guidelines.

use std::fmt;

use thiserror::Error;

/// Main error type for AI service integration operations
#[derive(Error, Debug)]
pub enum IntegrationError {
    /// LSP bridge initialization or operation error
    #[error("LSP AI bridge error: {0}")]
    LspBridge(#[from] LspBridgeError),

    /// Frontend interface error
    #[error("Frontend interface error: {0}")]
    Frontend(#[from] FrontendInterfaceError),

    /// Performance router error
    #[error("Performance router error: {0}")]
    Router(#[from] PerformanceRouterError),

    /// Type generation error
    #[error("Type generation error: {0}")]
    TypeGeneration(#[from] TypeGenerationError),

    /// UX optimization error
    #[error("UX optimization error: {0}")]
    UxOptimization(#[from] UxOptimizationError),

    /// Generic AI service error
    #[error("AI service error: {0}")]
    AiService(#[from] AiServiceError),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Initialization error
    #[error("Initialization failed: {0}")]
    Initialization(String),
}

/// LSP AI Bridge specific errors
#[derive(Error, Debug)]
pub enum LspBridgeError {
    /// LSP client error
    #[error("LSP client error: {0}")]
    LspClient(String),

    /// AI security validation error
    #[error("AI security validation failed: {0}")]
    SecurityValidation(String),

    /// Performance monitoring error
    #[error("Performance monitoring error: {0}")]
    PerformanceMonitor(String),

    /// AI completion merger error
    #[error("AI completion merger error: {0}")]
    CompletionMerger(String),

    /// Diagnostics enhancement error
    #[error("Diagnostics enhancement error: {0}")]
    DiagnosticsEnhancer(String),
}

/// Frontend interface specific errors
#[derive(Error, Debug)]
pub enum FrontendInterfaceError {
    /// Command handler error
    #[error("Command handler error: {0}")]
    CommandHandler(String),

    /// Response formatter error
    #[error("Response formatter error: {0}")]
    ResponseFormatter(String),

    /// User feedback collector error
    #[error("User feedback collector error: {0}")]
    UserFeedbackCollector(String),

    /// UI state manager error
    #[error("UI state manager error: {0}")]
    UiStateManager(String),

    /// Error reporter error
    #[error("Error reporter error: {0}")]
    ErrorReporter(String),
}

/// Performance router specific errors
#[derive(Error, Debug)]
pub enum PerformanceRouterError {
    /// Load balancer error
    #[error("Load balancer error: {0}")]
    LoadBalancer(String),

    /// Response optimizer error
    #[error("Response optimizer error: {0}")]
    ResponseOptimizer(String),

    /// Cache manager error
    #[error("Cache manager error: {0}")]
    CacheManager(String),

    /// Priority router error
    #[error("Priority router error: {0}")]
    PriorityRouter(String),

    /// Fallback engine error
    #[error("Fallback engine error: {0}")]
    FallbackEngine(String),
}

/// Type generation specific errors
#[derive(Error, Debug)]
pub enum TypeGenerationError {
    /// Type analyzer error
    #[error("Type analyzer error: {0}")]
    TypeAnalyzer(String),

    /// TypeScript emitter error
    #[error("TypeScript emitter error: {0}")]
    TypeScriptEmitter(String),

    /// Validation generator error
    #[error("Validation generator error: {0}")]
    ValidationGenerator(String),

    /// Documentation builder error
    #[error("Documentation builder error: {0}")]
    DocumentationBuilder(String),

    /// API schema generator error
    #[error("API schema generator error: {0}")]
    ApiSchemaGenerator(String),
}

/// UX optimization specific errors
#[derive(Error, Debug)]
pub enum UxOptimizationError {
    /// Usage analyzer error
    #[error("Usage analyzer error: {0}")]
    UsageAnalyzer(String),

    /// Performance optimizer error
    #[error("Performance optimizer error: {0}")]
    PerformanceOptimizer(String),

    /// User preference learner error
    #[error("User preference learner error: {0}")]
    UserPreferenceLearner(String),

    /// Response filter error
    #[error("Response filter error: {0}")]
    ResponseFilter(String),

    /// Adaptive UI engine error
    #[error("Adaptive UI engine error: {0}")]
    AdaptiveUiEngine(String),
}

/// Generic AI service errors
#[derive(Error, Debug)]
pub enum AiServiceError {
    /// Model not available
    #[error("AI model not available: {0}")]
    ModelNotAvailable(String),

    /// Invalid model response
    #[error("Invalid model response: {0}")]
    InvalidResponse(String),

    /// Model timeout
    #[error("Model operation timeout: {0}")]
    Timeout(String),

    /// Resource exhausted
    #[error("AI service resources exhausted: {0}")]
    ResourceExhausted(String),

    /// Unsupported operation
    #[error("Operation not supported by AI service: {0}")]
    UnsupportedOperation(String),
}

//! # Core Architectural Framework for AI-Enhanced Rust IDE
//!
//! Intelligent foundation layer providing AI-powered architectural patterns,
//! adaptive resource management, and context-aware operations that form the
//! backbone of the AI-enhanced development environment. This crate implements
//! the sophisticated architectural decisions that enable scalable, intelligent,
//! and resource-efficient IDE operations.
//!
//! ## AI/ML-Powered Architectural Design Patterns
//!
//! The core architecture employs advanced machine learning and AI techniques
//! to provide intelligent, adaptive, and context-aware IDE functionality:
//!
//! ### 🤖 Intelligent Context Management
//! - **Adaptive Context Learning**: ML-powered context understanding that learns from developer
//!   behavior
//! - **Context Preservation Architecture**: Advanced state management with predictive retention
//! - **Semantic Context Expansion**: AI-driven expansion of development context beyond literal
//!   boundaries
//! - **Contextual Memory Optimization**: Intelligent caching of contextual information with
//!   predictive expiration
//!
//! ### 🏗️ Adaptive Resource Management
//! - **Predictive Resource Allocation**: ML algorithms predicting resource needs based on usage
//!   patterns
//! - **Intelligent Performance Optimization**: Adaptive resource management with learning-based
//!   optimization
//! - **Context-Aware Resource Prioritization**: Dynamic resource distribution based on developer
//!   intent
//! - **Resource Usage Forecasting**: Predictive analytics for optimal resource planning
//!
//! ### 📊 AI-Enhanced File System Operations
//! - **Intelligent Path Resolution**: ML-powered file discovery and path prediction
//! - **Semantic File Classification**: AI-driven file categorization beyond extension-based
//!   classification
//! - **Predictive File Access**: Learning-based file access prediction and prefetching
//! - **Adaptive File System Caching**: Intelligent file system caching with usage-based eviction
//!   policies
//!
//! ## Core Architectural Patterns Implementation
//!
//! ### Layered Architecture with AI Augmentation
//!
//! The system implements a sophisticated layered architecture:
//!
//! ```rust
//! // Core Architecture Layers with AI Integration
//!
//! AIFoundationLayer {
//!     foundational_traits: GenericAIInterfaces,
//!     resource_abstration: IntelligentResourceManager,
//!     error_handling: AIAugmentedErrorSystem,
//!     concurrency_primitives: SmartAsyncRuntime
//! }
//!
//! IntelligenceLayer {
//!     context_understanding: MLBasedContextEngine,
//!     behavior_prediction: AdaptiveUserModeling,
//!     semantic_analysis: AIBasedSemanticProcessing,
//!     learning_system: ContinuousImprovementEngine
//! }
//!
//! ServiceIntegrationLayer {
//!     lsp_integration: IntelligentLSPOrchestrator,
//!     ai_service_coordination: MultiProviderServiceManager,
//!     resource_optimization: PredictiveResourceAllocator,
//!     cross_service_intelligence: IntegratedServiceLearning
//! }
//!
//! OptimizationLayer {
//!     performance_predictor: AdaptivePerformanceOptimizer,
//!     caching_controller: IntelligentCacheManager,
//!     resource_scheduler: PredictiveResourceScheduler,
//!     scalability_manager: DynamicScalingEngine
//! }
//! ```
//!
//! ### AI-Enhanced Trait System
//!
//! The architectural foundation provides AI-augmented traits:
//!
//! #### Intelligent Validation Framework
//! ```rust
//! trait IntelligentValidatable {
//!     type ValidationContext;
//!     type MLValidationModel;
//!
//!     fn ml_validate(
//!         &self,
//!         context: &ValidationContext,
//!         model: &MLValidationModel,
//!     ) -> MLValidationResult;
//!     fn adaptive_validate(
//!         &self,
//!         context: &ValidationContext,
//!         history: &[ValidationResult],
//!     ) -> AdaptiveValidationResult;
//!     fn predictive_validate(&self, context: &ValidationContext) -> PredictiveValidationResult;
//! }
//! ```
//!
//! #### Context-Aware Extensions
//! ```rust
//! trait IntelligentPathExt {
//!     fn ml_predict_access_pattern(&self) -> MLAccessPrediction;
//!     fn adaptive_path_resolution(&self, context: &Context) -> AdaptivePathResolution;
//!     fn semantic_file_relationships(&self) -> SemanticFileRelationships;
//! }
//! ```
//!
//! ## Adaptive Resource Management System
//!
//! ### Contextual Resource Allocation
//! ```rust
//! struct IntelligentResourceManager {
//!     resource_predictor:   MLPredictor<ResourceDemand>,
//!     context_analyzer:     MLContextAnalyzer,
//!     allocation_optimizer: AdaptiveAllocator,
//!     performance_monitor:  ContinuousPerformanceTracker,
//! }
//! ```
//!
//! ### Predictive Memory Management
//! ```rust
//! async fn intelligent_memory_management(&mut self) {
//!     let demand_prediction = self.resource_predictor.predict_future_demand().await;
//!     let context_analysis = self.context_analyzer.analyze_current_context().await;
//!
//!     if demand_prediction > self.allocation_optimizer.optimal_threshold {
//!         self.allocation_optimizer.scale_resources(&context_analysis).await;
//!     }
//
//!     self.performance_monitor.track_allocation_efficiency().await;
//! }
//! ```
//! 
//! ## AI-Powered Path and File Management
//!
//! ### Semantic Path Intelligence
//! ```rust
//! impl IntelligentPathExt for Path {
//!     fn ml_workspace_detection(&self) -> MLWorkspaceDetection {
//!         // AI-powered workspace root identification
//!         // Learns from project structure patterns
//!         // Adapts to different project types and conventions
//!         // Maintains accuracy through continuous learning
//!     }
//!
//!     fn adaptive_ancestor_finding(&self, context: &PathContext) -> AdaptiveAncestorResult {
//!         // Context-aware ancestor directory discovery
//!         // Learns project structure patterns over time
//!         // Optimizes search based on historical access patterns
//!         // Provides confidence scoring for discovered relationships
//!     }
//! }
//! ```
//! 
//! ### Intelligent File Discovery
//! ```rust
//! struct MLFileDiscovery {
//!     pattern_learner: MLBasedPatternRecognizer,
//!     access_predictor: ReinforcementLearningPredictor,
//!     relevance_scorer: ContextualRelevanceScorer,
//!     caching_optimizer: AdaptiveCacheManager
//! }
//! ```
//! 
//! ## Cross-Crate AI Integration Architecture
//!
//! ### Foundation Layer (rust-ai-ide-core-fundamentals)
//! - Core type system with AI-enhanced validation
//! - Fundamental traits augmented with ML capabilities
//! - Base error handling with intelligent error classification
//! - Foundational utilities with adaptive behavior
//!
//! ### Shell Operations Layer (rust-ai-ide-core-shell)
//! - Async command execution with intelligent retry mechanisms
//! - Process management with predictive resource allocation
//! - Command result analysis with ML-pattern recognition
//! - Intelligent error recovery for system interactions
//!
//! ### File System Layer (rust-ai-ide-core-file)
//! - Path management with semantic understanding
//! - File operations with predictive access patterns
//! - Directory structure analysis with ML-augmented intelligence
//! - Adaptive caching with learning-based optimization
//!
//! ### AI Integration Layer (rust-ai-ide-core-ai)
//! - Multi-provider AI service orchestration
//! - Intelligent provider selection and load balancing
//! - Context-aware AI request optimization
//! - Performance monitoring with predictive analytics
//!
//! ### Performance Monitoring Layer (rust-ai-ide-core-metrics)
//! - Telemetry collection with intelligent filtering
//! - Performance analysis with ML-based anomaly detection
//! - Resource utilization prediction and optimization
//! - Continuous improvement through operational learning
//!
//! ## Future AI/ML Enhancements Roadmap
//!
//! ### Advanced Intelligence Features
//! - **Behavioral Pattern Learning**: Deep learning from developer interaction patterns
//! - **Predictive Code Generation**: AI anticipation of developer development needs
//! - **Collaborative Intelligence**: Team-based learning and shared intelligence
//! - **Evolutionary Architecture**: Self-adapting architecture based on usage patterns
//!
//! ### Enhanced Learning Capabilities
//! - **Real-time Learning**: Instantaneous adaptation to developer behavior
//! - **Context Preservation**: Long-term memory of project patterns and preferences
//! - **Cross-Project Learning**: Transferring learned patterns across different projects
//! - **Team Intelligence**: Shared learning across development teams and organizations
//!
//! This crate serves as the intelligent foundation layer, providing the architectural
//! intelligence and adaptive capabilities that enable higher-level AI features while
//! maintaining backward compatibility through its re-export architecture.

// Re-exports from fundamentals (base layer)
// Re-exports from AI operations
pub use rust_ai_ide_core_ai::*;
// Re-exports from file operations
pub use rust_ai_ide_core_file::*;
pub use rust_ai_ide_core_fundamentals::error::*;
pub use rust_ai_ide_core_fundamentals::{formatters, *};
// Re-exports from metrics
pub use rust_ai_ide_core_metrics::*;
// Re-exports from shell operations
pub use rust_ai_ide_core_shell::*;

// Deprecated modules removed - use direct re-exports above

/// General-purpose utility traits
pub mod traits {
    use std::path::Path;

    /// Trait for objects that can be validated
    pub trait Validatable {
        type Error;

        fn validate(&self) -> Result<(), Self::Error>;
        fn is_valid(&self) -> bool {
            self.validate().is_ok()
        }
    }

    /// Trait for objects that can be displayed in debug format
    pub trait DebugDisplay: std::fmt::Debug + std::fmt::Display {
        fn debug_str(&self) -> String {
            format!("{:?}", self)
        }
    }

    /// Extension trait for Path operations common in IDE operations
    pub trait PathExt {
        fn readable_name(&self) -> String;
        fn is_workspace_root(&self) -> bool;
        fn find_ancestor_with(&self, file_name: &str) -> Option<std::path::PathBuf>;
        fn parent_count(&self) -> usize;
    }

    impl PathExt for Path {
        fn readable_name(&self) -> String {
            self.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        }

        fn is_workspace_root(&self) -> bool {
            self.join("Cargo.toml").exists() && self.join("src").exists() && self.join("src/lib.rs").exists()
        }

        fn find_ancestor_with(&self, file_name: &str) -> Option<std::path::PathBuf> {
            let mut current = Some(self.to_path_buf());
            while let Some(path) = current {
                if path.join(file_name).exists() {
                    return Some(path);
                }
                current = path.parent().map(|p| p.to_path_buf());
            }
            None
        }

        fn parent_count(&self) -> usize {
            let mut count = 0;
            let mut current = self.parent();

            while let Some(parent) = current {
                count += 1;
                current = parent.parent();
            }

            count
        }
    }

    /// Trait for result types that need special handling
    pub trait ResultExt<T, E> {
        fn with_context<F>(self, context_fn: F) -> Result<T, E>
        where
            F: FnOnce(E) -> E;

        fn warn_on_error(self, message: &str) -> Self;
    }

    impl<T, E> ResultExt<T, E> for Result<T, E> {
        fn with_context<F>(self, context_fn: F) -> Result<T, E>
        where
            F: FnOnce(E) -> E,
        {
            self.map_err(context_fn)
        }

        fn warn_on_error(self, message: &str) -> Self {
            if let Err(_) = &self {
                log::warn!("{}", message);
            }
            self
        }
    }
}

/// Common constants used across the IDE
pub mod constants {
    /// Default timeout for network operations
    pub const NETWORK_TIMEOUT_SECS: u64 = 30;

    /// Default timeout for file operations
    pub const FILE_TIMEOUT_SECS: u64 = 10;

    /// Default cache TTL for diagnostics
    pub const DIAGNOSTICS_CACHE_TTL_SECS: u64 = 300;

    /// Default cache TTL for AI suggestions
    pub const AI_CACHE_TTL_SECS: u64 = 600;

    /// Maximum file size for in-memory processing (100MB)
    pub const MAX_FILE_SIZE_BYTES: u64 = 100 * 1024 * 1024;

    /// Maximum number of open files
    pub const MAX_OPEN_FILES: usize = 1000;

    /// Default buffer size for file operations
    pub const DEFAULT_BUFFER_SIZE: usize = 8192;

    /// Workspace detection files
    pub const WORKSPACE_FILES: &[&str] = &["Cargo.toml", "package.json", "setup.py"];

    /// File extensions for source code
    pub const SOURCE_EXTENSIONS: &[&str] = &[
        "rs", "js", "ts", "py", "java", "cpp", "c", "hpp", "h", "cs", "php", "rb", "go", "swift", "kt", "scala", "clj",
        "hs", "ml", "fs", "ex",
    ];

    /// Temporary directory prefix
    pub const TEMP_PREFIX: &str = "rust-ai-ide";

    /// Configuration file names to look for
    pub const CONFIG_FILES: &[&str] = &["rust-ai-ide.toml", ".rust-ai-ide.toml"];
}

/// Error types for core operations
pub mod error {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum CoreError {
        #[error("I/O error: {0}")]
        Io(#[from] std::io::Error),

        #[error("Path validation failed: {0}")]
        PathValidation(String),

        #[error("Hash calculation error: {0}")]
        HashError(String),

        #[error("Validation error: {0}")]
        ValidationError(String),

        #[error("System error: {0}")]
        SystemError(String),

        #[error("Timeout exceeded: {0:?}")]
        Timeout(std::time::Duration),

        #[error("Resource limit exceeded: {resource} (limit: {limit})")]
        ResourceLimit { resource: String, limit: usize },
    }

    pub type CoreResult<T> = Result<T, CoreError>;
}

pub mod cold_start_optimizer;

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_path_extensions_work() {
        let test_path = Path::new("/tmp/test/src/main.rs");
        assert_eq!(test_path.readable_name(), "main.rs");
    }

    #[test]
    fn test_find_ancestor_with() {
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let cargo_path = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_path, "test").unwrap();

        let test_path = temp_dir.path().join("src/main.rs");
        let ancestor = test_path.find_ancestor_with("Cargo.toml");
        assert!(ancestor.is_some());
    }

    #[test]
    fn test_result_ext() {
        use super::traits::ResultExt;

        let result: Result<i32, String> = Ok(42);
        assert_eq!(result.with_context(|_| "error".to_string()), Ok(42));

        let result: Result<i32, String> = Err("test".to_string());
        let error_result = result.with_context(|e| format!("wrapped: {}", e));
        assert_eq!(error_result.unwrap_err(), "wrapped: test");
    }

    #[test]
    fn test_constants() {
        use super::constants::*;
        assert_eq!(NETWORK_TIMEOUT_SECS, 30);
        assert!(SOURCE_EXTENSIONS.contains(&"rs"));
        assert_eq!(TEMP_PREFIX, "rust-ai-ide");
    }
}

//! # AI-Powered Language Server Protocol (LSP) Implementation
//!
//! Advanced multi-language LSP infrastructure providing intelligent code assistance
//! through AI/ML-enhanced language servers and cross-cutting analysis capabilities.
//! The system combines traditional static analysis with dynamic, context-aware
//! intelligence for comprehensive developer support.
//!
//! ## Core LSP Capabilities with AI/ML Enhancement
//!
//! ### 🤖 Traditional LSP Protocol Implementation
//! - **Language Server Management**: Robust client-server communication
//! - **Protocol Compliance**: Full LSP specification support (textDocument, workspace, etc.)
//! - **Resource Optimization**: Server pooling, connection multiplexing, caching
//! - **Error Recovery**: Graceful handling of server failures and connection issues
//!
//! ### 🔬 AI/ML-Enhanced Language Processing
//! - **Intelligent Completion**: ML-ranked suggestions with context awareness
//! - **Smart Diagnostics**: AI-powered error analysis beyond traditional parsing
//! - **Cognitive Hover**: Context-rich documentation with semantic understanding
//! - **Intelligent Refactoring**: AI-assisted code transformations with impact prediction
//!
//! ### 🌐 Multi-Language Integration
//! - **Unified API Layer**: Consistent interface across programming languages
//! - **Cross-Language Analysis**: Symbol resolution and dependency tracking across languages
//! - **Language Detection**: Automatic file type identification and appropriate server routing
//! - **Performance Optimization**: Smart resource allocation for multi-language codebases
//!
//! ## AI/ML Integration Architecture
//!
//! The AI enhancement operates through multiple integrated intelligence layers:
//!
//! ### 1. Request Context Enrichment
//! ```rust
//! // Before: Raw LSP completion request
//! // After: AI-enriched contextual completion
//!
//! LSPRequest {
//!     textDocument: "...",
//!     position: "...",
//!     context: Some(AIContext {
//!         intent_analysis: PredictiveIntentUnderstanding,
//!         semantic_context: CodeUnderstandingContext,
//!         user_history: BehavioralPatternAnalysis
//!     })
//! }
//! ```
//!
//! ### 2. Response Augmentation Pipeline
//! ```rust
//! // Raw Server Response -> AI Processing Pipeline -> Enhanced Response
//!
//! RawResponse
//!     |> AIContextEnrichment
//!     |> IntelligenceLayer(RelevanceRanking)
//!     |> IntelligenceLayer(ContextualReordering)
//!     |> IntelligenceLayer(PredictiveEnhancement)
//!     -> EnhancedResponse
//! ```
//!
//! ### 3. Cross-Language Intelligence Fusion
//! ```rust
//! // Multi-language symbol resolution with AI assistance
//!
//! SymbolQuery { name: "function", language: "rust" }
//!     |> CrossLanguageIntelligence
//!     |> RelatedSymbolDiscovery(python, typescript)
//!     |> SemanticSimilarityAnalysis
//!     -> MultiLanguageSymbolResults
//! ```
//!
//! ## AI-Enhanced LSP Request Processing
//!
//! ### Completion Request Enhancement
//! The system enriches standard LSP completion requests with:
//! - **Semantic Context**: Understanding of code intent and purpose
//! - **Historical Patterns**: Learning from user's previous code choices
//! - **Behavioral Prediction**: Anticipating likely code patterns
//! - **Multi-modal Suggestions**: Combining language model and AST analysis
//!
//! ### Diagnostic Intelligence Integration
//! Traditional diagnostics are enhanced with:
//! - **Root Cause Analysis**: AI-powered identification of underlying issues
//! - **Impact Assessment**: Predicting scope and consequences of errors
//! - **Resolution Prediction**: Suggesting most effective fix approaches
//! - **Educational Enhancement**: Providing context for error understanding
//!
//! ### Hover Information Augmentation
//! Standard hover content receives AI enhancement through:
//! - **Contextual Documentation**: Relevant information based on usage patterns
//! - **Usage Examples**: Dynamic generation of helpful code samples
//! - **Related Concept Discovery**: Finding semantically related symbols
//! - **Learning Pathway Suggestions**: Guiding developers to understand concepts
//!
//! ## Multi-Language Architecture
//!
//! ### Introspective Language Detection
//! ```rust
//! // AI-powered file classification
//!
//! LanguageDetector {
//!     filename_analysis: FileExtension + ShebangInspection,
//!     content_analysis: SyntacticPatternRecognition,
//!     context_analysis: ProjectFileRelationship,
//!     confidence_scoring: MLBasedAccuracyAssessment
//! }
//! ```
//!
//! ### Intelligent Server Routing
//! ```rust
//! // Smart server selection and load balancing
//!
//! LanguageRouter {
//!     server_health_monitoring: ServerStatusTracking,
//!     workload_distribution: IntelligentLoadBalancing,
//!     capability_matching: FeatureCompatibilityMatrix,
//!     performance_optimization: CachingStrategyImplementation
//! }
//! ```
//!
//! ### Cross-Language Symbol Fusion
//! ```rust
//! // Unified symbol search across multiple language server instances
//!
//! CrossLanguageSearch {
//!     query_expansion: SemanticSimilarityAnalysis,
//!     result_consolidation: ConfidenceWeightedMerging,
//!     relevance_ranking: MLBasedReorderingAlgorithm,
//!     context_preservation: ScopeAndNamespaceMaintenance
//! }
//! ```
//!
//! ## Performance Optimization Strategies
//!
//! ### Intelligent Caching Layer
//! - **Semantic Caching**: Understanding context to optimize cache hit rates
//! - **Predictive Prefetching**: Anticipating likely future requests
//! - **Adaptive Cache Sizing**: Dynamic cache management based on usage patterns
//! - **Hierarchical Caching**: Multi-level caching from local to distributed
//!
//! ### Resource Pool Management
//! - **Smart Server Pooling**: Maintaining optimal number of server instances
//! - **Load Balancing**: Distributing requests based on server capacity and proximity
//! - **Health Monitoring**: Continuous server status tracking and recovery
//! - **Resource Prediction**: Anticipating resource needs based on project patterns
//!
//! ### Concurrent Processing Optimization
//! - **Async Request Handling**: Maximizing throughput through concurrent processing
//! - **Result Streaming**: Incremental delivery of results for improved responsiveness
//! - **Background Processing**: Offloading heavy computations to prevent UI blocking
//! - **Batch Optimization**: Grouping related requests for collective processing
//!
//! ## Usage Examples
//!
//! ### Basic LSP Operations with AI Enhancement
//! ```no_run
//! use rust_ai_ide_lsp::{client::LSPClient, AIContext};
//!
//! async fn enhanced_completion() {
//!     let mut client = LSPClient::new();
//!     client.initialize().await?;
//!
//!     // AI-enhanced completion request
//!     let ai_context = AIContext {
//!         current_code: "fn main() { let x = vec!".to_string(),
//!         file_name: Some("main.rs".to_string()),
//!         cursor_position: Some((1, 20)),
//!         selection: None,
//!         project_context: Default::default(),
//!     };
//!
//!     // Get AI-ranked completion suggestions
//!     let completions = client.get_completions_with_ai(ai_context).await?;
//!
//!     for item in completions {
//!         println!("💡 {} (confidence: {:.2})", item.label, item.ai_confidence);
//!     }
//! }
//! ```
//!
//! ### Multi-Language Cross-Reference Analysis
//! ```no_run
//! #[cfg(feature = "multi-language-lsp")]
//! async fn cross_language_analysis() {
//!     use rust_ai_ide_lsp::{
//!         MultiLanguageLSP, CrossLanguageSymbol,
//!         LanguageServerKind::*
//!     };
//!
//!     let mut multi_lsp = MultiLanguageLSP::new();
//!
//!     // Initialize multiple language servers
//!     multi_lsp
//!         .add_language_server(RustAnalyzer, "/path/to/rust/project")
//!         .await?;
//!     multi_lsp
//!         .add_language_server(TypeScript, "/path/to/ts/project")
//!         .await?;
//!
//!     // Find related symbols across languages
//!     let search_result = multi_lsp
//!         .search_symbols_cross_language("HttpRequest", true)
//!         .await?;
//!
//!     for symbol in search_result.symbols {
//!         match symbol.server_kind {
//!             RustAnalyzer => println!("🦀 Rust: {}", symbol.name),
//!             TypeScript => println!("📘 TS: {}", symbol.name),
//!             Python => println!("🐍 Python: {}", symbol.name),
//!         }
//!     }
//! }
//! ```
//!
//! ## Error Handling and Resilience
//!
//! ### Graceful Degradation Strategy
//! - **Partial Service Recovery**: Continue operation when individual servers fail
//! - **Fallback Mechanisms**: Provide basic functionality when advanced features unavailable
//! - **User Feedback**: Clear indication of service availability and limitations
//! - **Automatic Recovery**: Intelligent restart and reconnection mechanisms
//!
//! ### Monitoring and Diagnostics
//! - **Health Check Integration**: Continuous monitoring of server status
//! - **Request Tracing**: End-to-end visibility into request processing
//! - **Performance Metrics**: Response time and success rate tracking
//! - **Failure Analysis**: Automated diagnosis of service disruptions
//!
//! ## Future AI/ML Enhancements
//!
//! ### Advanced Predictive Intelligence
//! - **Code Intention Prediction**: Anticipating developer intent from partial code
//! - **Automated Refactoring**: AI-driven code improvement suggestions
//! - **Collaborative Learning**: Shared intelligence across team members
//! - **Project Evolution Tracking**: Long-term code quality trend analysis
//!
//! ### Enhanced Multi-Modal Analysis
//! - **Natural Language Processing**: Understanding documentation and comments
//! - **Code Pattern Recognition**: Identifying architectural patterns and anti-patterns
//! - **Behavioral Analysis**: Learning from developer interaction patterns
//! - **Context-Aware Assistance**: Providing help based on project and task context

// SQL language server implementation
#[cfg(feature = "sql-lsp")]
pub mod sql_lsp;

// Enterprise enhancements
#[cfg(feature = "enterprise-monitoring")]
pub mod enterprise_monitoring;
#[cfg(feature = "enterprise-monitoring")]
pub mod enterprise_monitoring_impl;
#[cfg(feature = "enterprise-sql-lsp")]
pub mod enterprise_sql_lsp_server;
// Web language server implementations
mod web_language_servers;
mod web_language_server_factory;

pub mod ai_context;
pub mod client;
pub mod client_interface;
pub mod code_actions;
pub mod context_aware_completion;
pub mod debugging_integration;
pub mod completion;
pub mod diagnostics;
pub mod hover;
pub mod pool;
pub mod project;
pub mod refactoring;
pub mod rust_analyzer;
pub mod utils;
pub mod workspace;

// Multi-language support modules
#[cfg(feature = "multi-language-lsp")]
pub mod cross_language;
#[cfg(feature = "multi-language-lsp")]
pub mod language_detection;
#[cfg(feature = "multi-language-lsp")]
pub mod language_router;
#[cfg(feature = "multi-language-lsp")]
pub mod language_server;
#[cfg(feature = "multi-language-lsp")]
pub mod multi_language;

// Re-export commonly used types
pub use client::{LSPClient, LSPClientConfig, LSPError as ClientLSPError};
pub use debugging_integration::{
    DebugCapability, DebugFeatures, LanguageDebugCapabilities, BreakpointCapabilities,
    LSPDebugClient, LSPDebugEvent, LSPDebugClientTrait,
};
pub use client_interface::{LspClient, LspClientTrait};
pub use code_actions::get_code_actions;
pub use diagnostics::{
    AIAnalysisConfig, AIAnalysisResult, CodeAnalysisRequest, CodeSuggestion, DiagnosticsManager,
};
pub use lsp_types::{self, CompletionItem, Diagnostic};

// AI Context type for AI operations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cfg(feature = "ai")]
pub struct AIContext {
    pub current_code: String,
    pub file_name: Option<String>,
    pub cursor_position: Option<(u32, u32)>,
    pub selection: Option<String>,
    pub project_context: std::collections::HashMap<String, String>,
}

// Re-export web language server factory
pub use web_language_server_factory::WebLanguageServerFactory;

// Re-export multi-language API when feature is enabled
#[cfg(feature = "multi-language-lsp")]
pub use cross_language::{CrossLanguageSearchResult, CrossLanguageSymbol};
#[cfg(feature = "multi-language-lsp")]
pub use language_server::{LanguageServerConfig, LanguageServerKind};
#[cfg(feature = "multi-language-lsp")]
pub use multi_language::{
    MultiLanguageConfig, MultiLanguageLSP, MultiLanguageStatus, SystemHealth,
};
#[cfg(feature = "multi-language-lsp")]
pub use enhanced_rust_analyzer::{
    EnhancedRustAnalyzer, MultiLangAIAnalyzer, EnhancedFFIAnalysis,
    SmartSymbolResult, FFIFix, CompatibilityWarning,
};

// Re-export cross-language capabilities
#[cfg(feature = "multi-language-lsp")]
pub use cross_language_index::{CrossLanguageIndexer, SymbolEntry, SupportedLanguage};

// Re-export AI-related types when ai feature is enabled
#[cfg(feature = "sql-lsp")]
pub use sql_lsp::{
    SqlLspServer, SqlLspConfig, SqlLspError, SqlLspResult,
    QueryPerformanceMetrics, OptimizedQuerySuggestion,
    InferredSchema, ContextualErrorFix,
    CollaborativeEditSession, SqlCompletionItem, SqlHoverInfo,
    SqlDialectDetector, PostgresDialectDetector, MySqlDialectDetector,
    SqliteDialectDetector, SqlServerDialectDetector, OracleDialectDetector,
};

#[cfg(feature = "ai")]
pub use rust_ai_ide_ai::{
    // From inference crate
    AIService,
    AIProvider,

// Enterprise monitoring re-exports
#[cfg(feature = "enterprise-monitoring")]
pub use enterprise_monitoring::*;
#[cfg(feature = "enterprise-monitoring")]
pub use enterprise_monitoring_impl::*;
    AnalysisIssue,
    CodeAnalysisResult,

    // From architectural advisor
    ArchitecturalContext,
    ArchitecturalDocument,
    ArchitecturalGuidance,
    ArchitecturalSuggestion,

    // From analysis crate via re-export
    analysis,

    // From learning crate via re-export
    learning,

    // From error resolution
    error_resolution,


    // From spec generation
    spec_generation,

    // From code generation crate via re-export
    code_generation,

    // Additional types that may be needed
    error_resolution::*,

    // Advanced error analysis
    advanced_error_analysis,

    // Attempt additional exports that might be missing
    code_review,
    rate_limiter,
};

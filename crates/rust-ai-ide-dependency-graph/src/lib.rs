//! Advanced dependency graph library with petgraph integration
//!
//! This library provides comprehensive dependency graph analysis and resolution
//! capabilities with features for conflict detection, caching, and parallel processing.

pub mod async_operations;
pub mod builders;
pub mod cache;
pub mod conflict_analyzer;
pub mod error;
pub mod graph;
pub mod resolution;
pub mod resolver;
pub mod serialization;
pub mod workspace;

// Re-exports for convenient access
pub use async_operations::{
    AsyncGraphProcessor, AsyncOperation, AsyncOperationConfig, AsyncProcessorActor, AsyncResult, BatchOperationQueue,
    OperationResult,
};
pub use builders::{
    DependencyGraphConfig, DependencyGraphService, DependencyGraphServiceBuilder, GraphValidationResult,
    ValidationError, ValidationSummary, ValidationWarning, WorkspaceResolverBuilder,
};
pub use cache::{
    CacheConfig, DependencyResolutionEntry, DependencyResolutionKey, GraphCache,
    SharedDependencyGraph as CachedDependencyGraph,
};
pub use conflict_analyzer::{
    ComprehensiveConflictAnalyzer, ConflictAnalyzer, ConflictStats, ConstraintInfo, ImpactAnalysis, ResolutionPlan,
    RiskLevel, VersionConflict as AnalyzerVersionConflict,
};
pub use error::{DependencyError, DependencyResult, ErrorAggregator, ErrorSuggestion};
pub use graph::{
    DependencyEdge, DependencyGraph, DependencyGraphStats, DependencyNode, DependencyType, SharedDependencyGraph,
};
pub use resolution::{
    AdvancedDependencyResolver, ConflictSeverity, ConstraintAnalysis, ConstraintSource, CrossCrateAnalysis,
    EnhancedVersionConflict, PackageDependencyConstraint, PackageMetadata, ResolutionConfig, ResolutionContext,
    ResolutionStats, ResolutionWorker, SecurityAdvisory, SecuritySeverity,
};
pub use resolver::{DependencyResolver, PackageConstraint, ResolutionStrategy, VersionConflict};
pub use serialization::{GraphExporter, GraphImporter, GraphSerializer, SerializationFormat};
pub use workspace::{
    GraphIntegrityReport, ManifestType, PublicationStatus, WorkspaceAnalysis, WorkspaceAwareManager, WorkspaceConfig,
    WorkspaceMember, WorkspaceResolutionResult, WorkspaceResolver, WorkspaceStats,
};

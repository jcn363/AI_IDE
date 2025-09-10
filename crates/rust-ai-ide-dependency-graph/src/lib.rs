//! Advanced dependency graph library with petgraph integration
//!
//! This library provides comprehensive dependency graph analysis and resolution
//! capabilities with features for conflict detection, caching, and parallel processing.

pub mod error;
pub mod graph;
pub mod resolver;
pub mod cache;
pub mod conflict_analyzer;
pub mod builders;
pub mod serialization;
pub mod async_operations;
pub mod workspace;
pub mod resolution;

// Re-exports for convenient access
pub use error::{DependencyError, DependencyResult, ErrorAggregator, ErrorSuggestion};
pub use graph::{
    DependencyGraph, DependencyNode, DependencyEdge, DependencyType,
    SharedDependencyGraph, DependencyGraphStats,
};
pub use resolver::{
    DependencyResolver, ResolutionStrategy, VersionConflict,
    PackageConstraint,
};
pub use cache::{
    GraphCache, CacheConfig, DependencyResolutionKey, DependencyResolutionEntry,
    SharedDependencyGraph as CachedDependencyGraph,
};
pub use conflict_analyzer::{
    ConflictAnalyzer, VersionConflict as AnalyzerVersionConflict,
    ConstraintInfo, ConflictStats, ComprehensiveConflictAnalyzer,
    ResolutionPlan, ImpactAnalysis, RiskLevel,
};
pub use builders::{
    DependencyGraphConfig, DependencyGraphServiceBuilder,
    DependencyGraphService, GraphValidationResult, ValidationWarning,
    ValidationError, ValidationSummary, WorkspaceResolverBuilder,
};
pub use serialization::{
    SerializationFormat, GraphSerializer, GraphExporter, GraphImporter,
};
pub use async_operations::{
    AsyncGraphProcessor, AsyncProcessorActor, BatchOperationQueue,
    AsyncOperation, OperationResult, AsyncResult, AsyncOperationConfig,
};
pub use workspace::{
    WorkspaceResolver, WorkspaceConfig, WorkspaceMember, ManifestType,
    PublicationStatus, WorkspaceResolutionResult, WorkspaceStats,
    WorkspaceAnalysis, GraphIntegrityReport, WorkspaceAwareManager,
};
pub use resolution::{
    AdvancedDependencyResolver, ResolutionContext, ResolutionConfig,
    PackageDependencyConstraint, ConstraintSource, PackageMetadata,
    SecurityAdvisory, SecuritySeverity, CrossCrateAnalysis,
    ResolutionStats, EnhancedVersionConflict, ConflictSeverity,
    ConstraintAnalysis, ResolutionWorker,
};
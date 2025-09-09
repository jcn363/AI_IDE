// Core modules
pub mod advanced_error_analysis;
pub mod architectural_advisor;
pub mod code_review;
pub mod error_resolution;
pub mod rate_limiter;

// Re-export refactoring functionality from separate crate
pub mod refactoring {
    pub use rust_ai_ide_ai_refactoring::*;
}
pub mod spec_generation;

// Re-export analysis functionality from separate crate
pub mod analysis {
    pub use rust_ai_ide_ai_analysis::*;
}

// The architectural_advisor module is defined in this crate
pub use architectural_advisor::*;

// Re-export inference modules from separate crate
pub mod inference {
    pub use rust_ai_ide_ai_inference::inference::*;
    pub use rust_ai_ide_ai_inference::loaders;
    pub use rust_ai_ide_ai_inference::model_loader;
}

// Re-export learning functionality from separate crate
pub mod learning {
    pub use rust_ai_ide_ai_learning::*;
}
pub mod model_loader {
    pub use rust_ai_ide_ai_inference::loaders;
    pub use rust_ai_ide_ai_inference::model_loader::*;
}

// Explicit import of all types items to prevent namespace conflicts with glob
pub use architectural_advisor::types::{
    AntiPattern,
    ArchitecturalContext,
    ArchitecturalDocument,
    ArchitecturalGuidance,
    ArchitecturalOverview as AdvisorArchitecturalOverview, // avoid naming conflict
    ArchitecturalOverview,
    ArchitecturalRecommendation,
    ArchitecturalSuggestion,
    CohesionAnalysis,
    ComplexityAssessment,
    ComponentDocument,
    CouplingAnalysis,
    DecisionAnalysis,
    DecisionOption,
    DecisionRecommendation,
    DecisionRecord,
    DecisionStatus,
    DeploymentDocument,
    DeploymentEnvironment,
    DeploymentProcedures,
    DeploymentRequirements,
    DetectedPattern,
    InterfaceDocument,
    InterfaceMethod,
    MethodParameter,
    PatternAnalysis,
    PatternDocument,
    ProjectType,
    QualityAttribute,
    QualityAttributesDocument,
    QualityMetric,
    QualityMetrics,
    QualityScenario,
    RiskAssessment,
};

// Re-export inference types from the inference crate
pub use rust_ai_ide_ai_inference::{
    AIAnalysisConfig, AIProvider, AIService, AnalysisIssue, CodeAnalysisResult, ModelDevice,
    ModelHandle, ModelInfo, ModelLoadConfig, ModelSize, Quantization,
};

// Re-export code generation functionality from the codegen crate
pub mod code_generation {
    pub use rust_ai_ide_ai_codegen::*;
}

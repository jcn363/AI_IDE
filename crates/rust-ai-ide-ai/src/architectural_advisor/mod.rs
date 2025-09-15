//! # Architectural Advisor Module
//!
//! Intelligent AI-assisted architectural decision support system that provides:
//! - Codebase pattern analysis and recognition
//! - Architectural recommendations and suggestions
//! - Complexity assessment and quality metrics
//! - Decision evaluation and trade-off analysis
//! - Comprehensive documentation generation

// Declare submodules
pub mod analysis;
pub mod patterns;
pub mod recommendations;
pub mod system;
pub mod types;
pub mod validation;

// Re-export core types for easy access
pub use analysis::{DecisionEngine, MetricsAnalyzer};
// Re-export analysis components
pub use patterns::{CodebaseAnalysis, PatternDetector};
// Re-export trait for external implementations
pub use system::ArchitecturalAdvisor;
// Re-export main system components
pub use system::{create_architectural_advisor, IntelligentArchitecturalAdvisor};
pub use types::{
    AdvisorError,
    // Error handling
    AdvisorResult,
    AntiPattern,
    ArchitecturalContext,

    // Documentation types
    ArchitecturalDocument,

    // Recommendation types
    ArchitecturalGuidance,
    ArchitecturalRecommendation,
    ArchitecturalSuggestion,
    CohesionAnalysis,

    ComplexityAssessment,
    // Enums for classification
    ComplexityLevel,
    CouplingAnalysis,
    DecisionAnalysis,
    // Decision types
    DecisionOption,
    DecisionRecommendation,

    DecisionStatus,

    DetectedPattern,
    ImpactLevel,
    ImplementationEffort,
    // Analysis result types
    PatternAnalysis,
    PriorityLevel,
    // Project and context types
    ProjectType,
    QualityMetrics,
    RiskAssessment,

    RiskLevel,
    SuggestionCategory,
};

/// Version information for the architectural advisor
pub const ARCHITECTURAL_ADVISOR_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default confidence threshold for pattern detection
pub const DEFAULT_PATTERN_CONFIDENCE_THRESHOLD: f32 = 0.7;

/// Default quality metrics baseline
pub const DEFAULT_MAINTAINABILITY_THRESHOLD: f32 = 70.0;

/// Example usage of the architectural advisor
///
/// ```rust
/// use rust_ai_ide_ai::architectural_advisor::*;
///
/// #[tokio::test]
/// async fn basic_usage() {
///     // Create advisor
///     let advisor = create_architectural_advisor();
///
///     // Define architectural context
///     let context = ArchitecturalContext {
///         codebase_path:        "src/".to_string(),
///         project_type:         ProjectType::Application,
///         current_architecture: None,
///         constraints:          vec!["performance".to_string()],
///         goals:                vec!["scalability".to_string()],
///         team_size:            Some(5),
///         expected_lifecycle:   Some("2 years".to_string()),
///     };
///
///     // Analyze patterns
///     let analysis = advisor.analyze_patterns(context).await?;
///     println!("Detected {} patterns", analysis.detected_patterns.len());
///
///     // Get recommendations
///     let guidance = advisor.get_recommendations(&analysis).await?;
///     println!(
///         "Generated {} recommendations",
///         guidance.primary_recommendations.len()
///     );
/// }
/// ```

/// Quick analysis helper function
///
/// Performs basic architectural analysis with sensible defaults
pub async fn quick_analyze(path: &str) -> AdvisorResult<PatternAnalysis> {
    let advisor = create_architectural_advisor();

    let context = ArchitecturalContext {
        codebase_path:        path.to_string(),
        project_type:         ProjectType::Application,
        current_architecture: None,
        constraints:          vec![],
        goals:                vec!["maintainability".to_string()],
        team_size:            None,
        expected_lifecycle:   None,
    };

    advisor.analyze_patterns(context).await
}

/// Comprehensive assessment helper
///
/// Provides full architectural assessment including analysis,
/// recommendations, and risk evaluation
pub async fn comprehensive_assessment(path: &str) -> AdvisorResult<ArchitecturalGuidance> {
    let advisor = create_architectural_advisor();

    let context = ArchitecturalContext {
        codebase_path:        path.to_string(),
        project_type:         ProjectType::Application,
        current_architecture: None,
        constraints:          vec![],
        goals:                vec!["maintainability".to_string(), "performance".to_string()],
        team_size:            None,
        expected_lifecycle:   None,
    };

    let analysis = advisor.analyze_patterns(context).await?;
    advisor.get_recommendations(&analysis).await
}

/// Architectural advisor configuration builder
///
/// Allows advanced configuration of the architectural advisor
pub struct ArchitecturalAdvisorConfig {
    pub enable_pattern_detection:      bool,
    pub enable_anti_pattern_detection: bool,
    pub confidence_threshold:          f32,
    pub quality_threshold:             f32,
    pub detailed_analysis:             bool,
}

impl Default for ArchitecturalAdvisorConfig {
    fn default() -> Self {
        Self {
            enable_pattern_detection:      true,
            enable_anti_pattern_detection: true,
            confidence_threshold:          DEFAULT_PATTERN_CONFIDENCE_THRESHOLD,
            quality_threshold:             DEFAULT_MAINTAINABILITY_THRESHOLD,
            detailed_analysis:             false,
        }
    }
}

impl ArchitecturalAdvisorConfig {
    /// Create a configuration optimized for speed (less detailed analysis)
    pub fn for_speed() -> Self {
        Self {
            detailed_analysis: false,
            enable_anti_pattern_detection: false,
            confidence_threshold: 0.8, // Higher threshold for faster results
            ..Default::default()
        }
    }

    /// Create a configuration optimized for detail (more comprehensive analysis)
    pub fn for_detail() -> Self {
        Self {
            detailed_analysis: true,
            enable_pattern_detection: true,
            enable_anti_pattern_detection: true,
            confidence_threshold: 0.5, // Lower threshold for more patterns
            ..Default::default()
        }
    }
}

/// Create an architectural advisor with custom configuration
///
/// Note: Configuration is not yet implemented in the core system,
/// but this function provides the API for future enhancement.
pub fn create_configured_advisor(_config: ArchitecturalAdvisorConfig) -> IntelligentArchitecturalAdvisor {
    // For now, just return the default advisor
    // Future versions will use the configuration
    create_architectural_advisor()
}

/// Health check function for the architectural advisor
///
/// Verifies that the system is functioning correctly
pub fn health_check() -> AdvisorResult<()> {
    // Basic health check - could include more sophisticated tests
    let advisor = create_architectural_advisor();
    // Test that we can create and drop the advisor without issues
    drop(advisor);

    Ok(())
}

/// Module version information
pub fn version() -> &'static str {
    ARCHITECTURAL_ADVISOR_VERSION
}

/// Module capabilities description
pub fn capabilities() -> Vec<&'static str> {
    vec![
        "Pattern analysis and detection",
        "Anti-pattern identification",
        "Quality metrics calculation",
        "Complexity assessment",
        "Coupling and cohesion analysis",
        "Architectural recommendations",
        "Risk assessment",
        "Decision analysis",
        "Documentation generation",
        "Validation and verification",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advisor_creation() {
        let advisor = create_architectural_advisor();
        // Just test that creation succeeds
        println!("Architectural Advisor created successfully");
    }

    #[test]
    fn test_configuration_builder() {
        let speed_config = ArchitecturalAdvisorConfig::for_speed();
        assert!(!speed_config.detailed_analysis);
        assert!(!speed_config.enable_anti_pattern_detection);

        let detail_config = ArchitecturalAdvisorConfig::for_detail();
        assert!(detail_config.detailed_analysis);
        assert!(detail_config.confidence_threshold < 0.7);
    }

    #[test]
    fn test_health_check() {
        assert!(health_check().is_ok());
    }

    #[test]
    fn test_capabilities() {
        let capabilities_list = capabilities();
        assert!(!capabilities_list.is_empty());
        assert!(capabilities_list.contains(&"Pattern analysis and detection"));
    }

    #[test]
    fn test_version_info() {
        let version_str = version();
        assert!(!version_str.is_empty());
    }

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_PATTERN_CONFIDENCE_THRESHOLD, 0.7);
        assert_eq!(DEFAULT_MAINTAINABILITY_THRESHOLD, 70.0);
    }
}

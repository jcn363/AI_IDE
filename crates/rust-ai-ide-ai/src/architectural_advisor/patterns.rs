// Pattern detection and architectural analysis algorithms for the architectural advisor

use std::collections::HashMap;

use super::types::*;
use super::AdvisorResult;

/// Pattern detector for architectural patterns and anti-patterns
#[derive(Debug)]
pub struct PatternDetector {
    pattern_knowledge_base: HashMap<String, PatternTemplate>,
    anti_pattern_rules:     Vec<AntiPatternRule>,
}

impl Default for PatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternDetector {
    /// Create a new pattern detector
    pub fn new() -> Self {
        let pattern_knowledge_base = Self::initialize_pattern_knowledge_base();
        let anti_pattern_rules = Self::initialize_anti_pattern_rules();

        Self {
            pattern_knowledge_base,
            anti_pattern_rules,
        }
    }

    /// Detect architectural patterns in the codebase
    pub async fn detect_patterns(&self, codebase_analysis: &CodebaseAnalysis) -> AdvisorResult<Vec<DetectedPattern>> {
        let mut detected_patterns = Vec::new();

        // Analyze directory structure for layering patterns
        let layering_patterns = self.detect_layering_patterns(&codebase_analysis.directory_structure)?;
        detected_patterns.extend(layering_patterns);

        // Analyze module dependencies for patterns
        let dependency_patterns = self.detect_dependency_patterns(&codebase_analysis.dependencies)?;
        detected_patterns.extend(dependency_patterns);

        // Analyze interface patterns
        let interface_patterns = self.detect_interface_patterns(&codebase_analysis.module_organization)?;
        detected_patterns.extend(interface_patterns);

        Ok(detected_patterns)
    }

    /// Identify anti-patterns in the codebase
    pub async fn identify_anti_patterns(
        &self,
        codebase_analysis: &CodebaseAnalysis,
    ) -> AdvisorResult<Vec<AntiPattern>> {
        let mut anti_patterns = Vec::new();

        for rule in &self.anti_pattern_rules {
            if let Some(anti_pattern) = rule.detect(codebase_analysis).await? {
                anti_patterns.push(anti_pattern);
            }
        }

        Ok(anti_patterns)
    }

    /// Detect layering architectural patterns
    fn detect_layering_patterns(
        &self,
        directory_structure: &DirectoryStructure,
    ) -> AdvisorResult<Vec<DetectedPattern>> {
        let mut patterns = Vec::new();

        // Detect layered architecture pattern
        if Self::has_layered_structure(directory_structure) {
            patterns.push(DetectedPattern {
                pattern_type: "Layered Architecture".to_string(),
                confidence:   0.85,
                location:     PatternLocation {
                    files:   vec![],
                    modules: vec![],
                    lines:   None,
                },
                description:  "Clear separation of concerns through layering".to_string(),
                benefits:     vec![
                    "Improved maintainability".to_string(),
                    "Ability to evolve layers independently".to_string(),
                    "Clear boundaries between concerns".to_string(),
                ],
                applications: vec![
                    "Presentation layer for user interfaces".to_string(),
                    "Business logic layer".to_string(),
                    "Data access layer".to_string(),
                ],
            });
        }

        Ok(patterns)
    }

    /// Detect dependency patterns
    fn detect_dependency_patterns(&self, dependencies: &DependencyAnalysis) -> AdvisorResult<Vec<DetectedPattern>> {
        let mut patterns = Vec::new();

        // Detect hexagonal architecture (ports and adapters)
        if Self::has_hexagonal_dependencies(dependencies) {
            patterns.push(DetectedPattern {
                pattern_type: "Hexagonal Architecture".to_string(),
                confidence:   0.75,
                location:     PatternLocation {
                    files:   vec![],
                    modules: vec![],
                    lines:   None,
                },
                description:  "Ports and adapters pattern for clean architecture".to_string(),
                benefits:     vec![
                    "Technology agnostic business logic".to_string(),
                    "Easy testing with mocked adapters".to_string(),
                    "Flexible integration with external systems".to_string(),
                ],
                applications: vec![
                    "Adhapters for database access".to_string(),
                    "Ports for business logic interfaces".to_string(),
                    "UI frameworks as adapter implementations".to_string(),
                ],
            });
        }

        Ok(patterns)
    }

    /// Detect interface patterns
    fn detect_interface_patterns(
        &self,
        module_organization: &ModuleOrganization,
    ) -> AdvisorResult<Vec<DetectedPattern>> {
        let mut patterns = Vec::new();

        // Detect repository pattern usage
        if Self::has_repository_interfaces(module_organization) {
            patterns.push(DetectedPattern {
                pattern_type: "Repository Pattern".to_string(),
                confidence:   0.80,
                location:     PatternLocation {
                    files:   vec![],
                    modules: vec![],
                    lines:   None,
                },
                description:  "Abstraction of data access through repository interfaces".to_string(),
                benefits:     vec![
                    "Decoupling of business logic from data access".to_string(),
                    "Testability through repository mocks".to_string(),
                    "Consistent data access patterns".to_string(),
                ],
                applications: vec![
                    "UserRepository for user data access".to_string(),
                    "ProductRepository for product management".to_string(),
                ],
            });
        }

        Ok(patterns)
    }

    /// Check if directory structure indicates layered architecture
    fn has_layered_structure(directory_structure: &DirectoryStructure) -> bool {
        let organization_patterns = &directory_structure.organization_patterns;

        let layer_indicators = [
            "layer",
            "presentation",
            "business",
            "data",
            "ui",
            "domain",
            "infrastructure",
            "application",
        ];

        let has_layer_indicators = layer_indicators.iter().any(|&indicator| {
            organization_patterns
                .iter()
                .any(|pattern| pattern.contains(indicator))
        });

        // Check for common layer naming patterns
        let has_common_layers = directory_structure.directories.iter().any(|dir| {
            dir.to_lowercase().contains("ui")
                || dir.to_lowercase().contains("controller")
                || dir.to_lowercase().contains("service")
                || dir.to_lowercase().contains("repository")
                || dir.to_lowercase().contains("model")
        });

        has_layer_indicators || has_common_layers
    }

    /// Check for hexagonal architecture dependency patterns
    fn has_hexagonal_dependencies(_dependencies: &DependencyAnalysis) -> bool {
        // This would involve analyzing dependency directions and abstractions
        // For now, return false - would implement based on actual dependency analysis
        false
    }

    /// Check for repository pattern usage
    fn has_repository_interfaces(_module_organization: &ModuleOrganization) -> bool {
        // Would check for interfaces ending with "Repository" or similar patterns
        false
    }

    /// Initialize the pattern knowledge base
    fn initialize_pattern_knowledge_base() -> HashMap<String, PatternTemplate> {
        let mut kb = HashMap::new();

        // MVC Pattern
        kb.insert("MVC".to_string(), PatternTemplate {
            name:             "Model-View-Controller".to_string(),
            description:      "Separation of presentation, business logic, and data layers".to_string(),
            indicators:       vec![
                "controller".to_string(),
                "view".to_string(),
                "model".to_string(),
                "presentation".to_string(),
                "business".to_string(),
            ],
            confidence_rules: vec![ConfidenceRule {
                condition: "Has separate layers".to_string(),
                weight:    0.8,
            }],
        });

        // Microservices Pattern
        kb.insert("Microservices".to_string(), PatternTemplate {
            name:             "Microservices Architecture".to_string(),
            description:      "Decoupled services with bounded contexts".to_string(),
            indicators:       vec![
                "service".to_string(),
                "api".to_string(),
                "bounded context".to_string(),
                "domain".to_string(),
            ],
            confidence_rules: vec![ConfidenceRule {
                condition: "Has multiple independent services".to_string(),
                weight:    0.9,
            }],
        });

        kb
    }

    /// Initialize anti-pattern detection rules
    fn initialize_anti_pattern_rules() -> Vec<AntiPatternRule> {
        vec![
            AntiPatternRule {
                name:                    "God Object".to_string(),
                description:             "A class or module that knows too much and does too much".to_string(),
                severity:                0.8,
                detection_criteria:      vec![DetectionCriterion::MetricThreshold(
                    "module_size_lines".to_string(),
                    1000.0,
                    Comparison::GreaterThan,
                )],
                refactoring_suggestions: vec![
                    "Apply Single Responsibility Principle".to_string(),
                    "Split into smaller, focused modules".to_string(),
                    "Extract feature-specific submodules".to_string(),
                ],
            },
            AntiPatternRule {
                name:                    "Circular Dependencies".to_string(),
                description:             "Modules that depend on each other creating tight coupling".to_string(),
                severity:                0.9,
                detection_criteria:      vec![DetectionCriterion::DependencyPattern(
                    "circular".to_string(),
                )],
                refactoring_suggestions: vec![
                    "Introduce interface segregation".to_string(),
                    "Use dependency injection".to_string(),
                    "Create mediator pattern".to_string(),
                ],
            },
        ]
    }
}

/// Pattern template for detection rules
#[derive(Debug)]
struct PatternTemplate {
    name:             String,
    description:      String,
    indicators:       Vec<String>,
    confidence_rules: Vec<ConfidenceRule>,
}

/// Confidence rule for pattern detection
#[derive(Debug)]
struct ConfidenceRule {
    condition: String,
    weight:    f32,
}

/// Anti-pattern detection rule
#[derive(Debug)]
struct AntiPatternRule {
    name:                    String,
    description:             String,
    severity:                f32,
    detection_criteria:      Vec<DetectionCriterion>,
    refactoring_suggestions: Vec<String>,
}

impl AntiPatternRule {
    async fn detect(&self, _codebase_analysis: &CodebaseAnalysis) -> AdvisorResult<Option<AntiPattern>> {
        // Would implement actual detection logic based on criteria
        // For now, return None
        Ok(None)
    }
}

/// Detection criteria for anti-patterns
#[derive(Debug)]
enum DetectionCriterion {
    MetricThreshold(String, f32, Comparison),
    DependencyPattern(String),
    CodePattern(Vec<String>),
}

/// Comparison operators for metric thresholds
#[derive(Debug)]
enum Comparison {
    LessThan,
    GreaterThan,
    Equal,
}

/// Codebase analysis structure
#[derive(Debug, Default)]
pub struct CodebaseAnalysis {
    pub directory_structure: DirectoryStructure,
    pub module_organization: ModuleOrganization,
    pub dependencies:        DependencyAnalysis,
}

/// Directory structure analysis
#[derive(Debug, Default)]
pub struct DirectoryStructure {
    pub total_files:           usize,
    pub directories:           Vec<String>,
    pub file_types:            HashMap<String, usize>,
    pub organization_patterns: Vec<String>,
    pub issues:                Vec<String>,
}

/// Module organization analysis
#[derive(Debug, Default)]
pub struct ModuleOrganization {
    pub modules:               Vec<String>,
    pub module_hierarchy:      HashMap<String, Vec<String>>,
    pub public_interfaces:     Vec<String>,
    pub internal_dependencies: HashMap<String, Vec<String>>,
    pub circular_dependencies: Vec<(String, String)>,
}

/// Dependency analysis
#[derive(Debug, Default)]
pub struct DependencyAnalysis {
    pub internal_dependencies: HashMap<String, Vec<String>>,
    pub external_dependencies: Vec<String>,
    pub dependency_depth:      HashMap<String, usize>,
    pub shared_dependencies:   Vec<String>,
    pub unused_dependencies:   Vec<String>,
}

/// Pattern analysis utilities
pub mod analysis_utils {
    use super::*;

    /// Calculate pattern confidence based on evidence
    pub fn calculate_pattern_confidence(
        detected_indicators: usize,
        total_indicators: usize,
        strength_factors: Vec<f32>,
    ) -> f32 {
        if total_indicators == 0 {
            return 0.0;
        }

        let base_confidence = detected_indicators as f32 / total_indicators as f32;
        let strength_bonus = strength_factors.iter().sum::<f32>() / strength_factors.len() as f32;

        (base_confidence + strength_bonus * 0.2).clamp(0.0, 1.0)
    }

    /// Analyze coupling level between modules
    pub fn analyze_coupling_level(dependencies: &DependencyAnalysis) -> CouplingLevel {
        let total_dependencies: usize = dependencies
            .internal_dependencies
            .values()
            .map(|deps| deps.len())
            .sum();

        match total_dependencies {
            0..=10 => CouplingLevel::Low,
            11..=25 => CouplingLevel::Moderate,
            26..=50 => CouplingLevel::High,
            _ => CouplingLevel::VeryHigh,
        }
    }

    /// Coupling level classification
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum CouplingLevel {
        Low,
        Moderate,
        High,
        VeryHigh,
    }

    /// Detect technology stack from file extensions
    pub fn detect_technology_stack(file_types: &HashMap<String, usize>) -> Vec<String> {
        let mut technologies = Vec::new();

        if file_types.contains_key("rs") {
            technologies.push("Rust".to_string());
        }

        if file_types.contains_key("js") || file_types.contains_key("ts") {
            technologies.push("JavaScript/TypeScript".to_string());
        }

        if file_types.contains_key("py") {
            technologies.push("Python".to_string());
        }

        if file_types.contains_key("java") {
            technologies.push("Java".to_string());
        }

        if file_types.contains_key("html") {
            technologies.push("Web Frontend".to_string());
        }

        technologies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_detector_creation() {
        let detector = PatternDetector::new();
        assert!(!detector.pattern_knowledge_base.is_empty());
        assert!(!detector.anti_pattern_rules.is_empty());
    }

    #[test]
    fn test_has_layered_structure_detection() {
        let structure = DirectoryStructure {
            directories: vec![
                "src/ui".to_string(),
                "src/business".to_string(),
                "src/data".to_string(),
            ],
            organization_patterns: vec![],
            ..Default::default()
        };

        assert!(PatternDetector::has_layered_structure(&structure));
    }

    #[test]
    fn test_confidence_calculation() {
        let confidence = analysis_utils::calculate_pattern_confidence(3, 5, vec![0.8, 0.6]);
        assert!(confidence > 0.5 && confidence < 0.8); // Should be around 0.68
    }

    #[test]
    fn test_coupling_analysis() {
        let mut deps = DependencyAnalysis::default();
        deps.internal_dependencies
            .insert("module_a".to_string(), vec![
                "module_b".to_string(),
                "module_c".to_string(),
            ]);

        let coupling = analysis_utils::analyze_coupling_level(&deps);
        assert_eq!(coupling, analysis_utils::CouplingLevel::Moderate);
    }
}

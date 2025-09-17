//! Architecture pattern suggestions

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use rust_ai_ide_ai_analysis::ArchitectureSuggestion;
use uuid::Uuid;

/// Analyzer for architecture patterns and suggestions
#[derive(Clone)]
pub struct ArchitecturePatternAnalyzer {
    known_patterns: std::collections::HashMap<String, PatternTemplate>,
}

#[derive(Clone, Debug)]
pub struct PatternTemplate {
    pub name: String,
    pub category: PatternCategory,
    pub detector: PatternDetector,
    pub suggestion: String,
    pub confidence_threshold: f64,
}

#[derive(Clone, Debug)]
pub enum PatternDetector {
    /// Detect based on function names
    FunctionPattern(Vec<String>),
    /// Detect based on struct/enum names
    TypePattern(Vec<String>),
    /// Detect based on trait names
    TraitPattern(Vec<String>),
    /// Detect based on module structure
    ModulePattern(Vec<String>),
    /// Combined detection strategy
    Composite(Vec<PatternDetector>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PatternCategory {
    Creational,
    Structural,
    Behavioral,
}

impl ArchitecturePatternAnalyzer {
    /// Create a new pattern analyzer
    pub fn new() -> Self {
        let mut analyzer = Self {
            known_patterns: std::collections::HashMap::new(),
        };
        analyzer.load_standard_patterns();
        analyzer
    }

    /// Load standard design patterns
    fn load_standard_patterns(&mut self) {
        // MVC Pattern
        self.known_patterns.insert(
            "MVC".to_string(),
            PatternTemplate {
                name: "Model-View-Controller".to_string(),
                category: PatternCategory::Structural,
                detector: PatternDetector::ModulePattern(vec![
                    "model".to_string(),
                    "view".to_string(),
                    "controller".to_string(),
                ]),
                suggestion: "Consider more modern alternatives like MVVM or clean architecture"
                    .to_string(),
                confidence_threshold: 0.8,
            },
        );

        // Repository Pattern
        self.known_patterns.insert(
            "Repository".to_string(),
            PatternTemplate {
                name: "Repository Pattern".to_string(),
                category: PatternCategory::Behavioral,
                detector: PatternDetector::Composite(vec![
                    PatternDetector::TraitPattern(vec!["Repository".to_string()]),
                    PatternDetector::FunctionPattern(vec![
                        "find_by".to_string(),
                        "save".to_string(),
                        "delete".to_string(),
                    ]),
                ]),
                suggestion: "Excellent pattern for data access abstraction!".to_string(),
                confidence_threshold: 0.6,
            },
        );

        // Factory Pattern
        self.known_patterns.insert(
            "Factory".to_string(),
            PatternTemplate {
                name: "Factory Pattern".to_string(),
                category: PatternCategory::Creational,
                detector: PatternDetector::FunctionPattern(vec![
                    "create_".to_string(),
                    "make_".to_string(),
                    "build_".to_string(),
                ]),
                suggestion: "Strong creational pattern usage detected".to_string(),
                confidence_threshold: 0.7,
            },
        );

        // Strategy Pattern
        self.known_patterns.insert(
            "Strategy".to_string(),
            PatternTemplate {
                name: "Strategy Pattern".to_string(),
                category: PatternCategory::Behavioral,
                detector: PatternDetector::Composite(vec![
                    PatternDetector::TraitPattern(vec![
                        "Strategy".to_string(),
                        "Algorithm".to_string(),
                    ]),
                    PatternDetector::TypePattern(vec!["Box<dyn".to_string(), "Arc<".to_string()]),
                ]),
                suggestion: "Runtime algorithm selection pattern - ensure proper error handling"
                    .to_string(),
                confidence_threshold: 0.8,
            },
        );

        // Adapter Pattern
        self.known_patterns.insert(
            "Adapter".to_string(),
            PatternTemplate {
                name: "Adapter Pattern".to_string(),
                category: PatternCategory::Structural,
                detector: PatternDetector::TraitPattern(vec![
                    "Adapter".to_string(),
                    "Wrapper".to_string(),
                ]),
                suggestion: "Good for interface compatibility".to_string(),
                confidence_threshold: 0.6,
            },
        );

        // Monolithic Pattern (anti-pattern detection)
        self.known_patterns.insert(
            "Monolithic".to_string(),
            PatternTemplate {
                name: "Potential Monolithic Structure".to_string(),
                category: PatternCategory::Structural,
                detector: PatternDetector::ModulePattern(vec![
                    "huge_modules".to_string(),
                    "many_functions".to_string(),
                ]),
                suggestion: "Consider breaking down into smaller, focused modules".to_string(),
                confidence_threshold: 0.5,
            },
        );
    }

    /// Analyze multiple files for patterns
    pub async fn analyze_multiple_files(
        &self,
        files: &[(&str, &str)],
    ) -> Result<Vec<ArchitectureSuggestion>, super::StyleCheckError> {
        let mut suggestions = Vec::new();
        let mut all_patterns_found = HashSet::new();

        for (file_path, content) in files {
            let file_patterns = self.analyze_single_file(file_path, content).await?;
            for pattern in &file_patterns {
                all_patterns_found.insert(pattern.pattern.clone());
                suggestions.push(pattern.clone());
            }
        }

        // Add cross-file suggestions
        let cross_file_suggestions = self.generate_cross_file_suggestions(&all_patterns_found);
        suggestions.extend(cross_file_suggestions);

        Ok(suggestions)
    }

    /// Analyze a single file for patterns
    pub async fn analyze_single_file(
        &self,
        file_path: &str,
        content: &str,
    ) -> Result<Vec<ArchitectureSuggestion>, super::StyleCheckError> {
        let mut suggestions = Vec::new();

        for (pattern_name, template) in &self.known_patterns {
            let confidence = self.detect_pattern(content, &template.detector);

            if confidence >= template.confidence_threshold {
                suggestions.push(ArchitectureSuggestion {
                    pattern: template.name.clone(),
                    confidence,
                    location: rust_ai_ide_ai_analysis::Location {
                        file: file_path.to_string(),
                        line: 1,
                        column: 0,
                        offset: 0,
                    },
                    description: format!("{} pattern detected", template.name),
                    benefits: vec![
                        "Separation of concerns".to_string(),
                        "Maintainability".to_string(),
                        "Testability".to_string(),
                    ],
                    implementation_steps: vec![
                        "Implement the pattern interfaces".to_string(),
                        "Move code to appropriate modules".to_string(),
                        "Update imports and dependencies".to_string(),
                    ],
                });
            }
        }

        Ok(suggestions)
    }

    /// Detect a specific pattern in content
    fn detect_pattern(&self, content: &str, detector: &PatternDetector) -> f64 {
        match detector {
            PatternDetector::FunctionPattern(patterns) => {
                let found_count = patterns
                    .iter()
                    .filter(|pattern| content.contains(*pattern))
                    .count();
                found_count as f64 / patterns.len() as f64
            }
            PatternDetector::TypePattern(patterns) => {
                let found_count = patterns
                    .iter()
                    .filter(|pattern| content.contains(*pattern))
                    .count();
                found_count as f64 / patterns.len() as f64
            }
            PatternDetector::TraitPattern(patterns) => {
                let found_count = patterns
                    .iter()
                    .filter(|pattern| content.contains(*pattern))
                    .count();
                found_count as f64 / patterns.len() as f64
            }
            PatternDetector::ModulePattern(patterns) => {
                // For module patterns, we check for structural indicators
                let content_lines = content.lines().count();
                let function_lines = content
                    .lines()
                    .filter(|line| {
                        line.trim().starts_with("fn ") | line.trim().starts_with("pub fn ")
                    })
                    .count();

                // High ratio of functions to lines suggests potential monolithic structure
                if content_lines > 100 && function_lines > 10 {
                    0.7
                } else {
                    0.0
                }
            }
            PatternDetector::Composite(detectors) => {
                let total_confidence = detectors
                    .iter()
                    .map(|d| self.detect_pattern(content, d))
                    .sum::<f64>();

                total_confidence / detectors.len() as f64
            }
        }
    }

    /// Generate suggestions based on cross-file analysis
    fn generate_cross_file_suggestions(
        &self,
        patterns_found: &HashSet<String>,
    ) -> Vec<ArchitectureSuggestion> {
        let mut suggestions = Vec::new();
        let pattern_names: HashSet<String> = patterns_found.iter().cloned().collect();

        // Check for incompatible patterns
        if pattern_names.contains("MVC") && pattern_names.contains("Repository") {
            suggestions.push(ArchitectureSuggestion {
                pattern: "Potential Architecture Mismatch".to_string(),
                confidence: 0.8,
                location: rust_ai_ide_ai_analysis::Location {
                    file: "multiple_files".to_string(),
                    line: 1,
                    column: 0,
                    offset: 0,
                },
                description: "MVC and Repository patterns detected together".to_string(),
                benefits: vec![
                    "Unified architecture".to_string(),
                    "Clear separation".to_string(),
                ],
                implementation_steps: vec![
                    "Document the relationship between MVC and Repository".to_string(),
                    "Ensure consistent data flow".to_string(),
                    "Consider moving to clean architecture".to_string(),
                ],
            });
        }

        // Check for missing patterns
        if pattern_names.contains("Factory") && !pattern_names.contains("Strategy") {
            suggestions.push(ArchitectureSuggestion {
                pattern: "Missing Strategy Pattern".to_string(),
                confidence: 0.6,
                location: rust_ai_ide_ai_analysis::Location {
                    file: "multiple_files".to_string(),
                    line: 1,
                    column: 0,
                    offset: 0,
                },
                description: "Factory pattern without Strategy pattern may limit flexibility"
                    .to_string(),
                benefits: vec![
                    "Replaced object creation".to_string(),
                    "Configuration flexibility".to_string(),
                ],
                implementation_steps: vec![
                    "Define strategy interfaces".to_string(),
                    "Implement different algorithms".to_string(),
                    "Use factory to select strategies".to_string(),
                ],
            });
        }

        // Suggest hexagonal architecture
        if pattern_names.contains("Repository") && pattern_names.contains("Adapter") {
            suggestions.push(ArchitectureSuggestion {
                pattern: "Hexagonal Architecture Candidate".to_string(),
                confidence: 0.7,
                location: rust_ai_ide_ai_analysis::Location {
                    file: "multiple_files".to_string(),
                    line: 1,
                    column: 0,
                    offset: 0,
                },
                description: "Your patterns lend themselves well to hexagonal architecture"
                    .to_string(),
                benefits: vec![
                    "Technology independence".to_string(),
                    "Testability".to_string(),
                    "Business logic isolation".to_string(),
                ],
                implementation_steps: vec![
                    "Define domain entities".to_string(),
                    "Create application services".to_string(),
                    "Implement ports and adapters".to_string(),
                    "Configure dependency injection".to_string(),
                ],
            });
        }

        suggestions
    }

    /// Get available patterns for reference
    pub fn get_available_patterns(&self) -> Vec<&PatternTemplate> {
        self.known_patterns.values().collect()
    }

    /// Add a custom pattern
    pub fn add_pattern(&mut self, name: &str, template: PatternTemplate) {
        self.known_patterns.insert(name.to_string(), template);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_analyzer_creation() {
        let analyzer = ArchitecturePatternAnalyzer::new();
        assert!(!analyzer.known_patterns.is_empty());
    }

    #[test]
    fn test_standard_patterns_loaded() {
        let analyzer = ArchitecturePatternAnalyzer::new();
        assert!(analyzer.known_patterns.contains_key("MVC"));
        assert!(analyzer.known_patterns.contains_key("Repository"));
        assert!(analyzer.known_patterns.contains_key("Factory"));
    }

    #[test]
    fn test_pattern_detection() {
        let analyzer = ArchitecturePatternAnalyzer::new();

        // Test repository pattern detection
        let repo_code = r#"
            trait Repository {
                fn find_by_id(&self, id: u64) -> Option<Entity>;
                fn save(&self, entity: Entity) -> Result<(), Error>;
                fn delete(&self, id: u64) -> Result<(), Error>;
            }
        "#;

        if let Some(repository_template) = analyzer.known_patterns.get("Repository") {
            let confidence = analyzer.detect_pattern(repo_code, &repository_template.detector);
            assert!(confidence > 0.0);
        }
    }

    #[tokio::test]
    async fn test_single_file_analysis() {
        let analyzer = ArchitecturePatternAnalyzer::new();

        let factory_code = r#"
            pub fn create_connection(addr: &str) -> Result<TcpConnection, Error> {
                // Factory implementation
            }

            pub fn make_http_client(config: ClientConfig) -> Result<HttpClient, Error> {
                // Factory implementation
            }
        "#;

        let suggestions = analyzer
            .analyze_single_file("test.rs", factory_code)
            .await
            .unwrap();

        // Should detect factory pattern
        assert!(suggestions.iter().any(|s| s.pattern.contains("Factory")));
    }
}

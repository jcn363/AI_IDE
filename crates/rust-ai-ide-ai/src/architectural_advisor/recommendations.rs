// Recommendation generation engine and risk assessment for architectural advice

use super::types::*;
use super::AdvisorResult;

/// Recommendation generator for architectural guidance
#[derive(Debug)]
pub struct RecommendationGenerator {
    recommendation_templates: std::collections::HashMap<String, RecommendationTemplate>,
}

impl Default for RecommendationGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RecommendationGenerator {
    /// Create a new recommendation generator
    pub fn new() -> Self {
        Self {
            recommendation_templates: Self::initialize_templates(),
        }
    }

    /// Generate primary recommendations based on analysis
    pub async fn generate_primary_recommendations(
        &self,
        analysis: &PatternAnalysis,
        quality: &QualityAssessment,
    ) -> AdvisorResult<Vec<ArchitecturalRecommendation>> {
        let mut recommendations = Vec::new();

        // Recommendations based on anti-patterns
        for anti_pattern in &analysis.anti_patterns {
            if anti_pattern.severity > 0.7 {
                let recommendation = self.generate_anti_pattern_recommendation(anti_pattern)?;
                recommendations.push(recommendation);
            }
        }

        // Recommendations based on quality metrics
        if quality.complexity_score < 0.5 {
            recommendations.push(self.generate_complexity_recommendation().await?);
        }

        // Recommendations based on coupling issues
        if quality.coupling_score > 0.7 {
            recommendations.push(self.generate_coupling_recommendation().await?);
        }

        // Recommendations based on cohesion issues
        if quality.cohesion_score < 0.5 {
            recommendations.push(self.generate_cohesion_recommendation().await?);
        }

        Ok(recommendations)
    }

    /// Generate secondary suggestions
    pub async fn generate_secondary_suggestions(
        &self,
        analysis: &PatternAnalysis,
    ) -> AdvisorResult<Vec<ArchitecturalSuggestion>> {
        let mut suggestions = Vec::new();

        // Generate suggestions based on detected patterns
        for pattern in &analysis.detected_patterns {
            if let Some(template) = self.recommendation_templates.get(&pattern.pattern_type) {
                if let Some(suggestion) = self.generate_pattern_suggestion(pattern, template).await
                {
                    suggestions.push(suggestion);
                }
            }
        }

        // Generic improvement suggestions
        if analysis.complexity_assessment.overall_complexity == ComplexityLevel::High {
            suggestions.push(ArchitecturalSuggestion {
                title: "Consider implementing CQRS pattern".to_string(),
                description: "Separate read and write operations for better scalability"
                    .to_string(),
                category: SuggestionCategory::DesignPattern,
                priority: PriorityLevel::Medium,
                impact: ImpactLevel::Major,
            });
        }

        Ok(suggestions)
    }

    /// Assess risks based on analysis
    pub async fn assess_risks(
        &self,
        analysis: &PatternAnalysis,
        quality: &QualityAssessment,
    ) -> AdvisorResult<RiskAssessment> {
        let mut risk_factors = Vec::new();

        // Calculate risks from anti-patterns
        for anti_pattern in &analysis.anti_patterns {
            if anti_pattern.severity > 0.6 {
                risk_factors.push(RiskFactor {
                    factor: anti_pattern.anti_pattern_type.clone(),
                    probability: anti_pattern.severity,
                    impact: 0.7,
                    description: format!(
                        "{} increases system risk",
                        anti_pattern.anti_pattern_type
                    ),
                });
            }
        }

        // Calculate risks from quality metrics
        if quality.complexity_score < 0.5 {
            risk_factors.push(RiskFactor {
                factor: "High Complexity".to_string(),
                probability: 0.8,
                impact: 0.6,
                description: "High complexity increases maintenance and bug fix costs".to_string(),
            });
        }

        let overall_risk = if risk_factors.is_empty() {
            0.2
        } else {
            risk_factors
                .iter()
                .map(|r| r.probability * r.impact)
                .sum::<f32>()
                / risk_factors.len() as f32
        };

        Ok(RiskAssessment {
            overall_risk,
            risk_factors,
            mitigation_strategies: self.generate_mitigation_strategies(overall_risk),
        })
    }

    /// Identify priority actions
    pub fn identify_priority_actions(
        &self,
        primary: &[ArchitecturalRecommendation],
        secondary: &[ArchitecturalSuggestion],
    ) -> AdvisorResult<Vec<PriorityAction>> {
        let mut actions = Vec::new();

        // Add high-priority primary recommendations as priority actions
        for rec in primary {
            if rec.implementation_effort <= ImplementationEffort::Medium {
                actions.push(PriorityAction {
                    action: rec.title.clone(),
                    deadline: Some("30 days".to_string()),
                    responsible: None,
                    dependencies: rec.prerequisites.clone(),
                });
            }
        }

        // Add impactful secondary suggestions
        for sug in secondary {
            if sug.impact >= ImpactLevel::Major && sug.priority >= PriorityLevel::High {
                actions.push(PriorityAction {
                    action: sug.title.clone(),
                    deadline: Some("60 days".to_string()),
                    responsible: None,
                    dependencies: vec![],
                });
            }
        }

        Ok(actions)
    }

    /// Generate architectural roadmap
    pub fn generate_roadmap(
        &self,
        primary: &[ArchitecturalRecommendation],
        secondary: &[ArchitecturalSuggestion],
    ) -> AdvisorResult<ArchitecturalRoadmap> {
        let mut short_term = Vec::new();
        let mut medium_term = Vec::new();
        let mut long_term = Vec::new();

        for rec in primary {
            match rec.implementation_effort {
                ImplementationEffort::Low => {
                    short_term.push(RoadmapItem {
                        title: rec.title.clone(),
                        description: rec.description.clone(),
                        timeline: "1-3 months".to_string(),
                        dependencies: rec.prerequisites.clone(),
                        success_criteria: vec![
                            "Implementation completed".to_string(),
                            "No breaking changes introduced".to_string(),
                            "Performance improvements verified".to_string(),
                        ],
                    });
                }
                ImplementationEffort::Medium => {
                    medium_term.push(RoadmapItem {
                        title: rec.title.clone(),
                        description: rec.description.clone(),
                        timeline: "3-6 months".to_string(),
                        dependencies: rec.prerequisites.clone(),
                        success_criteria: vec![
                            "Implementation completed".to_string(),
                            "Team trained on new patterns".to_string(),
                            "Performance benchmarks established".to_string(),
                        ],
                    });
                }
                ImplementationEffort::High | ImplementationEffort::VeryHigh => {
                    long_term.push(RoadmapItem {
                        title: rec.title.clone(),
                        description: rec.description.clone(),
                        timeline: "6-12 months".to_string(),
                        dependencies: rec.prerequisites.clone(),
                        success_criteria: vec![
                            "Major refactoring completed".to_string(),
                            "System performance improved".to_string(),
                            "Team competency established".to_string(),
                        ],
                    });
                }
            }
        }

        Ok(ArchitecturalRoadmap {
            short_term,
            medium_term,
            long_term,
        })
    }

    /// Generate recommendation for specific anti-pattern
    fn generate_anti_pattern_recommendation(
        &self,
        anti_pattern: &AntiPattern,
    ) -> AdvisorResult<ArchitecturalRecommendation> {
        Ok(ArchitecturalRecommendation {
            title: format!("Address {}", anti_pattern.anti_pattern_type),
            description: format!(
                "High-severity anti-pattern detected: {}",
                anti_pattern.description
            ),
            rationale: "Anti-patterns reduce system maintainability and can lead to technical debt"
                .to_string(),
            expected_benefits: vec![
                "Improved system stability".to_string(),
                "Reduced maintenance costs".to_string(),
                "Better code quality".to_string(),
            ],
            implementation_effort: match anti_pattern.severity {
                0.0..=0.5 => ImplementationEffort::Low,
                0.5..=0.7 => ImplementationEffort::Medium,
                0.7..=0.9 => ImplementationEffort::High,
                _ => ImplementationEffort::VeryHigh,
            },
            risk_level: if anti_pattern.severity > 0.8 {
                RiskLevel::High
            } else {
                RiskLevel::Medium
            },
            prerequisites: vec![
                "Code review scheduled".to_string(),
                "Testing strategy prepared".to_string(),
            ],
            alternatives: anti_pattern.refactoring_suggestions.clone(),
        })
    }

    /// Generate complexity reduction recommendation
    async fn generate_complexity_recommendation(
        &self,
    ) -> AdvisorResult<ArchitecturalRecommendation> {
        Ok(ArchitecturalRecommendation {
            title: "Refactor Complex Modules".to_string(),
            description: "High system complexity detected that impacts maintainability".to_string(),
            rationale: "Complex systems are harder to maintain, test, and modify".to_string(),
            expected_benefits: vec![
                "Easier maintenance".to_string(),
                "Reduced bug introduction rate".to_string(),
                "Faster feature development".to_string(),
            ],
            implementation_effort: ImplementationEffort::High,
            risk_level: RiskLevel::High,
            prerequisites: vec![
                "Code complexity analysis completed".to_string(),
                "Refactoring strategy approved".to_string(),
                "Testing resources allocated".to_string(),
            ],
            alternatives: vec![
                "Extract microservices".to_string(),
                "Apply design patterns".to_string(),
                "Create domain-specific languages".to_string(),
            ],
        })
    }

    /// Generate coupling reduction recommendation
    async fn generate_coupling_recommendation(&self) -> AdvisorResult<ArchitecturalRecommendation> {
        Ok(ArchitecturalRecommendation {
            title: "Reduce Module Coupling".to_string(),
            description: "High coupling detected between modules".to_string(),
            rationale: "Tight coupling makes changes difficult and increases risk".to_string(),
            expected_benefits: vec![
                "Independent module evolution".to_string(),
                "Reduced change impact".to_string(),
                "Easier testing".to_string(),
            ],
            implementation_effort: ImplementationEffort::Medium,
            risk_level: RiskLevel::Medium,
            prerequisites: vec!["Dependency analysis completed".to_string()],
            alternatives: vec![
                "Interface segregation".to_string(),
                "Dependency injection".to_string(),
                "Event-driven architecture".to_string(),
            ],
        })
    }

    /// Generate cohesion improvement recommendation
    async fn generate_cohesion_recommendation(&self) -> AdvisorResult<ArchitecturalRecommendation> {
        Ok(ArchitecturalRecommendation {
            title: "Improve Module Cohesion".to_string(),
            description: "Low cohesion detected within modules".to_string(),
            rationale: "Low cohesion indicates modules have mixed responsibilities".to_string(),
            expected_benefits: vec![
                "Single responsibility principle".to_string(),
                "Easier testing and maintenance".to_string(),
                "Clear module boundaries".to_string(),
            ],
            implementation_effort: ImplementationEffort::Medium,
            risk_level: RiskLevel::Low,
            prerequisites: vec!["Module analysis completed".to_string()],
            alternatives: vec![
                "Extract related functionality".to_string(),
                "Apply SOLID principles".to_string(),
                "Domain-driven design".to_string(),
            ],
        })
    }

    /// Generate suggestion based on detected pattern
    async fn generate_pattern_suggestion(
        &self,
        pattern: &DetectedPattern,
        template: &RecommendationTemplate,
    ) -> Option<ArchitecturalSuggestion> {
        (template.suggestion_generator)(pattern).await.ok()
    }

    /// Generate mitigation strategies based on risk level
    fn generate_mitigation_strategies(&self, overall_risk: f32) -> Vec<String> {
        let mut strategies = Vec::new();

        strategies.push("Regular code reviews".to_string());

        if overall_risk > 0.7 {
            strategies.push("Automated testing implementation".to_string());
            strategies.push("Continuous integration improvements".to_string());
        }

        if overall_risk > 0.5 {
            strategies.push("Architecture documentation updates".to_string());
            strategies.push("Team training on identified patterns".to_string());
        }

        strategies.push("Regular architectural assessments".to_string());

        strategies
    }

    /// Initialize recommendation templates
    fn initialize_templates() -> std::collections::HashMap<String, RecommendationTemplate> {
        let mut templates = std::collections::HashMap::new();

        templates.insert(
            "God Object".to_string(),
            RecommendationTemplate {
                pattern_name: "God Object".to_string(),
                suggestion_generator: Box::new(move |pattern: &DetectedPattern| {
                    Box::pin(async move {
                        Ok(ArchitecturalSuggestion {
                            title: format!("Refactor {}", pattern.pattern_type),
                            description: format!(
                                "Apply Single Responsibility Principle to {}",
                                pattern.pattern_type.to_lowercase()
                            ),
                            category: SuggestionCategory::Refactoring,
                            priority: PriorityLevel::High,
                            impact: ImpactLevel::Major,
                        })
                    })
                }),
            },
        );

        templates
    }
}

/// Recommendation template for pattern-based suggestions
pub struct RecommendationTemplate {
    pattern_name: String,
    suggestion_generator: Box<
        dyn Fn(
                &DetectedPattern,
            )
                -> futures::future::BoxFuture<'_, AdvisorResult<ArchitecturalSuggestion>>
            + Send
            + Sync,
    >,
}

impl std::fmt::Debug for RecommendationTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecommendationTemplate")
            .field("pattern_name", &self.pattern_name)
            .finish()
    }
}

/// Quality assessment using analysis results
#[derive(Debug, Clone)]
pub struct QualityAssessment {
    pub overall_score: f32,
    pub maintainability_score: f32,
    pub complexity_score: f32,
    pub coupling_score: f32,
    pub cohesion_score: f32,
    pub grading_scale: String,
}

/// Decision analysis using evaluation framework
#[derive(Debug)]
pub struct DecisionAnalysisResult {
    pub decision: DecisionOption,
    pub recommendation: DecisionRecommendation,
    pub analysis: std::collections::HashMap<String, f32>,
    pub trade_offs: Vec<TradeOff>,
    pub risks: Vec<String>,
    pub assumptions: Vec<String>,
}

/// Analysis utilities for recommendation generation
pub mod recommendation_utils {
    use super::*;

    /// Calculate priority score for suggestions
    pub fn calculate_priority_score(suggestion: &ArchitecturalSuggestion) -> f32 {
        let priority_weight = match suggestion.priority {
            PriorityLevel::High => 1.0,
            PriorityLevel::Medium => 0.6,
            PriorityLevel::Low => 0.3,
        };

        let impact_weight = match suggestion.impact {
            ImpactLevel::Critical => 1.0,
            ImpactLevel::Major => 0.8,
            ImpactLevel::Moderate => 0.5,
            ImpactLevel::Minor => 0.2,
        };

        priority_weight * impact_weight
    }

    /// Group recommendations by priority level
    pub fn group_recommendations_by_priority(
        recommendations: &[ArchitecturalRecommendation],
    ) -> std::collections::HashMap<PriorityLevel, Vec<&ArchitecturalRecommendation>> {
        let mut grouped = std::collections::HashMap::new();

        for rec in recommendations {
            // Determine priority based on risk and effort
            let priority = match (&rec.risk_level, &rec.implementation_effort) {
                (RiskLevel::Critical, _) => PriorityLevel::High,
                (RiskLevel::High, ImplementationEffort::Low | ImplementationEffort::Medium) => {
                    PriorityLevel::High
                }
                (RiskLevel::High, _) => PriorityLevel::Medium,
                (RiskLevel::Medium, ImplementationEffort::VeryHigh) => PriorityLevel::Low,
                (RiskLevel::Medium, _) => PriorityLevel::Medium,
                (RiskLevel::Low, ImplementationEffort::Low) => PriorityLevel::Medium,
                _ => PriorityLevel::Low,
            };

            grouped.entry(priority).or_insert_with(Vec::new).push(rec);
        }

        grouped
    }

    /// Generate implementation plan from recommendations
    pub fn generate_implementation_plan(
        primary: &[ArchitecturalRecommendation],
    ) -> Vec<(String, String, String)> {
        primary
            .iter()
            .map(|rec| {
                (
                    rec.title.clone(),
                    format!(
                        "Effort: {:?}, Risk: {:?}",
                        rec.implementation_effort, rec.risk_level
                    ),
                    format!("Benefits: {}", rec.expected_benefits.join(", ")),
                )
            })
            .collect()
    }

    /// Calculate risk mitigation effectiveness
    pub fn calculate_risk_mitigation_effectiveness(
        risk_assessment: &RiskAssessment,
        mitigation_strategies: &[String],
    ) -> f32 {
        let base_risk = risk_assessment.overall_risk;
        let mitigation_factor = mitigation_strategies.len() as f32 / 10.0; // Simple heuristic

        (base_risk * (1.0 - mitigation_factor)).clamp(0.0, base_risk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommendation_generator_creation() {
        let generator = RecommendationGenerator::new();
        assert!(!generator.recommendation_templates.is_empty());
    }

    #[test]
    fn test_priority_score_calculation() {
        let suggestion = ArchitecturalSuggestion {
            title: "Test suggestion".to_string(),
            description: "Test description".to_string(),
            category: SuggestionCategory::Refactoring,
            priority: PriorityLevel::High,
            impact: ImpactLevel::Major,
        };

        let score = recommendation_utils::calculate_priority_score(&suggestion);
        assert!(score > 0.5); // High priority + major impact should be > 0.5
    }

    #[test]
    fn test_mitigation_effectiveness() {
        let risk_assessment = RiskAssessment {
            overall_risk: 0.8,
            risk_factors: vec![],
            mitigation_strategies: vec![
                "Code review".to_string(),
                "Testing".to_string(),
                "Documentation".to_string(),
            ],
        };

        let effectiveness = recommendation_utils::calculate_risk_mitigation_effectiveness(
            &risk_assessment,
            &risk_assessment.mitigation_strategies,
        );

        assert!(effectiveness < 0.8); // Mitigation should reduce risk
    }

    #[tokio::test]
    async fn test_risk_assessment_generation() {
        let generator = RecommendationGenerator::new();
        let analysis = PatternAnalysis {
            detected_patterns: vec![],
            anti_patterns: vec![AntiPattern {
                anti_pattern_type: "God Object".to_string(),
                severity: 0.8,
                location: PatternLocation {
                    files: vec!["src/main.rs".to_string()],
                    modules: vec![],
                    lines: Some((1, 100)),
                },
                description: "Large class with too many responsibilities".to_string(),
                consequences: vec!["Difficult to maintain".to_string()],
                refactoring_suggestions: vec!["Apply SRP".to_string()],
            }],
            quality_metrics: QualityMetrics {
                maintainability_index: 75.0,
                cyclomatic_complexity: 25.0,
                halstead_complexity: 30.0,
                lines_of_code: 1000,
                technical_debt_ratio: 0.2,
                test_coverage: Some(0.7),
            },
            complexity_assessment: ComplexityAssessment {
                overall_complexity: ComplexityLevel::High,
                hotspot_complexity: vec![],
                complexity_trends: vec![],
            },
            coupling_analysis: CouplingAnalysis {
                afferent_coupling: std::collections::HashMap::new(),
                efferent_coupling: std::collections::HashMap::new(),
                instability: std::collections::HashMap::new(),
                abstractness: std::collections::HashMap::new(),
                distance_from_main: std::collections::HashMap::new(),
            },
            cohesion_analysis: CohesionAnalysis {
                lack_of_cohesion: std::collections::HashMap::new(),
                functional_cohesion: std::collections::HashMap::new(),
            },
        };

        let quality = QualityAssessment {
            overall_score: 0.7,
            maintainability_score: 0.6,
            complexity_score: 0.4,
            coupling_score: 0.8,
            cohesion_score: 0.5,
            grading_scale: "0.0-1.0".to_string(),
        };

        let risk_assessment = generator.assess_risks(&analysis, &quality).await.unwrap();
        assert!(risk_assessment.overall_risk > 0.0);
        assert!(!risk_assessment.risk_factors.is_empty());
    }
}

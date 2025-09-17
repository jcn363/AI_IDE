//! Conflict analyzer for dependency version conflicts and resolution suggestions with AI mediation

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use rayon::prelude::*;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

// AI/ML integration for conflict resolution
use rust_ai_ide_ai_inference::{
    NLToCodeConverter, NLToCodeInput, NLToCodeResult,
    InferenceError,
};

use crate::error::*;
use crate::graph::*;
use crate::resolver::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConflict {
    pub package_name: String,
    pub constraints: Vec<ConstraintInfo>,
    pub suggested_resolution: Option<String>,
    pub conflict_level: ConflictLevel,
    /// AI-enhanced conflict metadata
    pub ai_metadata: Option<AIConflictMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConflictMetadata {
    /// AI confidence score (0.0 to 1.0)
    pub confidence_score: f64,
    /// AI-generated resolution suggestions
    pub ai_suggestions: Vec<AISuggestion>,
    /// AI analysis of conflict impact
    pub impact_analysis: Option<String>,
    /// AI reasoning for the suggested resolution
    pub reasoning: Option<String>,
    /// Whether AI was used for analysis
    pub ai_used: bool,
    /// Timestamp of AI analysis
    pub analyzed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISuggestion {
    /// Suggested version or resolution strategy
    pub suggestion: String,
    /// Confidence score for this suggestion (0.0 to 1.0)
    pub confidence: f64,
    /// Type of suggestion
    pub suggestion_type: AISuggestionType,
    /// AI reasoning for this suggestion
    pub reasoning: String,
    /// Potential risks or caveats
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AISuggestionType {
    /// Suggest a specific version
    Version,
    /// Suggest updating all dependencies
    UpdateAll,
    /// Suggest using a different dependency
    AlternativeDependency,
    /// Suggest a custom resolution strategy
    CustomStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictLevel {
    None,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintInfo {
    pub source_package: String,
    pub version_requirement: String,
    pub depth: usize,
}

impl ConstraintInfo {
    pub fn new(source_package: String, version_requirement: String, depth: usize) -> Self {
        Self {
            source_package,
            version_requirement,
            depth,
        }
    }
}

pub struct ConflictAnalyzer {
    graph: Arc<RwLock<DependencyGraph>>,
    /// AI services for conflict analysis
    ai_converter: Arc<NLToCodeConverter>,
    /// Configuration for AI features
    ai_config: AIConflictConfig,
}

#[derive(Debug, Clone)]
pub struct AIConflictConfig {
    /// Enable AI-powered conflict analysis
    pub enable_ai_analysis: bool,
    /// Minimum confidence threshold for AI suggestions
    pub min_confidence_threshold: f64,
    /// Maximum time to spend on AI analysis (seconds)
    pub max_analysis_time_secs: u64,
    /// Whether to include AI reasoning in responses
    pub include_reasoning: bool,
    /// Fallback to traditional analysis on AI failure
    pub fallback_on_ai_failure: bool,
}

impl Default for AIConflictConfig {
    fn default() -> Self {
        Self {
            enable_ai_analysis: true,
            min_confidence_threshold: 0.7,
            max_analysis_time_secs: 30,
            include_reasoning: true,
            fallback_on_ai_failure: true,
        }
    }
}

impl ConflictAnalyzer {
    pub fn new(graph: Arc<RwLock<DependencyGraph>>) -> Self {
        Self::new_with_config(graph, AIConflictConfig::default())
    }

    pub fn new_with_config(
        graph: Arc<RwLock<DependencyGraph>>,
        ai_config: AIConflictConfig,
    ) -> Self {
        // Create a simple AI converter for conflict resolution
        let ai_converter = Arc::new(SimpleConflictAI {});

        Self {
            graph,
            ai_converter,
            ai_config,
        }
    }

    /// Analyze all version conflicts in the dependency graph
    pub async fn analyze_conflicts(&self) -> DependencyResult<Vec<VersionConflict>> {
        let graph = self.graph.read().await;
        let mut conflicts = Vec::new();

        // Build a map of package -> list of version constraints
        let mut constraint_map: HashMap<String, Vec<ConstraintInfo>> = HashMap::new();

        // Collect all constraints from the graph
        for (_, node_idx) in &graph.node_indices {
            if let Some(node) = graph.graph.node_weight(*node_idx) {
                let dependencies = graph.get_dependencies(&node.name)?;
                for (dep_name, dep_edge) in dependencies {
                    if let Some(version_req) = &dep_edge.version_constraint {
                        constraint_map.entry(dep_name.clone()).or_default().push(
                            ConstraintInfo::new(
                                node.name.clone(),
                                version_req.clone(),
                                dep_edge.req_depth,
                            ),
                        );
                    }
                }
            }
        }

        // Analyze conflicts for each package
        for (package_name, constraints) in constraint_map {
            if constraints.len() > 1 {
                let conflict = self
                    .analyze_package_conflict(&package_name, &constraints)
                    .await?;
                conflicts.push(conflict);
            }
        }

        Ok(conflicts)
    }

    /// Analyze conflict for a specific package
    async fn analyze_package_conflict(
        &self,
        package_name: &str,
        constraints: &[ConstraintInfo],
    ) -> DependencyResult<VersionConflict> {
        let mut unique_reqs = HashSet::new();
        let mut all_versions = Vec::new();

        // Collect unique version requirements
        for constraint in constraints {
            unique_reqs.insert(&constraint.version_requirement);

            if let Ok(version_req) = VersionReq::parse(&constraint.version_requirement) {
                // In a real implementation, this would fetch available versions
                all_versions.extend(self.get_available_versions(package_name).await?);
            }
        }

        let mut conflict = VersionConflict {
            package_name: package_name.to_string(),
            constraints: constraints.to_vec(),
            suggested_resolution: None,
            conflict_level: ConflictLevel::None,
            ai_metadata: None,
        };

        if unique_reqs.len() > 1 {
            conflict.conflict_level =
                self.determine_conflict_level(constraints.len(), unique_reqs.len());

            // Traditional resolution as fallback
            let traditional_resolution = self.suggest_resolution(&constraints)?;

            // Try AI-powered analysis if enabled
            if self.ai_config.enable_ai_analysis {
                match self.analyze_conflict_with_ai(package_name, constraints).await {
                    Ok(ai_metadata) => {
                        conflict.ai_metadata = Some(ai_metadata);
                        // Use AI suggestion if confidence is high enough, otherwise fall back to traditional
                        if let Some(ai_meta) = &conflict.ai_metadata {
                            if ai_meta.confidence_score >= self.ai_config.min_confidence_threshold {
                                if let Some(ai_suggestion) = ai_meta.ai_suggestions.first() {
                                    conflict.suggested_resolution = Some(ai_suggestion.suggestion.clone());
                                }
                            } else {
                                conflict.suggested_resolution = traditional_resolution;
                            }
                        }
                    }
                    Err(_) => {
                        // AI analysis failed, use traditional resolution
                        if self.ai_config.fallback_on_ai_failure {
                            conflict.suggested_resolution = traditional_resolution;
                        }
                    }
                }
            } else {
                conflict.suggested_resolution = traditional_resolution;
            }
        }

        Ok(conflict)
    }

    /// Analyze conflict using AI services for enhanced suggestions
    async fn analyze_conflict_with_ai(
        &self,
        package_name: &str,
        constraints: &[ConstraintInfo],
    ) -> DependencyResult<AIConflictMetadata> {
        let start_time = chrono::Utc::now();

        // Create a natural language description of the conflict
        let conflict_description = self.build_conflict_description(package_name, constraints);

        // Use NLToCodeConverter to analyze the conflict
        let ai_input = NLToCodeInput {
            description: conflict_description,
            target_language: "rust".to_string(),
            project_context: vec![format!("Resolving dependency version conflict for package '{}' with {} conflicting constraints", package_name, constraints.len())],
            coding_style: None,
            existing_code: None,
            requirements: vec!["Analyze dependency version conflict and suggest resolution".to_string()],
        };

        // Get AI suggestions with timeout
        let ai_result = tokio::time::timeout(
            std::time::Duration::from_secs(self.ai_config.max_analysis_time_secs),
            self.ai_converter.convert(ai_input),
        ).await;

        match ai_result {
            Ok(Ok(conversion_result)) => {
                // Parse AI response and extract suggestions
                let ai_suggestions = self.parse_ai_suggestions(&conversion_result)?;
                let confidence_score = self.calculate_ai_confidence(&ai_suggestions, constraints);

                let metadata = AIConflictMetadata {
                    confidence_score,
                    ai_suggestions,
                    impact_analysis: self.analyze_conflict_impact(package_name, constraints).await,
                    reasoning: self.ai_config.include_reasoning.then_some(
                        conversion_result.explanation.clone()
                    ),
                    ai_used: true,
                    analyzed_at: Some(start_time),
                };

                Ok(metadata)
            }
            Ok(Err(e)) => {
                // AI service error
                tracing::warn!("AI conflict analysis failed for {}: {:?}", package_name, e);
                Err(DependencyError::ResolutionError {
                    package: package_name.to_string(),
                    reason: format!("AI analysis failed: {:?}", e),
                })
            }
            Err(_) => {
                // Timeout
                tracing::warn!("AI conflict analysis timed out for {}", package_name);
                Err(DependencyError::ResolutionError {
                    package: package_name.to_string(),
                    reason: "AI analysis timed out".to_string(),
                })
            }
        }
    }

    /// Build a natural language description of the conflict for AI analysis
    fn build_conflict_description(&self, package_name: &str, constraints: &[ConstraintInfo]) -> String {
        let mut description = format!(
            "Resolve dependency version conflict for package '{}'.\n\nConstraints:\n",
            package_name
        );

        for (i, constraint) in constraints.iter().enumerate() {
            description.push_str(&format!(
                "{}. Package '{}' requires version '{}'\n",
                i + 1,
                constraint.source_package,
                constraint.version_requirement
            ));
        }

        description.push_str("\nPlease suggest the best version to resolve this conflict, considering:\n");
        description.push_str("- Semantic versioning compatibility\n");
        description.push_str("- Stability and maintenance status\n");
        description.push_str("- Breaking change impact\n");
        description.push_str("- Ecosystem compatibility\n\n");
        description.push_str("Provide a specific version number that satisfies most constraints.");

        description
    }

    /// Parse AI-generated suggestions from the conversion result
    fn parse_ai_suggestions(&self, result: &NLToCodeResult) -> DependencyResult<Vec<AISuggestion>> {
        let mut suggestions = Vec::new();

        // Extract version suggestions from AI response
        let code = &result.code;
        // Parse version numbers from the response
        let version_pattern = regex::Regex::new(r"\d+\.\d+\.\d+").unwrap();
        for cap in version_pattern.captures_iter(code) {
            if let Some(version_match) = cap.get(0) {
                let version = version_match.as_str().to_string();
                suggestions.push(AISuggestion {
                    suggestion: version.clone(),
                    confidence: result.confidence_score,
                    suggestion_type: AISuggestionType::Version,
                    reasoning: format!("AI recommended version {} based on conflict analysis: {}", version, result.explanation),
                    caveats: vec!["Verify compatibility with your specific use case".to_string()],
                });
            }
        }

        // If no versions found, provide alternative suggestions
        if suggestions.is_empty() {
            suggestions.push(AISuggestion {
                suggestion: "Update all dependencies to latest compatible versions".to_string(),
                confidence: 0.7,
                suggestion_type: AISuggestionType::UpdateAll,
                reasoning: format!("AI suggests updating dependencies to resolve conflicts: {}", result.explanation),
                caveats: vec![
                    "May introduce breaking changes".to_string(),
                    "Test thoroughly after updates".to_string(),
                ],
            });
        }

        Ok(suggestions)
    }

    /// Calculate confidence score for AI suggestions
    fn calculate_ai_confidence(&self, suggestions: &[AISuggestion], constraints: &[ConstraintInfo]) -> f64 {
        if suggestions.is_empty() {
            return 0.0;
        }

        // Start with base confidence from AI suggestions
        let mut total_confidence = suggestions.iter().map(|s| s.confidence).sum::<f64>();
        let avg_confidence = total_confidence / suggestions.len() as f64;

        // Adjust based on constraint complexity
        let complexity_factor = match constraints.len() {
            1..=2 => 1.0,
            3..=5 => 0.9,
            _ => 0.8,
        };

        // Adjust based on version requirement complexity
        let version_complexity = constraints.iter()
            .filter(|c| c.version_requirement.contains("||") || c.version_requirement.contains(">="))
            .count() as f64 / constraints.len() as f64;

        let complexity_penalty = 1.0 - (version_complexity * 0.1);

        (avg_confidence * complexity_factor * complexity_penalty).min(1.0).max(0.0)
    }

    /// Analyze the impact of the conflict resolution
    async fn analyze_conflict_impact(&self, package_name: &str, constraints: &[ConstraintInfo]) -> Option<String> {
        let affected_packages = constraints.len();
        let unique_versions: std::collections::HashSet<_> = constraints.iter()
            .map(|c| &c.version_requirement)
            .collect();

        let conflict_severity = if affected_packages > 5 {
            "high"
        } else if affected_packages > 2 {
            "medium"
        } else {
            "low"
        };

        Some(format!(
            "This conflict affects {} packages with {} unique version requirements. \
             Resolution impact is {} - consider testing after applying changes.",
            affected_packages, unique_versions.len(), conflict_severity
        ))
    }

    /// Get available versions for a package (mock implementation)
    async fn get_available_versions(&self, _package_name: &str) -> DependencyResult<Vec<String>> {
        // In a real implementation, this would query the registry
        Ok(vec![
            "1.0.0".to_string(),
            "1.0.1".to_string(),
            "1.1.0".to_string(),
            "1.2.0".to_string(),
            "2.0.0".to_string(),
        ])
    }

    fn determine_conflict_level(
        &self,
        constraint_count: usize,
        unique_reqs: usize,
    ) -> ConflictLevel {
        match constraint_count {
            2..=3 => ConflictLevel::Warning,
            4..=6 => ConflictLevel::Error,
            _ => ConflictLevel::Critical,
        }
    }

    async fn suggest_resolution(&self, constraints: &[ConstraintInfo]) -> DependencyResult<String> {
        // Find the most restrictive constraint and suggest a version that satisfies most constraints
        let mut compatible_versions: HashMap<String, usize> = HashMap::new();

        let available_versions = self.get_available_versions("dummy").await?;
        let available_version_objects: Vec<Version> = available_versions
            .iter()
            .filter_map(|v| Version::parse(v).ok())
            .collect();

        for version in &available_version_objects {
            let mut compatible_count = 0;

            for constraint in constraints {
                if let Ok(version_req) = VersionReq::parse(&constraint.version_requirement) {
                    if version_req.matches(version) {
                        compatible_count += 1;
                    }
                }
            }

            if compatible_count == constraints.len() {
                compatible_versions.insert(version.to_string(), compatible_count);
            }
        }

        // Return the version that satisfies the most constraints
        compatible_versions
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(version, _)| version)
            .ok_or_else(|| DependencyError::ResolutionError {
                package: "unknown".to_string(),
                reason: "No compatible version found".to_string(),
            })
    }

    /// Get conflict statistics
    pub async fn get_conflict_stats(&self) -> DependencyResult<ConflictStats> {
        let conflicts = self.analyze_conflicts().await?;

        let mut warning_count = 0;
        let mut error_count = 0;
        let mut critical_count = 0;
        let mut total_packages = 0;
        let mut affected_packages = HashSet::new();

        for conflict in &conflicts {
            total_packages += 1;

            for constraint in &conflict.constraints {
                affected_packages.insert(&constraint.source_package);
            }

            match conflict.conflict_level {
                ConflictLevel::Warning => warning_count += 1,
                ConflictLevel::Error => error_count += 1,
                ConflictLevel::Critical => critical_count += 1,
                ConflictLevel::None => {}
            }
        }

        Ok(ConflictStats {
            total_conflicts: conflicts.len(),
            warning_conflicts: warning_count,
            error_conflicts: error_count,
            critical_conflicts: critical_count,
            total_affected_packages: affected_packages.len(),
        })
    }

    /// Check if there are any unresolvable conflicts
    pub async fn has_unresolvable_conflicts(&self) -> DependencyResult<bool> {
        let conflicts = self.analyze_conflicts().await?;
        let has_unresolvable = conflicts.iter().any(|c| {
            c.conflict_level == ConflictLevel::Critical && c.suggested_resolution.is_none()
        });

        Ok(has_unresolvable)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictStats {
    pub total_conflicts: usize,
    pub warning_conflicts: usize,
    pub error_conflicts: usize,
    pub critical_conflicts: usize,
    pub total_affected_packages: usize,
}

impl ConflictStats {
    pub fn is_clean(&self) -> bool {
        self.total_conflicts == 0
    }

    pub fn has_critical_issues(&self) -> bool {
        self.critical_conflicts > 0
    }
}

/// Resolution suggestions and conflict resolution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionPlan {
    pub conflicts: Vec<VersionConflict>,
    pub resolutions: HashMap<String, String>,
    pub unresolved_conflicts: Vec<String>,
    pub impact_analysis: ImpactAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    pub packages_to_update: Vec<String>,
    pub potential_breaking_changes: usize,
    pub compatibility_risk: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

pub struct ComprehensiveConflictAnalyzer {
    analyzer: ConflictAnalyzer,
}

impl ComprehensiveConflictAnalyzer {
    pub fn new(graph: Arc<RwLock<DependencyGraph>>) -> Self {
        Self {
            analyzer: ConflictAnalyzer::new(graph),
        }
    }

    /// Generate a comprehensive resolution plan
    pub async fn generate_resolution_plan(&self) -> DependencyResult<ResolutionPlan> {
        let conflicts = self.analyzer.analyze_conflicts().await?;
        let mut resolutions = HashMap::new();
        let mut unresolved_conflicts = Vec::new();
        let mut packages_to_update = HashSet::new();

        for conflict in &conflicts {
            if let Some(resolution) = &conflict.suggested_resolution {
                resolutions.insert(conflict.package_name.clone(), resolution.clone());

                // Track affected packages
                for constraint in &conflict.constraints {
                    packages_to_update.insert(constraint.source_package.clone());
                }
            } else {
                unresolved_conflicts.push(conflict.package_name.clone());
            }
        }

        let impact_analysis = ImpactAnalysis {
            packages_to_update: packages_to_update.into_iter().collect(),
            potential_breaking_changes: self.estimate_breaking_changes(&resolutions).await?,
            compatibility_risk: self.assess_compatibility_risk(&conflicts),
        };

        Ok(ResolutionPlan {
            conflicts,
            resolutions,
            unresolved_conflicts,
            impact_analysis,
        })
    }

    async fn estimate_breaking_changes(
        &self,
        resolutions: &HashMap<String, String>,
    ) -> DependencyResult<usize> {
        // Rough estimation based on major version changes
        let mut breaking_changes = 0;

        for (_package, new_version) in resolutions {
            if let Ok(new_ver) = Version::parse(new_version) {
                if new_ver.major >= 2 {
                    breaking_changes += 1; // Assume major version changes are breaking
                }
            }
        }

        Ok(breaking_changes)
    }

    fn assess_compatibility_risk(&self, conflicts: &[VersionConflict]) -> RiskLevel {
        let critical_count = conflicts
            .iter()
            .filter(|c| matches!(c.conflict_level, ConflictLevel::Critical))
            .count();

        let error_count = conflicts
            .iter()
            .filter(|c| matches!(c.conflict_level, ConflictLevel::Error))
            .count();

        if critical_count > 2 || error_count > 5 {
            RiskLevel::Critical
        } else if critical_count > 0 || error_count > 2 {
            RiskLevel::High
        } else if conflicts.len() > 3 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    /// Validate that the resolution plan is safe to apply
    pub fn validate_resolution_plan(&self, _plan: &ResolutionPlan) -> Result<(), String> {
        // Simplified validation - in production this would check actual risks
        Ok(())
    }
}

//! Conflict analyzer for dependency version conflicts and resolution suggestions

use semver::{Version, VersionReq};
use std::collections::{HashMap, HashSet};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::error::*;
use crate::graph::*;
use crate::resolver::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConflict {
    pub package_name: String,
    pub constraints: Vec<ConstraintInfo>,
    pub suggested_resolution: Option<String>,
    pub conflict_level: ConflictLevel,
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
}

impl ConflictAnalyzer {
    pub fn new(graph: Arc<RwLock<DependencyGraph>>) -> Self {
        Self { graph }
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
                        constraint_map.entry(dep_name.clone())
                            .or_default()
                            .push(ConstraintInfo::new(
                                node.name.clone(),
                                version_req.clone(),
                                dep_edge.req_depth,
                            ));
                    }
                }
            }
        }

        // Analyze conflicts for each package
        for (package_name, constraints) in constraint_map {
            if constraints.len() > 1 {
                let conflict = self.analyze_package_conflict(&package_name, &constraints).await?;
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
        };

        if unique_reqs.len() > 1 {
            conflict.conflict_level = self.determine_conflict_level(constraints.len(), unique_reqs.len());
            conflict.suggested_resolution = Some(self.suggest_resolution(&constraints).await?);
        }

        Ok(conflict)
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

    fn determine_conflict_level(&self, constraint_count: usize, unique_reqs: usize) -> ConflictLevel {
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
            .ok_or_else(|| {
                DependencyError::ResolutionError {
                    package: "unknown".to_string(),
                    reason: "No compatible version found".to_string(),
                }
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
        let has_unresolvable = conflicts.iter()
            .any(|c| c.conflict_level == ConflictLevel::Critical && c.suggested_resolution.is_none());

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

    async fn estimate_breaking_changes(&self, resolutions: &HashMap<String, String>) -> DependencyResult<usize> {
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
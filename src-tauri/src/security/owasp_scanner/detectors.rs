//! OWASP Top 10 Implementation - Detectors Module
//!
//! This module contains specific detectors for each OWASP Top 10 vulnerability category.

use std::path::Path;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::security::*;
use crate::security::SecurityCategory;
use regex::Regex;
use walkdir::WalkDir;

// Re-export parent types
pub use super::{OWASPCategory, ExploitabilityScore, ImpactScore, OWASPVulnerability, AttackVector, AttackComplexity, PrivilegesRequired, UserInteraction, Scope, ConfidentialityImpact, IntegrityImpact, AvailabilityImpact};

// Re-export trait
pub use self::r#trait::{OWASPDetector, DetectionResult};

// Detection result from individual detectors
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub security_issue: SecurityIssue,
    pub exploitability: ExploitabilityScore,
    pub impact: ImpactScore,
    pub ai_confidence: f32,
    pub patterns_detected: Vec<String>,
    pub remediation_steps: Vec<String>,
}

// Trait for OWASP detectors
#[derive(Debug, Clone)]
pub struct OWASPDetector {
    pub category: OWASPCategory,
    pub name: String,
    pub patterns: Vec<SecurityRule>,
    pub ai_enhanced: bool,
}

impl OWASPDetector {
    async fn analyze_codebase(&self, workspace_path: &Path) -> Result<Vec<DetectionResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        let file_extension_patterns = [".rs", ".toml", ".toml"];

        for entry in WalkDir::new(workspace_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file() && file_extension_patterns.iter().any(|ext| e.path().extension().map_or(false, |fext| fext == &ext[1..])))
        {
            if let Ok(code) = std::fs::read_to_string(entry.path()) {
                let file_path = entry.path().to_string_lossy().to_string();
                results.extend(self.analyze_file(&code, &file_path));
            }
        }

        Ok(results)
    }

    fn analyze_file(&self, code: &str, file_path: &str) -> Vec<DetectionResult> {
        let mut results = Vec::new();

        for rule in &self.patterns {
            for mat in rule.pattern.find_iter(code) {
                let line_number = code[..mat.start()].lines().count();

                let exploitability = self.calculate_exploitability(&rule.category);
                let impact = self.calculate_impact(&rule.severity);

                results.push(DetectionResult {
                    security_issue: SecurityIssue {
                        category: rule.category.clone(),
                        severity: rule.severity.clone(),
                        title: rule.name.clone(),
                        description: rule.description.clone(),
                        file_path: file_path.to_string(),
                        line_number: Some(line_number),
                        column: Some(mat.start()),
                        code_snippet: Some(mat.as_str().to_string()),
                        remediation: rule.remediation.clone(),
                        confidence: 0.8,
                        cwe_id: rule.cwe_id,
                    },
                    exploitability,
                    impact,
                    ai_confidence: 0.75, // Would be calculated by AI
                    patterns_detected: vec![rule.name.clone()],
                    remediation_steps: self.generate_remediation_steps(&rule),
                });
            }
        }

        results
    }

    fn calculate_exploitability(&self, category: &SecurityCategory) -> ExploitabilityScore {
        match category {
            SecurityCategory::CommandInjection | SecurityCategory::SqlInjection => {
                ExploitabilityScore {
                    attack_vector: AttackVector::Network,
                    attack_complexity: AttackComplexity::Low,
                    privileges_required: PrivilegesRequired::None,
                    user_interaction: UserInteraction::None,
                    scope: Scope::Unchanged,
                }
            },
            SecurityCategory::HardcodedSecrets => {
                ExploitabilityScore {
                    attack_vector: AttackVector::Local,
                    attack_complexity: AttackComplexity::Low,
                    privileges_required: PrivilegesRequired::Low,
                    user_interaction: UserInteraction::None,
                    scope: Scope::Unchanged,
                }
            },
            _ => ExploitabilityScore {
                attack_vector: AttackVector::Local,
                attack_complexity: AttackComplexity::High,
                privileges_required: PrivilegesRequired::None,
                user_interaction: UserInteraction::None,
                scope: Scope::Unchanged,
            }
        }
    }

    fn calculate_impact(&self, severity: &SecuritySeverity) -> ImpactScore {
        match severity {
            SecuritySeverity::Critical => ImpactScore {
                confidentiality: ConfidentialityImpact::High,
                integrity: IntegrityImpact::High,
                availability: AvailabilityImpact::High,
            },
            SecuritySeverity::High => ImpactScore {
                confidentiality: ConfidentialityImpact::High,
                integrity: IntegrityImpact::High,
                availability: AvailabilityImpact::Low,
            },
            SecuritySeverity::Medium => ImpactScore {
                confidentiality: ConfidentialityImpact::Low,
                integrity: IntegrityImpact::Low,
                availability: AvailabilityImpact::Low,
            },
            _ => ImpactScore {
                confidentiality: ConfidentialityImpact::None,
                integrity: IntegrityImpact::None,
                availability: AvailabilityImpact::None,
            }
        }
    }

    fn generate_remediation_steps(&self, rule: &SecurityRule) -> Vec<String> {
        vec![
            rule.remediation.clone(),
            format!("Implement comprehensive security testing for {}", rule.name),
            "Add security logging and monitoring for this vulnerability type".to_string(),
            "Conduct security code review focusing on this area".to_string(),
        ]
    }
}

// Implementation moved to separate modules for better organization
pub mod broken_access_control;
pub mod cryptographic_failures;
pub mod injection;
pub mod insecure_design;
pub mod security_misconfiguration;
pub mod vulnerable_components;
pub mod identification_authentication;
pub mod software_data_integrity;
pub mod logging_failures;
pub mod ssrf;

pub use broken_access_control::*;
pub use cryptographic_failures::*;
pub use injection::*;
pub use insecure_design::*;
pub use security_misconfiguration::*;
pub use vulnerable_components::*;
pub use identification_authentication::*;
pub use software_data_integrity::*;
pub use logging_failures::*;
pub use ssrf::*;
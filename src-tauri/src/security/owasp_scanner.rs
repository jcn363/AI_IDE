//! OWASP Top 10 Security Scanner
//!
//! Advanced security analysis system implementing OWASP Top 10 vulnerability detection
//! with AI-enhanced pattern recognition and comprehensive supply chain analysis.

pub mod detectors;
pub mod supply_chain;
pub mod ai_enhancements;

use std::path::Path;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::security::*;
use detectors::*;

// Re-export key components
pub use detectors::{OWASPDetector, DetectionResult, VulnerabilityFinding};
pub use supply_chain::{SupplyChainScanner, DependencyAnalysis, LicenseCompliance};
pub use ai_enhancements::{AIEnhancedAnalyzer, SecurityPattern};

// OWASP Top 10 Categories according to 2021 standard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OWASPCategory {
    A01_2021_BrokenAccessControl,
    A02_2021_CryptographicFailures,
    A03_2021_Injection,
    A04_2021_InsecureDesign,
    A05_2021_SecurityMisconfiguration,
    A06_2021_VulnerableOutdatedComponents,
    A07_2021_IDAuthenticationFailures,
    A08_2021_SoftwareDataIntegrityFailures,
    A09_2021_SecurityLoggingFailures,
    A10_2021_ServerSideRequestForgery,
}

// Enhanced vulnerability finding with OWASP context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OWASPVulnerability {
    pub owasp_category: OWASPCategory,
    pub security_issue: SecurityIssue,
    pub exploitability: ExploitabilityScore,
    pub impact: ImpactScore,
    pub risk_score: f32,
    pub ai_confidence: f32,
    pub patterns_detected: Vec<String>,
    pub remediation_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploitabilityScore {
    pub attack_vector: AttackVector,
    pub attack_complexity: AttackComplexity,
    pub privileges_required: PrivilegesRequired,
    pub user_interaction: UserInteraction,
    pub scope: Scope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackVector {
    Network, AdjacentNetwork, Local, Physical
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackComplexity {
    Low, High
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivilegesRequired {
    None, Low, High
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserInteraction {
    None, Required
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Scope {
    Unchanged, Changed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactScore {
    pub confidentiality: ConfidentialityImpact,
    pub integrity: IntegrityImpact,
    pub availability: AvailabilityImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfidentialityImpact {
    None, Low, High
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrityImpact {
    None, Low, High
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AvailabilityImpact {
    None, Low, High
}

// Main OWASP scanner that orchestrates all detection
pub struct OWASPScanner {
    detectors: HashMap<OWASPCategory, Box<dyn OWASPDetector>>,
    supply_chain_scanner: SupplyChainScanner,
    ai_analyzer: AIEnhancedAnalyzer,
}

impl OWASPScanner {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut detectors = HashMap::new();

        // Initialize all OWASP Top 10 detectors
        detectors.insert(
            OWASPCategory::A01_2021_BrokenAccessControl,
            Box::new(BrokenAccessControlDetector::new()) as Box<dyn OWASPDetector>
        );

        detectors.insert(
            OWASPCategory::A02_2021_CryptographicFailures,
            Box::new(CryptographicFailuresDetector::new()) as Box<dyn OWASPDetector>
        );

        detectors.insert(
            OWASPCategory::A03_2021_Injection,
            Box::new(InjectionDetector::new()) as Box<dyn OWASPDetector>
        );

        detectors.insert(
            OWASPCategory::A04_2021_InsecureDesign,
            Box::new(InsecureDesignDetector::new()) as Box<dyn OWASPDetector>
        );

        detectors.insert(
            OWASPCategory::A05_2021_SecurityMisconfiguration,
            Box::new(SecurityMisconfigurationDetector::new()) as Box<dyn OWASPDetector>
        );

        detectors.insert(
            OWASPCategory::A06_2021_VulnerableOutdatedComponents,
            Box::new(VulnerableComponentsDetector::new()) as Box<dyn OWASPDetector>
        );

        detectors.insert(
            OWASPCategory::A07_2021_IDAuthenticationFailures,
            Box::new(IdentificationAuthenticationDetector::new()) as Box<dyn OWASPDetector>
        );

        detectors.insert(
            OWASPCategory::A08_2021_SoftwareDataIntegrityFailures,
            Box::new(SoftwareDataIntegrityDetector::new()) as Box<dyn OWASPDetector>
        );

        detectors.insert(
            OWASPCategory::A09_2021_SecurityLoggingFailures,
            Box::new(LoggingFailuresDetector::new()) as Box<dyn OWASPDetector>
        );

        detectors.insert(
            OWASPCategory::A10_2021_ServerSideRequestForgery,
            Box::new(SSRFDetector::new()) as Box<dyn OWASPDetector>
        );

        let supply_chain_scanner = SupplyChainScanner::new().await?;
        let ai_analyzer = AIEnhancedAnalyzer::new().await?;

        Ok(Self {
            detectors,
            supply_chain_scanner,
            ai_analyzer,
        })
    }

    /// Comprehensive OWASP Top 10 analysis of a codebase
    pub async fn analyze_codebase(&self, workspace_path: &Path) -> Result<OWASPScanResult, Box<dyn std::error::Error>> {
        let mut vulnerabilities = Vec::new();
        let mut ai_insights = Vec::new();

        // Run all OWASP Top 10 detectors concurrently
        let detector_futures = self.detectors.iter().map(|(category, detector)| {
            let cat = category.clone();
            async move {
                match detector.analyze_codebase(workspace_path).await {
                    Ok(results) => (cat, results),
                    Err(e) => {
                        eprintln!("Detector error for {:?}: {}", cat, e);
                        (cat, Vec::new())
                    }
                }
            }
        });

        let detector_results = futures::future::join_all(detector_futures).await;

        // Process detector results
        for (category, findings) in detector_results {
            for finding in findings {
                let vulnerability = OWASPVulnerability {
                    owasp_category: category.clone(),
                    security_issue: finding.security_issue,
                    exploitability: finding.exploitability,
                    impact: finding.impact,
                    risk_score: self.calculate_risk_score(&finding.exploitability, &finding.impact),
                    ai_confidence: finding.ai_confidence,
                    patterns_detected: finding.patterns_detected,
                    remediation_steps: finding.remediation_steps,
                };
                vulnerabilities.push(vulnerability);
            }
        }

        // Supply chain analysis
        let supply_chain_report = self.supply_chain_scanner.analyze_dependencies(workspace_path).await?;
        ai_insights.extend(self.ai_analyzer.generate_insights(&vulnerabilities, &supply_chain_report).await?);

        // Calculate OWASP-specific summary
        let owasp_summary = self.calculate_owasp_summary(&vulnerabilities);

        Ok(OWASPScanResult {
            vulnerabilities,
            supply_chain_report,
            ai_insights,
            summary: owasp_summary,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Calculate comprehensive risk score based on CVSS-like scoring
    fn calculate_risk_score(&self, exploitability: &ExploitabilityScore, impact: &ImpactScore) -> f32 {
        let exploitability_weight = match exploitability.attack_vector {
            AttackVector::Network => 0.85,
            AttackVector::AdjacentNetwork => 0.62,
            AttackVector::Local => 0.55,
            AttackVector::Physical => 0.2,
        } * match exploitability.attack_complexity {
            AttackComplexity::Low => 0.77,
            AttackComplexity::High => 0.44,
        } * match exploitability.privileges_required {
            PrivilegesRequired::None => 0.85,
            PrivilegesRequired::Low => 0.62,
            PrivilegesRequired::High => 0.27,
        } * match exploitability.user_interaction {
            UserInteraction::None => 0.85,
            UserInteraction::Required => 0.62,
        };

        let impact_weight = match impact.confidentiality {
            ConfidentialityImpact::None => 0.0,
            ConfidentialityImpact::Low => 0.22,
            ConfidentialityImpact::High => 0.56,
        } + match impact.integrity {
            IntegrityImpact::None => 0.0,
            IntegrityImpact::Low => 0.22,
            IntegrityImpact::High => 0.56,
        } + match impact.availability {
            AvailabilityImpact::None => 0.0,
            AvailabilityImpact::Low => 0.22,
            AvailabilityImpact::High => 0.56,
        };

        impact_weight / match exploitability.scope {
            Scope::Unchanged => 1.0,
            Scope::Changed => 1.08,
        } + (7.52 * (impact_weight - 0.029)) - 3.25 * (impact_weight - 0.02).powf(15.0)
    }

    fn calculate_owasp_summary(&self, vulnerabilities: &[OWASPVulnerability]) -> OWASPSummary {
        let mut category_counts = HashMap::new();

        for vuln in vulnerabilities {
            *category_counts.entry(vuln.owasp_category.clone()).or_insert(0) += 1;
        }

        OWASPSummary {
            total_vulnerabilities: vulnerabilities.len(),
            category_breakdown: category_counts,
            critical_vulnerabilities: vulnerabilities.iter().filter(|v| v.risk_score > 8.0).count(),
            high_vulnerabilities: vulnerabilities.iter().filter(|v| v.risk_score > 6.0 && v.risk_score <= 8.0).count(),
            medium_vulnerabilities: vulnerabilities.iter().filter(|v| v.risk_score > 4.0 && v.risk_score <= 6.0).count(),
            low_vulnerabilities: vulnerabilities.iter().filter(|v| v.risk_score <= 4.0).count(),
            average_risk_score: if vulnerabilities.is_empty() {
                0.0
            } else {
                vulnerabilities.iter().map(|v| v.risk_score).sum::<f32>() / vulnerabilities.len() as f32
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OWASPScanResult {
    pub vulnerabilities: Vec<OWASPVulnerability>,
    pub supply_chain_report: DependencyAnalysis,
    pub ai_insights: Vec<SecurityPattern>,
    pub summary: OWASPSummary,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OWASPSummary {
    pub total_vulnerabilities: usize,
    pub category_breakdown: HashMap<OWASPCategory, usize>,
    pub critical_vulnerabilities: usize,
    pub high_vulnerabilities: usize,
    pub medium_vulnerabilities: usize,
    pub low_vulnerabilities: usize,
    pub average_risk_score: f32,
}
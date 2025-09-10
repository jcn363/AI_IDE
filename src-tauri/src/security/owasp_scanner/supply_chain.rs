//! Supply Chain Security Analysis Framework
//!
//! Comprehensive supply chain analysis integrating with existing dependency
//! vulnerability scanners, license compliance, and SBOM generation.

use std::path::Path;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use tokio::sync::RwLock;
use moka::future::Cache;
use crate::dependency::models::{DependencyGraph, LicenseInfo};
use crate::rustsec_integration::RustsecScanner;
use crate::license::compliance_checker::LicenseComplianceChecker;

// Supply chain analysis types and enums
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupplyChainRiskLevel {
    Critical,
    High,
    Medium,
    Low,
    Negligible,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyChainVulnerability {
    pub dependency_name: String,
    pub version: String,
    pub vulnerability_id: String,
    pub severity: SupplyChainRiskLevel,
    pub cve_id: Option<String>,
    pub cvss_score: Option<f32>,
    pub published_date: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub description: String,
    pub fix_available: bool,
    pub remediation_available: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaliciousPackageIndicator {
    pub dependency_name: String,
    pub risk_factors: Vec<MaliciousIndicator>,
    pub confidence_score: f32,
    pub detection_methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaliciousIndicator {
    UnknownDeveloper,
    Typosquatting,
    OverambitiousScope,
    CodeInjection,
    ZeroDayVulnerability,
    StaleRepository,
    LackOfDocumentation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    pub vulnerabilities: Vec<SupplyChainVulnerability>,
    pub malicious_packages: Vec<MaliciousPackageIndicator>,
    pub license_issues: Vec<LicenseComplianceIssue>,
    pub sbom: Option<SoftwareBillOfMaterials>,
    pub dependency_chain_risk_score: f32,
    pub supply_chain_health_score: f32,
    pub recommendations: Vec<SupplyChainRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareBillOfMaterials {
    pub dependencies: Vec<SBOMDependency>,
    pub licenses: HashMap<String, String>, // dependency -> license_hash
    pub checksums: HashMap<String, String>, // dependency -> sha256
    pub provenance: HashMap<String, ProvenanceData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SBOMDependency {
    pub name: String,
    pub version: String,
    pub registry: String,
    pub hashes: HashMap<String, String>, // algorithm -> hash
    pub licenses: Vec<String>,
    pub authors: Vec<String>,
    pub repository: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceData {
    pub creator: String,
    pub created: DateTime<Utc>,
    pub verified: bool,
    pub build_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseComplianceIssue {
    pub dependency_name: String,
    pub problematic_license: String,
    pub compliance_category: LicenseComplianceCategory,
    pub risk_level: SupplyChainRiskLevel,
    pub justification_required: bool,
    pub remediation_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LicenseComplianceCategory {
    BannedLicense,
    CopyLeftLicense,
    CommercialRestriction,
    AttributionRequirement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyChainRecommendation {
    pub priority: u32, // 1-5, 5 being highest
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub estimated_effort: EffortLevel,
    pub impact_score: f32, // 0.0-10.0
    pub dependency_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    SecurityUpdate,
    LicenseChange,
    DependencyReplacement,
    MonitoringSetup,
    ProcessImprovement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Minimal,
    Low,
    Medium,
    High,
    VeryHigh,
}

// Main supply chain scanner implementation
pub struct SupplyChainScanner {
    rustsec_scanner: RwLock<RustsecScanner>,
    license_checker: LicenseComplianceChecker,
    vulnerability_cache: Cache<String, Vec<SupplyChainVulnerability>>,
    reputation_cache: Cache<String, Option<MaliciousPackageIndicator>>,
}

impl SupplyChainScanner {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let rustsec_scanner = RwLock::new(RustsecScanner::new()?);
        let license_checker = LicenseComplianceChecker::new()?;

        // Initialize caches with appropriate TTL
        let vulnerability_cache = Cache::builder()
            .time_to_live(std::time::Duration::from_secs(60 * 60)) // 1 hour
            .build();

        let reputation_cache = Cache::builder()
            .time_to_live(std::time::Duration::from_secs(60 * 60 * 24)) // 24 hours
            .build();

        Ok(Self {
            rustsec_scanner,
            license_checker,
            vulnerability_cache,
            reputation_cache,
        })
    }

    /// Comprehensive supply chain analysis of a workspace
    pub async fn analyze_dependencies(&self, workspace_path: &Path) -> Result<DependencyAnalysis, Box<dyn std::error::Error>> {
        let manifest_path = workspace_path.join("Cargo.toml");

        if !manifest_path.exists() {
            return Err("Cargo.toml not found in workspace".into());
        }

        // Run analyses concurrently
        let (vulnerabilities, malicious_packages, license_issues, sbom) = tokio::try_join!(
            self.scan_vulnerabilities(&manifest_path),
            self.scan_malicious_packages(&manifest_path),
            self.check_license_compliance(workspace_path),
            self.generate_sbom(&manifest_path)
        )?;

        // Calculate security metrics
        let dependency_chain_risk_score = self.calculate_dependency_risk_score(&vulnerabilities, &malicious_packages);
        let supply_chain_health_score = self.calculate_supply_chain_health_score(
            &vulnerabilities, &malicious_packages, &license_issues
        );

        // Generate recommendations
        let recommendations = self.generate_recommendations(
            &vulnerabilities, &malicious_packages, &license_issues
        );

        Ok(DependencyAnalysis {
            vulnerabilities,
            malicious_packages,
            license_issues,
            sbom: Some(sbom),
            dependency_chain_risk_score,
            supply_chain_health_score,
            recommendations,
        })
    }

    /// Scan for vulnerabilities using multiple sources
    async fn scan_vulnerabilities(&self, manifest_path: &Path) -> Result<Vec<SupplyChainVulnerability>, Box<dyn std::error::Error>> {
        let cache_key = format!("vuls_{}", manifest_path.display());
        if let Some(cached) = self.vulnerability_cache.get(&cache_key).await {
            return Ok(cached);
        }

        let scanner = self.rustsec_scanner.read().await;
        let lockfile_path = manifest_path.with_file_name("Cargo.lock");

        let mut vulnerabilities = Vec::new();

        // Scan using RustSec
        if lockfile_path.exists() {
            if let Ok(reports) = scanner.scan_lockfile(&lockfile_path) {
                for report in reports {
                    vulnerabilities.push(SupplyChainVulnerability {
                        dependency_name: report.package,
                        version: report.version,
                        vulnerability_id: report.advisory_id,
                        cve_id: None, // Could be extracted from advisory
                        cvss_score: report.cvss,
                        severity: self.map_rustsec_severity(&report.severity),
                        published_date: Utc::now(), // Placeholder - should come from advisory
                        last_modified: Utc::now(), // Placeholder
                        description: report.description,
                        fix_available: self.check_fix_available(&report),
                        remediation_available: None,
                    });
                }
            }
        }

        // Additional vulnerability sources could be integrated here
        // - GitHub Security Advisories
        // - NVD database
        // - OpenSSF Scorecard

        self.vulnerability_cache.insert(cache_key, vulnerabilities.clone()).await;
        Ok(vulnerabilities)
    }

    /// Scan for malicious packages using reputation analysis
    async fn scan_malicious_packages(&self, manifest_path: &Path) -> Result<Vec<MaliciousPackageIndicator>, Box<dyn std::error::Error>> {
        // Placeholder implementation - would integrate with:
        // - OpenSSF Scorecard
        // - Sonatype OSS Index
        // - GitHub Security tab
        // - Manual reputation databases

        let cache_key = format!("malicious_{}", manifest_path.display());
        if let Some(cached) = self.reputation_cache.get(&cache_key).await {
            return Ok(cached.unwrap_or_default());
        }

        // For now, return empty vector - would implement real scanning
        let malicious = Vec::new();

        self.reputation_cache.insert(cache_key, Some(malicious.clone())).await;
        Ok(malicious)
    }

    /// Check license compliance across all dependencies
    async fn check_license_compliance(&self, workspace_path: &Path) -> Result<Vec<LicenseComplianceIssue>, Box<dyn std::error::Error>> {
        self.license_checker.check_workspace_compliance(workspace_path).await
    }

    /// Generate Software Bill of Materials
    async fn generate_sbom(&self, manifest_path: &Path) -> Result<SoftwareBillOfMaterials, Box<dyn std::error::Error>> {
        use std::process::Command;

        // Use cargo tree to get dependency information
        let output = Command::new("cargo")
            .arg("tree")
            .arg("--manifest-path")
            .arg(manifest_path)
            .arg("--format=^package,features,checksum,dependencies")
            .output()
            .map_err(|e| format!("Failed to run cargo tree: {}", e))?;

        if !output.status.success() {
            return Err("cargo tree command failed".into());
        }

        let tree_output = String::from_utf8_lossy(&output.stdout);

        // Parse dependency information
        let mut dependencies = Vec::new();
        let mut licenses = HashMap::new();
        let mut checksums = HashMap::new();
        let mut provenance = HashMap::new();

        for line in tree_output.lines() {
            // Parse cargo tree output format
            // This is a simplified parser - real implementation would be more sophisticated
            if line.starts_with("  ") && !line.trim().is_empty() {
                if let Some((name, version)) = Self::parse_dependency_line(line) {
                    dependencies.push(SBOMDependency {
                        name: name.clone(),
                        version: version.clone(),
                        registry: "crates.io".to_string(),
                        hashes: HashMap::new(), // Would populate with actual checksums
                        licenses: vec!["MIT OR Apache-2.0".to_string()], // Placeholder
                        authors: vec![],
                        repository: None,
                        description: String::new(),
                    });

                    checksums.insert(name.clone(), "placeholder_checksum".to_string());
                    provenance.insert(name, ProvenanceData {
                        creator: "unknown".to_string(),
                        created: Utc::now(),
                        verified: false,
                        build_info: None,
                    });
                }
            }
        }

        Ok(SoftwareBillOfMaterials {
            dependencies,
            licenses,
            checksums,
            provenance,
        })
    }

    fn parse_dependency_line(line: &str) -> Option<(String, String)> {
        // Simplified parsing of cargo tree output
        let parts: Vec<&str> = line.split('@').collect();
        if parts.len() == 2 {
            let name = parts[0].trim().to_string();
            let version = parts[1].split(' ').next()?.to_string();
            Some((name, version))
        } else {
            None
        }
    }

    fn map_rustsec_severity(&self, severity: &str) -> SupplyChainRiskLevel {
        match severity {
            "Critical" => SupplyChainRiskLevel::Critical,
            "High" => SupplyChainRiskLevel::High,
            "Medium" => SupplyChainRiskLevel::Medium,
            "Low" => SupplyChainRiskLevel::Low,
            _ => SupplyChainRiskLevel::Unknown,
        }
    }

    fn check_fix_available(&self, report: &crate::rustsec_integration::VulnerabilityReport) -> bool {
        !report.patched_versions.is_empty()
    }

    fn calculate_dependency_risk_score(&self, vulnerabilities: &[SupplyChainVulnerability],
                                     malicious_packages: &[MaliciousPackageIndicator]) -> f32 {
        let vuln_score: f32 = vulnerabilities.iter()
            .map(|v| match v.severity {
                SupplyChainRiskLevel::Critical => 10.0,
                SupplyChainRiskLevel::High => 7.0,
                SupplyChainRiskLevel::Medium => 4.0,
                SupplyChainRiskLevel::Low => 2.0,
                SupplyChainRiskLevel::Negligible => 1.0,
                SupplyChainRiskLevel::Unknown => 5.0,
            })
            .sum();

        let malicious_score: f32 = malicious_packages.iter()
            .map(|m| m.confidence_score * 10.0)
            .sum();

        // Normalize to 0-10 scale
        let total_score = vuln_score + malicious_score;
        (total_score.min(100.0) / 10.0).min(10.0)
    }

    fn calculate_supply_chain_health_score(&self,
                                         vulnerabilities: &[SupplyChainVulnerability],
                                         malicious_packages: &[MaliciousPackageIndicator],
                                         license_issues: &[LicenseComplianceIssue]) -> f32 {
        let total_issues = vulnerabilities.len() + malicious_packages.len() + license_issues.len();

        if total_issues == 0 {
            return 10.0; // Perfect score
        }

        let critical_count = vulnerabilities.iter()
            .filter(|v| matches!(v.severity, SupplyChainRiskLevel::Critical))
            .count();

        let banned_license_count = license_issues.iter()
            .filter(|l| matches!(l.compliance_category, LicenseComplianceCategory::BannedLicense))
            .count();

        // Calculate health score as inverse of weighted issues
        let weighted_issues = critical_count as f32 * 3.0 + total_issues as f32 + banned_license_count as f32 * 5.0;

        (10.0 - weighted_issues.min(10.0)).max(0.0)
    }

    fn generate_recommendations(&self,
                              vulnerabilities: &[SupplyChainVulnerability],
                              malicious_packages: &[MaliciousPackageIndicator],
                              license_issues: &[LicenseComplianceIssue]) -> Vec<SupplyChainRecommendation> {
        let mut recommendations = Vec::new();

        // Generate vulnerability fix recommendations
        for vuln in vulnerabilities {
            if vuln.fix_available {
                recommendations.push(SupplyChainRecommendation {
                    priority: match vuln.severity {
                        SupplyChainRiskLevel::Critical => 5,
                        SupplyChainRiskLevel::High => 4,
                        SupplyChainRiskLevel::Medium => 3,
                        _ => 2,
                    },
                    category: RecommendationCategory::SecurityUpdate,
                    title: format!("Update {} to address {}", vuln.dependency_name, vuln.vulnerability_id),
                    description: vuln.description.clone(),
                    estimated_effort: EffortLevel::Low,
                    impact_score: match vuln.severity {
                        SupplyChainRiskLevel::Critical => 9.0,
                        SupplyChainRiskLevel::High => 7.0,
                        SupplyChainRiskLevel::Medium => 5.0,
                        SupplyChainRiskLevel::Low => 3.0,
                        _ => 1.0,
                    },
                    dependency_names: vec![vuln.dependency_name.clone()],
                });
            }
        }

        // Generate license compliance recommendations
        for issue in license_issues {
            recommendations.push(SupplyChainRecommendation {
                priority: match issue.risk_level {
                    SupplyChainRiskLevel::Critical => 5,
                    SupplyChainRiskLevel::High => 4,
                    SupplyChainRiskLevel::Medium => 3,
                    _ => 2,
                },
                category: RecommendationCategory::LicenseChange,
                title: format!("Address license compliance for {}", issue.dependency_name),
                description: format!("{} license may have compliance issues", issue.problematic_license),
                estimated_effort: match issue.compliance_category {
                    LicenseComplianceCategory::BannedLicense => EffortLevel::High,
                    _ => EffortLevel::Medium,
                },
                impact_score: 8.0, // License issues are serious
                dependency_names: vec![issue.dependency_name.clone()],
            });
        }

        // Sort by priority and impact
        recommendations.sort_by(|a, b| {
            (b.priority, b.impact_score).cmp(&(a.priority, a.impact_score))
        });

        recommendations
    }
}
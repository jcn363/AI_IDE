//! # Wave 1 Advanced Security Vulnerability Detection
//!
//! Comprehensive security scanning and vulnerability analysis system for codebases,
//! providing intelligent threat detection, risk assessment, and remediation suggestions.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use regex::Regex;

/// Security vulnerability scanner
#[derive(Debug)]
pub struct SecurityScanner {
    rules: Vec<SecurityRule>,
    cve_database: CVEDatabase,
    risk_assessor: RiskAssessmentEngine,
}

impl SecurityScanner {
    pub fn new() -> Self {
        let rules = Self::initialize_security_rules();
        Self {
            rules,
            cve_database: CVEDatabase::new(),
            risk_assessor: RiskAssessmentEngine::new(),
        }
    }

    pub async fn scan_code(&self, code: &str, language: &str) -> Result<SecurityReport, SecurityError> {
        let mut vulnerabilities = Vec::new();

        // Scan for each security rule
        for rule in &self.rules {
            if rule.languages.contains(&language.to_string()) || rule.languages.contains(&"generic") {
                let results = rule.scan_code(code)?;
                vulnerabilities.extend(results);
            }
        }

        // Assess risk for all findings
        let risk_assessment = self.risk_assessor.assess_overall_risk(&vulnerabilities);
        let recommendations = self.generate_recommendations(&vulnerabilities);

        Ok(SecurityReport {
            vulnerabilities,
            risk_assessment,
            recommendations,
            scanned_at: chrono::Utc::now(),
            total_lines: code.lines().count(),
        })
    }

    fn initialize_security_rules() -> Vec<SecurityRule> {
        vec![
            SecurityRule {
                id: "SQL_INJECTION".to_string(),
                name: "SQL Injection Vulnerability".to_string(),
                severity: Severity::Critical,
                cwe_id: Some(89),
                description: "Potential SQL injection vulnerability detected".to_string(),
                languages: vec!["rust".to_string(), "sql".to_string()],
                patterns: vec![SecurityPattern::Regex(r#"SELECT.*\+.*\{t\}"#.to_string())],
                recommendations: vec![
                    "Use prepared statements or parameterized queries".to_string(),
                    "Validate and sanitize all user inputs".to_string(),
                ],
            },
            SecurityRule {
                id: "XSS_VULNERABILITY".to_string(),
                name: "Cross-Site Scripting (XSS)".to_string(),
                severity: Severity::High,
                cwe_id: Some(79),
                description: "Potential XSS vulnerability in HTML content".to_string(),
                languages: vec!["html".to_string(), "javascript".to_string()],
                patterns: vec![SecurityPattern::Regex(r#"innerHTML\s*=.*\+.*"#.to_string())],
                recommendations: vec![
                    "Use textContent or innerText instead of innerHTML".to_string(),
                    "Sanitize HTML input using proper libraries".to_string(),
                    "Use Content Security Policy (CSP)".to_string(),
                ],
            },
            SecurityRule {
                id: "BUFFER_OVERFLOW".to_string(),
                name: "Buffer Overflow Risk".to_string(),
                severity: Severity::High,
                cwe_id: Some(120),
                description: "Potential buffer overflow vulnerability".to_string(),
                languages: vec!["c".to_string(), "cpp".to_string(), "rust".to_string()],
                patterns: vec![SecurityPattern::Regex(r#"unsafe \{.*\[.*\].*=.*\}"#.to_string())],
                recommendations: vec![
                    "Use safe Rust abstractions instead of unsafe code".to_string(),
                    "Implement bounds checking".to_string(),
                    "Use standard library collections".to_string(),
                ],
            },
            SecurityRule {
                id: "HARD_CODED_CREDENTIALS".to_string(),
                name: "Hard-coded Credentials".to_string(),
                severity: Severity::Critical,
                cwe_id: Some(798),
                description: "Hard-coded credentials detected".to_string(),
                languages: vec!["generic".to_string()],
                patterns: vec![
                    SecurityPattern::Regex(r#"password\s*=.*["'].*["']"#.to_string()),
                    SecurityPattern::Regex(r#"api_key\s*=.*["'].*["']"#.to_string()),
                ],
                recommendations: vec![
                    "Use environment variables for credentials".to_string(),
                    "Implement secure credential management".to_string(),
                    "Use configuration files outside source control".to_string(),
                ],
            },
            SecurityRule {
                id: "PATH_TRAVERSAL".to_string(),
                name: "Path Traversal Vulnerability".to_string(),
                severity: Severity::High,
                cwe_id: Some(22),
                description: "Potential path traversal vulnerability".to_string(),
                languages: vec!["generic".to_string()],
                patterns: vec![SecurityPattern::Regex(r#"(\.\./|\.\.\\)"#.to_string())],
                recommendations: vec![
                    "Validate and canonicalize all file paths".to_string(),
                    "Use whitelist approach for allowed paths".to_string(),
                    "Reject any path containing '..' characters".to_string(),
                ],
            },
        ]
    }

    fn generate_recommendations(&self, vulnerabilities: &[Vulnerability]) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Group recommendations by type
        let mut recommendation_set: HashSet<String> = HashSet::new();

        for vuln in vulnerabilities {
            for rec in &vuln.recommendations {
                recommendation_set.insert(rec.clone());
            }
        }

        recommendations.extend(recommendation_set.into_iter());
        recommendations
    }
}

/// Security rule definition
#[derive(Debug, Clone)]
pub struct SecurityRule {
    pub id: String,
    pub name: String,
    pub severity: Severity,
    pub cwe_id: Option<u32>,
    pub description: String,
    pub languages: Vec<String>,
    pub patterns: Vec<SecurityPattern>,
    pub recommendations: Vec<String>,
}

impl SecurityRule {
    pub fn scan_code(&self, code: &str) -> Result<Vec<Vulnerability>, SecurityError> {
        let mut vulnerabilities = Vec::new();

        for pattern in &self.patterns {
            match pattern {
                SecurityPattern::Regex(pattern_str) => {
                    if let Ok(regex) = Regex::new(pattern_str) {
                        for line_num in 1..=code.lines().count() {
                            if let Some(line) = code.lines().nth(line_num - 1) {
                                if regex.is_match(line) {
                                    vulnerabilities.push(Vulnerability {
                                        rule_id: self.id.clone(),
                                        name: self.name.clone(),
                                        severity: self.severity,
                                        cwe_id: self.cwe_id,
                                        location: CodeLocation {
                                            file: "current".to_string(),
                                            line: line_num,
                                            column: 0,
                                        },
                                        description: self.description.clone(),
                                        code_snippet: line.to_string(),
                                        recommendations: self.recommendations.clone(),
                                        cvss_score: None,
                                        confidence: 0.9,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(vulnerabilities)
    }
}

/// Security pattern types
#[derive(Debug, Clone)]
pub enum SecurityPattern {
    Regex(String),
    Semantic(String),
}

/// Vulnerability finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub rule_id: String,
    pub name: String,
    pub severity: Severity,
    pub cwe_id: Option<u32>,
    pub location: CodeLocation,
    pub description: String,
    pub code_snippet: String,
    pub recommendations: Vec<String>,
    pub cvss_score: Option<f64>,
    pub confidence: f64,
}

/// Severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Informational,
    Low,
    Medium,
    High,
    Critical,
}

/// Code location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

/// Risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk: RiskLevel,
    pub risk_score: f64,
    pub vulnerability_count: usize,
    pub high_severity_count: usize,
    pub critical_severity_count: usize,
}

/// Risk assessment engine
#[derive(Debug)]
pub struct RiskAssessmentEngine;

impl RiskAssessmentEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn assess_overall_risk(&self, vulnerabilities: &[Vulnerability]) -> RiskAssessment {
        let vulnerability_count = vulnerabilities.len();
        let high_severity_count = vulnerabilities.iter()
            .filter(|v| matches!(v.severity, Severity::High))
            .count();
        let critical_severity_count = vulnerabilities.iter()
            .filter(|v| matches!(v.severity, Severity::Critical))
            .count();

        // Calculate risk score (0-100)
        let risk_score = if vulnerability_count == 0 {
            0.0
        } else {
            (critical_severity_count * 10 + high_severity_count * 5 + vulnerability_count) as f64 * 2.0
        };

        let overall_risk = if risk_score > 80.0 {
            RiskLevel::Critical
        } else if risk_score > 60.0 {
            RiskLevel::High
        } else if risk_score > 30.0 {
            RiskLevel::Medium
        } else if risk_score > 10.0 {
            RiskLevel::Low
        } else {
            RiskLevel::Minimal
        };

        RiskAssessment {
            overall_risk,
            risk_score: risk_score.min(100.0),
            vulnerability_count,
            high_severity_count,
            critical_severity_count,
        }
    }
}

/// Risk levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Minimal,
    Low,
    Medium,
    High,
    Critical,
}

/// Security report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    pub vulnerabilities: Vec<Vulnerability>,
    pub risk_assessment: RiskAssessment,
    pub recommendations: Vec<String>,
    pub scanned_at: chrono::DateTime<chrono::Utc>,
    pub total_lines: usize,
}

/// CVE database integration
#[derive(Debug)]
pub struct CVEDatabase {
    cve_cache: HashMap<String, CVEEntry>,
}

impl CVEDatabase {
    pub fn new() -> Self {
        Self {
            cve_cache: HashMap::new(),
        }
    }

    pub async fn lookup_cve(&self, cve_id: &str) -> Option<&CVEEntry> {
        // In a real implementation, this would query a CVE database
        self.cve_cache.get(cve_id)
    }
}

/// CVE entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CVEEntry {
    pub id: String,
    pub severity: Severity,
    pub cvss_score: f64,
    pub description: String,
    pub published_date: chrono::DateTime<chrono::Utc>,
}

/// Security analysis error
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Pattern compilation failed: {0}")]
    PatternError(String),

    #[error("Scan failed: {0}")]
    ScanError(String),

    #[error("Invalid language: {0}")]
    InvalidLanguage(String),

    #[error("Database connection failed: {0}")]
    DatabaseError(String),
}

// Example usage:
// ```
// use rust_ai_ide_ai1_security::SecurityScanner;
// let scanner = SecurityScanner::new();
// let report = scanner.scan_code(code, "rust").await?;
// ```

pub use SecurityScanner;
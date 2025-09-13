//! A01:2021 - Broken Access Control Detector
//! Implementation of OWASP Top 10 A01:2021 Broken Access Control detection

use super::{
    AttackComplexity, AttackVector, AvailabilityImpact, ConfidentialityImpact, DetectionResult,
    ExploitabilityScore, ImpactScore, IntegrityImpact, OWASPCategory, OWASPDetector,
    PrivilegesRequired, Scope, UserInteraction,
};
use crate::security::*;
use async_trait::async_trait;
use regex::Regex;
use std::path::Path;

pub struct BrokenAccessControlDetector {
    access_patterns: Vec<Regex>,
    authorization_patterns: Vec<Regex>,
    elevation_patterns: Vec<Regex>,
}

impl BrokenAccessControlDetector {
    pub fn new() -> Self {
        let access_patterns = Self::initialize_access_patterns();
        let authorization_patterns = Self::initialize_authorization_patterns();
        let elevation_patterns = Self::initialize_elevation_patterns();

        Self {
            access_patterns,
            authorization_patterns,
            elevation_patterns,
        }
    }

    fn initialize_access_patterns() -> Vec<Regex> {
        vec![
            Regex::new(
                r"pub\s+(?:struct|fn)\s+(?:get_|set_|add_|remove_|modify_|admin_|super_)\w+",
            )
            .unwrap(), // Public sensitive methods
            Regex::new(r"allow_anonymous_access\s*:\s*true").unwrap(), // Anonymous access enabled
            Regex::new(r"require_authentication\s*:\s*false").unwrap(), // Missing authentication requirement
            Regex::new(r"skip_authorization").unwrap(),                 // Skipped authorization
            Regex::new(r"\.force_wrap_unchecked\(").unwrap(), // Forced unchecked operations
        ]
    }

    fn initialize_authorization_patterns() -> Vec<Regex> {
        vec![
            Regex::new(r"role.*==.*admin|admin.*==.*role").unwrap(), // Hardcoded admin role checks
            Regex::new(r"\bREAD\b.*\|\s*\WRITE\b.*\|\s*\DELETE\b").unwrap(), // File permission checks
            Regex::new(r"#\[derive.*Debug.*\]").unwrap(), // Debug trait that may leak sensitive data
            Regex::new(r"unsafe\s*\{.*std::fs::.*\}").unwrap(), // Unsafe file operations
        ]
    }

    fn initialize_elevation_patterns() -> Vec<Regex> {
        vec![
            Regex::new(r"setuid.*setgid|geteuid.*getegid").unwrap(), // UID/GID manipulation
            Regex::new(r"sudo|su\s+").unwrap(), // Privilege escalation commands
            Regex::new(r"pkexec|gksudo").unwrap(), // Administrative privilege escalation
            Regex::new(r"cap_set_proc|cap_get_proc").unwrap(), // Linux capabilities manipulation
            Regex::new(r"schg|sappnd").unwrap(), // File system flags for immutability bypass
        ]
    }
}

#[async_trait]
impl OWASPDetector for BrokenAccessControlDetector {
    fn category(&self) -> OWASPCategory {
        OWASPCategory::A01_2021_BrokenAccessControl
    }

    fn name(&self) -> &str {
        "Broken Access Control Detector"
    }

    async fn analyze_codebase(
        &self,
        workspace_path: &Path,
    ) -> Result<Vec<DetectionResult>, Box<dyn std::error::Error>> {
        use walkdir::WalkDir;
        let mut results = Vec::new();

        for entry in WalkDir::new(workspace_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().map_or(false, |ext| ext == "rs") {
                if let Ok(code) = std::fs::read_to_string(entry.path()) {
                    let file_path = entry.path().to_string_lossy().to_string();
                    results.extend(self.analyze_file(&code, &file_path));
                }
            }
        }

        Ok(results)
    }

    fn analyze_file(&self, code: &str, file_path: &str) -> Vec<DetectionResult> {
        let mut results = Vec::new();

        results.extend(self.detect_missing_access_controls(code, file_path));
        results.extend(self.detect_insecure_authorization(code, file_path));
        results.extend(self.detect_privilege_escalation(code, file_path));

        results
    }
}

impl BrokenAccessControlDetector {
    fn detect_missing_access_controls(&self, code: &str, file_path: &str) -> Vec<DetectionResult> {
        let mut results = Vec::new();

        for pattern in &self.access_patterns {
            for mat in pattern.find_iter(code) {
                let line_number = code[..mat.start()].lines().count();
                let exploitability = ExploitabilityScore {
                    attack_vector: AttackVector::Network,
                    attack_complexity: AttackComplexity::Low,
                    privileges_required: PrivilegesRequired::None,
                    user_interaction: UserInteraction::None,
                    scope: Scope::Unchanged,
                };
                let impact = ImpactScore {
                    confidentiality: ConfidentialityImpact::High,
                    integrity: IntegrityImpact::High,
                    availability: AvailabilityImpact::Low,
                };

                results.push(DetectionResult {
                    security_issue: SecurityIssue {
                        category: SecurityCategory::command_injection, // Should map to appropriate category
                        severity: SecuritySeverity::High,
                        title: "Missing Access Control".to_string(),
                        description: "Detected potentially sensitive public access without proper access controls".to_string(),
                        file_path: file_path.to_string(),
                        line_number: Some(line_number),
                        column: Some(mat.start()),
                        code_snippet: Some(mat.as_str().to_string()),
                        remediation: "Implement proper access control checks. Use role-based access control (RBAC) or attribute-based access control (ABAC).".to_string(),
                        confidence: 0.8,
                        cwe_id: Some(862), // CWE-862: Missing Authorization
                    },
                    exploitability,
                    impact,
                    ai_confidence: 0.85,
                    patterns_detected: vec!["public sensitive method".to_string()],
                    remediation_steps: vec![
                        "Implement authorization checks before sensitive operations".to_string(),
                        "Use middleware or guards for access control".to_string(),
                        "Implement principle of least privilege".to_string(),
                        "Add comprehensive audit logging".to_string(),
                    ],
                });
            }
        }

        results
    }

    fn detect_insecure_authorization(&self, code: &str, file_path: &str) -> Vec<DetectionResult> {
        let mut results = Vec::new();

        for pattern in &self.authorization_patterns {
            for mat in pattern.find_iter(code) {
                let line_number = code[..mat.start()].lines().count();
                let exploitability = ExploitabilityScore {
                    attack_vector: AttackVector::AdjacentNetwork,
                    attack_complexity: AttackComplexity::Low,
                    privileges_required: PrivilegesRequired::Low,
                    user_interaction: UserInteraction::None,
                    scope: Scope::Unchanged,
                };
                let impact = ImpactScore {
                    confidentiality: ConfidentialityImpact::High,
                    integrity: IntegrityImpact::High,
                    availability: AvailabilityImpact::Low,
                };

                results.push(DetectionResult {
                    security_issue: SecurityIssue {
                        category: SecurityCategory::unsafe_code, // Should map to appropriate category
                        severity: SecuritySeverity::High,
                        title: "Insecure Authorization Logic".to_string(),
                        description: "Detected potential authorization bypass or insecure authorization mechanism".to_string(),
                        file_path: file_path.to_string(),
                        line_number: Some(line_number),
                        column: Some(mat.start()),
                        code_snippet: Some(mat.as_str().to_string()),
                        remediation: "Implement secure authorization mechanisms. Avoid hardcoded role checks and use proper access control systems.".to_string(),
                        confidence: 0.75,
                        cwe_id: Some(285), // CWE-285: Improper Authorization
                    },
                    exploitability,
                    impact,
                    ai_confidence: 0.80,
                    patterns_detected: vec!["hardcoded authorization".to_string()],
                    remediation_steps: vec![
                        "Replace hardcoded role checks with dynamic authorization".to_string(),
                        "Implement proper permission validation".to_string(),
                        "Use ACLs or RBAC for complex permissions".to_string(),
                        "Add security audit logging".to_string(),
                    ],
                });
            }
        }

        results
    }

    fn detect_privilege_escalation(&self, code: &str, file_path: &str) -> Vec<DetectionResult> {
        let mut results = Vec::new();

        for pattern in &self.elevation_patterns {
            for mat in pattern.find_iter(code) {
                let line_number = code[..mat.start()].lines().count();
                let exploitability = ExploitabilityScore {
                    attack_vector: AttackVector::Local,
                    attack_complexity: AttackComplexity::High,
                    privileges_required: PrivilegesRequired::Low,
                    user_interaction: UserInteraction::None,
                    scope: Scope::Changed,
                };
                let impact = ImpactScore {
                    confidentiality: ConfidentialityImpact::High,
                    integrity: IntegrityImpact::High,
                    availability: AvailabilityImpact::High,
                };

                results.push(DetectionResult {
                    security_issue: SecurityIssue {
                        category: SecurityCategory::command_injection, // Should map to appropriate category
                        severity: SecuritySeverity::Critical,
                        title: "Privilege Escalation Vulnerability".to_string(),
                        description: "Detected potential privilege escalation or unauthorized privilege manipulation".to_string(),
                        file_path: file_path.to_string(),
                        line_number: Some(line_number),
                        column: Some(mat.start()),
                        code_snippet: Some(mat.as_str().to_string()),
                        remediation: "Remove privilege manipulation code or implement strict privilege validation. Use system-level privilege management.".to_string(),
                        confidence: 0.95,
                        cwe_id: Some(269), // CWE-269: Improper Privilege Management
                    },
                    exploitability,
                    impact,
                    ai_confidence: 0.90,
                    patterns_detected: vec!["privilege escalation".to_string()],
                    remediation_steps: vec![
                        "Remove manual privilege manipulation".to_string(),
                        "Use secure privilege management APIs".to_string(),
                        "Implement privilege bounding".to_string(),
                        "Add comprehensive privilege audit logging".to_string(),
                    ],
                });
            }
        }

        results
    }

    fn supports_ai_enhancement() -> bool {
        true
    }
}

//! Security Misconfiguration Detector
//!
//! This module implements detection for OWASP A05:2021 - Security Misconfiguration.
//! It identifies insecure configurations and default settings.

use super::*;

/// Detector for security misconfigurations
#[derive(Debug, Default)]
pub struct SecurityMisconfigurationDetector;

impl OWASPDetector for SecurityMisconfigurationDetector {
    fn detect(&self, code: &str, file_path: &str) -> Vec<DetectionResult> {
        let mut findings = Vec::new();

        // Check for debug mode in production
        if code.contains("debug = true") || code.contains("debug!(") {
            findings.push(self.create_finding(
                SecurityCategory::SecurityMisconfiguration,
                SecuritySeverity::High,
                "Debug mode enabled in potentially production code",
                file_path,
                "CONFIG-001",
                "Debug Mode Enabled",
                "Debug mode should be disabled in production environments.",
                "Debug mode enabled",
                vec![
                    "Disable debug mode in production",
                    "Use environment-specific configuration",
                    "Remove or protect debug endpoints",
                ],
                0.9,
            ));
        }

        // Check for disabled security headers
        if code.contains("disable_security_headers") || code.contains("security_headers = false") {
            findings.push(self.create_finding(
                SecurityCategory::SecurityMisconfiguration,
                SecuritySeverity::Medium,
                "Security headers are disabled",
                file_path,
                "HEADER-001",
                "Missing Security Headers",
                "Security headers help protect against various attacks and should be properly configured.",
                "Disabled security headers",
                vec![
                    "Enable and properly configure security headers",
                    "Include headers like X-Content-Type-Options, X-Frame-Options, etc.",
                    "Use security headers middleware",
                ],
                0.85,
            ));
        }

        // Check for permissive CORS settings
        if code.contains("Access-Control-Allow-Origin: *")
            || code.contains("allow_any_origin()")
            || code.contains("allow_credentials(true)") && code.contains("allow_any_origin()")
        {
            findings.push(self.create_finding(
                SecurityCategory::SecurityMisconfiguration,
                SecuritySeverity::Medium,
                "Overly permissive CORS settings",
                file_path,
                "CORS-001",
                "Permissive CORS Policy",
                "Overly permissive CORS settings can expose your application to security risks.",
                "Permissive CORS configuration",
                vec![
                    "Restrict CORS to specific origins",
                    "Avoid using wildcard (*) for Access-Control-Allow-Origin with credentials",
                    "Set appropriate CORS headers and methods",
                ],
                0.8,
            ));
        }

        findings
    }

    fn get_detector_name(&self) -> &'static str {
        "Security Misconfiguration Detector"
    }
}

impl SecurityMisconfigurationDetector {
    fn create_finding(
        &self,
        category: SecurityCategory,
        severity: SecuritySeverity,
        message: &str,
        file_path: &str,
        rule_id: &str,
        rule_name: &str,
        rule_description: &str,
        pattern: &str,
        remediation_steps: Vec<&str>,
        confidence: f32,
    ) -> DetectionResult {
        DetectionResult {
            security_issue:    SecurityIssue {
                category,
                severity,
                message: message.to_string(),
                file_path: file_path.to_string(),
                line: 0, // Will be set by the caller
                rule: SecurityRule::new(
                    rule_id.to_string(),
                    rule_name.to_string(),
                    rule_description.to_string(),
                ),
            },
            exploitability:    self.calculate_exploitability(&category),
            impact:            self.calculate_impact(&severity),
            ai_confidence:     confidence,
            patterns_detected: vec![pattern.to_string()],
            remediation_steps: remediation_steps
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_mode_detection() {
        let detector = SecurityMisconfigurationDetector::default();
        let code = r#"
            // In config.rs
            pub struct Config {
                pub debug: bool,
            }

            impl Default for Config {
                fn default() -> Self {
                    Self { debug: true } // Debug mode enabled by default!
                }
            }
        "#;

        let results = detector.detect(code, "config.rs");
        assert!(!results.is_empty());
        assert!(results[0]
            .security_issue
            .message
            .contains("Debug mode enabled"));
    }

    #[test]
    fn test_permissive_cors_detection() {
        let detector = SecurityMisconfigurationDetector::default();
        let code = r#"
            // In main.rs
            let cors = Cors::default()
                .allow_any_origin()
                .allow_credentials(true);
        "#;

        let results = detector.detect(code, "main.rs");
        assert!(!results.is_empty());
        assert!(results[0]
            .security_issue
            .message
            .contains("permissive CORS"));
    }
}

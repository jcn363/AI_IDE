//! Insecure Design Detector
//!
//! This module implements detection for OWASP A04:2021 - Insecure Design.
//! It identifies patterns that indicate potential design flaws and insecure defaults.

use super::*;

/// Detector for insecure design patterns
#[derive(Debug, Default)]
pub struct InsecureDesignDetector;

impl OWASPDetector for InsecureDesignDetector {
    fn detect(&self, code: &str, file_path: &str) -> Vec<DetectionResult> {
        let mut findings = Vec::new();

        // Check for missing authentication/authorization
        if code.contains("#[allow(unused_imports)]") && code.contains("auth") {
            findings.push(self.create_finding(
                SecurityCategory::InsecureDesign,
                SecuritySeverity::High,
                "Missing or bypassed authentication/authorization check",
                file_path,
                "AUTH-001",
                "Insufficient Authentication/Authorization",
                "The code appears to bypass or lack proper authentication/authorization checks.",
                "Missing auth check",
                vec![
                    "Implement proper authentication and authorization checks",
                    "Use a well-established authentication framework",
                    "Follow the principle of least privilege",
                ],
                0.8,
            ));
        }

        // Check for hardcoded credentials
        if code.contains("password") && (code.contains("123") || code.contains("admin")) {
            findings.push(self.create_finding(
                SecurityCategory::InsecureDesign,
                SecuritySeverity::Critical,
                "Hardcoded credentials detected",
                file_path,
                "CREDS-001",
                "Hardcoded Credentials",
                "The code contains hardcoded credentials, which is a security risk.",
                "Hardcoded password or credential",
                vec![
                    "Remove hardcoded credentials",
                    "Use environment variables or a secure secrets management system",
                    "Implement proper credential management",
                ],
                0.95,
            ));
        }

        // Check for disabled security features
        if code.contains("#[allow(unsafe_code)]") || code.contains("unsafe {") {
            findings.push(self.create_finding(
                SecurityCategory::InsecureDesign,
                SecuritySeverity::Medium,
                "Security features disabled or unsafe code used",
                file_path,
                "SEC-001",
                "Disabled Security Features",
                "The code disables security features or uses unsafe blocks without proper justification.",
                "Disabled security feature or unsafe block",
                vec![
                    "Avoid disabling security features without proper justification",
                    "If unsafe code is necessary, document the reasons and safety measures",
                    "Consider safer alternatives to unsafe code",
                ],
                0.75,
            ));
        }

        findings
    }

    fn get_detector_name(&self) -> &'static str {
        "Insecure Design Detector"
    }
}

impl InsecureDesignDetector {
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
    fn test_missing_auth_detection() {
        let detector = InsecureDesignDetector::default();
        let code = r#"
            #[allow(unused_imports)]
            use some_auth_lib::*;

            // Missing auth check
            fn get_sensitive_data() -> String {
                "sensitive data".to_string()
            }
        "#;

        let results = detector.detect(code, "test.rs");
        assert!(!results.is_empty());
        assert!(results[0]
            .security_issue
            .message
            .contains("authentication/authorization"));
    }

    #[test]
    fn test_hardcoded_credentials() {
        let detector = InsecureDesignDetector::default();
        let code = r#"
            const DB_PASSWORD: &str = "admin123";
            const API_KEY: &str = "abcdef123456";
        "#;

        let results = detector.detect(code, "test.rs");
        assert!(!results.is_empty());
        assert!(results[0]
            .security_issue
            .message
            .contains("Hardcoded credentials"));
    }
}

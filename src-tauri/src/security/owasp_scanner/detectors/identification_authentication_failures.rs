//! Identification and Authentication Failures Detector
//!
//! This module implements detection for OWASP A07:2021 - Identification and Authentication
//! Failures. It identifies issues related to authentication mechanisms and session management.

use super::*;

/// Detector for identification and authentication failures
#[derive(Debug, Default)]
pub struct IdentificationAuthFailuresDetector;

impl OWASPDetector for IdentificationAuthFailuresDetector {
    fn detect(&self, code: &str, file_path: &str) -> Vec<DetectionResult> {
        let mut findings = Vec::new();

        // Check for weak password policies
        if code.contains("password")
            && (code.contains("min_length") && code.contains("< 8")
                || !code.contains("uppercase")
                || !code.contains("lowercase")
                || !code.contains("number"))
        {
            findings.push(self.create_finding(
                SecurityCategory::IdentificationAuthFailures,
                SecuritySeverity::Medium,
                "Weak password policy detected",
                file_path,
                "AUTH-002",
                "Weak Password Policy",
                "The password policy does not enforce strong password requirements.",
                "Weak password validation",
                vec![
                    "Enforce minimum password length of at least 12 characters",
                    "Require a mix of uppercase, lowercase, numbers, and special characters",
                    "Implement password strength meter for user feedback",
                    "Consider using a password policy library",
                ],
                0.85,
            ));
        }

        // Check for hardcoded credentials
        if code.contains("password") && (code.contains("123") || code.contains("admin")) {
            findings.push(self.create_finding(
                SecurityCategory::IdentificationAuthFailures,
                SecuritySeverity::Critical,
                "Hardcoded credentials detected",
                file_path,
                "CREDS-002",
                "Hardcoded Credentials",
                "The code contains hardcoded credentials which is a security risk.",
                "Hardcoded password or credential",
                vec![
                    "Remove hardcoded credentials from source code",
                    "Use environment variables or a secure secrets management system",
                    "Implement proper credential management",
                ],
                0.95,
            ));
        }

        // Check for missing MFA
        if (code.contains("login") || code.contains("authenticate"))
            && !code.contains("mfa")
            && !code.contains("2fa")
            && !code.contains("two_factor")
        {
            findings.push(self.create_finding(
                SecurityCategory::IdentificationAuthFailures,
                SecuritySeverity::High,
                "Multi-factor authentication (MFA) not implemented",
                file_path,
                "MFA-001",
                "Missing Multi-Factor Authentication",
                "The authentication mechanism does not implement multi-factor authentication.",
                "Missing MFA implementation",
                vec![
                    "Implement multi-factor authentication for all users",
                    "Support time-based one-time passwords (TOTP)",
                    "Consider WebAuthn for passwordless authentication",
                    "Provide recovery options for lost MFA devices",
                ],
                0.8,
            ));
        }

        // Check for session management issues
        if code.contains("session")
            && (code.contains("expires: None") || code.contains("infinite") || code.contains("never_expires"))
        {
            findings.push(self.create_finding(
                SecurityCategory::IdentificationAuthFailures,
                SecuritySeverity::High,
                "Insecure session configuration detected",
                file_path,
                "SESS-001",
                "Insecure Session Configuration",
                "Sessions do not have proper expiration settings.",
                "Missing session expiration",
                vec![
                    "Set reasonable session expiration times",
                    "Implement session timeout after periods of inactivity",
                    "Allow users to view and revoke active sessions",
                    "Regenerate session IDs after login",
                ],
                0.9,
            ));
        }

        findings
    }

    fn get_detector_name(&self) -> &'static str {
        "Identification and Authentication Failures Detector"
    }
}

impl IdentificationAuthFailuresDetector {
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
    fn test_weak_password_policy() {
        let detector = IdentificationAuthFailuresDetector::default();

        // Test weak password policy
        let weak_auth_code = r#"
            fn validate_password(password: &str) -> bool {
                password.len() >= 6  // Too short
            }
        "#;

        let results = detector.detect(weak_auth_code, "auth.rs");
        assert!(!results.is_empty());
        assert!(results[0]
            .security_issue
            .message
            .contains("Weak password policy"));

        // Test strong password policy (should not trigger)
        let strong_auth_code = r#"
            fn validate_password(password: &str) -> bool {
                let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
                let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
                let has_digit = password.chars().any(|c| c.is_ascii_digit());
                password.len() >= 12 && has_upper && has_lower && has_digit
            }
        "#;

        let safe_results = detector.detect(strong_auth_code, "auth.rs");
        assert!(!safe_results
            .iter()
            .any(|r| r.security_issue.message.contains("Weak password policy")));
    }

    #[test]
    fn test_missing_mfa() {
        let detector = IdentificationAuthFailuresDetector::default();

        // Test missing MFA
        let login_code = r#"
            fn login(username: &str, password: &str) -> Result<Session, Error> {
                // Authenticate user
                let user = authenticate_user(username, password)?;

                // Create session
                let session = create_session(&user.id);

                Ok(session)
            }
        "#;

        let results = detector.detect(login_code, "auth.rs");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r
            .security_issue
            .message
            .contains("Multi-factor authentication")));
    }
}

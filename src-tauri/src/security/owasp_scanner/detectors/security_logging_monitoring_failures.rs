//! Security Logging and Monitoring Failures Detector
//!
//! This module implements detection for OWASP A09:2021 - Security Logging and Monitoring Failures.
//! It identifies issues related to insufficient logging, monitoring, and alerting.

use super::*;

/// Detector for security logging and monitoring failures
#[derive(Debug, Default)]
pub struct SecurityLoggingMonitoringDetector;

impl OWASPDetector for SecurityLoggingMonitoringDetector {
    fn detect(&self, code: &str, file_path: &str) -> Vec<DetectionResult> {
        let mut findings = Vec::new();

        // Check for missing security event logging
        if (code.contains("login") || code.contains("auth") || code.contains("failed"))
            && !code.contains("log::")
            && !code.contains("tracing::")
            && !code.contains("slog::")
            && !code.contains("info!")
            && !code.contains("warn!")
            && !code.contains("error!")
        {
            findings.push(self.create_finding(
                SecurityCategory::SecurityLoggingMonitoringFailures,
                SecuritySeverity::Medium,
                "Missing security event logging",
                file_path,
                "LOG-001",
                "Missing Security Event Logging",
                "Security-relevant events are not being logged, which can hinder incident response and investigation.",
                "Missing security event logging",
                vec![
                    "Log all authentication attempts, both successful and failed",
                    "Log security-relevant actions (e.g., password changes, privilege changes)",
                    "Include sufficient context in logs (timestamps, user IDs, IP addresses)",
                    "Ensure logs are protected from tampering",
                ],
                0.85,
            ));
        }

        // Check for sensitive data in logs
        if (code.contains("log::info") || code.contains("println!"))
            && (code.contains("password")
                || code.contains("token")
                || code.contains("secret")
                || code.contains("api[_-]?key"))
        {
            findings.push(self.create_finding(
                SecurityCategory::SecurityLoggingMonitoringFailures,
                SecuritySeverity::High,
                "Sensitive data exposure in logs",
                file_path,
                "LOG-002",
                "Sensitive Data in Logs",
                "Sensitive information is being written to logs, which could be exposed to unauthorized users.",
                "Sensitive data in logs",
                vec![
                    "Avoid logging sensitive information (passwords, tokens, API keys, etc.)",
                    "Implement log redaction for sensitive data",
                    "Use structured logging with appropriate log levels",
                    "Review log output in development and production",
                ],
                0.9,
            ));
        }

        // Check for missing audit trails
        if (code.contains("fn update") || code.contains("fn delete") || code.contains("fn modify"))
            && !code.contains("audit")
            && !code.contains("log")
        {
            findings.push(self.create_finding(
                SecurityCategory::SecurityLoggingMonitoringFailures,
                SecuritySeverity::Medium,
                "Missing audit trail for sensitive operations",
                file_path,
                "AUDIT-001",
                "Missing Audit Trail",
                "Sensitive operations are not being audited, making it difficult to track changes and detect misuse.",
                "Missing audit logging",
                vec![
                    "Implement audit trails for all sensitive operations",
                    "Log who performed the action, what was changed, and when",
                    "Store audit logs securely and protect them from tampering",
                    "Implement log retention policies",
                ],
                0.8,
            ));
        }

        // Check for missing monitoring
        if (code.contains("error") || code.contains("panic"))
            && !code.contains("metrics!")
            && !code.contains("counter!")
            && !code.contains("gauge!")
            && !code.contains("prometheus")
            && !code.contains("statsd")
            && !code.contains("monitor")
        {
            findings.push(self.create_finding(
                SecurityCategory::SecurityLoggingMonitoringFailures,
                SecuritySeverity::Low,
                "Missing error monitoring and metrics",
                file_path,
                "MON-001",
                "Missing Error Monitoring",
                "The application does not appear to have proper error monitoring and metrics collection.",
                "Missing error monitoring",
                vec![
                    "Implement error tracking and monitoring",
                    "Set up alerts for unusual error rates or patterns",
                    "Collect and analyze application metrics",
                    "Integrate with monitoring solutions like Prometheus, Datadog, or New Relic",
                ],
                0.75,
            ));
        }

        findings
    }

    fn get_detector_name(&self) -> &'static str {
        "Security Logging and Monitoring Failures Detector"
    }
}

impl SecurityLoggingMonitoringDetector {
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
    fn test_missing_security_logging() {
        let detector = SecurityLoggingMonitoringDetector::default();

        // Test missing security logging
        let auth_code = r#"
            fn authenticate(username: &str, password: &str) -> Result<User, Error> {
                // No logging of authentication attempts
                let user = find_user(username)?;
                if !verify_password(&user.password_hash, password) {
                    return Err(Error::AuthenticationFailed);
                }
                Ok(user)
            }
        "#;

        let results = detector.detect(auth_code, "auth.rs");
        assert!(!results.is_empty());
        assert!(results[0]
            .security_issue
            .message
            .contains("Missing security event logging"));

        // Test with proper logging (should not trigger)
        let logged_auth_code = r#"
            fn authenticate(username: &str, password: &str) -> Result<User, Error> {
                info!("Authentication attempt for user: {}", username);
                let user = find_user(username)?;
                if !verify_password(&user.password_hash, password) {
                    warn!("Failed authentication for user: {}", username);
                    return Err(Error::AuthenticationFailed);
                }
                info!("Successful authentication for user: {}", username);
                Ok(user)
            }
        "#;

        let safe_results = detector.detect(logged_auth_code, "auth.rs");
        assert!(!safe_results.iter().any(|r| r
            .security_issue
            .message
            .contains("Missing security event logging")));
    }

    #[test]
    fn test_sensitive_data_in_logs() {
        let detector = SecurityLoggingMonitoringDetector::default();

        // Test sensitive data in logs
        let bad_logging = r#"
            fn process_login(request: &LoginRequest) -> Result<(), Error> {
                info!("Processing login for user: {} with password: {}", request.username, request.password);
                // ...
            }
        "#;

        let results = detector.detect(bad_logging, "auth.rs");
        assert!(!results.is_empty());
        assert!(results[0]
            .security_issue
            .message
            .contains("Sensitive data exposure in logs"));

        // Test with redacted logging (should not trigger)
        let safe_logging = r#"
            fn process_login(request: &LoginRequest) -> Result<(), Error> {
                info!("Processing login for user: {}", request.username);
                // ...
            }
        "#;

        let safe_results = detector.detect(safe_logging, "auth.rs");
        assert!(!safe_results.iter().any(|r| r
            .security_issue
            .message
            .contains("Sensitive data exposure in logs")));
    }
}

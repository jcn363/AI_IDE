//! Server-Side Request Forgery (SSRF) Detector
//!
//! This module implements detection for OWASP A10:2021 - Server-Side Request Forgery (SSRF).
//! It identifies patterns that could lead to SSRF vulnerabilities.

use super::*;

/// Detector for Server-Side Request Forgery (SSRF) vulnerabilities
#[derive(Debug, Default)]
pub struct SsrfDetector;

impl OWASPDetector for SsrfDetector {
    fn detect(&self, code: &str, file_path: &str) -> Vec<DetectionResult> {
        let mut findings = Vec::new();

        // Check for direct URL construction from user input
        if (code.contains("reqwest::get(") ||
            code.contains("ureq::get(") ||
            code.contains("hyper::Client::new()")) &&
           (code.contains("&str") || code.contains("String")) &&
           !code.contains("// Safe URL construction") &&
           !code.contains("SSRF-safe") {

            findings.push(self.create_finding(
                SecurityCategory::ServerSideRequestForgery,
                SecuritySeverity::High,
                "Potential SSRF vulnerability detected",
                file_path,
                "SSRF-001",
                "Unvalidated URL in HTTP Request",
                "The code constructs HTTP requests from unvalidated input, which could lead to SSRF vulnerabilities.",
                "Unvalidated URL in HTTP request",
                vec![
                    "Validate and sanitize all URLs before making requests",
                    "Use an allowlist of allowed domains or IP ranges",
                    "Implement proper URL parsing and validation",
                    "Consider using a dedicated HTTP client with SSRF protection",
                ],
                0.9,
            ));
        }

        // Check for DNS rebinding protection bypass
        if (code.contains("reqwest::") || code.contains("ureq::")) &&
           (code.contains("localhost") ||
            code.contains("127.0.0.1") ||
            code.contains("0.0.0.0") ||
            code.contains("[::1]")) &&
           !code.contains("// SSRF-safe") {

            findings.push(self.create_finding(
                SecurityCategory::ServerSideRequestForgery,
                SecuritySeverity::High,
                "Potential SSRF to localhost/internal services",
                file_path,
                "SSRF-002",
                "Localhost/Internal Service Access",
                "The code allows requests to localhost or internal services, which could be exploited in SSRF attacks.",
                "Request to localhost/internal service",
                vec![
                    "Block requests to internal IP ranges and localhost",
                    "Implement strict URL validation",
                    "Use network segmentation to limit access to internal services",
                    "Consider using a proxy with SSRF protection",
                ],
                0.85,
            ));
        }

        // Check for file:// protocol usage
        if (code.contains("file://") || code.contains("file:\\")) &&
           !code.contains("// Safe file access") {
            findings.push(self.create_finding(
                SecurityCategory::ServerSideRequestForgery,
                SecuritySeverity::Critical,
                "File protocol usage detected",
                file_path,
                "SSRF-003",
                "File Protocol Usage",
                "The code uses the file:// protocol which can be dangerous and lead to local file disclosure.",
                "file:// protocol usage",
                vec![
                    "Avoid using the file:// protocol with user-controlled input",
                    "If file access is necessary, validate and sanitize all paths",
                    "Use absolute paths and verify they're within allowed directories",
                    "Consider using a secure file access abstraction",
                ],
                0.95,
            ));
        }

        // Check for insecure URL parsing
        if (code.contains("Url::parse(") || code.contains("Uri::from_str")) &&
           !code.contains("// Validated URL") &&
           !code.contains("// SSRF-safe") {
            findings.push(self.create_finding(
                SecurityCategory::ServerSideRequestForgery,
                SecuritySeverity::Medium,
                "Insecure URL parsing detected",
                file_path,
                "SSRF-004",
                "Insecure URL Parsing",
                "URLs are being parsed without proper validation, which could lead to SSRF vulnerabilities.",
                "Unvalidated URL parsing",
                vec![
                    "Validate URLs before parsing",
                    "Use a whitelist of allowed schemes and domains",
                    "Implement proper URL normalization",
                    "Consider using a dedicated URL validation library",
                ],
                0.8,
            ));
        }

        findings
    }

    fn get_detector_name(&self) -> &'static str {
        "Server-Side Request Forgery (SSRF) Detector"
    }
}

impl SsrfDetector {
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
            security_issue: SecurityIssue {
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
            exploitability: self.calculate_exploitability(&category),
            impact: self.calculate_impact(&severity),
            ai_confidence: confidence,
            patterns_detected: vec![pattern.to_string()],
            remediation_steps: remediation_steps.into_iter().map(|s| s.to_string()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssrf_vulnerability() {
        let detector = SsrfDetector::default();

        // Test SSRF vulnerability
        let vulnerable_code = r#"
            fn fetch_url(url: &str) -> Result<String, Error> {
                let response = reqwest::blocking::get(url)?.text()?;
                Ok(response)
            }
        "#;

        let results = detector.detect(vulnerable_code, "api.rs");
        assert!(!results.is_empty());
        assert!(results[0].security_issue.message.contains("Potential SSRF"));

        // Test safe URL handling (should not trigger)
        let safe_code = r#"
            // SSRF-safe URL validation and fetching
            fn fetch_url(url: &str) -> Result<String, Error> {
                // Validate URL against allowed domains
                if !is_allowed_domain(url) {
                    return Err(Error::InvalidUrl);
                }

                let response = reqwest::blocking::Client::new()
                    .get(url)
                    .send()?
                    .text()?;

                Ok(response)
            }

            fn is_allowed_domain(url: &str) -> bool {
                // Implementation of domain whitelist check
                true
            }
        "#;

        let safe_results = detector.detect(safe_code, "api.rs");
        assert!(!safe_results.iter().any(|r| r.security_issue.message.contains("SSRF")));
    }

    #[test]
    fn test_localhost_access() {
        let detector = SsrfDetector::default();

        // Test localhost access
        let localhost_code = r#"
            fn check_local_service() -> Result<(), Error> {
                let response = reqwest::blocking::get("http://localhost:8080/status")?;
                // Process response
                Ok(())
            }
        "#;

        let results = detector.detect(localhost_code, "health_check.rs");
        assert!(!results.is_empty());
        assert!(results[0].security_issue.message.contains("localhost"));
    }

    #[test]
    test_file_protocol_detection() {
        let detector = SsrfDetector::default();

        // Test file:// protocol usage
        let file_code = r#"
            fn read_file(path: &str) -> Result<String, Error> {
                let url = format!("file://{}", path);
                let content = reqwest::blocking::get(&url)?.text()?;
                Ok(content)
            }
        "#;

        let results = detector.detect(file_code, "file_reader.rs");
        assert!(!results.is_empty());
        assert!(results[0].security_issue.message.contains("file://"));
    }
}

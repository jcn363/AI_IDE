//! Cryptographic Failures Detector
//!
//! This module implements detection for OWASP A02:2021 - Cryptographic Failures.
//! It identifies issues related to weak or missing cryptographic algorithms,
//! hardcoded secrets, and other cryptographic misconfigurations.

use super::*;

/// Detector for cryptographic failures
#[derive(Debug, Default)]
pub struct CryptographicFailuresDetector;

impl OWASPDetector for CryptographicFailuresDetector {
    fn detect(&self, code: &str, file_path: &str) -> Vec<DetectionResult> {
        let mut findings = Vec::new();

        // Check for weak hashing algorithms
        let weak_hash_patterns = [
            (
                r#"MD[0-9]"#,
                "MD5 hash function is cryptographically broken and should not be used",
            ),
            (
                r#"SHA1\(|\s|"|')"#,
                "SHA-1 is considered cryptographically broken for security-critical applications",
            ),
            (
                r#"DES\b"#,
                "DES encryption is considered weak and should not be used",
            ),
            (
                r#"RC4\b"#,
                "RC4 cipher is considered cryptographically broken",
            ),
        ];

        for (pattern, message) in &weak_hash_patterns {
            if let Some(caps) = regex::Regex::new(pattern).unwrap().captures(code) {
                findings.push(DetectionResult {
                    security_issue:    SecurityIssue {
                        category:  SecurityCategory::CryptographicFailures,
                        severity:  SecuritySeverity::High,
                        message:   message.to_string(),
                        file_path: file_path.to_string(),
                        line:      0, // Will be set by the caller
                        rule:      SecurityRule::new(
                            "CRYPTO-001".to_string(),
                            "Use of weak cryptographic primitive".to_string(),
                            "The code uses a cryptographic primitive that is considered weak or broken. Use stronger \
                             alternatives like SHA-256, SHA-3, or AES-256."
                                .to_string(),
                        ),
                    },
                    exploitability:    self.calculate_exploitability(&SecurityCategory::CryptographicFailures),
                    impact:            self.calculate_impact(&SecuritySeverity::High),
                    ai_confidence:     0.9,
                    patterns_detected: vec![caps[0].to_string()],
                    remediation_steps: vec![
                        "Replace with a stronger cryptographic primitive".to_string(),
                        "Use platform-recommended cryptographic libraries".to_string(),
                        "Ensure proper key management practices are followed".to_string(),
                    ],
                });
            }
        }

        // Check for hardcoded secrets
        let secret_patterns = [
            (
                r###"(password|pwd|secret|token|key|api[_-]?key|auth|credential)[\s=:]+['\"][a-zA-Z0-9]{12,}['\"]"###,
                "Hardcoded secret detected",
            ),
            (
                r###"(aws[_-]?access[_-]?key[_-]?id|aws[_-]?secret[_-]?access[_-]?key)[\s=:]+['\"][a-zA-Z0-9/+=]{20,}['\"]"###,
                "Hardcoded AWS credentials detected",
            ),
        ];

        for (pattern, message) in &secret_patterns {
            if let Some(caps) = regex::Regex::new(pattern).unwrap().captures(code) {
                findings.push(DetectionResult {
                    security_issue:    SecurityIssue {
                        category:  SecurityCategory::CryptographicFailures,
                        severity:  SecuritySeverity::Critical,
                        message:   message.to_string(),
                        file_path: file_path.to_string(),
                        line:      0,
                        rule:      SecurityRule::new(
                            "SECRET-001".to_string(),
                            "Hardcoded secret detected".to_string(),
                            "The code contains what appears to be a hardcoded secret or credential. This is a \
                             security risk as it can be easily exposed."
                                .to_string(),
                        ),
                    },
                    exploitability:    self.calculate_exploitability(&SecurityCategory::CryptographicFailures),
                    impact:            self.calculate_impact(&SecuritySeverity::Critical),
                    ai_confidence:     0.95,
                    patterns_detected: vec![caps[0].to_string()],
                    remediation_steps: vec![
                        "Remove hardcoded secrets from source code".to_string(),
                        "Use environment variables or secure secret management systems".to_string(),
                        "Consider using a secrets management solution like HashiCorp Vault or AWS Secrets Manager"
                            .to_string(),
                    ],
                });
            }
        }

        // Check for weak random number generation
        if code.contains("rand()") || code.contains("random()") || code.contains("Math.random()") {
            findings.push(DetectionResult {
                security_issue:    SecurityIssue {
                    category:  SecurityCategory::CryptographicFailures,
                    severity:  SecuritySeverity::High,
                    message:   "Insecure random number generation detected".to_string(),
                    file_path: file_path.to_string(),
                    line:      0,
                    rule:      SecurityRule::new(
                        "CRYPTO-002".to_string(),
                        "Insecure random number generation".to_string(),
                        "The code uses an insecure random number generator for cryptographic purposes. Use a \
                         cryptographically secure random number generator (CSPRNG) instead."
                            .to_string(),
                    ),
                },
                exploitability:    self.calculate_exploitability(&SecurityCategory::CryptographicFailures),
                impact:            self.calculate_impact(&SecuritySeverity::High),
                ai_confidence:     0.85,
                patterns_detected: vec!["rand() or random() usage".to_string()],
                remediation_steps: vec![
                    "Use a cryptographically secure random number generator (CSPRNG)".to_string(),
                    "In Rust, use the `rand` crate with a secure RNG like `rand::rngs::OsRng`".to_string(),
                    "For tokens or session IDs, use a library that generates cryptographically secure random strings"
                        .to_string(),
                ],
            });
        }

        findings
    }

    fn get_detector_name(&self) -> &'static str {
        "Cryptographic Failures Detector"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_hash_detection() {
        let detector = CryptographicFailuresDetector::default();
        let code = r#"
            let hash = md5::compute("password");
            let sha1_hash = sha1::Sha1::from("test");
            let des = DES::new(key);
        "#;

        let results = detector.detect(code, "test.rs");
        assert_eq!(results.len(), 3);
        assert!(results[0].security_issue.message.contains("MD5"));
        assert!(results[1].security_issue.message.contains("SHA-1"));
        assert!(results[2].security_issue.message.contains("DES"));
    }

    #[test]
    fn test_hardcoded_secrets() {
        let detector = CryptographicFailuresDetector::default();
        let code = r#"
            const API_KEY = "AKIAIOSFODNN7EXAMPLE";
            let password = "s3cr3tP@ssw0rd";
        "#;

        let results = detector.detect(code, "test.rs");
        assert!(!results.is_empty());
        assert!(results
            .iter()
            .any(|r| r.security_issue.message.contains("Hardcoded")));
    }
}

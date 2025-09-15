//! Software and Data Integrity Failures Detector
//!
//! This module implements detection for OWASP A08:2021 - Software and Data Integrity Failures.
//! It identifies issues related to integrity verification of software and data.

use super::*;

/// Detector for software and data integrity failures
#[derive(Debug, Default)]
pub struct SoftwareDataIntegrityDetector;

impl OWASPDetector for SoftwareDataIntegrityDetector {
    fn detect(&self, code: &str, file_path: &str) -> Vec<DetectionResult> {
        let mut findings = Vec::new();

        // Check for missing integrity checks on downloaded files
        if (code.contains("reqwest::get") || code.contains("ureq::get")) &&
           !code.contains("verify") && !code.contains("checksum") {
            findings.push(self.create_finding(
                SecurityCategory::SoftwareDataIntegrityFailures,
                SecuritySeverity::High,
                "Missing integrity check on downloaded content",
                file_path,
                "INTEGRITY-001",
                "Missing File Integrity Check",
                "Downloaded files should be verified using checksums or digital signatures.",
                "Download without verification",
                vec![
                    "Verify file checksums after download",
                    "Use digital signatures to verify authenticity",
                    "Consider using package managers that handle verification",
                ],
                0.9,
            ));
        }

        // Check for insecure deserialization
        if code.contains("serde_json::from_str") ||
           code.contains("bincode::deserialize") ||
           code.contains("rmp_serde::from_read") {

            // Check if there's any validation or schema checking
            let has_validation = code.contains("serde_valid") ||
                                code.contains("validator") ||
                                code.contains("schema") ||
                                code.contains("validate");

            if !has_validation {
                findings.push(self.create_finding(
                    SecurityCategory::SoftwareDataIntegrityFailures,
                    SecuritySeverity::High,
                    "Insecure deserialization detected",
                    file_path,
                    "SER-001",
                    "Insecure Deserialization",
                    "Untrusted data is being deserialized without proper validation, which can lead to remote code execution.",
                    "Unvalidated deserialization",
                    vec![
                        "Validate all input before deserialization",
                        "Use strongly typed objects instead of generic deserialization",
                        "Consider using schema validation libraries",
                        "Implement allowlists for expected types",
                    ],
                    0.95,
                ));
            }
        }

        // Check for missing dependency verification
        if file_path.ends_with("Cargo.toml") &&
           !code.contains("[package.metadata.docs.rs]\nrustdoc-args = ["--cfg", "docsrs"]") {
            findings.push(self.create_finding(
                SecurityCategory::SoftwareDataIntegrityFailures,
                SecuritySeverity::Medium,
                "Missing dependency verification in build process",
                file_path,
                "DEPS-001",
                "Missing Dependency Verification",
                "The build process does not verify the integrity of dependencies.",
                "Missing dependency verification",
                vec![
                    "Use Cargo.lock to pin dependency versions",
                    "Consider using cargo-audit to check for vulnerable dependencies",
                    "Implement reproducible builds",
                ],
                0.8,
            ));
        }

        // Check for missing file permissions
        if (code.contains("std::fs::File::create") || code.contains("std::fs::set_permissions")) &&
           !code.contains("0o600") && !code.contains("0o400") {
            findings.push(self.create_finding(
                SecurityCategory::SoftwareDataIntegrityFailures,
                SecuritySeverity::Medium,
                "Insecure file permissions detected",
                file_path,
                "PERM-001",
                "Insecure File Permissions",
                "Files are being created with overly permissive access rights.",
                "Insecure file permissions",
                vec![
                    "Set restrictive file permissions (e.g., 0o600 for sensitive files)",
                    "Avoid world-writable files",
                    "Set appropriate umask before creating files",
                ],
                0.85,
            ));
        }

        findings
    }

    fn get_detector_name(&self) -> &'static str {
        "Software and Data Integrity Failures Detector"
    }
}

impl SoftwareDataIntegrityDetector {
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
    fn test_insecure_deserialization() {
        let detector = SoftwareDataIntegrityDetector::default();

        // Test insecure deserialization
        let insecure_code = r#"
            let data: Value = serde_json::from_str(untrusted_input)?;
            // Process untrusted data without validation
        "#;

        let results = detector.detect(insecure_code, "deserialize.rs");
        assert!(!results.is_empty());
        assert!(results[0].security_issue.message.contains("Insecure deserialization"));

        // Test secure deserialization with validation (should not trigger)
        let secure_code = r#"
            #[derive(serde::Deserialize, validator::Validate)]
            struct UserInput {
                #[validate(length(min = 1, max = 100))]
                name: String,
                #[validate(range(min = 1, max = 120))]
                age: u8,
            }

            let input: UserInput = serde_json::from_str(untrusted_input)?;
            input.validate()?;
        "#;

        let safe_results = detector.detect(secure_code, "deserialize.rs");
        assert!(!safe_results.iter().any(|r| r.security_issue.message.contains("Insecure deserialization")));
    }

    #[test]
    fn test_missing_file_integrity() {
        let detector = SoftwareDataIntegrityDetector::default();

        // Test download without verification
        let download_code = r#"
            let response = reqwest::get("https://example.com/download")?;
            let content = response.bytes()?;
            std::fs::write("downloaded_file", content)?;
        "#;

        let results = detector.detect(download_code, "download.rs");
        assert!(!results.is_empty());
        assert!(results[0].security_issue.message.contains("Missing integrity check"));

        // Test download with verification (should not trigger)
        let secure_download = r#"
            // Download file
            let response = reqwest::get("https://example.com/file")?;
            let content = response.bytes()?;

            // Download checksum
            let checksum_response = reqwest::get("https://example.com/file.sha256")?;
            let expected_checksum = checksum_response.text()?;

            // Verify checksum
            let actual_checksum = sha2::Sha256::digest(&content);
            if format!("{:x}", actual_checksum) != expected_checksum.trim() {
                return Err("Checksum verification failed".into());
            }

            std::fs::write("downloaded_file", content)?;
        "#;

        let safe_results = detector.detect(secure_download, "download.rs");
        assert!(!safe_results.iter().any(|r| r.security_issue.message.contains("Missing integrity check")));
    }
}

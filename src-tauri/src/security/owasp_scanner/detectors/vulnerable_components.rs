//! Vulnerable Components Detector
//!
//! This module implements detection for OWASP A06:2021 - Vulnerable and Outdated Components.
//! It identifies the use of known vulnerable dependencies and outdated libraries.

use std::collections::HashSet;

use super::*;

/// Detector for vulnerable and outdated components
#[derive(Debug, Default)]
pub struct VulnerableComponentsDetector {
    /// Known vulnerable crates and their versions
    vulnerable_crates: Vec<(&'static str, &'static str, &'static str, SecuritySeverity)>,
}

impl Default for VulnerableComponentsDetector {
    fn default() -> Self {
        let mut detector = VulnerableComponentsDetector {
            vulnerable_crates: Vec::new(),
        };

        // Add known vulnerable crates (this would ideally come from a database)
        detector.vulnerable_crates.extend(vec![
            // Format: (crate_name, vulnerable_version, advisory_url, severity)
            (
                "serde",
                "<1.0.130",
                "https://rustsec.org/advisories/RUSTSEC-2021-0123",
                SecuritySeverity::High,
            ),
            (
                "tokio",
                "<1.8.0",
                "https://rustsec.org/advisories/RUSTSEC-2021-0124",
                SecuritySeverity::Critical,
            ),
            (
                "regex",
                "<1.5.4",
                "https://rustsec.org/advisories/RUSTSEC-2022-0013",
                SecuritySeverity::High,
            ),
            (
                "time",
                "<0.3.0",
                "https://rustsec.org/advisories/RUSTSEC-2020-0071",
                SecuritySeverity::Medium,
            ),
            (
                "smallvec",
                "<1.6.1",
                "https://rustsec.org/advisories/RUSTSEC-2021-0003",
                SecuritySeverity::High,
            ),
        ]);

        detector
    }
}

impl OWASPDetector for VulnerableComponentsDetector {
    fn detect(&self, code: &str, file_path: &str) -> Vec<DetectionResult> {
        let mut findings = Vec::new();

        // Check for Cargo.toml or Cargo.lock files
        if file_path.ends_with("Cargo.toml") || file_path.ends_with("Cargo.lock") {
            // This is a simplified check - in a real implementation, you would parse the TOML
            // and check against a vulnerability database
            for (crate_name, version, advisory_url, severity) in &self.vulnerable_crates {
                if code.contains(&format!("{} = \"{}\"", crate_name, version))
                    || code.contains(&format!("{} = {{ version = \"{}", crate_name, version))
                {
                    let message = format!("Vulnerable dependency detected: {} {}", crate_name, version);
                    let advisory = format!("Advisory: {}", advisory_url);

                    findings.push(DetectionResult {
                        security_issue:    SecurityIssue {
                            category:  SecurityCategory::VulnerableComponents,
                            severity:  severity.clone(),
                            message:   message.clone(),
                            file_path: file_path.to_string(),
                            line:      0,
                            rule:      SecurityRule::new(
                                format!("VULN-{}", crate_name.to_uppercase()),
                                format!("Vulnerable {} version", crate_name),
                                format!(
                                    "The version of {} in use has known security vulnerabilities. {}",
                                    crate_name, advisory_url
                                ),
                            ),
                        },
                        exploitability:    self.calculate_exploitability(&SecurityCategory::VulnerableComponents),
                        impact:            self.calculate_impact(severity),
                        ai_confidence:     0.95,
                        patterns_detected: vec![format!("{} = \"{}\"", crate_name, version)],
                        remediation_steps: vec![
                            format!("Upgrade {} to the latest stable version", crate_name),
                            format!("Review the security advisory at: {}", advisory_url),
                            "Run `cargo update` to update dependencies".to_string(),
                            "Consider using `cargo audit` to check for known vulnerabilities".to_string(),
                        ],
                    });
                }
            }
        }

        findings
    }

    fn get_detector_name(&self) -> &'static str {
        "Vulnerable Components Detector"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulnerable_dependency_detection() {
        let detector = VulnerableComponentsDetector::default();

        // Test with a vulnerable version
        let cargo_toml = r#"
            [dependencies]
            serde = "1.0.120"  # Vulnerable version
            tokio = { version = "1.7.0", features = ["full"] }
        "#;

        let results = detector.detect(cargo_toml, "Cargo.toml");
        assert!(!results.is_empty());
        assert!(results
            .iter()
            .any(|r| r.security_issue.message.contains("serde")));
        assert!(results
            .iter()
            .any(|r| r.security_issue.message.contains("tokio")));

        // Test with a non-vulnerable version
        let safe_cargo_toml = r#"
            [dependencies]
            serde = "1.0.130"  # Safe version
        "#;

        let safe_results = detector.detect(safe_cargo_toml, "Cargo.toml");
        assert!(
            !safe_results
                .iter()
                .any(|r| r.security_issue.message.contains("serde")),
            "Should not flag non-vulnerable versions"
        );
    }
}

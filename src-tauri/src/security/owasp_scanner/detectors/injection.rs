//! Injection Detector
//!
//! This module implements detection for OWASP A03:2021 - Injection.
//! It identifies potential injection vulnerabilities such as SQL injection,
//! command injection, and other injection flaws.

use super::*;

/// Detector for injection vulnerabilities
#[derive(Debug, Default)]
pub struct InjectionDetector;

impl OWASPDetector for InjectionDetector {
    fn detect(&self, code: &str, file_path: &str) -> Vec<DetectionResult> {
        let mut findings = Vec::new();

        // Check for potential SQL injection patterns
        let sql_patterns = [
            (r"(?i)SELECT\s+\*\s+FROM\s+\w+\s+WHERE\s+\w+\s*=\s*[\"']?\$?\{\w+\}[\"']?", "Potential SQL injection - string concatenation in SQL query"),
            (r"(?i)INSERT\s+INTO\s+\w+\s+VALUES\s*\([^)]*\$?\{\w+\}[^)]*\)", "Potential SQL injection - dynamic values in INSERT statement"),
            (r"(?i)UPDATE\s+\w+\s+SET\s+\w+\s*=\s*[\"']?\$?\{\w+\}[\"']?", "Potential SQL injection - dynamic values in UPDATE statement"),
            (r"(?i)DELETE\s+FROM\s+\w+\s+WHERE\s+\w+\s*=\s*[\"']?\$?\{\w+\}[\"']?", "Potential SQL injection - dynamic values in DELETE statement"),
        ];

        for (pattern, message) in &sql_patterns {
            if let Some(caps) = regex::Regex::new(pattern).unwrap().captures(code) {
                findings.push(self.create_finding(
                    SecurityCategory::Injection,
                    SecuritySeverity::High,
                    message,
                    file_path,
                    "SQLI-001",
                    "Potential SQL Injection",
                    "The code appears to construct SQL queries using string concatenation with user input, which can lead to SQL injection vulnerabilities.",
                    &caps[0],
                    vec![
                        "Use parameterized queries or prepared statements instead of string concatenation",
                        "Consider using an ORM that provides built-in protection against SQL injection",
                        "Validate and sanitize all user input before using it in database queries",
                    ],
                    0.85,
                ));
            }
        }

        // Check for command injection patterns
        let cmd_patterns = [
            (r"(?i)std::process::Command::new\([^)]*\$?\{\w+\}[^)]*\)", "Potential command injection - dynamic command construction"),
            (r"(?i)std::process::Command::arg\([^)]*\$?\{\w+\}[^)]*\)", "Potential command injection - dynamic command arguments"),
            (r"(?i)std::process::Command::args\([^)]*\$?\{\w+\}[^)]*\)", "Potential command injection - dynamic command arguments"),
        ];

        for (pattern, message) in &cmd_patterns {
            if let Some(caps) = regex::Regex::new(pattern).unwrap().captures(code) {
                findings.push(self.create_finding(
                    SecurityCategory::Injection,
                    SecuritySeverity::Critical,
                    message,
                    file_path,
                    "CMDI-001",
                    "Potential Command Injection",
                    "The code constructs system commands using untrusted input, which can lead to command injection vulnerabilities.",
                    &caps[0],
                    vec![
                        "Avoid constructing commands from user input when possible",
                        "If command construction is necessary, validate and sanitize all input",
                        "Use allowlists for command arguments when possible",
                        "Consider using higher-level abstractions instead of direct command execution",
                    ],
                    0.9,
                ));
            }
        }

        // Check for template injection
        if code.contains("render_template") && (code.contains("{{ ") || code.contains("{% ") || code.contains("${ ")) {
            findings.push(self.create_finding(
                SecurityCategory::Injection,
                SecuritySeverity::High,
                "Potential template injection - dynamic template rendering with user input",
                file_path,
                "TMPLI-001",
                "Potential Template Injection",
                "The code appears to render templates with user-controlled input, which can lead to server-side template injection.",
                "render_template with dynamic content",
                vec![
                    "Avoid using user input directly in templates",
                    "Use template engines that automatically escape variables by default",
                    "Sandbox template rendering when user input must be included",
                ],
                0.8,
            ));
        }

        findings
    }

    fn get_detector_name(&self) -> &'static str {
        "Injection Detector"
    }
}

impl InjectionDetector {
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
    fn test_sql_injection_detection() {
        let detector = InjectionDetector::default();
        let code = r#"
            let query = format!("SELECT * FROM users WHERE username = '{}'", user_input);
            let result = conn.execute(&query);
        "#;

        let results = detector.detect(code, "test.rs");
        assert!(!results.is_empty());
        assert!(results[0].security_issue.message.contains("SQL injection"));
    }

    #[test]
    fn test_command_injection_detection() {
        let detector = InjectionDetector::default();
        let code = r#"
            let output = Command::new("ls")
                .arg("-la")
                .arg(user_input)
                .output()?;
        "#;

        let results = detector.detect(code, "test.rs");
        assert!(!results.is_empty());
        assert!(results[0].security_issue.message.contains("command injection"));
    }
}

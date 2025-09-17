use std::collections::HashMap;

use regex::Regex;
use syn::visit::Visit;
use syn::*;
use uuid::Uuid;

use crate::analysis::types::*;
use crate::error_handling::{AnalysisConfig, AnalysisResult};

/// Security scanner for detecting vulnerabilities
#[derive(Clone, Debug)]
pub struct SecurityScanner {
    rules: HashMap<String, SecurityRule>,
    config: AnalysisConfig,
}

#[derive(Clone, Debug)]
pub struct SecurityRule {
    pub pattern: Regex,
    pub category: SecurityCategory,
    pub severity: Severity,
    pub description: String,
    pub mitigation: String,
}

impl SecurityScanner {
    /// Create a new security scanner with default rules
    pub fn new() -> Self {
        Self::with_config(AnalysisConfig::default())
    }

    /// Create a new security scanner with custom configuration
    pub fn with_config(config: AnalysisConfig) -> Self {
        let mut scanner = Self {
            rules: HashMap::new(),
            config,
        };
        scanner.load_default_rules();
        scanner
    }

    /// Load default security rules
    fn load_default_rules(&mut self) {
        // SQL Injection patterns
        self.add_rule(
            "sql_concat",
            SecurityRule {
                pattern: Regex::new(r#"(?i)(select|insert|update|delete).*\+\s*".*\$\{.*?\}"#)
                    .unwrap(),
                category: SecurityCategory::Injection,
                severity: Severity::Critical,
                description: "Potential SQL injection through string concatenation".to_string(),
                mitigation: "Use parameterized queries or prepared statements".to_string(),
            },
        );

        // XSS in HTML generation
        self.add_rule(
            "xss_innerHTML",
            SecurityRule {
                pattern: Regex::new(r#"(?i)innerHTML\s*\+=\s*".*\$\{.*?\}"#).unwrap(),
                category: SecurityCategory::InputValidation,
                severity: Severity::Error,
                description: "Potential XSS vulnerability in HTML manipulation".to_string(),
                mitigation: "Use DOM methods or sanitize input with a trusted library".to_string(),
            },
        );

        // Unsafe code patterns
        self.add_rule(
            "use_after_free",
            SecurityRule {
                pattern: Regex::new(r#"(?i)Rc|Arc|Box).*drop.*\1"#).unwrap(),
                category: SecurityCategory::Memory,
                severity: Severity::Warning,
                description: "Potential use-after-free vulnerability".to_string(),
                mitigation: "Ensure proper lifetime management and memory safety".to_string(),
            },
        );

        // Cryptographic issues
        self.add_rule(
            "weak_randomness",
            SecurityRule {
                pattern: Regex::new(r#"(?i)rand::thread_rng\(\)"#).unwrap(),
                category: SecurityCategory::Cryptography,
                severity: Severity::Warning,
                description: "Using weak randomness for sensitive operations".to_string(),
                mitigation: "Use cryptographically secure random number generators".to_string(),
            },
        );

        // Race conditions
        self.add_rule(
            "race_condition",
            SecurityRule {
                pattern: Regex::new(r#"(?i)(&mut|&)\s+(Arc|Rc)\s*<.*>\s*,.*,.*&"#).unwrap(),
                category: SecurityCategory::RaceConditions,
                severity: Severity::Error,
                description: "Potential race condition detected".to_string(),
                mitigation: "Use proper synchronization primitives or message passing".to_string(),
            },
        );

        // Command injection
        self.add_rule(
            "command_injection",
            SecurityRule {
                pattern: Regex::new(r#"Command::new.*\+.*\$"#).unwrap(),
                category: SecurityCategory::Injection,
                severity: Severity::Critical,
                description: "Potential command injection vulnerability".to_string(),
                mitigation: "Use validated input or safe alternatives".to_string(),
            },
        );

        // Hardcoded passwords
        self.add_rule(
            "hardcoded_password",
            SecurityRule {
                pattern: Regex::new(r#"(?i)(password|passwd|secret).*=\s*".*"#).unwrap(),
                category: SecurityCategory::Configuration,
                severity: Severity::Warning,
                description: "Hardcoded secrets detected".to_string(),
                mitigation: "Use environment variables or secure key management".to_string(),
            },
        );
    }

    /// Add a custom security rule
    pub fn add_rule(&mut self, name: &str, rule: SecurityRule) {
        self.rules.insert(name.to_string(), rule);
    }

    /// Remove a security rule
    pub fn remove_rule(&mut self, name: &str) -> bool {
        self.rules.remove(name).is_some()
    }

    /// Scan source code for security vulnerabilities
    pub fn scan_code(&self, content: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for line_no in 0..content.lines().count() {
            if let Some(line) = content.lines().nth(line_no) {
                for (_rule_name, rule) in &self.rules {
                    let captures = rule.pattern.find_iter(line);
                    for capture in captures {
                        // Regex match capture is available, proceed with analysis
                        let issue = SecurityIssue {
                            id: Uuid::new_v4(),
                            cwe_id: self.map_to_cwe(&rule.category),
                            title: rule.description.clone(),
                            description: format!(
                                "Security vulnerability found: {}",
                                rule.description
                            ),
                            severity: rule.severity,
                            location: Location {
                                file: file_path.to_string(),
                                line: line_no + 1,
                                column: capture.start(),
                                offset: capture.start(),
                            },
                            evidence: format!("Found '{}'", capture.as_str()),
                            mitigation: rule.mitigation.clone(),
                            category: rule.category.clone(),
                        };
                        issues.push(issue);
                    }
                }
            }
        }

        issues
    }

    /// Scan AST for security vulnerabilities
    pub async fn scan(&self, ast: &File) -> AnalysisResult<Vec<SecurityIssue>> {
        let mut issues = Vec::new();

        // Convert AST to string for pattern matching
        let ast_string = quote::quote!(#ast).to_string();

        // Scan the AST as string
        issues.extend(self.scan_code(&ast_string, "AST"));

        // Additional AST-specific checks
        issues.extend(self.scan_unsafe_blocks(ast));
        issues.extend(self.scan_exposed_secrets(ast));
        issues.extend(self.scan_insecure_configs(ast));

        Ok(issues)
    }

    /// Check for unsafe blocks
    fn scan_unsafe_blocks(&self, ast: &File) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();
        let mut visitor = UnsafeBlockVisitor {
            issues: &mut issues,
            file: "AST",
        };
        visitor.visit_file(ast);
        issues
    }

    /// Check for exposed secrets in the code
    fn scan_exposed_secrets(&self, ast: &File) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();
        let mut visitor = SecretVulnerabilityVisitor {
            issues: &mut issues,
            file: "AST",
        };
        visitor.visit_file(ast);
        issues
    }

    /// Check for insecure configurations
    fn scan_insecure_configs(&self, ast: &File) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();
        let mut visitor = ConfigVulnerabilityVisitor {
            issues: &mut issues,
            file: "AST",
        };
        visitor.visit_file(ast);
        issues
    }

    /// Map security category to CWE ID
    fn map_to_cwe(&self, category: &SecurityCategory) -> Option<String> {
        match category {
            SecurityCategory::Injection => Some("CWE-79".to_string()), // XSS as example
            SecurityCategory::Authentication => Some("CWE-287".to_string()),
            SecurityCategory::Authorization => Some("CWE-285".to_string()),
            SecurityCategory::Cryptography => Some("CWE-327".to_string()),
            SecurityCategory::InputValidation => Some("CWE-20".to_string()),
            SecurityCategory::Configuration => Some("CWE-16".to_string()),
            SecurityCategory::DenialOfService => Some("CWE-400".to_string()),
            SecurityCategory::RaceConditions => Some("CWE-1315".to_string()),
            SecurityCategory::Memory => Some("CWE-416".to_string()),
            _ => None,
        }
    }

    /// Get available security rules
    pub fn get_rules(&self) -> Vec<String> {
        self.rules.keys().cloned().collect()
    }
}

/// AST visitor for unsafe block detection
#[derive(Debug)]
struct UnsafeBlockVisitor<'a> {
    issues: &'a mut Vec<SecurityIssue>,
    file: &'a str,
}

impl<'a, 'ast> Visit<'ast> for UnsafeBlockVisitor<'a> {
    fn visit_expr_unsafe(&mut self, node: &'ast ExprUnsafe) {
        let issue = SecurityIssue {
            id: Uuid::new_v4(),
            cwe_id: Some("CWE-120".to_string()),
            title: "Unsafe block detected".to_string(),
            description: "Usage of unsafe block - requires manual safety verification".to_string(),
            severity: Severity::Warning,
            location: Location {
                file: self.file.to_string(),
                // Using simplified span info for token-level spans
                line: 0, // AST nodes don't have reliable line info from span
                column: 0,
                offset: 0,
            },
            evidence: "unsafe { ... }".to_string(),
            mitigation: "Review unsafe code thoroughly for memory safety".to_string(),
            category: SecurityCategory::Memory,
        };
        self.issues.push(issue);
        syn::visit::visit_expr_unsafe(self, node);
    }
}

/// AST visitor for secret exposure detection
struct SecretVulnerabilityVisitor<'a> {
    issues: &'a mut Vec<SecurityIssue>,
    file: &'a str,
}

impl<'a, 'ast> Visit<'ast> for SecretVulnerabilityVisitor<'a> {
    fn visit_lit_str(&mut self, node: &'ast LitStr) {
        let value = node.value();
        let lower = value.to_lowercase();

        if (lower.contains("password") || lower.contains("secret") || lower.contains("token"))
            && !lower.contains("env::var")
        {
            let issue = SecurityIssue {
                id: Uuid::new_v4(),
                cwe_id: Some("CWE-798".to_string()),
                title: "Hardcoded secret detected".to_string(),
                description: "Potential hardcoded secret or credential".to_string(),
                severity: Severity::Error,
                location: Location {
                    file: self.file.to_string(),
                    line: 0, // AST nodes don't have reliable line info from span
                    column: 0,
                    offset: 0,
                },
                evidence: format!("Literal: {}", value.chars().take(20).collect::<String>()),
                mitigation: "Move secrets to environment variables or secure config".to_string(),
                category: SecurityCategory::Configuration,
            };
            self.issues.push(issue);
        }

        syn::visit::visit_lit_str(self, node);
    }
}

/// AST visitor for insecure configuration detection
struct ConfigVulnerabilityVisitor<'a> {
    issues: &'a mut Vec<SecurityIssue>,
    file: &'a str,
}

impl<'a, 'ast> Visit<'ast> for ConfigVulnerabilityVisitor<'a> {
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Expr::Path(ref func) = *node.func {
            // Check for insecure HTTP configurations
            if func.path.segments.iter().any(|seg| {
                seg.ident == "allow_http"
                    || seg.ident == "danger_accept_invalid_certs"
                    || seg.ident == "disable_ssl_verify"
            }) {
                let issue = SecurityIssue {
                    id: Uuid::new_v4(),
                    cwe_id: Some("CWE-16".to_string()),
                    title: "Insecure configuration detected".to_string(),
                    description: "Security-critical configuration that may allow attacks"
                        .to_string(),
                    severity: Severity::Critical,
                    location: Location {
                        file: self.file.to_string(),
                        line: 0, // AST nodes don't have reliable line info from span
                        column: 0,
                        offset: 0,
                    },
                    evidence: "Insecure config call".to_string(),
                    mitigation: "Review security implications of this configuration".to_string(),
                    category: SecurityCategory::Configuration,
                };
                self.issues.push(issue);
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_injection_detection() {
        let scanner = SecurityScanner::new();
        let code =
            r#"let query = "SELECT * FROM users WHERE id = ".to_string() + &user_input + ";"#;

        let issues = scanner.scan_code(code, "test.rs");

        assert!(issues
            .iter()
            .any(|i| i.category == SecurityCategory::Injection));
    }

    #[test]
    fn test_hardcoded_password_detection() {
        let scanner = SecurityScanner::new();
        let code = r#"const PASSWORD: &str = "secret123";"#;

        let issues = scanner.scan_code(code, "test.rs");

        assert!(issues.iter().any(|i| i.title.contains("Hardcoded secret")));
    }

    #[test]
    fn test_weak_randomness_detection() {
        let scanner = SecurityScanner::new();
        let code = r#"let random = rand::thread_rng().next_u32();"#;

        let issues = scanner.scan_code(code, "test.rs");

        assert!(issues.iter().any(|i| i.title.contains("weak randomness")));
    }

    #[tokio::test]
    async fn test_ast_scanning() {
        let scanner = SecurityScanner::new();
        let code = r#"unsafe { std::ptr::null_mut::<i32>().write(42); }"#;
        let ast = syn::parse_file(code).unwrap();

        let issues = scanner.scan(&ast).await.unwrap();

        assert!(issues.iter().any(|i| i.title.contains("Unsafe block")));
    }
}

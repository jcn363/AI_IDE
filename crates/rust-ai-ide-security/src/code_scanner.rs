use std::collections::HashMap;
use std::path::{Path, PathBuf};

use regex::Regex;
use syn::visit::Visit;
use syn::{ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, Spanned};
use walkdir::WalkDir;

use super::bin::owasp_scanner::{Finding, Severity};

/// Scans Rust source code for security issues
pub async fn scan_code(
    path: &std::path::PathBuf,
    min_severity: &str,
    limit: usize,
) -> Result<Vec<Finding>, anyhow::Error> {
    let min_severity: Severity = min_severity.parse().unwrap_or(Severity::Medium);
    let mut findings = Vec::new();

    // Collect all Rust files
    let mut files = Vec::new();
    if path.is_dir() {
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                !e.file_type().is_dir()
                && e.path().extension().map_or(false, |ext| ext == "rs")
                // Skip target directory
                && !e.path().to_string_lossy().contains("/target/")
            })
        {
            files.push(entry.path().to_path_buf());
        }
    } else if path.extension().map_or(false, |ext| ext == "rs") {
        files.push(path.clone());
    }

    // Process each file
    for file_path in files {
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            // Check for unsafe code
            if content.contains("unsafe") {
                let mut unsafe_blocks = Vec::new();
                let mut line_num = 0;

                for (i, line) in content.lines().enumerate() {
                    if line.trim().starts_with("unsafe") {
                        unsafe_blocks.push(i + 1);
                    }
                    line_num = i + 1;
                }

                if !unsafe_blocks.is_empty() {
                    findings.push(Finding {
                        id:             "unsafe-code".to_string(),
                        title:          "Unsafe code block detected".to_string(),
                        description:    "The code contains unsafe Rust blocks which can lead to memory safety issues."
                            .to_string(),
                        severity:       Severity::High,
                        file:           file_path.display().to_string(),
                        line:           Some(unsafe_blocks[0] as u32),
                        column:         Some(1),
                        category:       "Memory Safety".to_string(),
                        remediation:    "Review and minimize the use of unsafe code. Ensure proper bounds checking \
                                         and validation."
                            .to_string(),
                        cwe_id:         Some(119), /* CWE-119: Improper Restriction of Operations within the Bounds
                                                    * of a Memory Buffer */
                        owasp_category: Some("A08:2021-Software and Data Integrity Failures".to_string()),
                        metadata:       [
                            (
                                "lines".to_string(),
                                unsafe_blocks
                                    .iter()
                                    .map(|n| n.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", "),
                            ),
                            ("total_lines".to_string(), line_num.to_string()),
                        ]
                        .iter()
                        .cloned()
                        .collect(),
                        source:         "owasp-scanner".to_string(),
                    });
                }
            }

            // Check for common security issues using regex patterns
            let patterns = get_security_patterns();
            for (i, line) in content.lines().enumerate() {
                for (pattern, rule) in &patterns {
                    if pattern.is_match(line) {
                        // Skip if the finding's severity is below our threshold
                        if rule.severity < min_severity {
                            continue;
                        }

                        findings.push(Finding {
                            id:             rule.id.clone(),
                            title:          rule.title.clone(),
                            description:    rule.description.clone(),
                            severity:       rule.severity.clone(),
                            file:           file_path.display().to_string(),
                            line:           Some(i as u32 + 1),
                            column:         Some(1),
                            category:       rule.category.clone(),
                            remediation:    rule.remediation.clone(),
                            cwe_id:         rule.cwe_id,
                            owasp_category: rule.owasp_category.clone(),
                            metadata:       HashMap::new(),
                            source:         "owasp-scanner".to_string(),
                        });
                    }
                }
            }

            // Parse with syn for more complex analysis
            if let Ok(syntax) = syn::parse_file(&content) {
                let mut visitor = SecurityVisitor::new(&file_path);
                visitor.visit_file(&syntax);
                findings.extend(visitor.findings);
            }
        }
    }

    // Limit the number of findings
    if findings.len() > limit {
        findings.truncate(limit);
    }

    Ok(findings)
}

/// Security patterns to scan for in code
#[derive(Clone)]
struct SecurityPattern {
    id:             String,
    title:          String,
    description:    String,
    severity:       Severity,
    category:       String,
    remediation:    String,
    cwe_id:         Option<u32>,
    owasp_category: Option<String>,
}

/// Get all security patterns to scan for
fn get_security_patterns() -> Vec<(Regex, SecurityPattern)> {
    let mut patterns = Vec::new();

    // Hardcoded credentials
    let pattern = Regex::new(r#"(password|secret|key|token|api[_-]?key|auth|credential)[\s=:]+['\"].*?['\"]"#).unwrap();
    patterns.push((pattern, SecurityPattern {
        id:             "hardcoded-credential".to_string(),
        title:          "Hardcoded Credential".to_string(),
        description:    "The code contains what appears to be a hardcoded credential or secret.".to_string(),
        severity:       Severity::Critical,
        category:       "Authentication".to_string(),
        remediation:    "Remove hardcoded credentials and use secure configuration management or secret management \
                         solution."
            .to_string(),
        cwe_id:         Some(798), // CWE-798: Use of Hard-coded Credentials
        owasp_category: Some("A07:2021-Identification and Authentication Failures".to_string()),
    }));

    // SQL injection
    let pattern = Regex::new(r#"\.(execute|query|query_map|query_row|query_row_named|query_row_and_then|query_map_and_then)\([^)]*\$[a-zA-Z0-9_]+\)"#).unwrap();
    patterns.push((pattern, SecurityPattern {
        id:             "sql-injection".to_string(),
        title:          "Potential SQL Injection".to_string(),
        description:    "The code appears to use string formatting in SQL queries which could lead to SQL injection."
            .to_string(),
        severity:       Severity::High,
        category:       "Injection".to_string(),
        remediation:    "Use prepared statements or query builders with parameterized queries instead of string \
                         formatting."
            .to_string(),
        cwe_id:         Some(89), // CWE-89: SQL Injection
        owasp_category: Some("A03:2021-Injection".to_string()),
    }));

    // Command injection
    let pattern = Regex::new(r#"std::process::Command::new\([^)]*\$[a-zA-Z0-9_]+\)"#).unwrap();
    patterns.push((pattern, SecurityPattern {
        id:             "command-injection".to_string(),
        title:          "Potential Command Injection".to_string(),
        description:    "The code appears to use user input in command execution which could lead to command \
                         injection."
            .to_string(),
        severity:       Severity::High,
        category:       "Injection".to_string(),
        remediation:    "Avoid using user input directly in command execution. If necessary, validate and sanitize \
                         all inputs."
            .to_string(),
        cwe_id:         Some(78), // CWE-78: OS Command Injection
        owasp_category: Some("A03:2021-Injection".to_string()),
    }));

    // TODO: Add more security patterns

    patterns
}

/// Visitor for AST-based security analysis
struct SecurityVisitor {
    file_path: PathBuf,
    findings:  Vec<Finding>,
}

impl SecurityVisitor {
    fn new(file_path: &Path) -> Self {
        Self {
            file_path: file_path.to_path_buf(),
            findings:  Vec::new(),
        }
    }

    fn add_finding(&mut self, finding: Finding) {
        self.findings.push(finding);
    }

    fn visit_file(&mut self, file: &syn::File) {
        // Visit all items in the file
        for item in &file.items {
            self.visit_item(item);
        }
    }
}

impl<'ast> Visit<'ast> for SecurityVisitor {
    fn visit_item_fn(&mut self, item_fn: &'ast ItemFn) {
        // Check for missing error handling
        if item_fn.sig.output == syn::ReturnType::Default {
            self.add_finding(Finding {
                id:             "missing-error-handling".to_string(),
                title:          "Missing Error Handling".to_string(),
                description:    "Function does not return a Result type, which may indicate missing error handling."
                    .to_string(),
                severity:       Severity::Medium,
                file:           self.file_path.display().to_string(),
                line:           Some(item_fn.sig.span().start().line as u32),
                column:         Some(item_fn.sig.span().start().column as u32),
                category:       "Error Handling".to_string(),
                remediation:    "Consider using Result<T, E> for functions that can fail.".to_string(),
                cwe_id:         Some(703), // CWE-703: Improper Check or Handling of Exceptional Conditions
                owasp_category: Some("A10:2021-Server-Side Request Forgery".to_string()),
                metadata:       HashMap::new(),
                source:         "owasp-scanner".to_string(),
            });
        }

        syn::visit::visit_item_fn(self, item_fn);
    }

    // TODO: Add more visitor methods for other AST nodes
}

// Use the Severity from the main module

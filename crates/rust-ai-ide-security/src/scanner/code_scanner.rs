//! Code scanning functionality for the OWASP security scanner

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use proc_macro2::{LineColumn, Span};
use regex::Regex;
use sha2::{Digest, Sha256};
use syn::visit::Visit;
use syn::File as SynFile;

use super::types::{Finding, Severity};

// Helper function to get line and column from a span using the span's debug representation
fn get_line_column(span: Span) -> (u32, u32) {
    let span_str = format!("{:?}", span);
    // Parse the span string which looks like "bytes(start..end)"
    if let Some(byte_range) = span_str
        .strip_prefix("bytes(")
        .and_then(|s| s.strip_suffix(")"))
    {
        if let Some((start, _)) = byte_range.split_once("..") {
            if let Ok(offset) = start.parse::<usize>() {
                // This is a simplified approach - in a real implementation, you'd want to map
                // byte offsets to line/column numbers using the file content
                return (1, (offset % 1000) as u32 + 1);
            }
        }
    }
    (1, 1) // Default values if parsing fails
}
use walkdir::WalkDir;

lazy_static::lazy_static! {
    static ref SECURITY_PATTERNS: Vec<(&'static str, &'static str, Severity, Option<u32>, Option<&'static str>)> = vec![
        (r"unsafe\s*\{\s*[^}]*\}", "Unsafe block detected", Severity::High, Some(676), Some("A7:2021-Identification and Authentication Failures")),
        (r#"password\s*=\s*['\"].*?['\"]"#, "Hardcoded password detected", Severity::High, Some(259), Some("A7:2021-Identification and Authentication Failures")),
        (r#"secret\s*=\s*['\"].*?['\"]"#, "Hardcoded secret detected", Severity::High, Some(798), Some("A7:2021-Identification and Authentication Failures")),
        (r#"api[_-]?key\s*=\s*['\"].*?['\"]"#, "Hardcoded API key detected", Severity::High, Some(798), Some("A7:2021-Identification and Authentication Failures")),
        (r#"token\s*=\s*['\"].*?['\"]"#, "Hardcoded token detected", Severity::High, Some(798), Some("A7:2021-Identification and Authentication Failures")),
        (r#"execute\(|exec\(|system\(|sp\(|spawn\("#, "Potentially dangerous shell execution", Severity::High, Some(78), Some("A3:2021-Injection")),
        (r#"(?i)SELECT\s+\*\s+FROM"#, "Unbounded SELECT query", Severity::Medium, Some(89), Some("A1:2021-Broken Access Control")),
        (r#"eval\(|Function\(|new\s+Function\("#, "Dangerous eval function", Severity::High, Some(95), Some("A3:2021-Injection")),
    ];
}

/// Scans Rust source code for security issues
pub async fn scan_code(path: &PathBuf, min_severity: &str, limit: usize) -> Result<Vec<Finding>> {
    let min_severity: Severity = min_severity.parse().unwrap_or(Severity::Medium);
    let mut findings = Vec::new();

    // Collect all Rust files
    let mut files = Vec::new();
    if path.is_dir() {
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir() && e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
        {
            files.push(entry.path().to_path_buf());
        }
    } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
        files.push(path.clone());
    }

    // Compile patterns once
    let patterns: Vec<_> = SECURITY_PATTERNS
        .iter()
        .map(|(pat, title, severity, cwe, owasp)| (Regex::new(pat).unwrap(), *title, *severity, *cwe, *owasp))
        .collect();

    // Scan each file
    for file_path in files {
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            // Check patterns
            for (line_num, line) in content.lines().enumerate() {
                for (pattern, title, severity, cwe_id, owasp) in &patterns {
                    if severity < &min_severity {
                        continue;
                    }

                    if pattern.is_match(line) {
                        let finding = Finding {
                            id:             format!(r#"pattern-{:x}"#, Sha256::digest(line.as_bytes())),
                            title:          title.to_string(),
                            description:    format!(r#"Found potential security issue: {}"#, title),
                            severity:       *severity,
                            file:           file_path.display().to_string(),
                            line:           Some(line_num as u32 + 1),
                            column:         None,
                            category:       "Code Pattern".to_string(),
                            remediation:    format!(r#"Review and secure the following code: {}"#, line.trim()),
                            cwe_id:         *cwe_id,
                            owasp_category: owasp.map(|s| s.to_string()),
                            metadata:       HashMap::new(),
                            source:         "owasp-scanner".to_string(),
                        };
                        findings.push(finding);

                        if findings.len() >= limit {
                            return Ok(findings);
                        }
                    }
                }
            }

            // Parse AST for more complex patterns
            if let Ok(syntax_tree) = syn::parse_file(&content) {
                let mut visitor = SecurityVisitor::new(&file_path);
                visitor.visit_file(&syntax_tree);

                for finding in visitor.findings {
                    if finding.severity >= min_severity {
                        findings.push(finding);

                        if findings.len() >= limit {
                            return Ok(findings);
                        }
                    }
                }
            }
        }
    }

    Ok(findings)
}

/// Visits AST nodes to detect security issues
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
}

impl<'ast> syn::visit::Visit<'ast> for SecurityVisitor {
    fn visit_item_fn(&mut self, item_fn: &'ast syn::ItemFn) {
        // Check for missing error handling
        if item_fn.sig.output == syn::ReturnType::Default {
            let ident_span = item_fn.sig.ident.span();
            let (line, column) = get_line_column(ident_span);

            let mut metadata = HashMap::new();
            metadata.insert("function_name".to_string(), item_fn.sig.ident.to_string());

            self.add_finding(Finding {
                id: format!(
                    "function-handling-{:x}",
                    Sha256::digest(item_fn.sig.ident.to_string().as_bytes())
                ),
                title: "Function handling".to_string(),
                description: format!("Function '{}' may need security review", item_fn.sig.ident),
                severity: Severity::Low,
                file: self.file_path.to_string_lossy().to_string(),
                line: Some(line as u32),
                column: Some(column as u32),
                category: "Code Quality".to_string(),
                remediation: "Consider adding proper error handling and security checks".to_string(),
                cwe_id: None,
                owasp_category: None,
                metadata,
                source: "code_scanner".to_string(),
            });
        }

        // Check for unsafe blocks in the function body
        {
            let mut unsafe_visitor = UnsafeBlockVisitor::new(&self.file_path, &item_fn.sig.ident.to_string());
            syn::visit::visit_item_fn(&mut unsafe_visitor, item_fn);
            self.findings.extend(unsafe_visitor.findings);
        }

        // Continue visiting nested items
        syn::visit::visit_item_fn(self, item_fn);
    }
}

/// Visits unsafe blocks in the code
struct UnsafeBlockVisitor {
    file_path: PathBuf,
    context:   String,
    findings:  Vec<Finding>,
}

impl UnsafeBlockVisitor {
    fn new(file_path: &Path, context: &str) -> Self {
        Self {
            file_path: file_path.to_path_buf(),
            context:   context.to_string(),
            findings:  Vec::new(),
        }
    }

    fn add_finding(&mut self, span: Span) {
        let mut metadata = HashMap::new();
        metadata.insert("context".to_string(), self.context.clone());

        // Get line and column information from the span
        let (line, column) = get_line_column(span);

        self.findings.push(Finding {
            id: format!(
                "unsafe-block-{:x}",
                Sha256::digest(format!("{}:{:?}", self.context, span).as_bytes())
            ),
            title: "Unsafe Block".to_string(),
            description: "Unsafe block detected. Ensure proper safety measures are in place.".to_string(),
            severity: Severity::High,
            file: self.file_path.to_string_lossy().to_string(),
            line: Some(line as u32),
            column: Some(column as u32),
            category: "Memory Safety".to_string(),
            remediation: "Review the unsafe block and ensure all safety invariants are properly documented and \
                          verified."
                .to_string(),
            cwe_id: Some(676),
            owasp_category: Some("A7:2021-Identification and Authentication Failures".to_string()),
            metadata,
            source: "code_scanner".to_string(),
        });
    }
}

impl<'ast> syn::visit::Visit<'ast> for UnsafeBlockVisitor {
    fn visit_expr_unsafe(&mut self, expr: &'ast syn::ExprUnsafe) {
        self.add_finding(expr.unsafe_token.span);
        syn::visit::visit_expr_unsafe(self, expr);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn test_scan_code_unsafe() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        std::fs::write(
            &file_path,
            r#"
            fn main() {
                unsafe {
                    let x = 5;
                }
            }
        "#,
        )
        .unwrap();

        let findings = scan_code(&file_path, "low", 10).await.unwrap();
        assert!(!findings.is_empty(), "Should find unsafe block");
        assert_eq!(findings[0].title, "Unsafe Block Detected");
    }

    #[tokio::test]
    async fn test_scan_code_hardcoded_password() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        std::fs::write(
            &file_path,
            r#"
            fn main() {
                let password = "secret123";
            }
        "#,
        )
        .unwrap();

        let findings = scan_code(&file_path, "low", 10).await.unwrap();
        assert!(!findings.is_empty(), "Should find hardcoded password");
        assert_eq!(findings[0].title, "Hardcoded password detected");
    }
}

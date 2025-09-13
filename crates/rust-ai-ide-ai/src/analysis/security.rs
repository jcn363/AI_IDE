//! Enhanced security analysis patterns for comprehensive security vulnerability detection
//!
//! This module provides advanced security analyzers that complement the existing `AISecurityAnalyzer`
//! by detecting sophisticated security patterns including TOCTOU race conditions, cryptographic
//! vulnerabilities, input validation issues, and concurrency security problems.

use crate::{Severity, SecurityIssue, SecurityIssueType, Range};
use std::path::Path;
use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use syn::{
    visit::Visit, Expr, ExprAsync, ExprAwait, ExprCall, ExprField, ExprMethodCall, ExprPath,
    ExprUnsafe, File, Item, ItemFn, ItemImpl, ItemStruct, Lit, Pat, PatIdent, Stmt, Type,
};

/// Advanced pattern detector for sophisticated security vulnerabilities
pub struct AdvancedPatternDetector {
    toctou_patterns: Vec<Regex>,
    deserialization_patterns: Vec<Regex>,
    transmute_patterns: Vec<Regex>,
    memory_safety_patterns: Vec<Regex>,
    command_injection_patterns: Vec<Regex>,
}

impl AdvancedPatternDetector {
    pub fn new() -> Result<Self> {
        let toctou_patterns = vec![
            Regex::new(r"std::fs::metadata.*std::fs::(read|write|remove)")?,
            Regex::new(r"Path::exists.*std::fs::(File::open|File::create)")?,
            Regex::new(r"is_file\(\).*File::(open|create)")?,
        ];

        let deserialization_patterns = vec![
            Regex::new(r"serde_json::from_str.*<.*dyn")?,
            Regex::new(r"bincode::deserialize.*<.*dyn")?,
            Regex::new(r"ron::from_str.*<.*dyn")?,
            Regex::new(r"toml::from_str.*<.*dyn")?,
        ];

        let transmute_patterns = vec![
            Regex::new(r"std::mem::transmute.*<.*\*")?,
            Regex::new(r"transmute.*<.*&.*&")?,
            Regex::new(r"transmute.*<.*Vec.*\*")?,
        ];

        let memory_safety_patterns = vec![
            Regex::new(r"std::ptr::write.*\\*")?,
            Regex::new(r"std::ptr::read.*\\*")?,
            Regex::new(r"std::slice::from_raw_parts")?,
            Regex::new(r"Box::from_raw")?,
        ];

        let command_injection_patterns = vec![
            // Direct command execution with user input
            Regex::new(r"std::process::Command::new\([^)]*\)\s*\.arg\([^)]*[^a-zA-Z0-9_][a-z0-9_]*[^a-zA-Z0-9_]\)")?,
            // Shell command execution with user input
            Regex::new(r"std::process::Command::new\([^)]*sh[^)]*\)\s*\.arg\([^)]*-c[^)]*\)\.arg\([^)]*[^a-zA-Z0-9_][a-z0-9_]*[^a-zA-Z0-9_]\)")?,
            // Potential OS command injection via env vars
            Regex::new(r"std::env::var\([^)]*\)\s*[.][^.]*\s*std::process::Command")?,
        ];

        Ok(Self {
            toctou_patterns,
            deserialization_patterns,
            transmute_patterns,
            memory_safety_patterns,
            command_injection_patterns,
        })
    }

    pub fn analyze(&self, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        // Detect TOCTOU race conditions
        issues.extend(self.detect_toctou_vulnerabilities(code, file_path));

        // Detect deserialization vulnerabilities
        issues.extend(self.detect_deserialization_vulnerabilities(code, file_path));

        // Detect unsafe transmute patterns
        issues.extend(self.detect_unsafe_transmute_patterns(code, file_path));

        // Detect memory safety violations
        issues.extend(self.detect_memory_safety_violations(code, file_path));

        // Detect command injection vulnerabilities
        issues.extend(self.detect_command_injection(code, file_path));

        issues
    }

    fn detect_toctou_vulnerabilities(&self, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            for pattern in &self.toctou_patterns {
                if pattern.is_match(line) {
                    issues.push(SecurityIssue {
                        id: format!("toctou_{}", line_num),
                        issue_type: SecurityIssueType::RaceCondition,
                        severity: Severity::Error,
                        message: "Potential Time-of-Check-Time-of-Use (TOCTOU) race condition detected".to_string(),
                        file_path: file_path.to_string(),
                        line_range: (line_num as u32 + 1, line_num as u32 + 1),
                        column_range: (1, line.len() as u32),
                        cwe_id: Some("CWE-367".to_string()),
                        recommendation: "Use atomic file operations or proper locking mechanisms to prevent race conditions between check and use".to_string(),
                        confidence: 0.75,
                    });
                }
            }
        }

        issues
    }

    fn detect_deserialization_vulnerabilities(&self, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            for pattern in &self.deserialization_patterns {
                if pattern.is_match(line) {
                    issues.push(SecurityIssue {
                        id: format!("deser_{}", line_num),
                        issue_type: SecurityIssueType::UnvalidatedInput,
                        severity: Severity::Warning,
                        message: "Potentially unsafe deserialization with dynamic types detected".to_string(),
                        file_path: file_path.to_string(),
                        line_range: (line_num as u32 + 1, line_num as u32 + 1),
                        column_range: (1, line.len() as u32),
                        cwe_id: Some("CWE-502".to_string()),
                        recommendation: "Validate input data and avoid deserializing into dynamic types. Use concrete types and implement proper validation".to_string(),
                        confidence: 0.70,
                    });
                }
            }
        }

        issues
    }

    fn detect_unsafe_transmute_patterns(&self, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            for pattern in &self.transmute_patterns {
                if pattern.is_match(line) {
                    let severity = if line.contains("*") || line.contains("Vec") {
                        Severity::Critical
                    } else {
                        Severity::Error
                    };

                    issues.push(SecurityIssue {
                        id: format!("transmute_{}", line_num),
                        issue_type: SecurityIssueType::MemoryLeak,
                        severity,
                        message: "Dangerous transmute pattern detected that may cause memory safety violations".to_string(),
                        file_path: file_path.to_string(),
                        line_range: (line_num as u32 + 1, line_num as u32 + 1),
                        column_range: (1, line.len() as u32),
                        cwe_id: Some("CWE-704".to_string()),
                        recommendation: "Avoid transmute with pointers or complex types. Use safe alternatives like From/Into traits or consider using union types".to_string(),
                        confidence: 0.85,
                    });
                }
            }
        }

        issues
    }

    fn detect_memory_safety_violations(&self, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            for pattern in &self.memory_safety_patterns {
                if pattern.is_match(line) {
                    issues.push(SecurityIssue {
                        id: format!("memory_safety_{}", line_num),
                        issue_type: SecurityIssueType::MemorySafety,
                        severity: Severity::Error,
                        message: "Potential memory safety violation detected".to_string(),
                        file_path: file_path.to_string(),
                        line_range: (line_num as u32 + 1, line_num as u32 + 1),
                        column_range: (1, line.len() as u32),
                        cwe_id: Some("CWE-119".to_string()),
                        recommendation: "Use safe abstractions instead of direct pointer manipulation when possible".to_string(),
                        confidence: 0.8,
                    });
                }
            }
        }

        issues
    }

    /// Detect potential command injection vulnerabilities in the code
    fn detect_command_injection(&self, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();
        let ast = match syn::parse_file(code) {
            Ok(ast) => ast,
            Err(_) => return issues, // Skip parsing errors
        };

        let mut visitor = CommandInjectionVisitor::new();
        visitor.visit_file(&ast);

        // Add pattern-based detections
        for (line_num, line) in code.lines().enumerate() {
            for pattern in &self.command_injection_patterns {
                if pattern.is_match(line) {
                    let issue = SecurityIssue {
                        id: format!("command_injection_{}", line_num),
                        issue_type: SecurityIssueType::CommandInjection,
                        severity: Severity::High,
                        message: "Potential command injection vulnerability detected".to_string(),
                        file_path: file_path.to_string(),
                        line_range: (line_num as u32 + 1, line_num as u32 + 1),
                        column_range: (1, line.len() as u32),
                        cwe_id: Some("CWE-78".to_string()),
                        recommendation: "Validate and sanitize all user input used in command execution. Consider using whitelisting for allowed characters.".to_string(),
                        confidence: 0.85,
                    };

                    // Avoid duplicate issues
                    if !issues.iter().any(|i: &SecurityIssue| i.line_range == issue.line_range && i.issue_type == issue.issue_type) {
                        issues.push(issue);
                    }
                }
            }
        }

        // Add AST-based detections
        for (func_name, (line, col)) in visitor.unsafe_commands {
            let issue = SecurityIssue {
                id: format!("command_injection_ast_{}_{}", line, col),
                issue_type: SecurityIssueType::CommandInjection,
                severity: Severity::High,
                message: format!("Potential command injection in function '{}' - command arguments should be validated", func_name),
                file_path: file_path.to_string(),
                line_range: (line, line),
                column_range: (col, col + 10), // Approximate column range
                cwe_id: Some("CWE-78".to_string()),
                recommendation: "Validate and sanitize all user input used in command execution. Consider using whitelisting for allowed characters.".to_string(),
                confidence: 0.9,
            };

            // Avoid duplicate issues
            if !issues.iter().any(|i: &SecurityIssue| i.line_range == issue.line_range && i.issue_type == issue.issue_type) {
                issues.push(issue);
            }
        }

        issues
    }

    fn detect_integer_overflow_risks(&self, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            for pattern in &self.overflow_patterns {
                if pattern.is_match(line) {
                    let severity = if line.contains("wrapping") {
                        Severity::Info
                    } else {
                        Severity::Warning
                    };

                    issues.push(SecurityIssue {
                        id: format!("overflow_{}", line_num),
                        issue_type: SecurityIssueType::IntegerOverflow,
                        severity,
                        message: "Potential integer overflow in arithmetic operation".to_string(),
                        file_path: file_path.to_string(),
                        line_range: (line_num as u32 + 1, line_num as u32 + 1),
                        column_range: (1, line.len() as u32),
                        cwe_id: Some("CWE-190".to_string()),
                        recommendation: "Use checked arithmetic operations or explicitly handle overflow cases with saturating or wrapping operations".to_string(),
                        confidence: 0.65,
                    });
                }
            }
        }

        issues
    }
}

/// Visitor for detecting command injection patterns
struct CommandInjectionVisitor {
    pub unsafe_commands: Vec<(String, (u32, u32))>,
    current_function: Option<String>,
}

impl CommandInjectionVisitor {
    fn new() -> Self {
        Self {
            unsafe_commands: Vec::new(),
            current_function: None,
        }
    }

    fn check_unsafe_command_arg(&mut self, expr: &Expr, line: u32, col: u32) {
        match expr {
            Expr::Path(expr_path) => {
                // Check if this is a function argument that comes from user input
                if let Some(ident) = expr_path.path.get_ident() {
                    let var_name = ident.to_string();
                    if var_name.contains("input") || var_name.contains("user") || var_name.contains("param") {
                        if let Some(func_name) = &self.current_function {
                            self.unsafe_commands.push((func_name.clone(), (line, col)));
                        }
                    }
                }
            },
            Expr::Call(expr_call) => {
                // Check for env::var() calls in command arguments
                if let Expr::Path(path) = &*expr_call.func {
                    if let Some(ident) = path.path.get_ident() {
                        if ident == "var" || ident == "var_os" {
                            if let Some(func_name) = &self.current_function {
                                self.unsafe_commands.push((func_name.clone(), (line, col)));
                            }
                        }
                    }
                }
            },
            _ => {}
        }
    }
}

impl<'ast> Visit<'ast> for CommandInjectionVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        // Track the current function name
        let old_function = self.current_function.take();
        self.current_function = Some(node.sig.ident.to_string());

        // Visit the function body
        syn::visit::visit_item_fn(self, node);

        // Restore the previous function context
        self.current_function = old_function;
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        // Check for Command::new().arg() patterns
        if let Expr::Path(expr_path) = &*node.receiver {
            if let Some(ident) = expr_path.path.get_ident() {
                if ident == "Command" {
                    // Check if this is a method call on Command
                    for arg in &node.args {
                        self.check_unsafe_command_arg(arg, node.method.span().start().line, node.method.span().start().column);
                    }
                }
            }
        }

        // Continue visiting
        syn::visit::visit_expr_method_call(self, node);
    }
}

/// Cryptographic analyzer for detecting weak cryptographic practices
pub struct CryptographicAnalyzer {
    weak_random_patterns: Vec<Regex>,
    weak_crypto_patterns: Vec<Regex>,
    key_management_patterns: Vec<Regex>,
    iv_nonce_patterns: Vec<Regex>,
}

impl CryptographicAnalyzer {
    pub fn new() -> Result<Self> {
        let weak_random_patterns = vec![
            Regex::new(r"rand::random\(\)")?,
            Regex::new(r"thread_rng\(\)\.gen\(\)")?,
            Regex::new(r"SmallRng::")?,
            Regex::new(r"StdRng::seed_from_u64")?,
        ];

        let weak_crypto_patterns = vec![
            Regex::new(r"use.*md5")?,
            Regex::new(r"use.*sha1")?,
            Regex::new(r"Md5::")?,
            Regex::new(r"Sha1::")?,
            Regex::new(r"DES::")?,
            Regex::new(r"RC4::")?,
        ];

        let key_management_patterns = vec![
            Regex::new(r#"(?i)(key|secret|password)\s*=\s*["'][^"']{8,}["']"#)?,
            Regex::new(r"const.*KEY.*=.*[\"']")?,
            Regex::new(r"static.*SECRET.*=.*[\"']")?,
        ];

        let iv_nonce_patterns = vec![
            Regex::new(r"encrypt.*\[0u8")?,
            Regex::new(r"encrypt.*vec!\[0")?,
            Regex::new(r"Cipher::new.*&\[0")?,
        ];

        Ok(Self {
            weak_random_patterns,
            weak_crypto_patterns,
            key_management_patterns,
            iv_nonce_patterns,
        })
    }

    pub fn analyze(&self, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        issues.extend(self.detect_weak_random_generation(code, file_path));
        issues.extend(self.detect_weak_cryptographic_algorithms(code, file_path));
        issues.extend(self.detect_poor_key_management(code, file_path));
        issues.extend(self.detect_improper_iv_nonce_usage(code, file_path));

        issues
    }

    fn detect_weak_random_generation(&self, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            for pattern in &self.weak_random_patterns {
                if pattern.is_match(line) {
                    issues.push(SecurityIssue {
                        id: format!("weak_random_{}", line_num),
                        issue_type: SecurityIssueType::InsecureRandomness,
                        severity: Severity::Warning,
                        message: "Weak random number generation detected for potential cryptographic use".to_string(),
                        file_path: file_path.to_string(),
                        line_range: (line_num as u32 + 1, line_num as u32 + 1),
                        column_range: (1, line.len() as u32),
                        cwe_id: Some("CWE-338".to_string()),
                        recommendation: "Use cryptographically secure random number generators like OsRng from the rand crate or ring::rand for security-sensitive operations".to_string(),
                        confidence: 0.75,
                    });
                }
            }
        }

        issues
    }

    fn detect_weak_cryptographic_algorithms(&self, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            for pattern in &self.weak_crypto_patterns {
                if pattern.is_match(line) {
                    let severity = if line.contains("md5") || line.contains("sha1") {
                        Severity::Error
                    } else {
                        Severity::Warning
                    };

                    issues.push(SecurityIssue {
                        id: format!("weak_crypto_{}", line_num),
                        issue_type: SecurityIssueType::WeakCryptography,
                        severity,
                        message: "Deprecated or weak cryptographic algorithm detected".to_string(),
                        file_path: file_path.to_string(),
                        line_range: (line_num as u32 + 1, line_num as u32 + 1),
                        column_range: (1, line.len() as u32),
                        cwe_id: Some("CWE-327".to_string()),
                        recommendation: "Replace with modern cryptographic algorithms: use SHA-256, SHA-3, or BLAKE2 for hashing; AES-GCM or ChaCha20-Poly1305 for encryption".to_string(),
                        confidence: 0.90,
                    });
                }
            }
        }

        issues
    }

    fn detect_poor_key_management(&self, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            for pattern in &self.key_management_patterns {
                if pattern.is_match(line) {
                    issues.push(SecurityIssue {
                        id: format!("key_mgmt_{}", line_num),
                        issue_type: SecurityIssueType::WeakCryptography,
                        severity: Severity::Critical,
                        message: "Hardcoded cryptographic key or secret detected".to_string(),
                        file_path: file_path.to_string(),
                        line_range: (line_num as u32 + 1, line_num as u32 + 1),
                        column_range: (1, line.len() as u32),
                        cwe_id: Some("CWE-798".to_string()),
                        recommendation: "Store cryptographic keys in environment variables, secure key management systems, or encrypted configuration files. Never hardcode secrets in source code".to_string(),
                        confidence: 0.95,
                    });
                }
            }
        }

        issues
    }

    fn detect_improper_iv_nonce_usage(&self, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            for pattern in &self.iv_nonce_patterns {
                if pattern.is_match(line) {
                    issues.push(SecurityIssue {
                        id: format!("iv_nonce_{}", line_num),
                        issue_type: SecurityIssueType::WeakCryptography,
                        severity: Severity::Error,
                        message: "Potentially reused or zero IV/nonce detected in encryption".to_string(),
                        file_path: file_path.to_string(),
                        line_range: (line_num as u32 + 1, line_num as u32 + 1),
                        column_range: (1, line.len() as u32),
                        cwe_id: Some("CWE-329".to_string()),
                        recommendation: "Use unique, randomly generated IVs/nonces for each encryption operation. Never reuse IVs with the same key".to_string(),
                        confidence: 0.80,
                    });
                }
            }
        }

        issues
    }
}

/// Input validation analyzer for detecting missing or insufficient input validation
pub struct InputValidationAnalyzer {
    public_api_patterns: Vec<Regex>,
    bounds_check_patterns: Vec<Regex>,
    overflow_patterns: Vec<Regex>,
    parsing_patterns: Vec<Regex>,
}

impl InputValidationAnalyzer {
    pub fn new() -> Result<Self> {
        let public_api_patterns = vec![
            Regex::new(r"pub fn.*\(.*String")?,
            Regex::new(r"pub fn.*\(.*&str")?,
            Regex::new(r"pub fn.*\(.*Vec<")?,
        ];

        let bounds_check_patterns = vec![
            Regex::new(r"\[.*\].*without.*bounds")?,
            Regex::new(r"get_unchecked")?,
            Regex::new(r"slice::from_raw_parts")?,
        ];

        let overflow_patterns = vec![
            Regex::new(r"\+.*without.*overflow")?,
            Regex::new(r"\*.*without.*overflow")?,
            Regex::new(r"wrapping_add")?,
            Regex::new(r"wrapping_mul")?,
        ];

        let parsing_patterns = vec![
            Regex::new(r"unwrap\(\).*parse")?,
            Regex::new(r"expect\(.*parse")?,
            Regex::new(r"from_str.*unwrap")?,
        ];

        Ok(Self {
            public_api_patterns,
            bounds_check_patterns,
            overflow_patterns,
            parsing_patterns,
        })
    }

    pub fn analyze(&self, ast: &File, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        issues.extend(self.detect_missing_input_validation(ast, file_path));
        issues.extend(self.detect_bounds_checking_issues(code, file_path));
        issues.extend(self.detect_integer_overflow_risks(code, file_path));
        issues.extend(self.detect_parsing_error_handling(code, file_path));

        issues
    }

    fn detect_missing_input_validation(&self, ast: &File, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();
        let mut visitor = PublicApiVisitor::new();
        visitor.visit_file(ast);

        for func in visitor.public_functions {
            if func.has_string_params && !func.has_validation {
                issues.push(SecurityIssue {
                    id: format!("input_validation_{}", func.name),
                    issue_type: SecurityIssueType::UnvalidatedInput,
                    severity: Severity::Warning,
                    message: format!("Public function '{}' accepts string input without apparent validation", func.name),
                    file_path: file_path.to_string(),
                    line_range: (func.line, func.line),
                    column_range: (1, 50),
                    cwe_id: Some("CWE-20".to_string()),
                    recommendation: "Add input validation to check for malicious or malformed input. Validate length, format, and content before processing".to_string(),
                    confidence: 0.60,
                });
            }
        }

        issues
    }

    fn detect_bounds_checking_issues(&self, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            for pattern in &self.bounds_check_patterns {
                if pattern.is_match(line) {
                    issues.push(SecurityIssue {
                        id: format!("bounds_check_{}", line_num),
                        issue_type: SecurityIssueType::BufferOverflow,
                        severity: Severity::Error,
                        message: "Potentially unsafe array/vector access without bounds checking".to_string(),
                        file_path: file_path.to_string(),
                        line_range: (line_num as u32 + 1, line_num as u32 + 1),
                        column_range: (1, line.len() as u32),
                        cwe_id: Some("CWE-125".to_string()),
                        recommendation: "Use safe indexing methods like .get() or add explicit bounds checking before array access".to_string(),
                        confidence: 0.75,
                    });
                }
            }
        }

        issues
    }

    fn detect_integer_overflow_risks(&self, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            for pattern in &self.overflow_patterns {
                if pattern.is_match(line) {
                    let severity = if line.contains("wrapping") {
                        Severity::Info
                    } else {
                        Severity::Warning
                    };

                    issues.push(SecurityIssue {
                        id: format!("overflow_{}", line_num),
                        issue_type: SecurityIssueType::IntegerOverflow,
                        severity,
                        message: "Potential integer overflow in arithmetic operation".to_string(),
                        file_path: file_path.to_string(),
                        line_range: (line_num as u32 + 1, line_num as u32 + 1),
                        column_range: (1, line.len() as u32),
                        cwe_id: Some("CWE-190".to_string()),
                        recommendation: "Use checked arithmetic operations or explicitly handle overflow cases with saturating or wrapping operations".to_string(),
                        confidence: 0.65,
                    });
                }
            }
        }

        issues
    }

    fn detect_parsing_error_handling(&self, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            for pattern in &self.parsing_patterns {
                if pattern.is_match(line) {
                    issues.push(SecurityIssue {
                        id: format!("parsing_{}", line_num),
                        issue_type: SecurityIssueType::UnvalidatedInput,
                        severity: Severity::Warning,
                        message: "Parsing operation without proper error handling detected".to_string(),
                        file_path: file_path.to_string(),
                        line_range: (line_num as u32 + 1, line_num as u32 + 1),
                        column_range: (1, line.len() as u32),
                        cwe_id: Some("CWE-754".to_string()),
                        recommendation: "Handle parsing errors gracefully instead of using unwrap() or expect(). Use proper error handling with Result types".to_string(),
                        confidence: 0.70,
                    });
                }
            }
        }

        issues
    }
}

/// Concurrency security analyzer for detecting data races and synchronization issues
pub struct ConcurrencySecurityAnalyzer {
    shared_state_patterns: Vec<Regex>,
    sync_primitive_patterns: Vec<Regex>,
    deadlock_patterns: Vec<Regex>,
}

impl ConcurrencySecurityAnalyzer {
    pub fn new() -> Result<Self> {
        let shared_state_patterns = vec![
            Regex::new(r"static.*mut")?,
            Regex::new(r"Arc<.*Mutex<.*mut")?,
            Regex::new(r"Rc<.*RefCell<.*mut")?,
        ];

        let sync_primitive_patterns = vec![
            Regex::new(r"Mutex::new")?,
            Regex::new(r"RwLock::new")?,
            Regex::new(r"Condvar::new")?,
        ];

        let deadlock_patterns = vec![
            Regex::new(r"lock\(\).*lock\(\)")?,
            Regex::new(r"write\(\).*write\(\)")?,
            Regex::new(r"read\(\).*write\(\)")?,
        ];

        Ok(Self {
            shared_state_patterns,
            sync_primitive_patterns,
            deadlock_patterns,
        })
    }

    pub fn analyze(&self, ast: &File, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        issues.extend(self.detect_data_races(ast, file_path));
        issues.extend(self.detect_shared_mutable_state(code, file_path));
        issues.extend(self.detect_potential_deadlocks(code, file_path));
        issues.extend(self.detect_improper_synchronization(ast, file_path));

        issues
    }

    fn detect_data_races(&self, ast: &File, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();
        let mut visitor = ConcurrencyVisitor::new();
        visitor.visit_file(ast);

        for race_risk in visitor.race_risks {
            issues.push(SecurityIssue {
                id: format!("data_race_{}", race_risk.line),
                issue_type: SecurityIssueType::RaceCondition,
                severity: Severity::Error,
                message: format!("Potential data race detected in concurrent access to '{}'", race_risk.variable),
                file_path: file_path.to_string(),
                line_range: (race_risk.line, race_risk.line),
                column_range: (1, 50),
                cwe_id: Some("CWE-362".to_string()),
                recommendation: "Use proper synchronization primitives like Mutex, RwLock, or atomic types to protect shared data".to_string(),
                confidence: 0.70,
            });
        }

        issues
    }

    fn detect_shared_mutable_state(&self, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            for pattern in &self.shared_state_patterns {
                if pattern.is_match(line) {
                    issues.push(SecurityIssue {
                        id: format!("shared_mut_{}", line_num),
                        issue_type: SecurityIssueType::RaceCondition,
                        severity: Severity::Warning,
                        message: "Shared mutable state detected that may lead to data races".to_string(),
                        file_path: file_path.to_string(),
                        line_range: (line_num as u32 + 1, line_num as u32 + 1),
                        column_range: (1, line.len() as u32),
                        cwe_id: Some("CWE-362".to_string()),
                        recommendation: "Consider using immutable data structures or proper synchronization to prevent data races".to_string(),
                        confidence: 0.65,
                    });
                }
            }
        }

        issues
    }

    fn detect_potential_deadlocks(&self, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            for pattern in &self.deadlock_patterns {
                if pattern.is_match(line) {
                    issues.push(SecurityIssue {
                        id: format!("deadlock_{}", line_num),
                        issue_type: SecurityIssueType::RaceCondition,
                        severity: Severity::Warning,
                        message: "Potential deadlock scenario detected with multiple lock acquisitions".to_string(),
                        file_path: file_path.to_string(),
                        line_range: (line_num as u32 + 1, line_num as u32 + 1),
                        column_range: (1, line.len() as u32),
                        cwe_id: Some("CWE-833".to_string()),
                        recommendation: "Ensure consistent lock ordering or use timeout-based locking to prevent deadlocks".to_string(),
                        confidence: 0.60,
                    });
                }
            }
        }

        issues
    }

    fn detect_improper_synchronization(&self, ast: &File, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();
        let mut visitor = SynchronizationVisitor::new();
        visitor.visit_file(ast);

        for sync_issue in visitor.sync_issues {
            issues.push(SecurityIssue {
                id: format!("sync_{}", sync_issue.line),
                issue_type: SecurityIssueType::RaceCondition,
                severity: Severity::Info,
                message: format!("Synchronization primitive '{}' usage should be reviewed for correctness", sync_issue.primitive_type),
                file_path: file_path.to_string(),
                line_range: (sync_issue.line, sync_issue.line),
                column_range: (1, 50),
                cwe_id: Some("CWE-662".to_string()),
                recommendation: "Verify that synchronization primitives are used correctly and consistently throughout the codebase".to_string(),
                confidence: 0.50,
            });
        }

        issues
    }
}

/// Visitor for detecting public API functions and their validation patterns
struct PublicApiVisitor {
    public_functions: Vec<PublicFunction>,
}

#[derive(Debug)]
struct PublicFunction {
    name: String,
    line: u32,
    has_string_params: bool,
    has_validation: bool,
}

impl PublicApiVisitor {
    fn new() -> Self {
        Self {
            public_functions: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for PublicApiVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let is_public = node.vis.to_token_stream().to_string().contains("pub");

        if is_public {
            let name = node.sig.ident.to_string();
            let line = node.span().start().line as u32;

            let has_string_params = node.sig.inputs.iter().any(|input| {
                if let syn::FnArg::Typed(pat_type) = input {
                    let type_str = quote::quote!(#pat_type.ty).to_string();
                    type_str.contains("String") || type_str.contains("str")
                } else {
                    false
                }
            });

            // Simple heuristic for validation - look for common validation patterns in function body
            let has_validation = if let Some(block) = &node.block {
                let block_str = quote::quote!(#block).to_string();
                block_str.contains("is_empty") ||
                block_str.contains("len()") ||
                block_str.contains("validate") ||
                block_str.contains("check") ||
                block_str.contains("verify")
            } else {
                false
            };

            self.public_functions.push(PublicFunction {
                name,
                line,
                has_string_params,
                has_validation,
            });
        }

        syn::visit::visit_item_fn(self, node);
    }
}

/// Visitor for detecting concurrency-related race condition risks
struct ConcurrencyVisitor {
    race_risks: Vec<RaceRisk>,
    current_function: Option<String>,
}

#[derive(Debug)]
struct RaceRisk {
    variable: String,
    line: u32,
    function: String,
}

impl ConcurrencyVisitor {
    fn new() -> Self {
        Self {
            race_risks: Vec::new(),
            current_function: None,
        }
    }
}

impl<'ast> Visit<'ast> for ConcurrencyVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let old_function = self.current_function.clone();
        self.current_function = Some(node.sig.ident.to_string());

        syn::visit::visit_item_fn(self, node);

        self.current_function = old_function;
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let method_name = node.method.to_string();

        if method_name == "clone" || method_name == "lock" || method_name == "write" {
            if let Expr::Path(path_expr) = &*node.receiver {
                if let Some(ident) = path_expr.path.get_ident() {
                    let var_name = ident.to_string();
                    let line = node.span().start().line as u32;

                    if let Some(func_name) = &self.current_function {
                        self.race_risks.push(RaceRisk {
                            variable: var_name,
                            line,
                            function: func_name.clone(),
                        });
                    }
                }
            }
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

/// Visitor for detecting synchronization primitive usage
struct SynchronizationVisitor {
    sync_issues: Vec<SyncIssue>,
}

#[derive(Debug)]
struct SyncIssue {
    primitive_type: String,
    line: u32,
}

impl SynchronizationVisitor {
    fn new() -> Self {
        Self {
            sync_issues: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for SynchronizationVisitor {
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Expr::Path(path_expr) = &*node.func {
            let path_str = quote::quote!(#path_expr).to_string();

            if path_str.contains("Mutex::new") ||
               path_str.contains("RwLock::new") ||
               path_str.contains("Condvar::new") {

                let line = node.span().start().line as u32;
                self.sync_issues.push(SyncIssue {
                    primitive_type: path_str,
                    line,
                });
            }
        }

        syn::visit::visit_expr_call(self, node);
    }
}

/// Main enhanced security analyzer that coordinates all sub-analyzers
pub struct EnhancedSecurityAnalyzer {
    advanced_detector: AdvancedPatternDetector,
    crypto_analyzer: CryptographicAnalyzer,
    input_validator: InputValidationAnalyzer,
    concurrency_analyzer: ConcurrencySecurityAnalyzer,
}

impl EnhancedSecurityAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            advanced_detector: AdvancedPatternDetector::new()?,
            crypto_analyzer: CryptographicAnalyzer::new()?,
            input_validator: InputValidationAnalyzer::new()?,
            concurrency_analyzer: ConcurrencySecurityAnalyzer::new()?,
        })
    }

    /// Perform comprehensive security analysis on the given code
    pub fn analyze(&self, code: &str, file_path: &str) -> Result<Vec<SecurityIssue>> {
        let mut all_issues = Vec::new();

        // Parse AST for advanced analysis
        let ast = syn::parse_file(code)?;

        // Run all analyzers
        all_issues.extend(self.advanced_detector.analyze(code, file_path));
        all_issues.extend(self.crypto_analyzer.analyze(code, file_path));
        all_issues.extend(self.input_validator.analyze(&ast, code, file_path));
        all_issues.extend(self.concurrency_analyzer.analyze(&ast, code, file_path));

        // Sort by severity and confidence
        all_issues.sort_by(|a, b| {
            b.severity.cmp(&a.severity)
                .then_with(|| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal))
        });

        Ok(all_issues)
    }

    /// Get security analysis summary
    pub fn get_security_summary(&self, issues: &[SecurityIssue]) -> SecuritySummary {
        let total_issues = issues.len();
        let critical_count = issues.iter().filter(|i| i.severity == Severity::Critical).count();
        let error_count = issues.iter().filter(|i| i.severity == Severity::Error).count();
        let warning_count = issues.iter().filter(|i| i.severity == Severity::Warning).count();
        let info_count = issues.iter().filter(|i| i.severity == Severity::Info).count();

        let security_score = if total_issues == 0 {
            100.0
        } else {
            let weighted_score = (critical_count * 10 + error_count * 5 + warning_count * 2 + info_count) as f32;
            let max_possible = total_issues as f32 * 10.0;
            ((max_possible - weighted_score) / max_possible * 100.0).max(0.0)
        };

        SecuritySummary {
            total_issues,
            critical_count,
            error_count,
            warning_count,
            info_count,
            security_score,
        }
    }
}

/// Security analysis summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySummary {
    pub total_issues: usize,
    pub critical_count: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub security_score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toctou_detection() {
        let analyzer = AdvancedPatternDetector::new().unwrap();
        let code = r#"
            use std::fs;
            fn vulnerable_function(path: &str) {
                if fs::metadata(path).is_ok() {
                    let content = fs::read_to_string(path).unwrap();
                    // TOCTOU vulnerability: file could be changed between check and use
                }
            }
        "#;

        let issues = analyzer.analyze(code, "test.rs");
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.issue_type == SecurityIssueType::RaceCondition));
    }

    #[test]
    fn test_weak_crypto_detection() {
        let analyzer = CryptographicAnalyzer::new().unwrap();
        let code = r#"
            use md5::Md5;
            use sha1::Sha1;

            fn weak_crypto() {
                let hash = Md5::digest(b"data");
                let sha1_hash = Sha1::digest(b"data");
            }
        "#;

        let issues = analyzer.analyze(code, "test.rs");
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.issue_type == SecurityIssueType::WeakCryptography));
    }

    #[test]
    fn test_input_validation_detection() {
        let analyzer = InputValidationAnalyzer::new().unwrap();
        let code = r#"
            pub fn process_user_input(input: String) -> String {
                // No validation of input
                input.to_uppercase()
            }
        "#;

        let ast = syn::parse_file(code).unwrap();
        let issues = analyzer.analyze(&ast, code, "test.rs");
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.issue_type == SecurityIssueType::UnvalidatedInput));
    }

    #[test]
    fn test_concurrency_analysis() {
        let analyzer = ConcurrencySecurityAnalyzer::new().unwrap();
        let code = r#"
            use std::sync::{Arc, Mutex};

            fn potential_deadlock() {
                let mutex1 = Arc::new(Mutex::new(0));
                let mutex2 = Arc::new(Mutex::new(0));

                let _guard1 = mutex1.lock().unwrap();
                let _guard2 = mutex2.lock().unwrap(); // Potential deadlock
            }
        "#;

        let ast = syn::parse_file(code).unwrap();
        let issues = analyzer.analyze(&ast, code, "test.rs");
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_enhanced_analyzer_integration() {
        let analyzer = EnhancedSecurityAnalyzer::new().unwrap();
        let code = r#"
            use md5::Md5;
            use std::fs;

            pub fn vulnerable_function(user_input: String, path: &str) {
                // Multiple security issues:
                // 1. No input validation
                // 2. TOCTOU vulnerability
                // 3. Weak cryptography

                if fs::metadata(path).is_ok() {
                    let content = fs::read_to_string(path).unwrap();
                    let hash = Md5::digest(user_input.as_bytes());
                }
            }
        "#;

        let issues = analyzer.analyze(code, "test.rs").unwrap();
        assert!(issues.len() >= 3); // Should detect multiple issues

        let summary = analyzer.get_security_summary(&issues);
        assert!(summary.total_issues >= 3);
        assert!(summary.security_score < 100.0);
    }
}
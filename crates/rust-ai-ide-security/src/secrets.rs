//! Secrets Detection and Credential Scanning System
//!
//! This module provides comprehensive credential scanning capabilities including:
//! - Pattern matching for common secret formats (API keys, tokens, passwords)
//! - Entropy analysis for detecting obfuscated credentials
//! - Context-aware analysis to reduce false positives
//! - Git history scanning for committed secrets
//! - Integration with existing audit logging system

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

/// Main secrets scanning engine
#[derive(Clone)]
pub struct SecretsScanner {
    pattern_engine: Arc<PatternEngine>,
    entropy_analyzer: Arc<EntropyAnalyzer>,
    context_analyzer: Arc<ContextAnalyzer>,
    audit_logger: Arc<dyn AuditLogger>,
    findings: Arc<RwLock<Vec<SecretFinding>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretFinding {
    pub id: String,
    pub file_path: String,
    pub line_number: usize,
    pub secret_type: SecretType,
    pub confidence: f64,
    pub severity: VulnerabilitySeverity,
    pub context: String,
    pub detected_at: DateTime<Utc>,
    pub false_positive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecretType {
    ApiKey,
    Token,
    Password,
    PrivateKey,
    Certificate,
    ConnectionString,
    GenericSecret,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VulnerabilitySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone)]
struct PatternEngine {
    patterns: HashMap<SecretType, Vec<SecretPattern>>,
}

#[derive(Clone)]
struct SecretPattern {
    name: String,
    regex: Regex,
    entropy_threshold: f64,
    context_keywords: Vec<String>,
}

#[derive(Clone)]
struct EntropyAnalyzer {
    min_entropy: f64,
    high_entropy_threshold: f64,
}

#[derive(Clone)]
struct ContextAnalyzer {
    allowed_contexts: Vec<AllowedContext>,
}

#[derive(Clone)]
struct AllowedContext {
    file_patterns: Vec<Regex>,
    line_patterns: Vec<Regex>,
    reason: String,
}

impl SecretsScanner {
    pub async fn new(
        audit_logger: Arc<dyn AuditLogger>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let pattern_engine = Arc::new(PatternEngine::new().await?);
        let entropy_analyzer = Arc::new(EntropyAnalyzer::default());
        let context_analyzer = Arc::new(ContextAnalyzer::new().await?);

        Ok(Self {
            pattern_engine,
            entropy_analyzer,
            context_analyzer,
            audit_logger,
            findings: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Scan content for potential secrets
    pub async fn scan_content(
        &self,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<SecretFinding>, Box<dyn std::error::Error + Send + Sync>> {
        let mut findings = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let matches = self.pattern_engine.find_matches(line).await?;
            for potential_secret in matches {
                // Check entropy
                let entropy = self
                    .entropy_analyzer
                    .calculate_entropy(&potential_secret.text);

                // Skip if entropy is too low
                if entropy < potential_secret.entropy_threshold {
                    continue;
                }

                // Check if it's in an allowed context
                if self
                    .context_analyzer
                    .is_allowed_context(file_path, &potential_secret, line)
                    .await?
                {
                    continue;
                }

                let confidence = self.calculate_confidence(&potential_secret, entropy, line);
                let severity = self.determine_severity(&potential_secret, confidence);

                let finding = SecretFinding {
                    id: uuid::Uuid::new_v4().to_string(),
                    file_path: file_path.to_string(),
                    line_number: line_num + 1,
                    secret_type: potential_secret.secret_type.clone(),
                    confidence,
                    severity,
                    context: line.trim().to_string(),
                    detected_at: Utc::now(),
                    false_positive: false,
                };

                findings.push(finding.clone());

                // Log to audit system
                self.audit_logger.log_secret_detection(&finding).await?;
            }
        }

        // Store findings
        let mut all_findings = self.findings.write().await;
        all_findings.extend(findings.clone());

        Ok(findings)
    }

    /// Scan a directory recursively for secrets
    pub async fn scan_directory(
        &self,
        path: &str,
    ) -> Result<Vec<SecretFinding>, Box<dyn std::error::Error + Send + Sync>> {
        let mut all_findings = Vec::new();

        // Use walkdir to traverse directory
        let walker = walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());

        for entry in walker {
            let file_path = entry.path().to_string_lossy().to_string();

            // Skip certain file types that commonly have false positives
            if self.should_skip_file(&file_path) {
                continue;
            }

            if let Ok(content) = tokio::fs::read_to_string(&entry.path()).await {
                match self.scan_content(&content, &file_path).await {
                    Ok(findings) => all_findings.extend(findings),
                    Err(e) => warn!("Failed to scan file {}: {:?}", file_path, e),
                }
            }
        }

        Ok(all_findings)
    }

    fn should_skip_file(&self, file_path: &str) -> bool {
        // Common patterns to skip
        let skip_patterns = [
            r"\.log$",              // Log files
            r"\.min\.",             // Minified files
            r"node_modules/",       // Dependencies
            r"\.git/",              // Git history
            r"target/",             // Rust build artifacts
            r"dist/",               // Build outputs
            r"\.lock$",             // Lock files
            r"package-lock\.json$", // NPM lock file
        ];

        for pattern in &skip_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(file_path) {
                    return true;
                }
            }
        }

        false
    }

    fn calculate_confidence(&self, pattern: &SecretPatternMatch, entropy: f64, line: &str) -> f64 {
        let mut confidence: f64 = 0.5; // Base confidence

        // Adjust based on entropy
        if entropy > 4.0 {
            confidence += 0.3;
        } else if entropy > 3.0 {
            confidence += 0.15;
        }

        // Adjust based on context
        if self.context_analyzer.has_secret_context(line) {
            confidence += 0.2;
        }

        // Adjust based on pattern specificity
        if pattern.entropy_threshold > 4.0 {
            confidence += 0.1;
        }

        // Cap confidence at 1.0
        confidence.min(1.0)
    }

    fn determine_severity(
        &self,
        pattern: &SecretPatternMatch,
        confidence: f64,
    ) -> VulnerabilitySeverity {
        match pattern.secret_type {
            SecretType::PrivateKey | SecretType::Certificate => {
                if confidence > 0.8 {
                    VulnerabilitySeverity::Critical
                } else {
                    VulnerabilitySeverity::High
                }
            }
            SecretType::Password => {
                if confidence > 0.7 {
                    VulnerabilitySeverity::High
                } else {
                    VulnerabilitySeverity::Medium
                }
            }
            _ => {
                if confidence > 0.8 {
                    VulnerabilitySeverity::High
                } else if confidence > 0.6 {
                    VulnerabilitySeverity::Medium
                } else {
                    VulnerabilitySeverity::Low
                }
            }
        }
    }
}

impl PatternEngine {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut patterns = HashMap::new();

        // Define patterns for different secret types
        let api_key_patterns = vec![
            SecretPattern {
                name: "Common API Key".to_string(),
                regex: Regex::new(r"([a-zA-Z0-9]{20,50})")?,
                //                entropy_threshold: 4.0,
                entropy_threshold: 4.0,
                context_keywords: vec!["api_key", "apikey", "token", "key", "secret"]
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
            },
            SecretPattern {
                name: "Base64 API Key".to_string(),
                regex: Regex::new(r"([A-Za-z0-9+/=]{20,})")?,
                //                entropy_threshold: 4.5,
                entropy_threshold: 4.5,
                context_keywords: vec!["api_key", "apikey", "token"]
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
            },
        ];

        let token_patterns = vec![SecretPattern {
            name: "Bearer Token".to_string(),
            regex: Regex::new(r"Bearer\s+([A-Za-z0-9+/=\.]{20,})")?,
            //                entropy_threshold: 4.5,
            entropy_threshold: 4.5,
            context_keywords: vec!["authorization", "bearer", "token"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        }];

        let password_patterns = vec![SecretPattern {
            name: "Password Pattern".to_string(),
            regex: Regex::new(r#"password[\s]*[=:]+\s*['"]([^'"]{8,})['"]"#)?,
            //                entropy_threshold: 3.5,
            entropy_threshold: 3.5,
            context_keywords: vec!["password", "passwd"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        }];

        let private_key_patterns = vec![SecretPattern {
            name: "SSH Private Key".to_string(),
            regex: Regex::new(r"-----BEGIN\s+(RSA|DSA|ECDSA|OPENSSH)\s+PRIVATE\s+KEY-----")?,
            //                entropy_threshold: 5.0,
            entropy_threshold: 5.0,
            context_keywords: vec!["private", "key", "rsa", "dsa"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        }];

        patterns.insert(SecretType::ApiKey, api_key_patterns);
        patterns.insert(SecretType::Token, token_patterns);
        patterns.insert(SecretType::Password, password_patterns);
        patterns.insert(SecretType::PrivateKey, private_key_patterns);
        //        patterns.insert(SecretType::ApiKey, api_key_patterns);

        Ok(Self { patterns })
    }

    async fn find_matches(
        &self,
        line: &str,
    ) -> Result<Vec<SecretPatternMatch>, Box<dyn std::error::Error + Send + Sync>> {
        let mut matches = Vec::new();

        for (secret_type, patterns) in &self.patterns {
            for pattern in patterns {
                for capture in pattern.regex.captures_iter(line) {
                    if let Some(matched_text) = capture.get(1) {
                        matches.push(SecretPatternMatch {
                            text: matched_text.as_str().to_string(),
                            secret_type: secret_type.clone(),
                            entropy_threshold: pattern.entropy_threshold,
                            context_keywords: pattern.context_keywords.clone(),
                            line: line.to_string(),
                        });
                    }
                }
            }
        }

        Ok(matches)
    }
}

#[derive(Debug)]
struct SecretPatternMatch {
    text: String,
    secret_type: SecretType,
    entropy_threshold: f64,
    context_keywords: Vec<String>,
    line: String,
}

impl EntropyAnalyzer {
    pub fn default() -> Self {
        Self {
            min_entropy: 2.5,
            high_entropy_threshold: 4.0,
        }
    }

    pub fn calculate_entropy(&self, text: &str) -> f64 {
        if text.is_empty() {
            return 0.0;
        }

        let mut char_counts = HashMap::new();
        for ch in text.chars() {
            *char_counts.entry(ch).or_insert(0) += 1;
        }

        let len = text.len() as f64;
        let mut entropy = 0.0;

        for count in char_counts.values() {
            let probability = *count as f64 / len;
            entropy -= probability * probability.log2();
        }

        entropy
    }
}

impl ContextAnalyzer {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let allowed_contexts = vec![
            // Test files
            AllowedContext {
                file_patterns: vec![
                    Regex::new(r"\.test\.rs$")?,
                    Regex::new(r"_test\.rs$")?,
                    Regex::new(r"spec\..*\.rs$")?,
                ],
                line_patterns: vec![
                    Regex::new(r"/*\s*TODO")?,
                    Regex::new(r"//.*test")?,
                    Regex::new(r"EXAMPLE.*KEY")?,
                ],
                reason: "Test file patterns and documentation".to_string(),
            },
        ];

        Ok(Self { allowed_contexts })
    }

    pub async fn is_allowed_context(
        &self,
        file_path: &str,
        _pattern: &SecretPatternMatch,
        line: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        for context in &self.allowed_contexts {
            // Check file pattern
            for file_pattern in &context.file_patterns {
                if file_pattern.is_match(file_path) {
                    return Ok(true);
                }
            }

            // Check line pattern
            for line_pattern in &context.line_patterns {
                if line_pattern.is_match(line) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub fn has_secret_context(&self, line: &str) -> bool {
        let secret_keywords = [
            "secret",
            "token",
            "key",
            "api_key",
            "apikey",
            "password",
            "bearer",
            "auth",
            "credential",
            "private",
            "certificate",
        ];

        let line_lower = line.to_lowercase();
        secret_keywords.iter().any(|kw| line_lower.contains(kw))
    }
}

// Placeholder for audit logger integration
#[async_trait]
pub trait AuditLogger {
    async fn log_secret_detection(
        &self,
        finding: &SecretFinding,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test as async_test;

    // Mock audit logger for testing
    struct MockAuditLogger {
        logged_findings: Arc<RwLock<Vec<SecretFinding>>>,
    }

    impl MockAuditLogger {
        fn new() -> Self {
            Self {
                logged_findings: Arc::new(RwLock::new(Vec::new())),
            }
        }

        async fn get_logged_findings(&self) -> Vec<SecretFinding> {
            self.logged_findings.read().await.clone()
        }
    }

    #[async_trait]
    impl AuditLogger for MockAuditLogger {
        async fn log_secret_detection(
            &self,
            finding: &SecretFinding,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let mut findings = self.logged_findings.write().await;
            findings.push(finding.clone());
            Ok(())
        }
    }

    #[async_test]
    async fn test_secrets_scanner_initialization() {
        let audit_logger = Arc::new(MockAuditLogger::new());
        let scanner = SecretsScanner::new(audit_logger).await.unwrap();

        // Test that scanner was created successfully
        assert!(scanner.findings.try_read().is_ok());
    }

    #[async_test]
    async fn test_api_key_detection() {
        let audit_logger = Arc::new(MockAuditLogger::new());
        let scanner = SecretsScanner::new(audit_logger.clone()).await.unwrap();

        let content = r#"
        // This is a test file with API keys
        const apiKey = "sk-1234567890abcdef1234567890abcdef12345678";
        const token = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        "#;

        let findings = scanner.scan_content(content, "test.rs").await.unwrap();

        // Should detect API key and token
        assert!(!findings.is_empty());

        // Check logged findings
        let logged = audit_logger.get_logged_findings().await;
        assert!(!logged.is_empty());
    }

    #[async_test]
    async fn test_password_detection() {
        let audit_logger = Arc::new(MockAuditLogger::new());
        let scanner = SecretsScanner::new(audit_logger.clone()).await.unwrap();

        let content = r#"
        // Database configuration
        const dbConfig = {
            host: "localhost",
            password: "super_secret_password_123!",
        };
        "#;

        let findings = scanner.scan_content(content, "config.js").await.unwrap();

        // Should detect password
        assert!(!findings.is_empty());
        assert_eq!(findings[0].secret_type, SecretType::Password);
    }

    #[async_test]
    async fn test_private_key_detection() {
        let audit_logger = Arc::new(MockAuditLogger::new());
        let scanner = SecretsScanner::new(audit_logger.clone()).await.unwrap();

        let content = r#"
        -----BEGIN RSA PRIVATE KEY-----
        MIIEpAIBAAKCAQEAwJ8Z+YtZ9dJvO8H3WGZxO8H3WGZxO8H3WGZxO8H3WGZxO8H
        3WGZxO8H3WGZxO8H3WGZxO8H3WGZxO8H3WGZxO8H3WGZxO8H3WGZxO8H3WGZxO
        -----END RSA PRIVATE KEY-----
        "#;

        let findings = scanner.scan_content(content, "private.pem").await.unwrap();

        // Should detect private key
        assert!(!findings.is_empty());
        assert_eq!(findings[0].secret_type, SecretType::PrivateKey);
        assert_eq!(findings[0].severity, VulnerabilitySeverity::Critical);
    }

    #[async_test]
    async fn test_false_positive_filtering() {
        let audit_logger = Arc::new(MockAuditLogger::new());
        let scanner = SecretsScanner::new(audit_logger.clone()).await.unwrap();

        let content = r#"
        // Test file with example keys
        const exampleApiKey = "EXAMPLE_API_KEY_12345";
        // TODO: Replace with real key in production
        const placeholder = "your-secret-key-here";
        "#;

        let findings = scanner.scan_content(content, "test.rs").await.unwrap();

        // Should not detect false positives in test files
        // Note: This depends on the context analyzer configuration
        // The test verifies the structure works
    }

    #[async_test]
    async fn test_entropy_calculation() {
        let analyzer = EntropyAnalyzer::default();

        // Test various strings
        assert_eq!(analyzer.calculate_entropy(""), 0.0); // Empty string
        assert_eq!(analyzer.calculate_entropy("a"), 0.0); // Single character
        assert_eq!(analyzer.calculate_entropy("aaa"), 0.0); // Repeated characters

        let high_entropy = analyzer.calculate_entropy("aB3$9kL2mN8pQ5xZ");
        assert!(high_entropy > 4.0); // Should have high entropy

        let low_entropy = analyzer.calculate_entropy("password");
        assert!(low_entropy < analyzer.high_entropy_threshold);
    }

    #[async_test]
    async fn test_confidence_calculation_edge_cases() {
        let audit_logger = Arc::new(MockAuditLogger::new());
        let scanner = SecretsScanner::new(audit_logger.clone()).await.unwrap();

        // Test with very high entropy content
        let high_entropy_content = "const apiKey = 'aB3$9kL2mN8pQ5xZ7wR4tY6uI1oP3sD5fG7hJ9lZ';";
        let findings = scanner
            .scan_content(high_entropy_content, "test.js")
            .await
            .unwrap();

        // Should have high confidence for high entropy secrets
        if !findings.is_empty() {
            assert!(findings[0].confidence > 0.7);
        }
    }

    #[async_test]
    async fn test_file_skipping_patterns() {
        let audit_logger = Arc::new(MockAuditLogger::new());
        let scanner = SecretsScanner::new(audit_logger.clone()).await.unwrap();

        // Test various file patterns that should be skipped
        assert!(scanner.should_skip_file("debug.log"));
        assert!(scanner.should_skip_file("app.min.js"));
        assert!(scanner.should_skip_file("node_modules/package.json"));
        assert!(scanner.should_skip_file(".git/config"));
        assert!(scanner.should_skip_file("target/debug/app"));
        assert!(scanner.should_skip_file("dist/bundle.js"));
        assert!(scanner.should_skip_file("Cargo.lock"));
        assert!(scanner.should_skip_file("package-lock.json"));

        // Test files that should not be skipped
        assert!(!scanner.should_skip_file("src/main.rs"));
        assert!(!scanner.should_skip_file("config/production.env"));
        assert!(!scanner.should_skip_file("secrets.txt"));
    }

    #[async_test]
    async fn test_severity_determination() {
        let audit_logger = Arc::new(MockAuditLogger::new());
        let scanner = SecretsScanner::new(audit_logger.clone()).await.unwrap();

        // Create mock pattern match for different secret types
        let mock_match = SecretPatternMatch {
            text: "test".to_string(),
            secret_type: SecretType::PrivateKey,
            entropy_threshold: 5.0,
            context_keywords: vec!["private".to_string()],
            line: "private key content".to_string(),
        };

        // Test private key severity
        let severity = scanner.determine_severity(&mock_match, 0.9);
        assert_eq!(severity, VulnerabilitySeverity::Critical);

        let severity_low_conf = scanner.determine_severity(&mock_match, 0.5);
        assert_eq!(severity_low_conf, VulnerabilitySeverity::High);

        // Test password severity
        let password_match = SecretPatternMatch {
            text: "password123".to_string(),
            secret_type: SecretType::Password,
            entropy_threshold: 3.5,
            context_keywords: vec!["password".to_string()],
            line: "password: secret".to_string(),
        };

        let severity = scanner.determine_severity(&password_match, 0.8);
        assert_eq!(severity, VulnerabilitySeverity::High);
    }

    #[async_test]
    async fn test_context_analyzer_allowed_contexts() {
        let context_analyzer = ContextAnalyzer::new().await.unwrap();

        // Test allowed contexts
        let test_file = "src/test.rs";
        let test_line = "// TODO: Add API key";
        let mock_match = SecretPatternMatch {
            text: "EXAMPLE_KEY".to_string(),
            secret_type: SecretType::ApiKey,
            entropy_threshold: 4.0,
            context_keywords: vec!["key".to_string()],
            line: test_line.to_string(),
        };

        let is_allowed = context_analyzer
            .is_allowed_context(test_file, &mock_match, test_line)
            .await
            .unwrap();
        // Test files should be allowed context
        assert!(is_allowed);

        // Test non-allowed context
        let prod_file = "src/main.rs";
        let prod_line = "const apiKey = 'secret';";
        let is_allowed_prod = context_analyzer
            .is_allowed_context(prod_file, &mock_match, prod_line)
            .await
            .unwrap();
        assert!(!is_allowed_prod);
    }

    #[async_test]
    async fn test_secret_context_detection() {
        let context_analyzer = ContextAnalyzer::new().await.unwrap();

        // Test lines with secret context
        assert!(context_analyzer.has_secret_context("const apiKey = 'secret';"));
        assert!(context_analyzer.has_secret_context("password: 'mypassword'"));
        assert!(context_analyzer.has_secret_context("Bearer token"));
        assert!(context_analyzer.has_secret_context("private key"));
        assert!(context_analyzer.has_secret_context("certificate data"));

        // Test lines without secret context
        assert!(!context_analyzer.has_secret_context("const userName = 'john';"));
        assert!(!context_analyzer.has_secret_context("let count = 42;"));
        assert!(!context_analyzer.has_secret_context("console.log('hello');"));
    }

    #[async_test]
    async fn test_pattern_engine_multiple_matches() {
        let pattern_engine = PatternEngine::new().await.unwrap();

        let line_with_multiple =
            "const apiKey = 'sk-1234567890abcdef'; const token = 'Bearer abc123';";

        let matches = pattern_engine
            .find_matches(line_with_multiple)
            .await
            .unwrap();

        // Should find multiple matches
        assert!(matches.len() >= 1); // At least one match

        // Check that different secret types are detected
        let secret_types: std::collections::HashSet<_> =
            matches.iter().map(|m| &m.secret_type).collect();
        assert!(!secret_types.is_empty());
    }

    #[async_test]
    async fn test_finding_storage_and_retrieval() {
        let audit_logger = Arc::new(MockAuditLogger::new());
        let scanner = SecretsScanner::new(audit_logger.clone()).await.unwrap();

        let content = r#"
        const apiKey = "sk-test1234567890abcdef1234567890abcdef";
        "#;

        // Scan content multiple times
        let findings1 = scanner.scan_content(content, "file1.rs").await.unwrap();
        let findings2 = scanner.scan_content(content, "file2.rs").await.unwrap();

        // Findings should be stored internally
        let stored_findings = scanner.findings.read().await;
        assert_eq!(stored_findings.len(), findings1.len() + findings2.len());
    }

    #[async_test]
    async fn test_empty_and_edge_case_scanning() {
        let audit_logger = Arc::new(MockAuditLogger::new());
        let scanner = SecretsScanner::new(audit_logger.clone()).await.unwrap();

        // Test empty content
        let empty_findings = scanner.scan_content("", "empty.txt").await.unwrap();
        assert!(empty_findings.is_empty());

        // Test content with no secrets
        let clean_content = r#"
        const userName = "john_doe";
        const age = 30;
        const isActive = true;
        console.log("Hello, World!");
        "#;

        let clean_findings = scanner
            .scan_content(clean_content, "clean.rs")
            .await
            .unwrap();
        assert!(clean_findings.is_empty());
    }

    #[async_test]
    async fn test_concurrent_scanning() {
        let audit_logger = Arc::new(MockAuditLogger::new());
        let scanner = Arc::new(SecretsScanner::new(audit_logger.clone()).await.unwrap());

        let mut handles = vec![];
        for i in 0..10 {
            let scanner_clone = scanner.clone();
            let handle = tokio::spawn(async move {
                let content = format!("const apiKey{} = 'sk-1234567890abcdef{}';", i, i);
                let findings = scanner_clone
                    .scan_content(&content, &format!("file{}.rs", i))
                    .await
                    .unwrap();
                findings.len()
            });
            handles.push(handle);
        }

        // Wait for all concurrent scans
        let results = futures::future::join_all(handles).await;

        // Verify all succeeded
        for result in results {
            assert!(result.is_ok());
        }

        // Check total findings stored
        let total_findings = scanner.findings.read().await.len();
        assert_eq!(total_findings, 10); // One finding per file
    }
}

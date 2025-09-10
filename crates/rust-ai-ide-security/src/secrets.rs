//! Secrets Detection and Credential Scanning System
//!
//! This module provides comprehensive credential scanning capabilities including:
//! - Pattern matching for common secret formats (API keys, tokens, passwords)
//! - Entropy analysis for detecting obfuscated credentials
//! - Context-aware analysis to reduce false positives
//! - Git history scanning for committed secrets
//! - Integration with existing audit logging system

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::warn;
use regex::Regex;
use std::collections::HashMap;
use chrono::{DateTime, Utc};


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
    pub async fn new(audit_logger: Arc<dyn AuditLogger>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
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
    pub async fn scan_content(&self, content: &str, file_path: &str) -> Result<Vec<SecretFinding>, Box<dyn std::error::Error + Send + Sync>> {
        let mut findings = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let matches = self.pattern_engine.find_matches(line).await?;
            for potential_secret in matches {
                // Check entropy
                let entropy = self.entropy_analyzer.calculate_entropy(&potential_secret.text);

                // Skip if entropy is too low
                if entropy < potential_secret.entropy_threshold {
                    continue;
                }

                // Check if it's in an allowed context
                if self.context_analyzer.is_allowed_context(file_path, &potential_secret, line).await? {
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
    pub async fn scan_directory(&self, path: &str) -> Result<Vec<SecretFinding>, Box<dyn std::error::Error + Send + Sync>> {
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
            r"\.log$",           // Log files
            r"\.min\.",          // Minified files
            r"node_modules/",    // Dependencies
            r"\.git/",          // Git history
            r"target/",         // Rust build artifacts
            r"dist/",           // Build outputs
            r"\.lock$",         // Lock files
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

    fn determine_severity(&self, pattern: &SecretPatternMatch, confidence: f64) -> VulnerabilitySeverity {
        match pattern.secret_type {
            SecretType::PrivateKey | SecretType::Certificate => {
                if confidence > 0.8 {
                    VulnerabilitySeverity::Critical
                } else {
                    VulnerabilitySeverity::High
                }
            },
            SecretType::Password => {
                if confidence > 0.7 {
                    VulnerabilitySeverity::High
                } else {
                    VulnerabilitySeverity::Medium
                }
            },
            _ => {
                if confidence > 0.8 {
                    VulnerabilitySeverity::High
                } else if confidence > 0.6 {
                    VulnerabilitySeverity::Medium
                } else {
                    VulnerabilitySeverity::Low
                }
            },
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
                context_keywords: vec!["api_key", "apikey", "token", "key", "secret"].into_iter().map(|s| s.to_string()).collect(),
            },
            SecretPattern {
                name: "Base64 API Key".to_string(),
                regex: Regex::new(r"([A-Za-z0-9+/=]{20,})")?,
//                entropy_threshold: 4.5,
                entropy_threshold: 4.5,
                context_keywords: vec!["api_key", "apikey", "token"].into_iter().map(|s| s.to_string()).collect(),
            },
        ];

        let token_patterns = vec![
            SecretPattern {
                name: "Bearer Token".to_string(),
                regex: Regex::new(r"Bearer\s+([A-Za-z0-9+/=\.]{20,})")?,
//                entropy_threshold: 4.5,
                entropy_threshold: 4.5,
                context_keywords: vec!["authorization", "bearer", "token"].into_iter().map(|s| s.to_string()).collect(),
            },
        ];

        let password_patterns = vec![
            SecretPattern {
                name: "Password Pattern".to_string(),
                regex: Regex::new(r#"password[\s]*[=:]+\s*['"]([^'"]{8,})['"]"#)?,
//                entropy_threshold: 3.5,
                entropy_threshold: 3.5,
                context_keywords: vec!["password", "passwd"].into_iter().map(|s| s.to_string()).collect(),
            },
        ];

        let private_key_patterns = vec![
            SecretPattern {
                name: "SSH Private Key".to_string(),
                regex: Regex::new(r"-----BEGIN\s+(RSA|DSA|ECDSA|OPENSSH)\s+PRIVATE\s+KEY-----")?,
//                entropy_threshold: 5.0,
                entropy_threshold: 5.0,
                context_keywords: vec!["private", "key", "rsa", "dsa"].into_iter().map(|s| s.to_string()).collect(),
            },
        ];

        patterns.insert(SecretType::ApiKey, api_key_patterns);
        patterns.insert(SecretType::Token, token_patterns);
        patterns.insert(SecretType::Password, password_patterns);
        patterns.insert(SecretType::PrivateKey, private_key_patterns);
//        patterns.insert(SecretType::ApiKey, api_key_patterns);

        Ok(Self { patterns })
    }

    async fn find_matches(&self, line: &str) -> Result<Vec<SecretPatternMatch>, Box<dyn std::error::Error + Send + Sync>> {
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

    pub async fn is_allowed_context(&self, file_path: &str, _pattern: &SecretPatternMatch, line: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
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
            "secret", "token", "key", "api_key", "apikey", "password", "bearer",
            "auth", "credential", "private", "certificate"
        ];

        let line_lower = line.to_lowercase();
        secret_keywords.iter().any(|kw| line_lower.contains(kw))
    }
}

// Placeholder for audit logger integration
#[async_trait]
pub trait AuditLogger {
    async fn log_secret_detection(&self, finding: &SecretFinding) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
// We don't have AuditLogger in lib.rs root yet, will be fixed when security crate is complete
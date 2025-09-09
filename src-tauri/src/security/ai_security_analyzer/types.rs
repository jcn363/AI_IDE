//! Security analysis types and enums
//!
//! This module defines the core types, enums, and data structures
//! used throughout the AI Security Analyzer for classifying and
//! reporting security issues.

use serde::{Deserialize, Serialize};

/// Security severity levels indicating the urgency of response required
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Critical,  // Immediate security risk requiring urgent attention
    High,      // High priority security issue
    Medium,    // Medium priority security issue
    Low,       // Low priority security issue
    Info,      // Informational security finding
}

/// Categories of security issues organized by attack vectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityCategory {
    HardcodedSecrets,    // Hardcoded passwords, API keys, etc.
    UnsafeCode,         // Use of unsafe code blocks
    SqlInjection,       // SQL injection vulnerabilities
    PathTraversal,      // Path traversal attacks
    InsecureRandom,     // Weak random number generation
    MemorySafety,       // Memory safety violations
    CryptographicIssues,// Weak cryptography usage
    InputValidation,    // Input validation problems
    CommandInjection,   // Command injection attacks
    Dependencies,       // Vulnerable dependencies
}

/// Comprehensive security issue structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub category: SecurityCategory,
    pub severity: SecuritySeverity,
    pub title: String,
    pub description: String,
    pub file_path: String,
    pub line_number: Option<usize>,
    pub column: Option<usize>,
    pub code_snippet: Option<String>,
    pub remediation: String,
    pub confidence: f32, // 0.0 to 1.0 confidence level
    pub cwe_id: Option<u32>, // CWE identifier
}

/// Analysis result containing all security findings and summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalysisResult {
    pub issues: Vec<SecurityIssue>,
    pub summary: SecuritySummary,
}

/// Summary statistics for security analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySummary {
    pub total_issues: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub info_count: usize,
    pub overall_score: f32, // 0.0 to 100.0
}
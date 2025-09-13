//! Security analysis module for detecting security vulnerabilities in Rust code.
//!
//! This module provides various analyzers for detecting different types of security issues,
//! including TOCTOU race conditions, cryptographic vulnerabilities, input validation issues,
//! and concurrency security problems.

mod advanced_pattern;
mod analyzer;
mod concurrency;
mod cryptographic;
mod input_validation;

// Only include test module in test builds
#[cfg(test)]
mod tests;

pub use advanced_pattern::AdvancedPatternDetector;
pub use analyzer::SecurityAnalyzer;
pub use concurrency::ConcurrencySecurityAnalyzer;
pub use cryptographic::CryptographicAnalyzer;
pub use input_validation::InputValidationAnalyzer;
use serde::{Deserialize, Serialize};

use crate::analysis::CodeLocation;

/// Represents a security issue found during analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    /// The type of security issue
    pub issue_type:  SecurityIssueType,
    /// Description of the issue
    pub description: String,
    /// Severity level of the issue
    pub severity:    Severity,
    /// Confidence level in the detection (0.0 to 1.0)
    pub confidence:  f32,
    /// Location in the source code where the issue was found
    pub location:    CodeLocation,
    /// Suggested remediation for the issue
    pub remediation: String,
}

/// Types of security issues that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityIssueType {
    /// Time-of-check to time-of-use race condition
    Toctou,
    /// Weak cryptographic algorithm or implementation
    WeakCrypto,
    /// Missing or insufficient input validation
    InputValidation,
    /// Potential data race or concurrency issue
    DataRace,
    /// Memory safety violation
    MemorySafety,
    /// Potential command injection
    CommandInjection,
    /// Integer overflow risk
    IntegerOverflow,
    /// Other type of security issue
    Other,
}

/// Severity levels for security issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational finding that doesn't necessarily indicate a problem
    Info,
    /// Low severity issue
    Low,
    /// Medium severity issue
    Medium,
    /// High severity issue
    High,
    /// Critical severity issue
    Critical,
}

impl From<Severity> for crate::analysis::Severity {
    fn from(severity: Severity) -> Self {
        match severity {
            Severity::Info => Self::Info,
            Severity::Low => Self::Low,
            Severity::Medium => Self::Medium,
            Severity::High => Self::High,
            Severity::Critical => Self::Critical,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::CodeLocation;

    #[test]
    fn test_severity_conversion() {
        assert_eq!(
            crate::analysis::Severity::from(Severity::Info),
            crate::analysis::Severity::Info
        );
        assert_eq!(
            crate::analysis::Severity::from(Severity::High),
            crate::analysis::Severity::High
        );
    }

    #[test]
    fn test_security_issue_creation() {
        let issue = SecurityIssue {
            issue_type:  SecurityIssueType::CommandInjection,
            description: "Potential command injection detected".to_string(),
            severity:    Severity::High,
            confidence:  0.9,
            location:    CodeLocation::new("src/main.rs", 42, 5, 42, 25),
            remediation: "Use proper argument escaping or parameterized APIs".to_string(),
        };

        assert_eq!(issue.issue_type, SecurityIssueType::CommandInjection);
        assert_eq!(issue.severity, Severity::High);
        assert!((issue.confidence - 0.9).abs() < f32::EPSILON);
    }
}

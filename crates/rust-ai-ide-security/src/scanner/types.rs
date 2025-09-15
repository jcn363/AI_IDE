//! Core types for the OWASP security scanner

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Severity levels for security findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl std::str::FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(Severity::Info),
            "low" => Ok(Severity::Low),
            "medium" => Ok(Severity::Medium),
            "high" => Ok(Severity::High),
            "critical" => Ok(Severity::Critical),
            _ => Err(format!("Invalid severity level: {}", s)),
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Low => write!(f, "low"),
            Severity::Medium => write!(f, "medium"),
            Severity::High => write!(f, "high"),
            Severity::Critical => write!(f, "critical"),
        }
    }
}

/// A security finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Unique identifier for the finding
    pub id:             String,
    /// Human-readable title of the finding
    pub title:          String,
    /// Detailed description of the finding
    pub description:    String,
    /// Severity level (critical, high, medium, low, info)
    pub severity:       Severity,
    /// File path where the finding was detected
    pub file:           String,
    /// Line number where the finding was detected
    pub line:           Option<u32>,
    /// Column number where the finding was detected
    pub column:         Option<u32>,
    /// OWASP category this finding belongs to
    pub category:       String,
    /// Suggested remediation steps
    pub remediation:    String,
    /// CWE ID if applicable
    pub cwe_id:         Option<u32>,
    /// OWASP category if applicable
    pub owasp_category: Option<String>,
    /// Additional metadata
    pub metadata:       HashMap<String, String>,
    /// Source of the finding (e.g., "cargo-audit", "cargo-deny", "owasp-scanner")
    pub source:         String,
}

/// Scan results in a format suitable for CI/CD integration
#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResults {
    /// Timestamp of the scan
    pub timestamp:        String,
    /// Duration of the scan in seconds
    pub duration_seconds: f64,
    /// Number of files scanned
    pub files_scanned:    usize,
    /// List of security findings
    pub findings:         Vec<Finding>,
    /// Summary of findings by severity
    pub summary:          HashMap<Severity, usize>,
}

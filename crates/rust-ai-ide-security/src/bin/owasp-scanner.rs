//! OWASP Security Scanner for Rust AI IDE
//!
//! This binary provides a command-line interface for running OWASP security scans
//! on Rust codebases. It's designed to be integrated into CI/CD pipelines and
//! provides machine-readable output for automated processing.

use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use rust_ai_ide_security::scanner::{self, Finding, ScanResults, Severity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;
use regex::Regex;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::str::FromStr;
use std::process::Stdio;
use std::path::Path;

mod code_scanner;
use code_scanner::{scan_code};

/// Re-export types from the scanner module
pub use scanner::{Finding, ScanResults, Severity};

/// CLI arguments for the OWASP scanner
/// Scan results in a format suitable for CI/CD integration
#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResults {
    /// Timestamp of the scan
    pub timestamp: String,
    /// Duration of the scan in seconds
    pub duration_seconds: f64,
    /// Number of files scanned
    pub files_scanned: usize,
    /// List of security findings
    pub findings: Vec<Finding>,
    /// Summary of findings by severity
    pub summary: HashMap<String, usize>,
}

/// A security finding
/// Severity levels for findings
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl FromStr for Severity {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Finding {
    /// Unique identifier for the finding
    pub id: String,
    /// Human-readable title of the finding
    pub title: String,
    /// Detailed description of the finding
    pub description: String,
    /// Severity level (critical, high, medium, low, info)
    #[serde(rename = "severity")]
    pub severity: Severity,
    /// File path where the finding was detected
    pub file: String,
    /// Line number where the finding was detected
    pub line: Option<u32>,
    /// Column number where the finding was detected
    pub column: Option<u32>,
    /// OWASP category this finding belongs to
    pub category: String,
    /// Suggested remediation steps
    pub remediation: String,
    /// CWE ID if applicable
    pub cwe_id: Option<u32>,
    /// OWASP category if applicable
    pub owasp_category: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Source of the finding (e.g., "cargo-audit", "cargo-deny")
    pub source: String,
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "OWASP Security Scanner for Rust AI IDE",
    long_about = "Scans Rust codebases for security vulnerabilities based on OWASP Top 10"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Output format (json, text)
    #[arg(long, default_value = "json", global = true)]
    output_format: String,

    /// Fail on warnings (for CI/CD integration)
    #[arg(long, global = true)]
    fail_on_warning: bool,
}

/// Available subcommands
#[derive(Subcommand, Debug)]
enum Commands {
    /// Scan a directory or file for security issues
    Scan {
        /// Path to the directory or file to scan
        #[arg(short, long, value_name = "PATH", default_value = ".")]
        path: PathBuf,

        /// Output file for the scan results (default: stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Enable all security checks (default: true)
        #[arg(long, default_value_t = true)]
        all: bool,

        /// Enable dependency vulnerability scanning
        #[arg(long)]
        dependencies: bool,

        /// Enable code security scanning
        #[arg(long)]
        code: bool,

        /// Enable configuration file scanning
        #[arg(long)]
        configs: bool,

        /// Minimum severity level to report (critical, high, medium, low, info)
        #[arg(long, default_value = "medium")]
        severity: String,

        /// Maximum number of findings to report (0 for unlimited)
        #[arg(long, default_value_t = 1000)]
        limit: usize,

        /// Generate a JUnit XML report
        #[arg(long, value_name = "FILE")]
        junit: Option<PathBuf>,

        /// Generate a SARIF report
        #[arg(long, value_name = "FILE")]
        sarif: Option<PathBuf>,

        /// Exit code to use when findings are found (0-255)
        #[arg(long, default_value_t = 1)]
        exit_code: i32,
        configs: bool,

        /// Fail the scan if any issues are found
        #[arg(short, long)]
        fail_on_issues: bool,
    },

    /// List available security rules
    Rules {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

/// Main entry point for the OWASP scanner
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Scan {
            path,
            output,
            all,
            dependencies,
            code,
            configs,
            fail_on_issues,
        } => {
            let start_time = Instant::now();

            // Determine which checks to run
            let check_deps = *all || *dependencies;
            let check_code = *all || *code;
            let check_configs = *all || *configs;

            println!(r#"🔍 Starting OWASP security scan at: {}"#, path.display());
            println!(r#"  - Dependencies: {}"#, check_deps);
            println!(r#"  - Code: {}"#, check_code);
            println!(r#"  - Configs: {}"#, check_configs);

            // Run the appropriate scans
            let mut findings = Vec::new();

            if check_deps {
                println!(r#"\n🔒 Scanning dependencies for known vulnerabilities..."#);
                findings.extend(scanner::scan_dependencies(&path, &severity, *limit).await?);
            }

            if check_code {
                info!(r#"🛡️  Running code security scan..."#);
                findings.extend(scanner::scan_code(&path, &severity, *limit).await?);
            }

            let duration = start_time.elapsed();
            let files_scanned = count_rust_files(&path)?;

            // Generate summary
            let mut summary = HashMap::new();
            for finding in &findings {
                *summary.entry(finding.severity.clone()).or_insert(0) += 1;
            }

            // Create the report
            let report = ScanResults {
                timestamp: Utc::now().to_rfc3339(),
                duration_seconds: duration.as_secs_f64(),
                files_scanned,
                findings: findings.clone(),
                summary: summary.clone(),
            };

            // Output the report in the requested format
            let output_json = match cli.output_format.to_lowercase().as_str() {
                "text" => format_report_text(&report, &severity, limit),
                _ => serde_json::to_string_pretty(&report)?,
            };

            // Write to file or stdout
            if let Some(output_path) = output {
                std::fs::write(&output_path, &output_json)
                    .with_context(|| format!(r#"Failed to write to {}"#, output_path.display()))?;
                info!(r#"✅ Report written to {}"#, output_path.display());
            } else {
                println!(r#"{}"#, output_json);
            }

            // Generate JUnit report if requested
            if let Some(junit_path) = junit {
                let junit_xml = generate_junit_report(&report)?;
                std::fs::write(&junit_path, junit_xml)
                    .with_context(|| format!(r#"Failed to write JUnit report to {}"#, junit_path.display()))?;
                info!(r#"📊 JUnit report written to {}"#, junit_path.display());
            }

            // Generate SARIF report if requested
            if let Some(sarif_path) = sarif {
                let sarif_json = generate_sarif_report(&report)?;
                std::fs::write(&sarif_path, sarif_json)
                    .with_context(|| format!(r#"Failed to write SARIF report to {}"#, sarif_path.display()))?;
                info!(r#"📝 SARIF report written to {}"#, sarif_path.display());
            }

            // Print summary
            info!(r#"\n📊 Scan Summary:"#);
            info!(r#"  Duration: {:.2} seconds"#, duration.as_secs_f64());
            info!(r#"  Files scanned: {}"#, files_scanned);
            info!(r#"  Findings:"#);
            for (severity, count) in &summary {
                info!(r#"    {}: {}"#, severity, count);
            }

            // Exit with appropriate status code
            if !findings.is_empty() && (fail_on_warning || findings.iter().any(|f|
                f.severity == "critical" || f.severity == "high" || f.severity == "medium"
            )) {
                std::process::exit(exit_code);
            }
        }

        Commands::Rules { format } => {
            let rules = get_security_rules();

            if format.eq_ignore_ascii_case("json") {
                println!(r#"{}"#, serde_json::to_string_pretty(&rules)?);
            } else {
                // Print rules in a formatted text table
                println!(r#"🔒 Available Security Rules:"#);
                println!("=======================================\n");

                for (category, category_rules) in rules {
                    println!(r#"📌 {}"#, category);
                    println!("---------------------------------------");

                    for rule in category_rules {
                        println!(r#"\n🔹 {}"#, rule.id);
                        println!(r#"   {}"#, rule.description);
                        println!(r#"   Severity: {}"#, rule.severity);
                        println!(r#"   Category: {}"#, rule.category);

                        if let Some(ref cwe) = rule.cwe_id {
                            println!(r#"   CWE: {}"#, cwe);
                        }
                        println!(r#"\n"#);
                    }
                },
            }
        },
    }

    Ok(())
}

/// Represents a security rule
#[derive(serde::Serialize)]
struct SecurityRule {
    id: String,
    severity: String,
    description: String,
    category: String,
    remediation: Vec<String>,
    references: Vec<String>,
}

/// Returns all available security rules
fn get_security_rules() -> std::collections::HashMap<String, Vec<SecurityRule>> {
    let mut rules = std::collections::HashMap::new();

    // Authentication & Session Management
    rules.insert("Authentication & Session Management".to_string(), vec![
        SecurityRule {
            id: "AUTH-001".to_string(),
            severity: "High".to_string(),
            category: "Authentication".to_string(),
            description: "Weak password policy detected".to_string(),
            remediation: vec![
                "Enforce minimum password length of at least 12 characters".to_string(),
                "Require a mix of character types".to_string(),
                "Implement password strength meter".to_string(),
            ],
            references: vec![
                "OWASP: https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html".to_string(),
            ],
        },
        // Add more rules...
    ]);

    // Data Protection
    rules.insert("Data Protection".to_string(), vec![
        SecurityRule {
            id: "CRYPTO-001".to_string(),
            severity: "Critical".to_string(),
            category: "Encryption".to_string(),
            description: "Use of weak cryptographic algorithm".to_string(),
            remediation: vec![
                "Use AES-256-GCM or ChaCha20-Poly1305 for encryption".to_string(),
                "Use Argon2id for password hashing".to_string(),
            ],
            references: vec![
                "OWASP: https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html".to_string(),
            ],
        },
        // Add more rules...
    ]);

    // Input Validation
    rules.insert("Input Validation".to_string(), vec![
        SecurityRule {
            id: "INPUT-001".to_string(),
            severity: "High".to_string(),
            category: "Validation".to_string(),
            description: "Missing input validation".to_string(),
            remediation: vec![
                "Validate all user inputs on the server side".to_string(),
                "Use strong typing and validation libraries".to_string(),
                "Implement allow-list validation".to_string(),
            ],
            references: vec![
                "OWASP: https://cheatsheetseries.owasp.org/cheatsheets/Input_Validation_Cheat_Sheet.html".to_string(),
            ],
        },
        // Add more rules...
    ]);

    // Dependencies
    rules.insert("Dependencies".to_string(), vec![
        SecurityRule {
            id: "DEPS-001".to_string(),
            severity: "High".to_string(),
            category: "Vulnerabilities".to_string(),
            description: "Vulnerable dependencies detected".to_string(),
            remediation: vec![
                "Update dependencies to non-vulnerable versions".to_string(),
                "Use `cargo audit` to check for known vulnerabilities".to_string(),
                "Regularly update dependencies".to_string(),
            ],
            references: vec![
                "RustSec Advisory Database: https://rustsec.org/advisories/".to_string(),
            ],
        },
        // Add more rules...
    ]);

    rules
}

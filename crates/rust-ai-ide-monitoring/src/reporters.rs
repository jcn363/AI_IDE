//! Reporting system for monitoring results

use crate::{
    errors::{MonitoringError, Result},
    types::{AnalysisReport, Severity},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Reporter trait for different output formats
#[async_trait::async_trait]
pub trait Reporter: Send + Sync {
    /// Report analysis results
    async fn report(&self, report: &AnalysisReport) -> Result<()>;

    /// Get the reporter name
    fn name(&self) -> &str;

    /// Check if this reporter supports the given format
    fn supports_format(&self, format: &str) -> bool;
}

/// Report format configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportFormat {
    /// Output format ("json", "yaml", "markdown", "html", "text")
    pub format: String,

    /// Pretty print (for JSON/YAML)
    pub pretty_print: bool,

    /// Include detailed findings
    pub include_details: bool,

    /// Include system information
    pub include_system_info: bool,

    /// Include performance metrics
    pub include_performance: bool,

    /// Output file path (None for stdout)
    pub output_file: Option<String>,
}

/// JSON reporter for structured data output
pub struct JsonReporter {
    format: ReportFormat,
}

impl JsonReporter {
    /// Create a new JSON reporter
    pub fn new(format: ReportFormat) -> Self {
        Self { format }
    }
}

#[async_trait::async_trait]
impl Reporter for JsonReporter {
    async fn report(&self, report: &AnalysisReport) -> Result<()> {
        let output = if self.format.pretty_print {
            serde_json::to_string_pretty(report)
        } else {
            serde_json::to_string(report)
        }
        .map_err(|e| MonitoringError::other(format!("JSON serialization error: {}", e)))?;

        self.write_output(&output).await
    }

    fn name(&self) -> &str {
        "json"
    }

    fn supports_format(&self, format: &str) -> bool {
        format == "json"
    }
}

/// Markdown reporter for human-readable reports
pub struct MarkdownReporter {
    format: ReportFormat,
}

impl MarkdownReporter {
    /// Create a new markdown reporter
    pub fn new(format: ReportFormat) -> Self {
        Self { format }
    }
}

#[async_trait::async_trait]
impl Reporter for MarkdownReporter {
    async fn report(&self, report: &AnalysisReport) -> Result<()> {
        let output = self.generate_markdown(report)?;
        self.write_output(&output).await
    }

    fn name(&self) -> &str {
        "markdown"
    }

    fn supports_format(&self, format: &str) -> bool {
        matches!(format, "markdown" | "md")
    }
}

impl MarkdownReporter {
    /// Generate markdown report content
    fn generate_markdown(&self, report: &AnalysisReport) -> Result<String> {
        let mut md = String::new();

        // Header
        md.push_str(&format!(
            "# Monitoring Report - {}\n\n",
            report.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Summary
        md.push_str("## Summary\n\n");
        md.push_str(&format!("- **Quality Score**: {:.1}%\n", report.quality_score));
        md.push_str(&format!("- **Total Issues**: {}\n", report.metrics.total_issues));
        md.push_str(&format!("- **Analysis Time**: {:.2}s\n", report.duration_seconds));
        md.push_str(&format!("- **Analyzers Run**: {}\n\n", report.results.len()));

        // Severity breakdown
        md.push_str("## Issue Breakdown by Severity\n\n");
        md.push_str(&format!("- **Critical**: {}\n", report.metrics.critical_issues));
        md.push_str(&format!("- **High**: {}\n", report.metrics.high_issues));
        md.push_str(&format!("- **Medium**: {}\n", report.metrics.medium_issues));
        md.push_str(&format!("- **Low**: {}\n", report.metrics.low_issues));
        md.push_str(&format!("- **Info**: {}\n\n", report.metrics.info_issues));

        // Analyzer results
        md.push_str("## Analyzer Results\n\n");
        for result in &report.results {
            md.push_str(&format!("### {}\n\n", result.analyzer));
            md.push_str(&format!("- **Status**: {}\n", if result.success { "✅ Passed" } else { "❌ Failed" }));
            md.push_str(&format!("- **Issues Found**: {}\n", result.issue_count));

            if self.format.include_performance {
                if let Some(ref perf) = result.performance {
                    md.push_str(&format!("- **Analysis Time**: {:.2}s\n", perf.duration_seconds));
                    if let Some(mem) = perf.memory_mb {
                        md.push_str(&format!("- **Memory Usage**: {:.1}MB\n", mem));
                    }
                }
            }

            if !result.findings.is_empty() && self.format.include_details {
                md.push_str("\n**Findings:**\n");
                for finding in &result.findings {
                    let severity_icon = match finding.severity {
                        Severity::Critical => "🚨",
                        Severity::High => "🔴",
                        Severity::Medium => "🟠",
                        Severity::Low => "🟡",
                        Severity::Info => "ℹ️",
                        Severity::None => "⚪",
                    };

                    md.push_str(&format!("- {} {}\n", severity_icon, finding.message));
                }
            }

            md.push_str("\n");
        }

        // System info
        if self.format.include_system_info {
            md.push_str("## System Information\n\n");
            md.push_str(&format!("- **OS**: {}\n", report.system_info.os));
            md.push_str(&format!("- **Architecture**: {}\n", report.system_info.arch));
            md.push_str(&format!("- **Rust Version**: {}\n", report.system_info.rust_version));
            md.push_str(&format!("- **Cargo Version**: {}\n", report.system_info.cargo_version));
            md.push_str(&format!("- **CPU Cores**: {}\n", report.system_info.cpu_count));
            md.push_str(&format!("- **Total Memory**: {}MB\n", report.system_info.total_memory_mb));
            md.push_str(&format!("- **Available Memory**: {}MB\n\n", report.system_info.available_memory_mb));
        }

        Ok(md)
    }
}

/// Console/Terminal reporter for immediate feedback
pub struct ConsoleReporter {
    format: ReportFormat,
    verbose: bool,
}

impl ConsoleReporter {
    /// Create a new console reporter
    pub fn new(format: ReportFormat, verbose: bool) -> Self {
        Self { format, verbose }
    }
}

#[async_trait::async_trait]
impl Reporter for ConsoleReporter {
    async fn report(&self, report: &AnalysisReport) -> Result<()> {
        println!("🔍 Monitoring Analysis Complete");
        println!("════════════════════════════════════════════════════");
        println!("🕐 Timestamp: {}", report.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("⏱️  Duration: {:.2}s", report.duration_seconds);
        println!();

        // Quality score with color
        let score_color = match report.quality_score {
            90.0..=100.0 => "🟢", // Green
            75.0..=90.0 => "🟡",   // Yellow
            _ => "🔴",              // Red
        };
        println!("🎯 Quality Score: {}{:.1}%{}", score_color, report.quality_score,
                 if report.quality_score >= 90.0 { " (Excellent)" }
                 else if report.quality_score >= 75.0 { " (Good)" }
                 else if report.quality_score >= 50.0 { " (Needs Attention)" }
                 else { " (Critical)" });
        println!();

        // Issue summary
        println!("📊 Issue Summary:");
        println!("  • Critical: {}", report.metrics.critical_issues);
        println!("  • High: {}", report.metrics.high_issues);
        println!("  • Medium: {}", report.metrics.medium_issues);
        println!("  • Low: {}", report.metrics.low_issues);
        println!("  • Info: {}", report.metrics.info_issues);
        println!("  • Total: {}", report.metrics.total_issues);

        // Analyzer results
        println!();
        println!("🔧 Analyzer Results:");
        for result in &report.results {
            let status_icon = if result.success { "✅" } else { "❌" };
            println!("  {} {}: {} issues", status_icon, result.analyzer, result.issue_count);

            if !result.success {
                if let Some(ref error) = result.error {
                    println!("    Error: {}", error);
                }
            }

            if self.verbose && !result.findings.is_empty() {
                for finding in result.findings.iter().take(5) {  // Show first 5 findings
                    let severity_icon = match finding.severity {
                        Severity::Critical => "🚨",
                        Severity::High => "🔴",
                        Severity::Medium => "🟠",
                        Severity::Low => "🟡",
                        Severity::Info => "ℹ️",
                        Severity::None => "⚪",
                    };
                    println!("      {} {}", severity_icon, finding.message);
                }
                if result.findings.len() > 5 {
                    println!("      ... and {} more findings", result.findings.len() - 5);
                }
            }
        }

        // Recommendations
        if report.quality_score < 75.0 {
            println!();
            println!("💡 Recommendations:");
            if report.metrics.critical_issues > 0 {
                println!("  • Address critical issues immediately");
            }
            if report.metrics.high_issues > 0 || report.metrics.medium_issues > 5 {
                println!("  • Review high and medium priority findings");
            }
            if report.metrics.low_issues > 10 {
                println!("  • Consider addressing low priority issues for better code quality");
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "console"
    }

    fn supports_format(&self, format: &str) -> bool {
        format == "console" || format == "terminal"
    }
}

/// Trait implementations common to all reporters
trait ReportWriter {
    async fn write_output(&self, output: &str) -> Result<()> where Self: Sized;
}

impl<T> ReportWriter for T where T: Reporter {
    async fn write_output(&self, _output: &str) -> Result<()> {
        // Default implementation writes to stdout
        // In practice, this would be implemented differently for each reporter
        println!("{}", _output);
        Ok(())
    }
}
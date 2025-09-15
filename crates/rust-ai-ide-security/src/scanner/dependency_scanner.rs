//! Dependency scanning functionality for the OWASP security scanner

use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use regex::Regex;
use serde_json::Value;
use tokio::process::Command as AsyncCommand;

use super::types::{Finding, Severity};

/// Scans project dependencies for known vulnerabilities
pub async fn scan_dependencies(path: &std::path::Path, min_severity: &str, limit: usize) -> Result<Vec<Finding>> {
    let min_severity: Severity = min_severity.parse().unwrap_or(Severity::Medium);
    let mut findings = Vec::new();

    // Check if Cargo.lock exists
    let cargo_lock = path.join("Cargo.lock");
    if !cargo_lock.exists() {
        log::warn!("No Cargo.lock found, skipping dependency scan");
        return Ok(findings);
    }

    // Run cargo-audit if available
    if let Ok(output) = AsyncCommand::new("cargo")
        .args(["audit", "--json"])
        .current_dir(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
    {
        if let Ok(audit_output) = String::from_utf8(output.stdout) {
            if let Ok(vulns) = parse_cargo_audit(&audit_output, min_severity, limit) {
                findings.extend(vulns);
            }
        }
    } else {
        log::info!("cargo-audit not found, skipping audit scan");
    }

    // Run cargo-deny if available
    if let Ok(output) = Command::new("cargo")
        .args(["deny", "check", "--format=json"])
        .current_dir(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        if let Ok(deny_output) = String::from_utf8(output.stdout) {
            if let Ok(vulns) = parse_cargo_deny(&deny_output, min_severity, limit) {
                findings.extend(vulns);
            }
        }
    } else {
        log::info!("cargo-deny not found, skipping deny scan");
    }

    // Limit the number of findings
    if findings.len() > limit {
        findings.truncate(limit);
    }

    Ok(findings)
}

/// Parses cargo-audit JSON output into Findings
fn parse_cargo_audit(json_str: &str, min_severity: Severity, _limit: usize) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    let json: Value = serde_json::from_str(json_str)?;

    if let Some(vulns) = json.get("vulnerabilities").and_then(|v| v.get("list")) {
        for vuln in vulns.as_array().unwrap_or(&vec![]) {
            let severity = vuln
                .get("advisory")
                .and_then(|a| a.get("cvss"))
                .and_then(|c| c.get("score"))
                .and_then(|s| s.as_f64())
                .map(|score| match score {
                    s if s >= 9.0 => Severity::Critical,
                    s if s >= 7.0 => Severity::High,
                    s if s >= 4.0 => Severity::Medium,
                    _ => Severity::Low,
                })
                .unwrap_or(Severity::Medium);

            if severity < min_severity {
                continue;
            }

            let advisory = vuln.get("advisory").unwrap();
            let id = advisory
                .get("id")
                .and_then(|i| i.as_str())
                .unwrap_or("unknown");

            let finding = Finding {
                id: id.to_string(),
                title: advisory
                    .get("title")
                    .and_then(|t| t.as_str())
                    .unwrap_or("Vulnerable dependency found")
                    .to_string(),
                description: advisory
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("No description available")
                    .to_string(),
                severity,
                file: "Cargo.lock".to_string(),
                line: None,
                column: None,
                category: "Dependency Vulnerability".to_string(),
                remediation: format!(
                    "Update {} to a non-vulnerable version. Run `cargo update -p {}`",
                    vuln.get("package")
                        .and_then(|p| p.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("package"),
                    vuln.get("package")
                        .and_then(|p| p.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("")
                ),
                cwe_id: advisory
                    .get("cwes")
                    .and_then(|c| c.as_array())
                    .and_then(|c| c.first())
                    .and_then(|c| c.get("cwe"))
                    .and_then(|c| c.as_str())
                    .and_then(|c| c.strip_prefix("CWE-"))
                    .and_then(|c| c.parse::<u32>().ok()),
                owasp_category: advisory
                    .get("categories")
                    .and_then(|c| c.as_array())
                    .and_then(|c| c.first())
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string()),
                metadata: HashMap::new(),
                source: "cargo-audit".to_string(),
            };

            findings.push(finding);
        }
    }

    Ok(findings)
}

/// Parses cargo-deny JSON output into Findings
fn parse_cargo_deny(json_str: &str, min_severity: Severity, _limit: usize) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    let json: Value = serde_json::from_str(json_str)?;

    // Process advisories
    if let Some(advisories) = json.get("advisories") {
        for (pkg_name, pkg_advisories) in advisories.as_object().unwrap_or(&serde_json::Map::new()) {
            for advisory in pkg_advisories.as_array().unwrap_or(&vec![]) {
                let severity = advisory
                    .get("advisory")
                    .and_then(|a| a.get("cvss"))
                    .and_then(|c| c.get("score"))
                    .and_then(|s| s.as_f64())
                    .map(|score| match score {
                        s if s >= 9.0 => Severity::Critical,
                        s if s >= 7.0 => Severity::High,
                        s if s >= 4.0 => Severity::Medium,
                        _ => Severity::Low,
                    })
                    .unwrap_or(Severity::Medium);

                if severity < min_severity {
                    continue;
                }

                let id = advisory
                    .get("advisory")
                    .and_then(|a| a.get("id"))
                    .and_then(|i| i.as_str())
                    .unwrap_or("unknown");

                let finding = Finding {
                    id: id.to_string(),
                    title: advisory
                        .get("advisory")
                        .and_then(|a| a.get("title"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("Vulnerable dependency found")
                        .to_string(),
                    description: advisory
                        .get("advisory")
                        .and_then(|a| a.get("description"))
                        .and_then(|d| d.as_str())
                        .unwrap_or("No description available")
                        .to_string(),
                    severity,
                    file: "Cargo.lock".to_string(),
                    line: None,
                    column: None,
                    category: "Dependency Vulnerability".to_string(),
                    remediation: format!(
                        "Update {} to a non-vulnerable version. Run `cargo update -p {}",
                        pkg_name, pkg_name
                    ),
                    cwe_id: advisory
                        .get("advisory")
                        .and_then(|a| a.get("cwes"))
                        .and_then(|c| c.as_array())
                        .and_then(|c| c.first())
                        .and_then(|c| c.get("cwe"))
                        .and_then(|c| c.as_str())
                        .and_then(|c| c.strip_prefix("CWE-"))
                        .and_then(|c| c.parse::<u32>().ok()),
                    owasp_category: advisory
                        .get("advisory")
                        .and_then(|a| a.get("categories"))
                        .and_then(|c| c.as_array())
                        .and_then(|c| c.first())
                        .and_then(|c| c.as_str())
                        .map(|s| s.to_string()),
                    metadata: HashMap::new(),
                    source: "cargo-deny".to_string(),
                };

                findings.push(finding);
            }
        }
    }

    // Process license issues
    if let Some(licenses) = json.get("licenses") {
        for (pkg_name, pkg_licenses) in licenses.as_object().unwrap_or(&serde_json::Map::new()) {
            if let Some(issues) = pkg_licenses.get("license_issues") {
                for issue in issues.as_array().unwrap_or(&vec![]) {
                    let severity = match issue
                        .get("severity")
                        .and_then(|s| s.as_str())
                        .unwrap_or("low")
                    {
                        "critical" => Severity::Critical,
                        "high" => Severity::High,
                        "medium" => Severity::Medium,
                        _ => Severity::Low,
                    };

                    if severity < min_severity {
                        continue;
                    }

                    let finding = Finding {
                        id: format!(
                            "license-{}-{}",
                            pkg_name,
                            issue
                                .get("license")
                                .and_then(|l| l.as_str())
                                .unwrap_or("unknown")
                        ),
                        title: format!(
                            "License issue in {}: {}",
                            pkg_name,
                            issue
                                .get("license")
                                .and_then(|l| l.as_str())
                                .unwrap_or("unknown")
                        ),
                        description: issue
                            .get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("License issue detected")
                            .to_string(),
                        severity,
                        file: "Cargo.lock".to_string(),
                        line: None,
                        column: None,
                        category: "License Issue".to_string(),
                        remediation: format!(
                            "Resolve license issue for {}: {}",
                            pkg_name,
                            issue
                                .get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or("")
                        ),
                        cwe_id: None,
                        owasp_category: Some("A9:2021-Security Logging & Monitoring Failures".to_string()),
                        metadata: HashMap::new(),
                        source: "cargo-deny".to_string(),
                    };

                    findings.push(finding);
                }
            }
        }
    }

    Ok(findings)
}

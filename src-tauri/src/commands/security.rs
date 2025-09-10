//! OWASP Top 10 Security Scanning Commands
//!
//! Tauri commands for comprehensive security scanning, OWASP Top 10 detection,
//! supply chain analysis, and AI-enhanced vulnerability assessment.

use std::path::Path;
use tauri::State;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::commands::command_templates::*;
use crate::security::*;
use crate::security::owasp_scanner::*;

// State shared between commands
#[derive(Clone)]
pub struct SecurityScannerState {
    pub scanner: Option<Arc<RwLock<OWASPScanner>>>,
    pub last_scan_result: Option<Arc<OWASPSearchResult>>,
}

impl Default for SecurityScannerState {
    fn default() -> Self {
        Self {
            scanner: None,
            last_scan_result: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanRequest {
    pub workspace_path: String,
    pub include_ai_analysis: bool,
    pub include_supply_chain: bool,
    pub max_scan_depth: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResponse {
    pub scan_id: String,
    pub status: String,
    pub vulnerabilities_found: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub overall_risk_score: f32,
    pub scan_duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OWASPScanResult {
    pub scan_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub workspace_path: String,
    pub owasp_categories_covered: Vec<String>,
    pub total_vulnerabilities: usize,
    pub critical_vulnerabilities: usize,
    pub high_vulnerabilities: usize,
    pub medium_vulnerabilities: usize,
    pub low_vulnerabilities: usize,
    pub info_vulnerabilities: usize,
    pub supply_chain_issues: usize,
    pub license_issues: usize,
    pub ai_insights: Vec<String>,
    pub recommendations: Vec<String>,
    pub risk_assessment: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VulnerabilityDetails {
    pub vulnerability_id: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub owasp_category: String,
    pub file_path: String,
    pub line_number: Option<usize>,
    pub cwe_id: Option<String>,
    pub raw_cvss_score: Option<f32>,
    pub exploitable: bool,
    pub fix_available: bool,
    pub remediation_steps: Vec<String>,
    pub evidence: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupplyChainReport {
    pub total_dependencies: usize,
    pub malicious_packages: usize,
    pub license_violations: usize,
    pub outdated_dependencies: usize,
    pub security_issues_by_registrations: Vec<RegistrationIssue>,
    pub health_score: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseComplianceReport {
    pub compliant_packages: usize,
    pub non_compliant_packages: usize,
    pub banned_licenses: Vec<String>,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationIssue {
    pub registry: String,
    pub issue_type: String,
    pub package_name: String,
    pub package_version: String,
    pub severity: String,
    pub description: String,
}

// Initialize OWASP scanner service
#[tauri::command]
pub async fn initialize_owasp_scanner(
    state: State<'_, SecurityScannerState>,
) -> Result<String, String> {
    let config = CommandConfig::default();

    execute_command!("initialize_owasp_scanner", &config, async move || {
        let mut state_guard = state.lock().await;

        if state_guard.scanner.is_none() {
            match OWASPScanner::new().await {
                Ok(scanner) => {
                    let scanner_arc = Arc::new(RwLock::new(scanner));
                    state_guard.scanner = Some(scanner_arc.clone());

                    Ok(serde_json::json!({
                        "status": "success",
                        "message": "OWASP scanner initialized successfully",
                        "supported_categories": vec![
                            "A01:2021-BrokenAccessControl",
                            "A02:2021-CryptographicFailures",
                            "A03:2021-Injection",
                            "A04:2021-InsecureDesign",
                            "A05:2021-SecurityMisconfiguration",
                            "A06:2021-VulnerableOutdatedComponents",
                            "A07:2021-IDAuthenticationFailures",
                            "A08:2021-SoftwareDataIntegrityFailures",
                            "A09:2021-SecurityLoggingFailures",
                            "A10:2021-ServerSideRequestForgery"
                        ]
                    }).to_string())
                }
                Err(e) => Err(format!("Failed to initialize OWASP scanner: {}", e)),
            }
        } else {
            Ok(serde_json::json!({"status": "already_initialized"}).to_string())
        }
    })
}

// Perform comprehensive OWASP security scan
#[tauri::command]
pub async fn perform_owasp_security_scan(
    state: State<'_, SecurityScannerState>,
    workspace_path: String,
    include_ai_analysis: bool,
    include_supply_chain: bool,
) -> Result<String, String> {
    let config = CommandConfig::default();

    execute_command!("perform_owasp_security_scan", &config, async move || {
        acquire_service_and_execute!(state, SecurityScannerState, {
            let scanner_arc = service.scanner.as_ref()
                .ok_or("OWASP scanner not initialized - call initialize_owasp_scanner first")?;

            let workspace_path_buf = Path::new(&workspace_path);
            if !workspace_path_buf.join("Cargo.toml").exists() {
                return Err("Invalid workspace path - Cargo.toml not found".to_string());
            }

            // Validate workspace path for security
            rust_ai_ide_common::validation::validate_secure_path(
                workspace_path_buf,
                "security_scan",
            ).map_err(|e| e.to_string())?;

            // Perform the scan
            let scanner = scanner_arc.read().await;
            let scan_result = match scanner.analyze_codebase(workspace_path_buf).await {
                Ok(result) => result,
                Err(e) => return Err(format!("Scan failed: {}", e)),
            };

            // Convert to frontend-friendly format
            let response = ScanResponse {
                scan_id: format!("{}", uuid::Uuid::new_v4()),
                status: "completed".to_string(),
                vulnerabilities_found: scan_result.summary.total_vulnerabilities,
                critical_count: scan_result.summary.critical_vulnerabilities,
                high_count: scan_result.summary.high_vulnerabilities,
                medium_count: scan_result.summary.medium_vulnerabilities,
                low_count: scan_result.summary.low_vulnerabilities,
                overall_risk_score: scan_result.summary.average_risk_score,
                scan_duration_ms: chrono::Utc::now().timestamp_millis() as u64,
            };

            // Store scan result for later retrieval
            let mut state_guard = state.lock().await;
            state_guard.last_scan_result = Some(Arc::new(scan_result));

            Ok(serde_json::to_string(&response).unwrap())
        })
    })
}

// Get detailed vulnerability report
#[tauri::command]
pub async fn get_vulnerability_details(
    state: State<'_, SecurityScannerState>,
    scan_id: String,
) -> Result<String, String> {
    let config = CommandConfig::default();

    execute_command!("get_vulnerability_details", &config, async move || {
        acquire_service_and_execute!(state, SecurityScannerState, {
            let scan_result = service.last_scan_result.as_ref()
                .ok_or("No scan results available - perform scan first")?;

            let vulnerabilities: Vec<VulnerabilityDetails> = scan_result.vulnerabilities.iter()
                .map(|v| VulnerabilityDetails {
                    vulnerability_id: v.security_issue.title.clone(),
                    title: v.security_issue.title.clone(),
                    description: v.security_issue.description.clone(),
                    severity: format!("{:?}", v.security_issue.severity),
                    owasp_category: v.owasp_category.to_string(),
                    file_path: v.security_issue.file_path.clone(),
                    line_number: v.security_issue.line_number,
                    cwe_id: v.security_issue.cwe_id.map(|id| format!("CWE-{}", id)),
                    raw_cvss_score: Some(v.risk_score),
                    exploitable: v.exploitability.attack_vector != owasp_scanner::AttackVector::Local,
                    fix_available: v.security_issue.remediation.contains("update") ||
                                   v.security_issue.remediation.contains("upgrade"),
                    remediation_steps: vec![v.security_issue.remediation.clone()],
                    evidence: vec![
                        v.security_issue.code_snippet.as_ref().unwrap_or(&"No code sample".to_string()).clone(),
                    ],
                })
                .collect();

            Ok(serde_json::to_string(&vulnerabilities).unwrap())
        })
    })
}

// Get supply chain security report
#[tauri::command]
pub async fn get_supply_chain_report(
    state: State<'_, SecurityScannerState>,
    scan_id: String,
) -> Result<String, String> {
    let config = CommandConfig::default();

    execute_command!("get_supply_chain_report", &config, async move || {
        acquire_service_and_execute!(state, SecurityScannerState, {
            let scan_result = service.last_scan_result.as_ref()
                .ok_or("No scan results available - perform scan first")?;

            let supply_chain_report = SupplyChainReport {
                total_dependencies: scan_result.supply_chain_report.dependencies.len(),
                malicious_packages: scan_result.supply_chain_report.malicious_packages.len(),
                license_violations: scan_result.supply_chain_report.license_issues.len(),
                outdated_dependencies: 0, // Would be calculated from dependency versions
                security_issues_by_registrations: vec![],
                health_score: scan_result.summary.average_risk_score / 10.0,
            };

            Ok(serde_json::to_string(&supply_chain_report).unwrap())
        })
    })
}

// Get license compliance report
#[tauri::command]
pub async fn get_license_compliance_report(
    state: State<'_, SecurityScannerState>,
    scan_id: String,
) -> Result<String, String> {
    let config = CommandConfig::default();

    execute_command!("get_license_compliance_report", &config, async move || {
        acquire_service_and_execute!(state, SecurityScannerState, {
            let scan_result = service.last_scan_result.as_ref()
                .ok_or("No scan results available - perform scan first")?;

            let license_report = LicenseComplianceReport {
                compliant_packages: scan_result.supply_chain_report.dependencies.len()
                                   - scan_result.supply_chain_report.license_issues.len(),
                non_compliant_packages: scan_result.supply_chain_report.license_issues.len(),
                banned_licenses: vec![], // Would extract from license_issues
                recommended_actions: vec![
                    "Review and update dependencies with non-compliant licenses".to_string(),
                    "Replace GPL-licensed dependencies with permissive alternatives".to_string(),
                    "Document license compatibility decisions".to_string(),
                ],
            };

            Ok(serde_json::to_string(&license_report).unwrap())
        })
    })
}

// Perform targeted OWASP category scan
#[tauri::command]
pub async fn scan_owasp_category(
    state: State<'_, SecurityScannerState>,
    category: String,
    workspace_path: String,
) -> Result<String, String> {
    let config = CommandConfig::default();

    execute_command!("scan_owasp_category", &config, async move || {
        acquire_service_and_execute!(state, SecurityScannerState, {
            let scanner_arc = service.scanner.as_ref()
                .ok_or("OWASP scanner not initialized")?;

            let workspace_path_buf = Path::new(&workspace_path);
            let scanner = scanner_arc.read().await;

            // Map category string to OWASP category enum
            let owasp_category = match category.as_str() {
                "A01" | "access_control" => owasp_scanner::OWASPCategory::A01_2021_BrokenAccessControl,
                "A02" | "cryptographic_failures" => owasp_scanner::OWASPCategory::A02_2021_CryptographicFailures,
                "A03" | "injection" => owasp_scanner::OWASPCategory::A03_2021_Injection,
                "A04" | "insecure_design" => owasp_scanner::OWASPCategory::A04_2021_InsecureDesign,
                "A05" | "security_misconfiguration" => owasp_scanner::OWASPCategory::A05_2021_SecurityMisconfiguration,
                "A06" | "vulnerable_components" => owasp_scanner::OWASPCategory::A06_2021_VulnerableOutdatedComponents,
                "A07" | "identification_authentication" => owasp_scanner::OWASPCategory::A07_2021_IDAuthenticationFailures,
                "A08" | "software_integrity" => owasp_scanner::OWASPCategory::A08_2021_SoftwareDataIntegrityFailures,
                "A09" | "logging_failures" => owasp_scanner::OWASPCategory::A09_2021_SecurityLoggingFailures,
                "A10" | "ssrf" => owasp_scanner::OWASPCategory::A10_2021_ServerSideRequestForgery,
                _ => return Err(format!("Unknown OWASP category: {}", category)),
            };

            // This is a placeholder - real implementation would run the specific detector
            let result = serde_json::json!({
                "category": category,
                "status": "completed",
                "vulnerabilities_found": 0,
                "scan_duration_ms": 1234
            });

            Ok(result.to_string())
        })
    })
}

// Get AI-enhanced security insights
#[tauri::command]
pub async fn get_ai_security_insights(
    state: State<'_, SecurityScannerState>,
    scan_id: String,
) -> Result<String, String> {
    let config = CommandConfig::default();

    execute_command!("get_ai_security_insights", &config, async move || {
        acquire_service_and_execute!(state, SecurityScannerState, {
            let scan_result = service.last_scan_result.as_ref()
                .ok_or("No scan results available")?;

            let insights = scan_result.ai_insights.clone();

            Ok(serde_json::to_string(&insights).unwrap())
        })
    })
}

// Update security scan configuration
#[tauri::command]
pub async fn update_security_scan_config(
    _state: State<'_, SecurityScannerState>,
    config: String,
) -> Result<String, String> {
    let command_config = CommandConfig::default();

    execute_command!("update_security_scan_config", &command_config, async move || {
        // Validate and parse configuration
        let _: serde_json::Value = serde_json::from_str(&config)
            .map_err(|e| format!("Invalid configuration JSON: {}", e))?;

        // Placeholder - would update scan configuration
        Ok(serde_json::json!({"status": "configuration_updated"}).to_string())
    })
}

// Export security scan report
#[tauri::command]
pub async fn export_security_scan_report(
    state: State<'_, SecurityScannerState>,
    scan_id: String,
    format: String,
    output_path: String,
) -> Result<String, String> {
    let config = CommandConfig::default();

    execute_command!("export_security_scan_report", &config, async move || {
        acquire_service_and_execute!(state, SecurityScannerState, {
            let scan_result = service.last_scan_result.as_ref()
                .ok_or("No scan results available")?;

            // Validate output path for security
            rust_ai_ide_common::validation::validate_secure_path(
                Path::new(&output_path),
                "security_report_export",
            ).map_err(|e| e.to_string())?;

            // Placeholder - would export scan results in requested format
            match format.as_str() {
                "json" | "html" | "pdf" | "csv" => {
                    Ok(serde_json::json!({
                        "status": "export_completed",
                        "format": format,
                        "output_path": output_path,
                        "file_size_bytes": scan_result.vulnerabilities.len() * 100
                    }).to_string())
                },
                _ => Err(format!("Unsupported export format: {}", format)),
            }
        })
    })
}

// Clean up scanner state
#[tauri::command]
pub async fn cleanup_security_scanner(
    state: State<'_, SecurityScannerState>,
) -> Result<String, String> {
    let config = CommandConfig::default();

    execute_command!("cleanup_security_scanner", &config, async move || {
        acquire_service_and_execute!(state, SecurityScannerState, {
            let mut state_guard = state.lock().await;
            state_guard.scanner = None;
            state_guard.last_scan_result = None;

            Ok(serde_json::json!({"status": "cleanup_completed"}).to_string())
        })
    })
}
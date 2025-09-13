use crate::license;
use crate::security::rustsec_integration::RustsecScanner;
use crate::security::vulnerability_scanner::{
    VulnerabilityReport as VulnerabilityReportSimple, VulnerabilityScanner,
};
use std::path::Path;

/// Security and vulnerability scanning commands using existing modules
#[tauri::command]
pub async fn scan_for_vulnerabilities(
    project_path: String,
) -> Result<Vec<crate::security::rustsec_integration::VulnerabilityReport>, String> {
    let scanner = RustsecScanner::new()
        .map_err(|e| format!("Failed to initialize RustSec scanner: {}", e))?;

    let lockfile_path = Path::new(&project_path).join("Cargo.lock");
    if !lockfile_path.exists() {
        return Err("Cargo.lock not found. Please run 'cargo build' first.".to_string());
    }

    scanner
        .scan_lockfile(&lockfile_path)
        .map_err(|e| format!("Failed to scan for vulnerabilities: {}", e))
}

#[tauri::command]
pub async fn check_vulnerabilities(
    manifest_path: String,
) -> Result<Vec<VulnerabilityReportSimple>, String> {
    let scanner = VulnerabilityScanner::new().map_err(|e| e.to_string())?;
    Ok(scanner.check_dependencies(Path::new(&manifest_path)))
}

#[tauri::command]
pub async fn load_license_policy(policy_path: String) -> Result<license::LicensePolicy, String> {
    license::LicensePolicy::from_file(Path::new(&policy_path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_license_policy(
    policy_path: String,
    policy: license::LicensePolicy,
) -> Result<(), String> {
    policy
        .save_to_file(Path::new(&policy_path))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_license_against_policy(
    license: String,
    policy: license::LicensePolicy,
) -> Result<license::LicenseCompliance, String> {
    Ok(policy.check_license(&license))
}

#[tauri::command]
pub async fn check_license_compliance(
    license: String,
) -> Result<license::LicenseCompliance, String> {
    let checker = license::LicenseComplianceChecker::default();
    Ok(checker.check_license(&license))
}

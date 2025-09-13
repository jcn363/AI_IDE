use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::errors::IDEError;
use crate::security::audit_logger;

/// Configuration for supply chain security analysis
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SupplyChainConfig {
    /// Workspace root path
    pub workspace_path:                PathBuf,
    /// Allowed licenses (default: MIT, Apache-2.0)
    pub allowed_licenses:              Vec<String>,
    /// Enable vulnerability scanning
    pub enable_vulnerability_scanning: bool,
    /// Enable license compliance checking
    pub enable_license_checking:       bool,
    /// Enable cargo-deny integration
    pub enable_cargo_deny:             bool,
}

impl Default for SupplyChainConfig {
    fn default() -> Self {
        Self {
            workspace_path:                PathBuf::from("."),
            allowed_licenses:              vec!["MIT".to_string(), "Apache-2.0".to_string()],
            enable_vulnerability_scanning: true,
            enable_license_checking:       true,
            enable_cargo_deny:             true,
        }
    }
}

/// Result of supply chain analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyChainAnalysisResult {
    /// Dependency analysis results
    pub dependencies:      Vec<DependencyInfo>,
    /// License compliance issues
    pub license_issues:    Vec<LicenseIssue>,
    /// Vulnerability findings
    pub vulnerabilities:   Vec<Vulnerability>,
    /// Cargo-deny results
    pub cargo_deny_result: Option<CargoDenyResult>,
    /// Overall status
    pub status:            AnalysisStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisStatus {
    Passed,
    Warnings,
    Failed,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub name:         String,
    pub version:      String,
    pub source:       String,
    pub license:      Option<String>,
    pub dependencies: Vec<String>,
}

/// License compliance issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseIssue {
    pub package: String,
    pub license: String,
    pub issue:   String,
}

/// Vulnerability finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub package:     String,
    pub advisory_id: String,
    pub severity:    String,
    pub description: String,
}

/// Cargo-deny result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoDenyResult {
    pub output:    String,
    pub exit_code: i32,
}

/// Supply chain security service
pub struct SupplyChainService {
    config: Arc<Mutex<SupplyChainConfig>>,
}

impl SupplyChainService {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(SupplyChainConfig::default())),
        }
    }

    pub async fn update_config(&self, config: SupplyChainConfig) -> Result<(), IDEError> {
        let mut cfg = self.config.lock().await;
        *cfg = config;
        audit_logger::log_security_event(
            "Supply chain config updated",
            &cfg.workspace_path.to_string_lossy(),
        )
        .await;
        Ok(())
    }

    /// Perform comprehensive supply chain analysis
    pub async fn audit_supply_chain(&self) -> Result<SupplyChainAnalysisResult, IDEError> {
        let config = self.config.lock().await.clone();
        audit_logger::log_security_event(
            "Starting supply chain audit",
            &config.workspace_path.to_string_lossy(),
        )
        .await;

        let mut errors = Vec::new();

        // Analyze dependencies
        let dependencies = match self.analyze_dependencies(&config.workspace_path).await {
            Ok(deps) => deps,
            Err(e) => {
                errors.push(e);
                Vec::new()
            }
        };

        // Check license compliance
        let license_issues = if config.enable_license_checking {
            match self.check_license_compliance(&dependencies, &config).await {
                Ok(issues) => issues,
                Err(e) => {
                    errors.push(e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        // Scan vulnerabilities
        let vulnerabilities = if config.enable_vulnerability_scanning {
            match self.scan_vulnerabilities(&dependencies).await {
                Ok(vulns) => vulns,
                Err(e) => {
                    errors.push(e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        // Run cargo-deny
        let cargo_deny_result = if config.enable_cargo_deny {
            match self.run_cargo_deny(&config.workspace_path).await {
                Ok(result) => Some(result),
                Err(e) => {
                    errors.push(e);
                    None
                }
            }
        } else {
            None
        };

        let status = if errors.is_empty() && vulnerabilities.is_empty() && license_issues.is_empty() {
            AnalysisStatus::Passed
        } else if !vulnerabilities.is_empty() || !license_issues.is_empty() {
            AnalysisStatus::Failed
        } else {
            AnalysisStatus::Warnings
        };

        audit_logger::log_security_event(
            "Supply chain audit completed",
            &format!("Status: {:?}", status),
        )
        .await;

        if !errors.is_empty() {
            return Err(IDEError::SupplyChainAnalysisFailed {
                reason: format!("Analysis errors: {:?}", errors),
            });
        }

        Ok(SupplyChainAnalysisResult {
            dependencies,
            license_issues,
            vulnerabilities,
            cargo_deny_result,
            status,
        })
    }

    /// Analyze project dependencies using cargo_metadata
    async fn analyze_dependencies(&self, workspace_path: &PathBuf) -> Result<Vec<DependencyInfo>, IDEError> {
        use cargo_metadata::{MetadataCommand, Package};

        let metadata = MetadataCommand::new()
            .manifest_path(workspace_path.join("Cargo.toml"))
            .exec()
            .map_err(|e| IDEError::DependencyAnalysisFailed {
                reason: format!("Failed to get cargo metadata: {}", e),
            })?;

        let mut dependencies = Vec::new();

        for package in metadata.packages {
            let deps = package
                .dependencies
                .iter()
                .map(|d| d.name.clone())
                .collect::<Vec<_>>();

            dependencies.push(DependencyInfo {
                name:         package.name,
                version:      package.version.to_string(),
                source:       package
                    .source
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "local".to_string()),
                license:      package.license,
                dependencies: deps,
            });
        }

        Ok(dependencies)
    }

    /// Check license compliance using spdx
    async fn check_license_compliance(
        &self,
        dependencies: &[DependencyInfo],
        config: &SupplyChainConfig,
    ) -> Result<Vec<LicenseIssue>, IDEError> {
        use spdx::Licensee;

        let mut issues = Vec::new();

        for dep in dependencies {
            if let Some(license_str) = &dep.license {
                // Parse license using spdx
                let licensee = match Licensee::parse(license_str) {
                    Ok(l) => l,
                    Err(_) => {
                        issues.push(LicenseIssue {
                            package: dep.name.clone(),
                            license: license_str.clone(),
                            issue:   "Invalid SPDX license expression".to_string(),
                        });
                        continue;
                    }
                };

                // Check if license is allowed
                let allowed = config
                    .allowed_licenses
                    .iter()
                    .any(|allowed| licensee.matches(allowed));

                if !allowed {
                    issues.push(LicenseIssue {
                        package: dep.name.clone(),
                        license: license_str.clone(),
                        issue:   format!(
                            "License '{}' not in allowed list: {:?}",
                            license_str, config.allowed_licenses
                        ),
                    });
                }
            } else {
                issues.push(LicenseIssue {
                    package: dep.name.clone(),
                    license: "None".to_string(),
                    issue:   "No license specified".to_string(),
                });
            }
        }

        Ok(issues)
    }

    /// Scan for vulnerabilities using rustsec
    async fn scan_vulnerabilities(&self, dependencies: &[DependencyInfo]) -> Result<Vec<Vulnerability>, IDEError> {
        use rustsec::Database;

        let database = Database::fetch()
            .await
            .map_err(|e| IDEError::VulnerabilityScanFailed {
                reason: format!("Failed to fetch vulnerability database: {}", e),
            })?;

        let mut vulnerabilities = Vec::new();

        for dep in dependencies {
            let package_name =
                rustsec::package::Name::parse(&dep.name).map_err(|e| IDEError::VulnerabilityScanFailed {
                    reason: format!("Invalid package name '{}': {}", dep.name, e),
                })?;

            let version = rustsec::Version::parse(&dep.version).map_err(|e| IDEError::VulnerabilityScanFailed {
                reason: format!("Invalid version '{}': {}", dep.version, e),
            })?;

            let package = rustsec::package::Package {
                name: package_name,
                version,
            };

            let vulns = database.query(&package);
            for vuln in vulns {
                vulnerabilities.push(Vulnerability {
                    package:     dep.name.clone(),
                    advisory_id: vuln.advisory.id.to_string(),
                    severity:    vuln.advisory.severity.to_string(),
                    description: vuln.advisory.description.clone(),
                });
            }
        }

        Ok(vulnerabilities)
    }

    /// Run cargo-deny check
    async fn run_cargo_deny(&self, workspace_path: &PathBuf) -> Result<CargoDenyResult, IDEError> {
        use std::process::Command;

        let output = Command::new("cargo")
            .args(&["deny", "check"])
            .current_dir(workspace_path)
            .output()
            .map_err(|e| IDEError::CargoDenyFailed {
                reason: format!("Failed to run cargo deny: {}", e),
            })?;

        let result = CargoDenyResult {
            output:    String::from_utf8_lossy(&output.stdout).to_string(),
            exit_code: output.status.code().unwrap_or(1),
        };

        Ok(result)
    }
}

/// Default implementation
impl Default for SupplyChainService {
    fn default() -> Self {
        Self::new()
    }
}

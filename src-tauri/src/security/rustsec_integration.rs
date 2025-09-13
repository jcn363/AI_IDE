use anyhow::{Context, Result};
use rustsec::{Database, Error as RustsecError, ErrorKind};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct VulnerabilityReport {
    pub package: String,
    pub version: String,
    pub advisory_id: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub cvss: Option<f32>,
    pub patched_versions: Vec<String>,
    pub unaffected_versions: Vec<String>,
}

pub struct RustsecScanner {
    db: Database,
}

impl RustsecScanner {
    /// Creates a new RustsecScanner with an up-to-date vulnerability database
    pub fn new() -> Result<Self> {
        let db = Database::fetch().context("Failed to fetch RustSec advisory database")?;

        Ok(Self { db })
    }

    /// Scans a Cargo.lock file for vulnerabilities
    pub fn scan_lockfile(&self, lockfile_path: &Path) -> Result<Vec<VulnerabilityReport>> {
        let lockfile =
            rustsec::Lockfile::load(lockfile_path).context("Failed to load Cargo.lock")?;

        let mut reports = Vec::new();

        for vulnerability in self.db.vulnerabilities(&lockfile) {
            let advisory = &vulnerability.advisory;
            let package = &vulnerability.package;

            reports.push(VulnerabilityReport {
                package: package.name.to_string(),
                version: package.version.to_string(),
                advisory_id: advisory.id.to_string(),
                title: advisory.title.clone(),
                description: advisory.description.clone(),
                severity: format!(
                    "{:?}",
                    advisory
                        .cvss
                        .as_ref()
                        .map(|c| c.severity())
                        .unwrap_or(rustsec::Severity::Medium)
                ),
                cvss: advisory.cvss.as_ref().map(|c| c.score().to_f32().unwrap()),
                patched_versions: vuln
                    .versions
                    .patched()
                    .iter()
                    .map(|v| v.to_string())
                    .collect(),
                unaffected_versions: vuln
                    .versions
                    .unaffected()
                    .iter()
                    .map(|v| v.to_string())
                    .collect(),
            });
        }

        Ok(reports)
    }

    /// Updates the local vulnerability database
    pub fn update_database(&mut self) -> Result<()> {
        self.db = Database::fetch().context("Failed to update RustSec advisory database")?;
        Ok(())
    }

    /// Returns the timestamp of the last database update
    pub fn last_updated(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        // Note: rustsec Database may not expose last updated timestamp
        None
    }
}

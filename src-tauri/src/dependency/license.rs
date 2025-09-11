//! License handling and compliance checking for dependencies.
//!
//! This module provides functionality for:
//! - License identification and parsing
//! - License compatibility checking
//! - Compliance policy enforcement

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensePolicy {
    pub allowed_licenses: Vec<String>,
    pub denied_licenses: Vec<String>,
    pub license_groups: HashMap<String, Vec<String>>,
    pub strict_mode: bool,
}

impl Default for LicensePolicy {
    fn default() -> Self {
        Self {
            allowed_licenses: vec![
                "MIT".to_string(),
                "Apache-2.0".to_string(),
                "BSD-3-Clause".to_string(),
                "BSD-2-Clause".to_string(),
                "ISC".to_string(),
            ],
            denied_licenses: Vec::new(),
            license_groups: HashMap::new(),
            strict_mode: false,
        }
    }
}

impl LicensePolicy {
    pub fn from_file<P: AsRef<std::path::Path>>(_path: P) -> Result<Self, std::io::Error> {
        Ok(Self::default())
    }

    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, _path: P) -> Result<(), std::io::Error> {
        // Placeholder implementation
        Ok(())
    }

    pub fn check_license(&self, license: &str) -> LicenseCompliance {
        let allowed = self.allowed_licenses.contains(&license.to_string())
            || self.allowed_licenses.iter().any(|l| license.contains(l));

        let denied = self.denied_licenses.contains(&license.to_string())
            || self.denied_licenses.iter().any(|l| license.contains(l));

        if allowed && !denied {
            LicenseCompliance::Compliant
        } else if denied {
            LicenseCompliance::ViolatesPolicy
        } else if self.strict_mode {
            LicenseCompliance::NotInAllowList
        } else {
            LicenseCompliance::Warning
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LicenseCompliance {
    Compliant,
    Warning,
    NotInAllowList,
    ViolatesPolicy,
    UnknownLicense,
}

#[derive(Debug, Clone)]
pub struct LicenseComplianceChecker {
    // Placeholder implementation
}

impl Default for LicenseComplianceChecker {
    fn default() -> Self {
        Self {}
    }
}

impl LicenseComplianceChecker {
    pub fn check_license(&self, license: &str) -> LicenseCompliance {
        // Simple license checking logic
        match license.to_lowercase().as_str() {
            "mit" | "apache-2.0" | "bsd-3-clause" | "bsd-2-clause" | "isc" => {
                LicenseCompliance::Compliant
            }
            "gpl-2.0" | "gpl-3.0" | "lgpl-2.1" | "lgpl-3.0" => LicenseCompliance::NotInAllowList,
            "" | "none" | "unknown" => LicenseCompliance::UnknownLicense,
            _ => LicenseCompliance::Warning,
        }
    }
}

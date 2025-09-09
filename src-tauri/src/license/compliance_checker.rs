use cargo_metadata::Package;
use spdx::{Expression, Licensee};
use std::collections::HashSet;

pub struct LicenseComplianceChecker {
    allowed_licenses: HashSet<&'static str>,
    denied_licenses: HashSet<&'static str>,
}

impl Default for LicenseComplianceChecker {
    fn default() -> Self {
        let mut allowed = HashSet::new();
        allowed.insert("MIT");
        allowed.insert("Apache-2.0");
        allowed.insert("BSD-3-Clause");
        allowed.insert("ISC");
        allowed.insert("Unlicense");
        
        let mut denied = HashSet::new();
        denied.insert("GPL-3.0");
        denied.insert("AGPL-3.0");
        
        Self {
            allowed_licenses: allowed,
            denied_licenses: denied,
        }
    }
}

impl LicenseComplianceChecker {
    pub fn new(allowed: Option<Vec<&'static str>>, denied: Option<Vec<&'static str>>) -> Self {
        let mut checker = Self::default();
        
        if let Some(allowed) = allowed {
            checker.allowed_licenses = allowed.into_iter().collect();
        }
        
        if let Some(denied) = denied {
            checker.denied_licenses = denied.into_iter().collect();
        }
        
        checker
    }
    
    pub fn check_license(&self, license: &str) -> LicenseCompliance {
        match license.parse::<Expression>() {
            Err(_) => LicenseCompliance::Invalid,
            Ok(expr) => self.check_license_expr(&expr),
        }
    }

    fn check_license_expr(&self, expr: &Expression) -> LicenseCompliance {
        // Check for denied licenses first
        if let Some(denied) = self.check_denied_licenses(expr) {
            return LicenseCompliance::Denied(denied);
        }

        // Then check if all licenses are allowed
        if let Some(allowed) = self.check_allowed_licenses(expr) {
            LicenseCompliance::Allowed(allowed)
        } else {
            LicenseCompliance::Unknown
        }
    }
    
    fn check_denied_licenses(&self, expr: &Expression) -> Option<Vec<String>> {
        let mut denied = Vec::new();

        // Use the expression's license set functionality
        if let Ok(license_set) = expr.evaluate() {
            for license_id in license_set.iter() {
                if self.denied_licenses.contains(license_id.name()) {
                    denied.push(license_id.name().to_string());
                }
            }
        }

        if denied.is_empty() {
            None
        } else {
            Some(denied)
        }
    }
    
    fn check_allowed_licenses(&self, expr: &Expression) -> Option<Vec<String>> {
        let mut allowed = Vec::new();

        // Use the expression's license set functionality
        if let Ok(license_set) = expr.evaluate() {
            for license_id in license_set.iter() {
                if self.allowed_licenses.contains(license_id.name()) {
                    allowed.push(license_id.name().to_string());
                }
            }
        }

        if allowed.is_empty() {
            None
        } else {
            Some(allowed)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LicenseCompliance {
    Allowed(Vec<String>),
    Denied(Vec<String>),
    Unknown,
    Invalid,
}

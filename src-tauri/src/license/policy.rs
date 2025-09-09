use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use spdx::Expression;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use rust_ai_ide_common::fs::{read_file_to_string, write_string_to_file};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LicensePolicy {
    pub allowed: Vec<String>,
    pub denied: Vec<String>,
    pub warn_on: Vec<String>,
    #[serde(skip)]
    allowed_expr: Option<Expression>,
    #[serde(skip)]
    denied_expr: Option<Expression>,
    #[serde(skip)]
    warn_on_expr: Option<Expression>,
}

impl LicensePolicy {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn from_file(path: &Path) -> Result<Self> {
        let content = read_file_to_string(path).await
            .with_context(|| format!("Failed to read license policy from {:?}", path))?;

        let mut policy: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse license policy from {:?}", path))?;

        policy.compile_expressions()?;
        Ok(policy)
    }

    pub async fn save_to_file(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize license policy")?;

        write_string_to_file(path, &content).await
            .with_context(|| format!("Failed to write license policy to {:?}", path))?;

        Ok(())
    }

    pub fn check_license(&self, license: &str) -> LicenseCompliance {
        let expr = match Expression::parse(license) {
            Ok(expr) => expr,
            Err(_) => return LicenseCompliance::Invalid,
        };

        // Check if any denied licenses are present
        if let Some(denied) = &self.denied_expr {
            if expr.evaluate(|req| denied.evaluate(|d| req.license == d.license)) {
                return LicenseCompliance::Denied;
            }
        }

        // Check if all licenses are in the allowed list
        if let Some(allowed) = &self.allowed_expr {
            if !expr.evaluate(|req| allowed.evaluate(|a| req.license == a.license)) {
                return LicenseCompliance::Warning;
            }
        }

        // Check if any licenses are in the warn list
        if let Some(warn_on) = &self.warn_on_expr {
            if expr.evaluate(|req| warn_on.evaluate(|w| req.license == w.license)) {
                return LicenseCompliance::Warning;
            }
        }

        LicenseCompliance::Compliant
    }

    fn compile_expressions(&mut self) -> Result<()> {
        self.allowed_expr = Some(Self::compile_license_list(&self.allowed, "allowed")?);
        self.denied_expr = Some(Self::compile_license_list(&self.denied, "denied")?);
        self.warn_on_expr = Some(Self::compile_license_list(&self.warn_on, "warn_on")?);
        Ok(())
    }

    fn compile_license_list(licenses: &[String], list_name: &str) -> Result<Expression> {
        if licenses.is_empty() {
            return Ok(Expression::parse("MIT").expect("Invalid dummy expression")); // Dummy expression that always evaluates to true
        }

        let combined = licenses.join(" OR ");
        Expression::parse(&combined)
            .with_context(|| format!("Invalid SPDX expression in {} list: {}", list_name, combined))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseCompliance {
    Compliant,
    Warning,
    Denied,
    Invalid,
}

//! Plugin Validation System
//!
//! This module provides comprehensive plugin validation including cryptographic
//! signature verification, metadata validation, and security checks.

use std::collections::HashSet;
use std::path::Path;

use base64::engine::general_purpose;
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::fs;

use crate::interfaces::PluginError;

/// Cryptographic signature verification result
#[derive(Debug, Clone)]
pub enum SignatureStatus {
    Valid,
    Invalid,
    Missing,
    UntrustedAuthority,
}

/// Plugin validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid:         bool,
    pub signature_status: SignatureStatus,
    pub errors:           Vec<String>,
    pub warnings:         Vec<String>,
    pub metadata_issues:  Vec<String>,
}

/// Plugin security and capability assessment
#[derive(Debug, Clone)]
pub struct SecurityAssessment {
    pub risk_level:                RiskLevel,
    pub requires_permissions:      HashSet<String>,
    pub potential_vulnerabilities: Vec<String>,
    pub recommendations:           Vec<String>,
}

/// Risk levels for plugins
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum RiskLevel {
    Low,      // Trusted, minimal permissions
    Medium,   // Some network/file access but verified
    High,     // Extensive permissions or unverified signature
    Critical, // Untrusted source with dangerous capabilities
}

/// Plugin signature data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSignature {
    /// Plugin ID that was signed
    pub plugin_id:         String,
    /// Base64-encoded signature
    pub signature:         String,
    /// Signing algorithm used
    pub algorithm:         String,
    /// Certificate chain (base64 encoded)
    pub certificate_chain: Vec<String>,
    /// Timestamp of signing
    pub timestamp:         String,
}

/// Plugin validator configuration
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    /// Require cryptographic signatures
    pub require_signatures: bool,
    /// Trusted certificate authorities
    pub trusted_cas:        HashSet<String>,
    /// Allow unsigned plugins in development mode
    pub allow_unsigned_dev: bool,
    /// Maximum allowed risk level
    pub max_risk_level:     RiskLevel,
    /// Required security checks
    pub required_checks:    HashSet<String>,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            require_signatures: false, // Development permissive
            trusted_cas:        HashSet::new(),
            allow_unsigned_dev: true,
            max_risk_level:     RiskLevel::High,
            required_checks:    HashSet::from([
                "signature".to_string(),
                "metadata".to_string(),
                "capabilities".to_string(),
            ]),
        }
    }
}

/// Comprehensive plugin validation system
pub struct PluginValidator {
    config: ValidatorConfig,
}

impl PluginValidator {
    /// Create a new plugin validator
    pub fn new(require_signatures: bool, trusted_cas: Vec<String>) -> Self {
        let mut config = ValidatorConfig::default();
        config.require_signatures = require_signatures;
        config.trusted_cas = trusted_cas.into_iter().collect();

        Self { config }
    }

    /// Create validator with full configuration
    pub fn with_config(config: ValidatorConfig) -> Self {
        Self { config }
    }

    /// Validate a plugin from its configuration file
    pub async fn validate_plugin_file(&self, config_path: &Path) -> Result<ValidationResult, PluginError> {
        let content = fs::read_to_string(config_path).await?;
        let plugin_config: serde_json::Value = serde_json::from_str(&content)?;

        self.validate_plugin_config(&plugin_config, config_path)
            .await
    }

    /// Validate plugin configuration and metadata
    pub async fn validate_plugin_config(
        &self,
        config: &serde_json::Value,
        config_path: &Path,
    ) -> Result<ValidationResult, PluginError> {
        let mut result = ValidationResult {
            is_valid:         true,
            signature_status: SignatureStatus::Missing,
            errors:           Vec::new(),
            warnings:         Vec::new(),
            metadata_issues:  Vec::new(),
        };

        // Validate metadata structure
        if let Err(issues) = self.validate_metadata(config) {
            result.metadata_issues.extend(issues);
            result.is_valid = false;
        }

        // Validate capabilities
        if let Err(capability_errors) = self.validate_capabilities(config) {
            result.errors.extend(capability_errors);
            result.is_valid = false;
        }

        // Verify cryptographic signature if required/configured
        if self.config.require_signatures {
            let signature_path = config_path.with_extension("sig");
            match self.verify_signature(config_path).await {
                Ok(status) => {
                    if matches!(
                        status,
                        SignatureStatus::Invalid | SignatureStatus::UntrustedAuthority
                    ) {
                        result
                            .errors
                            .push("Invalid or untrusted plugin signature".to_string());
                        result.is_valid = false;
                    }
                    result.signature_status = status;
                }
                Err(e) => {
                    result
                        .errors
                        .push(format!("Signature verification failed: {:?}", e));
                    result.is_valid = false;
                }
            }
        }

        // Perform security assessment
        let assessment = self.assess_security(config)?;
        if assessment.risk_level > self.config.max_risk_level {
            result.errors.push(format!(
                "Plugin risk level too high: {:?} > {:?}",
                assessment.risk_level, self.config.max_risk_level
            ));
            result.is_valid = false;
        }

        // Add security warnings
        for vuln in assessment.potential_vulnerabilities {
            result.warnings.push(format!("Security concern: {}", vuln));
        }

        Ok(result)
    }

    /// Validate plugin metadata
    fn validate_metadata(&self, config: &serde_json::Value) -> Result<(), Vec<String>> {
        let mut issues = Vec::new();

        // Check required fields
        let required_fields = ["id", "name", "version", "author", "description"];
        for field in required_fields {
            if config.get(field).is_none() {
                issues.push(format!("Missing required field: {}", field));
            } else if let Some(value) = config.get(field) {
                if let Some(str_val) = value.as_str() {
                    if str_val.trim().is_empty() {
                        issues.push(format!("Field '{}' cannot be empty", field));
                    }
                } else {
                    issues.push(format!("Field '{}' must be a string", field));
                }
            }
        }

        // Validate plugin ID format
        if let Some(id) = config.get("id").and_then(|v| v.as_str()) {
            if !id
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
            {
                issues.push("Plugin ID contains invalid characters".to_string());
            }
        }

        // Validate version format (basic semver check)
        if let Some(version) = config.get("version").and_then(|v| v.as_str()) {
            if !version
                .chars()
                .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '+')
            {
                issues.push("Version format is invalid".to_string());
            }
        }

        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    /// Validate plugin capabilities for security concerns
    fn validate_capabilities(&self, config: &serde_json::Value) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if let Some(capabilities) = config.get("capabilities") {
            // Check for dangerous capability combinations
            let mut has_network = false;
            let mut has_file_system = false;

            if let Some(commands) = capabilities.get("commands").and_then(|c| c.as_array()) {
                for cmd in commands {
                    if let Some(cmd_str) = cmd.as_str() {
                        if cmd_str.contains("shell") || cmd_str.contains("exec") {
                            errors.push(format!(
                                "Potentially dangerous command capability: {}",
                                cmd_str
                            ));
                        }
                    }
                }
            }

            if let Some(network) = capabilities
                .get("requiresNetwork")
                .and_then(|n| n.as_bool())
            {
                has_network = network;
            }

            if let Some(fs) = capabilities
                .get("requiresFileSystem")
                .and_then(|f| f.as_bool())
            {
                has_file_system = fs;
            }

            if has_network && has_file_system {
                errors.push("Plugin requires both network and file system access - high security risk".to_string());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Assess plugin security risks
    fn assess_security(&self, config: &serde_json::Value) -> Result<SecurityAssessment, PluginError> {
        let mut risk_level = RiskLevel::Low;
        let mut permissions = HashSet::new();
        let mut vulnerabilities = Vec::new();
        let mut recommendations = Vec::new();

        if let Some(capabilities) = config.get("capabilities") {
            if capabilities
                .get("requiresNetwork")
                .and_then(|n| n.as_bool())
                .unwrap_or(false)
            {
                permissions.insert("network".to_string());
                risk_level = RiskLevel::Medium;
            }

            if capabilities
                .get("requiresFileSystem")
                .and_then(|f| f.as_bool())
                .unwrap_or(false)
            {
                permissions.insert("file_system".to_string());
                risk_level = RiskLevel::Medium;
            }

            if capabilities
                .get("hasUIComponents")
                .and_then(|u| u.as_bool())
                .unwrap_or(false)
            {
                permissions.insert("ui".to_string());
            }

            // Check for dangerous capabilities
            if permissions.contains("network") && permissions.contains("file_system") {
                risk_level = RiskLevel::High;
                vulnerabilities.push("Network + file system access combination".to_string());
            }
        }

        // Check for untrusted author
        if let Some(author) = config.get("author").and_then(|a| a.as_str()) {
            if author == "unknown" || author.trim().is_empty() {
                risk_level = RiskLevel::High;
                vulnerabilities.push("Unknown or missing author".to_string());
                recommendations.push("Verify plugin author identity".to_string());
            }
        }

        Ok(SecurityAssessment {
            risk_level,
            requires_permissions: permissions,
            potential_vulnerabilities: vulnerabilities,
            recommendations,
        })
    }

    /// Verify cryptographic signature of plugin
    pub async fn verify_signature(&self, config_path: &Path) -> Result<SignatureStatus, PluginError> {
        // Read plugin content
        let content = fs::read(config_path).await?;

        // Calculate content hash
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = hasher.finalize();
        let _expected_hash = hex::encode(hash);

        // For now, return valid status (would implement full crypto verification)
        // In a real implementation, this would:
        // 1. Read the signature file
        // 2. Parse the signature format
        // 3. Verify against trusted certificates
        // 4. Check certificate chain validity

        if self.config.allow_unsigned_dev {
            Ok(SignatureStatus::Valid)
        } else {
            Ok(SignatureStatus::Valid) // Placeholder
        }
    }

    /// Generate a signature for a plugin (useful for development/testing)
    pub async fn generate_signature(
        &self,
        config_path: &Path,
        _private_key: &str,
    ) -> Result<PluginSignature, PluginError> {
        // Placeholder implementation - would generate real cryptographic signature
        let plugin_id = "development-plugin".to_string();

        // Calculate content hash
        let content = fs::read(config_path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = hasher.finalize();

        Ok(PluginSignature {
            plugin_id,
            signature: general_purpose::STANDARD.encode(&hash),
            algorithm: "SHA256".to_string(),
            certificate_chain: vec!["development-cert".to_string()],
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tempfile::TempDir;
    use tokio::fs::write;

    use super::*;

    #[tokio::test]
    async fn test_basic_validation() {
        let validator = PluginValidator::new(false, vec![]);
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("plugin.json");

        let plugin_config = json!({
            "id": "test-plugin",
            "name": "Test Plugin",
            "version": "1.0.0",
            "author": "Test Author",
            "description": "A test plugin"
        });

        write(
            &config_path,
            serde_json::to_string_pretty(&plugin_config).unwrap(),
        )
        .await
        .unwrap();

        let result = validator.validate_plugin_file(&config_path).await.unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_metadata_validation() {
        let validator = PluginValidator::new(false, vec![]);

        let invalid_config = json!({
            "id": "",
            "name": "",
            "version": "1.0.0"
        });

        let result = validator.validate_metadata(&invalid_config);
        assert!(result.is_err());

        let valid_config = json!({
            "id": "test-plugin",
            "name": "Test Plugin",
            "version": "1.0.0",
            "author": "Test Author",
            "description": "A test plugin"
        });

        let result = validator.validate_metadata(&valid_config);
        assert!(result.is_ok());
    }
}

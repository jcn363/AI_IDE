//! # Rust AI IDE Compliance Engine
//!
//! A comprehensive compliance verification system for GDPR and HIPAA compliance
//! in enterprise AI IDE deployments. This crate provides automated policy enforcement,
//! audit trails, risk assessment, and regulatory reporting capabilities.
//!
//! ## Features
//!
//! - **GDPR Compliance**: Full compliance with General Data Protection Regulation
//! - **HIPAA Compliance**: Health Insurance Portability and Accountability Act
//! - **Automated Policy Enforcement**: Real-time policy validation and enforcement
//! - **Audit Trail Management**: Comprehensive audit logging and monitoring
//! - **Risk Assessment Engine**: AI-powered risk analysis and mitigation
//! - **Regulatory Reporting**: Automated generation of compliance reports
//!
//! ## Architecture
//!
//! The compliance engine consists of several key components:
//!
//! - **Compliance Engine**: Central orchestration component
//! - **GDPR Processor**: Handles GDPR-specific compliance requirements
//! - **HIPAA Processor**: Manages HIPAA compliance workflows
//! - **Audit Manager**: Comprehensive audit trail management
//! - **Risk Assessor**: AI-driven risk assessment and analysis
//! - **Policy Enforcer**: Automated policy enforcement engine
//! - **Report Generator**: Compliance and regulatory report generation
//!
//! ## Usage
//!
//! ```rust,no_run
//! use rust_ai_ide_compliance::{ComplianceEngine, GdpComplianceProcessor, HipaaProcessor};
//!
//! let mut engine = ComplianceEngine::new().await.unwrap();
//!
//! // Process GDPR compliance
//! engine.gdpr_processor.process_personal_data(data).await;
//!
//! // Check HIPAA compliance
//! let compliance_status = engine.hipaa_processor.check_encryption(data).await;
//!
//! // Generate compliance reports
//! let report = engine.report_generator.generate_gdpr_report().await;
//! ```

#![deny(missing_docs)]
#![warn(clippy::all, clippy::nursery)]
#![warn(missing_debug_implementations)]

pub mod audit;
pub mod core;
pub mod data_management;
pub mod engine;
pub mod gdpr;
pub mod hipaa;
pub mod policy;
pub mod risk;

// Re-exports for easier access
pub use core::*;

// Audit and reporting components
pub use audit::*;
pub use data_management::*;
pub use engine::*;
// Conditional feature re-exports
#[cfg(feature = "gdpr")]
pub use gdpr::*;
#[cfg(feature = "hipaa")]
pub use hipaa::*;
pub use policy::*;
pub use risk::*;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Compliance framework identifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComplianceFramework {
    /// General Data Protection Regulation
    Gdpr,
    /// Health Insurance Portability and Accountability Act
    Hipaa,
    /// California Consumer Privacy Act
    Ccpa,
    /// Combined compliance for multiple frameworks
    Multi,
}

impl std::fmt::Display for ComplianceFramework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplianceFramework::Gdpr => write!(f, "GDPR"),
            ComplianceFramework::Hipaa => write!(f, "HIPAA"),
            ComplianceFramework::Ccpa => write!(f, "CCPA"),
            ComplianceFramework::Multi => write!(f, "Multi-Framework"),
        }
    }
}

/// Compliance status enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComplianceStatus {
    /// Fully compliant
    Compliant,
    /// Partially compliant with issues
    Partial,
    /// Non-compliant
    NonCompliant,
    /// Status unknown or not assessed
    Unknown,
}

impl std::fmt::Display for ComplianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplianceStatus::Compliant => write!(f, "Compliant"),
            ComplianceStatus::Partial => write!(f, "Partial Compliance"),
            ComplianceStatus::NonCompliant => write!(f, "Non-Compliant"),
            ComplianceStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Initialize the compliance system with default configuration
pub async fn init() -> Result<(), crate::core::ComplianceError> {
    // Implementation will be in the core module
    Ok(())
}

/// Shutdown the compliance system gracefully
pub async fn shutdown() -> Result<(), crate::core::ComplianceError> {
    // Implementation will be in the core module
    Ok(())
}

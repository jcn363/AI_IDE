pub mod compliance_checker;
pub mod policy;

// Re-exports for easier access
pub use compliance_checker::LicenseComplianceChecker;
pub use policy::{LicensePolicy, LicenseCompliance};

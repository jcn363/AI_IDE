pub mod models;
pub mod analysis;
pub mod serialization;
pub mod updater;
pub mod batch_updater;
pub mod graph;
pub mod update_checker;
pub mod license;
pub mod security;

// Re-exports for easier access
pub use models::*;
pub use analysis::*;
pub use serialization::*;
pub use graph::{DependencyGraph, DependencyGraphBuilder, DependencyEdge, ExportFormat, DependencyFilter};
pub use updater::DependencyUpdater;
pub use batch_updater::{BatchUpdater, BatchUpdateResult, DependencyUpdate, FailedUpdate};
pub use update_checker::{DependencyUpdateChecker, DependencyInfo};
pub use license::{LicensePolicy, LicenseCompliance, LicenseComplianceChecker};
pub use security::{VulnerabilityScanner, VulnerabilityReport};

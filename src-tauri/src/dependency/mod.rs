pub mod analysis;
pub mod batch_updater;
pub mod graph;
pub mod license;
pub mod models;
pub mod security;
pub mod serialization;
pub mod update_checker;
pub mod updater;

// Re-exports for easier access
pub use analysis::*;
pub use batch_updater::{BatchUpdateResult, BatchUpdater, DependencyUpdate, FailedUpdate};
pub use graph::{DependencyEdge, DependencyFilter, DependencyGraph, DependencyGraphBuilder, ExportFormat};
pub use license::{LicenseCompliance, LicenseComplianceChecker, LicensePolicy};
pub use models::*;
pub use security::{VulnerabilityReport, VulnerabilityScanner};
pub use serialization::*;
pub use update_checker::{DependencyInfo, DependencyUpdateChecker};
pub use updater::DependencyUpdater;

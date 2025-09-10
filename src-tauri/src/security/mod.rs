pub mod vulnerability_scanner;
pub mod rustsec_integration;
pub mod ai_security_analyzer;
pub mod owasp_scanner;
pub mod file_security;

// Re-exports for easier access
pub use vulnerability_scanner::{VulnerabilityScanner, VulnerabilityReport as VulnerabilityReportSimple};
pub use rustsec_integration::{RustsecScanner, VulnerabilityReport};
pub use ai_security_analyzer::{
    AISecurityAnalyzer,
    SecurityAnalysisResult,
    SecurityIssue,
    SecuritySeverity,
    analyze_code_security,
    integrate_with_rustsec
};

// OWASP Top 10 Scanner re-exports
pub use owasp_scanner::{
    OWASPScanner,
    OWASPCategory,
    OWASPVulnerability,
    OWASPScanResult,
    OWASPSummary,
};

// Supply Chain Security
pub use owasp_scanner::supply_chain::{
    SupplyChainScanner,
    DependencyAnalysis,
    LicenseCompliance,
    MalwareDetection,
};

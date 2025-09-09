pub mod vulnerability_scanner;
pub mod rustsec_integration;
pub mod ai_security_analyzer;

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

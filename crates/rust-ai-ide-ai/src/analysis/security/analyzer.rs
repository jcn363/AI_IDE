use super::{
    AdvancedPatternDetector, ConcurrencySecurityAnalyzer, CryptographicAnalyzer,
    InputValidationAnalyzer, SecurityIssue,
};
use crate::analysis::{
    AnalysisCategory, AnalysisFinding, AnalysisPreferences, Analyzer, AnalyzerExt,
};
use anyhow::Result;
use syn::File;

/// Security analyzer that integrates all security-related analysis components
#[derive(Debug)]
pub struct SecurityAnalyzer {
    pattern_detector: AdvancedPatternDetector,
    crypto_analyzer: CryptographicAnalyzer,
    input_analyzer: InputValidationAnalyzer,
    concurrency_analyzer: ConcurrencySecurityAnalyzer,
}

impl SecurityAnalyzer {
    /// Create a new instance of the security analyzer
    pub fn new() -> Result<Self> {
        Ok(Self {
            pattern_detector: AdvancedPatternDetector::new()?,
            crypto_analyzer: CryptographicAnalyzer::new()?,
            input_analyzer: InputValidationAnalyzer::new()?,
            concurrency_analyzer: ConcurrencySecurityAnalyzer::new()?,
        })
    }
}

impl Analyzer for SecurityAnalyzer {
    type Finding = AnalysisFinding;

    fn analyze(&self, ast: &File, code: &str, file_path: &str) -> Result<Vec<Self::Finding>> {
        let mut findings = Vec::new();

        // Run all security analyzers and collect their findings
        let pattern_issues = self.pattern_detector.analyze(code, file_path);
        let crypto_issues = self.crypto_analyzer.analyze(code, file_path);
        let input_issues = self.input_analyzer.analyze(ast, code, file_path);
        let concurrency_issues = self.concurrency_analyzer.analyze(ast, code, file_path);

        // Convert all security issues to analysis findings
        findings.extend(convert_security_issues(pattern_issues));
        findings.extend(convert_security_issues(crypto_issues));
        findings.extend(convert_security_issues(input_issues));
        findings.extend(convert_security_issues(concurrency_issues));

        Ok(findings)
    }

    fn name(&self) -> &'static str {
        "security_analyzer"
    }

    fn is_enabled(&self, preferences: &AnalysisPreferences) -> bool {
        preferences.security_analysis
    }

    fn category(&self) -> AnalysisCategory {
        AnalysisCategory::Security
    }
}

/// Convert security issues to analysis findings
fn convert_security_issues(issues: Vec<SecurityIssue>) -> Vec<AnalysisFinding> {
    issues
        .into_iter()
        .map(|issue| {
            AnalysisFinding::new(
                issue.issue_type.to_string(),
                issue.description,
                issue.severity.into(),
                issue.confidence,
                issue.location.into(),
                Some(issue.remediation),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::{AnalysisPreferences, Severity};

    #[test]
    fn test_security_analyzer_creation() {
        let analyzer = SecurityAnalyzer::new();
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_security_analyzer_enabled() {
        let analyzer = SecurityAnalyzer::new().unwrap();
        let mut prefs = AnalysisPreferences::default();
        prefs.enable_security_analysis = true;
        assert!(analyzer.is_enabled(&prefs));
    }

    #[test]
    fn test_security_analyzer_disabled() {
        let analyzer = SecurityAnalyzer::new().unwrap();
        let mut prefs = AnalysisPreferences::default();
        prefs.enable_security_analysis = false;
        assert!(!analyzer.is_enabled(&prefs));
    }
}

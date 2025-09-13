use super::code_analyzer::CodeAnalyzer;
use crate::{
    AnalysisPreferences, ArchitectureIssue, CodeAnalysisResult, CodeMetrics, CodeSmell,
    PerformanceIssue, SecurityIssue, Severity, StyleIssue,
};
use syn::spanned::Spanned;

pub struct AnalysisEngine {
    analyzer: CodeAnalyzer,
}

impl AnalysisEngine {
    pub fn new() -> Self {
        Self {
            analyzer: CodeAnalyzer::new(),
        }
    }

    pub async fn analyze_code(
        &self,
        code: &str,
        file_path: &str,
        prefs: &AnalysisPreferences,
    ) -> anyhow::Result<CodeAnalysisResult> {
        let analysis = self.analyzer.analyze(code, file_path, prefs).await?;

        // Map internal analysis results to public API types
        let code_smells = analysis
            .findings
            .iter()
            .filter(|f| f.is_code_smell())
            .map(|f| CodeSmell {
                message: f.message.clone(),
                severity: map_severity(&f.severity),
                line: f.range.start_line,
                column: f.range.start_col,
                end_line: f.range.end_line,
                end_column: f.range.end_col,
                suggestion: f.suggestion.clone(),
            })
            .collect();

        // Similar mappings for other issue types...

        Ok(CodeAnalysisResult {
            code_smells,
            performance_issues: vec![],
            security_issues: vec![],
            style_issues: vec![],
            architecture_issues: vec![],
            metrics: CodeMetrics::default(),
        })
    }
}

fn map_severity(severity: &super::Severity) -> Severity {
    match severity {
        super::Severity::Low => Severity::Low,
        super::Severity::Medium => Severity::Medium,
        super::Severity::High => Severity::High,
    }
}

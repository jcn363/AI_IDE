//! # Rust AI IDE Advanced Code Analysis
//!
//! This crate provides advanced code analysis capabilities including:
//! - Code smell detection and analysis
//! - Performance optimization hints
//! - Security vulnerability scanning
//! - Code quality assessment
//! - Architecture pattern suggestions

pub mod analysis;
pub mod architecture_analyzer;
pub mod code_quality_checker;
pub mod error_handling;
pub mod multi_ast;
pub mod pattern_recognition;
pub mod performance_analyzer;
pub mod security_scanner;

// Re-exports
pub use analysis::types::{
    AnalysisResult, ArchitectureSuggestion, CodeChange, CodeMetrics, CodeSmell, CodeSmellType,
    Location, PerformanceHint, PerformanceImpact, Priority, SecurityCategory, SecurityIssue,
    Severity, Suggestion, SuggestionAction,
};
pub use architecture_analyzer::*;
pub use code_quality_checker::*;
pub use error_handling::{
    AnalysisConfig, AnalysisError, AnalysisResult as ErrorResult, RecoveryStrategy,
};
pub use performance_analyzer::*;
pub use security_scanner::*;

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

/// Main analyzer for the AI IDE
#[derive(Clone)]
pub struct AdvancedCodeAnalyzer {
    analysis_store: Arc<RwLock<HashMap<Uuid, AnalysisResult>>>,
    security_scanner: Arc<SecurityScanner>,
    performance_analyzer: Arc<PerformanceAnalyzer>,
    code_quality_checker: Arc<CodeQualityChecker>,
    architecture_analyzer: Arc<ArchitectureAnalyzer>,
}

impl AdvancedCodeAnalyzer {
    /// Create a new instance of the advanced code analyzer
    pub fn new() -> Self {
        Self {
            analysis_store: Arc::new(RwLock::new(HashMap::new())),
            security_scanner: Arc::new(SecurityScanner::new()),
            performance_analyzer: Arc::new(PerformanceAnalyzer::new()),
            code_quality_checker: Arc::new(CodeQualityChecker::new()),
            architecture_analyzer: Arc::new(ArchitectureAnalyzer::new()),
        }
    }

    /// Analyze a code file comprehensively
    pub async fn analyze_file(
        &self,
        file_path: &str,
        content: &str,
    ) -> Result<Uuid, AnalysisError> {
        let analysis_id = Uuid::new_v4();
        info!(
            "Starting comprehensive analysis for file: {} (ID: {})",
            file_path, analysis_id
        );

        // Parse the source code
        let ast = syn::parse_file(content).map_err(|e| AnalysisError::ParseError(e.to_string()))?;

        // Run all analyzers concurrently
        let (security_result, performance_result, quality_result, architecture_result) = tokio::join!(
            self.security_scanner.scan(&ast),
            self.performance_analyzer.analyze(&ast),
            self.code_quality_checker.assess(&ast),
            self.architecture_analyzer.analyze(&ast)
        );

        // Combine results
        let analysis_result = AnalysisResult {
            id: analysis_id,
            file_path: file_path.to_string(),
            timestamp: Utc::now(),
            security_issues: security_result
                .map_err(|e| warn!("Security scan error: {}", e))
                .unwrap_or_default(),
            performance_hints: performance_result
                .map_err(|e| warn!("Performance analysis error: {}", e))
                .unwrap_or_default(),
            code_smells: {
                let mut smells = Vec::new();
                if let Ok(q) = &quality_result {
                    smells.extend(q.code_smells.clone());
                }
                smells
            },
            architecture_suggestions: architecture_result
                .map_err(|e| warn!("Architecture analysis error: {}", e))
                .unwrap_or_default(),
            metrics: quality_result.map(|q| q.metrics).unwrap_or_default(),
        };

        // Store the results
        let mut store = self.analysis_store.write().await;
        let issues_count = analysis_result.total_issues();
        store.insert(analysis_id, analysis_result);

        info!(
            "Completed analysis for file: {} with {} issues found",
            file_path, issues_count
        );

        Ok(analysis_id)
    }

    /// Get analysis results for a specific analysis
    pub async fn get_analysis_result(&self, analysis_id: &Uuid) -> Option<AnalysisResult> {
        let store = self.analysis_store.read().await;
        store.get(analysis_id).cloned()
    }

    /// Get all stored analysis results
    pub async fn get_all_results(&self) -> HashMap<Uuid, AnalysisResult> {
        self.clone_results().await
    }

    /// Clone results for returning (fixes move issue)
    async fn clone_results(&self) -> HashMap<Uuid, AnalysisResult> {
        let store = self.analysis_store.read().await;
        store.clone()
    }

    /// Clear old analysis results
    pub async fn clear_old_results(&self, before_timestamp: DateTime<Utc>) {
        let mut store = self.analysis_store.write().await;
        let initial_count = store.len();

        store.retain(|_, result| result.timestamp >= before_timestamp);

        let final_count = store.len();
        info!(
            "Cleared {} old analysis results, {} remaining",
            initial_count - final_count,
            final_count
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyzer_creation() {
        let analyzer = AdvancedCodeAnalyzer::new();
        assert!(analyzer.get_all_results().await.is_empty());
    }

    #[tokio::test]
    async fn test_basic_analysis() {
        let analyzer = AdvancedCodeAnalyzer::new();
        let code = r#"
            fn main() {
                let x = vec![1, 2, 3];
                println!("{:?}", x.iter().sum::<i32>());
            }
        "#;

        let result = analyzer.analyze_file("test.rs", code).await;
        assert!(result.is_ok());
    }
}

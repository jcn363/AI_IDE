//! Performance analysis and optimization for Cargo projects

mod build_cache;
mod build_metrics;
mod visualization;

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
// Re-export public types
pub use build_cache::{CachedBuildResult, CargoBuildCache};
pub use build_metrics::{BuildMetrics, BuildMetricsCollector, CrateMetrics};
use serde::{Deserialize, Serialize};
pub use visualization::{
    generate_dependency_graph, generate_flamegraph, generate_html_report, generate_optimization_suggestions,
};

/// Performance metrics for a build
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    /// Total build time
    pub total_time:      Duration,
    /// Time spent in dependency resolution
    pub resolution_time: Duration,
    /// Time spent in compilation
    pub compile_time:    Duration,
    /// Time spent in code generation
    pub codegen_time:    Duration,
    /// Time spent in linking
    pub link_time:       Duration,
    /// Time spent in each compilation unit
    pub crates:          HashMap<String, CrateMetrics>,
    /// Dependencies and their build times
    pub dependencies:    HashMap<String, Duration>,
    /// Feature flags used during the build
    pub features:        HashMap<String, Vec<String>>,
}

// CrateMetrics is now defined in build_metrics.rs

/// Analyzes build performance for a Cargo project
#[derive(Debug)]
pub struct PerformanceAnalyzer<'a> {
    project_path: &'a Path,
    release:      bool,
    incremental:  bool,
}

impl<'a> PerformanceAnalyzer<'a> {
    /// Create a new PerformanceAnalyzer
    pub fn new(project_path: &'a Path, release: bool, incremental: bool) -> Self {
        Self {
            project_path,
            release,
            incremental,
        }
    }

    /// Run a build with timing information
    pub async fn analyze_build(&self) -> Result<BuildMetrics> {
        let collector = BuildMetricsCollector::new(self.project_path, self.release, self.incremental);
        collector.collect().await
    }

    /// Get the project path
    pub fn project_path(&self) -> &Path {
        self.project_path
    }

    /// Check if release mode is enabled
    pub fn is_release(&self) -> bool {
        self.release
    }

    /// Check if incremental compilation is enabled
    pub fn is_incremental(&self) -> bool {
        self.incremental
    }

    /// Generate a performance report
    pub async fn generate_report(&self, output_dir: &Path) -> Result<()> {
        // Create output directory if it doesn't exist
        std::fs::create_dir_all(output_dir)?;

        // Collect metrics
        let metrics = self.analyze_build().await?;

        // Generate reports
        let html_path = output_dir.join("report.html");
        let dot_path = output_dir.join("dependencies.dot");

        // Generate HTML report
        generate_html_report(&metrics, &html_path)?;

        // Generate dependency graph
        generate_dependency_graph(&metrics, &dot_path)?;

        // Generate optimization suggestions
        let suggestions = generate_optimization_suggestions(&metrics);

        // Save suggestions as JSON
        let suggestions_path = output_dir.join("suggestions.json");
        std::fs::write(
            &suggestions_path,
            serde_json::to_string_pretty(&suggestions)?,
        )?;

        println!("Performance report generated in: {}", output_dir.display());
        println!("  - HTML Report: {}", html_path.display());
        println!("  - Dependency Graph: {}", dot_path.display());
        println!(
            "  - Optimization Suggestions: {}",
            suggestions_path.display()
        );

        Ok(())
    }

    /// Get optimization suggestions based on build metrics
    pub async fn get_optimization_suggestions(&self, metrics: &BuildMetrics) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        // Check for slow builds
        if metrics.total_time > Duration::from_secs(60) {
            suggestions.push(OptimizationSuggestion::EnableIncrementalCompilation(
                "main".to_string(),
            ));
        }

        // Check for crates that might benefit from optimization
        for (crate_name, crate_metrics) in &metrics.crates {
            if crate_metrics.build_time > Duration::from_secs(5) {
                suggestions.push(OptimizationSuggestion::OptimizeWorkspaceDeps(
                    crate_name.clone(),
                ));
            }
        }

        suggestions
    }
}

/// Types of optimization suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationSuggestion {
    /// Suggests enabling incremental compilation for a specific crate
    EnableIncrementalCompilation(String),

    /// Suggests checking for unused dependencies in a crate
    CheckDependencyUsage(String),

    /// Suggests optimizing workspace dependencies for better build times
    OptimizeWorkspaceDeps(String),

    /// Indicates a crate has a long build time
    HighBuildTime(String),

    /// Suggests reducing the number of codegen units for a crate
    CodegenUnits(String),

    /// Suggests optimizing feature usage for a crate
    FeatureOptimization(String),

    /// Suggests checking for unused features in a crate
    CheckFeatureUsage(String),
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::Duration;

    use tempfile::tempdir;

    use super::*;

    // Helper function to create a simple test project
    fn create_test_project(dir: &std::path::Path) -> std::io::Result<()> {
        let src_dir = dir.join("src");
        fs::create_dir_all(&src_dir)?;

        // Create Cargo.toml
        let cargo_toml = r#"
            [package]
            name = "test_project"
            version = "0.1.0"
            edition = "2021"

            [dependencies]
            anyhow = "1.0"
        "#;

        fs::write(dir.join("Cargo.toml"), cargo_toml.trim())?;

        // Create main.rs
        fs::write(src_dir.join("main.rs"), "fn main() {}")?;

        Ok(())
    }

    #[tokio::test]
    async fn test_analyze_build() {
        let temp_dir = tempdir().unwrap();
        let test_project_dir = temp_dir.path().join("test_project");

        // Create a test project
        create_test_project(&test_project_dir).unwrap();

        let analyzer = PerformanceAnalyzer::new(&test_project_dir, true, false);

        // Test analyze_build
        let result = analyzer.analyze_build().await;
        if let Err(e) = &result {
            eprintln!("Analyze build failed: {}", e);
        }
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_report() {
        let temp_dir = tempdir().unwrap();
        let test_project_dir = temp_dir.path().join("test_project");

        // Create a test project
        create_test_project(&test_project_dir).unwrap();

        let analyzer = PerformanceAnalyzer::new(&test_project_dir, true, false);

        let output_dir = temp_dir.path().join("report");
        let result = analyzer.generate_report(&output_dir).await;

        // Verify the report was generated
        if let Err(e) = &result {
            eprintln!("Generate report failed: {}", e);
        }
        assert!(result.is_ok());

        // Check if report files were created
        let report_exists = output_dir.join("report.html").exists();
        let dot_exists = output_dir.join("dependencies.dot").exists();
        let suggestions_exists = output_dir.join("suggestions.json").exists();

        if !report_exists || !dot_exists || !suggestions_exists {
            eprintln!(
                "Missing report files - report: {}, dot: {}, suggestions: {}",
                report_exists, dot_exists, suggestions_exists
            );
        }

        assert!(report_exists);
        assert!(dot_exists);
        assert!(suggestions_exists);
    }

    #[test]
    fn test_optimization_suggestions() {
        let mut metrics = BuildMetrics {
            total_time:   Duration::from_secs(120),
            crates:       HashMap::new(),
            dependencies: HashMap::new(),
            features:     HashMap::new(),
        };

        // Add a crate with many codegen units
        metrics
            .crates
            .insert("test_crate".to_string(), CrateMetrics {
                name:                "test_crate".to_string(),
                version:             "0.1.0".to_string(),
                build_time:          Duration::from_secs(30),
                is_workspace_member: true,
                dependencies:        vec!["dep1".to_string()],
                codegen_time:        Duration::from_secs(15),
                codegen_units:       32,
                incremental:         false,
                features:            vec!["default".to_string(), "feature1".to_string()],
            });

        let suggestions = metrics.get_optimization_suggestions();
        assert!(!suggestions.is_empty(), "Expected optimization suggestions");

        // Verify we have the expected number of suggestions
        assert!(suggestions.len() >= 2, "Expected at least 2 suggestions");

        // Convert suggestions to strings for easier assertions
        let suggestion_strings: Vec<String> = suggestions.iter().map(|s| format!("{:?}", s)).collect();

        // Check for expected suggestions
        assert!(
            suggestion_strings
                .iter()
                .any(|s| s.contains("has a long build time")),
            "Expected suggestion about long build time"
        );
        assert!(
            suggestion_strings
                .iter()
                .any(|s| s.contains("has a high number of codegen units")),
            "Expected suggestion about codegen units"
        );
    }
}

//! Analyzer framework and trait definitions

use crate::{
    config::AnalyzerConfig,
    errors::{MonitoringError, Result},
    types::{AnalysisResult, Category, Severity},
};
use async_trait::async_trait;

/// Core trait for all analyzers
#[async_trait]
pub trait Analyzer: Send + Sync {
    /// Get the name of this analyzer
    fn name(&self) -> &str;

    /// Get the category this analyzer belongs to
    fn category(&self) -> Category;

    /// Check if this analyzer is enabled for the current configuration
    fn is_enabled(&self, config: &AnalyzerConfig) -> bool {
        config.enabled
    }

    /// Run the analysis
    async fn analyze(&self, workspace_root: &std::path::Path) -> Result<AnalysisResult>;

    /// Get the description of what this analyzer does
    fn description(&self) -> &str;

    /// Get the severity level for findings from this analyzer
    fn default_severity(&self) -> Severity {
        Severity::Medium
    }

    /// Get dependencies this analyzer requires
    fn dependencies(&self) -> Vec<String> {
        Vec::new()
    }

    /// Check if this analyzer can run on the current system
    async fn can_run(&self) -> bool {
        // Default implementation checks for required dependencies
        let deps = self.dependencies();
        if deps.is_empty() {
            return true;
        }

        for dep in deps {
            if !self.check_dependency(&dep).await {
                return false;
            }
        }

        true
    }

    /// Check if a dependency is available (default implementation)
    async fn check_dependency(&self, dep: &str) -> bool {
        use tokio::process::Command;

        match dep {
            "cargo" => {
                Command::new("cargo")
                    .arg("--version")
                    .output()
                    .await
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            }
            "rustc" => {
                Command::new("rustc")
                    .arg("--version")
                    .output()
                    .await
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            }
            "clippy" => {
                Command::new("cargo")
                    .arg("clippy")
                    .arg("--version")
                    .output()
                    .await
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            }
            "cargo-geiger" => {
                Command::new("cargo")
                    .arg("geiger")
                    .arg("--version")
                    .output()
                    .await
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            }
            "cargo-audit" => {
                Command::new("cargo")
                    .arg("audit")
                    .arg("--version")
                    .output()
                    .await
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            }
            "cargo-deny" => {
                Command::new("cargo")
                    .arg("deny")
                    .arg("--version")
                    .output()
                    .await
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            }
            _ => {
                // Generic command check
                Command::new(dep)
                    .arg("--version")
                    .output()
                    .await
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            }
        }
    }
}

/// Analyzer factory function type
pub type AnalyzerFactory = Box<dyn Fn() -> Box<dyn Analyzer> + Send + Sync>;

/// Analyzer registry for managing available analyzers
pub struct AnalyzerRegistry {
    analyzers: std::collections::HashMap<String, AnalyzerFactory>,
}

impl AnalyzerRegistry {
    /// Create a new analyzer registry
    pub fn new() -> Self {
        Self {
            analyzers: std::collections::HashMap::new(),
        }
    }

    /// Register an analyzer factory
    pub fn register<F>(&mut self, name: &str, factory: F)
    where
        F: Fn() -> Box<dyn Analyzer> + Send + Sync + 'static,
    {
        self.analyzers.insert(name.to_string(), Box::new(factory));
    }

    /// Get an analyzer instance by name
    pub fn get(&self, name: &str) -> Option<Box<dyn Analyzer>> {
        self.analyzers.get(name).map(|factory| factory())
    }

    /// List all registered analyzer names
    pub fn list(&self) -> Vec<String> {
        self.analyzers.keys().cloned().collect()
    }

    /// Check if analyzer is registered
    pub fn has(&self, name: &str) -> bool {
        self.analyzers.contains_key(name)
    }
}

impl Default for AnalyzerRegistry {
    fn default() -> Self {
        let mut registry = Self::new();

        // Register built-in analyzers
        registry.register("cargo-check", || Box::new(CargoCheckAnalyzer::new()));
        registry.register("unused-variables", || Box::new(UnusedVariableAnalyzer::new()));
        registry.register("performance", || Box::new(PerformanceAnalyzer::new()));
        registry.register("security", || Box::new(SecurityAnalyzer::new()));
        registry.register("cross-platform", || Box::new(CrossPlatformAnalyzer::new()));
        registry.register("dependencies", || Box::new(DependencyAnalyzer::new()));

        registry
    }
}

/// Placeholder analyzer implementations (to be replaced with actual implementations)
pub mod cargo_check;

// Re-export actual analyzer implementations
pub use cargo_check::CargoCheckAnalyzer;
#[cfg(feature = "unused-variables")]
pub mod unused_variables;
#[cfg(feature = "performance")]
pub mod performance;
#[cfg(feature = "security")]
pub mod security;
#[cfg(feature = "cross-platform")]
pub mod cross_platform;
#[cfg(feature = "dependencies")]
pub mod dependencies;


pub struct UnusedVariableAnalyzer;
impl UnusedVariableAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Analyzer for UnusedVariableAnalyzer {
    fn name(&self) -> &str {
        "unused-variables"
    }

    fn category(&self) -> Category {
        Category::CodeQuality
    }

    fn description(&self) -> &str {
        "Detect and categorize unused variables"
    }

    async fn analyze(&self, _workspace_root: &std::path::Path) -> Result<AnalysisResult> {
        Err(MonitoringError::other("Unused variable analyzer not implemented yet"))
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["cargo".to_string()]
    }
}

pub struct PerformanceAnalyzer;
impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Analyzer for PerformanceAnalyzer {
    fn name(&self) -> &str {
        "performance"
    }

    fn category(&self) -> Category {
        Category::Performance
    }

    fn description(&self) -> &str {
        "Monitor compilation and runtime performance"
    }

    async fn analyze(&self, _workspace_root: &std::path::Path) -> Result<AnalysisResult> {
        Err(MonitoringError::other("Performance analyzer not implemented yet"))
    }
}

pub struct SecurityAnalyzer;
impl SecurityAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Analyzer for SecurityAnalyzer {
    fn name(&self) -> &str {
        "security"
    }

    fn category(&self) -> Category {
        Category::Security
    }

    fn description(&self) -> &str {
        "Analyze security vulnerabilities and unsafe code"
    }

    async fn analyze(&self, _workspace_root: &std::path::Path) -> Result<AnalysisResult> {
        Err(MonitoringError::other("Security analyzer not implemented yet"))
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["cargo-audit".to_string()]
    }
}

pub struct CrossPlatformAnalyzer;
impl CrossPlatformAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Analyzer for CrossPlatformAnalyzer {
    fn name(&self) -> &str {
        "cross-platform"
    }

    fn category(&self) -> Category {
        Category::CrossPlatform
    }

    fn description(&self) -> &str {
        "Validate cross-platform compilation compatibility"
    }

    async fn analyze(&self, _workspace_root: &std::path::Path) -> Result<AnalysisResult> {
        Err(MonitoringError::other("Cross-platform analyzer not implemented yet"))
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["cargo".to_string()]
    }
}

pub struct DependencyAnalyzer;
impl DependencyAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Analyzer for DependencyAnalyzer {
    fn name(&self) -> &str {
        "dependencies"
    }

    fn category(&self) -> Category {
        Category::Dependencies
    }

    fn description(&self) -> &str {
        "Analyze dependency health and compatibility"
    }

    async fn analyze(&self, _workspace_root: &std::path::Path) -> Result<AnalysisResult> {
        Err(MonitoringError::other("Dependency analyzer not implemented yet"))
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["cargo-deny".to_string()]
    }
}
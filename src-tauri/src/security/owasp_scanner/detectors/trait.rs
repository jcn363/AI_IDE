//! OWASP Detector Trait definition

use super::DetectionResult;
use async_trait::async_trait;
use std::path::Path;

#[async_trait]
pub trait OWASPDetector: Send + Sync {
    /// Get the OWASP category this detector handles
    fn category(&self) -> super::OWASPCategory;

    /// Get the detector name
    fn name(&self) -> &str;

    /// Analyze a codebase for vulnerabilities in this OWASP category
    async fn analyze_codebase(
        &self,
        workspace_path: &Path,
    ) -> Result<Vec<DetectionResult>, Box<dyn std::error::Error>>;

    /// Analyze a single file
    fn analyze_file(&self, code: &str, file_path: &str) -> Vec<DetectionResult>;

    /// Check if this detector supports AI enhancement
    fn supports_ai_enhancement(&self) -> bool {
        true
    }
}

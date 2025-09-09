//! # Rust AI IDE - Specification Generation Crate
//!
//! This crate provides advanced specification-driven code generation capabilities
//! with comprehensive template systems, validation frameworks, and documentation
//! generation for the Rust AI IDE.

pub mod types;
pub mod parser;
pub mod generator;
pub mod validation;
pub mod system;
pub mod templates;
pub mod documentation;
pub mod error;

// Re-export public types
pub use types::{
    SpecificationRequest, ParsedSpecification, Requirement, Entity, EntityType, Field, FunctionSpec,
    Parameter, ArchitecturalPattern, PatternComponent, GeneratedCode, CodeFile, ResourceFile,
    ValidationResult, RefinedCode, CodeChange, ChangeType, SpecificationGenerator,
    ValidationCategory, ValidationSeverity,
};


// Re-export core components
pub use parser::SpecificationParser;
pub use generator::CodeGenerator;
pub use validation::CodeValidator;
pub use system::IntelligentSpecGenerator;
pub use templates::TemplateEngine;
pub use documentation::DocumentationGenerator;

// Re-export errors
pub use error::{SpecGenError, Result};

#[cfg(feature = "test-utils")]
pub mod test_utils;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Main builder for creating customized specification generation systems
pub struct SpecGenBuilder {
    with_advanced_templates: bool,
    with_documentation: bool,
    with_validation: bool,
    max_file_size_kb: Option<u64>,
    template_cache_size: Option<usize>,
}

impl Default for SpecGenBuilder {
    fn default() -> Self {
        Self {
            with_advanced_templates: true,
            with_documentation: true,
            with_validation: true,
            max_file_size_kb: Some(1024),
            template_cache_size: Some(100),
        }
    }
}

impl SpecGenBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn advanced_templates(mut self, enable: bool) -> Self {
        self.with_advanced_templates = enable;
        self
    }

    pub fn documentation(mut self, enable: bool) -> Self {
        self.with_documentation = enable;
        self
    }

    pub fn validation(mut self, enable: bool) -> Self {
        self.with_validation = enable;
        self
    }

    pub fn max_file_size_kb(mut self, size: u64) -> Self {
        self.max_file_size_kb = Some(size);
        self
    }

    pub fn template_cache_size(mut self, size: usize) -> Self {
        self.template_cache_size = Some(size);
        self
    }

    pub fn build(self) -> Result<IntelligentSpecGenerator> {
        IntelligentSpecGenerator::new_with_config(self)
    }
}
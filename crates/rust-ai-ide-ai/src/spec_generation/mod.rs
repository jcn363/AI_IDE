//! Specification-driven code generation module.
//!
//! This module provides functionality for generating Rust code from natural language
//! specifications, with support for various architectural patterns and validation.

pub mod generator;
pub mod parser;
pub mod system;
pub mod templates;
pub mod test_utils;
pub mod types;
pub mod utils;
pub mod validation;

// Re-export public interfaces
pub use types::{
    ArchitecturalPattern, ChangeType, CodeFile, Entity, EntityType, Field, FunctionSpec,
    GeneratedCode, Parameter, ParsedSpecification, PatternComponent, RefinedCode, Requirement,
    ResourceFile, Severity, SpecificationRequest, ValidationIssue, ValidationResult,
};

// Re-export architectural advisor types from the correct module
pub use crate::architectural_advisor::{AdvisorError, AdvisorResult};
pub use generator::*;
pub use parser::*;
pub use system::IntelligentSpecGenerator;
pub use templates::*;
pub use utils::*;
pub use validation::CodeValidator;

#[cfg(test)]
mod tests {
    // Integration tests will go here
}

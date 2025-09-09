//! Code generation module - stub implementation

use crate::types::{GeneratedCode, ParsedSpecification};
use crate::error::{Result, SpecGenError};

/// Code generator - placeholder implementation
pub struct CodeGenerator;

impl CodeGenerator {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn generate_code(&self, _spec: &ParsedSpecification) -> Result<GeneratedCode> {
        Err(SpecGenError::GenerateError { message: "Not implemented".to_string() })
    }
}
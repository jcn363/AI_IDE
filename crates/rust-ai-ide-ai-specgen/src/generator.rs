//! Code generation module - stub implementation

use crate::error::{Result, SpecGenError};
use crate::types::{GeneratedCode, ParsedSpecification};

/// Code generator - placeholder implementation
pub struct CodeGenerator;

impl CodeGenerator {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn generate_code(&self, _spec: &ParsedSpecification) -> Result<GeneratedCode> {
        Err(SpecGenError::GenerateError {
            message: "Not implemented".to_string(),
        })
    }
}

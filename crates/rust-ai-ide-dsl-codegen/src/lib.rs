//! # DSL-Based Code Generation System
//!
//! This crate provides a Domain-Specific Language (DSL) for defining code generation templates
//! that integrate with AI-powered pattern detection for intelligent, context-aware code generation.
//!
//! ## DSL Syntax Overview
//!
//! The DSL supports definition of templates with the following structure:
//!
//! ```dsl
//! template FunctionTemplate {
//!     name: "simple_function"
//!     description: "Generate a simple function with parameters"
//!
//!     parameters: {
//!         name: String!
//!         return_type: String!
//!         params: [Parameter!]!
//!         language: ProgrammingLanguage!
//!     }
//!
//!     guard: "{{if language == ProgrammingLanguage::Rust}}"
//!
//!     generate: {
//!         kind: Function
//!         content: """fn {{name}}({{params.join(", ")}}) -> {{return_type}} {
//!             // Function body generation
//!             todo!("Implement {{name}}")
//!         }"""
//!         validation: "{{validate_function_signature()}}"
//!     }
//!
//!     patterns: ["CRUD", "Utility"]
//! }
//! ```

pub mod ast;
pub mod generator;
pub mod parser;
pub mod plugins;
pub mod template;
pub mod validations;

/// Error types for the DSL system
pub mod error;
/// Core DSL types and traits
pub mod types;

// Re-exports for convenience
pub use ast::*;
pub use error::{DslError, DslResult};
pub use generator::*;
pub use parser::*;
pub use template::*;
pub use types::*;

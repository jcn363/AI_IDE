//! Types and data structures for architectural analysis

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::analysis::{CodeLocation, Severity};

/// Type of dependency between modules or items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyType {
    /// Module import/use statement
    ModuleImport,
    /// Type reference
    TypeReference,
    /// Trait implementation
    TraitImplementation,
    /// Function call
    FunctionCall,
    /// Field access
    FieldAccess,
    /// Trait bound
    TraitBound,
    /// Type alias
    TypeAlias,
    /// Macro use
    MacroUse,
}

impl fmt::Display for DependencyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DependencyType::ModuleImport => write!(f, "module import"),
            DependencyType::TypeReference => write!(f, "type reference"),
            DependencyType::TraitImplementation => write!(f, "trait implementation"),
            DependencyType::FunctionCall => write!(f, "function call"),
            DependencyType::FieldAccess => write!(f, "field access"),
            DependencyType::TraitBound => write!(f, "trait bound"),
            DependencyType::TypeAlias => write!(f, "type alias"),
            DependencyType::MacroUse => write!(f, "macro use"),
        }
    }
}

/// Represents an architectural finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalFinding {
    /// Unique identifier for the finding
    pub id: String,
    /// Human-readable message describing the finding
    pub message: String,
    /// Severity level of the finding
    pub severity: Severity,
    /// Location in the source code where the finding was detected
    pub location: CodeLocation,
    /// Optional suggestion for fixing the issue
    pub suggestion: Option<String>,
    /// Confidence level of the finding (0.0 to 1.0)
    pub confidence: f32,
    /// Identifier for the rule that generated this finding
    pub rule_id: String,
}

/// Trait for analysis findings
pub trait Finding {
    /// Get the unique identifier for the finding
    fn id(&self) -> &str;
    /// Get the human-readable message
    fn message(&self) -> &str;
    /// Get the severity level
    fn severity(&self) -> Severity;
    /// Get the location in the source code
    fn location(&self) -> &CodeLocation;
    /// Get an optional suggestion for fixing the issue
    fn suggestion(&self) -> Option<&str>;
    /// Get the confidence level (0.0 to 1.0)
    fn confidence(&self) -> f32;
    /// Get the rule identifier
    fn rule_id(&self) -> &str;
}

impl Finding for ArchitecturalFinding {
    fn id(&self) -> &str {
        &self.id
    }

    fn message(&self) -> &str {
        &self.message
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn location(&self) -> &CodeLocation {
        &self.location
    }

    fn suggestion(&self) -> Option<&str> {
        self.suggestion.as_deref()
    }

    fn confidence(&self) -> f32 {
        self.confidence
    }

    fn rule_id(&self) -> &str {
        &self.rule_id
    }
}

/// Represents a violation of the Interface Segregation Principle
#[derive(Debug)]
pub struct InterfaceSegregationViolation {
    /// Description of the violation
    pub message: String,
    /// Location in the source code
    pub location: CodeLocation,
}

/// Represents a violation of the Dependency Inversion Principle
#[derive(Debug)]
pub struct DependencyInversionViolation {
    /// Description of the violation
    pub message: String,
    /// Location in the source code
    pub location: CodeLocation,
    /// The concrete type that should be behind an interface
    pub concrete_type: String,
}

//! Pattern detection types and definitions
//!
//! This module defines architectural patterns, anti-patterns, and pattern detection
//! results for AI-powered code analysis.

use crate::analysis::{AnalysisCategory, Severity};
use rust_ai_ide_common::{IdeResult, IdeError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core architectural pattern types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ArchitecturalPattern {
    /// Model-View-Controller pattern
    Mvc,
    /// Repository pattern
    Repository,
    /// Factory pattern
    Factory,
    /// Singleton pattern
    Singleton,
    /// Strategy pattern
    Strategy,
    /// Observer pattern
    Observer,
    /// Decorator pattern
    Decorator,
    /// Command pattern
    Command,
    /// State pattern
    State,
    /// Adapter pattern
    Adapter,
    /// Layered Architecture
    LayeredArchitecture,
    /// Dependency Injection
    DependencyInjection,
    /// Error Handling pattern
    ErrorHandling,
    /// Async/Await pattern
    AsyncAwait,
    /// Builder pattern
    Builder,
    /// Iterator pattern
    Iterator,
    /// Visitor pattern
    Visitor,
    /// Template Method
    TemplateMethod,
}

/// Anti-pattern types that indicate potential architectural issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AntiPattern {
    /// God object - single object that knows too much
    GodObject,
    /// Long method - method that does too much
    LongMethod,
    /// Large class - class with too many responsibilities
    LargeClass,
    /// Code duplication - repeated code segments
    CodeDuplication,
    /// Tight coupling - excessive dependencies
    TightCoupling,
    /// Primitive obsession - excessive use of primitive types
    PrimitiveObsession,
    /// Feature envy - method accessing data from another object
    FeatureEnvy,
    /// Data clump - group of data items that should be a single unit
    DataClump,
    /// Message chain - chain of method calls
    MessageChain,
    /// Switch statement overuse
    SwitchStatement,
    /// Circular dependency
    CircularDependency,
    /// Lack of encapsulation
    LackOfEncapsulation,
    /// Synchronous blocking in async context
    SynchronousBlocking,
    /// Excessive nesting
    ExcessiveNesting,
    /// Inconsistent error handling
    InconsistentErrorHandling,
}

/// Detected pattern result with confidence scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    pub pattern_type: ArchitecturalPattern,
    pub confidence: f32,
    pub location: CodeLocation,
    pub context: PatternContext,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Detected anti-pattern result with severity and suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedAntiPattern {
    pub anti_pattern_type: AntiPattern,
    pub severity: Severity,
    pub confidence: f32,
    pub location: CodeLocation,
    pub suggestions: Vec<String>,
    pub context: PatternContext,
    pub metrics: AntiPatternMetrics,
}

/// Context information for pattern detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternContext {
    pub code_snippet: String,
    pub surrounding_context: String,
    pub structural_info: StructuralInfo,
    pub semantic_info: SemanticInfo,
}

/// Code location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file_path: String,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub function_name: Option<String>,
    pub class_name: Option<String>,
}

/// Structural information extracted from code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralInfo {
    pub lines_of_code: usize,
    pub cyclomatic_complexity: u32,
    pub nesting_depth: u32,
    pub method_count: usize,
    pub field_count: usize,
    pub dependency_count: usize,
}

/// Semantic information from LSP analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticInfo {
    pub symbols: Vec<String>,
    pub references: Vec<String>,
    pub definitions: Vec<String>,
    pub usages: HashMap<String, usize>,
}

/// Metrics specific to anti-pattern detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPatternMetrics {
    pub violation_score: f32,
    pub maintainability_impact: f32,
    pub testability_impact: f32,
    pub performance_impact: f32,
    pub affected_lines: usize,
    pub refactoring_effort_days: f32,
}

/// Intelligence suggestion with ML-enhanced confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceSuggestion {
    pub category: SuggestionCategory,
    pub title: String,
    pub description: String,
    pub confidence: f32,
    pub priority: Priority,
    pub location: CodeLocation,
    pub refactoring_type: RefactoringType,
    pub expected_benefits: Vec<String>,
    pub implementation_guidance: String,
    pub automated_fix: Option<AutomatedFix>,
}

/// Categories of intelligence suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionCategory {
    Performance,
    Maintainability,
    Reliability,
    Security,
    Readability,
    Architecture,
}

/// Priority levels for suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Types of refactoring suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefactoringType {
    ExtractMethod,
    ExtractClass,
    InlineMethod,
    RemoveDuplication,
    SimplifyConditional,
    ImproveErrorHandling,
    BreakCircularDependency,
    IntroduceDependencyInjection,
    ApplyDesignPattern,
    ImproveAsynchronous,
    EncapsulateFields,
    ReduceNesting,
    ConsolidateConditions,
}

/// Automated fix specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomatedFix {
    pub kind: FixKind,
    pub actions: Vec<RefactoringAction>,
    pub prerequisites: Vec<String>,
}

/// Types of fixes that can be automated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FixKind {
    QuickFix,
    ContextualAction,
    Refactoring,
}

/// Specific refactoring action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringAction {
    pub action_type: ActionType,
    pub range: CodeLocation,
    pub new_text: String,
    pub old_text: Option<String>,
}

/// Types of refactoring actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    ReplaceText,
    InsertText,
    DeleteText,
    CreateFile,
    Rename,
    Move,
}

impl ArchitecturalPattern {
    /// Get a human-readable description of the pattern
    pub fn description(&self) -> &'static str {
        match self {
            ArchitecturalPattern::Mvc => "Model-View-Controller separation of concerns",
            ArchitecturalPattern::Repository => "Data access abstraction layer",
            ArchitecturalPattern::Factory => "Object creation abstraction",
            ArchitecturalPattern::Singleton => "Single instance pattern",
            ArchitecturalPattern::Strategy => "Algorithm selection at runtime",
            ArchitecturalPattern::Observer => "Subject-observer notification",
            ArchitecturalPattern::Decorator => "Dynamic behavior extension",
            ArchitecturalPattern::Command => "Encapsulated operations",
            ArchitecturalPattern::State => "State-driven behavior",
            ArchitecturalPattern::Adapter => "Interface adaptation",
            ArchitecturalPattern::LayeredArchitecture => "Hierarchical separation",
            ArchitecturalPattern::DependencyInjection => "Dependency externalization",
            ArchitecturalPattern::ErrorHandling => "Structured error management",
            ArchitecturalPattern::AsyncAwait => "Asynchronous operation handling",
            ArchitecturalPattern::Builder => "Complex object construction",
            ArchitecturalPattern::Iterator => "Sequential collection access",
            ArchitecturalPattern::Visitor => "Operation on object structures",
            ArchitecturalPattern::TemplateMethod => "Algorithm skeleton definition",
        }
    }
}

impl AntiPattern {
    /// Get a human-readable description of the anti-pattern
    pub fn description(&self) -> &'static str {
        match self {
            AntiPattern::GodObject => "Single class with too many responsibilities",
            AntiPattern::LongMethod => "Method that is too long and does too much",
            AntiPattern::LargeClass => "Class that has grown too large",
            AntiPattern::CodeDuplication => "Repeated code segments",
            AntiPattern::TightCoupling => "Excessive interdependencies",
            AntiPattern::PrimitiveObsession => "Overuse of primitive types instead of objects",
            AntiPattern::FeatureEnvy => "Method that uses too much data from another class",
            AntiPattern::DataClump => "Parameters that should be grouped together",
            AntiPattern::MessageChain => "Chain of method calls to reach a distant object",
            AntiPattern::SwitchStatement => "Overuse of switch statements",
            AntiPattern::CircularDependency => "Mutual dependencies between modules",
            AntiPattern::LackOfEncapsulation => "Exposed implementation details",
            AntiPattern::SynchronousBlocking => "Synchronous operations in async context",
            AntiPattern::ExcessiveNesting => "Deep nesting making code hard to read",
            AntiPattern::InconsistentErrorHandling => "Inconsistent or missing error handling",
        }
    }

    /// Get typical severity level for this anti-pattern
    pub fn default_severity(&self) -> Severity {
        match self {
            AntiPattern::GodObject | AntiPattern::CircularDependency => Severity::Error,
            AntiPattern::LongMethod | AntiPattern::LargeClass | AntiPattern::CodeDuplication => Severity::Warning,
            AntiPattern::TightCoupling | AntiPattern::SynchronousBlocking => Severity::Error,
            _ => Severity::Info,
        }
    }
}

impl IntelligenceSuggestion {
    /// Create a new intelligence suggestion
    pub fn new(
        category: SuggestionCategory,
        title: impl Into<String>,
        description: impl Into<String>,
        confidence: f32,
        priority: Priority,
        location: CodeLocation,
        refactoring_type: RefactoringType,
    ) -> Self {
        Self {
            category,
            title: title.into(),
            description: description.into(),
            confidence,
            priority,
            location,
            refactoring_type,
            expected_benefits: Vec::new(),
            implementation_guidance: String::new(),
            automated_fix: None,
        }
    }

    /// Add expected benefits
    pub fn with_benefits(mut self, benefits: Vec<String>) -> Self {
        self.expected_benefits = benefits;
        self
    }

    /// Add implementation guidance
    pub fn with_guidance(mut self, guidance: impl Into<String>) -> Self {
        self.implementation_guidance = guidance.into();
        self
    }

    /// Add an automated fix
    pub fn with_automated_fix(mut self, fix: AutomatedFix) -> Self {
        self.automated_fix = Some(fix);
        self
    }
}
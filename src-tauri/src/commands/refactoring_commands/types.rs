//! Type mappings and DTOs for refactoring commands
//!
//! This module defines all the request/response types used by the
//! refactoring command handlers, ensuring proper serialization
//! between Rust and the frontend.

use rust_ai_ide_ai_refactoring::types::*;
use serde::{Deserialize, Serialize};

/// Request to execute a refactoring operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringExecutionRequest {
    pub file_path: String,
    pub operation_type: RefactoringType,
    pub context: RefactoringContextDTO,
    pub options: RefactoringOptionsDTO,
}

/// Request to analyze refactoring candidates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringAnalysisRequest {
    pub file_path: String,
    pub operation_types: Vec<RefactoringType>,
    pub target_symbol: Option<String>,
    pub symbol_kind: Option<SymbolKindDTO>,
    pub cursor_position: Option<PositionDTO>,
    pub selection: Option<SelectionDTO>,
    pub project_root: String,
}

/// Request for batch refactoring operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRefactoringRequest {
    pub operations: Vec<BatchRefactoringOperationDTO>,
    pub parallel_execution: bool,
    pub stop_on_failure: bool,
    pub max_concurrent_operations: usize,
}

/// Request for generating tests after refactoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestGenerationRequest {
    pub file_path: String,
    pub operation_type: RefactoringType,
    pub original_content: String,
    pub symbol_name: Option<String>,
    pub symbol_kind: Option<SymbolKind>,
    pub cursor_line: usize,
    pub cursor_character: usize,
    pub selection: Option<CodeRange>,
    pub project_root: String,
}

/// Request for undo operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoRequest {
    pub operation_id: String,
}

/// Request for safety validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyValidationRequest {
    pub file_path: String,
    pub operation_type: RefactoringType,
    pub context: RefactoringContextDTO,
}

/// Response for refactoring analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringAnalysisResponse {
    pub file_path: String,
    pub candidates: Vec<RefactoringCandidate>,
    pub analysis_summary: Vec<String>,
    pub total_candidates: usize,
}

/// Response for batch refactoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRefactoringResult {
    pub operation_count: usize,
    pub successes: usize,
    pub failures: usize,
    pub warning_count: usize,
    pub execution_time_ms: u64,
    pub results: Vec<RefactoringResult>,
    pub progress_summary: String,
}

/// Response for test generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestGenerationResponse {
    pub operation_type: RefactoringType,
    pub generated_tests: Vec<GeneratedTestInfo>,
    pub test_count: usize,
    pub coverage_estimate: f64,
}

/// Response for undo operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoResult {
    pub success: bool,
    pub operation_id: String,
    pub reverted_changes: Vec<CodeChange>,
    pub warnings: Vec<String>,
}

/// Response for safety validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyValidationResult {
    pub operation_type: RefactoringType,
    pub is_safe: bool,
    pub confidence_score: f64,
    pub potential_impact: RefactoringImpactDTO,
    pub breaking_changes: Vec<String>,
    pub suggested_alternatives: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Refactoring operation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringOperationInfo {
    pub operation_type: RefactoringType,
    pub name: String,
    pub description: String,
    pub requires_selection: bool,
    pub is_experimental: bool,
    pub typical_confidence_score: f64,
}

/// Generated test information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTestInfo {
    pub test_type: String,
    pub language: String,
    pub framework: String,
    pub content: String,
    pub filename: String,
    pub dependencies: Vec<String>,
}

/// Refactoring candidate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringCandidate {
    pub operation_type: RefactoringType,
    pub confidence_score: f64,
    pub suitability_reasons: Vec<String>,
    pub potential_impact: RefactoringImpactDTO,
    pub breaking_changes: Vec<String>,
    pub affected_files: Vec<String>,
}

/// Batch refactoring operation DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRefactoringOperationDTO {
    pub operation_type: RefactoringType,
    pub context: RefactoringContextDTO,
    pub options: RefactoringOptionsDTO,
}

/// Position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionDTO {
    pub line: u32,
    pub character: u32,
}

/// Selection range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionDTO {
    pub start: PositionDTO,
    pub end: PositionDTO,
}

/// DTO versions of core types for better serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringContextDTO {
    pub file_path: String,
    pub symbol_name: Option<String>,
    pub symbol_kind: Option<SymbolKindDTO>,
    pub cursor_line: usize,
    pub cursor_character: usize,
    pub selection: Option<CodeRangeDTO>,
    pub project_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringOptionsDTO {
    pub create_backup: bool,
    pub generate_tests: bool,
    pub dry_run: bool,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringPreferences {
    pub enable_auto_preview: bool,
    pub enable_safety_validation: bool,
    pub default_confidence_threshold: f64,
    pub auto_generate_tests: bool,
    pub enable_backup: bool,
    pub max_concurrent_operations: usize,
    pub experimental_features_enabled: bool,
    pub lsp_integration_level: String,
}

/// Symbol kind enumeration for DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolKindDTO {
    Function,
    Method,
    Struct,
    Class,
    Enum,
    Variable,
    Constant,
    Module,
    Interface,
    Type,
    Field,
    Property,
    Unknown,
}

/// Refactoring impact enumeration for DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefactoringImpactDTO {
    Low,
    Medium,
    High,
}

/// Code range for DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRangeDTO {
    pub start_line: usize,
    pub start_character: usize,
    pub end_line: usize,
    pub end_character: usize,
}

// Conversion implementations

impl TryFrom<RefactoringContextDTO> for RefactoringContext {
    type Error = String;

    fn try_from(dto: RefactoringContextDTO) -> Result<Self, Self::Error> {
        Ok(RefactoringContext {
            file_path: dto.file_path,
            symbol_name: dto.symbol_name,
            symbol_kind: dto.symbol_kind.map(|k| k.into()),
            cursor_line: dto.cursor_line,
            cursor_character: dto.cursor_character,
            selection: dto.selection.map(|s| s.into()),
            context_lines: vec![],               // Not provided in DTO
            language: ProgrammingLanguage::Rust, // Default for now
            project_root: dto.project_root,
        })
    }
}

impl TryFrom<RefactoringOptionsDTO> for RefactoringOptions {
    type Error = String;

    fn try_from(dto: RefactoringOptionsDTO) -> Result<Self, Self::Error> {
        Ok(RefactoringOptions {
            dry_run: dto.dry_run,
            preview_changes: false, // DTO doesn't provide this
            backup_original: dto.create_backup,
            generate_tests: dto.generate_tests,
            preserve_references: true,       // Default
            apply_to_all_occurrences: false, // Default
            extra_options: None,             // Not provided in DTO
            timeout_seconds: dto.timeout_seconds.unwrap_or(30),
            max_memory_mb: 512,        // Default
            allow_partial: false,      // Default
            validate_after: true,      // Default
            rollback_on_failure: true, // Default
        })
    }
}

impl From<SymbolKindDTO> for SymbolKind {
    fn from(dto: SymbolKindDTO) -> Self {
        match dto {
            SymbolKindDTO::Function => SymbolKind::Function,
            SymbolKindDTO::Method => SymbolKind::Method,
            SymbolKindDTO::Struct => SymbolKind::Struct,
            SymbolKindDTO::Class => SymbolKind::Class,
            SymbolKindDTO::Enum => SymbolKind::Enum,
            SymbolKindDTO::Variable => SymbolKind::Variable,
            SymbolKindDTO::Constant => SymbolKind::Constant,
            SymbolKindDTO::Module => SymbolKind::Module,
            SymbolKindDTO::Interface => SymbolKind::Interface,
            SymbolKindDTO::Type => SymbolKind::Type,
            SymbolKindDTO::Field => SymbolKind::Field,
            SymbolKindDTO::Property => SymbolKind::Property,
            SymbolKindDTO::Unknown => SymbolKind::Unknown,
        }
    }
}

impl From<CodeRangeDTO> for CodeRange {
    fn from(dto: CodeRangeDTO) -> Self {
        CodeRange {
            start_line: dto.start_line,
            start_character: dto.start_character,
            end_line: dto.end_line,
            end_character: dto.end_character,
        }
    }
}

impl From<SelectionDTO> for CodeRange {
    fn from(dto: SelectionDTO) -> Self {
        CodeRange {
            start_line: dto.start.line as usize,
            start_character: dto.start.character as usize,
            end_line: dto.end.line as usize,
            end_character: dto.end.character as usize,
        }
    }
}

impl From<RefactoringImpactDTO> for RefactoringImpact {
    fn from(dto: RefactoringImpactDTO) -> Self {
        match dto {
            RefactoringImpactDTO::Low => RefactoringImpact::Low,
            RefactoringImpactDTO::Medium => RefactoringImpact::Medium,
            RefactoringImpactDTO::High => RefactoringImpact::High,
        }
    }
}

impl From<RefactoringImpact> for RefactoringImpactDTO {
    fn from(impact: RefactoringImpact) -> Self {
        match impact {
            RefactoringImpact::Low => RefactoringImpactDTO::Low,
            RefactoringImpact::Medium => RefactoringImpactDTO::Medium,
            RefactoringImpact::High => RefactoringImpactDTO::High,
        }
    }
}

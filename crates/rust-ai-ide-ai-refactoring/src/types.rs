use serde::{Deserialize, Serialize};

/// Refactoring types that define the kind of refactoring operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum RefactoringType {
    Rename,
    ExtractFunction,
    ExtractVariable,
    ExtractClass,
    ExtractInterface,
    InlineFunction,
    InlineVariable,
    InlineMethod,
    MoveMethod,
    MoveClass,
    MoveFile,
    RemoveParameter,
    IntroduceParameter,
    ReplaceConstructor,
    ReplaceConditionals,
    ConvertMethodToFunction,
    SplitClass,
    MergeClasses,
    ChangeSignature,
    AddDelegation,
    RemoveDelegation,
    EncapsulateField,
    LocalizeVariable,
    AddMissingImports,
    SortImports,
    ConvertToAsync,
    GenerateGettersSetters,
    PatternConversion,
    BatchInterfaceExtraction,
    BatchPatternConversion,
}

impl std::fmt::Display for RefactoringType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefactoringType::Rename => write!(f, "Rename"),
            RefactoringType::ExtractFunction => write!(f, "Extract Function"),
            RefactoringType::ExtractVariable => write!(f, "Extract Variable"),
            RefactoringType::ExtractClass => write!(f, "Extract Class"),
            RefactoringType::ExtractInterface => write!(f, "Extract Interface"),
            RefactoringType::InlineFunction => write!(f, "Inline Function"),
            RefactoringType::InlineVariable => write!(f, "Inline Variable"),
            RefactoringType::InlineMethod => write!(f, "Inline Method"),
            RefactoringType::MoveMethod => write!(f, "Move Method"),
            RefactoringType::MoveClass => write!(f, "Move Class"),
            RefactoringType::MoveFile => write!(f, "Move File"),
            RefactoringType::RemoveParameter => write!(f, "Remove Parameter"),
            RefactoringType::IntroduceParameter => write!(f, "Introduce Parameter"),
            RefactoringType::ReplaceConstructor => write!(f, "Replace Constructor"),
            RefactoringType::ReplaceConditionals => write!(f, "Replace Conditionals"),
            RefactoringType::ConvertMethodToFunction => write!(f, "Convert Method to Function"),
            RefactoringType::SplitClass => write!(f, "Split Class"),
            RefactoringType::MergeClasses => write!(f, "Merge Classes"),
            RefactoringType::ChangeSignature => write!(f, "Change Signature"),
            RefactoringType::AddDelegation => write!(f, "Add Delegation"),
            RefactoringType::RemoveDelegation => write!(f, "Remove Delegation"),
            RefactoringType::EncapsulateField => write!(f, "Encapsulate Field"),
            RefactoringType::LocalizeVariable => write!(f, "Localize Variable"),
            RefactoringType::AddMissingImports => write!(f, "Add Missing Imports"),
            RefactoringType::SortImports => write!(f, "Sort Imports"),
            RefactoringType::ConvertToAsync => write!(f, "Convert to Async"),
            RefactoringType::GenerateGettersSetters => write!(f, "Generate Getters/Setters"),
            RefactoringType::PatternConversion => write!(f, "Pattern Conversion"),
            RefactoringType::BatchInterfaceExtraction => write!(f, "Batch Interface Extraction"),
            RefactoringType::BatchPatternConversion => write!(f, "Batch Pattern Conversion"),
        }
    }
}

impl RefactoringType {
    /// Try to convert from string to RefactoringType
    pub fn try_from(s: &str) -> Result<Self, String> {
        match s {
            "rename" => Ok(RefactoringType::Rename),
            "extractFunction" => Ok(RefactoringType::ExtractFunction),
            "extractVariable" => Ok(RefactoringType::ExtractVariable),
            "extractClass" => Ok(RefactoringType::ExtractClass),
            "extractInterface" => Ok(RefactoringType::ExtractInterface),
            "inlineFunction" => Ok(RefactoringType::InlineFunction),
            "inlineVariable" => Ok(RefactoringType::InlineVariable),
            "inlineMethod" => Ok(RefactoringType::InlineMethod),
            "moveMethod" => Ok(RefactoringType::MoveMethod),
            "moveClass" => Ok(RefactoringType::MoveClass),
            "moveFile" => Ok(RefactoringType::MoveFile),
            "removeParameter" => Ok(RefactoringType::RemoveParameter),
            "introduceParameter" => Ok(RefactoringType::IntroduceParameter),
            "replaceConstructor" => Ok(RefactoringType::ReplaceConstructor),
            "replaceConditionals" => Ok(RefactoringType::ReplaceConditionals),
            "convertMethodToFunction" => Ok(RefactoringType::ConvertMethodToFunction),
            "splitClass" => Ok(RefactoringType::SplitClass),
            "mergeClasses" => Ok(RefactoringType::MergeClasses),
            "changeSignature" => Ok(RefactoringType::ChangeSignature),
            "addDelegation" => Ok(RefactoringType::AddDelegation),
            "removeDelegation" => Ok(RefactoringType::RemoveDelegation),
            "encapsulateField" => Ok(RefactoringType::EncapsulateField),
            "localizeVariable" => Ok(RefactoringType::LocalizeVariable),
            "addMissingImports" => Ok(RefactoringType::AddMissingImports),
            "sortImports" => Ok(RefactoringType::SortImports),
            "convertToAsync" => Ok(RefactoringType::ConvertToAsync),
            "generateGettersSetters" => Ok(RefactoringType::GenerateGettersSetters),
            "patternConversion" => Ok(RefactoringType::PatternConversion),
            "batchInterfaceExtraction" => Ok(RefactoringType::BatchInterfaceExtraction),
            "batchPatternConversion" => Ok(RefactoringType::BatchPatternConversion),
            _ => Err(format!("Unknown refactoring type: {}", s)),
        }
    }
}

/// Context information for a refactoring operation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RefactoringContext {
    pub file_path:        String,
    pub cursor_line:      usize,
    pub cursor_character: usize,
    pub selection:        Option<CodeRange>,
    pub symbol_name:      Option<String>,
    pub symbol_kind:      Option<SymbolKind>,
}

/// Range in the code for selections or changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRange {
    pub start_line:      usize,
    pub start_character: usize,
    pub end_line:        usize,
    pub end_character:   usize,
}

/// Different types of symbols in code
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Variable,
    Class,
    Interface,
    Module,
    Method,
    Struct,
    Enum,
}

/// Options that control how a refactoring is performed
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RefactoringOptions {
    pub create_backup:            bool,
    pub generate_tests:           bool,
    pub apply_to_all_occurrences: bool,
    pub preserve_references:      bool,
    pub ignore_safe_operations:   bool,
    pub extra_options:            Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// Result of a refactoring operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringResult {
    pub id:            Option<String>,
    pub success:       bool,
    pub changes:       Vec<CodeChange>,
    pub error_message: Option<String>,
    pub warnings:      Vec<String>,
    pub new_content:   Option<String>,
}

/// A single change to apply to code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub file_path:   String,
    pub range:       CodeRange,
    pub old_text:    String,
    pub new_text:    String,
    pub change_type: ChangeType,
}

/// Types of changes that can be made
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChangeType {
    Insertion,
    Replacement,
    Deletion,
}

/// Analysis result that evaluates if a refactoring can be safely applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringAnalysis {
    pub is_safe:          bool,
    pub confidence_score: f64,
    pub potential_impact: RefactoringImpact,
    pub affected_files:   Vec<String>,
    pub affected_symbols: Vec<String>,
    pub breaking_changes: Vec<String>,
    pub suggestions:      Vec<String>,
    pub warnings:         Vec<String>,
}

/// Impact level of a refactoring operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RefactoringImpact {
    Low,
    Medium,
    High,
    Critical,
}

/// Impact level for refactoring operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
}

/// Target information for a refactoring operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringTarget {
    pub target_type: SymbolKind,
    pub name:        String,
    pub range:       CodeRange,
    pub context:     RefactoringContext,
}

/// Batch refactoring operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRefactoring {
    pub operations:             Vec<BatchOperation>,
    pub validate_independently: bool,
    pub stop_on_first_error:    bool,
    pub backup_strategy:        BackupStrategy,
}

/// Single operation within a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperation {
    pub refactoring_type: RefactoringType,
    pub context:          RefactoringContext,
    pub options:          RefactoringOptions,
    pub dependencies:     Vec<String>,
}

/// Backup strategies for batch operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BackupStrategy {
    NoBackup,
    SingleBackup,
    PerOperationBackup,
    GitBackup,
}

/// Configuration for refactoring operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringConfiguration {
    pub enabled_refactorings: Vec<RefactoringType>,
    pub default_options:      RefactoringOptions,
    pub analysis_depth:       AnalysisDepth,
    pub max_search_files:     usize,
    pub timeout_seconds:      u32,
}

/// Depth of analysis to perform
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AnalysisDepth {
    Basic,
    Standard,
    Comprehensive,
}

/// LSP response for refactoring operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LSPRefactoringResponse {
    pub success: bool,
    pub changes: Vec<LSPTextEdit>,
    pub error:   Option<String>,
}

/// LSP text edit structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LSPTextEdit {
    pub range:    LSPRange,
    pub new_text: String,
    pub old_text: Option<String>,
}

/// LSP range structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LSPRange {
    pub start: LSPPosition,
    pub end:   LSPPosition,
}

/// LSP position structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LSPPosition {
    pub line:      usize,
    pub character: usize,
}

/// Refactoring request sent to LSP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LSPRefactoringRequest {
    pub refactoring_type: RefactoringType,
    pub context:          RefactoringContext,
    pub options:          RefactoringOptions,
}

/// Type alias for LSP refactoring request (backward compatibility)
pub type RefactoringRequest = LSPRefactoringRequest;

/// Type alias for refactoring result
pub type RefactoringOperationResult = RefactoringResult;

/// Suggestion type for refactoring suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringSuggestion {
    pub refactoring_type: RefactoringType,
    pub confidence:       f64,
    pub description:      String,
    pub context:          RefactoringContext,
    pub expected_changes: Vec<CodeChange>,
}

// ToTokens implementations for syn quote! macro support
use quote::{quote, ToTokens};

/// Helper struct for extracted method information
pub struct ExtractedMethod {
    pub signature: syn::Signature,
    pub attrs:     Vec<syn::Attribute>,
}

impl ToTokens for ExtractedMethod {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for attr in &self.attrs {
            attr.to_tokens(tokens);
        }
        self.signature.to_tokens(tokens);
    }
}

/// Analyzed field information for splitting
#[derive(Clone)]
pub struct FieldInfo {
    pub name:          String,
    pub ty:            syn::Type,
    pub visibility:    syn::Visibility,
    pub methods_using: Vec<String>,
}

impl ToTokens for FieldInfo {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // For FieldInfo, we'll just quote the type since it's primarily used for code generation
        self.ty.to_tokens(tokens);
    }
}

/// Method information for splitting
#[derive(Clone)]
pub struct MethodInfo {
    pub name:        String,
    pub signature:   syn::Signature,
    pub fields_used: Vec<String>,
    pub visibility:  syn::Visibility,
}

impl ToTokens for MethodInfo {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.signature.to_tokens(tokens);
    }
}

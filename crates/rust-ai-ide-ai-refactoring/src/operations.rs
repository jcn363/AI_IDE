use crate::types::*;
use async_trait::async_trait;
use std::fs;

// AST manipulation imports
use prettyplease;
use syn::{visit_mut::VisitMut, Ident};

/// Check if a file is supported by AST-based operations (Rust files)
pub fn is_ast_supported(file_path: &str) -> bool {
    use std::path::Path;
    rust_ai_ide_shared_utils::get_extension(Path::new(file_path))
        .map(|ext| ext == "rs")
        .unwrap_or(false)
}

// Re-export main types and operations for public API
pub use crate::async_operations::ConvertToAsyncOperation;
pub use crate::batch_operations::BatchInterfaceExtractionOperation;
pub use crate::class_struct_operations::EncapsulateFieldOperation;
pub use crate::class_struct_operations::ExtractClassOperation;
pub use crate::class_struct_operations::ExtractInterfaceOperation;
pub use crate::class_struct_operations::GenerateGettersSettersOperation;
pub use crate::class_struct_operations::MergeClassesOperation;
pub use crate::class_struct_operations::SplitClassOperation;
pub use crate::code_organization::AddMissingImportsOperation;
pub use crate::code_organization::SortImportsOperation;
pub use crate::core_traits::RefactoringOperation;
pub use crate::delegation_operations::AddDelegationOperation;
pub use crate::delegation_operations::RemoveDelegationOperation;
pub use crate::file_operations::MoveClassOperation;
pub use crate::file_operations::MoveFileOperation;
pub use crate::function_method_operations::ConvertMethodToFunctionOperation;
pub use crate::function_method_operations::ExtractFunctionOperation;
pub use crate::function_method_operations::InlineFunctionOperation;
pub use crate::function_method_operations::InlineMethodOperation;
pub use crate::function_method_operations::MoveMethodOperation;
pub use crate::operation_factory::RefactoringOperationFactory;
pub use crate::pattern_recognition::BatchPatternConversionOperation;
pub use crate::pattern_recognition::PatternConversionOperation;
pub use crate::pattern_recognition::ReplaceConditionalsOperation;
pub use crate::rename_operations::RenameOperation;
pub use crate::signature_operations::ChangeSignatureOperation;
pub use crate::signature_operations::IntroduceParameterOperation;
pub use crate::signature_operations::RemoveParameterOperation;
pub use crate::signature_operations::ReplaceConstructorOperation;
pub use crate::variable_operations::ExtractVariableOperation;
pub use crate::variable_operations::InlineVariableOperation;
pub use crate::variable_operations::LocalizeVariableOperation;

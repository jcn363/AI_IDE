use std::fs;

use async_trait::async_trait;
// AST manipulation imports
use prettyplease;
use syn::visit_mut::VisitMut;
use syn::Ident;

use crate::types::*;

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
pub use crate::class_struct_operations::{
    EncapsulateFieldOperation, ExtractClassOperation, ExtractInterfaceOperation, GenerateGettersSettersOperation,
    MergeClassesOperation, SplitClassOperation,
};
pub use crate::code_organization::{AddMissingImportsOperation, SortImportsOperation};
pub use crate::core_traits::RefactoringOperation;
pub use crate::delegation_operations::{AddDelegationOperation, RemoveDelegationOperation};
pub use crate::file_operations::{MoveClassOperation, MoveFileOperation};
pub use crate::function_method_operations::{
    ConvertMethodToFunctionOperation, ExtractFunctionOperation, InlineFunctionOperation, InlineMethodOperation,
    MoveMethodOperation,
};
pub use crate::operation_factory::RefactoringOperationFactory;
pub use crate::pattern_recognition::{
    BatchPatternConversionOperation, PatternConversionOperation, ReplaceConditionalsOperation,
};
pub use crate::rename_operations::RenameOperation;
pub use crate::signature_operations::{
    ChangeSignatureOperation, IntroduceParameterOperation, RemoveParameterOperation, ReplaceConstructorOperation,
};
pub use crate::variable_operations::{ExtractVariableOperation, InlineVariableOperation, LocalizeVariableOperation};

use crate::async_operations::*;
use crate::batch_operations::*;
use crate::class_struct_operations::*;
use crate::code_organization::*;
use crate::delegation_operations::*;
use crate::file_operations::*;
use crate::function_method_operations::*;
use crate::pattern_recognition::*;
use crate::rename_operations::*;
use crate::signature_operations::*;
use crate::types::*;
use crate::variable_operations::*;
use crate::RefactoringOperation;

/// Factory for creating refactoring operations
pub struct RefactoringOperationFactory;

impl RefactoringOperationFactory {
    /// Create an operation instance for the given refactoring type
    pub fn create_operation(
        refactoring_type: &RefactoringType,
    ) -> Result<Box<dyn RefactoringOperation>, Box<dyn std::error::Error + Send + Sync>> {
        match refactoring_type {
            RefactoringType::Rename => Ok(Box::new(RenameOperation)),
            RefactoringType::ExtractFunction => Ok(Box::new(ExtractFunctionOperation)),
            RefactoringType::ExtractVariable => Ok(Box::new(ExtractVariableOperation)),
            RefactoringType::InlineVariable => Ok(Box::new(InlineVariableOperation)),
            RefactoringType::InlineFunction => Ok(Box::new(InlineFunctionOperation)),
            RefactoringType::InlineMethod => Ok(Box::new(InlineMethodOperation)),
            RefactoringType::ExtractInterface => Ok(Box::new(ExtractInterfaceOperation)),
            RefactoringType::ConvertToAsync => Ok(Box::new(ConvertToAsyncOperation)),
            RefactoringType::PatternConversion => Ok(Box::new(PatternConversionOperation)),
            RefactoringType::MoveMethod => Ok(Box::new(MoveMethodOperation)),
            RefactoringType::MoveClass => Ok(Box::new(MoveClassOperation)),
            RefactoringType::MoveFile => Ok(Box::new(MoveFileOperation)),
            RefactoringType::RemoveParameter => Ok(Box::new(RemoveParameterOperation)),
            RefactoringType::IntroduceParameter => Ok(Box::new(IntroduceParameterOperation)),
            RefactoringType::ReplaceConstructor => Ok(Box::new(ReplaceConstructorOperation)),
            RefactoringType::ReplaceConditionals => Ok(Box::new(ReplaceConditionalsOperation)),
            RefactoringType::ConvertMethodToFunction => {
                Ok(Box::new(ConvertMethodToFunctionOperation))
            }
            RefactoringType::SplitClass => Ok(Box::new(SplitClassOperation)),
            RefactoringType::MergeClasses => Ok(Box::new(MergeClassesOperation)),
            RefactoringType::ChangeSignature => Ok(Box::new(ChangeSignatureOperation)),
            RefactoringType::AddDelegation => Ok(Box::new(AddDelegationOperation)),
            RefactoringType::RemoveDelegation => Ok(Box::new(RemoveDelegationOperation)),
            RefactoringType::EncapsulateField => Ok(Box::new(EncapsulateFieldOperation)),
            RefactoringType::LocalizeVariable => Ok(Box::new(LocalizeVariableOperation)),
            RefactoringType::AddMissingImports => Ok(Box::new(AddMissingImportsOperation)),
            RefactoringType::SortImports => Ok(Box::new(SortImportsOperation)),
            RefactoringType::GenerateGettersSetters => {
                Ok(Box::new(GenerateGettersSettersOperation))
            }
            RefactoringType::ExtractClass => Ok(Box::new(ExtractClassOperation {})),
            RefactoringType::BatchInterfaceExtraction => {
                Ok(Box::new(BatchInterfaceExtractionOperation {}))
            }
            RefactoringType::BatchPatternConversion => {
                Ok(Box::new(BatchPatternConversionOperation {}))
            } // InterfaceExtraction removed - use ExtractInterface instead
              // AsyncAwaitPatternConversion removed - use ConvertToAsync instead
        }
    }

    /// Get all available refactoring types
    pub fn available_refactorings() -> Vec<RefactoringType> {
        vec![
            RefactoringType::Rename,
            RefactoringType::ExtractFunction,
            RefactoringType::ExtractVariable,
            RefactoringType::InlineVariable,
            RefactoringType::InlineFunction,
            RefactoringType::InlineMethod,
            RefactoringType::ExtractInterface,
            RefactoringType::ConvertToAsync,
            RefactoringType::PatternConversion,
            RefactoringType::MoveMethod,
            RefactoringType::MoveClass,
            RefactoringType::MoveFile,
            RefactoringType::RemoveParameter,
            RefactoringType::IntroduceParameter,
            RefactoringType::ReplaceConstructor,
            RefactoringType::ReplaceConditionals,
            RefactoringType::ConvertMethodToFunction,
            RefactoringType::SplitClass,
            RefactoringType::MergeClasses,
            RefactoringType::ChangeSignature,
            RefactoringType::AddDelegation,
            RefactoringType::RemoveDelegation,
            RefactoringType::EncapsulateField,
            RefactoringType::LocalizeVariable,
            RefactoringType::AddMissingImports,
            RefactoringType::SortImports,
            RefactoringType::GenerateGettersSetters,
            // Removed InterfaceExtraction - use ExtractInterface instead
            // Removed AsyncAwaitPatternConversion - use ConvertToAsync instead
        ]
    }
}

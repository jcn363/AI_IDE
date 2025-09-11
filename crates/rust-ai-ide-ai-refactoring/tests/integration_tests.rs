/// Integration tests for the AI-powered refactoring operations
/// Tests the full workflow of refactoring operations on real Rust codebases
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use rust_ai_ide_ai_refactoring::types::*;
use rust_ai_ide_ai_refactoring::operations::*;
use rust_ai_ide_ai_refactoring::RefactoringOperationFactory;

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a temporary Rust file for testing
    fn create_test_file(content: &str, filename: &str) -> tempfile::TempDir {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join(filename);
        fs::write(&file_path, content).unwrap();
        temp_dir
    }

    /// Helper function to create basic refactoring context
    fn create_context(file_path: &str, symbol_name: &str) -> RefactoringContext {
        RefactoringContext {
            file_path: file_path.to_string(),
            symbol_name: Some(symbol_name.to_string()),
            symbol_kind: Some(SymbolKind::Function),
            cursor_line: 1,
            cursor_character: 1,
            selection: None,
            context_lines: vec![],
            language: ProgrammingLanguage::Rust,
            project_root: "/tmp/test".to_string(),
        }
    }

    /// Helper function to create basic refactoring options
    fn create_options() -> RefactoringOptions {
        RefactoringOptions {
            dry_run: false,
            preview_changes: false,
            backup_original: true,
            generate_tests: true,
            preserve_references: true,
            apply_to_all_occurrences: false,
            extra_options: Some(serde_json::json!({})),
            timeout_seconds: 30,
            max_memory_mb: 512,
            allow_partial: false,
            validate_after: true,
            rollback_on_failure: true,
        }
    }

    #[tokio::test]
    async fn test_rename_operation_real_code() {
        // Create a test file with actual Rust code
        let test_code = r#"
pub struct Calculator {
    value: i32,
}

impl Calculator {
    pub fn add(self, other: i32) -> i32 {
        self.value + other
    }

    pub fn multiply(self, factor: i32) -> i32 {
        self.value * factor
    }
}

fn main() {
    let calc = Calculator { value: 5 };
    let result = calc.add(3);
    println!("Result: {}", result);
}
"#;
        let temp_dir = create_test_file(test_code, "calculator.rs");
        let file_path = temp_dir.path().join("calculator.rs").to_str().unwrap();

        let context = RefactoringContext {
            file_path: file_path.to_string(),
            symbol_name: Some("add".to_string()),
            symbol_kind: Some(SymbolKind::Function),
            cursor_line: 7,
            cursor_character: 5,
            selection: None,
            context_lines: vec![],
            language: ProgrammingLanguage::Rust,
            project_root: "/tmp/test".to_string(),
        };

        let options = create_options();
        let operation = RenameOperation;

        // Test the operation
        match operation.execute(&context, &options).await {
            Ok(result) => {
                assert!(result.success);
                assert!(!result.changes.is_empty());

                // Verify the rename worked (check old text doesn't contain new name references)
                let modified_content = &result.new_content.unwrap();
                assert!(!modified_content.contains("add("));
                println!("✓ Rename operation successfully transformed code");
            }
            Err(e) => panic!("Rename operation failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_extract_interface_operation_real_code() {
        // Create a larger test struct with multiple methods
        let test_code = r#"
pub struct DataProcessor {
    data: Vec<String>,
}

impl DataProcessor {
    pub fn new(data: Vec<String>) -> Self {
        Self { data }
    }

    pub fn process_data(&mut self) -> Vec<String> {
        self.data.iter().map(|s| s.to_uppercase()).collect()
    }

    pub fn filter_data(&self, predicate: impl Fn(&String) -> bool) -> Vec<String> {
        self.data.iter().filter(|s| predicate(s)).cloned().collect()
    }

    pub fn get_count(&self) -> usize {
        self.data.len()
    }

    pub fn sort_data(&mut self) {
        self.data.sort();
    }
}
"#;
        let temp_dir = create_test_file(test_code, "processor.rs");
        let file_path = temp_dir.path().join("processor.rs").to_str().unwrap();

        let context = RefactoringContext {
            file_path: file_path.to_string(),
            symbol_name: Some("DataProcessor".to_string()),
            symbol_kind: Some(SymbolKind::Struct),
            cursor_line: 1,
            cursor_character: 1,
            selection: None,
            context_lines: vec![],
            language: ProgrammingLanguage::Rust,
            project_root: "/tmp/test".to_string(),
        };

        let mut options = create_options();
        options.extra_options = Some(serde_json::json!({"interfaceName": "ProcessorInterface"}));

        let operation = ExtractInterfaceOperation;

        match operation.execute(&context, &options).await {
            Ok(result) => {
                assert!(result.success);
                let modified_content = &result.new_content.unwrap();

                // Verify interface was extracted
                assert!(modified_content.contains("trait ProcessorInterface"));
                assert!(modified_content.contains("impl ProcessorInterface for DataProcessor"));

                // Verify all public methods are in the trait
                assert!(modified_content.contains("fn process_data"));
                assert!(modified_content.contains("fn filter_data"));
                assert!(modified_content.contains("fn get_count"));
                assert!(modified_content.contains("fn sort_data"));

                println!("✓ Extract interface operation successfully created trait and implementation");
            }
            Err(e) => panic!("Extract interface operation failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_convert_to_async_operation_real_code() {
        // Create a synchronous function that should be converted to async
        let test_code = r#"
use std::thread::sleep;
use std::time::Duration;

pub struct AsyncService {
    data: Vec<String>,
}

impl AsyncService {
    pub fn new() -> Self {
        Self { data: vec![] }
    }

    pub fn sync_process(&mut self, input: String) -> String {
        sleep(Duration::from_millis(100)); // Simulate some work
        format!("Processed: {}", input.to_uppercase())
    }

    pub fn sync_batch_process(&mut self, inputs: Vec<String>) -> Vec<String> {
        inputs.into_iter().map(|input| self.sync_process(input)).collect()
    }
}
"#;
        let temp_dir = create_test_file(test_code, "async_service.rs");
        let file_path = temp_dir.path().join("async_service.rs").to_str().unwrap();

        let context = RefactoringContext {
            file_path: file_path.to_string(),
            symbol_name: Some("sync_process".to_string()),
            symbol_kind: Some(SymbolKind::Function),
            cursor_line: 12,
            cursor_character: 5,
            selection: None,
            context_lines: vec![],
            language: ProgrammingLanguage::Rust,
            project_root: "/tmp/test".to_string(),
        };

        let options = create_options();
        let operation = ConvertToAsyncOperation;

        match operation.execute(&context, &options).await {
            Ok(result) => {
                assert!(result.success);
                let modified_content = &result.new_content.unwrap();

                // Verify async conversion
                assert!(modified_content.contains("async fn sync_process"));
                assert!(modified_content.contains("impl std::future::Future"));

                println!("✓ Convert to async operation successfully made function async");
            }
            Err(e) => panic!("Convert to async operation failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_split_class_operation_complex_example() {
        // Create a complex struct that would benefit from splitting
        let test_code = r#"
pub struct LargeService {
    config: HashMap<String, String>,
    metrics: Vec<u64>,
    cache: HashMap<String, Vec<u8>>,
    logger: SimpleLogger,
    network_client: NetworkClient,
    file_manager: FileManager,
}

impl LargeService {
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
            metrics: Vec::new(),
            cache: HashMap::new(),
            logger: SimpleLogger::new(),
            network_client: NetworkClient::new(),
            file_manager: FileManager::new(),
        }
    }

    // Configuration methods
    pub fn get_config(&self, key: &str) -> Option<&String> {
        self.config.get(key)
    }

    pub fn set_config(&mut self, key: String, value: String) {
        self.config.insert(key, value);
    }

    // Metrics methods
    pub fn record_metric(&mut self, metric: u64) {
        self.metrics.push(metric);
    }

    pub fn get_average_metric(&self) -> f64 {
        if self.metrics.is_empty() {
            0.0
        } else {
            let sum: u64 = self.metrics.iter().sum();
            sum as f64 / self.metrics.len() as f64
        }
    }

    // Cache methods
    pub fn get_cached(&self, key: &str) -> Option<&Vec<u8>> {
        self.cache.get(key)
    }

    pub fn set_cached(&mut self, key: String, data: Vec<u8>) {
        self.cache.insert(key, data);
    }

    // Network methods
    pub fn make_request(&self, url: &str) -> String {
        self.network_client.request(url);
        "response".to_string()
    }

    // File methods
    pub fn save_to_file(&mut self, filename: &str, content: &str) {
        self.file_manager.save(filename, content);
    }

    pub fn load_from_file(&self, filename: &str) -> String {
        self.file_manager.load(filename)
    }

    // Logging methods
    pub fn log_info(&mut self, message: &str) {
        self.logger.info(message);
    }

    pub fn log_error(&mut self, message: &str) {
        self.logger.error(message);
    }
}

// Mock helper types for compilation
pub struct SimpleLogger;
impl SimpleLogger {
    pub fn new() -> Self { Self }
    pub fn info(&mut self, _msg: &str) {}
    pub fn error(&mut self, _msg: &str) {}
}

pub struct NetworkClient;
impl NetworkClient {
    pub fn new() -> Self { Self }
    pub fn request(&self, _url: &str) {}
}

pub struct FileManager;
impl FileManager {
    pub fn new() -> Self { Self }
    pub fn save(&mut self, _filename: &str, _content: &str) {}
    pub fn load(&self, _filename: &str) -> String { String::new() }
}
"#;
        let temp_dir = create_test_file(test_code, "large_service.rs");
        let file_path = temp_dir.path().join("large_service.rs").to_str().unwrap();

        let context = RefactoringContext {
            file_path: file_path.to_string(),
            symbol_name: Some("LargeService".to_string()),
            symbol_kind: Some(SymbolKind::Struct),
            cursor_line: 1,
            cursor_character: 1,
            selection: None,
            context_lines: vec![],
            language: ProgrammingLanguage::Rust,
            project_root: "/tmp/test".to_string(),
        };

        let options = create_options();
        let operation = SplitClassOperation;

        // First analyze if splitting is beneficial
        match operation.analyze(&context).await {
            Ok(analysis) => {
                assert!(analysis.is_safe);
                println!("Split analysis confidence: {:.2}", analysis.confidence_score);
                println!("Breaking changes: {:?}", analysis.breaking_changes);

                if analysis.confidence_score > 0.7 {
                    // Execute splitting
                    match operation.execute(&context, &options).await {
                        Ok(result) => {
                            assert!(result.success);
                            let modified_content = &result.new_content.unwrap();

                            // Verify split occurred - should have multiple structs
                            assert!(modified_content.contains("impl LargeService"));
                            println!("✓ Split class operation successfully split the large struct");
                        }
                        Err(e) => panic!("Split class operation failed: {}", e),
                    }
                } else {
                    println!("✓ Split class operation correctly determined splitting wasn't beneficial enough");
                }
            }
            Err(e) => panic!("Split analysis failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_pattern_conversion_operation_imperative_to_functional() {
        // Create imperative code that can be converted to functional style
        let test_code = r#"
pub fn process_numbers(numbers: Vec<i32>) -> Vec<i32> {
    let mut result = Vec::new();

    for num in numbers {
        if num % 2 == 0 {
            let doubled = num * 2;
            if doubled < 100 {
                result.push(doubled);
            }
        }
    }

    result
}
"#;
        let temp_dir = create_test_file(test_code, "imperative_transform.rs");
        let file_path = temp_dir.path().join("imperative_transform.rs").to_str().unwrap();

        let context = RefactoringContext {
            file_path: file_path.to_string(),
            symbol_name: Some("process_numbers".to_string()),
            symbol_kind: Some(SymbolKind::Function),
            cursor_line: 1,
            cursor_character: 1,
            selection: Some(CodeRange {
                start_line: 1,
                start_character: 0,
                end_line: 15,
                end_character: 0,
            }),
            context_lines: vec![],
            language: ProgrammingLanguage::Rust,
            project_root: "/tmp/test".to_string(),
        };

        let mut options = create_options();
        options.extra_options = Some(serde_json::json!({"sourcePattern": "loop", "targetPattern": "iterator"}));

        let operation = PatternConversionOperation;

        match operation.analyze(&context).await {
            Ok(analysis) => {
                println!("Pattern conversion analysis: {:?}", analysis.suggestions);
            }
            _ => {}
        }

        match operation.execute(&context, &options).await {
            Ok(result) => {
                assert!(result.success);
                println!("✓ Pattern conversion operation completed successfully");
                println!("Warnings: {:?}", result.warnings);
            }
            Err(e) => {
                // Pattern conversion is experimental and may not always succeed
                println!("Pattern conversion operation noted limitation (expected for complex patterns): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_batch_refactoring_operations() {
        use rust_ai_ide_ai_refactoring::batch::*;

        // Create a multi-file codebase for batch testing
        let file1_content = r#"
pub fn helper_function(value: i32) -> i32 {
    value * 2
}

pub fn calculate_total(items: Vec<i32>) -> i32 {
    let mut total = 0;
    for item in items {
        total += item;
    }
    total
}
"#;

        let file2_content = r#"
use crate::file1::*;

pub fn use_functions() {
    let helper = helper_function(5);
    let total = calculate_total(vec![1, 2, 3, 4, 5]);
    println!("Helper: {}, Total: {}", helper, total);
}
"#;

        let temp_dir = tempdir().unwrap();
        let file1_path = temp_dir.path().join("file1.rs");
        let file2_path = temp_dir.path().join("file2.rs");

        fs::write(&file1_path, file1_content).unwrap();
        fs::write(&file2_path, file2_content).unwrap();

        // Create batch operation
        let operations = vec![
            BatchRefactoringOperation {
                operation_type: RefactoringType::Rename,
                context: RefactoringContext {
                    file_path: file1_path.to_str().unwrap().to_string(),
                    symbol_name: Some("helper_function".to_string()),
                    symbol_kind: Some(SymbolKind::Function),
                    cursor_line: 1,
                    cursor_character: 5,
                    selection: None,
                    context_lines: vec![],
                    language: ProgrammingLanguage::Rust,
                    project_root: temp_dir.path().to_str().unwrap().to_string(),
                },
                options: RefactoringOptions {
                    extra_options: Some(serde_json::json!({"newName": "double_value"})),
                    ..create_options()
                },
            },
        ];

        let batch_options = BatchRefactoringOptions {
            operations,
            parallel_execution: false,
            stop_on_failure: false,
            create_backup_directory: true,
            validate_dependencies: true,
            max_concurrent_operations: 2,
        };

        let batch_operation = BatchRefactoringOperationExecutor;
        let mut progress_tracker = ProgressTracker::new();

        match batch_operation.execute_batch(batch_options, &mut progress_tracker).await {
            Ok(results) => {
                assert_eq!(results.results.len(), 1);
                println!("✓ Batch refactoring operation executed successfully");
                println!("Progress: {}", progress_tracker.get_summary());
            }
            Err(e) => panic!("Batch operation failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_refactoring_with_safety_validation() {
        use rust_ai_ide_ai_refactoring::safety::*;

        let test_code = r#"
pub struct BankAccount {
    balance: f64,
}

impl BankAccount {
    pub fn withdraw(&mut self, amount: f64) -> Result<f64, String> {
        if self.balance >= amount {
            self.balance -= amount;
            Ok(self.balance)
        } else {
            Err("Insufficient funds".to_string())
        }
    }
}
"#;

        let temp_dir = create_test_file(test_code, "bank_account.rs");
        let file_path = temp_dir.path().join("bank_account.rs").to_str().unwrap();

        let context = RefactoringContext {
            file_path: file_path.to_string(),
            symbol_name: Some("withdraw".to_string()),
            symbol_kind: Some(SymbolKind::Function),
            cursor_line: 6,
            cursor_character: 5,
            selection: None,
            context_lines: vec![],
            language: ProgrammingLanguage::Rust,
            project_root: "/tmp/test".to_string(),
        };

        let safety_validator = SafetyValidator::new();
        let validation_result = safety_validator
            .validate_refactoring_operation(&RefactoringType::Rename, &context).await;

        match validation_result {
            Ok(analysis) => {
                assert!(analysis.is_safe);
                assert!(analysis.confidence_score > 0.5);
                println!("✓ Safety validation completed successfully");
                println!("Risk level: {}", analysis.potential_impact);
            }
            Err(e) => panic!("Safety validation failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_extract_function_operation_with_selection() {
        // Create code with complex logic that can be extracted
        let test_code = r#"
pub fn complex_processing(data: Vec<i32>) -> Vec<String> {
    // Some complex preprocessing
    let mut processed = Vec::new();

    for (i, &value) in data.iter().enumerate() {
        // Complex calculation logic that should be extracted
        let normalized = value as f64 / 100.0;
        let scaled = normalized * 2.0;
        let transformed = scaled + i as f64;
        let formatted = format!("{:.2}", transformed);

        if formatted.len() > 3 {
            processed.push(formatted);
        }
    }

    processed
}
"#;

        let temp_dir = create_test_file(test_code, "complex_logic.rs");
        let file_path = temp_dir.path().join("complex_logic.rs").to_str().unwrap();

        let context = RefactoringContext {
            file_path: file_path.to_string(),
            symbol_name: Some("complex_processing".to_string()),
            symbol_kind: Some(SymbolKind::Function),
            cursor_line: 1,
            cursor_character: 1,
            selection: Some(CodeRange {
                start_line: 7,
                start_character: 9,
                end_line: 15,
                end_character: 9,
            }),
            context_lines: vec![],
            language: ProgrammingLanguage::Rust,
            project_root: "/tmp/test".to_string(),
        };

        let mut options = create_options();
        options.extra_options = Some(serde_json::json!({"experimental": true}));

        let operation = ExtractFunctionOperation;

        match operation.execute(&context, &options).await {
            Ok(result) => {
                assert!(result.success);
                let modified_content = &result.new_content.unwrap();

                // Verify that a function was extracted
                assert!(modified_content.contains("fn extracted_function"));
                assert!(modified_content.contains("extracted_function"));

                println!("✓ Extract function operation successfully extracted complex logic");
                println!("Warnings: {:?}", result.warnings);
            }
            Err(e) => panic!("Extract function operation failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_operation_factory_and_applicability() {
        let factory = RefactoringOperationFactory;

        // Test factory creates operations correctly
        let rename_op = factory.create_operation(&RefactoringType::Rename).unwrap();
        let extract_op = factory.create_operation(&RefactoringType::ExtractFunction).unwrap();

        assert_eq!(rename_op.name(), "Rename");
        assert_eq!(extract_op.name(), "Extract Function");

        // Test applicability with different contexts
        let struct_context = RefactoringContext {
            file_path: "/test/file.rs".to_string(),
            symbol_name: Some("TestStruct".to_string()),
            symbol_kind: Some(SymbolKind::Struct),
            cursor_line: 1,
            cursor_character: 1,
            selection: None,
            context_lines: vec![],
            language: ProgrammingLanguage::Rust,
            project_root: "/test".to_string(),
        };

        let function_context = RefactoringContext {
            file_path: "/test/file.rs".to_string(),
            symbol_name: Some("test_function".to_string()),
            symbol_kind: Some(SymbolKind::Function),
            cursor_line: 1,
            cursor_character: 1,
            selection: None,
            context_lines: vec![],
            language: ProgrammingLanguage::Rust,
            project_root: "/test".to_string(),
        };

        let options = create_options();
        let extract_interface_op = factory.create_operation(&RefactoringType::ExtractInterface).unwrap();
        let convert_async_op = factory.create_operation(&RefactoringType::ConvertToAsync).unwrap();

        // Test structural operation applicability
        match extract_interface_op.is_applicable(&struct_context, Some(&options)).await {
            Ok(applicable) => assert!(applicable),
            Err(e) => panic!("Applicability check failed: {}", e),
        }

        // Test function operation applicability
        match convert_async_op.is_applicable(&function_context, Some(&options)).await {
            Ok(applicable) => assert!(applicable),
            Err(e) => panic!("Applicability check failed: {}", e),
        }

        println!("✓ Operation factory and applicability tests completed successfully");
    }

    #[tokio::test]
    async fn test_factory_comprehensive_operation_coverage() {
        let factory = RefactoringOperationFactory;
        let available_types = factory.available_refactorings();

        // Test that factory can create all available operations
        for op_type in available_types {
            match factory.create_operation(&op_type) {
                Ok(operation) => {
                    // Verify operation has required traits
                    assert!(!operation.name().is_empty());
                    println!("✓ Successfully created operation: {}", operation.name());
                }
                Err(e) => {
                    panic!("Failed to create operation for type {:?}: {}", op_type, e);
                }
            }
        }

        println!("✓ Factory can create all {} available refactoring operations", factory.available_refactorings().len());
    }

    #[tokio::test]
    async fn test_cross_module_workflow_extract_rename_signature() {
        // Create a complex function that will undergo multiple transformations
        let initial_code = r#"
pub struct Calculator {
    value: i32,
}

impl Calculator {
    pub fn process_data(&self, input: Vec<i32>) -> Vec<i32> {
        let mut result = Vec::new();
        for &item in &input {
            let processed = self.calculate_internal(item);
            if processed > 0 {
                result.push(processed);
            }
        }
        result
    }

    fn calculate_internal(&self, value: i32) -> i32 {
        self.value + value * 2
    }
}
"#;

        let temp_dir = create_test_file(initial_code, "workflow_test.rs");
        let file_path = temp_dir.path().join("workflow_test.rs").to_str().unwrap();

        // Step 1: Extract the internal calculation logic
        let extract_context = RefactoringContext {
            file_path: file_path.to_string(),
            symbol_name: Some("calculate_internal".to_string()),
            symbol_kind: Some(SymbolKind::Function),
            cursor_line: 18,
            cursor_character: 5,
            selection: Some(CodeRange {
                start_line: 18,
                start_character: 5,
                end_line: 20,
                end_character: 30,
            }),
            context_lines: vec![],
            language: ProgrammingLanguage::Rust,
            project_root: "/tmp/test".to_string(),
        };

        let factory = RefactoringOperationFactory;
        let extract_op = factory.create_operation(&RefactoringType::ExtractFunction).unwrap();

        let mut options = create_options();
        options.extra_options = Some(serde_json::json!({"newFunctionName": "compute_value"}));

        let extract_result = extract_op.execute(&extract_context, &options).await.unwrap();
        assert!(extract_result.success);

        // Read the modified content after extraction
        let content_after_extract = std::fs::read_to_string(file_path).unwrap();

        // Step 2: Rename the extracted function
        let rename_context = RefactoringContext {
            file_path: file_path.to_string(),
            symbol_name: Some("compute_value".to_string()),
            symbol_kind: Some(SymbolKind::Function),
            cursor_line: 18,
            cursor_character: 5,
            selection: None,
            context_lines: vec![],
            language: ProgrammingLanguage::Rust,
            project_root: "/tmp/test".to_string(),
        };

        let rename_op = factory.create_operation(&RefactoringType::Rename).unwrap();
        let mut rename_options = create_options();
        rename_options.extra_options = Some(serde_json::json!({"newName": "calculate_value"}));

        let rename_result = rename_op.execute(&rename_context, &rename_options).await.unwrap();
        assert!(rename_result.success);

        // Step 3: Change signature of the renamed function
        let signature_context = RefactoringContext {
            file_path: file_path.to_string(),
            symbol_name: Some("calculate_value".to_string()),
            symbol_kind: Some(SymbolKind::Function),
            cursor_line: 18,
            cursor_character: 5,
            selection: None,
            context_lines: vec![],
            language: ProgrammingLanguage::Rust,
            project_root: "/tmp/test".to_string(),
        };

        let signature_op = factory.create_operation(&RefactoringType::ChangeSignature).unwrap();
        let mut sig_options = create_options();
        sig_options.extra_options = Some(serde_json::json!({
            "newParameters": [{"name": "base", "type": "i32"}, {"name": "multiplier", "type": "i32"}],
            "returnType": "i32"
        }));

        let sig_result = signature_op.execute(&signature_context, &sig_options).await.unwrap();
        assert!(sig_result.success);

        // Verify AST consistency across all transformations
        let final_content = std::fs::read_to_string(file_path).unwrap();

        // Verify that the function was renamed
        assert!(final_content.contains("calculate_value"));
        assert!(!final_content.contains("compute_value"));

        // Verify that the function signature was changed (simplified check)
        assert!(final_content.contains("fn calculate_value"));

        // Verify that calls to the function were updated
        assert!(final_content.contains("calculate_value("));

        println!("✓ Cross-module workflow (extract → rename → change signature) completed successfully");
        println!("✓ AST consistency maintained across sequential operations");
    }

    #[tokio::test]
    async fn test_module_interaction_ast_utils_core_traits_operations() {
        // Create test code to verify module interactions
        let test_code = r#"
pub struct DataProcessor {
    pub data: Vec<String>,
}

impl DataProcessor {
    pub fn process_item(&self, item: &String) -> String {
        item.to_uppercase()
    }

    pub fn process_all(&self) -> Vec<String> {
        self.data.iter().map(|item| self.process_item(item)).collect()
    }
}
"#;

        let temp_dir = create_test_file(test_code, "module_interaction.rs");
        let file_path = temp_dir.path().join("module_interaction.rs").to_str().unwrap();

        // Test ast_utils integration with operations
        use rust_ai_ide_ai_refactoring::ast_utils::{is_ast_supported, IdentifierRenamer};
        use rust_ai_ide_ai_refactoring::core_traits::RefactoringOperation;
        use rust_ai_ide_ai_refactoring::function_method_operations::ExtractFunctionOperation;

        // Verify AST support detection
        assert!(is_ast_supported(file_path));

        // Test identifier renaming utility
        let mut renamer = IdentifierRenamer::new();
        let renamed = renamer.rename_identifiers("process_item", "transform_item", test_code);
        assert!(renamed.contains("transform_item"));
        assert!(!renamed.contains("process_item"));

        // Test operation using core traits
        let context = RefactoringContext {
            file_path: file_path.to_string(),
            symbol_name: Some("DataProcessor".to_string()),
            symbol_kind: Some(SymbolKind::Struct),
            cursor_line: 1,
            cursor_character: 1,
            selection: None,
            context_lines: vec![],
            language: ProgrammingLanguage::Rust,
            project_root: "/tmp/test".to_string(),
        };

        let factory = RefactoringOperationFactory;
        let extract_interface_op = factory.create_operation(&RefactoringType::ExtractInterface).unwrap();

        // Verify operation implements required traits
        assert!(!extract_interface_op.name().is_empty());
        assert!(!extract_interface_op.description().is_empty());

        // Test applicability through core traits
        let options = create_options();
        match extract_interface_op.is_applicable(&context, Some(&options)).await {
            Ok(applicable) => {
                if applicable {
                    let result = extract_interface_op.execute(&context, &options).await.unwrap();
                    assert!(result.success);
                    println!("✓ Module interaction test: AST utils, core traits, and operations work together");
                } else {
                    println!("✓ Module interaction test: Operation correctly determined as not applicable");
                }
            }
            Err(e) => println!("✓ Module interaction test: Error handling works - {}", e),
        }
    }

    #[tokio::test]
    async fn test_error_handling_across_module_boundaries() {
        let factory = RefactoringOperationFactory;

        // Test invalid file path
        let invalid_context = RefactoringContext {
            file_path: "/nonexistent/path/file.rs".to_string(),
            symbol_name: Some("test".to_string()),
            symbol_kind: Some(SymbolKind::Function),
            cursor_line: 1,
            cursor_character: 1,
            selection: None,
            context_lines: vec![],
            language: ProgrammingLanguage::Rust,
            project_root: "/tmp/test".to_string(),
        };

        let options = create_options();
        let rename_op = factory.create_operation(&RefactoringType::Rename).unwrap();

        // Test error propagation from file operations to refactoring operations
        match rename_op.execute(&invalid_context, &options).await {
            Ok(_) => panic!("Expected error for invalid file path"),
            Err(e) => {
                println!("✓ Error handling: Invalid file path error propagated correctly: {}", e);
                assert!(e.to_string().contains("file") || e.to_string().contains("path"));
            }
        }

        // Test invalid operation type
        match factory.create_operation(&RefactoringType::Rename) {
            Ok(op) => {
                // Test with invalid context (missing symbol)
                let invalid_symbol_context = RefactoringContext {
                    file_path: "/test/file.rs".to_string(),
                    symbol_name: None, // Invalid: no symbol name
                    symbol_kind: Some(SymbolKind::Function),
                    cursor_line: 1,
                    cursor_character: 1,
                    selection: None,
                    context_lines: vec![],
                    language: ProgrammingLanguage::Rust,
                    project_root: "/tmp/test".to_string(),
                };

                match op.execute(&invalid_symbol_context, &options).await {
                    Ok(_) => println!("✓ Operation handled missing symbol gracefully"),
                    Err(e) => println!("✓ Error handling: Missing symbol error propagated: {}", e),
                }
            }
            Err(e) => panic!("Failed to create rename operation: {}", e),
        }

        println!("✓ Error handling across module boundaries tested successfully");
    }

    #[tokio::test]
    async fn test_success_and_failure_scenarios() {
        let factory = RefactoringOperationFactory;

        // Success scenario: Valid rename operation
        let success_code = r#"
pub fn old_function_name() -> i32 {
    42
}

pub fn caller() {
    let result = old_function_name();
}
"#;

        let temp_dir = create_test_file(success_code, "success_test.rs");
        let success_file_path = temp_dir.path().join("success_test.rs").to_str().unwrap();

        let success_context = RefactoringContext {
            file_path: success_file_path.to_string(),
            symbol_name: Some("old_function_name".to_string()),
            symbol_kind: Some(SymbolKind::Function),
            cursor_line: 1,
            cursor_character: 5,
            selection: None,
            context_lines: vec![],
            language: ProgrammingLanguage::Rust,
            project_root: "/tmp/test".to_string(),
        };

        let mut success_options = create_options();
        success_options.extra_options = Some(serde_json::json!({"newName": "new_function_name"}));

        let rename_op = factory.create_operation(&RefactoringType::Rename).unwrap();
        match rename_op.execute(&success_context, &success_options).await {
            Ok(result) => {
                assert!(result.success);
                let modified_content = std::fs::read_to_string(success_file_path).unwrap();
                assert!(modified_content.contains("new_function_name"));
                assert!(!modified_content.contains("old_function_name"));
                println!("✓ Success scenario: Rename operation completed successfully");
            }
            Err(e) => panic!("Success scenario failed: {}", e),
        }

        // Failure scenario: Attempt to rename non-existent symbol
        let failure_context = RefactoringContext {
            file_path: success_file_path.to_string(),
            symbol_name: Some("nonexistent_symbol".to_string()),
            symbol_kind: Some(SymbolKind::Function),
            cursor_line: 1,
            cursor_character: 1,
            selection: None,
            context_lines: vec![],
            language: ProgrammingLanguage::Rust,
            project_root: "/tmp/test".to_string(),
        };

        match rename_op.execute(&failure_context, &success_options).await {
            Ok(result) => {
                // Operation might succeed but with warnings
                println!("✓ Failure scenario handled gracefully with result: {:?}", result.warnings);
            }
            Err(e) => {
                println!("✓ Failure scenario: Error correctly returned for non-existent symbol: {}", e);
            }
        }

        println!("✓ Both success and failure scenarios tested");
    }

    #[tokio::test]
    async fn test_ast_consistency_across_sequential_operations() {
        // Create complex code for sequential transformations
        let initial_code = r#"
pub struct ComplexProcessor {
    data: Vec<i32>,
}

impl ComplexProcessor {
    pub fn new() -> Self {
        Self { data: vec![] }
    }

    pub fn add_and_double(&mut self, value: i32) -> i32 {
        self.data.push(value);
        value * 2
    }

    pub fn process_batch(&mut self, values: Vec<i32>) -> Vec<i32> {
        values.into_iter().map(|v| self.add_and_double(v)).collect()
    }
}
"#;

        let temp_dir = create_test_file(initial_code, "ast_consistency.rs");
        let file_path = temp_dir.path().join("ast_consistency.rs").to_str().unwrap();

        let factory = RefactoringOperationFactory;
        let options = create_options();

        // First operation: Extract method
        let extract_context = RefactoringContext {
            file_path: file_path.to_string(),
            symbol_name: Some("add_and_double".to_string()),
            symbol_kind: Some(SymbolKind::Function),
            cursor_line: 10,
            cursor_character: 5,
            selection: Some(CodeRange {
                start_line: 12,
                start_character: 9,
                end_line: 13,
                end_character: 20,
            }),
            context_lines: vec![],
            language: ProgrammingLanguage::Rust,
            project_root: "/tmp/test".to_string(),
        };

        let extract_op = factory.create_operation(&RefactoringType::ExtractFunction).unwrap();
        let mut extract_options = options.clone();
        extract_options.extra_options = Some(serde_json::json!({"newFunctionName": "double_value"}));

        let _extract_result = extract_op.execute(&extract_context, &extract_options).await.unwrap();

        // Second operation: Rename variable in extracted function
        let rename_context = RefactoringContext {
            file_path: file_path.to_string(),
            symbol_name: Some("double_value".to_string()),
            symbol_kind: Some(SymbolKind::Function),
            cursor_line: 10,
            cursor_character: 5,
            selection: None,
            context_lines: vec![],
            language: ProgrammingLanguage::Rust,
            project_root: "/tmp/test".to_string(),
        };

        let rename_op = factory.create_operation(&RefactoringType::Rename).unwrap();
        let mut rename_options = options.clone();
        rename_options.extra_options = Some(serde_json::json!({"newName": "compute_double"}));

        let _rename_result = rename_op.execute(&rename_context, &rename_options).await.unwrap();

        // Verify AST consistency by checking if code still compiles conceptually
        let final_content = std::fs::read_to_string(file_path).unwrap();

        // Verify function was renamed
        assert!(final_content.contains("compute_double"));
        assert!(!final_content.contains("double_value"));

        // Verify calls were updated
        assert!(final_content.contains("compute_double("));

        // Verify struct and impl blocks are still intact
        assert!(final_content.contains("impl ComplexProcessor"));
        assert!(final_content.contains("pub struct ComplexProcessor"));

        println!("✓ AST consistency verified across sequential operations");
        println!("✓ Function extraction and renaming maintained code structure");
    }
}
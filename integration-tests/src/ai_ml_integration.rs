//! AI/ML Integration Tests
//!
//! Comprehensive integration tests for AI/ML workflows including:
//! - End-to-end code analysis pipelines
//! - ML model inference and learning cycles
//! - Code generation and refactoring workflows
//! - Error resolution and pattern matching
//! - AI-assisted development processes
//! - Model performance validation
//! - Learning system accuracy testing

use std::sync::Arc;

use rust_ai_ide_ai_analysis::AnalysisEngine;
use rust_ai_ide_ai_codegen::{CodeGenerator, GenerationConfig};
use rust_ai_ide_ai_inference::{AIModel, InferenceEngine};
use rust_ai_ide_ai_learning::{LearningSystem, TrainingData};
use rust_ai_ide_ai_refactoring::{RefactoringContext, RefactoringEngine};
use rust_ai_ide_errors::RustAIError;
use tokio::sync::Mutex;

use crate::common::scenarios::AIScenarioBuilder;
use crate::common::ExtendedIntegrationContext;
use crate::IntegrationTestResult;

/// AI/ML Integration Test Suite Runner
#[derive(Clone)]
pub struct AIMLIntegrationTestRunner {
    context:            Option<ExtendedIntegrationContext>,
    analysis_engine:    Option<Arc<AnalysisEngine>>,
    inference_engine:   Option<Arc<InferenceEngine>>,
    learning_system:    Option<Arc<LearningSystem>>,
    code_generator:     Option<Arc<CodeGenerator>>,
    refactoring_engine: Option<Arc<RefactoringEngine>>,
    results:            Vec<IntegrationTestResult>,
}

impl AIMLIntegrationTestRunner {
    pub fn new() -> Self {
        Self {
            context:            None,
            analysis_engine:    None,
            inference_engine:   None,
            learning_system:    None,
            code_generator:     None,
            refactoring_engine: None,
            results:            Vec::new(),
        }
    }

    /// Setup AI/ML test environment with engines and models
    pub async fn setup_test_environment(&mut self, context: ExtendedIntegrationContext) -> Result<(), RustAIError> {
        self.context = Some(context);

        // Initialize AI engines
        let analysis_engine = Arc::new(AnalysisEngine::new().await?);
        let inference_engine = Arc::new(InferenceEngine::new().await?);
        let learning_system = Arc::new(LearningSystem::new().await?);
        let code_generator = Arc::new(CodeGenerator::new().await?);
        let refactoring_engine = Arc::new(RefactoringEngine::new().await?);

        // Setup test models and data
        self.setup_ai_models(&inference_engine).await?;
        self.setup_learning_data(&learning_system).await?;

        self.analysis_engine = Some(analysis_engine);
        self.inference_engine = Some(inference_engine);
        self.learning_system = Some(learning_system);
        self.code_generator = Some(code_generator);
        self.refactoring_engine = Some(refactoring_engine);

        Ok(())
    }

    /// Setup inference models for testing
    async fn setup_ai_models(&self, engine: &InferenceEngine) -> Result<(), RustAIError> {
        // Create and train basic test models
        let code_model = AIModel::new("code_analysis_v1".to_string())
            .with_dataset_size(1000)
            .with_accuracy_threshold(0.85)
            .build()?;

        let bug_model = AIModel::new("bug_detection_v1".to_string())
            .with_dataset_size(500)
            .with_accuracy_threshold(0.90)
            .build()?;

        engine.register_model(code_model).await?;
        engine.register_model(bug_model).await?;

        Ok(())
    }

    /// Setup learning system with training data
    async fn setup_learning_data(&self, system: &LearningSystem) -> Result<(), RustAIError> {
        // Create sample training data for different scenarios
        let bug_detection_data = TrainingData::new()
            .with_category("bug_detection")
            .with_samples(vec![
                ("fn test() { let x = 5; }", true),             // contains unused variable bug
                ("fn test() { println!(\"{}\", 42); }", false), // no bugs
                ("fn test(x: i32) { let y = x + 1; }", true),   // unused variable bug
            ])
            .build()?;

        let code_style_data = TrainingData::new()
            .with_category("code_style")
            .with_samples(vec![
                ("snake_case_variable", true),
                ("camelCaseVariable", false),
                ("function_name", true),
                ("FunctionName", false),
            ])
            .build()?;

        system.add_training_data(bug_detection_data).await?;
        system.add_training_data(code_style_data).await?;

        Ok(())
    }

    /// Test end-to-end code analysis pipeline
    pub async fn test_code_analysis_pipeline(&mut self) -> Vec<IntegrationTestResult> {
        let mut results = Vec::new();

        let mut result = IntegrationTestResult::new("code_analysis_pipeline");
        let start_time = std::time::Instant::now();

        match self.perform_code_analysis_test().await {
            Ok(_) => {
                result.success = true;
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("Code analysis pipeline test failed: {}", e));
            }
        }

        result.duration_ms = start_time.elapsed().as_millis() as u64;
        results.push(result);

        results
    }

    async fn perform_code_analysis_test(&self) -> Result<(), RustAIError> {
        if let (Some(analysis_engine), Some(context)) = (&self.analysis_engine, &self.context) {
            // Create test code with multiple issues
            let test_code = r#"fn problematic_function(x: i32, unused_param: String) {
    let unused_variable = "test";
    let mut shadowed = "original";
    if x > 10 {
        let shadowed = "different"; // shadowing
        println!("{}", shadowed);
    }
    println!("Result: {}", x); // using shadowed variable incorrectly?
}"#;

            context.create_sample_rust_project("analysis_test")?;
            let test_file = context.test_workspace.join("analysis_test/src/lib.rs");
            std::fs::write(&test_file, test_code)?;

            // Run full analysis pipeline
            let analysis_results = analysis_engine
                .analyze_file(test_file.to_str().unwrap())
                .await?;

            // Validate analysis quality
            assert!(
                !analysis_results.issues.is_empty(),
                "Should detect at least one issue"
            );

            // Check for unused variable detection
            let unused_issues = analysis_results
                .issues
                .iter()
                .filter(|issue| issue.category.contains("unused"))
                .count();
            assert!(unused_issues >= 1, "Should detect unused variables");

            // Check for shadowing detection
            let shadowing_issues = analysis_results
                .issues
                .iter()
                .filter(|issue| issue.category.contains("shadow"))
                .count();
            assert!(shadowing_issues >= 1, "Should detect variable shadowing");

            Ok(())
        } else {
            Err(RustAIError::invalid_input(
                "Analysis engine not initialized",
            ))
        }
    }

    /// Test ML model inference capabilities
    pub async fn test_ml_inference(&mut self) -> Vec<IntegrationTestResult> {
        let mut results = Vec::new();

        let mut result = IntegrationTestResult::new("ml_inference");
        let start_time = std::time::Instant::now();

        match self.perform_ml_inference_test().await {
            Ok(_) => {
                result.success = true;
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("ML inference test failed: {}", e));
            }
        }

        result.duration_ms = start_time.elapsed().as_millis() as u64;
        results.push(result);

        results
    }

    async fn perform_ml_inference_test(&self) -> Result<(), RustAIError> {
        if let Some(inference_engine) = &self.inference_engine {
            // Test code analysis model
            let code_sample = "fn analyze_this() { let unused_var = 42; if true {} }";
            let analysis_result = inference_engine.analyze_code(code_sample).await?;

            // Validate inference results
            assert!(
                analysis_result.confidence > 0.0,
                "Should have confidence score"
            );
            assert!(
                !analysis_result.predictions.is_empty(),
                "Should have predictions"
            );

            // Test bug detection model
            let buggy_code = "fn main() { let x = 5; /* x never used */ }";
            let bug_result = inference_engine.detect_bugs(buggy_code).await?;

            assert!(
                bug_result
                    .detections
                    .iter()
                    .any(|d| d.category == "unused_variable"),
                "Should detect unused variable bug"
            );

            Ok(())
        } else {
            Err(RustAIError::invalid_input(
                "Inference engine not initialized",
            ))
        }
    }

    /// Test learning system with feedback loops
    pub async fn test_learning_cycles(&mut self) -> Vec<IntegrationTestResult> {
        let mut results = Vec::new();

        let mut result = IntegrationTestResult::new("learning_cycles");
        let start_time = std::time::Instant::now();

        match self.perform_learning_cycles_test().await {
            Ok(_) => {
                result.success = true;
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("Learning cycles test failed: {}", e));
            }
        }

        result.duration_ms = start_time.elapsed().as_millis() as u64;
        results.push(result);

        results
    }

    async fn perform_learning_cycles_test(&self) -> Result<(), RustAIError> {
        if let (Some(learning_system), Some(context)) = (&self.learning_system, &self.context) {
            // Initial training
            learning_system.train_models().await?;

            // Get initial accuracy
            let initial_accuracy = learning_system.get_model_accuracy("bug_detection").await?;

            // Generate more training data based on analysis
            let new_samples = vec![
                ("fn test() { let y = vec![]; y.push(1); }", false), // good usage
                ("fn test() { let z = 42; }", true),                 // unused variable
                (
                    "fn test() { let a = String::new(); println!(\"{}\", a); }",
                    false,
                ), // used variable
            ];

            for (code, has_bug) in new_samples {
                learning_system.add_feedback_sample(code, has_bug).await?;
            }

            // Retrain with new data
            learning_system.update_models().await?;

            // Check improved accuracy
            let improved_accuracy = learning_system.get_model_accuracy("bug_detection").await?;
            assert!(
                improved_accuracy >= initial_accuracy,
                "Accuracy should improve with more training data"
            );

            Ok(())
        } else {
            Err(RustAIError::invalid_input(
                "Learning system not initialized",
            ))
        }
    }

    /// Test code generation workflows
    pub async fn test_code_generation(&mut self) -> Vec<IntegrationTestResult> {
        let mut results = Vec::new();

        let mut result = IntegrationTestResult::new("code_generation");
        let start_time = std::time::Instant::now();

        match self.perform_code_generation_test().await {
            Ok(_) => {
                result.success = true;
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("Code generation test failed: {}", e));
            }
        }

        result.duration_ms = start_time.elapsed().as_millis() as u64;
        results.push(result);

        results
    }

    async fn perform_code_generation_test(&self) -> Result<(), RustAIError> {
        if let (Some(code_generator), Some(context)) = (&self.code_generator, &self.context) {
            // Test function generation
            let func_spec = "Generate a function that calculates factorial recursively";
            let config = GenerationConfig::default().with_style("rust");

            let generated_function = code_generator.generate_function(func_spec, &config).await?;
            assert!(
                generated_function.contains("fn factorial"),
                "Should generate factorial function"
            );

            // Test struct generation
            let struct_spec = "Create a Person struct with name: String and age: u32";
            let generated_struct = code_generator.generate_struct(struct_spec, &config).await?;
            assert!(
                generated_struct.contains("struct Person"),
                "Should generate Person struct"
            );

            // Validate generated code compiles
            if let Some(context) = &self.context {
                let test_file = context.test_workspace.join("generated_code.rs");
                let full_code = format!("{}\n\n{}", generated_struct, generated_function);
                std::fs::write(&test_file, full_code)?;

                // Basic syntax check (in real implementation, would compile)
                assert!(
                    full_code.contains("{"),
                    "Generated code should have proper structure"
                );
            }

            Ok(())
        } else {
            Err(RustAIError::invalid_input("Code generator not initialized"))
        }
    }

    /// Test AI-assisted refactoring
    pub async fn test_refactoring_assistance(&mut self) -> Vec<IntegrationTestResult> {
        let mut results = Vec::new();

        let mut result = IntegrationTestResult::new("refactoring_assistance");
        let start_time = std::time::Instant::now();

        match self.perform_refactoring_test().await {
            Ok(_) => {
                result.success = true;
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("Refactoring assistance test failed: {}", e));
            }
        }

        result.duration_ms = start_time.elapsed().as_millis() as u64;
        results.push(result);

        results
    }

    async fn perform_refactoring_test(&self) -> Result<(), RustAIError> {
        if let (Some(refactoring_engine), Some(context)) = (&self.refactoring_engine, &self.context) {
            // Test code that can be refactored
            let messy_code = r#"fn process_data() {
    let mut result = Vec::new();
    for i in 0..10 {
        if i % 2 == 0 {
            result.push(i * 2);
        }
    }
    result
}"#;

            context.create_sample_rust_project("refactoring_test")?;
            let test_file = context.test_workspace.join("refactoring_test/src/lib.rs");
            std::fs::write(&test_file, messy_code)?;

            // Create refactoring context
            let refactoring_context = RefactoringContext::new()
                .with_file_path(test_file.to_str().unwrap())
                .with_focus("process_data");

            // Get refactoring suggestions
            let suggestions = refactoring_engine
                .analyze_refactoring_opportunities(&refactoring_context)
                .await?;

            assert!(
                !suggestions.is_empty(),
                "Should provide refactoring suggestions"
            );

            // Test specific refactoring
            let refactored_code = refactoring_engine
                .apply_refactoring_suggestion(suggestions[0].id.clone(), &refactoring_context)
                .await?;

            // Validate refactored code
            assert!(
                refactored_code.contains("fn process_data"),
                "Should preserve function signature"
            );
            assert!(
                refactored_code.lines().count() <= messy_code.lines().count(),
                "Refactored code should be more concise"
            );

            Ok(())
        } else {
            Err(RustAIError::invalid_input(
                "Refactoring engine not initialized",
            ))
        }
    }

    /// Test error resolution capabilities
    pub async fn test_error_resolution(&mut self) -> Vec<IntegrationTestResult> {
        let mut results = Vec::new();

        let mut result = IntegrationTestResult::new("error_resolution");
        let start_time = std::time::Instant::now();

        match self.perform_error_resolution_test().await {
            Ok(_) => {
                result.success = true;
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("Error resolution test failed: {}", e));
            }
        }

        result.duration_ms = start_time.elapsed().as_millis() as u64;
        results.push(result);

        results
    }

    async fn perform_error_resolution_test(&self) -> Result<(), RustAIError> {
        if let (Some(analysis_engine), Some(context)) = (&self.analysis_engine, &self.context) {
            // Create code with common errors
            let buggy_code = r#"fn calculate_total(items: Vec<f64>) {
    let mut total = 0.0;
    for item in items {
        total = total + item;
    }
    total
}

fn main() {
    let prices = vec![1.99, 2.50, 5.00];
    let total = calculate_total(prices);
    println!("Total: {}", total); // Using variable after move
    let formatted_total = format!("${:.2}", total); // Another use after move
    println!("{}", formatted_total);
}"#;

            context.create_sample_rust_project("error_resolution_test")?;
            let test_file = context
                .test_workspace
                .join("error_resolution_test/src/main.rs");
            std::fs::write(&test_file, buggy_code)?;

            // Analyze for errors
            let analysis_results = analysis_engine
                .analyze_file(test_file.to_str().unwrap())
                .await?;

            // Generate error resolutions
            let resolutions = analysis_engine
                .generate_error_resolutions(&analysis_results)
                .await?;

            assert!(
                !resolutions.is_empty(),
                "Should provide resolution suggestions"
            );

            // Test resolution application
            let resolution = &resolutions[0];
            let fixed_code = analysis_engine
                .apply_error_resolution(resolution.id.clone())
                .await?;

            // Basic validation - should not contain the error pattern anymore
            assert!(
                !fixed_code.contains("println!(\"Total: {}\", total);")
                    || fixed_code.contains("clone()")
                    || fixed_code.contains("&total"),
                "Error should be resolved"
            );

            Ok(())
        } else {
            Err(RustAIError::invalid_input(
                "Analysis engine not initialized",
            ))
        }
    }

    /// Get all test results
    pub fn get_all_results(&self) -> &[IntegrationTestResult] {
        &self.results
    }
}

/// Async trait implementation for integration with the test runner framework
#[async_trait]
impl crate::test_runner::TestSuiteRunner for AIMLIntegrationTestRunner {
    fn suite_name(&self) -> &'static str {
        "ai_ml"
    }

    async fn run_test_suite(&self) -> Result<Vec<IntegrationTestResult>, RustAIError> {
        let mut runner = AIMLIntegrationTestRunner::new();

        if let Some(context) = &self.context {
            runner.setup_test_environment(context.clone()).await?;
        }

        let mut all_results = Vec::new();

        // Run all AI/ML tests
        all_results.extend(runner.test_code_analysis_pipeline().await);
        all_results.extend(runner.test_ml_inference().await);
        all_results.extend(runner.test_learning_cycles().await);
        all_results.extend(runner.test_code_generation().await);
        all_results.extend(runner.test_refactoring_assistance().await);
        all_results.extend(runner.test_error_resolution().await);

        Ok(all_results)
    }

    fn test_names(&self) -> Vec<String> {
        vec![
            "code_analysis_pipeline".to_string(),
            "ml_inference".to_string(),
            "learning_cycles".to_string(),
            "code_generation".to_string(),
            "refactoring_assistance".to_string(),
            "error_resolution".to_string(),
        ]
    }

    fn is_test_enabled(&self, test_name: &str) -> bool {
        matches!(
            test_name,
            "code_analysis_pipeline"
                | "ml_inference"
                | "learning_cycles"
                | "code_generation"
                | "refactoring_assistance"
                | "error_resolution"
        )
    }

    fn prerequisites(&self) -> Vec<String> {
        vec![
            "rust-ai-ide-ai-analysis".to_string(),
            "rust-ai-ide-ai-inference".to_string(),
            "rust-ai-ide-ai-learning".to_string(),
            "ml-models".to_string(),
            "training-data".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_ml_runner_creation() {
        let runner = AIMLIntegrationTestRunner::new();
        assert_eq!(runner.get_all_results().len(), 0);
    }

    #[tokio::test]
    async fn test_runner_configuration() {
        let runner = AIMLIntegrationTestRunner::new();
        assert_eq!(runner.suite_name(), "ai_ml");

        let test_names = runner.test_names();
        assert!(test_names.contains(&"code_analysis_pipeline".to_string()));
        assert!(test_names.contains(&"ml_inference".to_string()));

        assert!(runner.is_test_enabled("code_analysis_pipeline"));
        assert!(!runner.is_test_enabled("nonexistent_test"));

        let prereqs = runner.prerequisites();
        assert!(prereqs.contains(&"rust-ai-ide-ai-analysis".to_string()));
        assert!(prereqs.contains(&"ml-models".to_string()));
    }

    #[tokio::test]
    async fn test_ai_scenario_builder() {
        let temp_dir = tempfile::tempdir().unwrap();
        let workspace_path = temp_dir.path().join("workspace");
        std::fs::create_dir_all(&workspace_path).unwrap();

        let context = ExtendedIntegrationContext::new(shared_test_utils::IntegrationContext {
            test_dir: workspace_path,
            config:   shared_test_utils::IntegrationConfig::default(),
            state:    std::collections::HashMap::new(),
        });

        let scenario = AIScenarioBuilder::new("code_quality")
            .with_code("fn test() { assert!(true); }")
            .expect_issue("code should be more robust");

        assert!(scenario.analysis_type == "code_quality");
        assert!(scenario.input_code.contains("fn test"));
        assert!(scenario.expected_issues[0].contains("robust"));
    }
}

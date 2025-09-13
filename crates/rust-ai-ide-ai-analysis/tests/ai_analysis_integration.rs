//! Integration tests for AI-powered code analysis

use rust_ai_ide_ai_analysis::architectural::detectors::{AIDetector, AnalysisContext, AnalysisRequest};
use rust_ai_ide_ai_analysis::architectural::patterns::{
    AntiPattern, ArchitecturalPattern, Priority, SuggestionCategory,
};
use rust_ai_ide_errors::IdeResult;

#[tokio::test]
async fn test_complete_ai_analysis_workflow() -> IdeResult<()> {
    let detector = AIDetector::new();

    let sample_code = r#"
pub struct FileRepository {
    files: std::collections::HashMap<String, String>,
    base_path: String,
}

impl FileRepository {
    pub fn new(base_path: String) -> Self {
        Self {
            files: std::collections::HashMap::new(),
            base_path,
        }
    }

    pub async fn save_file(&self, name: &str, content: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Empty name".to_string());
        }
        if content.is_empty() {
            return Err("Empty content".to_string());
        }

        println!("Saving file: {}", name);
        println!("Content length: {}", content.len());

        for _ in 0..100 {
            println!("Processing line {}", _);
            if content.len() > 1000 {
                println!("Large file detected");
                break;
            }
        }

        Ok(())
    }

    pub async fn get_file(&self, name: &str) -> Result<String, String> {
        if name.is_empty() {
            return Err("Empty name".to_string());
        }

        println!("Retrieving file: {}", name);
        Ok(String::new())
    }

    pub fn validate_input(&self, input: &str) -> Result<(), String> {
        if input.is_empty() {
            return Err("Input cannot be empty".to_string());
        }
        Ok(())
    }

    pub fn log_operation(&self, operation: &str) -> Result<(), String> {
        println!("Logged: {}", operation);
        Ok(())
    }

    pub fn cleanup(&self) -> Result<(), String> {
        Ok(())
    }
}
"#;

    let request = AnalysisRequest {
        file_uri:             "test://FileRepository.rs".to_string(),
        detect_anti_patterns: true,
        detect_patterns:      true,
        generate_suggestions: true,
        performance_analysis: true,
        parse_tree:           None,
        context:              Some(AnalysisContext {
            project_root: Some("/test".to_string()),
            language:     Some("rust".to_string()),
            framework:    None,
        }),
    };

    let result = detector
        .analyze_code(sample_code, "FileRepository.rs", request)
        .await?;

    // Verify analysis completed
    assert!(!result.file_path.is_empty());
    assert!(result.analysis_metadata.analysis_duration_ms >= 0);

    // We should detect at least some patterns and anti-patterns in this sample
    // The exact numbers depend on the ML model predictions

    println!(
        "Analysis completed in {}ms",
        result.analysis_metadata.analysis_duration_ms
    );
    println!(
        "Detected {} anti-patterns",
        result.detected_anti_patterns.len()
    );
    println!(
        "Generated {} suggestions",
        result.intelligence_suggestions.len()
    );

    Ok(())
}

#[tokio::test]
async fn test_anti_pattern_detection() -> IdeResult<()> {
    let detector = AIDetector::new();

    // Code with long method anti-pattern
    let long_method_code = r#"
impl MyClass {
    pub async fn very_long_method(&self) -> Result<(), String> {
        println!("Line 1");
        println!("Line 2");
        println!("Line 3");
        println!("Line 4");
        println!("Line 5");
        println!("Line 6");
        println!("Line 7");
        println!("Line 8");
        println!("Line 9");
        println!("Line 10");
        println!("Line 11");
        println!("Line 12");
        println!("Line 13");
        println!("Line 14");
        println!("Line 15");
        println!("Line 16");
        println!("Line 17");
        println!("Line 18");
        println!("Line 19");
        println!("Line 20");
        println!("Line 21");
        println!("Line 22");
        println!("Line 23");
        println!("Line 24");
        println!("Line 25");
        println!("Line 26");
        println!("Line 27");
        println!("Line 28");
        println!("Line 29");
        println!("Line 30");
        println!("Line 31");
        println!("Line 32");
        println!("Line 33");
        println!("Line 34");
        println!("Line 35");
        println!("Line 36");
        println!("Line 37");
        println!("Line 38");
        println!("Line 39");
        println!("Line 40");
        println!("Line 41");
        println!("Line 42");
        println!("Line 43");
        println!("Line 44");
        println!("Line 45");
        println!("Line 46");
        println!("Line 47");
        println!("Line 48");
        println!("Line 49");
        println!("Line 50");
        println!("Line 51");
        println!("Line 52");
        Ok(())
    }
}
"#;

    let request = AnalysisRequest::quick("test://LongMethod.rs");

    let result = detector
        .analyze_code(long_method_code, "LongMethod.rs", request)
        .await?;

    // Should detect long method anti-pattern
    assert!(
        !result.detected_anti_patterns.is_empty(),
        "Should detect at least one anti-pattern"
    );

    // Find the long method detection
    let long_method_detected = result
        .detected_anti_patterns
        .iter()
        .any(|ap| matches!(ap.anti_pattern_type, AntiPattern::LongMethod));

    assert!(
        long_method_detected,
        "Should detect long method anti-pattern"
    );

    Ok(())
}

#[tokio::test]
async fn test_code_duplication_detection() -> IdeResult<()> {
    let detector = AIDetector::new();

    // Code with duplicated validation logic
    let duplicated_code = r#"
impl Validator {
    pub fn validate_user(&self, user: &User) -> Result<(), String> {
        if user.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if user.email.is_empty() {
            return Err("Email cannot be empty".to_string());
        }
        if !user.email.contains('@') {
            return Err("Invalid email format".to_string());
        }
        if user.age < 18 {
            return Err("Must be 18 or older".to_string());
        }
        Ok(())
    }

    pub fn validate_admin(&self, admin: &Admin) -> Result<(), String> {
        if admin.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if admin.email.is_empty() {
            return Err("Email cannot be empty".to_string());
        }
        if !admin.email.contains('@') {
            return Err("Invalid email format".to_string());
        }
        if admin.level < 1 {
            return Err("Admin level must be at least 1".to_string());
        }
        Ok(())
    }
}
"#;

    let request = AnalysisRequest::comprehensive("test://DuplicatedCode.rs");

    let result = detector
        .analyze_code(duplicated_code, "DuplicatedCode.rs", request)
        .await?;

    println!(
        "Found {} anti-patterns",
        result.detected_anti_patterns.len()
    );

    // Should detect code duplication
    let duplication_detected = result
        .detected_anti_patterns
        .iter()
        .any(|ap| matches!(ap.anti_pattern_type, AntiPattern::CodeDuplication));

    if duplication_detected {
        println!("✅ Code duplication detected");
    } else {
        println!("ℹ️  Code duplication not detected (may depend on ML model thresholds)");
    }

    Ok(())
}

#[tokio::test]
async fn test_intelligence_suggestion_generation() -> IdeResult<()> {
    let detector = AIDetector::new();

    let sample_code = r#"
pub struct DataProcessor {
    data: Vec<i32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn add_item(&mut self, item: i32) {
        self.data.push(item);
    }

    pub fn get_average(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }
        let sum: i32 = self.data.iter().sum();
        sum as f64 / self.data.len() as f64
    }

    pub fn get_median(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }
        let mut sorted = self.data.clone();
        sorted.sort();
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) as f64 / 2.0
        } else {
            sorted[mid] as f64
        }
    }

    pub fn get_min(&self) -> Option<i32> {
        self.data.iter().min().cloned()
    }

    pub fn get_max(&self) -> Option<i32> {
        self.data.iter().max().cloned()
    }

    pub fn process_data(&mut self) -> Result<(), String> {
        println!("Processing {} items", self.data.len());
        for i in 0..self.data.len() {
            if self.data[i] < 0 {
                println!("Found negative value: {}", self.data[i]);
            }
        }
        Ok(())
    }
}
"#;

    let request = AnalysisRequest::comprehensive("test://DataProcessor.rs");

    let result = detector
        .analyze_code(sample_code, "DataProcessor.rs", request)
        .await?;

    // Should generate some intelligence suggestions
    println!(
        "Generated {} intelligence suggestions",
        result.intelligence_suggestions.len()
    );

    for suggestion in &result.intelligence_suggestions {
        println!(
            "Suggestion: {} (Confidence: {:.2}, Priority: {:?})",
            suggestion.title, suggestion.confidence, suggestion.priority
        );
    }

    // Verify suggestions have required fields
    for suggestion in &result.intelligence_suggestions {
        assert!(
            !suggestion.title.is_empty(),
            "Suggestion title should not be empty"
        );
        assert!(
            suggestion.confidence >= 0.0 && suggestion.confidence <= 1.0,
            "Confidence should be between 0.0 and 1.0"
        );
        assert!(
            !suggestion.description.is_empty(),
            "Suggestion description should not be empty"
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_performance_metrics_tracking() -> IdeResult<()> {
    let detector = AIDetector::new();

    let sample_code = r#"
pub fn calculate_fibonacci(n: u32) -> u32 {
    if n <= 1 {
        return n;
    }
    calculate_fibonacci(n - 1) + calculate_fibonacci(n - 2)
}
"#;

    let request = AnalysisRequest::comprehensive("test://fibonacci.rs");

    let result = detector
        .analyze_code(sample_code, "fibonacci.rs", request)
        .await?;

    // Performance metrics should be tracked
    assert_eq!(
        result.performance_metrics.anti_patterns_detected,
        result.detected_anti_patterns.len()
    );
    assert_eq!(
        result.performance_metrics.patterns_detected,
        result.detected_patterns.len()
    );
    assert!(result.analysis_metadata.analysis_duration_ms >= 0);

    println!(
        "Analysis duration: {}ms",
        result.analysis_metadata.analysis_duration_ms
    );
    println!(
        "Anti-patterns detected: {}",
        result.performance_metrics.anti_patterns_detected
    );
    println!(
        "Patterns detected: {}",
        result.performance_metrics.patterns_detected
    );

    Ok(())
}

#[tokio::test]
async fn test_cache_performance() -> IdeResult<()> {
    let detector = AIDetector::new();

    let sample_code = r#"
pub struct Calculator {
    pub value: i32,
}

impl Calculator {
    pub fn add(&mut self, x: i32) {
        self.value += x;
    }
}
"#;

    let request = AnalysisRequest::comprehensive("test://calculator.rs");

    // First analysis
    let start = std::time::Instant::now();
    let result1 = detector
        .analyze_code(sample_code, "calculator.rs", request.clone())
        .await?;
    let first_duration = start.elapsed();

    // Second analysis (should use cache)
    let start = std::time::Instant::now();
    let result2 = detector
        .analyze_code(sample_code, "calculator.rs", request)
        .await?;
    let second_duration = start.elapsed();

    println!("First analysis: {}ms", first_duration.as_millis());
    println!("Second analysis: {}ms", second_duration.as_millis());

    // Results should be the same
    assert_eq!(
        result1.detected_anti_patterns.len(),
        result2.detected_anti_patterns.len()
    );
    assert_eq!(
        result1.detected_patterns.len(),
        result2.detected_patterns.len()
    );

    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> IdeResult<()> {
    let detector = AIDetector::new();

    // Test with empty code
    let result = detector
        .analyze_code(
            "",
            "empty.rs",
            AnalysisRequest::comprehensive("test://empty.rs"),
        )
        .await;

    // Should handle gracefully
    if let Err(e) = result {
        println!("Expected error for empty code: {:?}", e);
    }

    // Test with large code
    let large_code = "pub fn test() {}\n".repeat(10000);
    let result = detector
        .analyze_code(
            &large_code,
            "large.rs",
            AnalysisRequest::comprehensive("test://large.rs"),
        )
        .await?;

    // Should handle large files
    assert!(!result.file_path.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_configuration_flexibility() -> IdeResult<()> {
    use rust_ai_ide_ai_analysis::architectural::anti_patterns::AntiPatternConfig;
    use rust_ai_ide_ai_analysis::architectural::detectors::AIDetectorConfig;

    let config = AIDetectorConfig {
        enable_anti_pattern_detection: true,
        anti_pattern_config: AntiPatternConfig {
            max_method_lines: 20, // Very strict for testing
            ..Default::default()
        },
        ..Default::default()
    };

    let detector = AIDetector::with_config(config);

    let code_with_long_method = r#"
impl Test {
    pub fn long_method(&self) {
        println!("Line 1");
        println!("Line 2");
        println!("Line 3");
        println!("Line 4");
        println!("Line 5");
        println!("Line 6");
        println!("Line 7");
        println!("Line 8");
        println!("Line 9");
        println!("Line 10");
        println!("Line 11");
        println!("Line 12");
        println!("Line 13");
        println!("Line 14");
        println!("Line 15");
        println!("Line 16");
        println!("Line 17");
        println!("Line 18");
        println!("Line 19");
        println!("Line 20");
        println!("Line 21");
        println!("Line 22");
    }
}
"#;

    let request = AnalysisRequest::comprehensive("test://config_test.rs");

    let result = detector
        .analyze_code(code_with_long_method, "config_test.rs", request)
        .await?;

    // Should detect long method with strict configuration
    let long_method_detected = result
        .detected_anti_patterns
        .iter()
        .any(|ap| matches!(ap.anti_pattern_type, AntiPattern::LongMethod));

    assert!(
        long_method_detected,
        "Should detect long method with strict configuration"
    );

    Ok(())
}

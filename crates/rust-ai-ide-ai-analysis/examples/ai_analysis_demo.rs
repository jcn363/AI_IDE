//! AI-Powered Code Analysis Demo
//!
//! This example demonstrates the AI analysis capabilities including pattern detection,
//! anti-pattern analysis, and intelligent suggestion generation.

use rust_ai_ide_ai_analysis::architectural::detectors::{AIDetector, AnalysisContext, AnalysisRequest};
use rust_ai_ide_ai_analysis::architectural::patterns::SuggestionCategory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Rust AI IDE - Intelligent Code Analysis Demo");
    println!("==============================================\n");

    // Sample code with various patterns and anti-patterns
    let sample_code = r#"
/// A sample repository pattern implementation
pub struct FileRepository {
    files: HashMap<String, Vec<u8>>,
    base_path: String,
    connection_string: String,
    max_connections: usize,
    timeout_duration: Duration,
    retry_count: u32,
    cache_size: usize,
    metrics_collector: MetricsCollector,
    error_handler: ErrorHandler,
    validation_rules: Vec<ValidationRule>,
    event_publisher: EventPublisher,
}

impl FileRepository {
    /// Create a new file repository
    pub fn new(
        base_path: String,
        connection_string: String,
        max_connections: usize,
        timeout_duration: Duration,
        retry_count: u32,
        cache_size: usize,
    ) -> Self {
        // This constructor is getting very long...
        let metrics_collector = MetricsCollector::new();
        let error_handler = ErrorHandler::new();
        let validation_rules = vec![
            ValidationRule::new("file_size", "max", "1048576"),
            ValidationRule::new("file_name", "pattern", r"^[a-zA-Z0-9_\-\.]+$"),
            ValidationRule::new("file_extension", "allowed", "txt,pdf,doc,docx"),
        ];
        let event_publisher = EventPublisher::new("file_repository_events");

        Self {
            files: HashMap::new(),
            base_path,
            connection_string,
            max_connections,
            timeout_duration,
            retry_count,
            cache_size,
            metrics_collector,
            error_handler,
            validation_rules,
            event_publisher,
        }
    }

    /// Save a file to the repository
    pub async fn save_file(&self, file_name: &str, content: &[u8]) -> Result<(), String> {
        // This method does too many things
        println!("Saving file: {}", file_name);

        // Validate input
        if file_name.is_empty() {
            return Err("File name cannot be empty".to_string());
        }
        if content.is_empty() {
            return Err("File content cannot be empty".to_string());
        }
        if file_name.len() > 255 {
            return Err("File name too long".to_string());
        }

        // Check file extension
        let extension_check = file_name.split('.').last().unwrap_or("");
        match extension_check {
            "txt" => println!("Text file detected"),
            "pdf" => println!("PDF file detected"),
            "doc" => println!("Word document detected"),
            "docx" => println!("Word document detected"),
            _ => return Err("Unsupported file type".to_string()),
        }

        // Log the operation
        println!("File operation logged: {}", file_name);

        // Actually save the file
        // ... implementation details ...

        Ok(())
    }

    /// Get a file from the repository
    pub async fn get_file(&self, file_name: &str) -> Result<Vec<u8>, String> {
        // Similar duplicated validation logic
        if file_name.is_empty() {
            return Err("File name cannot be empty".to_string());
        }
        if file_name.len() > 255 {
            return Err("File name too long".to_string());
        }

        // Log the operation
        println!("File retrieval logged: {}", file_name);

        // Actually retrieve the file
        // ... implementation details ...

        Ok(vec![])
    }

    /// Delete a file from the repository
    pub async fn delete_file(&self, file_name: &str) -> Result<(), String> {
        // Yet another copy of validation logic
        if file_name.is_empty() {
            return Err("File name cannot be empty".to_string());
        }
        if file_name.len() > 255 {
            return Err("File name too long".to_string());
        }

        // Log the operation
        println!("File deletion logged: {}", file_name);

        // Actually delete the file
        // ... implementation details ...

        Ok(())
    }

    // Many more methods would be here, violating Single Responsibility Principle
    pub fn get_metrics(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    pub fn validate_connection(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn cleanup_expired_files(&self) -> Result<usize, String> {
        Ok(0)
    }

    pub fn backup_repository(&self) -> Result<(), String> {
        Ok(())
    }

    // More methods making this class too large...
}
"#;

    // Create analysis detector
    let detector = AIDetector::new();

    // Create analysis request
    let request = AnalysisRequest {
        file_uri:             "demo://FileRepository.rs".to_string(),
        detect_anti_patterns: true,
        detect_patterns:      true,
        generate_suggestions: true,
        performance_analysis: true,
        parse_tree:           None,
        context:              Some(AnalysisContext {
            project_root: Some("/demo/project".to_string()),
            language:     Some("rust".to_string()),
            framework:    None,
        }),
    };

    println!("🔍 Analyzing code for patterns and anti-patterns...\n");

    // Perform analysis
    let result = detector
        .analyze_code(sample_code, "FileRepository.rs", request)
        .await?;

    // Display results
    println!("📊 Analysis Results:");
    println!("===================");
    println!("📁 File: {}", result.file_path);
    println!(
        "⏱️  Analysis Duration: {}ms",
        result.analysis_metadata.analysis_duration_ms
    );
    println!("🔍 Detected Patterns: {}", result.detected_patterns.len());
    println!(
        "🚨 Anti-patterns Found: {}",
        result.detected_anti_patterns.len()
    );
    println!(
        "💡 Intelligence Suggestions: {}",
        result.intelligence_suggestions.len()
    );

    // Display detected anti-patterns
    if !result.detected_anti_patterns.is_empty() {
        println!("\n🚨 Detected Anti-patterns:");
        println!("==========================");

        for (i, anti_pattern) in result.detected_anti_patterns.iter().enumerate() {
            println!(
                "{}. {} ({:.1}%)",
                i + 1,
                anti_pattern.anti_pattern_type.description(),
                anti_pattern.confidence * 100.0
            );
            println!(
                "   📍 Location: Lines {}-{} ({} lines)",
                anti_pattern.location.start_line,
                anti_pattern.location.end_line,
                anti_pattern.location.end_line - anti_pattern.location.start_line + 1
            );
            println!("   ⚠️  Severity: {:?}", anti_pattern.severity);
            println!("   🔧 Suggestions:");
            for suggestion in &anti_pattern.suggestions {
                println!("      • {}", suggestion);
            }
            println!("   📊 Metrics:");
            println!(
                "      • Violation Score: {:.2}",
                anti_pattern.metrics.violation_score
            );
            println!(
                "      • Maintainability Impact: {:.2}",
                anti_pattern.metrics.maintainability_impact
            );
            println!(
                "      • Refactoring Effort: {:.1} days",
                anti_pattern.metrics.refactoring_effort_days
            );
            println!();
        }
    }

    // Display intelligence suggestions
    if !result.intelligence_suggestions.is_empty() {
        println!("\n💡 Intelligent Suggestions:");
        println!("============================");

        for (i, suggestion) in result.intelligence_suggestions.iter().enumerate() {
            let priority_icon = match suggestion.priority {
                rust_ai_ide_ai_analysis::architectural::patterns::Priority::Critical => "🚨",
                rust_ai_ide_ai_analysis::architectural::patterns::Priority::High => "⚠️",
                rust_ai_ide_ai_analysis::architectural::patterns::Priority::Medium => "ℹ️",
                rust_ai_ide_ai_analysis::architectural::patterns::Priority::Low => "💡",
                rust_ai_ide_ai_analysis::architectural::patterns::Priority::Info => "💭",
            };

            println!(
                "{}. {} {} ({:.1}%)",
                i + 1,
                priority_icon,
                suggestion.title,
                suggestion.confidence * 100.0
            );
            println!("   📋 Category: {:?}", suggestion.category);
            println!("   🔄 Refactoring Type: {:?}", suggestion.refactoring_type);
            println!(
                "   📍 Location: Lines {}-{}",
                suggestion.location.start_line, suggestion.location.end_line
            );

            if !suggestion.expected_benefits.is_empty() {
                println!("   ✅ Expected Benefits:");
                for benefit in &suggestion.expected_benefits {
                    println!("      • {}", benefit);
                }
            }

            if !suggestion.implementation_guidance.is_empty() {
                println!(
                    "   🛠️  Implementation: {}",
                    suggestion.implementation_guidance
                );
            }

            println!();
        }
    }

    // Display detected patterns
    if !result.detected_patterns.is_empty() {
        println!("\n🏗️  Detected Architectural Patterns:");
        println!("===================================");

        for (i, pattern) in result.detected_patterns.iter().enumerate() {
            println!(
                "{}. {} ({:.1}%)",
                i + 1,
                pattern.pattern_type.description(),
                pattern.confidence * 100.0
            );
            println!(
                "   📍 Location: Lines {}-{}",
                pattern.location.start_line, pattern.location.end_line
            );
            println!("   🔍 Structural Info:");
            println!(
                "      • Lines of Code: {}",
                pattern.context.structural_info.lines_of_code
            );
            println!(
                "      • Cyclomatic Complexity: {}",
                pattern.context.structural_info.cyclomatic_complexity
            );
            println!(
                "      • Method Count: {}",
                pattern.context.structural_info.method_count
            );
            println!();
        }
    }

    // Display performance metrics
    println!("\n📈 Performance Metrics:");
    println!("=======================");
    println!(
        "⚡ Anti-patterns Detected: {}",
        result.performance_metrics.anti_patterns_detected
    );
    println!(
        "🏗️  Patterns Detected: {}",
        result.performance_metrics.patterns_detected
    );

    println!("\n🎯 Demo completed successfully!");
    println!("===============================");
    println!("The AI analysis engine successfully:");
    println!("✓ Detected anti-patterns with ML-enhanced confidence scoring");
    println!("✓ Generated prioritized refactoring suggestions");
    println!("✓ Identified architectural patterns");
    println!("✓ Provided implementation guidance and effort estimates");

    Ok(())
}

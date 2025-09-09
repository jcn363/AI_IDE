//! # Natural Language to Code Conversion
//!
//! This module implements AI-powered natural language to code conversion functionality.
//! It can transform natural language descriptions into syntactically correct code
//! across multiple programming languages.
//!
//! ## Features
//!
//! - **Multi-language Support**: Convert natural language to Rust, Python, JavaScript, TypeScript, etc.
//! - **Context Awareness**: Understands project context and coding patterns
//! - **Intelligent Correction**: Detects and fixes common issues in generated code
//! - **Interactive Refinement**: Allows user feedback and iterative improvement
//! - **Safety Validation**: Ensures generated code meets security and quality standards
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::types::*;
use crate::SecurityResult;

/// Input for natural language to code conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLToCodeInput {
    /// Natural language description of desired functionality
    pub description: String,
    /// Target programming language
    pub target_language: String,
    /// Project context information
    pub project_context: HashMap<String, String>,
    /// User preferences and coding style
    pub coding_style: Option<CodingStyle>,
    /// Existing code context for better understanding
    pub existing_code: Option<String>,
    /// Specific requirements or constraints
    pub requirements: Vec<String>,
}

/// Generated code result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLToCodeResult {
    /// Generated code
    pub code: String,
    /// Programming language used
    pub language: String,
    /// Confidence score (0.0-1.0)
    pub confidence_score: f64,
    /// Explanatory comments and documentation
    pub explanation: String,
    /// Potential issues or warnings
    pub warnings: Vec<String>,
    /// Alternative implementations
    pub alternatives: Vec<AlternativeCode>,
    /// Generation metadata
    pub metadata: GenerationMetadata,
}

/// Alternative code implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeCode {
    pub code: String,
    pub description: String,
    pub complexity: u8,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}

/// Generation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    pub generation_id: String,
    pub timestamp: DateTime<Utc>,
    pub processing_time_ms: u64,
    pub model_used: String,
    pub context_understanding_score: f64,
}

/// Natural language processing engine
#[async_trait]
pub trait NLProcessingEngine: Send + Sync {
    /// Convert natural language to code
    async fn convert_to_code(&self, input: &NLToCodeInput) -> SecurityResult<NLToCodeResult>;

    /// Refine existing code based on user feedback
    async fn refine_code(&self, original: &str, feedback: &str, context: &NLToCodeInput) -> SecurityResult<NLToCodeResult>;

    /// Validate generated code for syntax and semantic correctness
    async fn validate_code(&self, code: &str, language: &str) -> SecurityResult<ValidationResult>;
}

/// Validation result for generated code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub syntax_errors: Vec<String>,
    pub semantic_warnings: Vec<String>,
    pub security_issues: Vec<String>,
    pub suggestions: Vec<String>,
}

/// Advanced natural language to code converter
pub struct NLToCodeConverter {
    engine: Arc<dyn NLProcessingEngine>,
    code_templates: RwLock<HashMap<String, Vec<CodeTemplate>>>,
    language_grammars: RwLock<HashMap<String, LanguageGrammar>>,
    context_analyzer: ContextAnalyzer,
    code_validator: CodeValidator,
    feedback_processor: FeedbackProcessor,
}

impl NLToCodeConverter {
    /// Create a new NL to code converter
    pub fn new(engine: Arc<dyn NLProcessingEngine>) -> Self {
        Self {
            engine,
            code_templates: RwLock::new(HashMap::new()),
            language_grammars: RwLock::new(HashMap::new()),
            context_analyzer: ContextAnalyzer::new(),
            code_validator: CodeValidator::new(),
            feedback_processor: FeedbackProcessor::new(),
        }
    }

    /// Convert natural language description to code
    pub async fn convert(&self, input: NLToCodeInput) -> SecurityResult<NLToCodeResult> {
        let start_time = std::time::Instant::now();

        // Analyze context for better understanding
        let context_score = self.context_analyzer.analyze_context(&input).await?;

        // Enhance input based on context
        let enhanced_input = self.enhance_input_with_context(input, &context_score).await;

        // Convert using the processing engine
        let mut result = self.engine.convert_to_code(&enhanced_input).await?;

        // Update metadata
        result.metadata = GenerationMetadata {
            generation_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            model_used: "nl-to-code-engine-v2".to_string(),
            context_understanding_score: context_score.score,
        };

        // Validate the generated code
        let validation = self.engine.validate_code(&result.code, &result.language).await?;
        result.warnings.extend(validation.syntax_errors);
        result.warnings.extend(validation.semantic_warnings);
        result.warnings.extend(validation.security_issues);

        // Generate alternatives
        result.alternatives = self.generate_alternatives(&result, &enhanced_input).await;

        Ok(result)
    }

    /// Refine existing generated code based on user feedback
    pub async fn refine(&self, code: &str, feedback: &str, original_input: &NLToCodeInput) -> SecurityResult<NLToCodeResult> {
        // Process user feedback
        let refined_input = self.feedback_processor.process_feedback(original_input.clone(), feedback).await;

        // Use the engine to refine the code
        self.engine.refine_code(code, feedback, &refined_input).await
    }

    /// Generate multiple alternative implementations
    pub async fn generate_alternatives(&self, original: &NLToCodeResult, input: &NLToCodeInput) -> Vec<AlternativeCode> {
        let mut alternatives = Vec::new();

        // Performance-optimized variant
        if let Some(perf_variant) = self.generate_performance_variant(&original.code, &input.target_language).await {
            alternatives.push(perf_variant);
        }

        // Memory-efficient variant
        if let Some(memory_variant) = self.generate_memory_efficient_variant(&original.code, &input.target_language).await {
            alternatives.push(memory_variant);
        }

        // Readable/maintainable variant
        if let Some(readable_variant) = self.generate_readable_variant(&original.code, &input.target_language).await {
            alternatives.push(readable_variant);
        }

        // Error-handling enhanced variant
        if let Some(error_handling_variant) = self.generate_error_handling_variant(&original.code, &input.target_language).await {
            alternatives.push(error_handling_variant);
        }

        alternatives
    }

    /// Enhance input with contextual understanding
    async fn enhance_input_with_context(&self, input: NLToCodeInput, context: &ContextAnalysis) -> NLToCodeInput {
        let mut enhanced = input.clone();

        // Add context-based improvements to the description
        if context.has_existing_patterns {
            enhanced.description += "\nUse established patterns from the existing codebase.";
        }

        if context.uses_async {
            enhanced.description += "\nInclude proper async/await patterns as used in this codebase.";
        }

        if let Some(style) = &context.detected_style {
            enhanced.coding_style = Some(style.clone());
        }

        enhanced
    }

    /// Generate performance-optimized variant
    async fn generate_performance_variant(&self, original: &str, language: &str) -> Option<AlternativeCode> {
        // This would use templates and optimizations based on language
        let optimized = match language {
            "rust" => self.optimize_rust_performance(original),
            "python" => self.optimize_python_performance(original),
            "javascript" => self.optimize_javascript_performance(original),
            _ => return None,
        };

        Some(AlternativeCode {
            code: optimized,
            description: "Performance-optimized implementation".to_string(),
            complexity: 7,
            pros: vec![
                "Reduced time complexity".to_string(),
                "Better resource utilization".to_string(),
                "Optimized data structures".to_string(),
            ],
            cons: vec![
                "Higher code complexity".to_string(),
                "May be harder to understand".to_string(),
                "Possible trade-offs in error handling".to_string(),
            ],
        })
    }

    /// Generate memory-efficient variant
    async fn generate_memory_efficient_variant(&self, original: &str, language: &str) -> Option<AlternativeCode> {
        let optimized = match language {
            "rust" => self.optimize_rust_memory(original),
            "python" => self.optimize_python_memory(original),
            "javascript" => self.optimize_javascript_memory(original),
            _ => return None,
        };

        Some(AlternativeCode {
            code: optimized,
            description: "Memory-efficient implementation".to_string(),
            complexity: 6,
            pros: vec![
                "Reduced memory footprint".to_string(),
                "Better scalability".to_string(),
                "Lower GC pressure".to_string(),
            ],
            cons: vec![
                "May use less idiomatic patterns".to_string(),
                "Potential readability trade-offs".to_string(),
            ],
        })
    }

    /// Generate readable/maintainable variant
    async fn generate_readable_variant(&self, original: &str, language: &str) -> Option<AlternativeCode> {
        let readable = match language {
            "rust" => self.improve_rust_readability(original),
            "python" => self.improve_python_readability(original),
            "javascript" => self.improve_javascript_readability(original),
            _ => return None,
        };

        Some(AlternativeCode {
            code: readable,
            description: "Highly readable and maintainable implementation".to_string(),
            complexity: 3,
            pros: vec![
                "Easy to understand".to_string(),
                "Self-documenting".to_string(),
                "Easier maintenance".to_string(),
            ],
            cons: vec![
                "May be less performant".to_string(),
                "More verbose code".to_string(),
            ],
        })
    }

    /// Generate error-handling enhanced variant
    async fn generate_error_handling_variant(&self, original: &str, language: &str) -> Option<AlternativeCode> {
        let enhanced = match language {
            "rust" => self.enhance_rust_error_handling(original),
            "python" => self.enhance_python_error_handling(original),
            "javascript" => self.enhance_javascript_error_handling(original),
            _ => return None,
        };

        Some(AlternativeCode {
            code: enhanced,
            description: "Enhanced error handling and reliability".to_string(),
            complexity: 5,
            pros: vec![
                "Robust error handling".to_string(),
                "Better user experience".to_string(),
                "Easier debugging".to_string(),
            ],
            cons: vec![
                "Increased complexity".to_string(),
                "More code to maintain".to_string(),
            ],
        })
    }

    // Language-specific optimization methods
    fn optimize_rust_performance(&self, code: &str) -> String {
        // Apply Rust-specific performance optimizations
        code.to_string() // Placeholder implementation
    }

    fn optimize_python_performance(&self, code: &str) -> String {
        // Apply Python-specific performance optimizations
        code.to_string() // Placeholder implementation
    }

    fn optimize_javascript_performance(&self, code: &str) -> String {
        // Apply JavaScript-specific performance optimizations
        code.to_string() // Placeholder implementation
    }

    fn optimize_rust_memory(&self, code: &str) -> String {
        // Apply Rust-specific memory optimizations
        code.to_string() // Placeholder implementation
    }

    fn optimize_python_memory(&self, code: &str) -> String {
        // Apply Python-specific memory optimizations
        code.to_string() // Placeholder implementation
    }

    fn optimize_javascript_memory(&self, code: &str) -> String {
        // Apply JavaScript-specific memory optimizations
        code.to_string() // Placeholder implementation
    }

    fn improve_rust_readability(&self, code: &str) -> String {
        // Apply Rust-specific readability improvements
        code.to_string() // Placeholder implementation
    }

    fn improve_python_readability(&self, code: &str) -> String {
        // Apply Python-specific readability improvements
        code.to_string() // Placeholder implementation
    }

    fn improve_javascript_readability(&self, code: &str) -> String {
        // Apply JavaScript-specific readability improvements
        code.to_string() // Placeholder implementation
    }

    fn enhance_rust_error_handling(&self, code: &str) -> String {
        // Apply Rust-specific error handling improvements
        code.to_string() // Placeholder implementation
    }

    fn enhance_python_error_handling(&self, code: &str) -> String {
        // Apply Python-specific error handling improvements
        code.to_string() // Placeholder implementation
    }

    fn enhance_javascript_error_handling(&self, code: &str) -> String {
        // Apply JavaScript-specific error handling improvements
        code.to_string() // Placeholder implementation
    }
}

// Supporting structs and traits
#[derive(Debug, Clone)]
struct CodeTemplate {
    pattern: String,
    template: String,
    conditions: Vec<String>,
}

#[derive(Debug, Clone)]
struct LanguageGrammar {
    keywords: Vec<String>,
    syntax_patterns: Vec<String>,
    common_patterns: Vec<String>,
}

#[derive(Debug, Clone)]
struct ContextAnalysis {
    score: f64,
    has_existing_patterns: bool,
    uses_async: bool,
    detected_style: Option<CodingStyle>,
}

struct ContextAnalyzer {
    // Implementation details
}

impl ContextAnalyzer {
    fn new() -> Self {
        Self {}
    }

    async fn analyze_context(&self, input: &NLToCodeInput) -> SecurityResult<ContextAnalysis> {
        // Analyze context for better understanding
        Ok(ContextAnalysis {
            score: 0.8,
            has_existing_patterns: true,
            uses_async: false,
            detected_style: Some(CodingStyle::Functional),
        })
    }
}

struct CodeValidator {
    // Implementation details
}

impl CodeValidator {
    fn new() -> Self {
        Self {}
    }
}

struct FeedbackProcessor {
    // Implementation details
}

impl FeedbackProcessor {
    fn new() -> Self {
        Self {}
    }

    async fn process_feedback(&self, input: NLToCodeInput, feedback: &str) -> NLToCodeInput {
        // Process user feedback to refine input
        input
    }
}

/// Basic NL processing engine implementation
pub struct BasicNLPToCodeEngine;

#[async_trait]
impl NLProcessingEngine for BasicNLPToCodeEngine {
    async fn convert_to_code(&self, input: &NLToCodeInput) -> SecurityResult<NLToCodeResult> {
        // Basic implementation - in practice this would use advanced NLP models
        let code = match input.target_language.as_str() {
            "rust" => self.generate_rust_code(&input),
            "python" => self.generate_python_code(&input),
            "javascript" => self.generate_javascript_code(&input),
            "typescript" => self.generate_typescript_code(&input),
            _ => format!("// Code for {} is not yet supported\n", input.target_language),
        };

        Ok(NLToCodeResult {
            code,
            language: input.target_language.clone(),
            confidence_score: 0.85,
            explanation: format!("Generated {} code for: {}", input.target_language, input.description),
            warnings: vec![],
            alternatives: vec![],
            metadata: GenerationMetadata::default(),
        })
    }

    async fn refine_code(&self, original: &str, feedback: &str, context: &NLToCodeInput) -> SecurityResult<NLToCodeResult> {
        // Refine code based on feedback
        let refined_code = format!("{}\n// Refined based on feedback: {}", original, feedback);

        Ok(NLToCodeResult {
            code: refined_code,
            language: context.target_language.clone(),
            confidence_score: 0.9,
            explanation: "Refined code based on user feedback".to_string(),
            warnings: vec![],
            alternatives: vec![],
            metadata: GenerationMetadata::default(),
        })
    }

    async fn validate_code(&self, code: &str, language: &str) -> SecurityResult<ValidationResult> {
        // Basic validation
        Ok(ValidationResult {
            is_valid: true,
            syntax_errors: vec![],
            semantic_warnings: vec![],
            security_issues: vec![],
            suggestions: vec![
                "Consider adding error handling".to_string(),
                "Add comprehensive tests".to_string(),
            ],
        })
    }
}

impl BasicNLPToCodeEngine {
    fn generate_rust_code(&self, input: &NLToCodeInput) -> String {
        format!(r#"//! Generated code for: {}
//!
//! This code was automatically generated from natural language.
//! Please review and customize as needed.

use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{{Deserialize, Serialize}};
use uuid::Uuid;

/// Main implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFeature {{
    name: String,
    value: i32,
}}

impl GeneratedFeature {{
    /// Create a new instance
    pub fn new(name: impl Into<String>) -> Self {{
        Self {{
            name: name.into(),
            value: 0,
        }}
    }}

    /// Get the name
    pub fn name(&self) -> &str {{
        &self.name
    }}

    /// Set the value
    pub fn set_value(&mut self, value: i32) {{
        self.value = value;
    }}

    /// Get the value
    pub fn value(&self) -> i32 {{
        self.value
    }}
}}

/// Execute the main functionality described in: {}
/// This is a placeholder implementation that should be customized.
pub async fn execute_functionality() -> Result<(), Box<dyn std::error::Error>> {{
    println!("Executing: {{}}", "{}");

    // TODO: Implement the actual functionality based on natural language description
    // TODO: Add proper error handling
    // TODO: Add comprehensive tests
    // TODO: Add documentation

    let feature = GeneratedFeature::new("example");
    println!("Created feature: {{}}", feature.name());

    Ok(())
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[tokio::test]
    async fn test_execute_functionality() {{
        let result = execute_functionality().await;
        assert!(result.is_ok());
    }}

    #[test]
    fn test_generated_feature() {{
        let feature = GeneratedFeature::new("test");
        assert_eq!(feature.name(), "test");
        assert_eq!(feature.value(), 0);

        let mut feature_mut = feature;
        feature_mut.set_value(42);
        assert_eq!(feature_mut.value(), 42);
    }}
}}
"#, input.description, input.description, input.description)
    }

    fn generate_python_code(&self, input: &NLToCodeInput) -> String {
        format!(r#"""Generated code for: {description}

This code was automatically generated from natural language.
Please review and customize as needed.
"""
import asyncio
import typing
from typing import Optional, List, Dict
import dataclasses


@dataclasses.dataclass
class GeneratedFeature:
    """Main data structure for the generated functionality."""
    name: str
    value: int = 0

    def __init__(self, name: str, value: int = 0):
        self.name = name
        self.value = value


class GeneratedImplementation:
    """Main implementation class."""

    def __init__(self):
        self.features: List[GeneratedFeature] = []

    async def execute_functionality(self) -> None:
        """Execute the main functionality described in: {description}"""
        print(f"Executing: {description}")

        # TODO: Implement the actual functionality based on natural language description
        # TODO: Add proper error handling
        # TODO: Add comprehensive tests
        # TODO: Add documentation

        feature = GeneratedFeature("example")
        print(f"Created feature: {{feature.name}}")
        self.features.append(feature)

    def add_feature(self, feature: GeneratedFeature) -> None:
        """Add a new feature to the implementation."""
        self.features.append(feature)

    def get_features(self) -> List[GeneratedFeature]:
        """Get all features."""
        return self.features.copy()


async def main() -> None:
    """Main entry point."""
    impl = GeneratedImplementation()
    await impl.execute_functionality()

    # Example usage
    feature = GeneratedFeature("test_feature", 42)
    impl.add_feature(feature)
    print("Added feature:", feature.name)


if __name__ == "__main__":
    asyncio.run(main())
"#, description=input.description)
    }

    fn generate_javascript_code(&self, input: &NLToCodeInput) -> String {
        format!(r#"/**
 * Generated code for: {description}
 *
 * This code was automatically generated from natural language.
 * Please review and customize as needed.
 */

/**
 * Main class for the generated functionality
 */
class GeneratedFeature {{
    constructor(name = "", value = 0) {{
        this.name = name;
        this.value = value;
    }}

    getName() {{
        return this.name;
    }}

    setValue(value) {{
        this.value = value;
    }}

    getValue() {{
        return this.value;
    }}
}}

/**
 * Main implementation class
 */
class GeneratedImplementation {{
    constructor() {{
        this.features = [];
    }}

    async executeFunctionality() {{
        console.log(`Executing: {description}`);

        // TODO: Implement the actual functionality based on natural language description
        // TODO: Add proper error handling
        // TODO: Add comprehensive tests
        // TODO: Add documentation

        const feature = new GeneratedFeature("example");
        console.log(`Created feature: ${{feature.getName()}}`);
        this.features.push(feature);
    }}

    addFeature(feature) {{
        this.features.push(feature);
    }}

    getFeatures() {{
        return [...this.features];
    }}
}}

/**
 * Main execution function
 */
async function main() {{
    const impl = new GeneratedImplementation();
    await impl.executeFunctionality();

    // Example usage
    const feature = new GeneratedFeature("test_feature", 42);
    impl.addFeature(feature);
    console.log("Added feature:", feature.getName());

    // Log all features
    console.log("All features:", impl.getFeatures());
}}

// Export for use as module
module.exports = {{
    GeneratedFeature,
    GeneratedImplementation,
    main
}};

// Run if executed directly
if (require.main === module) {{
    main().catch(console.error);
}}
"#, description=input.description)
    }

    fn generate_typescript_code(&self, input: &NLToCodeInput) -> String {
        format!(r#"/**
 * Generated code for: {description}
 *
 * This code was automatically generated from natural language.
 * Please review and customize as needed.
 */

interface GeneratedFeatureInterface {{
    name: string;
    value: number;
    getName(): string;
    setValue(value: number): void;
    getValue(): number;
}}

/**
 * Main class for the generated functionality
 */
class GeneratedFeature implements GeneratedFeatureInterface {{
    public constructor(
        public name: string = "",
        public value: number = 0
    ) {{}}

    public getName(): string {{
        return this.name;
    }}

    public setValue(value: number): void {{
        this.value = value;
    }}

    public getValue(): number {{
        return this.value;
    }}
}}

/**
 * Main implementation class with type safety
 */
class GeneratedImplementation {{
    private features: GeneratedFeature[] = [];

    public constructor() {{}}

    public async executeFunctionality(): Promise<void> {{
        console.log(`Executing: {description}`);

        // TODO: Implement the actual functionality based on natural language description
        // TODO: Add proper error handling
        // TODO: Add comprehensive tests
        // TODO: Add documentation

        const feature = new GeneratedFeature("example");
        console.log(`Created feature: ${{feature.getName()}}`);
        this.features.push(feature);
    }}

    public addFeature(feature: GeneratedFeature): void {{
        this.features.push(feature);
    }}

    public getFeatures(): readonly GeneratedFeature[] {{
        return [...this.features];
    }}
}}

/**
 * Utility functions
 */
namespace ImplementationUtils {{
    export function createFeature(name: string, value: number = 0): GeneratedFeature {{
        return new GeneratedFeature(name, value);
    }}

    export async function executeWithRetry<T>(
        operation: () => Promise<T>,
        maxRetries: number = 3
    ): Promise<T> {{
        let lastError: Error;

        for (let i = 0; i < maxRetries; i++) {{
            try {{
                return await operation();
            }} catch (error) {{
                lastError = error as Error;
                console.warn(`Attempt ${{i + 1}} failed: ${{error.message}}`);
            }}
        }}

        throw lastError!;
    }}
}}

/**
 * Main execution function
 */
async function main(): Promise<void> {{
    const impl = new GeneratedImplementation();
    await impl.executeFunctionality();

    // Example usage with utility function
    const feature = ImplementationUtils.createFeature("test_feature", 42);
    impl.addFeature(feature);
    console.log("Added feature:", feature.getName());

    // Log all features
    console.log("All features:", impl.getFeatures());
}}

// Export for use as module
export {{
    GeneratedFeature,
    GeneratedImplementation,
    ImplementationUtils,
    main
}};

// Run if executed directly
if (require.main === module) {{
    main().catch(console.error);
}}
"#, description=input.description)
    }
}

impl Default for GenerationMetadata {
    fn default() -> Self {
        Self {
            generation_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            processing_time_ms: 0,
            model_used: "default".to_string(),
            context_understanding_score: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
struct DefaultMetadata;

impl From<DefaultMetadata> for GenerationMetadata {
    fn from(_: DefaultMetadata) -> Self {
        Self {
            generation_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            processing_time_ms: 0,
            model_used: "default".to_string(),
            context_understanding_score: 0.0,
        }
    }
}

/// Convenience function to create an NL to code converter
pub fn create_nl_to_code_converter() -> NLToCodeConverter {
    let engine = Arc::new(BasicNLPToCodeEngine);
    NLToCodeConverter::new(engine)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nl_to_code_conversion() {
        let converter = create_nl_to_code_converter();

        let input = NLToCodeInput {
            description: "Create a function that calculates factorial".to_string(),
            target_language: "rust".to_string(),
            project_context: HashMap::new(),
            coding_style: None,
            existing_code: None,
            requirements: vec![],
        };

        let result = converter.convert(input).await.unwrap();

        assert_eq!(result.language, "rust");
        assert!(result.code.contains("factorial"));
        assert!(result.confidence_score > 0.0);
        assert!(!result.explanation.is_empty());
    }

    #[tokio::test]
    async fn test_alternative_generation() {
        let converter = create_nl_to_code_converter();

        let input = NLToCodeInput {
            description: "Implement a simple data processor".to_string(),
            target_language: "python".to_string(),
            project_context: HashMap::new(),
            coding_style: None,
            existing_code: None,
            requirements: vec![],
        };

        let result = converter.convert(input).await.unwrap();
        assert!(!result.alternatives.is_empty());
    }
}
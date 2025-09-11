//! Dataset preparation for fine-tuning Rust models
//!
//! This module handles the collection, processing, and formatting of training data
//! specifically optimized for fine-tuning CodeLlama and StarCoder on Rust code.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Dataset builder for collecting and preparing training data
#[derive(Debug)]
pub struct DatasetBuilder {
    source_paths: Vec<PathBuf>,
    filters: DatasetFilters,
    processors: Vec<Box<dyn DataProcessor + Send + Sync>>,
    output_format: OutputFormat,
}

/// Dataset filters for quality control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetFilters {
    pub min_file_size: usize,
    pub max_file_size: usize,
    pub allowed_extensions: HashSet<String>,
    pub exclude_patterns: Vec<String>,
    pub quality_threshold: f32,
    pub min_complexity: u32,
    pub max_complexity: u32,
    pub max_nesting_depth: u32,
}

/// Output format for training data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    JsonL,
    Json,
    Text,
    Parquet,
}

/// Data sample for training
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSample {
    pub id: String,
    pub input: String,
    pub output: String,
    pub task_type: TaskType,
    pub metadata: SampleMetadata,
}

/// Task types for different training objectives
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    Completion,
    Infill,
    InstructionFollowing,
    ErrorCorrection,
    DocumentationGeneration,
    TestGeneration,
    Refactoring,
}

/// Metadata for training samples
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleMetadata {
    pub source_file: PathBuf,
    pub language: String,
    pub complexity: u32,
    pub quality_score: f32,
    pub tags: Vec<String>,
    pub dependencies: Vec<String>,
    pub context_window: Option<usize>,
}

/// Data processing pipeline for sample generation
#[async_trait::async_trait]
pub trait DataProcessor: Send + Sync {
    /// Process a source file and extract training samples
    async fn process_file(&self, file_path: &Path, content: &str) -> Result<Vec<TrainingSample>>;

    /// Filter samples based on quality criteria
    async fn filter_sample(&self, sample: &TrainingSample) -> Result<bool>;

    /// Enhance sample with additional context
    async fn enhance_sample(&self, mut sample: TrainingSample) -> Result<TrainingSample>;

    /// Get processor type for identification
    fn processor_type(&self) -> &str;
}

/// Code completion processor
#[derive(Debug)]
pub struct CodeCompletionProcessor {
    min_completion_length: usize,
    max_context_length: usize,
    overlap_ratio: f32,
}

impl CodeCompletionProcessor {
    pub fn new(min_length: usize, max_context: usize, overlap: f32) -> Self {
        Self {
            min_completion_length: min_length,
            max_context_length: max_context,
            overlap_ratio: overlap,
        }
    }

    /// Extract completion samples from Rust code
    fn extract_completions(&self, content: &str, file_path: &Path) -> Result<Vec<TrainingSample>> {
        let lines: Vec<&str> = content.lines().collect();
        let mut samples = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            if line.trim().is_empty()
                || line.trim().starts_with("//")
                || line.trim().starts_with("/*")
            {
                continue;
            }

            // Extract completion from this line as context
            let context_start = line_idx.saturating_sub(self.max_context_length / 10);
            let context: String = lines[context_start..line_idx + 1].join("\n");

            if context.len() >= 50 {
                // Minimum context size
                // Find the next non-empty, non-comment line as target
                let mut target_line = None;
                for j in (line_idx + 1)..lines.len().min(line_idx + 10) {
                    let next_line = lines[j].trim();
                    if !next_line.is_empty()
                        && !next_line.starts_with("//")
                        && !next_line.starts_with("/*")
                    {
                        target_line = Some((j, next_line));
                        break;
                    }
                }

                if let Some((target_idx, target_content)) = target_line {
                    let completion = if target_idx < lines.len() - 1 {
                        // Include next few lines if available
                        let end_idx = (target_idx + 3).min(lines.len());
                        lines[target_idx..end_idx].join("\n")
                    } else {
                        target_content.to_string()
                    };

                    if completion.len() >= self.min_completion_length {
                        samples.push(TrainingSample {
                            id: format!("completion_{}_{}", file_name(file_path), line_idx),
                            input: format!(
                                "{}\n{}",
                                context.trim(),
                                target_content.split_whitespace().next().unwrap_or("")
                            ),
                            output: completion.trim().to_string(),
                            task_type: TaskType::Completion,
                            metadata: SampleMetadata {
                                source_file: file_path.to_path_buf(),
                                language: "rust".to_string(),
                                complexity: calculate_lines_complexity(&[target_content]),
                                quality_score: 0.8,
                                tags: extract_rust_tags(target_content),
                                dependencies: vec![], // Would extract actual dependencies
                                context_window: Some(self.max_context_length),
                            },
                        });
                    }
                }
            }
        }

        Ok(samples)
    }
}

#[async_trait::async_trait]
impl DataProcessor for CodeCompletionProcessor {
    async fn process_file(&self, file_path: &Path, content: &str) -> Result<Vec<TrainingSample>> {
        self.extract_completions(content, file_path)
    }

    async fn filter_sample(&self, sample: &TrainingSample) -> Result<bool> {
        Ok(sample.output.len() >= self.min_completion_length
            && sample.input.len() <= self.max_context_length
            && sample.metadata.quality_score >= 0.5)
    }

    async fn enhance_sample(&self, sample: TrainingSample) -> Result<TrainingSample> {
        // Add Rust-specific enhancements
        Ok(sample)
    }

    fn processor_type(&self) -> &str {
        "code_completion"
    }
}

/// Error correction processor
#[derive(Debug)]
pub struct ErrorCorrectionProcessor {
    compiler_integration: Option<Box<dyn CompilerInterface>>,
}

#[async_trait::async_trait]
impl DataProcessor for ErrorCorrectionProcessor {
    async fn process_file(&self, file_path: &Path, content: &str) -> Result<Vec<TrainingSample>> {
        let mut samples = Vec::new();

        // Extract error patterns and their corrections from existing learning data
        // This would integrate with the existing learning system

        Ok(samples)
    }

    async fn filter_sample(&self, sample: &TrainingSample) -> Result<bool> {
        Ok(sample.task_type == TaskType::ErrorCorrection)
    }

    async fn enhance_sample(&self, sample: TrainingSample) -> Result<TrainingSample> {
        // Add error context information
        Ok(sample)
    }

    fn processor_type(&self) -> &str {
        "error_correction"
    }
}

/// Compiler interface trait for error processing
#[async_trait::async_trait]
pub trait CompilerInterface: Send + Sync {
    async fn get_diagnostics(&self, file_path: &Path) -> Result<Vec<CompilerDiagnostic>>;
}

/// Compiler diagnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerDiagnostic {
    pub message: String,
    pub severity: String,
    pub line: usize,
    pub column: usize,
    pub error_code: Option<String>,
}

/// Dataset statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetStatistics {
    pub total_samples: usize,
    pub samples_by_type: HashMap<TaskType, usize>,
    pub average_sample_length: usize,
    pub language_distribution: HashMap<String, usize>,
    pub quality_distribution: HashMap<String, usize>,
    pub file_coverage: f32,
}

/// Dataset augmentation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AugmentationConfig {
    pub variable_renaming: bool,
    pub comment_removal: bool,
    pub function_extraction: bool,
    pub pattern_variations: bool,
    pub noise_injection: f32,
}

/// Rust-specific data processing
pub mod rust_processing {
    use super::*;
    use syn::{visit::Visit, File, Item};

    /// Rust AST visitor for extracting semantic information
    pub struct RustCodeVisitor {
        functions: Vec<String>,
        structs: Vec<String>,
        traits: Vec<String>,
        complexity: u32,
    }

    impl RustCodeVisitor {
        pub fn new() -> Self {
            Self {
                functions: Vec::new(),
                structs: Vec::new(),
                traits: Vec::new(),
                complexity: 0,
            }
        }
    }

    impl<'ast> Visit<'ast> for RustCodeVisitor {
        fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
            self.functions.push(node.sig.ident.to_string());
            self.complexity += 1;

            // Visit nested items
            syn::visit::visit_item_fn(self, node);
        }

        fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
            self.structs.push(node.ident.to_string());
            syn::visit::visit_item_struct(self, node);
        }

        fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
            self.traits.push(node.ident.to_string());
            syn::visit::visit_item_trait(self, node);
        }
    }

    /// Extract Rust-specific metadata from code
    pub fn extract_rust_metadata(content: &str) -> Result<SampleMetadata> {
        let ast = syn::parse_file(content)?;
        let mut visitor = RustCodeVisitor::new();
        visitor.visit_file(&ast);

        Ok(SampleMetadata {
            source_file: PathBuf::new(), // Would be filled by caller
            language: "rust".to_string(),
            complexity: visitor.complexity,
            quality_score: calculate_rust_quality(content),
            tags: visitor.functions,
            dependencies: vec![], // Would extract actual dependencies
            context_window: None,
        })
    }
}

impl DatasetBuilder {
    /// Create a new dataset builder
    pub fn new() -> Self {
        Self {
            source_paths: Vec::new(),
            filters: DatasetFilters::default(),
            processors: vec![
                Box::new(CodeCompletionProcessor::new(10, 2048, 0.1)),
                Box::new(ErrorCorrectionProcessor {
                    compiler_integration: None,
                }),
            ],
            output_format: OutputFormat::JsonL,
        }
    }

    /// Add a source path for data collection
    pub fn add_source(&mut self, path: PathBuf) -> &mut Self {
        self.source_paths.push(path);
        self
    }

    /// Add a data processor to the pipeline
    pub fn add_processor(&mut self, processor: Box<dyn DataProcessor + Send + Sync>) -> &mut Self {
        self.processors.push(processor);
        self
    }

    /// Set dataset filters
    pub fn with_filters(&mut self, filters: DatasetFilters) -> &mut Self {
        self.filters = filters;
        self
    }

    /// Set output format
    pub fn with_output_format(&mut self, format: OutputFormat) -> &mut Self {
        self.output_format = format;
        self
    }

    /// Build the dataset from configured sources
    pub async fn build_dataset(&self, output_path: &Path) -> Result<DatasetStatistics> {
        let mut all_samples = Vec::new();
        let mut processed_files = 0;
        let mut total_files = 0;

        // Count total files first
        for source_path in &self.source_paths {
            if source_path.is_dir() {
                for entry in WalkDir::new(source_path).into_iter().filter_map(|e| e.ok()) {
                    if entry.file_type().is_file() && is_rust_file(entry.path()) {
                        total_files += 1;
                    }
                }
            } else if is_rust_file(source_path) {
                total_files += 1;
            }
        }

        log::info!("Processing {} Rust files for dataset creation", total_files);

        // Process each source path
        for source_path in &self.source_paths {
            if source_path.is_dir() {
                for entry in WalkDir::new(source_path).into_iter().filter_map(|e| e.ok()) {
                    if entry.file_type().is_file() && self.passes_filters(entry.path()) {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            if self.is_quality_content(&content) {
                                let samples = self.process_file(entry.path(), &content).await?;
                                all_samples.extend(samples);
                                processed_files += 1;

                                if processed_files % 100 == 0 {
                                    log::info!(
                                        "Processed {}/{} files",
                                        processed_files,
                                        total_files
                                    );
                                }
                            }
                        }
                    }
                }
            } else if self.passes_filters(source_path) {
                if let Ok(content) = fs::read_to_string(source_path) {
                    let samples = self.process_file(source_path, &content).await?;
                    all_samples.extend(samples);
                    processed_files += 1;
                }
            }
        }

        log::info!(
            "Generated {} training samples from {} files",
            all_samples.len(),
            processed_files
        );

        // Write dataset to output
        self.save_dataset(&all_samples, output_path).await?;

        // Calculate and return statistics
        let stats = self.calculate_statistics(&all_samples, processed_files, total_files);
        Ok(stats)
    }

    /// Process a single file through the processor pipeline
    async fn process_file(&self, file_path: &Path, content: &str) -> Result<Vec<TrainingSample>> {
        let mut all_samples = Vec::new();

        for processor in &self.processors {
            let mut samples = processor.process_file(file_path, content).await?;

            // Filter and enhance samples
            for sample in samples {
                if processor.filter_sample(&sample).await? {
                    let enhanced_sample = processor.enhance_sample(sample).await?;
                    all_samples.push(enhanced_sample);
                }
            }
        }

        Ok(all_samples)
    }

    /// Check if file passes configured filters
    fn passes_filters(&self, file_path: &Path) -> bool {
        // Check file extension
        let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        if !self.filters.allowed_extensions.contains(extension) {
            return false;
        }

        // Check file size
        if let Ok(metadata) = file_path.metadata() {
            let size = metadata.len() as usize;
            if size < self.filters.min_file_size || size > self.filters.max_file_size {
                return false;
            }
        }

        // Check exclude patterns
        if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
            for pattern in &self.filters.exclude_patterns {
                if file_name.contains(pattern) {
                    return false;
                }
            }
        }

        true
    }

    /// Check if content meets quality criteria
    fn is_quality_content(&self, content: &str) -> bool {
        // Basic quality checks
        let lines: Vec<&str> = content.lines().collect();
        let code_lines = lines
            .iter()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("/*")
            })
            .count();

        // Must have some actual code
        if code_lines < 5 {
            return false;
        }

        // Complex enough for training
        let complexity = calculate_lines_complexity(&lines);
        complexity >= self.filters.min_complexity && complexity <= self.filters.max_complexity
    }

    /// Save dataset to specified format
    async fn save_dataset(&self, samples: &[TrainingSample], output_path: &Path) -> Result<()> {
        match self.output_format {
            OutputFormat::JsonL => self.save_jsonl(samples, output_path).await,
            OutputFormat::Json => self.save_json(samples, output_path).await,
            OutputFormat::Text => self.save_text(samples, output_path).await,
            OutputFormat::Parquet => self.save_parquet(samples, output_path).await,
        }
    }

    /// Save as JSONL format
    async fn save_jsonl(&self, samples: &[TrainingSample], output_path: &Path) -> Result<()> {
        let mut content = String::new();
        for sample in samples {
            let json = serde_json::to_string(sample)?;
            content.push_str(&json);
            content.push('\n');
        }

        fs::write(output_path, content)?;
        Ok(())
    }

    /// Save as JSON format
    async fn save_json(&self, samples: &[TrainingSample], output_path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(samples)?;
        fs::write(output_path, json)?;
        Ok(())
    }

    /// Save as plain text format
    async fn save_text(&self, samples: &[TrainingSample], output_path: &Path) -> Result<()> {
        let mut content = String::new();
        for sample in samples {
            content.push_str(&format!("Input: {}\n", sample.input));
            content.push_str(&format!("Output: {}\n", sample.output));
            content.push_str("---\n");
        }

        fs::write(output_path, content)?;
        Ok(())
    }

    /// Save as Parquet format (placeholder)
    async fn save_parquet(&self, _samples: &[TrainingSample], _output_path: &Path) -> Result<()> {
        // Implementation would use parquet crate
        Err(anyhow::anyhow!("Parquet format not implemented yet"))
    }

    /// Calculate dataset statistics
    fn calculate_statistics(
        &self,
        samples: &[TrainingSample],
        processed_files: usize,
        total_files: usize,
    ) -> DatasetStatistics {
        let mut samples_by_type = HashMap::new();
        let mut language_distribution = HashMap::new();
        let mut quality_distribution = HashMap::new();

        let total_chars: usize = samples.iter().map(|s| s.input.len() + s.output.len()).sum();
        let average_sample_length = if !samples.is_empty() {
            total_chars / samples.len()
        } else {
            0
        };

        for sample in samples {
            *samples_by_type.entry(sample.task_type.clone()).or_insert(0) += 1;
            *language_distribution
                .entry(sample.metadata.language.clone())
                .or_insert(0) += 1;

            let quality_bucket = format!("{:.1}", sample.metadata.quality_score);
            *quality_distribution.entry(quality_bucket).or_insert(0) += 1;
        }

        DatasetStatistics {
            total_samples: samples.len(),
            samples_by_type,
            average_sample_length,
            language_distribution,
            quality_distribution,
            file_coverage: if total_files > 0 {
                processed_files as f32 / total_files as f32
            } else {
                0.0
            },
        }
    }

    /// Validate dataset quality
    pub async fn validate_dataset(&self, samples: &[TrainingSample]) -> Result<DatasetValidation> {
        let mut issues = Vec::new();
        let mut duplicates = HashSet::new();

        for (i, sample) in samples.iter().enumerate() {
            // Check for duplicates
            let key = format!("{}|{}", sample.input, sample.output);
            if duplicates.contains(&key) {
                issues.push(format!("Duplicate sample found at index {}", i));
            } else {
                duplicates.insert(key);
            }

            // Check input/output lengths
            if sample.input.trim().is_empty() {
                issues.push(format!("Empty input at index {}", i));
            }
            if sample.output.trim().is_empty() {
                issues.push(format!("Empty output at index {}", i));
            }

            // Check quality score
            if sample.metadata.quality_score < 0.0 || sample.metadata.quality_score > 1.0 {
                issues.push(format!("Invalid quality score at index {}", i));
            }
        }

        let diversity_score = self.calculate_diversity_score(samples);
        let balance_score = self.calculate_balance_score(samples);

        Ok(DatasetValidation {
            passed: issues.is_empty(),
            issues,
            quality_score: if samples.is_empty() {
                0.0
            } else {
                samples
                    .iter()
                    .map(|s| s.metadata.quality_score)
                    .sum::<f32>()
                    / samples.len() as f32
            },
            diversity_score,
            balance_score,
            recommendations: self.generate_recommendations(samples),
        })
    }

    fn calculate_diversity_score(&self, samples: &[TrainingSample]) -> f32 {
        let mut unique_inputs = HashSet::new();
        for sample in samples {
            unique_inputs.insert(&sample.input);
        }
        unique_inputs.len() as f32 / samples.len() as f32
    }

    fn calculate_balance_score(&self, samples: &[TrainingSample]) -> f32 {
        let mut type_counts = HashMap::new();
        for sample in samples {
            *type_counts.entry(sample.task_type.clone()).or_insert(0) += 1;
        }

        if type_counts.is_empty() {
            return 0.0;
        }

        let total = samples.len() as f32;
        let ideal_count = total / type_counts.len() as f32;
        let max_deviation = type_counts
            .values()
            .map(|&count| (count as f32 - ideal_count).abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        1.0 - (max_deviation / total).min(1.0)
    }

    fn generate_recommendations(&self, samples: &[TrainingSample]) -> Vec<String> {
        let mut recommendations = Vec::new();

        let diversity = self.calculate_diversity_score(samples);
        if diversity < 0.8 {
            recommendations.push("Consider adding more diverse training samples".to_string());
        }

        let balance = self.calculate_balance_score(samples);
        if balance < 0.7 {
            recommendations.push("Training data is imbalanced across task types".to_string());
        }

        if samples.len() < 1000 {
            recommendations.push("Consider collecting more training samples".to_string());
        }

        recommendations
    }
}

/// Dataset validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetValidation {
    pub passed: bool,
    pub issues: Vec<String>,
    pub quality_score: f32,
    pub diversity_score: f32,
    pub balance_score: f32,
    pub recommendations: Vec<String>,
}

impl Default for DatasetFilters {
    fn default() -> Self {
        let mut extensions = HashSet::new();
        extensions.insert("rs".to_string());

        Self {
            min_file_size: 100,
            max_file_size: 1_000_000, // 1MB
            allowed_extensions: extensions,
            exclude_patterns: vec![
                "target".to_string(),
                ".git".to_string(),
                "test".to_string(),
                "bench".to_string(),
            ],
            quality_threshold: 0.6,
            min_complexity: 2,
            max_complexity: 50,
            max_nesting_depth: 5,
        }
    }
}

/// Utility functions
fn is_rust_file(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("rs")
}

fn file_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

fn extract_rust_tags(content: &str) -> Vec<String> {
    let mut tags = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("fn ") && line.contains('(') {
            if let Some(end) = line.find('(') {
                let fn_name = line[3..end].trim();
                if fn_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    tags.push(fn_name.to_string());
                }
            }
        } else if line.starts_with("struct ")
            || line.starts_with("enum ")
            || line.starts_with("trait ")
        {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[1].split('<').next().unwrap_or(parts[1]);
                if name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    tags.push(name.to_string());
                }
            }
        }
    }

    tags
}

fn calculate_lines_complexity(lines: &[&str]) -> u32 {
    lines
        .iter()
        .map(|line| calculate_line_complexity(line))
        .sum()
}

fn calculate_line_complexity(line: &str) -> u32 {
    let mut complexity = 1; // Base complexity

    if line.contains("if ") || line.contains("else if ") || line.contains("match ") {
        complexity += 1;
    }
    if line.contains("while ") || line.contains("for ") || line.contains("loop ") {
        complexity += 1;
    }
    if line.contains("&&") || line.contains("||") {
        complexity += line.matches("&&").count() as u32;
        complexity += line.matches("||").count() as u32;
    }

    complexity
}

fn calculate_rust_quality(content: &str) -> f32 {
    let lines: Vec<&str> = content.lines().collect();
    let mut score = 0.8; // Base score

    // Check for proper formatting
    if content.contains("cargo fmt") || content.contains("rustfmt") {
        score += 0.1;
    }

    // Check for documentation
    let doc_lines = lines
        .iter()
        .filter(|line| line.trim().starts_with("///"))
        .count();
    if doc_lines > lines.len() / 10 {
        score += 0.1;
    }

    // Check for tests
    if content.contains("#[cfg(test)]") || content.contains("#[test]") {
        score += 0.05;
    }

    // Penalize for TODOs and unwraps
    let todo_count = content.matches("TODO").count() + content.matches("FIXME").count();
    let unwrap_count = content.matches("unwrap()").count();

    score - (todo_count as f32 * 0.01).min(0.2) - (unwrap_count as f32 * 0.005).min(0.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_processor() {
        let processor = CodeCompletionProcessor::new(5, 1000, 0.1);
        let content = r#"
        fn main() {
            println!("Hello");
            let x = 5;
            let y = x + 1;
        }
        "#;

        let samples = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { processor.process_file(Path::new("test.rs"), content).await })
            .unwrap();

        assert!(!samples.is_empty());
    }

    #[test]
    fn test_quality_scoring() {
        let good_code = r#"
        /// Calculate the sum of two numbers
        pub fn add(a: i32, b: i32) -> i32 {
            a + b
        }

        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn test_add() {
                assert_eq!(add(2, 3), 5);
            }
        }
        "#;

        let score = calculate_rust_quality(good_code);
        assert!(score > 0.8);
    }
}

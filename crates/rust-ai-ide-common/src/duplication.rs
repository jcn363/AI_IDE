//! Duplication detection and prevention utilities
//!
//! This module provides sophisticated tools for detecting and preventing
//! code duplication across the codebase. It includes structural similarity
//! analysis, trait implementation detection, and fuzzy matching capabilities.

use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Statistics about detected duplications
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DuplicationStats {
    pub total_files: usize,
    pub duplicated_functions: usize,
    pub similar_structs: usize,
    pub repeated_patterns: usize,
    pub trait_duplicates: usize,
    pub total_duplicates: usize,
}

/// A duplication detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationResult {
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
    pub kind: DuplicationKind,
    pub confidence: f64,
    pub similar_to: String,
    pub code_snippet: String,
}

/// Types of duplications that can be detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DuplicationKind {
    Function,
    Struct,
    TraitImpl,
    CodePattern(String),
    TypeDefinition,
}

/// Fuzzy matching result for structural similarity
#[derive(Debug, Clone)]
pub struct SimilarityMatch {
    pub similarity_score: f64,
    pub matched_content: String,
    pub source_location: String,
}

/// Detect duplications across a set of source files
pub fn detect_duplications(files: &HashMap<String, String>) -> Result<DuplicationStats, String> {
    let mut stats = DuplicationStats::default();
    stats.total_files = files.len();

    let mut duplicated_items = Vec::new();

    // Extract all functions, structs, and traits
    let mut all_functions = HashMap::new();
    let mut all_structs = HashMap::new();
    let mut all_traits = HashMap::new();

    for (path, content) in files {
        // Extract functions
        for (name, signature) in extract_functions(content)? {
            all_functions
                .entry(signature)
                .or_insert_with(Vec::new)
                .push((name, path.clone()));
        }

        // Extract structs
        for (name, definition) in extract_structs(content)? {
            all_structs
                .entry(definition)
                .or_insert_with(Vec::new)
                .push((name, path.clone()));
        }

        // Extract trait implementations
        for (name, impl_type) in extract_trait_implementations(content)? {
            all_traits
                .entry(impl_type)
                .or_insert_with(Vec::new)
                .push((name, path.clone()));
        }
    }

    // Find duplications in functions
    for (signature, _locations) in all_functions.iter().filter(|(_, locs)| locs.len() > 1) {
        duplicated_items.push(signature.clone());
        stats.duplicated_functions += 1;
    }

    // Find duplications in structs
    for (definition, _locations) in all_structs.iter().filter(|(_, locs)| locs.len() > 1) {
        duplicated_items.push(definition.clone());
        stats.similar_structs += 1;
    }

    // Find trait implementation duplications
    for (impl_type, _locations) in all_traits.iter().filter(|(_, locs)| locs.len() > 1) {
        duplicated_items.push(impl_type.clone());
        stats.trait_duplicates += 1;
    }

    // Detect repeated code patterns using fuzzy matching
    let mut code_fragments = Vec::new();
    for (path, content) in files {
        code_fragments.extend(extract_code_fragments(content, path));
    }

    stats.repeated_patterns = detect_repeated_patterns(&code_fragments, 0.8);
    stats.total_duplicates = stats.duplicated_functions
        + stats.similar_structs
        + stats.trait_duplicates
        + stats.repeated_patterns;

    Ok(stats)
}

/// Extract function signatures from code for duplication detection
pub fn extract_functions(code: &str) -> Result<Vec<(String, String)>, String> {
    static FN_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"fn\s+(\w+)\s*\([^)]*\)\s*(->\s*[^;\{]*)?\s*\{").expect("Invalid regex pattern")
    });

    let mut functions = Vec::new();
    for cap in FN_REGEX.captures_iter(code) {
        let name = cap.get(1).map_or("", |m| m.as_str());
        let signature = cap.get(0).map_or("", |m| m.as_str());
        functions.push((name.to_string(), signature.to_string()));
    }

    Ok(functions)
}

/// Extract struct definitions from code
pub fn extract_structs(code: &str) -> Result<Vec<(String, String)>, String> {
    static STRUCT_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"struct\s+(\w+)\s*\{[^}]*\}").expect("Invalid regex pattern"));

    let mut structs = Vec::new();
    for cap in STRUCT_REGEX.captures_iter(code) {
        let name = cap.get(1).map_or("", |m| m.as_str());
        let definition = cap.get(0).map_or("", |m| m.as_str());
        structs.push((name.to_string(), definition.to_string()));
    }

    Ok(structs)
}

/// Extract trait implementations
pub fn extract_trait_implementations(code: &str) -> Result<Vec<(String, String)>, String> {
    static IMPL_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"impl\s+<[^>]*>\s*(\w+)\s+for\s+(\w+)").expect("Invalid regex pattern")
    });

    let mut impls = Vec::new();
    for cap in IMPL_REGEX.captures_iter(code) {
        let trait_name = cap.get(1).map_or("", |m| m.as_str());
        let impl_type = cap.get(2).map_or("", |m| m.as_str());
        let impl_def = cap.get(0).map_or("", |m| m.as_str());
        impls.push((
            impl_def.to_string(),
            format!("{} for {}", trait_name, impl_type),
        ));
    }

    Ok(impls)
}

/// Extract code fragments for fuzzy matching
pub fn extract_code_fragments(code: &str, file_path: &str) -> Vec<CodeFragment> {
    let mut fragments = Vec::new();

    // Split code into meaningful fragments (blocks of 3-15 lines)
    let lines: Vec<&str> = code.lines().collect();

    for i in (0..lines.len()).step_by(3) {
        let end = (i + 12).min(lines.len()); // Take up to 12 lines per fragment
        if end - i >= 3 {
            // Skip fragments smaller than 3 lines
            let fragment = lines[i..end].join("\n").trim().to_string();
            if !fragment.is_empty() {
                fragments.push(CodeFragment {
                    content: fragment,
                    start_line: i + 1,
                    file_path: file_path.to_string(),
                });
            }
        }
    }

    fragments
}

/// A code fragment with metadata for comparison
#[derive(Debug, Clone)]
pub struct CodeFragment {
    pub content: String,
    pub start_line: usize,
    pub file_path: String,
}

/// Detect repeated patterns using fuzzy matching
pub fn detect_repeated_patterns(fragments: &[CodeFragment], threshold: f64) -> usize {
    let mut repeated_count = 0;
    let mut seen = HashSet::new();

    for (i, fragment) in fragments.iter().enumerate() {
        for (j, other_fragment) in fragments.iter().enumerate() {
            if i == j {
                continue;
            }

            // Skip if already compared
            let key = format!(
                "{}::{}::{}::{}",
                i, &fragment.content, j, &other_fragment.content
            );
            if seen.contains(&key) {
                continue;
            }
            seen.insert(key);

            // Skip fragments from the same file (close proximity)
            if fragment.file_path == other_fragment.file_path
                && (fragment.start_line as isize - other_fragment.start_line as isize).abs() < 50
            {
                continue;
            }

            let similarity = calculate_similarity(&fragment.content, &other_fragment.content);
            if similarity >= threshold {
                repeated_count += 1;
            }
        }
    }

    repeated_count
}

/// Calculate similarity between two code strings using Jaccard similarity
pub fn calculate_similarity(code1: &str, code2: &str) -> f64 {
    let tokens1: HashSet<&str> = tokenize_code(code1);
    let tokens2: HashSet<&str> = tokenize_code(code2);

    let intersection: HashSet<_> = tokens1.intersection(&tokens2).collect();
    let union: HashSet<_> = tokens1.union(&tokens2).collect();

    if union.is_empty() {
        0.0
    } else {
        intersection.len() as f64 / union.len() as f64
    }
}

/// Tokenize code for similarity comparison
pub fn tokenize_code(code: &str) -> HashSet<&str> {
    // Split on common delimiters and normalize whitespace
    let tokens: HashSet<&str> = code
        .split(
            &[
                ' ', '\t', '\n', '(', ')', '{', '}', '[', ']', ';', ':', ',', '=',
            ][..],
        )
        .filter(|s| !s.is_empty() && s.chars().any(char::is_alphanumeric))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    tokens
}

/// Utility to check for potential duplications before adding new code
pub fn check_potential_duplication(
    new_code: &str,
    existing_files: &HashMap<String, String>,
) -> Vec<SimilarityMatch> {
    let mut matches = Vec::new();
    let new_fragments = extract_code_fragments(new_code, "new_code");

    for (file_path, existing_code) in existing_files {
        let existing_fragments = extract_code_fragments(existing_code, file_path);

        for new_frag in &new_fragments {
            for existing_frag in &existing_fragments {
                let similarity = calculate_similarity(&new_frag.content, &existing_frag.content);
                if similarity >= 0.7 {
                    // High similarity threshold for warnings
                    matches.push(SimilarityMatch {
                        similarity_score: similarity,
                        matched_content: existing_frag.content.clone(),
                        source_location: format!(
                            "{}:{}",
                            existing_frag.file_path, existing_frag.start_line
                        ),
                    });
                }
            }
        }
    }

    // Sort by similarity score (highest first)
    matches.sort_by(|a, b| {
        b.similarity_score
            .partial_cmp(&a.similarity_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    matches
}

/// Create a duplication prevention template for new modules
pub fn create_duplication_prevention_template(module_name: &str) -> String {
    format!(
        r#"//! {} module - Duplication Prevention Guidelines
//!
//! This module follows the unified architecture patterns established in rust-ai-ide-core.
//! Before adding new structures or functions:
//! 1. Check for existing similar patterns in rust-ai-ide-common
//! 2. Use existing traits from traits.rs and utilities from utils.rs
//! 3. Follow the naming conventions and implementation patterns
//! 4. Run duplication detection before committing
//!
//! Related patterns to consider:
//! - Implement Loggable trait for consistent logging
//! - Use Cache trait from caching.rs for persistent data
//! - Leverage error types from errors.rs for consistent error handling
//!
//! # Prevent Duplication Checklist
//! - [ ] No similar structs exist in other modules
//! - [ ] No similar functions exist in utils
//! - [ ] Uses existing trait implementations where possible
//! - [ ] Follows established error handling patterns
//! - [ ] Has been checked with duplication detection tools

/// {} module statistics for duplication monitoring
pub struct ModuleDuplicationStats {{
    pub module_name: String,
    pub function_count: usize,
    pub struct_count: usize,
    pub trait_impl_count: usize,
    pub last_checked: chrono::DateTime<chrono::Utc>,
}}

impl {} {{
    /// Check module for potential duplications
    pub fn check_duplications(&self) -> Result<Vec<DuplicationResult>, String> {{
        // Implementation would call rust_ai_ide_common::duplication::detect_duplications
        unimplemented!("Duplication detection not implemented yet")
    }}
}}
"#,
        module_name, module_name, module_name
    )
}

/// Template for new function implementations with duplication prevention
pub fn create_safe_function_template(function_name: &str, parameters: &[&str]) -> String {
    let param_list = parameters.join(", ");
    let param_count = parameters.len();

    format!(
        r#"/// Safe function template with duplication prevention
///
/// This function follows established patterns from rust-ai-ide-common.
/// Before implementing, check for similar functions in:
/// - fs_utils.rs for file operations
/// - utils.rs for general utilities
/// - path_utils.rs for path manipulations
///
/// # Parameters
/// {} - Description of parameters
///
/// # Returns
/// Describe the return type and what it represents
///
/// # Safety
/// This function is designed with duplication prevention in mind.
/// If similar functionality exists, consider reusing existing implementations.
///
/// # Example
/// ```rust
/// // TODO: Add example usage
/// ```
pub fn {}( {} ) -> Result<(), IdeError> {{
    // TODO: Implement function logic
    // 1. Check for existing similar implementations
    // 2. Use proper error handling from errors.rs
    // 3. Follow logging patterns if needed
    // 4. Consider performance implications

    unimplemented!("Implementation pending duplication check")
}}"#,
        format!("/// * `{}` - Description", param_count),
        function_name,
        param_list
    )
}

/// Static verification function for compilation-time checks
pub fn verify_duplication_free(file_content: &str) -> Result<(), Vec<String>> {
    let mut warnings = Vec::new();

    // Check for common duplication patterns
    let patterns = [
        ("Multiple Debug impls", r"impl Debug for"),
        ("Repeated error handling", r"\.map_err\(\|.*_error"),
        ("Manual string manipulation", r"format!\(.*\)"),
        ("Custom Option handling", r"match.*Some\("),
    ];

    for (pattern_name, pattern) in &patterns {
        let re = Regex::new(pattern)
            .map_err(|_| vec![format!("Invalid regex pattern: {}", pattern_name)])?;
        let count = re.find_iter(file_content).count();
        if count > 3 {
            // Threshold for potential duplication
            warnings.push(format!(
                "Potential duplication: {} instances of '{}' found",
                count, pattern_name
            ));
        }
    }

    if warnings.is_empty() {
        Ok(())
    } else {
        Err(warnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_similarity_identical() {
        let code = "fn test() { println!(\"hello\"); }";
        let similarity = calculate_similarity(code, code);
        assert!((similarity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_similarity_different() {
        let code1 = "fn test() { println!(\"hello\"); }";
        let code2 = "struct Test; impl Test { fn new() -> Self { Test } }";
        let similarity = calculate_similarity(code1, code2);
        assert!(similarity < 0.5);
    }

    #[test]
    fn test_extract_functions() {
        let code = r#"
            fn main() { println!("hello"); }
            fn add(a: i32, b: i32) -> i32 { a + b }
        "#;

        let functions = extract_functions(code).unwrap();
        assert_eq!(functions.len(), 2);
        assert!(functions.iter().any(|(name, _)| name == "main"));
        assert!(functions.iter().any(|(name, _)| name == "add"));
    }

    #[test]
    fn test_verify_duplication_free_clean() {
        let clean_code = r#"
            struct Simple { value: i32 }
            impl Simple {
                fn get_value(&self) -> i32 { self.value }
            }
        "#;

        assert!(verify_duplication_free(clean_code).is_ok());
    }
}

// Make public exports available at crate level
pub mod exports {
    pub use super::{
        calculate_similarity, check_potential_duplication, create_duplication_prevention_template,
        create_safe_function_template, detect_duplications, verify_duplication_free,
        DuplicationKind, DuplicationResult, DuplicationStats, SimilarityMatch,
    };
}

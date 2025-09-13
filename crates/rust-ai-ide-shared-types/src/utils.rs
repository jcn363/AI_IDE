//! Shared utilities and helpers for shared types functionality
//!
//! This module contains utility functions, helper traits, and common
//! functionality used across the shared types crate.

use std::collections::HashMap;
use std::path::Path;

use crate::errors::TypeGenerationError;
use crate::types::*;

/// Utility functions for file operations
pub mod file_utils {
    use super::*;

    /// Read a Rust source file and return its contents
    pub fn read_rust_file(path: &Path) -> Result<String, TypeGenerationError> {
        std::fs::read_to_string(path).map_err(|e| TypeGenerationError::IoError(e))
    }

    /// Write generated content to a file
    pub fn write_generated_file(path: &Path, content: &str) -> Result<(), TypeGenerationError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| TypeGenerationError::IoError(e))?;
        }

        std::fs::write(path, content).map_err(|e| TypeGenerationError::IoError(e))
    }

    /// Check if a file is a valid Rust source file
    pub fn is_rust_file(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "rs")
            .unwrap_or(false)
    }

    /// Generate a timestamped filename
    pub fn generate_timestamped_filename(base_name: &str, extension: &str) -> String {
        use chrono::Utc;
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        format!("{}_{}.{}", base_name, timestamp, extension)
    }
}

/// Utility functions for string manipulation
pub mod string_utils {
    /// Convert a string to PascalCase
    pub fn to_pascal_case(s: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = true;

        for ch in s.chars() {
            if ch == '_' || ch == '-' {
                capitalize_next = true;
            } else if capitalize_next {
                result.extend(ch.to_uppercase());
                capitalize_next = false;
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Convert a string to camelCase
    pub fn to_camel_case(s: &str) -> String {
        let pascal = to_pascal_case(s);
        if let Some(first_char) = pascal.chars().next() {
            let rest: String = pascal.chars().skip(1).collect();
            format!("{}{}", first_char.to_lowercase(), rest)
        } else {
            pascal
        }
    }

    /// Convert a string to snake_case
    pub fn to_snake_case(s: &str) -> String {
        let mut result = String::new();

        for (i, ch) in s.char_indices() {
            if ch.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.extend(ch.to_lowercase());
        }

        result
    }

    /// Escape special characters for TypeScript strings
    pub fn escape_typescript_string(s: &str) -> String {
        s.replace("\\", "\\\\")
            .replace("\"", "\\\"")
            .replace("'", "\\'")
            .replace("\n", "\\n")
            .replace("\r", "\\r")
            .replace("\t", "\\t")
    }

    /// Check if a string is a valid identifier for a given platform
    pub fn is_valid_identifier(s: &str, platform: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        match platform {
            "typescript" | "javascript" => {
                // TypeScript/JavaScript identifiers
                let first_char = s.chars().next().unwrap();
                first_char.is_alphabetic() || first_char == '_' || first_char == '$'
            }
            "rust" => {
                // Rust identifiers
                let first_char = s.chars().next().unwrap();
                first_char.is_alphabetic() || first_char == '_'
            }
            _ => {
                // Default: alphanumeric with underscores
                let first_char = s.chars().next().unwrap();
                first_char.is_alphabetic() || first_char == '_'
            }
        }
    }
}

/// Collection utilities for type management
pub mod collection_utils {
    use super::*;

    /// Filter types by visibility
    pub fn filter_by_visibility(types: &[ParsedType], min_visibility: Visibility) -> Vec<ParsedType> {
        types
            .iter()
            .filter(|ty| visibility_level(&ty.visibility) >= visibility_level(&min_visibility))
            .cloned()
            .collect()
    }

    /// Group types by their kind
    pub fn group_by_kind(types: &[ParsedType]) -> HashMap<TypeKind, Vec<ParsedType>> {
        let mut groups = HashMap::new();

        for ty in types {
            groups
                .entry(ty.kind.clone())
                .or_insert_with(Vec::new)
                .push(ty.clone());
        }

        groups
    }

    /// Get numeric level for visibility (higher = more visible)
    fn visibility_level(vis: &Visibility) -> i32 {
        match vis {
            Visibility::Public => 3,
            Visibility::Crate => 2,
            Visibility::Module => 1,
            Visibility::Private => 0,
        }
    }

    /// Find types that depend on a given type
    pub fn find_dependencies(types: &[ParsedType], target_type: &str) -> Vec<String> {
        let mut deps = Vec::new();

        for ty in types {
            for field in &ty.fields {
                if field.ty.contains(target_type) {
                    deps.push(ty.name.clone());
                    break;
                }
            }

            for variant in &ty.variants {
                for field in &variant.fields {
                    if field.ty.contains(target_type) {
                        deps.push(ty.name.clone());
                        break;
                    }
                }
            }
        }

        deps
    }

    /// Topologically sort types based on dependencies
    pub fn topological_sort(types: &[ParsedType]) -> Vec<ParsedType> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();

        // Create a map for quick lookup
        let type_map: HashMap<String, &ParsedType> = types.iter().map(|ty| (ty.name.clone(), ty)).collect();

        fn visit(
            type_name: &str,
            type_map: &HashMap<String, &ParsedType>,
            result: &mut Vec<ParsedType>,
            visited: &mut std::collections::HashSet<String>,
            visiting: &mut std::collections::HashSet<String>,
        ) {
            if visited.contains(type_name) {
                return;
            }

            if visiting.contains(type_name) {
                // Cycle detected - for now, just skip
                return;
            }

            visiting.insert(type_name.to_string());

            if let Some(ty) = type_map.get(type_name) {
                for dep in &ty.dependencies {
                    // Extract base type name from complex types
                    let base_type = dep.split('<').next().unwrap_or(dep);
                    if type_map.contains_key(base_type) {
                        visit(base_type, type_map, result, visited, visiting);
                    }
                }
            }

            visiting.remove(type_name);
            visited.insert(type_name.to_string());

            if let Some(ty) = type_map.get(type_name) {
                result.push((*ty).clone());
            }
        }

        for ty in types {
            if !visited.contains(&ty.name) {
                visit(
                    &ty.name,
                    &type_map,
                    &mut result,
                    &mut visited,
                    &mut visiting,
                );
            }
        }

        result
    }
}

/// Validation utilities
pub mod validation_utils {
    use super::*;

    /// Validate that all generated types have unique names
    pub fn check_unique_names(types: &[ParsedType]) -> Result<(), TypeGenerationError> {
        let mut seen_names = std::collections::HashSet::new();

        for ty in types {
            if seen_names.contains(&ty.name) {
                return Err(TypeGenerationError::AnalysisError(format!(
                    "Duplicate type name found: {}",
                    ty.name
                )));
            }
            seen_names.insert(ty.name.clone());
        }

        Ok(())
    }

    /// Validate that all referenced types exist
    pub fn check_type_references(types: &[ParsedType]) -> Vec<String> {
        let mut warnings = Vec::new();
        let type_names: std::collections::HashSet<String> = types.iter().map(|ty| ty.name.clone()).collect();

        for ty in types {
            for field in &ty.fields {
                let referenced_type = extract_base_type(&field.ty);
                if !is_builtin_type(&referenced_type) && !type_names.contains(referenced_type) {
                    warnings.push(format!(
                        "Type '{}' references unknown type '{}' in field '{}'",
                        ty.name, referenced_type, field.name
                    ));
                }
            }
        }

        warnings
    }

    /// Extract base type from complex types like Option<T>, Vec<T>, etc.
    fn extract_base_type(type_str: &str) -> &str {
        if let Some(start) = type_str.find('<') {
            if let Some(end) = type_str.rfind('>') {
                return &type_str[start + 1..end];
            }
        }
        type_str
    }

    /// Check if a type is a built-in language type
    fn is_builtin_type(type_name: &str) -> bool {
        let builtins = [
            "String", "str", "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "usize", "isize",
        ];

        builtins.contains(&type_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pascal_case_conversion() {
        assert_eq!(string_utils::to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(string_utils::to_pascal_case("test-case"), "TestCase");
        assert_eq!(string_utils::to_pascal_case("simple"), "Simple");
    }

    #[test]
    fn test_camel_case_conversion() {
        assert_eq!(string_utils::to_camel_case("hello_world"), "helloWorld");
        assert_eq!(string_utils::to_camel_case("test_case"), "testCase");
        assert_eq!(string_utils::to_camel_case("simple"), "simple");
    }

    #[test]
    fn test_snake_case_conversion() {
        assert_eq!(string_utils::to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(string_utils::to_snake_case("TestCase"), "test_case");
        assert_eq!(string_utils::to_snake_case("simple"), "simple");
    }

    #[test]
    fn test_topological_sort() {
        let types = vec![
            ParsedType {
                name:             "User".to_string(),
                kind:             TypeKind::Struct,
                documentation:    None,
                visibility:       Visibility::Public,
                generics:         vec![],
                fields:           vec![Field {
                    name:          "address".to_string(),
                    ty:            "Address".to_string(),
                    documentation: None,
                    visibility:    Visibility::Public,
                    is_mutable:    false,
                    attributes:    vec![],
                }],
                variants:         vec![],
                associated_items: vec![],
                attributes:       vec![],
                source_location:  SourceLocation {
                    file:        "test.rs".to_string(),
                    line:        1,
                    column:      1,
                    module_path: vec![],
                },
                dependencies:     vec!["Address".to_string()],
                metadata:         TypeMetadata::default(),
            },
            ParsedType {
                name:             "Address".to_string(),
                kind:             TypeKind::Struct,
                documentation:    None,
                visibility:       Visibility::Public,
                generics:         vec![],
                fields:           vec![],
                variants:         vec![],
                associated_items: vec![],
                attributes:       vec![],
                source_location:  SourceLocation {
                    file:        "test.rs".to_string(),
                    line:        10,
                    column:      1,
                    module_path: vec![],
                },
                dependencies:     vec![],
                metadata:         TypeMetadata::default(),
            },
        ];

        let sorted = collection_utils::topological_sort(&types);
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].name, "Address"); // Should come first (no dependencies)
        assert_eq!(sorted[1].name, "User"); // Depends on Address
    }

    #[test]
    fn test_unique_names_validation() {
        let types = vec![
            ParsedType {
                name:             "TestType".to_string(),
                kind:             TypeKind::Struct,
                documentation:    None,
                visibility:       Visibility::Public,
                generics:         vec![],
                fields:           vec![],
                variants:         vec![],
                associated_items: vec![],
                attributes:       vec![],
                source_location:  SourceLocation {
                    file:        "test.rs".to_string(),
                    line:        1,
                    column:      1,
                    module_path: vec![],
                },
                dependencies:     vec![],
                metadata:         TypeMetadata::default(),
            },
            ParsedType {
                name:             "TestType".to_string(), // Duplicate name
                kind:             TypeKind::Enum,
                documentation:    None,
                visibility:       Visibility::Public,
                generics:         vec![],
                fields:           vec![],
                variants:         vec![],
                associated_items: vec![],
                attributes:       vec![],
                source_location:  SourceLocation {
                    file:        "test.rs".to_string(),
                    line:        10,
                    column:      1,
                    module_path: vec![],
                },
                dependencies:     vec![],
                metadata:         TypeMetadata::default(),
            },
        ];

        let result = validation_utils::check_unique_names(&types);
        assert!(result.is_err());
    }
}

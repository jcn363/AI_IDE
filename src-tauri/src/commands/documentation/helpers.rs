//! Documentation helper functions for compiler integration
//!
//! This module provides centralized functions for generating documentation links
//! used throughout the compiler integration system. These functions help provide
//! contextual help and reference materials for various Rust language features,
//! error codes, and development resources.

use crate::modules::shared::diagnostics::*;

/// Generate documentation links for a specific error code
pub fn get_error_documentation_links(error_code: &str) -> Vec<DocumentationLink> {
    vec![
        DocumentationLink {
            title: format!("Rust Error Index - {}", error_code),
            url: format!("https://doc.rust-lang.org/error-index.html#{}", error_code),
            description: "Official Rust documentation for this error".to_string(),
            category: "official".to_string(),
        },
        DocumentationLink {
            title: "Rust Compiler Error Index".to_string(),
            url: "https://doc.rust-lang.org/error-index.html".to_string(),
            description: "Complete list of Rust compiler errors".to_string(),
            category: "reference".to_string(),
        },
    ]
}

/// Generate documentation links for a specific keyword
pub fn get_keyword_documentation_links(keyword: &str) -> Vec<DocumentationLink> {
    vec![
        DocumentationLink {
            title: format!("Rust Reference - {}", keyword),
            url: format!("https://doc.rust-lang.org/reference/keywords.html#{}", keyword),
            description: format!("Official documentation for the '{}' keyword", keyword),
            category: "official".to_string(),
        },
        DocumentationLink {
            title: "Rust by Example".to_string(),
            url: "https://doc.rust-lang.org/rust-by-example/".to_string(),
            description: "Learn Rust with examples".to_string(),
            category: "tutorial".to_string(),
        },
    ]
}

/// Generate documentation links for a specific context or code pattern
pub fn get_context_documentation_links(context: &str) -> Vec<DocumentationLink> {
    // This would analyze the context and return relevant links
    // For now, return some general helpful links
    vec![
        DocumentationLink {
            title: "Rust Standard Library".to_string(),
            url: "https://doc.rust-lang.org/std/".to_string(),
            description: "Standard library documentation".to_string(),
            category: "reference".to_string(),
        },
    ]
}

/// Generate general documentation links for Rust learning and reference
pub fn get_general_documentation_links() -> Vec<DocumentationLink> {
    vec![
        DocumentationLink {
            title: "The Rust Programming Language".to_string(),
            url: "https://doc.rust-lang.org/book/".to_string(),
            description: "The official Rust book".to_string(),
            category: "official".to_string(),
        },
        DocumentationLink {
            title: "Rust Reference".to_string(),
            url: "https://doc.rust-lang.org/reference/".to_string(),
            description: "The Rust language reference".to_string(),
            category: "reference".to_string(),
        },
        DocumentationLink {
            title: "Rustlings".to_string(),
            url: "https://github.com/rust-lang/rustlings".to_string(),
            description: "Small exercises to get you used to reading and writing Rust code".to_string(),
            category: "tutorial".to_string(),
        },
        DocumentationLink {
            title: "Rust Community".to_string(),
            url: "https://www.rust-lang.org/community".to_string(),
            description: "Get help from the Rust community".to_string(),
            category: "community".to_string(),
        },
    ]
}
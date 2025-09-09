//! Error handling and explanation functionality

use crate::diagnostics::*;
use anyhow::Result;
use std::process::Command;
use tauri::State;
use tokio::io::AsyncBufReadExt;

/// Get cached error explanation
pub async fn get_cached_error_explanation(
    error_code: &str,
    explanation_cache: State<'_, ExplanationCacheState>,
    ttl_seconds: u64,
) -> Result<ErrorCodeExplanation> {
    // Try cache first
    {
        let cache_guard = explanation_cache.read().await;
        if let Some(cached) = cache_guard.get(error_code) {
            log::debug!("Returning cached explanation for error code: {}", error_code);
            return Ok(cached.explanation.clone());
        }
    }

    // Get fresh explanation
    let explanation = get_error_code_explanation(error_code).await?;

    // Cache the result
    {
        let mut cache_guard = explanation_cache.write().await;
        cache_guard.insert(error_code.to_string(), explanation.clone(), ttl_seconds);
    }

    Ok(explanation)
}

/// Get error code explanation by calling rustc --explain
pub async fn get_error_code_explanation(error_code: &str) -> Result<ErrorCodeExplanation> {
    log::debug!("Getting explanation for error code: {}", error_code);

    // Run rustc --explain for the error code
    let output = Command::new("rustc")
        .args(&["--explain", error_code])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run rustc --explain: {}", e))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("rustc --explain failed for error code: {}", error_code));
    }

    let explanation_text = String::from_utf8_lossy(&output.stdout);

    // Parse the explanation
    let (title, explanation, examples) = parse_rustc_explanation(&explanation_text);

    // Generate documentation links
    let documentation_links = get_error_documentation_links(error_code);

    // Extract related errors and common causes (placeholder implementations)
    let related_errors = extract_related_errors(&explanation_text);
    let common_causes = extract_common_causes(&explanation_text);
    let suggested_solutions = extract_suggested_solutions(&explanation_text);

    Ok(ErrorCodeExplanation {
        error_code: error_code.to_string(),
        title,
        explanation,
        examples,
        documentation_links,
    })
}

/// Parse rustc --explain output
pub fn parse_rustc_explanation(text: &str) -> (String, String, Vec<ErrorExample>) {
    let lines: Vec<&str> = text.lines().collect();

    let title = lines.first()
        .unwrap_or(&"")
        .trim()
        .to_string();

    let explanation = text.to_string();

    // Extract examples (simplified - would need more sophisticated parsing)
    let mut examples = Vec::new();
    let mut in_example = false;
    let mut current_example = String::new();
    let mut example_description = String::new();

    for line in lines {
        if line.trim().starts_with("```") {
            if in_example {
                // End of example
                examples.push(ErrorExample {
                    title: example_description.clone(),
                    code: current_example.clone(),
                    explanation: "Example code".to_string(),
                });
                current_example.clear();
                example_description.clear();
                in_example = false;
            } else {
                // Start of example
                in_example = true;
            }
        } else if in_example {
            current_example.push_str(line);
            current_example.push('\n');
        } else if !line.trim().is_empty() && !in_example {
            example_description = line.trim().to_string();
        }
    }

    (title, explanation, examples)
}

/// Extract related errors from explanation text
pub fn extract_related_errors(text: &str) -> Vec<String> {
    let mut related = Vec::new();

    // Look for error code patterns like E0001, E0002, etc.
    for line in text.lines() {
        if let Some(captures) = regex::Regex::new(r"E\d{4}")
            .ok()
            .and_then(|re| re.find(line)) {
            let error_code = captures.as_str().to_string();
            if !related.contains(&error_code) {
                related.push(error_code);
            }
        }
    }

    related
}

/// Extract common causes from explanation text
pub fn extract_common_causes(text: &str) -> Vec<String> {
    let mut causes = Vec::new();

    // Look for common patterns that indicate causes
    let cause_patterns = [
        "This error occurs when",
        "This happens when",
        "The cause of this error",
        "This is caused by",
    ];

    for line in text.lines() {
        for pattern in &cause_patterns {
            if line.contains(pattern) {
                causes.push(line.trim().to_string());
                break;
            }
        }
    }

    causes
}

/// Extract suggested solutions from explanation text
pub fn extract_suggested_solutions(text: &str) -> Vec<String> {
    let mut solutions = Vec::new();

    // Look for solution patterns
    let solution_patterns = [
        "To fix this",
        "You can fix this by",
        "The solution is",
        "Try",
        "Consider",
    ];

    for line in text.lines() {
        for pattern in &solution_patterns {
            if line.contains(pattern) {
                solutions.push(line.trim().to_string());
                break;
            }
        }
    }

    solutions
}

/// Get documentation links for error codes
pub fn get_error_documentation_links(error_code: &str) -> Vec<DocumentationLink> {
    vec![
        DocumentationLink {
            title: format!("Rust Error Index - {}", error_code),
            url: format!("https://doc.rust-lang.org/error-index.html#{}", error_code),
            description: "Official Rust documentation for this error".to_string(),
        },
        DocumentationLink {
            title: "Rust Compiler Error Index".to_string(),
            url: "https://doc.rust-lang.org/error-index.html".to_string(),
            description: "Complete list of Rust compiler errors".to_string(),
        },
    ]
}

/// Get documentation links for keywords
pub fn get_keyword_documentation_links(keyword: &str) -> Vec<DocumentationLink> {
    vec![
        DocumentationLink {
            title: format!("Rust Reference - {}", keyword),
            url: format!("https://doc.rust-lang.org/reference/keywords.html#{}", keyword),
            description: format!("Official documentation for the '{}' keyword", keyword),
        },
        DocumentationLink {
            title: "Rust by Example".to_string(),
            url: "https://doc.rust-lang.org/rust-by-example/".to_string(),
            description: "Learn Rust with examples".to_string(),
        },
    ]
}

/// Get documentation links for context
pub fn get_context_documentation_links(context: &str) -> Vec<DocumentationLink> {
    // This would analyze the context and return relevant links
    // For now, return some general helpful links
    vec![
        DocumentationLink {
            title: "Rust Standard Library".to_string(),
            url: "https://doc.rust-lang.org/std/".to_string(),
            description: "Standard library documentation".to_string(),
        },
    ]
}

/// Get general Rust documentation links
pub fn get_general_documentation_links() -> Vec<DocumentationLink> {
    vec![
        DocumentationLink {
            title: "The Rust Programming Language".to_string(),
            url: "https://doc.rust-lang.org/book/".to_string(),
            description: "The official Rust book".to_string(),
        },
        DocumentationLink {
            title: "Rust Reference".to_string(),
            url: "https://doc.rust-lang.org/reference/".to_string(),
            description: "The Rust language reference".to_string(),
        },
        DocumentationLink {
            title: "Rustlings".to_string(),
            url: "https://github.com/rust-lang/rustlings".to_string(),
            description: "Small exercises to get you used to reading and writing Rust code".to_string(),
        },
        DocumentationLink {
            title: "Rust Community".to_string(),
            url: "https://www.rust-lang.org/community".to_string(),
            description: "Get help from the Rust community".to_string(),
        },
    ]
}

/// Run cargo check and return output as string
pub async fn run_cargo_check(workspace_path: &str) -> Result<String> {
    log::debug!("Running cargo check in: {}", workspace_path);

    let mut cmd = tokio::process::Command::new("cargo")
        .args(&["check", "--message-format=json"])
        .current_dir(workspace_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to start cargo check: {}", e))?;

    let stdout = cmd.stdout.take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;

    let mut reader = tokio::io::BufReader::new(stdout);
    let mut output = String::new();
    loop {
        let line = tokio::io::AsyncBufReadExt::fill_buf(&mut reader).await?;
        if line.is_empty() {
            break;
        }
        output.push_str(std::str::from_utf8(line).unwrap_or(""));
        tokio::io::AsyncBufReadExt::consume(&mut reader, line.len());
    }

    let status = cmd.wait().await?;

    // Cargo check can return non-zero exit code even with successful compilation
    // if there are warnings or errors, so we don't treat non-zero as failure
    log::debug!("Cargo check completed with status: {}", status);

    Ok(output)
}

/// Lookup documentation for errors or keywords
pub fn lookup_documentation(request: DocumentationLookupRequest) -> Vec<DocumentationLink> {
    let mut links = Vec::new();

    // Add error-specific documentation
    if let Some(error_code) = &request.error_code {
        links.extend(get_error_documentation_links(error_code));
    }

    // Add keyword-specific documentation
    if let Some(keyword) = &request.keyword {
        links.extend(get_keyword_documentation_links(keyword));
    }

    // Add context-specific documentation
    if let Some(context) = &request.context {
        links.extend(get_context_documentation_links(context));
    }

    // Add general Rust documentation
    links.extend(get_general_documentation_links());

    links
}
//! Core diagnostic commands for compiler integration
//!
//! This module contains the primary diagnostic functions including
//! compilation analysis, error explanation, and documentation lookup.

use crate::modules::shared::diagnostics::*;
use crate::commands::documentation::helpers::*;
use crate::commands::utils::*;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use tauri::State;
use tokio::time::timeout;

/// Get compiler diagnostics for a workspace
#[tauri::command]
pub async fn get_compiler_diagnostics(
    request: CompilerDiagnosticsRequest,
    diagnostic_cache: State<'_, DiagnosticCacheState>,
    explanation_cache: State<'_, ExplanationCacheState>,
) -> Result<CompilerDiagnosticsResult, String> {
    log::info!("Getting compiler diagnostics for: {}", request.workspace_path);

    let cache_key = format!("{}:{}",
        request.workspace_path,
        if request.include_explanations { "with_explanations" } else { "basic" });

    // Check cache first if enabled
    if request.use_cache {
        let cache_guard = diagnostic_cache.read().await;
        if let Some(cached) = cache_guard.get(&cache_key) {
            log::debug!("Returning cached diagnostics for {}", request.workspace_path);
            let mut result = cached.value.clone();
            result.metadata.cached = true;
            return Ok(result);
        }
    }

    let start_time = SystemTime::now();

    // Run cargo check with timeout
    let timeout_duration = Duration::from_secs(request.timeout_seconds.unwrap_or(30));
    let diagnostics_result = timeout(
        timeout_duration,
        run_cargo_check(&request.workspace_path)
    ).await
    .map_err(|_| "Cargo check timed out".to_string())?
    .map_err(|e| format!("Cargo check failed: {}", e))?;

    let compilation_time = start_time.elapsed()
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    // Parse diagnostics
    let mut diagnostics = Vec::new();
    let mut explanations = HashMap::new();
    let mut suggested_fixes = Vec::new();
    let mut error_count = 0;
    let mut warning_count = 0;
    let mut note_count = 0;

    for line in diagnostics_result.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(message) = json.get("message") {
                if let Some(diagnostic) = parse_compiler_diagnostic(message, &request.workspace_path).await {
                    // Count diagnostic levels
                    match diagnostic.level.as_str() {
                        "error" => error_count += 1,
                        "warning" => warning_count += 1,
                        "note" => note_count += 1,
                        _ => {}
                    }

                    // Get explanation for error codes if requested
                    if request.include_explanations {
                        if let Some(code) = &diagnostic.code {
                            if let Ok(explanation) = get_cached_error_explanation(
                                &code.code,
                                explanation_cache.clone(),
                                request.cache_ttl_seconds.unwrap_or(3600)
                            ).await {
                                explanations.insert(code.code.clone(), explanation);
                            }
                        }
                    }

                    // Generate suggested fixes if requested
                    if request.include_suggested_fixes {
                        if let Ok(fixes) = generate_suggested_fixes(&diagnostic).await {
                            suggested_fixes.extend(fixes);
                        }
                    }

                    diagnostics.push(diagnostic);
                }
            }
        }
    }

    let metadata = DiagnosticMetadata {
        workspace_path: request.workspace_path.clone(),
        timestamp: chrono::Utc::now(),
        compilation_time_ms: compilation_time,
        total_errors: error_count,
        total_warnings: warning_count,
        total_notes: note_count,
        cached: false,
    };

    let result = CompilerDiagnosticsResult {
        diagnostics,
        explanations,
        suggested_fixes,
        metadata,
    };

    // Cache the result if enabled
    if request.use_cache {
        let mut cache_guard = diagnostic_cache.write().await;
        cache_guard.insert(
            cache_key,
            result.clone(),
            request.cache_ttl_seconds.unwrap_or(300) // 5 minutes default
        );
    }

    Ok(result)
}

/// Explain a specific error code
#[tauri::command]
pub async fn explain_error_code(
    request: ErrorCodeExplanationRequest,
    explanation_cache: State<'_, ExplanationCacheState>,
) -> Result<ErrorCodeExplanation, String> {
    log::info!("Explaining error code: {}", request.error_code);

    get_cached_error_explanation(
        &request.error_code,
        explanation_cache,
        request.cache_ttl_seconds.unwrap_or(86400) // 24 hours default
    ).await
    .map_err(|e| format!("Failed to get error explanation: {}", e))
}

/// Lookup documentation for errors or keywords
#[tauri::command]
pub async fn lookup_documentation(
    request: DocumentationLookupRequest,
) -> Result<Vec<DocumentationLink>, String> {
    log::info!("Looking up documentation for: {:?}", request);

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

    Ok(links)
}
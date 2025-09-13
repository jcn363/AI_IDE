//! Tauri commands for compiler integration functionality
//!
//! This module provides commands for integrating with the Rust compiler,
//! including diagnostic parsing, error explanations, documentation lookup,
//! and real-time diagnostic streaming.

// Sub-modules organized by functionality
pub mod core_diagnostics;
pub mod processing_helpers;
pub mod streaming_cache;
pub mod utils;
pub mod documentation {
    pub mod helpers;
}

// Re-export all public functions for backward compatibility
pub use core_diagnostics::*;
pub use documentation::helpers::*;
pub use processing_helpers::*;
pub use streaming_cache::*;

// Re-export shared diagnostics types
pub use crate::modules::shared::diagnostics::*;

/// Get compiler diagnostics for a workspace
#[tauri::command]
pub async fn get_compiler_diagnostics(
    request: CompilerDiagnosticsRequest,
    diagnostic_cache: State<'_, DiagnosticCacheState>,
    explanation_cache: State<'_, ExplanationCacheState>,
) -> Result<CompilerDiagnosticsResult, String> {
    log::info!(
        "Getting compiler diagnostics for: {}",
        request.workspace_path
    );

    let cache_key = format!(
        "{}:{}",
        request.workspace_path,
        if request.include_explanations {
            "with_explanations"
        } else {
            "basic"
        }
    );

    // Check cache first if enabled
    if request.use_cache {
        let cache_guard = diagnostic_cache.read().await;
        if let Some(cached) = cache_guard.get(&cache_key) {
            log::debug!(
                "Returning cached diagnostics for {}",
                request.workspace_path
            );
            let mut result = cached.value.clone();
            result.metadata.cached = true;
            return Ok(result);
        }
    }

    let start_time = SystemTime::now();

    // Run cargo check with timeout
    let timeout_duration = Duration::from_secs(request.timeout_seconds.unwrap_or(30));
    let diagnostics_result = timeout(timeout_duration, run_cargo_check(&request.workspace_path))
        .await
        .map_err(|_| "Cargo check timed out".to_string())?
        .map_err(|e| format!("Cargo check failed: {}", e))?;

    let compilation_time = start_time
        .elapsed()
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
                                request.cache_ttl_seconds.unwrap_or(3600),
                            )
                            .await
                            {
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
        workspace_path:      request.workspace_path.clone(),
        timestamp:           chrono::Utc::now(),
        compilation_time_ms: compilation_time,
        total_errors:        error_count,
        total_warnings:      warning_count,
        total_notes:         note_count,
        cached:              false,
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
            request.cache_ttl_seconds.unwrap_or(300), // 5 minutes default
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
        request.cache_ttl_seconds.unwrap_or(86400), // 24 hours default
    )
    .await
    .map_err(|e| format!("Failed to get error explanation: {}", e))
}

/// Lookup documentation for errors or keywords
#[tauri::command]
pub async fn lookup_documentation(request: DocumentationLookupRequest) -> Result<Vec<DocumentationLink>, String> {
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

/// Subscribe to real-time diagnostic updates
#[tauri::command]
pub async fn subscribe_to_diagnostics(
    request: DiagnosticStreamRequest,
    stream_state: State<'_, DiagnosticStreamState>,
) -> Result<String, String> {
    log::info!("Subscribing to diagnostics for: {}", request.workspace_path);

    let stream_id = uuid::Uuid::new_v4().to_string();

    let stream = DiagnosticStream {
        id:             stream_id.clone(),
        workspace_path: request.workspace_path.clone(),
        is_active:      true,
        last_update:    SystemTime::now(),
        subscribers:    vec![request.subscriber_id],
    };

    {
        let mut stream_guard = stream_state.write().await;
        stream_guard.insert(stream_id.clone(), stream);
    }

    // Start background task for periodic updates if auto-refresh is enabled
    if let Some(interval) = request.auto_refresh_interval_seconds {
        let stream_state_clone = stream_state.inner().clone();
        let stream_id_clone = stream_id.clone();
        let workspace_path = request.workspace_path.clone();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(Duration::from_secs(interval));

            loop {
                interval_timer.tick().await;

                // Check if stream is still active
                {
                    let stream_guard = stream_state_clone.read().await;
                    if let Some(stream) = stream_guard.get(&stream_id_clone) {
                        if !stream.is_active {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                // Run diagnostics and emit update
                if let Ok(diagnostics) = run_cargo_check(&workspace_path).await {
                    // In a real implementation, you would emit this to subscribers
                    log::debug!(
                        "Diagnostic update available for stream: {}",
                        stream_id_clone
                    );
                }
            }
        });
    }

    Ok(stream_id)
}

/// Unsubscribe from diagnostic updates
#[tauri::command]
pub async fn unsubscribe_from_diagnostics(
    stream_id: String,
    subscriber_id: String,
    stream_state: State<'_, DiagnosticStreamState>,
) -> Result<String, String> {
    log::info!("Unsubscribing from diagnostics: {}", stream_id);

    let mut stream_guard = stream_state.write().await;

    if let Some(stream) = stream_guard.get_mut(&stream_id) {
        stream.subscribers.retain(|id| id != &subscriber_id);

        // If no more subscribers, deactivate the stream
        if stream.subscribers.is_empty() {
            stream.is_active = false;
        }

        Ok("Unsubscribed successfully".to_string())
    } else {
        Err("Stream not found".to_string())
    }
}

/// Clear diagnostic cache
#[tauri::command]
pub async fn clear_diagnostic_cache(
    diagnostic_cache: State<'_, DiagnosticCacheState>,
    explanation_cache: State<'_, ExplanationCacheState>,
) -> Result<String, String> {
    async_command!("Clearing diagnostic caches", {
        {
            let mut cache_guard = diagnostic_cache.write().await;
            cache_guard.clear();
        }

        {
            let mut cache_guard = explanation_cache.write().await;
            cache_guard.clear();
        }

        Ok("Caches cleared successfully")
    })
    .await
}

/// Get cache statistics
#[tauri::command]
pub async fn get_cache_statistics(
    diagnostic_cache: State<'_, DiagnosticCacheState>,
    explanation_cache: State<'_, ExplanationCacheState>,
) -> Result<CacheStatistics, String> {
    let diagnostic_cache_guard = diagnostic_cache.read().await;
    let explanation_cache_guard = explanation_cache.read().await;

    let stats = CacheStatistics {
        diagnostic_cache_size:       diagnostic_cache_guard.len(),
        diagnostic_cache_max_size:   diagnostic_cache_guard.max_entries,
        explanation_cache_size:      explanation_cache_guard.len(),
        explanation_cache_max_size:  explanation_cache_guard.max_entries,
        diagnostic_cache_hit_ratio:  0.0, // Would need to track hits/misses
        explanation_cache_hit_ratio: 0.0, // Would need to track hits/misses
    };

    Ok(stats)
}

/// Cache statistics
#[derive(Debug, Serialize)]
pub struct CacheStatistics {
    pub diagnostic_cache_size:       usize,
    pub diagnostic_cache_max_size:   usize,
    pub explanation_cache_size:      usize,
    pub explanation_cache_max_size:  usize,
    pub diagnostic_cache_hit_ratio:  f32,
    pub explanation_cache_hit_ratio: f32,
}

// Helper functions

async fn run_cargo_check(workspace_path: &str) -> Result<String> {
    // Use centralized run_cargo_check function from error_handling module
    crate::diagnostics::error_handling::run_cargo_check(workspace_path).await
}

async fn parse_compiler_diagnostic(message: &serde_json::Value, workspace_path: &str) -> Option<CompilerDiagnostic> {
    // Use centralized parsing function
    parse_compiler_diagnostic(message, workspace_path).await
}

fn parse_compiler_span(span: &serde_json::Value) -> Option<CompilerSpan> {
    // Use centralized parsing function
    crate::diagnostics::parsing::parse_compiler_span(span)
}

fn parse_span_text(text: &serde_json::Value) -> Option<SpanText> {
    // Use centralized parsing function
    crate::diagnostics::parsing::parse_span_text(text)
}

// Helper functions using centralized diagnostics module

// Removed - using centralized module

async fn get_cached_error_explanation(
    error_code: &str,
    explanation_cache: State<'_, ExplanationCacheState>,
    ttl_seconds: u64,
) -> Result<ErrorCodeExplanation> {
    // Use centralized function
    crate::diagnostics::error_handling::get_cached_error_explanation(error_code, explanation_cache, ttl_seconds).await
}

async fn get_error_code_explanation(error_code: &str) -> Result<ErrorCodeExplanation> {
    // Use centralized function
    crate::diagnostics::error_handling::get_error_code_explanation(error_code).await
}

fn parse_rustc_explanation(text: &str) -> (String, String, Vec<ErrorExample>) {
    let lines: Vec<&str> = text.lines().collect();

    let title = lines.first().unwrap_or(&"").trim().to_string();

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
                    description: example_description.clone(),
                    code:        current_example.clone(),
                    explanation: "Example code".to_string(),
                    fix:         None,
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

fn extract_related_errors(text: &str) -> Vec<String> {
    let mut related = Vec::new();

    // Look for error code patterns like E0001, E0002, etc.
    for line in text.lines() {
        if let Some(captures) = regex::Regex::new(r"E\d{4}")
            .ok()
            .and_then(|re| re.find(line))
        {
            let error_code = captures.as_str().to_string();
            if !related.contains(&error_code) {
                related.push(error_code);
            }
        }
    }

    related
}

fn extract_common_causes(text: &str) -> Vec<String> {
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

fn extract_suggested_solutions(text: &str) -> Vec<String> {
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

async fn generate_suggested_fixes(diagnostic: &CompilerDiagnostic) -> Result<Vec<FixSuggestion>> {
    let mut fixes = Vec::new();

    // Extract suggestions from compiler spans
    for span in &diagnostic.spans {
        if let Some(replacement) = &span.suggested_replacement {
            let fix = FixSuggestion {
                id:               uuid::Uuid::new_v4().to_string(),
                title:            "Apply compiler suggestion".to_string(),
                description:      span
                    .label
                    .clone()
                    .unwrap_or_else(|| "Compiler suggested fix".to_string()),
                fix_type:         FixType::QuickFix,
                changes:          vec![CodeChange {
                    file_path:   span.file_name.clone(),
                    range:       (
                        span.line_start,
                        span.column_start,
                        span.line_end,
                        span.column_end,
                    ),
                    old_text:    String::new(), // Would need to extract from source
                    new_text:    replacement.clone(),
                    change_type: CompilerChangeType::Replace,
                }],
                confidence:       if span
                    .suggestion_applicability
                    .as_ref()
                    .map_or(false, |a| a == "machine-applicable")
                {
                    0.9
                } else {
                    0.7
                },
                estimated_effort: EstimatedEffort::Trivial,
                benefits:         vec!["Fixes compiler error".to_string()],
                risks:            vec![],
            };
            fixes.push(fix);
        }
    }

    Ok(fixes)
}

// Documentation link functions moved to src-tauri/src/commands/documentation/helpers.rs

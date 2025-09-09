//! AI commands module for the Rust AI IDE
//!
//! This module consolidates AI-related commands into the new modular structure.

use crate::commands::ai::services::AIServiceState;
use crate::utils;
use std::collections::HashMap;

// Core AI communication commands

/// Send a message to AI service
#[tauri::command]
pub async fn send_ai_message(
    message: String,
    context: serde_json::Value,
    ai_service_state: tauri::State<'_, AIServiceState>,
) -> Result<String, String> {
    log::info!("AI message: {}", message);
    log::debug!("AI context: {:#?}", context);

    // Get AI service from managed state
    let ai_service = utils::get_or_create_ai_service(&ai_service_state).await?;

    // Convert context to AI context with all required fields
    let current_code = match context.get("current_code").and_then(|v| v.as_str()).map(|s| s.to_string()) {
        Some(code) => {
            if code.is_empty() {
                log::info!("Empty current_code provided, using default");
            }
            code
        }
        None => {
            log::warn!("No current_code field found in context, using empty string");
            String::new()
        }
    };

    let ai_context = rust_ai_ide_lsp::AIContext {
        current_code,
        file_name: context
            .get("file_name")
            .and_then(|v| v.as_str().map(|s| s.to_string())),
        cursor_position: context
            .get("cursor_position")
            .and_then(|v| v.as_object())
            .and_then(|obj| {
                let line = obj.get("line")?.as_u64()? as u32;
                let character = obj.get("character")?.as_u64()? as u32;
                Some((line, character))
            }),
        selection: context
            .get("selection")
            .and_then(|v| v.as_str().map(|s| s.to_string())),
        project_context: match context.get("project_context").and_then(|v| v.as_object()) {
            Some(obj) => {
                let ctx_result: HashMap<String, String> = obj.iter()
                    .filter_map(|(k, v)| {
                        match v.as_str().map(|s| (k.clone(), s.to_string())) {
                            Some((key, val)) => {
                                if key.is_empty() || val.is_empty() {
                                    log::warn!("Empty key/value pair in project_context: key={}, val={}", key, val);
                                }
                                Some((key, val))
                            }
                            None => {
                                log::error!("Invalid string value in project_context for key: {}", k);
                                Some((k.clone(), String::new()))
                            }
                        }
                    })
                    .collect();

                if ctx_result.is_empty() {
                    log::debug!("No valid key-value pairs found in project_context");
                } else {
                    log::debug!("Extracted {} project context entries", ctx_result.len());
                }
                ctx_result
            }
            None => {
                log::debug!("No project_context provided in request");
                HashMap::new()
            }
        },
    };

    log::debug!("Created AI context: {:#?}", ai_context);

    // Process the message
    let completions = ai_service
        .get_completions(ai_context)
        .await
        .map_err(|e| format!("AI processing error: {}", e))?;
    let response = match completions.first() {
        Some(s) => s.text.clone(),
        None => {
            log::warn!("No completion generated for AI request");
            return Err("No completion generated".to_string());
        }
    };

    log::info!("AI response generated");
    Ok(response)
}

/// Get code completion suggestions
#[tauri::command]
pub async fn ai_code_completion(
    code: String,
    file_name: Option<String>,
    cursor_line: Option<u32>,
    cursor_char: Option<u32>,
    max_suggestions: Option<u32>,
    ai_service_state: tauri::State<'_, AIServiceState>,
) -> Result<Vec<String>, String> {
    // Get AI service from managed state
    let ai_service = utils::get_or_create_ai_service(&ai_service_state).await?;
    let ctx = rust_ai_ide_lsp::AIContext {
        current_code: code,
        file_name,
        cursor_position: match (cursor_line, cursor_char) { (Some(l), Some(c)) => Some((l, c)), _ => None },
        selection: None,
        project_context: Default::default(),
    };
    let list = ai_service.get_completions(ctx).await.map_err(|e| format!("AI error: {}", e))?;
    let n = max_suggestions.unwrap_or(5) as usize;
    Ok(list.into_iter().take(n).map(|c| c.text).collect())
}

/// Generate code from prompt
#[tauri::command]
pub async fn ai_generate_code(prompt: String, context_code: Option<String>, file_name: Option<String>) -> Result<String, String> {
    use rust_ai_ide_lsp::AIService;
    let ai_service = AIService::new(rust_ai_ide_lsp::AIProvider::Local {
        model_path: crate::utils::get_model_path(),
        endpoint: crate::utils::get_ai_endpoint(),
    });
    let ctx = rust_ai_ide_lsp::AIContext {
        current_code: match context_code {
            Some(code) if !code.is_empty() => code,
            Some(code) => {
                log::warn!("Empty context_code provided, using empty string");
                code
            }
            None => {
                log::warn!("No context_code provided, using empty string");
                String::new()
            }
        },
        file_name,
        cursor_position: None,
        selection: None,
        project_context: Default::default(),
    };
    let task = format!("Generate code for: {}\nReturn only code if possible.", prompt);
    let list = ai_service.get_task_response(ctx, task).await.map_err(|e| format!("AI error: {}", e))?;
    Ok(list)
}

/// Assist with documentation
#[tauri::command]
pub async fn ai_doc_assist(
    symbol: String,
    code_context: Option<String>,
    ai_service_state: tauri::State<'_, AIServiceState>,
) -> Result<String, String> {
    // Get AI service from managed state
    let ai_service = utils::get_or_create_ai_service(&ai_service_state).await?;

    let ctx = rust_ai_ide_lsp::AIContext {
        current_code: match code_context {
            Some(code) if !code.is_empty() => code,
            Some(code) => {
                log::warn!("Empty code_context provided for documentation");
                code
            }
            None => {
                log::warn!("No code_context provided for documentation");
                String::new()
            }
        },
        file_name: None,
        cursor_position: None,
        selection: None,
        project_context: Default::default()
    };
    let task = format!("Write Rust documentation for symbol `{}` found in the provided context. Include a brief summary, parameters, return value, and examples if relevant.", symbol);
    let out = ai_service.get_task_response(ctx, task).await.map_err(|e| format!("AI error: {}", e))?;
    Ok(out)
}

/// Refactor code with AI assistance
#[tauri::command]
pub async fn ai_refactor_code(
    code: String,
    instruction: String,
    ai_service_state: tauri::State<'_, AIServiceState>,
) -> Result<String, String> {
    // Get AI service from managed state
    let ai_service = utils::get_or_create_ai_service(&ai_service_state).await?;

    let ctx = rust_ai_ide_lsp::AIContext {
        current_code: code,
        file_name: None,
        cursor_position: None,
        selection: None,
        project_context: Default::default()
    };
    let task = format!("Refactor the code according to the following instruction. Respond with the full, updated code only.\nInstruction: {}", instruction);
    let out = ai_service.get_task_response(ctx, task).await.map_err(|e| format!("AI error: {}", e))?;
    Ok(out)
}

/// Explain code with AI assistance
#[tauri::command]
pub async fn ai_explain_code(
    code: String,
    selection: Option<String>,
    ai_service_state: tauri::State<'_, AIServiceState>,
) -> Result<String, String> {
    // Get AI service from managed state
    let ai_service = utils::get_or_create_ai_service(&ai_service_state).await?;

    let sel = match selection {
        Some(s) if !s.is_empty() => s,
        Some(s) => {
            log::warn!("Empty selection provided, using empty string");
            s
        }
        None => {
            log::debug!("No selection provided, analyzing entire code");
            String::new()
        }
    };
    let ctx = rust_ai_ide_lsp::AIContext {
        current_code: code,
        file_name: None,
        cursor_position: None,
        selection: Some(sel.clone()),
        project_context: Default::default()
    };
    let task = if sel.is_empty() { "Explain the provided Rust code in detail.".to_string() } else { format!("Explain the following selected Rust code: {}", sel) };
    let out = ai_service.get_task_response(ctx, task).await.map_err(|e| format!("AI error: {}", e))?;
    Ok(out)
}

/// Provide context-aware help
#[tauri::command]
pub async fn ai_context_help(
    question: String,
    code_context: Option<String>,
    file_name: Option<String>,
    ai_service_state: tauri::State<'_, AIServiceState>,
) -> Result<String, String> {
    // Get AI service from managed state
    let ai_service = utils::get_or_create_ai_service(&ai_service_state).await?;

    let ctx = rust_ai_ide_lsp::AIContext {
        current_code: match code_context {
            Some(code) if !code.is_empty() => code,
            Some(code) => {
                log::warn!("Empty code_context provided for help");
                code
            }
            None => {
                log::warn!("No code_context provided for help");
                String::new()
            }
        },
        file_name,
        cursor_position: None,
        selection: None,
        project_context: Default::default()
    };
    let task = format!("Question: {}\nAnswer with concise, context-aware guidance.", question);
    let out = ai_service.get_task_response(ctx, task).await.map_err(|e| format!("AI error: {}", e))?;
    Ok(out)
}
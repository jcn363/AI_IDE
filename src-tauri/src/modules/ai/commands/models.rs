use std::collections::HashMap;

/// AI model interaction and messaging commands module
///
/// This module handles direct AI model interactions, messaging,
/// and model-specific operations through the LSP service interface.
use crate::commands::ai::services::AIServiceState;
use crate::utils;

/// Send a message to AI service for processing
///
/// # Arguments
/// * `message` - The message to send to AI
/// * `context` - Context information (current_code, file_name, cursor_position, etc.)
/// * `ai_service_state` - Managed AI service state
///
/// # Returns
/// * `Result<String, String>` - AI response or error
///
/// # Errors
/// Returns error if AI service fails, message is invalid, or service unavailable
///
/// # Context Fields
/// - `current_code`: Current code content
/// - `file_name`: Optional file name
/// - `cursor_position`: Optional (line, character) tuple
/// - `selection`: Optional selected text
/// - `project_context`: HashMap of project-related context
#[tauri::command]
pub async fn send_ai_message(
    message: String,
    context: serde_json::Value,
    ai_service_state: tauri::State<'_, AIServiceState>,
) -> Result<String, String> {
    log::info!("AI message received: {}", message);

    // Validate input
    if message.is_empty() {
        log::warn!("Empty message provided for AI interaction");
        return Err("Message cannot be empty".to_string());
    }

    // Get AI service from managed state
    let ai_service = utils::get_or_create_ai_service(&ai_service_state).await?;

    // Validate and extract current_code
    let current_code = match context
        .get("current_code")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
    {
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

    // Extract cursor position
    let cursor_position = context
        .get("cursor_position")
        .and_then(|v| v.as_object())
        .and_then(|obj| {
            let line = obj.get("line")?.as_u64()? as u32;
            let character = obj.get("character")?.as_u64()? as u32;
            Some((line, character))
        });

    // Extract file name
    let file_name = context
        .get("file_name")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    // Extract selection
    let selection = context
        .get("selection")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    // Extract project context
    let project_context: HashMap<String, String> = match context
        .get("project_context")
        .and_then(|v| v.as_object())
    {
        Some(obj) => {
            let ctx_result: HashMap<String, String> = obj
                .iter()
                .filter_map(
                    |(k, v)| match v.as_str().map(|s| (k.clone(), s.to_string())) {
                        Some((key, val)) => {
                            if key.is_empty() || val.is_empty() {
                                log::warn!(
                                    "Empty key/value pair in project_context: key={}, val={}",
                                    key,
                                    val
                                );
                            }
                            Some((key, val))
                        }
                        None => {
                            log::error!("Invalid string value in project_context for key: {}", k);
                            Some((k.clone(), String::new()))
                        }
                    },
                )
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
    };

    // Create AI context
    let ai_context = rust_ai_ide_lsp::AIContext {
        current_code,
        file_name,
        cursor_position,
        selection,
        project_context,
    };

    log::debug!("Created AI context: {:#?}", ai_context);

    // Process the message through AI completions
    let completions = ai_service.get_completions(ai_context).await.map_err(|e| {
        log::error!("AI processing error for message '{}': {}", message, e);
        format!("AI processing error: {}", e)
    })?;

    // Get the first completion
    let response = match completions.first() {
        Some(s) => s.text.clone(),
        None => {
            log::warn!("No completion generated for AI message: {}", message);
            return Err("No completion generated".to_string());
        }
    };

    log::info!("AI response generated successfully");
    Ok(response)
}

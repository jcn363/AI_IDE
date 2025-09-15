use rust_ai_ide_sanitizer::TauriInputSanitizer;

/// AI code completion and generation commands module
///
/// This module handles code completion suggestions and code generation
/// using AI services integrated with the Rust AI IDE.
use crate::commands::ai::services::AIServiceState;
use crate::errors::{IDEError, IDEResult};
use crate::utils;

/// Get code completion suggestions from AI service
///
/// # Arguments
/// * `code` - The code to complete
/// * `file_name` - Optional file name for context
/// * `cursor_line` - Optional cursor line position
/// * `cursor_char` - Optional cursor character position
/// * `max_suggestions` - Optional maximum number of suggestions (default: 5)
/// * `ai_service_state` - Managed AI service state
///
/// # Returns
/// * `IDEResult<Vec<String>>` - List of completion suggestions or structured error
///
/// # Errors
/// Returns `IDEError` if AI service fails to generate completions or service is not available
#[tauri::command]
pub async fn ai_code_completion(
    code: String,
    file_name: Option<String>,
    cursor_line: Option<u32>,
    cursor_char: Option<u32>,
    max_suggestions: Option<u32>,
    ai_service_state: tauri::State<'_, AIServiceState>,
) -> IDEResult<Vec<String>> {
    log::info!("Code completion requested: code length {}", code.len());

    // Validate input using sanitization
    if code.is_empty() {
        log::warn!("Empty code provided for completion");
        return Err(IDEError::Validation("Code cannot be empty".to_string()));
    }

    // Sanitize text input to prevent injection attacks
    let sanitized_code = TauriInputSanitizer::sanitize_text(&code)
        .map_err(|e| IDEError::Validation(format!("Input sanitization failed: {}", e)))?;

    // Validate file_name if provided
    if let Some(ref fname) = file_name {
        if fname.is_empty() {
            return Err(IDEError::Validation(
                "File name cannot be empty".to_string(),
            ));
        }
        // Additional path validation for file names
        TauriInputSanitizer::validate_path(fname)
            .map_err(|e| IDEError::PathValidation(format!("Invalid file path: {}", e)))?;
    }

    // Get AI service from managed state with retry logic
    let ai_service = utils::get_or_create_ai_service(&ai_service_state)
        .await
        .map_err(|e| {
            log::error!("AI service acquisition failed: {}", e);
            IDEError::AIService(format!("Service unavailable: {}", e))
        })?;

    let ctx = rust_ai_ide_lsp::AIContext {
        current_code: sanitized_code,
        file_name,
        cursor_position: match (cursor_line, cursor_char) {
            (Some(l), Some(c)) => Some((l, c)),
            _ => None,
        },
        selection: None,
        project_context: Default::default(),
    };

    log::debug!("Requesting AI completion for context: {:#?}", ctx);

    let list = ai_service.get_completions(ctx).await.map_err(|e| {
        log::error!("AI completion error: {}", e);
        IDEError::AIService(format!("Completion generation failed: {}", e))
    })?;

    let n = max_suggestions.unwrap_or(5) as usize;
    let suggestions: Vec<String> = list.into_iter().take(n).map(|c| c.text).collect();

    log::info!("AI completion generated {} suggestions", suggestions.len());
    Ok(suggestions)
}

/// Generate code from natural language prompt
///
/// # Arguments
/// * `prompt` - Natural language description of desired code
/// * `context_code` - Optional existing code for context
/// * `file_name` - Optional file name for context
///
/// # Returns
/// * `IDEResult<String>` - Generated code or structured error
///
/// # Errors
/// Returns IDEError if AI service fails to generate code or invalid parameters
#[tauri::command]
pub async fn ai_generate_code(
    prompt: String,
    context_code: Option<String>,
    file_name: Option<String>,
) -> IDEResult<String> {
    log::info!("Code generation requested: {}", prompt);

    // Validate input
    if prompt.is_empty() {
        log::warn!("Empty prompt provided for code generation");
        return Err(IDEError::Validation("Prompt cannot be empty".to_string()));
    }

    // Sanitize prompt input
    let sanitized_prompt = TauriInputSanitizer::sanitize_text(&prompt)
        .map_err(|e| IDEError::Validation(format!("Prompt sanitization failed: {}", e)))?;

    // Validate context_code if provided
    if let Some(ref code) = context_code {
        if code.is_empty() {
            return Err(IDEError::Validation(
                "Context code cannot be empty string".to_string(),
            ));
        }
        TauriInputSanitizer::sanitize_text(code).map_err(|e| {
            IDEError::Validation(format!("Context code sanitization failed: {}", e))
        })?;
    }

    // Validate file_name if provided
    if let Some(ref fname) = file_name {
        if fname.is_empty() {
            return Err(IDEError::Validation(
                "File name cannot be empty".to_string(),
            ));
        }
        TauriInputSanitizer::validate_path(fname)
            .map_err(|e| IDEError::PathValidation(format!("Invalid file path: {}", e)))?;
    }

    // Create AI service instance with error handling
    let ai_service = rust_ai_ide_lsp::AIService::new(rust_ai_ide_lsp::AIProvider::Local {
        model_path: crate::utils::get_model_path(),
        endpoint: crate::utils::get_ai_endpoint(),
    });

    let ctx = rust_ai_ide_lsp::AIContext {
        current_code: context_code.unwrap_or_default(),
        file_name,
        cursor_position: None,
        selection: None,
        project_context: Default::default(),
    };

    let task = format!(
        "Generate code for: {}\nReturn only code if possible. Include comments for clarity.",
        sanitized_prompt
    );
    log::debug!("AI task: {}", task);

    let result = ai_service.get_task_response(ctx, task).await.map_err(|e| {
        log::error!("AI code generation error: {}", e);
        IDEError::AIService(format!("Code generation failed: {}", e))
    })?;

    log::info!("Code generation completed successfully");
    Ok(result)
}

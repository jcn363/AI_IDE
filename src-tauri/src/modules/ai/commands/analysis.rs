/// AI code analysis and assistance commands module
///
/// This module provides AI-powered code analysis, documentation,
/// refactoring, and contextual help features.
use crate::commands::ai::services::AIServiceState;
use crate::utils;

/// Explain code implementation and structure using AI
///
/// # Arguments
/// * `code` - The code to analyze and explain
/// * `selection` - Optional selected code portion to focus on
/// * `ai_service_state` - Managed AI service state
///
/// # Returns
/// * `Result<String, String>` - AI-generated explanation or error
///
/// # Errors
/// Returns error if AI service fails to generate explanation or service unavailable
#[tauri::command]
pub async fn ai_explain_code(
    code: String,
    selection: Option<String>,
    ai_service_state: tauri::State<'_, AIServiceState>,
) -> Result<String, String> {
    log::info!("Code explanation requested for {} characters", code.len());

    // Validate input
    if code.is_empty() {
        log::warn!("Empty code provided for explanation");
        return Err("Code cannot be empty".to_string());
    }

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
        current_code:    code,
        file_name:       None,
        cursor_position: None,
        selection:       Some(sel.clone()),
        project_context: Default::default(),
    };

    let task = if sel.is_empty() {
        "Explain the provided Rust code in detail. Include purpose, structure, key components, and any notable \
         patterns or implementation details."
            .to_string()
    } else {
        format!(
            "Explain this selected Rust code: {}\n\nFocus on what this code snippet does, how it works, and its \
             significance within the context.",
            sel
        )
    };

    log::debug!("AI explanation task: {}", task);

    let explanation = ai_service.get_task_response(ctx, task).await.map_err(|e| {
        log::error!("AI explanation error: {}", e);
        format!("Code explanation failed: {}", e)
    })?;

    log::info!("Code explanation generated successfully");
    Ok(explanation)
}

/// Assist with documentation generation for code symbols
///
/// # Arguments
/// * `symbol` - The symbol to document (function, struct, etc.)
/// * `code_context` - Optional surrounding code for context
/// * `ai_service_state` - Managed AI service state
///
/// # Returns
/// * `Result<String, String>` - Generated documentation or error
///
/// # Errors
/// Returns error if AI service fails or invalid symbol provided
#[tauri::command]
pub async fn ai_doc_assist(
    symbol: String,
    code_context: Option<String>,
    ai_service_state: tauri::State<'_, AIServiceState>,
) -> Result<String, String> {
    log::info!("Documentation assist requested for symbol: {}", symbol);

    // Validate input
    if symbol.is_empty() {
        log::warn!("Empty symbol provided for documentation");
        return Err("Symbol cannot be empty".to_string());
    }

    // Get AI service from managed state
    let ai_service = utils::get_or_create_ai_service(&ai_service_state).await?;

    let ctx = rust_ai_ide_lsp::AIContext {
        current_code:    match code_context {
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
        file_name:       None,
        cursor_position: None,
        selection:       None,
        project_context: Default::default(),
    };

    let task = format!(
        "Write comprehensive Rust documentation for the symbol `{}`. Include:
1. Brief summary of what it does
2. Parameter descriptions (if applicable)
3. Return value description (if applicable)
4. Usage examples
5. Important notes or considerations

Format the documentation using standard Rust doc comment conventions.",
        symbol
    );

    log::debug!("AI documentation task: {}", task);

    let documentation = ai_service.get_task_response(ctx, task).await.map_err(|e| {
        log::error!("AI documentation error: {}", e);
        format!("Documentation generation failed: {}", e)
    })?;

    log::info!("Documentation generated for symbol: {}", symbol);
    Ok(documentation)
}

/// Refactor code with AI-powered suggestions
///
/// # Arguments
/// * `code` - The code to refactor
/// * `instruction` - Refactoring instructions or goals
/// * `ai_service_state` - Managed AI service state
///
/// # Returns
/// * `Result<String, String>` - Refactored code or error
///
/// # Errors
/// Returns error if AI service fails to generate refactored code
#[tauri::command]
pub async fn ai_refactor_code(
    code: String,
    instruction: String,
    ai_service_state: tauri::State<'_, AIServiceState>,
) -> Result<String, String> {
    log::info!(
        "Code refactoring requested with instruction: {}",
        instruction
    );

    // Validate input
    if code.is_empty() {
        log::warn!("Empty code provided for refactoring");
        return Err("Code cannot be empty".to_string());
    }

    if instruction.is_empty() {
        log::warn!("Empty instruction provided for refactoring");
        return Err("Instruction cannot be empty".to_string());
    }

    // Get AI service from managed state
    let ai_service = utils::get_or_create_ai_service(&ai_service_state).await?;

    let ctx = rust_ai_ide_lsp::AIContext {
        current_code:    code,
        file_name:       None,
        cursor_position: None,
        selection:       None,
        project_context: Default::default(),
    };

    let task = format!(
        "Refactor the following Rust code according to this instruction: {}\n\nRespond with the fully refactored code \
         only. Ensure the code is syntactically correct and follows Rust best practices.",
        instruction
    );

    log::debug!("AI refactoring task: {}", task);

    let refactored_code = ai_service.get_task_response(ctx, task).await.map_err(|e| {
        log::error!("AI refactoring error: {}", e);
        format!("Code refactoring failed: {}", e)
    })?;

    log::info!("Code refactoring completed successfully");
    Ok(refactored_code)
}

/// Provide context-aware help and guidance
///
/// # Arguments
/// * `question` - The help question or topic
/// * `code_context` - Optional code context for more targeted help
/// * `file_name` - Optional file name for context
/// * `ai_service_state` - Managed AI service state
///
/// # Returns
/// * `Result<String, String>` - AI-generated help response or error
///
/// # Errors
/// Returns error if AI service fails or question is invalid
#[tauri::command]
pub async fn ai_context_help(
    question: String,
    code_context: Option<String>,
    file_name: Option<String>,
    ai_service_state: tauri::State<'_, AIServiceState>,
) -> Result<String, String> {
    log::info!("Context help requested: {}", question);

    // Validate input
    if question.is_empty() {
        log::warn!("Empty question provided for help");
        return Err("Question cannot be empty".to_string());
    }

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
        project_context: Default::default(),
    };

    let task = format!(
        "Question: {}\n\nProvide helpful, context-aware guidance for Rust development. Include practical examples \
         where relevant and consider best practices.",
        question
    );

    log::debug!("AI help task: {}", task);

    let help_response = ai_service.get_task_response(ctx, task).await.map_err(|e| {
        log::error!("AI help error: {}", e);
        format!("Help generation failed: {}", e)
    })?;

    log::info!("Context help provided successfully");
    Ok(help_response)
}

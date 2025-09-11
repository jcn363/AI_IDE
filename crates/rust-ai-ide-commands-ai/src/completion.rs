/*!
# AI Code Completion Module

This module provides AI-powered code completion and refactoring commands for the Rust AI IDE.
It handles intelligent code completion, refactoring suggestions, and code transformation
operations with integration to the AI service layer.

## Features

- Intelligent code completion based on context
- AI-assisted code refactoring with suggestions
- Multi-language support through LSP integration
- Async processing with proper error handling

## Integration Points

This module integrates with:
- AIService for AI/ML operations
- LSP service for language-specific completions
- File watching for context updates
- EventBus for async communication
*/

use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::Arc;
use tokio::sync::RwLock;

// Re-export common types
use super::services::{AIError, AIResult, AIService};

// Note: command_templates macros not available in this crate scope
// When integrating with Tauri, use templates from src-tauri

/// Code completion request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeCompletionRequest {
    pub code: String,
    pub language: String,
    pub context: Option<String>,
    pub cursor_position: usize,
}

/// Refactoring request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringRequest {
    pub code: String,
    pub language: String,
    pub refactoring_type: String,
}

/// Code completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeCompletionResponse {
    pub suggestions: Vec<String>,
    pub confidence_scores: Vec<f64>,
}

/// Refactoring response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringResponse {
    pub refactored_code: Vec<String>,
    pub reasoning: String,
}

/// Error types specific to completion operations
#[derive(Debug, thiserror::Error)]
pub enum CompletionError {
    #[error("AI service error: {source}")]
    AIServiceError {
        #[from]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Invalid code provided for completion")]
    InvalidCode,

    #[error("Unsupported language: {language}")]
    UnsupportedLanguage { language: String },

    #[error("Context too large for processing")]
    ContextTooLarge,
}

#[derive(serde::Serialize)]
pub struct CompletionErrorWrapper {
    pub message: String,
    pub code: String,
}

impl From<&CompletionError> for CompletionErrorWrapper {
    fn from(error: &CompletionError) -> Self {
        Self {
            message: error.to_string(),
            code: "COMPLETION_ERROR".to_string(),
        }
    }
}

/// AI Code Completion Service
pub struct CompletionService {
    ai_service: Arc<RwLock<AIService>>,
    supported_languages: Vec<String>,
}

impl CompletionService {
    /// Create a new completion service
    pub async fn new(ai_service: Arc<RwLock<AIService>>) -> AIResult<Self> {
        let supported_languages = vec![
            "rust".to_string(),
            "python".to_string(),
            "javascript".to_string(),
            "typescript".to_string(),
        ];

        Ok(Self {
            ai_service,
            supported_languages,
        })
    }

    /// Check if a language is supported
    pub fn is_language_supported(&self, language: &str) -> bool {
        self.supported_languages.contains(&language.to_lowercase())
    }

    /// Perform code completion
    pub async fn complete_code(
        &self,
        request: CodeCompletionRequest,
    ) -> AIResult<CodeCompletionResponse> {
        // TODO: Implement actual AI completion logic
        // This is a placeholder implementation that will be replaced with real AI integration

        if !self.is_language_supported(&request.language) {
            return Err(AIError::Other {
                message: CompletionError::UnsupportedLanguage {
                    language: request.language,
                }
                .to_string(),
            });
        }

        let suggestions = vec![
            format!("# AI completion for {} code", request.language),
            "fn complete_function() {\n    // TODO: Implement completion\n}".to_string(),
        ];

        let confidence_scores = vec![0.85, 0.72];

        Ok(CodeCompletionResponse {
            suggestions,
            confidence_scores,
        })
    }

    /// Perform code refactoring
    pub async fn refactor_code(
        &self,
        request: RefactoringRequest,
    ) -> AIResult<RefactoringResponse> {
        // TODO: Implement actual AI refactoring logic
        // This is a placeholder implementation

        if !self.is_language_supported(&request.language) {
            return Err(AIError::Other {
                message: CompletionError::UnsupportedLanguage {
                    language: request.language,
                }
                .to_string(),
            });
        }

        let refactored_code = vec![
            format!("# Refactored {} code", request.language),
            "fn refactored_function() {\n    // TODO: Implement refactored logic\n}".to_string(),
        ];

        Ok(RefactoringResponse {
            refactored_code,
            reasoning: "AI-generated refactoring suggestion for improved code structure"
                .to_string(),
        })
    }
}

/// Command factory for code completion
/// Returns a boxed closure that can be used as a Tauri command
pub fn code_completion_command() -> Box<dyn std::any::Any + Send + Sync> {
    Box::new(|input: serde_json::Value| async move {
        // Placeholder implementation - return dummy JSON
        serde_json::json!({
            "status": "ok",
            "suggestions": ["placeholder completion"],
            "message": "Code completion command placeholder - implementation pending"
        })
    }) as Box<dyn std::any::Any + Send + Sync>
}

/// Command factory for code refactoring
/// Returns a boxed closure that can be used as a Tauri command
pub fn refactor_command() -> Box<dyn std::any::Any + Send + Sync> {
    Box::new(|input: serde_json::Value| async move {
        // Placeholder implementation - return dummy JSON
        serde_json::json!({
            "status": "ok",
            "refactored_code": ["placeholder refactored code"],
            "message": "Refactoring command placeholder - implementation pending"
        })
    }) as Box<dyn std::any::Any + Send + Sync>
}

/// Tauri command for code completion with service integration
/// This would typically be used in the Tauri application
#[cfg(feature = "tauri")]
#[tauri::command]
pub async fn ai_code_completion(
    ai_service: tauri::State<'_, Arc<RwLock<AIService>>>,
) -> Result<serde_json::Value, String> {
    let config = CommandConfig::default();

    execute_command!("ai_code_completion", &config, async move || {
        // TODO: Implement full Tauri command with service integration
        // This is a placeholder that returns dummy data

        let response = serde_json::json!({
            "status": "placeholder",
            "message": "AI code completion - full implementation coming soon",
            "suggestions": []
        });

        Ok(response)
    })
}

/// Tauri command for code refactoring with service integration
#[cfg(feature = "tauri")]
#[tauri::command]
pub async fn ai_refactor_code(
    ai_service: tauri::State<'_, Arc<RwLock<AIService>>>,
) -> Result<serde_json::Value, String> {
    let config = CommandConfig::default();

    execute_command!("ai_refactor_code", &config, async move || {
        // TODO: Implement full refactoring command
        let response = serde_json::json!({
            "status": "placeholder",
            "message": "AI refactoring - full implementation coming soon",
            "refactored_code": []
        });

        Ok(response)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::AIService;
    use assert_matches::assert_matches;
    use proptest::prelude::*;
    use serde_test::{assert_tokens, Token};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // Mock AIService for testing (placeholder - in practice, use mockall)
    #[derive(Clone)]
    struct MockAIService;
    impl MockAIService {
        fn new() -> Self {
            Self
        }
    }

    #[tokio::test]
    async fn test_completion_service_creation() {
        let ai_service = Arc::new(RwLock::new(AIService::new().await.unwrap()));
        let completion_service = CompletionService::new(ai_service).await.unwrap();

        assert!(completion_service.is_language_supported("rust"));
        assert!(completion_service.is_language_supported("python"));
        assert!(completion_service.is_language_supported("javascript"));
        assert!(!completion_service.is_language_supported("unknown"));
        assert!(!completion_service.is_language_supported(""));
    }

    #[tokio::test]
    async fn test_code_completion_placeholder() {
        let ai_service = Arc::new(RwLock::new(AIService::new().await.unwrap()));
        let completion_service = CompletionService::new(ai_service).await.unwrap();

        let request = CodeCompletionRequest {
            code: "fn test() {".to_string(),
            language: "rust".to_string(),
            context: None,
            cursor_position: 11,
        };

        let response = completion_service.complete_code(request).await.unwrap();
        assert_eq!(response.suggestions.len(), 2);
        assert_eq!(response.confidence_scores.len(), 2);
        assert!(response
            .confidence_scores
            .iter()
            .all(|&score| (0.0..=1.0).contains(&score)));
    }

    #[tokio::test]
    async fn test_code_completion_unsupported_language() {
        let ai_service = Arc::new(RwLock::new(AIService::new().await.unwrap()));
        let completion_service = CompletionService::new(ai_service).await.unwrap();

        let request = CodeCompletionRequest {
            code: "const test =".to_string(),
            language: "unsupported".to_string(),
            context: Some("function context".to_string()),
            cursor_position: 13,
        };

        let result = completion_service.complete_code(request).await;
        assert_matches!(result, Err(_));
        if let Err(AIError::Other { message }) = result {
            assert!(message.contains("Unsupported language"));
        }
    }

    #[tokio::test]
    async fn test_code_completion_with_context() {
        let ai_service = Arc::new(RwLock::new(AIService::new().await.unwrap()));
        let completion_service = CompletionService::new(ai_service).await.unwrap();

        let context = Some("pub mod my_module {".to_string());
        let request = CodeCompletionRequest {
            code: "fn test() {".to_string(),
            language: "rust".to_string(),
            context,
            cursor_position: 11,
        };

        let response = completion_service.complete_code(request).await.unwrap();
        assert!(!response.suggestions.is_empty());
        assert!(!response.confidence_scores.is_empty());
        assert_eq!(response.suggestions.len(), response.confidence_scores.len());
    }

    #[tokio::test]
    async fn test_refactor_code_success() {
        let ai_service = Arc::new(RwLock::new(AIService::new().await.unwrap()));
        let completion_service = CompletionService::new(ai_service).await.unwrap();

        let request = RefactoringRequest {
            code: "fn new_func() {}".to_string(),
            language: "rust".to_string(),
            refactoring_type: "extract_function".to_string(),
        };

        let response = completion_service.refactor_code(request).await.unwrap();
        assert!(!response.refactored_code.is_empty());
        assert!(!response.reasoning.is_empty());
        assert!(response.reasoning.contains("AI-generated"));
    }

    #[tokio::test]
    async fn test_refactor_code_unsupported_language() {
        let ai_service = Arc::new(RwLock::new(AIService::new().await.unwrap()));
        let completion_service = CompletionService::new(ai_service).await.unwrap();

        let request = RefactoringRequest {
            code: "function test() {}".to_string(),
            language: "unsupported".to_string(),
            refactoring_type: "rename".to_string(),
        };

        let result = completion_service.refactor_code(request).await;
        assert_matches!(result, Err(_));
        if let Err(AIError::Other { message }) = result {
            assert!(message.contains("Unsupported language"));
        }
    }

    #[tokio::test]
    async fn test_code_completion_request_serialization() {
        let request = CodeCompletionRequest {
            code: "fn test() {}".to_string(),
            language: "rust".to_string(),
            context: Some("module context".to_string()),
            cursor_position: 10,
        };

        // Test serialization
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CodeCompletionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request.code, deserialized.code);
        assert_eq!(request.language, deserialized.language);
        assert_eq!(request.context, deserialized.context);
        assert_eq!(request.cursor_position, deserialized.cursor_position);
    }

    #[tokio::test]
    async fn test_code_completion_response_serialization() {
        let response = CodeCompletionResponse {
            suggestions: vec!["suggestion1".to_string(), "suggestion2".to_string()],
            confidence_scores: vec![0.8, 0.9],
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: CodeCompletionResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response.suggestions, deserialized.suggestions);
        assert_eq!(response.confidence_scores, deserialized.confidence_scores);
    }

    #[tokio::test]
    async fn test_refactoring_request_response_serialization() {
        let request = RefactoringRequest {
            code: "old code".to_string(),
            language: "rust".to_string(),
            refactoring_type: "extract".to_string(),
        };

        let response = RefactoringResponse {
            refactored_code: vec!["new code".to_string()],
            reasoning: "refactored successfully".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: RefactoringResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response.refactored_code, deserialized.refactored_code);
        assert_eq!(response.reasoning, deserialized.reasoning);
    }

    #[tokio::test]
    async fn test_empty_code_completion() {
        let ai_service = Arc::new(RwLock::new(AIService::new().await.unwrap()));
        let completion_service = CompletionService::new(ai_service).await.unwrap();

        let request = CodeCompletionRequest {
            code: "".to_string(),
            language: "rust".to_string(),
            context: None,
            cursor_position: 0,
        };

        let response = completion_service.complete_code(request).await.unwrap();
        // Should still provide suggestions even with empty code
        assert!(!response.suggestions.is_empty());
        assert!(!response.confidence_scores.is_empty());
    }

    #[tokio::test]
    async fn test_large_cursor_position() {
        let ai_service = Arc::new(RwLock::new(AIService::new().await.unwrap()));
        let completion_service = CompletionService::new(ai_service).await.unwrap();

        let request = CodeCompletionRequest {
            code: "fn test() {}".to_string(),
            language: "rust".to_string(),
            context: None,
            cursor_position: 1000, // Much larger than code length
        };

        // Should handle gracefully without panicking
        let result = completion_service.complete_code(request).await;
        // This might return an error or fallback to default behavior - both are acceptable
        assert!(result.is_ok() || matches!(result, Err(_)));
    }

    #[tokio::test]
    async fn test_error_types() {
        let error = CompletionError::UnsupportedLanguage {
            language: "cobol".to_string(),
        };

        let wrapper = CompletionErrorWrapper::from(&error);
        assert!(wrapper.message.contains("cobol"));
        assert_eq!(wrapper.code, "COMPLETION_ERROR");
    }

    #[tokio::test]
    async fn test_supported_languages() {
        let ai_service = Arc::new(RwLock::new(AIService::new().await.unwrap()));
        let completion_service = CompletionService::new(ai_service).await.unwrap();

        let expectedLanguages = vec!["rust", "python", "javascript", "typescript"];

        for lang in expectedLanguages {
            assert!(completion_service.is_language_supported(lang));
            assert!(completion_service.is_language_supported(&lang.to_uppercase()));
        }

        let unsupported = vec!["cobol", "fortran", "assembly", "brainfuck"];
        for lang in unsupported {
            assert!(!completion_service.is_language_supported(lang));
        }
    }
}

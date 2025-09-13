# Command Macro Examples

This document provides comprehensive examples of the complex macro system used throughout the Rust AI IDE project, focusing on command templates, validation macros, and architectural patterns.

## Table of Contents

- [Tauri Command Templates](#tauri-command-templates)
- [Validation Macros](#validation-macros)
- [Background Task Macros](#background-task-macros)
- [Error Handling Macros](#error-handling-macros)
- [Advanced Macro Patterns](#advanced-macro-patterns)
- [Macro Composition and Reuse](#macro-composition-and-reuse)
- [Debugging and Testing Macros](#debugging-and-testing-macros)

## Tauri Command Templates

### Basic Command Template

```rust
// Using the standard tauri_command_template! macro
tauri_command_template! {
    get_project_info,
    |request: GetProjectInfoRequest, state: AppState| async move {
        // Command logic here
        let project = state.project_service.get_project_info(&request.project_id).await?;
        Ok(ProjectInfoResponse {
            id: project.id,
            name: project.name,
            path: project.path,
            language: project.language,
            last_modified: project.last_modified,
        })
    }
}

// Generated code equivalent:
#[tauri::command]
pub async fn get_project_info(
    request: GetProjectInfoRequest,
    state: tauri::State<AppState>,
) -> Result<ProjectInfoResponse, Error> {
    sanitize_and_validate_command!(request, "get_project_info");

    let project = state.project_service.get_project_info(&request.project_id).await?;
    Ok(ProjectInfoResponse {
        id: project.id,
        name: project.name,
        path: project.path,
        language: project.language,
        last_modified: project.last_modified,
    })
}
```

### Advanced Command Template with Custom Logic

```rust
// Command template with complex business logic
tauri_command_template! {
    analyze_and_refactor_code,
    |request: CodeAnalysisRequest, state: AppState| async move {
        // Multi-step analysis process
        let analysis_result = state.ai_service.analyze_code(&request.code).await?;

        // Generate refactoring suggestions
        let refactoring_options = state.refactoring_service
            .generate_suggestions(&analysis_result)
            .await?;

        // Apply selected refactoring if auto-apply is enabled
        let applied_changes = if request.auto_apply {
            state.refactoring_service
                .apply_refactoring(&request.file_path, &refactoring_options[0])
                .await?
        } else {
            vec![]
        };

        Ok(RefactoringResponse {
            analysis: analysis_result,
            suggestions: refactoring_options,
            applied_changes,
            success: true,
        })
    }
}
```

### Service Acquisition Command Template

```rust
// Using acquire_service_and_execute! for service management
tauri_command_template! {
    process_ai_request,
    |request: AIProcessingRequest, state: AppState| async move {
        acquire_service_and_execute! {
            state.ai_service,
            |ai_service| async move {
                // Service is guaranteed to be available here
                let result = ai_service.process_request(&request).await?;

                // Additional processing
                let formatted_result = format_ai_response(result)?;
                let cached_result = cache_response(&request, &formatted_result).await?;

                Ok(AIProcessingResponse {
                    result: formatted_result,
                    cached: cached_result,
                    processing_time_ms: 0, // Would be calculated
                })
            }
        }
    }
}
```

## Validation Macros

### Input Validation Macro

```rust
// Custom validation macro for file operations
macro_rules! validate_file_operation {
    ($path:expr, $operation:expr) => {{
        // Path validation
        let validated_path = $crate::validate_secure_path($path, $operation)?;

        // File existence check
        if !$crate::fs::exists(&validated_path).await {
            return Err($crate::Error::FileNotFound(validated_path.to_string()));
        }

        // Permission check
        let permissions = $crate::fs::get_permissions(&validated_path).await?;
        if !permissions.readable() {
            return Err($crate::Error::PermissionDenied(validated_path.to_string()));
        }

        validated_path
    }};
}

// Usage in command
tauri_command_template! {
    read_secure_file,
    |request: FileReadRequest, _state: AppState| async move {
        let secure_path = validate_file_operation!(&request.file_path, "file_read")?;

        let content = tokio::fs::read_to_string(&secure_path).await?;
        let metadata = tokio::fs::metadata(&secure_path).await?;

        Ok(FileReadResponse {
            content,
            size: metadata.len(),
            last_modified: metadata.modified()?,
        })
    }
}
```

### Complex Validation Macro with Custom Rules

```rust
// Advanced validation macro with custom rules
macro_rules! validate_with_custom_rules {
    ($input:expr, $($rule:expr => $validator:expr),*) => {{
        let mut errors = Vec::new();

        $(
            if let Err(error) = $validator($input) {
                errors.push(format!("{}: {}", $rule, error));
            }
        )*

        if !errors.is_empty() {
            return Err($crate::Error::ValidationError(errors.join("; ")));
        }
    }};
}

// Custom validation functions
fn validate_project_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Project name cannot be empty".to_string());
    }
    if name.len() > 100 {
        return Err("Project name too long".to_string());
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err("Invalid characters in project name".to_string());
    }
    Ok(())
}

fn validate_project_path(path: &str) -> Result<(), String> {
    if !path.starts_with('/') {
        return Err("Project path must be absolute".to_string());
    }
    if path.contains("..") {
        return Err("Project path cannot contain '..'".to_string());
    }
    Ok(())
}

// Usage
tauri_command_template! {
    create_project,
    |request: CreateProjectRequest, state: AppState| async move {
        // Apply custom validation rules
        validate_with_custom_rules! {
            &request.name,
            "project_name" => validate_project_name,
            "project_path" => validate_project_path
        }

        // Additional validation
        if state.project_service.project_exists(&request.name).await? {
            return Err(Error::ProjectAlreadyExists(request.name));
        }

        // Create project
        let project = state.project_service.create_project(request).await?;

        Ok(CreateProjectResponse {
            project_id: project.id,
            success: true,
        })
    }
}
```

## Background Task Macros

### Spawn Background Task with Monitoring

```rust
// Background task macro with monitoring
macro_rules! spawn_monitored_task {
    ($task_name:expr, $task:expr, $monitor:expr) => {{
        use std::sync::atomic::{AtomicBool, Ordering};
        use tokio::task::JoinHandle;

        static TASK_RUNNING: AtomicBool = AtomicBool::new(false);

        if TASK_RUNNING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
            let task_name_clone = $task_name.to_string();
            let monitor_clone = $monitor.clone();

            let handle: JoinHandle<()> = tokio::spawn(async move {
                let start_time = std::time::Instant::now();

                // Notify monitor of task start
                monitor_clone.task_started(&task_name_clone).await;

                match $task.await {
                    Ok(_) => {
                        let duration = start_time.elapsed();
                        monitor_clone.task_completed(&task_name_clone, duration).await;
                        log::info!("Background task '{}' completed in {:?}", task_name_clone, duration);
                    },
                    Err(e) => {
                        let duration = start_time.elapsed();
                        monitor_clone.task_failed(&task_name_clone, &e, duration).await;
                        log::error!("Background task '{}' failed after {:?}: {}", task_name_clone, duration, e);
                    }
                }

                TASK_RUNNING.store(false, Ordering::SeqCst);
            });

            Some(handle)
        } else {
            log::warn!("Background task '{}' already running", $task_name);
            None
        }
    }};
}

// Usage with monitoring
tauri_command_template! {
    start_ai_analysis_task,
    |request: AIAnalysisRequest, state: AppState| async move {
        let task_name = format!("ai_analysis_{}", request.project_id);
        let monitor = state.task_monitor.clone();

        let handle = spawn_monitored_task! {
            &task_name,
            async {
                // Complex AI analysis task
                let analysis = state.ai_service.analyze_project(&request.project_id).await?;
                let report = state.report_service.generate_report(&analysis).await?;
                state.storage_service.save_report(&report).await?;
                Ok(())
            },
            monitor
        };

        Ok(StartTaskResponse {
            task_id: task_name,
            started: handle.is_some(),
        })
    }
}
```

### Task Chain Macro

```rust
// Macro for chaining background tasks
macro_rules! spawn_task_chain {
    ($tasks:expr) => {{
        use tokio::task::JoinHandle;

        let mut handles = Vec::new();

        for (i, task) in $tasks.into_iter().enumerate() {
            let task_name = format!("chain_task_{}", i);
            let handle = tokio::spawn(async move {
                log::info!("Starting task in chain: {}", task_name);
                let result = task().await;
                log::info!("Completed task in chain: {}", task_name);
                result
            });
            handles.push(handle);
        }

        handles
    }};
}

// Usage for complex workflows
tauri_command_template! {
    run_deployment_pipeline,
    |request: DeploymentRequest, state: AppState| async move {
        let tasks = vec![
            // Task 1: Validate deployment configuration
            || async {
                state.validation_service.validate_config(&request.config).await
            },

            // Task 2: Build application
            || async {
                state.build_service.build_project(&request.project_id).await
            },

            // Task 3: Run tests
            || async {
                state.test_service.run_test_suite(&request.test_suite).await
            },

            // Task 4: Deploy to staging
            || async {
                state.deployment_service.deploy_to_staging(&request).await
            },

            // Task 5: Run integration tests
            || async {
                state.test_service.run_integration_tests().await
            },

            // Task 6: Deploy to production
            || async {
                state.deployment_service.deploy_to_production(&request).await
            },
        ];

        let handles = spawn_task_chain!(tasks);

        // Wait for all tasks to complete
        let results = futures::future::join_all(handles).await;

        // Check results
        let mut success = true;
        for result in results {
            if let Err(e) = result? {
                log::error!("Deployment task failed: {}", e);
                success = false;
            }
        }

        Ok(DeploymentResponse {
            success,
            deployment_id: format!("deploy_{}", request.project_id),
        })
    }
}
```

## Error Handling Macros

### Retry with Exponential Backoff

```rust
// Macro for retry logic with exponential backoff
macro_rules! retry_with_backoff {
    ($operation:expr, $max_attempts:expr, $base_delay:expr) => {{
        let mut attempt = 0;
        let mut delay = $base_delay;

        loop {
            attempt += 1;

            match $operation.await {
                Ok(result) => break Ok(result),
                Err(e) => {
                    if attempt >= $max_attempts {
                        break Err(e);
                    }

                    log::warn!("Operation failed (attempt {}): {}. Retrying in {:?}", attempt, e, delay);
                    tokio::time::sleep(delay).await;

                    // Exponential backoff
                    delay = delay.saturating_mul(2);
                }
            }
        }
    }};
}

// Usage in commands
tauri_command_template! {
    sync_with_remote_repository,
    |request: SyncRequest, state: AppState| async move {
        let result = retry_with_backoff! {
            state.git_service.sync_repository(&request.repo_url),
            3,  // max attempts
            std::time::Duration::from_secs(1)  // base delay
        };

        match result {
            Ok(sync_result) => Ok(SyncResponse {
                success: true,
                synced_files: sync_result.files_changed,
                sync_time: sync_result.duration,
            }),
            Err(e) => Ok(SyncResponse {
                success: false,
                error_message: Some(e.to_string()),
                synced_files: 0,
                sync_time: std::time::Duration::from_secs(0),
            })
        }
    }
}
```

### Error Context Propagation

```rust
// Macro for adding context to errors
macro_rules! with_error_context {
    ($operation:expr, $context:expr) => {{
        match $operation {
            Ok(result) => Ok(result),
            Err(e) => {
                log::error!("Operation failed in {}: {}", $context, e);
                Err($crate::Error::ContextualError {
                    context: $context.to_string(),
                    source: Box::new(e),
                })
            }
        }
    }};
}

// Usage for better error reporting
tauri_command_template! {
    complex_data_processing,
    |request: DataProcessingRequest, state: AppState| async move {
        // Step 1: Load data
        let raw_data = with_error_context! {
            state.data_service.load_data(&request.data_source).await,
            "loading data from source"
        }?;

        // Step 2: Validate data
        let validated_data = with_error_context! {
            state.validation_service.validate_data(raw_data).await,
            "validating loaded data"
        }?;

        // Step 3: Process data
        let processed_data = with_error_context! {
            state.processing_service.process_data(validated_data).await,
            "processing validated data"
        }?;

        // Step 4: Store results
        let stored_result = with_error_context! {
            state.storage_service.store_result(&processed_data).await,
            "storing processing results"
        }?;

        Ok(DataProcessingResponse {
            success: true,
            result_id: stored_result.id,
            processing_time: stored_result.duration,
        })
    }
}
```

## Advanced Macro Patterns

### Conditional Compilation Macros

```rust
// Macro for feature-gated functionality
macro_rules! feature_gated_command {
    ($feature:literal, $command:expr) => {{
        #[cfg(feature = $feature)]
        {
            $command
        }

        #[cfg(not(feature = $feature))]
        {
            log::warn!("Feature '{}' not enabled, command not available", $feature);
            Ok(serde_json::json!({"error": format!("Feature '{}' not enabled", $feature)}))
        }
    }};
}

// Usage for optional features
tauri_command_template! {
    advanced_ai_features,
    |_request: AdvancedAIRequest, state: AppState| async move {
        feature_gated_command! {
            "ai-advanced",
            {
                // Advanced AI processing only available with feature flag
                let result = state.advanced_ai_service.process_advanced_request(_request).await?;
                Ok(AdvancedAIResponse {
                    result: result.data,
                    confidence: result.confidence,
                    processing_time: result.processing_time,
                })
            }
        }
    }
}
```

### Performance Monitoring Macros

```rust
// Macro for performance monitoring
macro_rules! with_performance_monitoring {
    ($operation_name:expr, $operation:expr) => {{
        let start_time = std::time::Instant::now();

        // Record operation start
        $crate::performance_monitor::record_operation_start($operation_name).await;

        let result = $operation.await;

        let duration = start_time.elapsed();

        // Record operation completion
        $crate::performance_monitor::record_operation_complete(
            $operation_name,
            duration,
            result.is_ok()
        ).await;

        // Log slow operations
        if duration > std::time::Duration::from_secs(5) {
            log::warn!("Slow operation '{}' took {:?}", $operation_name, duration);
        }

        result
    }};
}

// Usage in performance-critical commands
tauri_command_template! {
    intensive_computation,
    |request: ComputationRequest, state: AppState| async move {
        let result = with_performance_monitoring! {
            "intensive_computation",
            state.computation_service.perform_computation(&request)
        };

        match result {
            Ok(computation_result) => Ok(ComputationResponse {
                result: computation_result.value,
                computation_time: computation_result.duration,
                success: true,
            }),
            Err(e) => {
                log::error!("Computation failed: {}", e);
                Ok(ComputationResponse {
                    result: 0.0,
                    computation_time: std::time::Duration::from_secs(0),
                    success: false,
                })
            }
        }
    }
}
```

## Macro Composition and Reuse

### Composable Command Builder

```rust
// Base command builder macro
macro_rules! command_builder {
    ($name:ident, $input:ty, $output:ty, $logic:expr) => {
        tauri_command_template! {
            $name,
            |request: $input, state: AppState| async move {
                // Common pre-processing
                sanitize_and_validate_command!(request, stringify!($name));

                // Execute main logic
                let result = $logic(request, state).await?;

                // Common post-processing
                log::info!("Command '{}' completed successfully", stringify!($name));

                Ok(result)
            }
        }
    };
}

// Specialized command builders
macro_rules! ai_command_builder {
    ($name:ident, $input:ty, $output:ty, $ai_logic:expr) => {
        command_builder! {
            $name,
            $input,
            $output,
            |request, state| async move {
                // AI-specific pre-processing
                if !state.ai_service.is_available().await {
                    return Err(Error::AIServiceUnavailable);
                }

                // Execute AI logic with monitoring
                let result = with_performance_monitoring! {
                    stringify!($name),
                    $ai_logic(request, state)
                };

                result
            }
        }
    };
}

// Usage of composed macros
ai_command_builder! {
    generate_code_suggestions,
    CodeSuggestionRequest,
    CodeSuggestionResponse,
    |request, state| async move {
        let suggestions = state.ai_service.generate_suggestions(&request.code).await?;
        Ok(CodeSuggestionResponse {
            suggestions,
            total_count: suggestions.len(),
        })
    }
}
```

## Debugging and Testing Macros

### Debug Logging Macro

```rust
// Enhanced debug logging macro
macro_rules! debug_command_execution {
    ($command_name:expr, $request:expr, $result:expr) => {{
        log::debug!(
            "Command '{}' executed - Input: {:?}, Success: {}",
            $command_name,
            $request,
            $result.is_ok()
        );

        // Log performance metrics in debug mode
        #[cfg(debug_assertions)]
        {
            use std::time::Instant;
            let _start = Instant::now();
            // Additional debug logging
        }
    }};
}

// Usage in commands
tauri_command_template! {
    debug_example_command,
    |request: DebugRequest, state: AppState| async move {
        let result = state.debug_service.process_request(&request).await;

        // Debug logging
        debug_command_execution!("debug_example_command", request, &result);

        result
    }
}
```

### Test Helper Macros

```rust
// Macro for testing command responses
#[cfg(test)]
macro_rules! assert_command_success {
    ($result:expr) => {{
        match $result {
            Ok(response) => {
                assert!(response.success, "Command did not succeed: {:?}", response);
                response
            },
            Err(e) => panic!("Command failed with error: {}", e),
        }
    }};
}

#[cfg(test)]
macro_rules! assert_command_error {
    ($result:expr, $expected_error:pat) => {{
        match $result {
            Err($expected_error) => {},
            Err(other) => panic!("Expected error {:?}, got {:?}", stringify!($expected_error), other),
            Ok(response) => panic!("Expected error {:?}, got success: {:?}", stringify!($expected_error), response),
        }
    }};
}

// Usage in tests
#[cfg(test)]
mod tests {
    use super::*;
    use assert_command_success;
    use assert_command_error;

    #[tokio::test]
    async fn test_successful_command() {
        let request = TestRequest { data: "test".to_string() };
        let result = process_test_command(request).await;

        let response = assert_command_success!(result);
        assert_eq!(response.processed_data, "TEST");
    }

    #[tokio::test]
    async fn test_command_with_invalid_input() {
        let request = TestRequest { data: "".to_string() };
        let result = process_test_command(request).await;

        assert_command_error!(result, Error::InvalidInput(_));
    }
}
```

This comprehensive guide demonstrates the powerful macro system used throughout the Rust AI IDE project. The examples show how macros enable:

1. **Consistent command handling** with standardized validation and error handling
2. **Complex business logic** encapsulation with reusable patterns
3. **Performance monitoring** and debugging capabilities
4. **Testability** through helper macros and assertions
5. **Maintainability** through composable and reusable macro patterns

The macro system significantly reduces boilerplate code while ensuring consistency and reliability across the entire codebase.
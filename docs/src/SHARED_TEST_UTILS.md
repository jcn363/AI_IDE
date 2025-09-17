# 🧪 Shared Test Utils

A comprehensive testing utilities library for the Rust AI IDE workspace, providing consistent patterns and tools for writing maintainable tests.

## Overview

The **shared-test-utils** crate provides a complete testing infrastructure designed specifically for the complex requirements of the Rust AI IDE project. It eliminates boilerplate code, provides consistent testing patterns, and enables robust testing across all workspace components.

## Features

### 🗂️ **TempWorkspace**

Managed temporary directories with automatic cleanup for safe testing.

```rust
use shared_test_utils::TempWorkspace;

// Create a workspace that automatically cleans up
let workspace = TempWorkspace::new().unwrap();

// Perform file operations
workspace.create_file("config.toml", "content").unwrap();
workspace.create_dir("src").unwrap();

// Automatic cleanup when workspace goes out of scope
```

### 🔧 **Async Testing Utilities**

Timeout handling and concurrent task execution for async operations.

```rust
use shared_test_utils::{with_timeout, run_concurrent};
use std::time::Duration;

// Timeout protection for async operations
#[tokio::test]
async fn test_async_operation() {
    let result = with_timeout(async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        "operation_success"
    }, Duration::from_millis(200)).await;

    assert!(result.is_ok());
}
```

### 🏗️ **Test Fixtures**

Pre-configured test scenarios for common patterns.

```rust
use shared_test_utils::fixtures::FixturePresets;

// Create common Rust project structures
#[test]
fn test_rust_project() {
    let (workspace, fixture) = FixturePresets::rust_library().build(&TempWorkspace::new().unwrap()).unwrap();

    // Validate project structure
    assert!(workspace.file_exists("Cargo.toml"));
    assert!(workspace.file_exists("src/lib.rs"));
}
```

### 🎯 **Tauri Command Testing**

Mock framework for testing Tauri commands and IPC communication.

```rust
use shared_test_utils::command_tests::{CommandTestRunner, MockCommand};

// Test Tauri command interactions
#[test]
fn test_tauri_commands() {
    let mut runner = CommandTestRunner::new();
    runner.register_command(MockCommand::new("analyze", serde_json::json!({"data": "test"})));

    // Simulate command execution and validation
    assert_eq!(runner.called_commands().len(), 0);
}
```

### ✅ **Validation Utilities**

Content and path validation with security checks.

```rust
use shared_test_utils::ValidationUtils;

// Validate content patterns
assert!(ValidationUtils::validate_content("Hello World", &["Hello"]).is_ok());

// Test setup validation
let components = vec![Some("feature1"), Some("feature2")];
assert!(ValidationUtils::validate_test_setup(&components, &["Feature1", "Feature2"]).is_ok());
```

### 🎛️ **Integration Context**

State management and resource lifecycle handling for complex test scenarios.

```rust
use shared_test_utils::integration::IntegrationContext;

#[test]
fn test_integration_scenario() {
    // Setup integration context
    let mut context = IntegrationContext::default();

    // Store and retrieve state
    context.store_state("config", "test_value").unwrap();
    let retrieved: String = context.get_state("config").unwrap();
    assert_eq!(retrieved, "test_value");
}
```

## Installation

Add to any workspace crate's `Cargo.toml`:

```toml
[dev-dependencies]
shared-test-utils = { path = "../shared-test-utils" }
```

## Usage Patterns

### Basic Setup

```rust
use shared_test_utils::*;

// Simple workspace test
#[test]
fn basic_workspace_test() {
    let workspace = TempWorkspace::new().unwrap();
    workspace.create_file("data.txt", "test content").unwrap();

    assert!(workspace.file_exists("data.txt"));
    let content = workspace.read_file("data.txt").unwrap();
    assert_eq!(content, "test content");

    // Automatic cleanup
}
```

### Async Integration Testing

```rust
#[tokio::test]
async fn async_integration_test() {
    // Test with timeout protection
    let result = with_timeout(async {
        // Simulate async operation
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok("success")
    }, Duration::from_millis(500)).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}
```

### Complex Test Scenarios

```rust
#[test]
fn complex_test_scenario() {
    // Multiple fixtures and validation
    let (workspace, fixture) = FixturePresets::multi_module()
        .with_file("custom.txt", "custom content")
        .build(&TempWorkspace::new().unwrap()).unwrap();

    // Can validate multiple files and directories
    assert!(workspace.file_exists("src/lib.rs"));
    assert!(workspace.file_exists("src/math.rs"));
    assert!(workspace.file_exists("src/utils.rs"));
    assert!(workspace.file_exists("custom.txt"));

    // Content validation
    let custom_content = workspace.read_file("custom.txt").unwrap();
    assert_eq!(custom_content, "custom content");
}
```

## Module Reference

### Core Modules

#### `TempWorkspace`

- **Purpose**: Managed temporary directories
- **Features**: Auto-cleanup, file operations, directory management
- **Key Methods**: `new()`, `create_file()`, `read_file()`, `file_exists()`

#### `ValidationUtils`

- **Purpose**: Content and path validation
- **Features**: Pattern matching, security checks, test setup validation
- **Key Methods**: `validate_content()`, `validate_path_security()`, `validate_test_setup()`

#### `AsyncContext`

- **Purpose**: Async operation management
- **Features**: Timeout handling, concurrent execution
- **Key Methods**: `execute()`, `execute_concurrent()`

### Specialized Modules

#### `fixtures::FixturePresets`

- **Purpose**: Pre-configured test scenarios
- **Available Fixtures**:
  - `rust_library()` - Standard Rust library project
  - `cargo_workspace()` - Cargo workspace with multiple members
  - `multi_module()` - Multi-module Rust project
  - `json_config()` - JSON configuration file setup

#### `command_tests::*`

- **Purpose**: Tauri command testing
- **Components**: CommandTestRunner, MockCommand, CommandTestBuilder
- **Usage**: Mock Tauri commands for integration testing

#### `integration::*`

- **Purpose**: Integration test orchestration
- **Components**: IntegrationContext, IntegrationTestRunner
- **Usage**: Complex test setup and state management

#### `macros::*`

- **Purpose**: Convenient testing macros
- **Available Macros**:
  - `setup_test_workspace!()` - Create workspace with scenario
  - `with_test_fixture!()` - Setup fixture-based tests
  - `assert_test_file_exists!()` - File existence assertions
  - `assert_file_contains!()` - Content validation assertions

## Architecture Principles

### 1. **Consistency**

- Standard patterns across all workspace tests
- Predictable API design
- Consistent error handling

### 2. **Safety**

- Automatic resource cleanup
- Secure path validation
- Safe concurrent operations

### 3. **Extensibility**

- Modular design
- Easy addition of new fixtures
- Plugin-like macro system

### 4. **Performance**

- Efficient temporary directory management
- Optimized async operations
- Minimal overhead in test execution

## Testing Best Practices

### Workspace Management

```rust
// ✅ Good: Let TempWorkspace handle cleanup
let workspace = TempWorkspace::new().unwrap();
// ... use workspace
// Automatic cleanup when dropped
```

### Async Testing

```rust
// ✅ Good: Always use timeout protection for async tests
#[tokio::test]
async fn async_test() {
    let context = AsyncContext::with_timeout(Duration::from_secs(10));
    let result = context.execute(my_async_function()).await;
    assert!(result.is_ok());
}
```

### Error Handling

```rust
// ✅ Good: Use TestResult trait for consistent error handling
use shared_test_utils::error::TestResult;

#[test]
fn test_with_proper_errors() {
    let result = some_operation().expect_test("Operation should succeed");
}
```

### Fixture Usage

```rust
// ✅ Good: Start with pre-configured scenarios
#[test]
fn test_specific_scenario() {
    let (workspace, fixture) = FixturePresets::rust_library().build(&workspace).unwrap();

    // Add specific customizations for your test
    workspace.create_file("src/custom.rs", "fn custom() {}").unwrap();

    // Test your specific scenario
    assert!(workspace.file_exists("src/custom.rs"));
}
```

## Integration Examples

### Real-world Test Case: Cargo Command Testing

```rust
#[test]
fn test_cargo_command_integration() {
    let mut runner = CommandTestBuilder::new()
        .success_command("cargo_check",
                        serde_json::json!({"project_path": "/test/project"}),
                        serde_json::json!({"success": true}))
        .error_command("cargo_publish",
                      serde_json::json!({}),
                      "Configuration error")
        .build_runner();

    // Test successful cargo operation
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        runner.execute_command("cargo_check",
                             &serde_json::json!({"project_path": "/test/project"})).await
    });

    assert!(result.is_ok());

    // Test error handling
    let error_result = rt.block_on(async {
        runner.execute_command("cargo_publish", &serde_json::json!({})).await
    });

    assert!(error_result.is_err());
}
```

### Integration Test with File Operations

```rust
#[tokio::test]
async fn test_file_async_integration() {
    let workspace = TempWorkspace::new().unwrap();

    // Create complex file structure
    workspace.create_dir("integration_test").unwrap();
    workspace.create_file("integration_test/config.json", r#"{
        "test_mode": "async",
        "timeout_ms": 500
    }"#).unwrap();

    // Validate setup with async timeout
    let result = with_timeout(async {
        // Simulate async file processing
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify file content
        let content = workspace.read_file("integration_test/config.json").unwrap();
        serde_json::from_str::<serde_json::Value>(&content).unwrap()
    }, Duration::from_millis(300)).await;

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config["test_mode"], "async");
}
```

## Migration Guide

### For Existing Tests

1. Add `shared-test-utils` to dev-dependencies
2. Replace manual temp directory creation with `TempWorkspace::new()`
3. Use fixture presets instead of manual project setup
4. Replace custom async timeout logic with `with_timeout()`

### For New Tests

1. Start with appropriate fixture preset
2. Use `TempWorkspace` for all file operations
3. Apply timeout protection for async operations
4. Use validation utilities for setup verification

## Troubleshooting

### Common Issues

**Issue**: TempWorkspace not cleaning up

```rust
// Solution: Don't store reference long-term
let workspace = TempWorkspace::new().unwrap();
// Use within same scope
// Automatic cleanup when function returns
```

**Issue**: Async test timeouts

```rust
// Solution: Increase timeout or optimize async operation
let context = AsyncContext::with_timeout(Duration::from_secs(30));
let result = context.execute(long_running_operation()).await;
```

**Issue**: Fixture conflicts

```rust
// Solution: Use unique filenames or separate workspaces
let workspace1 = TempWorkspace::new().unwrap();
let workspace2 = TempWorkspace::new().unwrap();
// Different temp directories
```

## Contributing

When adding new utilities:

1. Follow existing module structure
2. Include comprehensive documentation
3. Add usage examples
4. Ensure test coverage above 90%
5. Update this documentation

## Version Compatibility

- **Rust**: 1.75+
- **Tokio**: 1.0+
- **Serde**: 1.0+

Compatible with all Rust AI IDE workspace crates.

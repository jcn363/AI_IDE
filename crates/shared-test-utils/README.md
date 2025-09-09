# Shared Test Utils

A comprehensive testing utilities library for the Rust AI IDE workspace, providing consistent patterns and tools for writing maintainable tests.

## Features

### 🗂️ **TempWorkspace**

Managed temporary directories with automatic cleanup for safe testing.

```rust
use shared_test_utils::TempWorkspace;

#[test]
fn test_file_operations() {
    let workspace = TempWorkspace::new().unwrap();

    // Create files and directories
    workspace.create_dir(std::path::Path::new("src")).unwrap();
    workspace.create_file(std::path::Path::new("test.txt"), "data").unwrap();

    // Verify operations
    assert!(workspace.file_exists(std::path::Path::new("test.txt")));
    let content = workspace.read_file(std::path::Path::new("test.txt")).unwrap();
    assert_eq!(content, "data");

    // Automatic cleanup on drop
}
```

### 🔧 **Async Testing Utilities**

Timeout handling and concurrent task execution for async operations.

```rust
use shared_test_utils::{with_timeout, AsyncContext};
use std::time::Duration;

#[tokio::test]
async fn test_async_operations() {
    // Timeout wrapper
    let result = with_timeout(async {
        tokio::time::sleep(Duration::from_millis(10)).await;
        "completed"
    }, Duration::from_millis(100)).await;

    assert!(result.is_ok());

    // Async context with consistent timeout management
    let context = AsyncContext::with_timeout(Duration::from_secs(5));
    let operation = context.execute(async {
        // Your async test operation here
        "success"
    }).await;

    assert!(operation.is_ok());
}
```

### 🏗️ **Test Fixtures**

Pre-configured test setups for common scenarios.

```rust
use shared_test_utils::fixtures::FixturePresets;

#[test]
fn test_rust_library() {
    let (workspace, fixture) = FixturePresets::rust_library().build(&TempWorkspace::new().unwrap()).unwrap();

    // Validates standard Rust library structure
    assert!(workspace.file_exists(Path::new("Cargo.toml")));
    assert!(workspace.file_exists(Path::new("src/lib.rs")));
}
```

### 🎯 **Tauri Command Testing**

Mock framework for testing Tauri commands.

```rust
use shared_test_utils::command_tests::{CommandTestBuilder, MockCommand};

#[test]
fn test_command_interactions() {
    let mut runner = CommandTestBuilder::new()
        .success_command("analyze", serde_json::json!({"data": "test"}), serde_json::json!({"result": "ok"}))
        .error_command("fail_command", serde_json::json!({}), "Error occurred")
        .build_runner();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        runner.execute_command("analyze", &serde_json::json!({"data": "test"})).await
    });

    assert!(result.is_ok());
}
```

## Integration Guide

### Adding shared-test-utils to Your Crate

1. **Update Cargo.toml dev-dependencies:**

```toml
[dev-dependencies]
shared-test-utils = { path = "../shared-test-utils" }
```

2. **Import and use in tests:**

```rust
use shared_test_utils::*;
```

### Integration Examples

#### **rust-ai-ide-core Integration**

```rust
// In your crate's tests
#[test]
fn test_core_with_shared_utils() {
    // Use TempWorkspace for isolated testing
    let workspace = TempWorkspace::new().unwrap();

    // Use validation utilities
    assert!(ValidationUtils::validate_content("test data", &["test"]).is_ok());

    // Use async utilities for timeout management
    let context = AsyncContext::with_timeout(Duration::from_secs(30));
    // ... test logic
}
```

#### **Testing Patterns**

**File System Testing:**

```rust
#[test]
fn test_project_creation() {
    let workspace = shared_test_utils::TempWorkspace::new().unwrap();
    workspace.create_dir(std::path::Path::new("project")).unwrap();

    assert!(workspace.file_exists(std::path::Path::new("project")));
}
```

**Async Operation Testing:**

```rust
#[tokio::test]
async fn test_async_workflow() {
    let result = shared_test_utils::with_timeout(async {
        // Simulate async operation
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok("success")
    }, Duration::from_millis(200)).await;

    assert!(result.is_ok());
}
```

**Fixture-Based Testing:**

```rust
#[test]
fn test_with_cargo_workspace() {
    let (workspace, _) = FixturePresets::cargo_workspace(&["app", "lib"])
        .build(&TempWorkspace::new().unwrap()).unwrap();

    // Workspace structure is automatically set up
    assert!(workspace.file_exists(Path::new("Cargo.toml")));
    assert!(workspace.file_exists(Path::new("app/Cargo.toml")));
    assert!(workspace.file_exists(Path::new("lib/src/lib.rs")));
}
```

## Available Modules

### Core Modules

- **`TempWorkspace`** - Managed temporary directories
- **`ValidationUtils`** - Content and path validation
- **`AsyncContext`** - Async operation management
- **`AsyncTestHelper`** - Async test utilities

### Specialized Modules

- **`fixtures::FixturePresets`** - Pre-configured test scenarios
- **`command_tests::*`** - Tauri command testing utilities
- **`integration::*`** - Integration testing context
- **`macros::*`** - Convenient testing macros

## Best Practices

### 1. **Workspace Management**

```rust
#[test]
fn my_test() {
    let workspace = TempWorkspace::new().unwrap();
    // Workspace automatically cleans up when test ends
    // No need for manual cleanup
}
```

### 2. **Async Testing**

```rust
#[tokio::test]
async fn async_test() {
    // Use consistent timeout handling
    let context = AsyncContext::with_timeout(Duration::from_secs(10));
    let result = context.execute(my_async_operation()).await;
    assert!(result.is_ok());
}
```

### 3. **Error Handling**

```rust
// Use TestError for consistent error handling in tests
use shared_test_utils::error::TestResult;

#[test]
fn test_with_error_handling() {
    let result = some_operation().expect_test("Operation should succeed");
}
```

### 4. **Fixture Usage**

```rust
#[test]
fn test_complex_scenario() {
    // Start with a pre-configured setup
    let (workspace, fixture) = FixturePresets::multi_module().build(&workspace).unwrap();

    // Build upon the fixture for specific test needs
    workspace.create_file(Path::new("test.rs"), "fn test() {}").unwrap();
}
```

## Contributing

When adding new utilities:

1. Follow existing patterns for consistency
2. Include comprehensive tests
3. Update this documentation
4. Consider async support where applicable

## Version Information

This library is part of the Rust AI IDE workspace and is version controlled alongside the main project.

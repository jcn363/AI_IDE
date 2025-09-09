# rust-ai-ide-core

The core crate provides the central integration point for the Rust AI IDE architecture. This crate has been refactored into focused micro-crates while maintaining backward compatibility through re-exports.

After v2.4.0 refactoring, this crate serves as the main entry point, re-exporting functionality from specialized micro-crates including fundamental types, file operations, AI integrations, metrics, and shell operations.

## 📦 Features

- **Backward Compatibility**: Complete re-exports from micro-crates
- **Core Types & Utilities**: Error handling, validation, path operations
- **Integration Layer**: Unified access to all core functionality
- **Performance Monitoring**: Built-in metrics and telemetry
- **File System Operations**: Path management and workspace detection

## 🔗 Architecture Integration

This crate acts as the central hub, integrating with:

- `rust-ai-ide-shared-types`: Shared data types and serialization
- `rust-ai-ide-cache`: Caching layer for performance
- `rust-ai-ide-security`: Security and access control
- `rust-ai-ide-lsp`: Language server protocol integration

## Micro-crate Structure (v2.4.0):

- `rust-ai-ide-core-fundamentals`: Core types, utilities, error handling
- `rust-ai-ide-core-shell`: Async shell operations and command execution
- `rust-ai-ide-core-file`: File system operations and path management
- `rust-ai-ide-core-ai`: AI provider abstractions and integrations
- `rust-ai-ide-core-metrics`: Performance monitoring and telemetry

## 🚀 Usage

### Basic Usage

```rust
use rust_ai_ide_core::*;

// Use traits for validation
use rust_ai_ide_core::traits::Validatable;

// Access constants
use rust_ai_ide_core::constants;

// Error handling
use rust_ai_ide_core::error::CoreResult;
```

### Advanced Configuration

```rust
// Workspace detection
let current_path = Path::new("/path/to/project");
if current_path.is_workspace_root() {
    println!("Found workspace root");
}

// Path operations
let readable = current_path.readable_name();

// Error with context
let result = some_operation().with_context(|e| {
    CoreError::SystemError(format!("Operation failed: {}", e))
});
```

## 📚 Integration Guide

This crate provides seamless integration through re-exports. All functionality from micro-crates is available at the top level, maintaining API stability while enabling modular development.

For new integrations, use the specific micro-crates directly to avoid full dependencies.

## 📈 Performance Characteristics

- **Lightweight Re-exports**: No additional runtime overhead
- **Optimized Dependencies**: Micro-crates allow selective imports
- **Memory Efficient**: Constants and traits have minimal footprint
- **Fast Compilation**: Reduced compilation time with modular structure

## 🔄 Migration Notes

### v2.4.0 Refactoring

The monolithic core crate has been split into focused micro-crates:

- **Before**: Single large crate with all functionality
- **After**: Modular micro-crates with centralized re-exports

### Migration Path

1. Update imports to use `rust_ai_ide_core::*` (no changes needed for most existing code)
2. For new development, consider direct imports from micro-crates to reduce dependencies
3. Update any direct references to internal modules to use re-exports

### Breaking Changes

None at the public API level. All functionality remains available through re-exports.
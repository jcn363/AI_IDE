# ADR-003: Tauri Integration and Command Handling Patterns

## Status

- **Date**: 2025-01-13
- **Status**: Accepted

## Context

The Rust AI IDE requires:

1. **Desktop Application Framework**: Native desktop app with web frontend integration
2. **Secure IPC Communication**: Safe communication between Rust backend and TypeScript frontend
3. **Performance Optimization**: Minimize IPC latency for AI/ML operations
4. **Security Hardening**: Protection against command injection and input validation
5. **State Management**: Complex async state handling across UI and backend
6. **Plugin Architecture**: Extensible command system for IDE features

### Forces Considered

- **Framework Maturity**: Tauri's ecosystem maturity vs. custom solutions
- **Security Requirements**: IPC security vs. development velocity
- **Performance Targets**: <500ms response times for AI operations vs. IPC overhead
- **Development Complexity**: Complex macro system vs. maintainability
- **State Synchronization**: UI-backend state consistency vs. performance overhead
- **Build Complexity**: Multi-target compilation (Windows, macOS, Linux) vs. development effort

## Decision

**Adopt Tauri v2.x** as the desktop framework with the following architectural patterns:

1. **Custom Command Macro System**: `tauri_command_template!`, `acquire_service_and_execute!`, `execute_command!`
2. **Mandatory Input Validation**: All commands use `TauriInputSanitizer` from `rust-ai-ide-common`
3. **Double-Locking State Pattern**: Lazy initialization with `Arc<Mutex<T>>` for async services
4. **IPC Optimization**: Minimize serialization overhead for performance-critical operations
5. **Security-First Command Design**: Path validation, command injection prevention, audit logging

### Command Architecture

```rust
// Command macro system hierarchy
tauri_command_template! {
    // Standardized command wrapper
    acquire_service_and_execute! {
        // Service acquisition with error handling
        execute_command! {
            // Async command execution with retry logic
        }
    }
}
```

## Consequences

### Positive

- **Native Performance**: Direct OS integration with minimal resource overhead
- **Security Framework**: Built-in IPC security with additional validation layers
- **Cross-Platform**: Single codebase for Windows, macOS, and Linux
- **Web Technologies**: Familiar React/TypeScript frontend development
- **Plugin Ecosystem**: Extensible architecture for IDE features
- **Build Optimization**: Incremental compilation and optimized bundle sizes

### Negative

- **Complex Build Process**: Multi-target compilation increases build complexity
- **IPC Latency**: Serialization overhead for complex data structures
- **Debugging Difficulty**: Cross-process debugging challenges
- **Version Compatibility**: Tauri ecosystem evolution may require updates
- **Learning Curve**: Custom macro system increases onboarding complexity

### Risks

- **IPC Bottlenecks**: AI/ML operations may exceed 500ms target due to serialization
- **Build Failures**: Cross-platform compilation issues during development
- **Security Vulnerabilities**: Webview isolation bypass or IPC exploits
- **Maintenance Burden**: Custom macro system requires ongoing maintenance
- **Performance Degradation**: State management overhead in large applications

#### Mitigation Strategies

- **IPC Optimization**: Use shared memory for large data transfers
- **Build Automation**: Comprehensive CI/CD for cross-platform testing
- **Security Audits**: Regular security reviews of IPC and webview components
- **Performance Monitoring**: Built-in performance tracking and optimization
- **Documentation**: Comprehensive guides for custom macro system usage

## Alternatives Considered

### Alternative 1: Electron-Based Solution
- **Reason Not Chosen**: Resource overhead (200MB+ vs. Tauri's <50MB) unacceptable for enterprise IDE
- **Impact**: Poor performance, high memory usage, slower startup times

### Alternative 2: Custom Native Framework
- **Reason Not Chosen**: Development time (12+ months) would delay project timeline unacceptably
- **Impact**: Increased costs, maintenance burden, platform compatibility issues

### Alternative 3: Web-Only Architecture
- **Reason Not Chosen**: Requires constant internet connectivity for AI features, violates offline requirements
- **Impact**: Dependency on cloud services, data privacy concerns, unreliable user experience

### Alternative 4: Qt/QML Framework
- **Reason Not Chosen**: Would require C++ integration, complex build process, and larger bundle sizes
- **Impact**: Increased complexity, longer development cycles, steeper learning curve

## Implementation Notes

### Command Template System

```rust
// src-tauri/src/command_templates.rs
#[macro_export]
macro_rules! tauri_command_template {
    ($cmd_name:ident, $handler:expr) => {
        #[tauri::command]
        pub async fn $cmd_name(
            request: TauriInputSanitized,
            state: tauri::State<AppState>,
        ) -> Result<serde_json::Value, Error> {
            // Standardized wrapper with validation
            sanitize_and_validate_command!(request, stringify!($cmd_name));
            $handler(request, state).await
        }
    };
}
```

### State Management Pattern

```rust
// Double-locking pattern for lazy initialization
pub struct AppState {
    ai_service: Arc<Mutex<Option<Arc<AIService>>>>,
    lsp_service: Arc<Mutex<Option<Arc<LSPService>>>>,
}

impl AppState {
    pub async fn get_ai_service(&self) -> Result<Arc<AIService>, Error> {
        let mut service_guard = self.ai_service.lock().await;
        if service_guard.is_none() {
            // Lazy initialization with double-locking
            let service = Arc::new(AIService::new().await?);
            *service_guard = Some(service.clone());
        }
        Ok(service_guard.as_ref().unwrap().clone())
    }
}
```

### IPC Optimization Techniques

```rust
// Minimize serialization for performance-critical operations
#[tauri::command]
pub async fn analyze_code_stream(
    mut request: CodeAnalysisRequest,
    state: tauri::State<AppState>,
) -> Result<CodeAnalysisStream, Error> {
    sanitize_and_validate_command!(request, "analyze_code_stream");

    // Use streaming to avoid large payload serialization
    let stream = state.ai_service.lock().await
        .as_ref()
        .unwrap()
        .analyze_code_stream(request.code)
        .await?;

    Ok(stream)
}
```

### Security Integration

```rust
// Integrated security validation
#[macro_export]
macro_rules! sanitize_and_validate_command {
    ($request:expr, $command_name:expr) => {
        // Path validation
        if let Some(path) = &$request.file_path {
            validate_secure_path(path, stringify!($command_name))?;
        }

        // Input sanitization
        let sanitizer = TauriInputSanitizer::new();
        $request = sanitizer.sanitize_command_args($request)?;

        // Audit logging
        audit_logger::log_command_execution($command_name, &$request).await?;
    };
}
```

### Build Configuration

```toml
# src-tauri/Cargo.toml
[package]
name = "rust-ai-ide"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["staticlib", "cdylib", "rlib"]

[dependencies]
tauri = { version = "2.0", features = ["shell-open"] }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
```

## Related ADRs

- [ADR-001: Multi-Crate Workspace Architecture](adr-001-multi-crate-workspace-architecture.md)
- [ADR-004: AI/ML Service Architecture](adr-004-ai-ml-service-architecture.md)
- [ADR-005: Security Framework](adr-005-security-framework.md)
- [ADR-006: Async Concurrency Patterns](adr-006-async-concurrency-patterns.md)
# AGENTS.md

This file provides guidance to agents when working with code in this repository.

## Project Overview

This is a large-scale Rust AI IDE project using:

- **Primary Language**: Rust (Nightly 2025-09-03)
- **Desktop Framework**: Tauri with React/TypeScript web frontend
- **Architecture**: Modular workspace with 67 crates across 5 layers
- **Key Technologies**: Async Tokio, LSP protocol, AI/ML models

## Build/Lint/Test Commands

### Rust (Root Directory)
- Build workspace: `cargo build --workspace` or `cargo build -p <crate_name>`
- Test single crate: `cargo test -p <crate_name>` or `cargo test -- --test <test_function_name>`
- Test specific crate + single test: `cargo test -p <crate_name> -- --test <test_function_name>`
- Lint: `cargo +nightly clippy` (requires nightly toolchain)
- Format: `cargo +nightly fmt` (requires nightly toolchain)

### Web Frontend (web/ directory)
- Build: `cd web && npm run build` (runs `vite build` and generates types)
- Test single: `cd web && vitest run --reporter=verbose` (all tests) or `vitest run <test-file>`
- Type check: `cd web && npm run type-check` (tsc noEmit)

## Key Non-Obvious Patterns

### Nightly Rust Usage
- Uses nightly channel (2025-09-03) with unstable features (impl_trait_in_bindings)
- Requires nightly toolchain components: rust-src, rustfmt, clippy
- rust-version in Cargo.toml is 1.91.0 (stable floor, but nightly required)

### Tauri Integration
- Crate types: ["staticlib", "cdylib", "rlib"] for FFI bindings
- Custom command templates using macros in src-tauri/src/command_templates.rs
- Extensive use of tauri::State with Arc<Mutex<T>> for state management
- Input sanitization required via rust-ai-ide-common::validation

### Command Macro System
- `execute_command!` - async command execution with retry logic
- `tauri_command_template!` - standardized Tauri command handlers
- `acquire_service_and_execute!` - service acquisition with error handling
- `validate_commands!` - command validation macros
- `spawn_background_task!` - background task spawning with cleanup

### State Management
- Heavy use of tokio::sync primitives (Mutex, RwLock, mpsc, oneshot)
- Double-locking patterns for async state initialization
- EventBus for pub-sub communication between modules
- Rate limiter and connection pooling in infra module

### Architecture Constraints
- Multi-crate workspace with circular dependencies in types packages (intentional)
- AI features require separate LSP service initialization
- Cloud integrations need webhook system initialization on port 3000
- Webview and extension communicate through strict IPC patterns only

### Security Policies
- cargo-deny bans openssl, md5, ring, quick-js (security reasons)
- Only crates.io registry allowed, specific GitHub orgs only
- MIT/Apache-2.0 licenses only (GPL variants banned)
- Git2 allowed despite GPL due to necessity

### Development Workflow
- Type generation: npm script runs Cargo bin to generate TS types from Rust
- Web build auto-generates types before build
- Extensive placeholder implementations (return dummy JSON as "placeholder")
- Many commands commented out as "missing implementations"

### Performance Considerations
- Tokio pinned and configured for optimal async performance
- Moka for LRU caching with future API
- Custom performance testing in utils/performance_testing.rs
- Optimized SQLite with bundled compilation and version enforcement

## Toolchain Requirements
- Rust nightly 2025-09-03 with rust-src, rustfmt, clippy
- Node.js/npm for web frontend
- SQLite development libraries (bundled via libsqlite3-sys)

## Common Pitfalls
- Many Tauri commands have placeholder implementations - check if stub vs. real
- Async initialization required before certain features (AI LSP, webhooks)
- Webview restrictions: no localStorage, limited external APIs
- Monorepo structure requires workspace-wide cargo operations
- License compliance scanning must use cargo-deny, not other tools
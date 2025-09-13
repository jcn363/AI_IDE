# ADR-001: Multi-Crate Workspace Architecture

## Status

- **Date**: 2025-01-13
- **Status**: Accepted

## Context

The Rust AI IDE project requires:

1. **Modular Architecture**: Separation of concerns across 5 distinct layers (Shared Foundation, Foundation, AI/ML Specialization, System Integration, Advanced Services)
2. **Large Scale**: 67+ crates across multiple domains (LSP, AI/ML, Security, Web, etc.)
3. **Team Collaboration**: Multiple teams working on different components simultaneously
4. **Performance Requirements**: Optimized build times and minimal rebuild cascades
5. **Dependency Management**: Complex inter-crate dependencies with circular references in type packages
6. **CI/CD Integration**: Automated testing and deployment of workspace components

### Forces Considered

- **Build Performance**: Large monoliths vs. modular crates
- **Dependency Complexity**: Circular dependencies in type packages
- **Team Organization**: Independent development vs. coordinated releases
- **Maintenance Overhead**: Managing 67+ crates vs. monolithic structure
- **Testing Strategy**: Unit testing vs. integration testing across crates
- **Version Management**: Semantic versioning across interconnected crates

## Decision

**Adopt a modular multi-crate workspace architecture** with the following characteristics:

1. **67 Specialized Crates** organized in 5 architectural layers
2. **Intentional Circular Dependencies** in type packages for shared abstractions
3. **Workspace-Level Build Optimization** with `cargo build --workspace`
4. **Selective Crate Compilation** for development workflows
5. **Version Enforcement** for critical dependencies (SQLite, security crates)

### Layer Structure

```
├── 1. Shared Foundation Layer (Core types, utilities, performance monitoring)
├── 2. Foundation Layer (LSP server, debugger, file operations, cargo integration)
├── 3. AI/ML Specialization Layer (Analysis, learning, inference, code generation)
├── 4. System Integration Layer (Security, monitoring, collaboration, plugins)
└── 5. Advanced Services Layer (Refactoring, predictive maintenance, parallel processing)
```

## Consequences

### Positive

- **Modular Development**: Teams can work independently on specific crates
- **Optimized Builds**: `cargo build --workspace` enables parallel compilation
- **Clear Separation**: Each layer has well-defined responsibilities
- **Selective Testing**: Individual crates can be tested in isolation
- **Reusability**: Core crates can be reused across different components
- **Scalability**: Architecture supports adding new features without major refactoring

### Negative

- **Build Complexity**: Managing dependencies across 67 crates increases complexity
- **Version Management**: Coordinating versions across interdependent crates
- **Development Setup**: More complex initial setup for new developers
- **Circular Dependencies**: Type packages intentionally have circular refs (maintenance burden)
- **CI/CD Complexity**: Testing and deploying interconnected crates requires sophisticated pipelines

### Risks

- **Build Time Degradation**: As workspace grows, build times may increase significantly
- **Dependency Hell**: Complex dependency graphs may lead to resolution conflicts
- **Onboarding Difficulty**: New developers may struggle with workspace complexity
- **Maintenance Overhead**: Managing 67+ crates requires significant coordination effort

#### Mitigation Strategies

- **Automated CI/CD**: Comprehensive pipeline for testing and deployment
- **Clear Documentation**: Detailed guides for workspace navigation and development
- **Code Generation**: Automated tools for managing repetitive crate structures
- **Performance Monitoring**: Continuous monitoring of build times and optimization

## Alternatives Considered

### Alternative 1: Monolithic Crate Structure
- **Reason Not Chosen**: Would violate single responsibility principle and make concurrent development impossible
- **Impact**: Single point of failure, difficult testing, poor maintainability

### Alternative 2: Microservices Architecture
- **Reason Not Chosen**: Overkill for a desktop IDE, would introduce unnecessary network complexity
- **Impact**: Increased latency, deployment complexity, resource overhead

### Alternative 3: Plugin-Based Architecture
- **Reason Not Chosen**: Would complicate the core IDE functionality and create integration challenges
- **Impact**: Plugin compatibility issues, inconsistent user experience

### Alternative 4: Layer-Free Flat Structure
- **Reason Not Chosen**: Would lose architectural clarity and make dependency management impossible
- **Impact**: Spaghetti architecture, difficult maintenance, unclear boundaries

## Implementation Notes

### Workspace Configuration

```toml
# Cargo.toml workspace configuration
[workspace]
members = [
    "crates/rust-ai-ide-*",
    "crates/shared/*",
    # ... 67 crate paths
]
resolver = "2"

[workspace.dependencies]
# Critical dependencies with enforced versions
rusqlite = "0.31.0"
libsqlite3-sys = "0.27.0"
tokio = { version = "1.0", features = ["full"] }
```

### Circular Dependency Management

```rust
// Type packages intentionally allow circular dependencies
// crates/rust-ai-ide-types/src/lib.rs
pub use rust_ai_ide_common_types::*;
pub use rust_ai_ide_lsp_types::*;
```

### Build Optimization

```bash
# Workspace-wide build
cargo build --workspace

# Selective crate building
cargo build -p rust-ai-ide-lsp
cargo build -p rust-ai-ide-ai

# Release optimization
cargo build --workspace --release
```

## Related ADRs

- [ADR-002: Nightly Rust Usage](adr-002-nightly-rust-usage.md)
- [ADR-003: Tauri Integration Patterns](adr-003-tauri-integration-patterns.md)
- [ADR-004: AI/ML Service Architecture](adr-004-ai-ml-service-architecture.md)
- [ADR-006: Async Concurrency Patterns](adr-006-async-concurrency-patterns.md)
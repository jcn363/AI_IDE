# ADR-002: Nightly Rust Usage and Feature Adoption

## Status

- **Date**: 2025-01-13
- **Status**: Accepted

## Context

The Rust AI IDE project requires:

1. **Advanced Language Features**: Access to cutting-edge Rust features not available in stable
2. **Performance Optimization**: Latest compiler optimizations and code generation improvements
3. **Tooling Ecosystem**: Advanced clippy lints, rustfmt options, and development tools
4. **Long-term Maintenance**: 18+ month development timeline requiring stable toolchain evolution
5. **Enterprise Requirements**: Predictable release cycles and LTS-like stability guarantees

### Forces Considered

- **Stability vs. Innovation**: Stable channel reliability vs. nightly feature access
- **CI/CD Complexity**: Managing nightly toolchain across development and deployment environments
- **Breaking Changes**: Risk of nightly feature instability affecting development velocity
- **Team Expertise**: Developer familiarity with nightly toolchain management
- **Production Deployment**: Ensuring nightly builds remain stable for production use
- **Migration Strategy**: Managing transition from nightly to stable as features stabilize

## Decision

**Adopt Rust Nightly 2025-09-03** as the primary toolchain with the following constraints:

1. **Pinned Nightly Version**: Use specific nightly version (2025-09-03) for reproducibility
2. **Stable Floor Requirement**: `rust-version = "1.91.0"` in Cargo.toml as minimum stable baseline
3. **Selective Feature Adoption**: Only use nightly features with proven stability
4. **Migration Planning**: Track nightly features for eventual stable migration
5. **CI/CD Integration**: Automated nightly toolchain management and validation

### Current Nightly Features in Use

```rust
// Key nightly features adopted:
#![feature(impl_trait_in_bindings)]     // Enhanced async trait support
#![feature(async_fn_in_trait)]         // Async functions in traits
#![feature(return_position_impl_trait)] // Improved impl Trait ergonomics
#![feature(never_type)]                // Never type (!) support
#![feature(try_blocks)]                // Try blocks for error handling
```

## Consequences

### Positive

- **Advanced Language Features**: Access to `impl_trait_in_bindings` for complex async patterns
- **Performance Benefits**: Latest compiler optimizations and code generation improvements
- **Tooling Excellence**: Advanced clippy lints and rustfmt capabilities
- **Future-Proofing**: Early adoption of features that will become stable
- **Competitive Advantage**: Leverage Rust's most advanced capabilities

### Negative

- **Toolchain Management**: More complex setup and maintenance for development team
- **CI/CD Complexity**: Additional tooling required for nightly toolchain management
- **Breaking Changes Risk**: Potential instability from nightly feature changes
- **Onboarding Difficulty**: New developers must learn nightly toolchain setup
- **Production Concerns**: Need for stable release channel validation

### Risks

- **Nightly Instability**: Breaking changes in nightly could disrupt development
- **Build Failures**: Incompatible nightly versions could break CI/CD pipelines
- **Feature Deprecation**: Adopted features might be removed or significantly changed
- **Migration Cost**: Eventually moving to stable might require significant refactoring

#### Mitigation Strategies

- **Pinned Versions**: Use specific nightly version with comprehensive testing
- **Stable Floor**: Minimum stable version ensures baseline compatibility
- **Feature Tracking**: Monitor nightly feature stabilization in stable releases
- **Gradual Migration**: Plan for phased migration to stable as features stabilize
- **Comprehensive Testing**: Extensive test coverage to catch nightly-related issues

## Alternatives Considered

### Alternative 1: Stable Rust Only
- **Reason Not Chosen**: Would prevent access to critical features like `impl_trait_in_bindings` needed for advanced async patterns
- **Impact**: Significant architectural compromises, reduced performance, development complexity

### Alternative 2: Mixed Toolchain Approach
- **Reason Not Chosen**: Would create maintenance nightmare with different toolchains for different crates
- **Impact**: Build complexity, CI/CD complications, inconsistent development experience

### Alternative 3: Wait for Feature Stabilization
- **Reason Not Chosen**: Would delay project timeline by 6-12 months waiting for required features
- **Impact**: Competitive disadvantage, delayed market entry, increased development costs

### Alternative 4: Fork/Custom Toolchain
- **Reason Not Chosen**: Would create unsustainable maintenance burden and security concerns
- **Impact**: Divergence from main Rust ecosystem, security vulnerabilities, support challenges

## Implementation Notes

### Toolchain Configuration

```toml
# rust-toolchain.toml
[toolchain]
channel = "nightly-2025-09-03"
components = ["rust-src", "rustfmt", "clippy"]
targets = ["x86_64-unknown-linux-gnu", "x86_64-apple-darwin", "x86_64-pc-windows-msvc"]
```

### Cargo Configuration

```toml
# Cargo.toml
[package]
rust-version = "1.91.0"  # Stable floor

[lints.rust]
unexpected_cfgs = { level = "warn", check-cfg = ['cfg(nightly)'] }
```

### Feature Flags and Conditional Compilation

```rust
// Feature detection for nightly-only code
#[cfg(nightly)]
use std::future::Future;
// Fallback for stable
#[cfg(not(nightly))]
use futures::future::Future;
```

### CI/CD Setup

```yaml
# .github/workflows/ci.yml
- name: Install Rust Nightly
  run: |
    rustup toolchain install nightly-2025-09-03
    rustup component add rust-src rustfmt clippy --toolchain nightly-2025-09-03
    rustup default nightly-2025-09-03

- name: Verify Toolchain
  run: |
    rustc --version  # Should show nightly-2025-09-03
    cargo --version
```

### Development Workflow

```bash
# Local development setup
rustup toolchain install nightly-2025-09-03
rustup component add rust-src rustfmt clippy
rustup default nightly-2025-09-03

# Verify nightly features
cargo check
cargo test
cargo +nightly clippy
```

## Related ADRs

- [ADR-001: Multi-Crate Workspace Architecture](adr-001-multi-crate-workspace-architecture.md)
- [ADR-006: Async Concurrency Patterns](adr-006-async-concurrency-patterns.md)
- [ADR-004: AI/ML Service Architecture](adr-004-ai-ml-service-architecture.md)
# CI/CD Pipeline Guide

This comprehensive guide covers the complete CI/CD pipeline setup for the Rust AI IDE project, including automated testing, performance benchmarking, deployment automation, and quality assurance.

## 📋 Pipeline Overview

The Rust AI IDE uses GitHub Actions for a comprehensive CI/CD pipeline with the following components:

- **Multi-OS Testing**: Linux, macOS, Windows support
- **Performance Benchmarking**: Automated cargo bench with regression detection
- **Multi-Language Validation**: Tree-sitter parser testing for 8+ languages
- **Quality Assurance**: Comprehensive code quality checks
- **Crate Publishing**: Automated publishing to crates.io
- **Deployment Automation**: Kubernetes Helm deployments with blue-green strategy

## ⚙️ CI Pipeline Configuration

### Core CI Workflow (`.github/workflows/ci.yml`)

The main CI pipeline runs on every push and pull request to main/develop branches:

```yaml
name: CI Pipeline

on:
  push:
    branches: [main, develop, staging]
  pull_request:
    branches: [main, develop, staging]
  workflow_dispatch:
    inputs:
      environment:
        description: 'Deployment environment'
        required: true
        default: 'staging'
        type: choice
        options: [staging, production]
```

## 🧪 Testing Strategy

### Multi-Platform Testing

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, windows-latest, macos-latest]
    rust_version: [stable, "1.91.0"]
```

### Testing Types

- **Unit Tests**: `cargo test --lib --workspace`
- **Integration Tests**: `cargo test --test '*' --workspace`
- **End-to-End Tests**: `cargo test --test e2e --workspace`

## 📊 Performance Benchmarking

### Benchmark Categories

1. **Compilation Benchmarks**
   ```bash
   cargo bench --package rust-ai-ide-core -- compilation
   ```

2. **Memory Benchmarks**
   ```bash
   cargo bench --package rust-ai-ide-cache -- memory
   ```

3. **AST Processing Benchmarks**
   ```bash
   cargo bench --package rust-ai-ide-lsp -- ast
   ```

### Regression Detection

The pipeline includes automated regression detection:

```bash
# Basic regression check
cargo bench --workspace > current_benchmarks.txt
# Compare against baseline (manual comparison required)
```

## 🌍 Multi-Language Support

### Tree-sitter Parser Validation

```yaml
jobs:
  tree-sitter-validation:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Tree-sitter Parser Tests
        run: |
          cargo test tree_sitter_parsers -- --nocapture
          cargo test cross_language_ast -- --nocapture
```

### Supported Languages

- Rust (primary)
- JavaScript/TypeScript
- Python
- Go
- Java
- C++
- C
- HTML/CSS

## 🔍 Quality Assurance

### Code Quality Checks

```yaml
- name: Cargo Format Check
  run: cargo fmt --all -- --check

- name: Cargo Clippy
  run: cargo clippy --all-targets --all-features -- -D warnings

- name: Unused Dependencies Analysis
  run: cargo udeps --workspace
```

### Documentation Generation

```yaml
- name: Cargo Doc Generation
  run: cargo doc --workspace --no-deps --document-private-items
```

## 🏗️ Deployment Automation

### Blue-Green Deployment Strategy

```yaml
environment: ${{ github.event.inputs.environment }}
steps:
  - name: Blue-Green Deployment
    run: |
      helm upgrade --install rust-ai-ide-green \
        --set image.tag=${{ github.sha }} \
        --strategy blue-green
```

### Rollback Procedures

The deployment workflow includes automated rollback scripts:

```bash
kubectl patch service rust-ai-ide \
  --type='json' \
  -p='[{"op": "replace", "path": "/spec/selector/color", "value": "blue"}]'
```

## 📦 Publishing Workflow

### Crate Publishing Automation

```yaml
jobs:
  publish-crates:
    steps:
      - name: Publish ${{ matrix.crate }}
        run: |
          cargo publish --registry crates-io --no-confirm
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CRATES_IO_TOKEN }}
```

### Release Management

The pipeline automatically creates GitHub releases for tagged versions:

```yaml
- name: Create GitHub Release
  uses: actions/create-release@v1
  with:
    tag_name: ${{ github.ref }}
    release_name: Release ${{ github.ref_name }}
    body_path: release_notes.md
```

## 🛠️ Developer Workflow

### Pre-commit Hooks & CI/CD

To avoid duplicate work between local pre-commit hooks and CI/CD pipelines, use:

```bash
# Skip pre-commit hooks for commits (since CI/CD handles these checks)
git commit --no-verify -m "Your commit message"

# Or globally disable hooks in CI environments
git config core.hooksPath /dev/null
```

> **Recommendation**: Use `--no-verify` for all commits when working locally, since the CI/CD pipeline already handles:
> - Code formatting (`cargo fmt`)
> - Linting (`cargo clippy`)
> - Testing (`cargo test`)
> - Documentation generation (`cargo doc`)
> - Security scanning (`cargo audit`)

### Local Development Setup

```bash
# Clone the repository
git clone https://github.com/your-org/rust-ai-ide.git
cd rust-ai-ide

# Install dependencies
rustup component add rustfmt clippy

# Run local checks before committing
cargo fmt --check
cargo clippy -- -D warnings
cargo test --all

# Commit with skip-hooks flag
git commit --no-verify -m "feat: add new feature"
```

## 🔧 Environment Configuration

### Required Secrets

```yaml
# GitHub Repository Secrets
CRATES_IO_TOKEN                  # For publishing to crates.io
KUBE_CONFIG                      # For Kubernetes deployments
DOCKER_HUB_TOKEN                 # For Docker registry access
CODECOV_TOKEN                    # For coverage reporting
SLACK_WEBHOOK_URL               # For CI/CD notifications
DEPLOYMENT_WEBHOOK_URL          # For deployment notifications
```

### Environment Variables

```yaml
CARGO_REGISTRY_TOKEN: ${{ secrets.CRATES_IO_TOKEN }}
```

## 📈 Monitoring & Analytics

### Pipeline Metrics

The CI/CD pipeline generates comprehensive metrics:

- Test coverage reports (`coverage-results/cobertura.xml`)
- Benchmark results (`target/criterion/`)
- Performance regressions
- Security scan results
- Code quality metrics

### Artifacts

```yaml
# Build Artifacts
rust-binaries-${{ matrix.os }}-${{ matrix.rust_version }}
benchmark-results-${{ github.sha }}
benchmark-summary
qa-reports
sbom-bom.xml
```

## 🚀 Best Practices

### CI/CD Principles

1. **Fail Fast**: Quick feedback on code quality issues
2. **Matrix Testing**: Ensure cross-platform compatibility
3. **Security First**: Automated security scanning and dependency checks
4. **Rolling Deployments**: Gradual rollout with rollback capability
5. **Zero Down Time**: Blue-green deployment strategy

### Pipeline Optimization

- **Caching**: Dependency caching for faster builds
- **Parallel Jobs**: Parallel execution of independent jobs
- **Timeout Management**: Reasonable timeouts to prevent hung jobs
- **Artifact Management**: Proper retention policies and cleanup

### Security Considerations

- Never hard-code credentials or tokens
- Use GitHub secrets for sensitive data
- Security scanning on all PRs
- SBOM (Software Bill of Materials) generation
- Dependency license compliance checks

---

**Workflow Execution Time**: Approximately 15-30 minutes for full matrix build
**Maintained Platforms**: Linux (Ubuntu), macOS, Windows
**Test Coverage Threshold**: 90% minimum coverage required
**Benchmark Timeout**: 20 minutes maximum for performance tests
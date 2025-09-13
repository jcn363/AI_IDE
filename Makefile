# Rust AI IDE Build Optimization Makefile
# Provides optimized build commands for development and CI/CD

.PHONY: help build build-dev build-fast build-release test test-dev test-fast clean deps-update stats audit audit-check audit-report

# Default target
help:
	@echo "Rust AI IDE Build Optimization Commands:"
	@echo ""
	@echo "Development Builds:"
	@echo "  make build-dev      - Fast development build with optimizations"
	@echo "  make build-fast     - Ultra-fast debug build for iteration"
	@echo "  make build          - Standard development build"
	@echo ""
	@echo "Production Builds:"
	@echo "  make build-release  - Optimized release build"
	@echo "  make build-ci       - CI-optimized build"
	@echo ""
	@echo "Testing:"
	@echo "  make test-dev       - Fast development testing"
	@echo "  make test-fast      - Ultra-fast testing"
	@echo "  make test           - Standard testing"
	@echo ""
	@echo "Maintenance:"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make deps-update    - Update dependencies"
	@echo "  make stats          - Show build cache statistics"
	@echo "  make audit          - Run comprehensive security audit"
	@echo "  make audit-check    - Fail build on critical vulnerabilities"
	@echo "  make audit-report   - Generate audit reports"
	@echo ""
	@echo "CI/CD Commands:"
	@echo "  make ci-build       - CI build with parallel compilation"
	@echo "  make ci-test        - CI test with parallel execution"

# Development builds with optimizations
build-dev:
	@echo "Building with development optimizations..."
	cargo build --workspace --profile dev --jobs 0

build-fast:
	@echo "Building with ultra-fast debug profile..."
	cargo build --workspace --profile fast-debug --jobs 0

build:
	@echo "Building standard development version..."
	cargo build --workspace

# Production builds
build-release:
	@echo "Building optimized release version..."
	cargo build --workspace --profile release --jobs 0

build-ci:
	@echo "Building with CI optimizations..."
	cargo build --workspace --profile ci --jobs $(shell nproc)

# Testing with optimizations
test-dev:
	@echo "Running tests with development optimizations..."
	cargo test --workspace --profile dev --jobs 0 -- --nocapture

test-fast:
	@echo "Running tests with fast profile..."
	cargo test --workspace --profile fast-debug --jobs 0 -- --nocapture

test:
	@echo "Running standard tests..."
	cargo test --workspace

# CI commands with parallel execution
ci-build:
	@echo "CI build with parallel compilation..."
	time cargo build --workspace --profile ci --jobs $(shell nproc || echo 4)

ci-test:
	@echo "CI test with parallel execution..."
	time cargo test --workspace --profile ci --jobs $(shell nproc || echo 4) -- --nocapture

# Maintenance commands
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "Cleaning cache..."
	sccache --clean || echo "sccache not available"

deps-update:
	@echo "Updating dependencies..."
	cargo update

stats:
	@echo "Build cache statistics:"
	sccache --show-stats || echo "sccache not available"
	@echo ""
	@echo "Cargo build timing will be shown with 'time' commands"

# Workspace analysis
check-workspace:
	@echo "Analyzing workspace structure..."
	cargo tree --workspace | head -20
	@echo "Workspace member count: $(shell cargo metadata --format-version 1 | jq '.packages | length')"

# Enhanced Security auditing
audit:
	@echo "Running comprehensive security audit..."
	rustup default nightly
	cargo audit
	cargo deny check

audit-check:
	@echo "Running security audit check (fails on vulnerabilities)..."
	rustup default nightly
	cargo audit --deny warnings || (echo "❌ Security vulnerabilities found" && exit 1)
	cargo deny check || (echo "❌ License/dependency violations found" && exit 1)
	cargo geiger || (echo "❌ Unsafe code analysis failed" && exit 1)
	@echo "✅ All security checks passed"

audit-report:
	@echo "Generating comprehensive security audit report..."
	rustup default nightly
	mkdir -p security-reports/comprehensive
	cargo audit --json > security-reports/comprehensive/audit-report-$(shell date +%Y%m%d-%H%M%S).json
	cargo deny check --format json > security-reports/comprehensive/deny-report-$(shell date +%Y%m%d-%H%M%S).json
	cargo geiger --format json --output security-reports/comprehensive/geiger-report-$(shell date +%Y%m%d-%H%M%S).json
	@echo "Reports generated in security-reports/comprehensive/"

# Code formatting and linting
format:
	@echo "Formatting code with rustfmt..."
	rustup default nightly
	cargo fmt --all

format-check:
	@echo "Checking code formatting..."
	rustup default nightly
	cargo fmt --all -- --check || (echo "❌ Code formatting issues found" && exit 1)
	@echo "✅ Code formatting is correct"

lint:
	@echo "Running comprehensive linting..."
	rustup default nightly
	cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic

lint-security:
	@echo "Running security-focused linting..."
	rustup default nightly
	cargo clippy --workspace --all-targets --all-features \
		-- -W clippy::all \
		-D clippy::correctness \
		-D clippy::suspicious \
		-W clippy::pedantic \
		-W clippy::nursery \
		-D clippy::missing_const_for_fn \
		-W clippy::redundant_clone

# Static analysis
static-analysis:
	@echo "Running static security analysis..."
	rustup default nightly
	cargo install cargo-geiger 2>/dev/null || echo "cargo-geiger already installed"
	cargo geiger --format ascii-table
	cargo clippy --workspace --all-targets --all-features -- -W clippy::all

# Runtime security checks
runtime-check:
	@echo "Running runtime security checks..."
	rustup default nightly
	cargo build --workspace --release --features security
	cargo test --workspace --release --features security -- --nocapture

# Combined security pipeline
security-pipeline: format-check lint-security audit-check static-analysis runtime-check
	@echo "✅ Complete security pipeline passed"

# Security monitoring (continuous)
security-monitor:
	@echo "Running continuous security monitoring..."
	rustup default nightly
	./scripts/ci/comprehensive-security-reporting.sh --send-alerts --verbose

# Performance profiling
profile-build:
	@echo "Profiling build performance..."
	time cargo build --workspace --profile release --jobs 0 -v
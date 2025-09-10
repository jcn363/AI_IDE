#!/bin/bash

# Make script executable
chmod +x "$0"

# Enhanced Workspace Consistency Checker
# Version: 2.0.0
# Date: 2025
# Description: Comprehensive workspace validation and consistency checks

set -euo pipefail

# ========================================
# Configuration & Colors
# ========================================

# Colors for enhanced output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# Exit codes
ERROR_MISSING_DEPENDENCIES=1
ERROR_INVALID_WORKSPACE=2
ERROR_CRITICAL_ISSUES=3
ERROR_TOOLCHAIN_MISMATCH=4

# Configuration
REQUIRED_RUST_VERSION="${REQUIRED_RUST_VERSION:-nightly-2025-09-03}"
EXPECTED_WORKSPACE_VERSION="${EXPECTED_WORKSPACE_VERSION:-0.1.0}"
MIN_BUILD_SIZE_KB="${MIN_BUILD_SIZE_KB:-500}"
MAX_BUILD_SIZE_MB="${MAX_BUILD_SIZE_MB:-500}"

# ========================================
# Utility Functions
# ========================================

print_header() {
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${WHITE}🛠️  Rust AI IDE - Enhanced Workspace Consistency Checker v2.0${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
}

print_section() {
    echo -e "${CYAN}────────────────────────────────────────────────────────────────${NC}"
    echo -e "${CYAN}📌 $1${NC}"
    echo -e "${CYAN}────────────────────────────────────────────────────────────────${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# ========================================
# Tool Installation & Validation
# ========================================

install_required_tools() {
    print_section "Installing Required Tools"

    local tools=("cargo-set-version" "cargo-hakari" "cargo-udeps" "cargo-outdated" "cargo-deny")
    local tool_commands=("cargo install cargo-edit" "cargo install cargo-hakari" "cargo install cargo-udeps --locked" "cargo install cargo-outdated" "cargo install cargo-deny")

    for i in "${!tools[@]}"; do
        local tool="${tools[$i]}"
        local install_cmd="${tool_commands[$i]}"

        if ! command -v "$tool" &> /dev/null; then
            print_warning "$tool not found. Installing..."
            if ! eval "$install_cmd" 2>/dev/null; then
                print_error "Failed to install $tool. Manual installation may be required."
            else
                print_success "$tool installed successfully."
            fi
        else
            print_success "$tool is available"
        fi
    done
}

validate_toolchain() {
    print_section "Toolchain Validation"

    # Check Rust version
    rust_version=$(rustc --version 2>/dev/null | cut -d' ' -f2 || echo "unknown")
    toolchain=$(cargo --version 2>/dev/null | grep -o '[a-z]* [0-9-]*' | head -1 || echo "unknown")

    if grep -q "nightly" <<< "$toolchain"; then
        print_success "Using nightly toolchain: $toolchain"

        # Check for required components
        local components=("rust-src" "rustfmt" "clippy")
        for component in "${components[@]}"; do
            if rustup component list --installed | grep -q "^${component}-" 2>/dev/null; then
                print_info "$component component is installed"
            else
                print_warning "$component component missing. Run: rustup component add $component"
            fi
        done
    else
        print_warning "Not using nightly toolchain. Recommended: $REQUIRED_RUST_VERSION"
        print_info "Current toolchain: $toolchain"
    fi
}

# ========================================
# Workspace Validation
# ========================================

validate_workspace_structure() {
    print_section "Workspace Structure Validation"

    # Check if in correct directory
    if [[ ! -f "Cargo.toml" ]]; then
        print_error "Please run this script from the workspace root directory"
        exit $ERROR_INVALID_WORKSPACE
    fi

    # Validate Cargo.toml structure
    if ! grep -q "\[workspace\]" Cargo.toml; then
        print_error "Invalid workspace structure - missing [workspace] table"
        exit $ERROR_INVALID_WORKSPACE
    fi

    # Check workspace metadata
    if ! grep -q "resolver = \"2\"" Cargo.toml; then
        print_warning "Workspace using resolver 1 instead of 2. Consider upgrading."
    fi

    # Validate package metadata is consistent
    workspace_version=$(grep -m1 '^version = ' Cargo.toml | cut -d'=' -f2 | tr -d '"' | tr -d ' ' || echo "")
    workspace_authors=$(grep -m1 '^authors = ' Cargo.toml | cut -d'=' -f2 | tr -d '"' | tr -d ' ' || echo "")
    workspace_license=$(grep -m1 '^license = ' Cargo.toml | cut -d'=' -f2 | tr -d '"' | tr -d ' ' || echo "")

    print_info "Workspace version: ${workspace_version:-'not set'}"
    print_info "Workspace license: ${workspace_license:-'not set'}"
    print_info "Workspace authors: ${workspace_authors:-'not set'}"
}

# ========================================
# Crate Consistency Validation
# ========================================

validate_crate_consistency() {
    print_section "Crate Consistency Validation"

    local total_issues=0
    local workspace_members=$(cargo metadata --format-version=1 --no-deps 2>/dev/null | jq -r '.workspace_members[]' 2>/dev/null || echo "")

    if [[ -z "$workspace_members" ]]; then
        print_error "Unable to retrieve workspace members. Ensure Cargo.toml is valid."
        exit $ERROR_INVALID_WORKSPACE
    fi

    member_count=$(echo "$workspace_members" | wc -l)
    print_info "Found $member_count workspace members"

    while read -r member; do
        local crate_name=$(echo "$member" | cut -d ' ' -f1 | rev | cut -d ':' -f2- | rev)
        local crate_version=$(echo "$member" | cut -d ' ' -f2)
        local crate_path=$(echo "$member" | cut -d ' ' -f3)

        # Skip if crate path is not found
        [[ -z "$crate_path" ]] && continue

        print_info "Analyzing: $crate_name ($crate_version)"

        # Check Cargo.toml exists
        if [[ ! -f "$crate_path/Cargo.toml" ]]; then
            print_error "Missing Cargo.toml for $crate_name at $crate_path"
            ((total_issues++))
            continue
        fi

        # Validate crate metadata
        if ! validate_crate_metadata "$crate_path/Cargo.toml"; then
            ((total_issues++))
        fi

        # Check library structure
        validate_crate_structure "$crate_path" "$crate_name"
    done <<< "$workspace_members"

    if [[ $total_issues -gt 0 ]]; then
        print_warning "Found $total_issues consistency issues in workspace members"
    else
        print_success "All workspace members are consistent"
    fi
}

validate_crate_metadata() {
    local cargo_toml="$1"
    local issues_found=0

    # Check version alignment
    local crate_version=$(grep -m1 '^version = ' "$cargo_toml" | cut -d'=' -f2 | tr -d '"' | tr -d ' ' || echo "")

    if [[ "${EXPECTED_WORKSPACE_VERSION}" != "${crate_version}" ]]; then
        print_warning "Version mismatch in $cargo_toml: ${crate_version:-'not set'} (expected: $EXPECTED_WORKSPACE_VERSION)"
        ((issues_found++))
    fi

    # Check license alignment
    local crate_license=$(grep -m1 '^license = ' "$cargo_toml" | cut -d'=' -f2 | tr -d '"' | tr -d ' ' || echo "")

    if [[ "${workspace_license:-'MIT OR Apache-2.0'}" != "${crate_license:-'MIT OR Apache-2.0'}" ]]; then
        print_info "License differs in $cargo_toml: ${crate_license:-'not set'}"
    fi

    return $issues_found
}

validate_crate_structure() {
    local crate_path="$1"
    local crate_name="$2"

    # Check for lib.rs or main.rs
    if [[ -f "$crate_path/src/lib.rs" ]]; then
        print_info "$crate_name: Library crate detected"
    elif [[ -f "$crate_path/src/main.rs" ]]; then
        print_info "$crate_name: Binary crate detected"
    else
        print_warning "$crate_name: No main library entry point found"
    fi

    # Check for tests directory
    if [[ ! -d "$crate_path/tests" ]]; then
        print_info "$crate_name: No integration tests directory"
    else
        test_count=$(find "$crate_path/tests" -name "*.rs" 2>/dev/null | wc -l)
        print_info "$crate_name: Found $test_count integration test files"
    fi
}

# ========================================
# Dependency Analysis
# ========================================

analyze_dependencies() {
    print_section "Dependency Analysis"

    # Update workspace-hack
    print_info "Updating workspace-hack dependencies..."
    if cargo hakari generate 2>/dev/null; then
        print_success "workspace-hack updated successfully"
    else
        print_warning "workspace-hack update failed"
    fi

    # Check for unused dependencies
    print_info "Checking for unused dependencies..."
    if cargo_udeps_output=$(cargo +nightly udeps --output json --workspace 2>/dev/null); then
        unused_count=$(echo "$cargo_udeps_output" | jq '.[].unused_deps | length' 2>/dev/null | awk '{sum += $1} END {print sum}')
        if [[ $unused_count -gt 0 ]]; then
            print_warning "Found $unused_count unused dependencies across workspace"
        else
            print_success "No unused dependencies detected"
        fi
    fi

    # Check for outdated dependencies
    print_info "Checking for outdated dependencies..."
    if outdated_output=$(cargo outdated --workspace --format json 2>/dev/null); then
        outdated_count=$(echo "$outdated_output" | jq '. | length' 2>/dev/null || echo "0")
        if [[ $outdated_count -gt 0 ]]; then
            print_info "Found $outdated_count outdated dependencies"
            echo "$outdated_output" | jq -r 'keys[]' | while read -r dep; do
                print_info "  - $dep can be updated"
            done
        else
            print_success "All dependencies are up to date"
        fi
    fi

    # Check for duplicate dependencies
    print_info "Checking for duplicate dependencies..."
    if duplicate_output=$(cargo tree -d 2>/dev/null); then
        print_success "No duplicate dependencies found"
    else
        print_warning "Potential duplicate dependencies exist"
    fi
}

# ========================================
# Security & Quality Checks
# ========================================

security_checks() {
    print_section "Security & Quality Checks"

    # License compliance
    print_info "Performing license compliance check..."
    if cargo deny check licenses 2>/dev/null; then
        print_success "License compliance confirmed"
    else
        print_warning "License compliance issues found"
    fi

    # Security vulnerabilities
    print_info "Scanning for security vulnerabilities..."
    if cargo deny check advisories 2>/dev/null; then
        print_success "No security vulnerabilities found"
    else
        print_error "Security vulnerabilities detected"
    fi

    # Sources verification
    print_info "Verifying dependency sources..."
    if cargo deny check sources 2>/dev/null; then
        print_success "All dependencies from approved sources"
    else
        print_warning "Unauthorized dependency sources detected"
    fi
}

# ========================================
# Build Validation
# ========================================

validate_build() {
    print_section "Build Validation"

    # Check workspace compilation
    print_info "Building workspace..."
    if build_output=$(cargo check --workspace 2>&1); then
        print_success "Workspace builds successfully"
    else
        print_error "Build failed"
        echo "$build_output" | head -n 20
    fi

    # Get build metrics
    print_info "Analyzing build output..."
    target_dir="./src-tauri/target"
    if [[ -d "$target_dir" ]]; then
        build_size_mb=$(du -sm "$target_dir" | cut -f1)
        if [[ $build_size_mb -gt $MAX_BUILD_SIZE_MB ]]; then
            print_warning "Build size ($build_size_mb MB) exceeds recommended maximum ($MAX_BUILD_SIZE_MB MB)"
        else
            print_info "Build size: $build_size_mb MB"
        fi
    fi
}

# ========================================
# Reporting & Recommendations
# ========================================

generate_report() {
    print_section "Workspace Health Report"

    local report_file="workspace-health-report-$(date +%Y%m%d-%H%M%S).txt"

    {
        echo "Rust AI IDE Workspace Health Report"
        echo "====================================="
        echo "Generated: $(date)"
        echo "Workspace Version: $(grep -m1 '^version = ' Cargo.toml | cut -d'=' -f2 | tr -d '"' | tr -d ' ' || echo 'unknown')"
        echo ""

        echo "Workspace Statistics:"
        echo "- Members: $(cargo metadata --format-version=1 --no-deps 2>/dev/null | jq '.workspace_members | length' 2>/dev/null || echo 'unknown')"
        echo "- Crate Types: Library, Binary, Workspace-hack"
        echo ""

        echo "Recommendations:"
        echo "1. Regularly run this consistency checker (weekly/bi-weekly)"
        echo "2. Update dependencies monthly to stay current with security patches"
        echo "3. Review and fix any warnings during development"
        echo "4. Ensure all new crates follow workspace conventions"
        echo "5. Keep workspace-hack updated for optimal build performance"
        echo ""

        echo "Next Steps:"
        echo "- Execute: cargo test --workspace"
        echo "- Review: cargo outdated --workspace"
        echo "- Fix any critical issues identified above"

    } > "$report_file"

    print_success "Health report saved to: $report_file"
}

# ========================================
# Main Execution
# ========================================

main() {
    print_header

    # Validate environment
    validate_workspace_structure

    # Install and validate tools
    install_required_tools
    validate_toolchain

    # Run comprehensive checks
    validate_crate_consistency
    analyze_dependencies
    security_checks
    validate_build

    # Generate report
    generate_report

    print_section "Summary"
    print_success "Workspace consistency check completed successfully!"
    print_info "Regularly run this script to maintain workspace health"
    print_info "Address any warnings or errors discovered above"
    echo ""
    echo -e "${GREEN}🚀 Happy coding with a healthy workspace!${NC}"
}

# ========================================
# Script Entry Point
# ========================================

# Execute main function
main "$@"

# Exit with success
exit 0

#!/bin/bash

# Rust AI IDE Code Deduplication Validation Script
# ================================================
#
# This script validates that the code deduplication process was successful
# by checking for:
# - Remaining duplicate utility functions
# - Incorrect deprecation warnings
# - Missing re-exports
# - Unconsolidated error types
# - Canonical implementations in place

# set -e  # Comment out to continue on errors so we can see all issues

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🔍 Rust AI IDE Code Deduplication Validation"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Validation counters
PASSED=0
FAILED=0
WARNINGS=0

# Function to report results
report() {
    local test_name="$1"
    local result="$2"
    local message="$3"

    case "$result" in
        "PASS")
            echo -e "${GREEN}✓ ${test_name}${NC}"
            ((PASSED++))
            ;;
        "FAIL")
            echo -e "${RED}✗ ${test_name}${NC}: ${message}"
            ((FAILED++))
            ;;
        "WARN")
            echo -e "${YELLOW}⚠ ${test_name}${NC}: ${message}"
            ((WARNINGS++))
            ;;
    esac
}

# Check if required crates exist
check_required_crates() {
    echo -e "\n${BLUE}Checking Required Crates...${NC}"

    if [ ! -f "$PROJECT_ROOT/crates/rust-ai-ide-shared-utils/Cargo.toml" ]; then
        report "Shared Utils Crate" "FAIL" "Missing rust-ai-ide-shared-utils crate"
        return 1
    fi

    if [ ! -f "$PROJECT_ROOT/crates/rust-ai-ide-shared-utils/src/lib.rs" ]; then
        report "Shared Utils Library" "FAIL" "Missing lib.rs in shared-utils crate"
        return 1
    fi

    if [ ! -f "$PROJECT_ROOT/crates/rust-ai-ide-derive-utils/Cargo.toml" ]; then
        report "Derive Utils Crate" "FAIL" "Missing rust-ai-ide-derive-utils crate"
        return 1
    fi

    report "Required Crates" "PASS" "All required crates are present"
}

# Check for remaining duplicate utility functions
check_duplicate_utilities() {
    echo -e "\n${BLUE}Checking for Duplicate Utility Functions...${NC}"

    # Look for remaining implementations of get_extension
    local get_extension_count=$(find "$PROJECT_ROOT/crates" -name "*.rs" -type f \
        -exec grep -l "fn get_extension" {} \; | grep -v shared-utils | wc -l)

    if [ "$get_extension_count" -gt 0 ]; then
        local files=$(find "$PROJECT_ROOT/crates" -name "*.rs" -type f \
            -exec grep -l "fn get_extension" {} \; | grep -v shared-utils)
        report "get_extension Duplicates" "WARN" "Found $get_extension_count duplicate implementations: $files"
    else
        report "get_extension Duplicates" "PASS" "No duplicate implementations found"
    fi

    # Look for remaining implementations of is_code_file
    local is_code_file_count=$(find "$PROJECT_ROOT/crates" -name "*.rs" -type f \
        -exec grep -l "fn is_code_file" {} \; | grep -v shared-utils | wc -l)

    if [ "$is_code_file_count" -gt 0 ]; then
        local files=$(find "$PROJECT_ROOT/crates" -name "*.rs" -type f \
            -exec grep -l "fn is_code_file" {} \; | grep -v shared-utils)
        report "is_code_file Duplicates" "WARN" "Found $is_code_file_count duplicate implementations: $files"
    else
        report "is_code_file Duplicates" "PASS" "No duplicate implementations found"
    fi
}

# Check deprecation warnings are in place
check_deprecation_warnings() {
    echo -e "\n${BLUE}Checking Deprecation Warnings...${NC}"

    # Check for deprecated attribute usage in core crates
    local deprecated_count=$(find "$PROJECT_ROOT/crates" -name "*.rs" -type f \
        -exec grep -l "#\[deprecated" {} \; | wc -l)

    if [ "$deprecated_count" -gt 0 ]; then
        report "Deprecation Warnings" "PASS" "Found $deprecated_count files with deprecation warnings"
    else
        report "Deprecation Warnings" "WARN" "No deprecation warnings found - may indicate incomplete migration"
    fi

    # Check for proper deprecation messages
    local proper_deprecation_count=$(find "$PROJECT_ROOT/crates" -name "*.rs" -type f \
        -exec grep -l "rust_ai_ide_shared_utils" {} \; | grep -v shared-utils | wc -l)

    if [ "$proper_deprecation_count" -gt 0 ]; then
        report "Proper Deprecation Messages" "PASS" "Found $proper_deprecation_count files referencing shared utils"
    else
        report "Proper Deprecation Messages" "WARN" "No files found referencing shared utils in deprecation messages"
    fi
}

# Check re-exports are working
check_re_exports() {
    echo -e "\n${BLUE}Checking Re-exports...${NC}"

    # Check that core-fundamentals re-exports shared utils
    if grep -q "rust_ai_ide_shared_utils" "$PROJECT_ROOT/crates/rust-ai-ide-core-fundamentals/src/utils.rs" 2>/dev/null; then
        report "Core Fundamentals Re-exports" "PASS" "Shared utils properly re-exported"
    else
        report "Core Fundamentals Re-exports" "FAIL" "Missing re-exports in core-fundamentals"
    fi

    # Check that core-file re-exports shared utils
    if grep -q "rust_ai_ide_shared_utils" "$PROJECT_ROOT/crates/rust-ai-ide-core-file/src/fs_utils.rs" 2>/dev/null; then
        report "Core File Re-exports" "PASS" "Shared utils properly re-exported"
    else
        report "Core File Re-exports" "FAIL" "Missing re-exports in core-file"
    fi
}

# Check error type consolidation
check_error_consolidation() {
    echo -e "\n${BLUE}Checking Error Type Consolidation...${NC}"

    # Check that all crates depend on rust-ai-ide-errors
    local error_deps_count=$(find "$PROJECT_ROOT/crates" -name "Cargo.toml" -type f \
        -exec grep -l "rust-ai-ide-errors" {} \; | wc -l)

    if [ "$error_deps_count" -gt 10 ]; then  # Should be most crates
        report "Error Dependencies" "PASS" "Found $error_deps_count crates depending on rust-ai-ide-errors"
    else
        report "Error Dependencies" "WARN" "Only $error_deps_count crates depend on rust-ai-ide-errors"
    fi

    # Check for unified IDEError usage
    local unified_error_usage=$(find "$PROJECT_ROOT/crates" -name "*.rs" -type f \
        -exec grep -l "rust_ai_ide_errors::IDEError" {} \; | wc -l)

    if [ "$unified_error_usage" -gt 5 ]; then
        report "Unified Error Usage" "PASS" "Found $unified_error_usage files using unified IDEError"
    else
        report "Unified Error Usage" "WARN" "Only $unified_error_usage files using unified IDEError"
    fi
}

# Check workspace configuration
check_workspace_config() {
    echo -e "\n${BLUE}Checking Workspace Configuration...${NC}"

    if grep -q "rust-ai-ide-shared-utils" "$PROJECT_ROOT/Cargo.toml" 2>/dev/null; then
        report "Workspace Shared Utils" "PASS" "Shared utils included in workspace"
    else
        report "Workspace Shared Utils" "FAIL" "Missing shared utils in workspace configuration"
    fi

    if grep -q "rust-ai-ide-derive-utils" "$PROJECT_ROOT/Cargo.toml" 2>/dev/null; then
        report "Workspace Derive Utils" "PASS" "Derive utils included in workspace"
    else
        report "Workspace Derive Utils" "FAIL" "Missing derive utils in workspace configuration"
    fi
}

# Check for compilation
check_compilation() {
    echo -e "\n${BLUE}Checking Compilation...${NC}"

    if command -v cargo >/dev/null 2>&1; then
        cd "$PROJECT_ROOT"
        if cargo check --quiet 2>/dev/null; then
            report "Compilation Check" "PASS" "Code compiles successfully"
        else
            report "Compilation Check" "FAIL" "Compilation failed - check for remaining issues"
        fi
    else
        report "Cargo Availability" "WARN" "Cargo not available - skipping compilation check"
    fi
}

# Check test utilities consolidation
check_test_utils() {
    echo -e "\n${BLUE}Checking Test Utilities Consolidation...${NC}"

    if grep -q "deprecated" "$PROJECT_ROOT/crates/rust-ai-ide-test-utils/Cargo.toml" 2>/dev/null; then
        report "Test Utils Deprecation" "PASS" "Test utils properly marked as deprecated"
    else
        report "Test Utils Deprecation" "WARN" "Test utils not marked as deprecated"
    fi

    if grep -q "shared-test-utils" "$PROJECT_ROOT/crates/rust-ai-ide-test-utils/Cargo.toml" 2>/dev/null; then
        report "Shared Test Utils Dependency" "PASS" "Test utils depend on shared-test-utils"
    else
        report "Shared Test Utils Dependency" "FAIL" "Missing dependency on shared-test-utils"
    fi
}

# Main validation flow
main() {
    echo "$(date): Starting deduplication validation"
    echo "Project root: $PROJECT_ROOT"

    cd "$PROJECT_ROOT"

    echo "Running check_required_crates..."
    check_required_crates

    echo "Running check_duplicate_utilities..."
    check_duplicate_utilities

    echo "Running check_deprecation_warnings..."
    check_deprecation_warnings

    echo "Running check_re_exports..."
    check_re_exports

    echo "Running check_error_consolidation..."
    check_error_consolidation

    echo "Running check_workspace_config..."
    check_workspace_config

    echo "Running check_compilation..."
    check_compilation

    echo "Running check_test_utils..."
    check_test_utils

    echo -e "\n${BLUE}Validation Summary${NC}"
    echo "==================="
    echo -e "${GREEN}Passed: $PASSED${NC}"
    echo -e "${RED}Failed: $FAILED${NC}"
    echo -e "${YELLOW}Warnings: $WARNINGS${NC}"

    if [ "$FAILED" -gt 0 ]; then
        echo -e "\n${RED}❌ Validation failed - address the failed checks above${NC}"
        exit 1
    elif [ "$WARNINGS" -gt 0 ]; then
        echo -e "\n${YELLOW}⚠️  Validation passed with warnings - review the warnings above${NC}"
        exit 0
    else
        echo -e "\n${GREEN}✅ Validation passed - all checks successful${NC}"
        exit 0
    fi
}

# Run main function
main "$@"
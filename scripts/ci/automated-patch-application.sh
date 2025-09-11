#!/bin/bash

# Automated Patch Application and Rollback System
# Comprehensive vulnerability remediation with testing and rollback capabilities
# Author: Security Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
WEB_DIR="${PROJECT_ROOT}/web"
SECURITY_LOG="${PROJECT_ROOT}/patch-application.log"
REPORT_DIR="${PROJECT_ROOT}/security-reports/patch-application"
BACKUP_DIR="${PROJECT_ROOT}/security-backups/$(date +%Y%m%d_%H%M%S)"
START_TIME=$(date +%s)

# Default configuration
DRY_RUN=false
AUTO_APPROVE=false
SEVERITY_THRESHOLD="medium"
ROLLBACK_ON_FAILURE=true
TEST_AFTER_PATCH=true
MAX_PATCH_ATTEMPTS=3

# Create directories
mkdir -p "${REPORT_DIR}"
mkdir -p "${BACKUP_DIR}"

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${SECURITY_LOG}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${SECURITY_LOG}" >&2
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${SECURITY_LOG}"
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${SECURITY_LOG}"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Automated patch application with rollback capabilities for Rust AI IDE.

OPTIONS:
    -h, --help                  Show this help message
    -v, --verbose               Enable verbose output
    --dry-run                   Run in dry-run mode (no changes applied)
    --auto-approve              Automatically approve patches without prompts
    --severity-threshold        Minimum severity to patch (low|medium|high) (default: medium)
    --no-rollback               Disable automatic rollback on failure
    --no-test                   Skip testing after patch application
    --max-attempts NUM          Maximum patch attempts per vulnerability (default: 3)
    --backup-dir DIR            Backup directory (default: auto-generated)
    --report-dir DIR            Output directory for reports (default: security-reports/patch-application)

EXAMPLES:
    $0 --dry-run --verbose
    $0 --auto-approve --severity-threshold high
    $0 --backup-dir /custom/backup/path

EOF
}

# Parse command line arguments
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --auto-approve)
            AUTO_APPROVE=true
            shift
            ;;
        --severity-threshold)
            SEVERITY_THRESHOLD="$2"
            shift 2
            ;;
        --no-rollback)
            ROLLBACK_ON_FAILURE=false
            shift
            ;;
        --no-test)
            TEST_AFTER_PATCH=false
            shift
            ;;
        --max-attempts)
            MAX_PATCH_ATTEMPTS="$2"
            shift 2
            ;;
        --backup-dir)
            BACKUP_DIR="$2"
            shift 2
            ;;
        --report-dir)
            REPORT_DIR="$2"
            shift 2
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Function to create backups
create_backups() {
    log_info "Creating backups in: ${BACKUP_DIR}"

    # Backup Rust dependencies
    if [[ -f "${PROJECT_ROOT}/Cargo.lock" ]]; then
        cp "${PROJECT_ROOT}/Cargo.lock" "${BACKUP_DIR}/Cargo.lock.backup"
        log_info "Backed up Cargo.lock"
    fi

    if [[ -f "${PROJECT_ROOT}/Cargo.toml" ]]; then
        cp "${PROJECT_ROOT}/Cargo.toml" "${BACKUP_DIR}/Cargo.toml.backup"
        log_info "Backed up Cargo.toml"
    fi

    # Backup web dependencies
    if [[ -f "${WEB_DIR}/package-lock.json" ]]; then
        cp "${WEB_DIR}/package-lock.json" "${BACKUP_DIR}/package-lock.json.backup"
        log_info "Backed up package-lock.json"
    fi

    if [[ -f "${WEB_DIR}/package.json" ]]; then
        cp "${WEB_DIR}/package.json" "${BACKUP_DIR}/package.json.backup"
        log_info "Backed up package.json"
    fi

    # Create backup manifest
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg backup_dir "$BACKUP_DIR" \
        --arg dry_run "$DRY_RUN" \
        '{
            timestamp: $timestamp,
            backup_dir: $backup_dir,
            dry_run: ($dry_run == "true"),
            files_backed_up: [
                "Cargo.lock",
                "Cargo.toml",
                "package-lock.json",
                "package.json"
            ]
        }' > "${BACKUP_DIR}/backup-manifest.json"

    log_success "Backups created successfully"
}

# Function to restore from backups
restore_backups() {
    if [[ "${DRY_RUN}" == true ]]; then
        log_info "Dry run - skipping backup restoration"
        return 0
    fi

    log_info "Restoring from backups..."

    # Restore Rust dependencies
    if [[ -f "${BACKUP_DIR}/Cargo.lock.backup" ]]; then
        cp "${BACKUP_DIR}/Cargo.lock.backup" "${PROJECT_ROOT}/Cargo.lock"
        log_info "Restored Cargo.lock"
    fi

    if [[ -f "${BACKUP_DIR}/Cargo.toml.backup" ]]; then
        cp "${BACKUP_DIR}/Cargo.toml.backup" "${PROJECT_ROOT}/Cargo.toml"
        log_info "Restored Cargo.toml"
    fi

    # Restore web dependencies
    if [[ -f "${BACKUP_DIR}/package-lock.json.backup" ]]; then
        cp "${BACKUP_DIR}/package-lock.json.backup" "${WEB_DIR}/package-lock.json"
        log_info "Restored package-lock.json"
    fi

    if [[ -f "${BACKUP_DIR}/package.json.backup" ]]; then
        cp "${BACKUP_DIR}/package.json.backup" "${WEB_DIR}/package.json"
        log_info "Restored package.json"
    fi

    log_success "Backups restored successfully"
}

# Function to analyze vulnerabilities
analyze_vulnerabilities() {
    log_info "Analyzing vulnerabilities from recent scans..."

    local rust_vulns=0
    local npm_vulns=0
    local high_severity_rust=0
    local high_severity_npm=0

    # Analyze cargo-audit results
    if [[ -f "${PROJECT_ROOT}/security-reports/dependency-security-report.json" ]]; then
        rust_vulns=$(jq -r '.issues_found // 0' "${PROJECT_ROOT}/security-reports/dependency-security-report.json" 2>/dev/null || echo "0")

        # Extract high severity Rust vulnerabilities
        if [[ -f "${PROJECT_ROOT}/security-reports/audit-results.json" ]]; then
            high_severity_rust=$(jq -r '.vulnerabilities.list[]? | select(.severity == "High") | .advisory.id' "${PROJECT_ROOT}/security-reports/audit-results.json" 2>/dev/null | wc -l || echo "0")
        fi
    fi

    # Analyze Snyk results
    if [[ -f "${PROJECT_ROOT}/security-reports/snyk/snyk-summary-report.json" ]]; then
        npm_vulns=$(jq -r '.vulnerabilities.total // 0' "${PROJECT_ROOT}/security-reports/snyk/snyk-summary-report.json" 2>/dev/null || echo "0")
        high_severity_npm=$(jq -r '.vulnerabilities.high // 0' "${PROJECT_ROOT}/security-reports/snyk/snyk-summary-report.json" 2>/dev/null || echo "0")
    fi

    log_info "Vulnerability Analysis:"
    log_info "  Rust dependencies: ${rust_vulns} total, ${high_severity_rust} high severity"
    log_info "  NPM dependencies: ${npm_vulns} total, ${high_severity_npm} high severity"

    # Generate vulnerability analysis report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg rust_total "$rust_vulns" \
        --arg rust_high "$high_severity_rust" \
        --arg npm_total "$npm_vulns" \
        --arg npm_high "$high_severity_npm" \
        --arg severity_threshold "$SEVERITY_THRESHOLD" \
        '{
            timestamp: $timestamp,
            severity_threshold: $severity_threshold,
            vulnerabilities: {
                rust: {
                    total: ($rust_total | tonumber),
                    high_severity: ($rust_high | tonumber)
                },
                npm: {
                    total: ($npm_total | tonumber),
                    high_severity: ($npm_high | tonumber)
                }
            },
            total_vulnerabilities: (($rust_total | tonumber) + ($npm_total | tonumber)),
            total_high_severity: (($rust_high | tonumber) + ($npm_high | tonumber))
        }' > "${REPORT_DIR}/vulnerability-analysis.json"

    # Determine if patching is needed
    local total_high=$(($high_severity_rust + $high_severity_npm))
    if [[ "$total_high" -gt 0 ]]; then
        log_warning "${total_high} high-severity vulnerabilities found - patching recommended"
        return 0
    else
        log_success "No high-severity vulnerabilities found"
        return 1
    fi
}

# Function to apply Rust patches
apply_rust_patches() {
    if [[ "${DRY_RUN}" == true ]]; then
        log_info "Dry run - would apply Rust patches"
        return 0
    fi

    log_info "Applying Rust dependency patches..."

    local patch_attempts=0
    local patch_success=false

    # Try cargo update first (safe updates)
    log_info "Attempting cargo update..."
    if cargo update > "${REPORT_DIR}/cargo-update.log" 2>&1; then
        log_success "cargo update completed successfully"
        patch_success=true
    else
        log_warning "cargo update failed"
    fi

    # Check if we need to use cargo-edit for specific updates
    if [[ "$patch_success" == false ]] && command -v cargo-edit >/dev/null 2>&1; then
        log_info "Attempting targeted updates with cargo-edit..."

        # This would need more specific vulnerability information
        # For now, we'll use a conservative approach
        log_info "Conservative patching approach - manual review recommended"
    fi

    # Generate Rust patch report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg patches_applied "$patch_success" \
        --arg method "cargo_update" \
        '{
            timestamp: $timestamp,
            language: "rust",
            patches_applied: ($patches_applied == "true"),
            method: $method,
            log_file: "cargo-update.log"
        }' > "${REPORT_DIR}/rust-patch-report.json"

    return $([ "$patch_success" == true ] && echo 0 || echo 1)
}

# Function to apply NPM patches
apply_npm_patches() {
    if [[ "${DRY_RUN}" == true ]]; then
        log_info "Dry run - would apply NPM patches"
        return 0
    fi

    log_info "Applying NPM dependency patches..."

    cd "${WEB_DIR}"

    local patch_success=false

    # Try npm audit fix first (safe automated fixes)
    log_info "Attempting npm audit fix..."
    if npm audit fix > "${REPORT_DIR}/npm-audit-fix.log" 2>&1; then
        log_success "npm audit fix completed successfully"
        patch_success=true
    else
        log_warning "npm audit fix failed or found no auto-fixable issues"
    fi

    # Try Snyk wizard if available
    if [[ "$patch_success" == false ]] && command -v snyk >/dev/null 2>&1; then
        log_info "Attempting Snyk wizard for additional fixes..."

        # Non-interactive mode for CI/CD
        if [[ "${AUTO_APPROVE}" == true ]]; then
            echo "y" | snyk wizard > "${REPORT_DIR}/snyk-wizard.log" 2>&1 || true
        else
            log_info "Snyk wizard requires manual intervention - skipping in automated mode"
        fi
    fi

    # Generate NPM patch report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg patches_applied "$patch_success" \
        --arg method "npm_audit_fix" \
        '{
            timestamp: $timestamp,
            language: "npm",
            patches_applied: ($patches_applied == "true"),
            method: $method,
            log_file: "npm-audit-fix.log"
        }' > "${REPORT_DIR}/npm-patch-report.json"

    return $([ "$patch_success" == true ] && echo 0 || echo 1)
}

# Function to run tests after patching
run_post_patch_tests() {
    if [[ "${TEST_AFTER_PATCH}" != true ]]; then
        log_info "Skipping post-patch tests"
        return 0
    fi

    log_info "Running tests after patch application..."

    local test_success=true

    # Test Rust components
    log_info "Testing Rust components..."
    if cargo test --workspace --quiet > "${REPORT_DIR}/rust-test-after-patch.log" 2>&1; then
        log_success "Rust tests passed"
    else
        log_error "Rust tests failed"
        test_success=false
    fi

    # Test web components
    log_info "Testing web components..."
    cd "${WEB_DIR}"
    if npm test > "${REPORT_DIR}/npm-test-after-patch.log" 2>&1; then
        log_success "NPM tests passed"
    else
        log_error "NPM tests failed"
        test_success=false
    fi

    # Generate test report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg tests_passed "$test_success" \
        --arg rust_tests "$( [[ -f "${REPORT_DIR}/rust-test-after-patch.log" ]] && echo "true" || echo "false" )" \
        --arg npm_tests "$( [[ -f "${REPORT_DIR}/npm-test-after-patch.log" ]] && echo "true" || echo "false" )" \
        '{
            timestamp: $timestamp,
            tests_passed: ($tests_passed == "true"),
            rust_tests_run: ($rust_tests == "true"),
            npm_tests_run: ($npm_tests == "true"),
            rust_test_log: "rust-test-after-patch.log",
            npm_test_log: "npm-test-after-patch.log"
        }' > "${REPORT_DIR}/post-patch-test-report.json"

    return $([ "$test_success" == true ] && echo 0 || echo 1)
}

# Function to generate comprehensive patch report
generate_patch_report() {
    log_info "Generating comprehensive patch application report..."

    local overall_success=true

    # Check if any patches were applied
    local rust_patches=$(jq -r '.patches_applied // false' "${REPORT_DIR}/rust-patch-report.json" 2>/dev/null || echo "false")
    local npm_patches=$(jq -r '.patches_applied // false' "${REPORT_DIR}/npm-patch-report.json" 2>/dev/null || echo "false")

    # Check if tests passed
    if [[ "${TEST_AFTER_PATCH}" == true ]]; then
        local tests_passed=$(jq -r '.tests_passed // false' "${REPORT_DIR}/post-patch-test-report.json" 2>/dev/null || echo "false")
        if [[ "$tests_passed" == "false" ]]; then
            overall_success=false
        fi
    fi

    # Generate final report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg duration "$(($(date +%s) - START_TIME))" \
        --arg backup_dir "$BACKUP_DIR" \
        --arg dry_run "$DRY_RUN" \
        --arg overall_success "$overall_success" \
        --arg rust_patches "$rust_patches" \
        --arg npm_patches "$npm_patches" \
        --arg tests_run "$TEST_AFTER_PATCH" \
        '{
            timestamp: $timestamp,
            duration_seconds: ($duration | tonumber),
            backup_dir: $backup_dir,
            dry_run: ($dry_run == "true"),
            patches_applied: {
                rust: ($rust_patches == "true"),
                npm: ($npm_patches == "true")
            },
            tests_run: ($tests_run == "true"),
            overall_success: ($overall_success == "true"),
            rollback_available: ($dry_run == "false")
        }' > "${REPORT_DIR}/patch-application-summary.json"

    # Create HTML report
    cat > "${REPORT_DIR}/patch-report.html" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>Rust AI IDE Patch Application Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f0f0f0; padding: 10px; border-radius: 5px; }
        .success { color: green; }
        .failure { color: red; }
        .warning { color: orange; }
        .section { margin: 20px 0; padding: 10px; border: 1px solid #ccc; border-radius: 5px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Rust AI IDE Patch Application Report</h1>
        <p><strong>Generated:</strong> $(date)</p>
        <p><strong>Duration:</strong> $(($(date +%s) - START_TIME)) seconds</p>
        <p><strong>Overall Status:</strong> <span class="${overall_success,,}">${overall_success^^}</span></p>
        <p><strong>Dry Run:</strong> ${DRY_RUN}</p>
    </div>

    <div class="section">
        <h3>Patches Applied</h3>
        <p><strong>Rust:</strong> ${rust_patches}</p>
        <p><strong>NPM:</strong> ${npm_patches}</p>
    </div>

    <div class="section">
        <h3>Backup Information</h3>
        <p><strong>Backup Directory:</strong> ${BACKUP_DIR}</p>
        <p><strong>Rollback Available:</strong> $([ "$DRY_RUN" == "false" ] && echo "Yes" || echo "No (dry run)")</p>
    </div>
</body>
</html>
EOF

    log_success "Comprehensive patch report generated: ${REPORT_DIR}/patch-application-summary.json"
}

# Function to handle rollback
perform_rollback() {
    if [[ "${ROLLBACK_ON_FAILURE}" != true ]] || [[ "${DRY_RUN}" == true ]]; then
        log_info "Skipping rollback"
        return 0
    fi

    log_warning "Performing rollback due to patch application failure..."

    restore_backups
    log_success "Rollback completed"
}

# Trap to handle cleanup and rollback on exit
trap 'EXIT_CODE=$?; log_info "Patch application completed (exit code: $EXIT_CODE)"; [[ $EXIT_CODE -ne 0 ]] && perform_rollback; [[ -d "${REPORT_DIR}" ]] && echo "Reports available in: ${REPORT_DIR}"' EXIT

# Main function
main() {
    log_info "Starting automated patch application"
    log_info "Log file: ${SECURITY_LOG}"
    log_info "Report directory: ${REPORT_DIR}"
    log_info "Backup directory: ${BACKUP_DIR}"
    log_info "Dry run: ${DRY_RUN}"

    mkdir -p "${REPORT_DIR}"
    mkdir -p "${BACKUP_DIR}"

    local exit_code=0

    # Create backups
    create_backups

    # Analyze vulnerabilities
    if analyze_vulnerabilities; then
        log_info "Vulnerabilities found - proceeding with patch application"

        # Apply patches
        if ! apply_rust_patches; then
            log_error "Rust patch application failed"
            exit_code=$((exit_code + 1))
        fi

        if ! apply_npm_patches; then
            log_error "NPM patch application failed"
            exit_code=$((exit_code + 1))
        fi

        # Run tests
        if ! run_post_patch_tests; then
            log_error "Post-patch tests failed"
            exit_code=$((exit_code + 1))
        fi
    else
        log_info "No vulnerabilities requiring patching found"
    fi

    # Generate reports
    generate_patch_report

    local end_time=$(date +%s)
    log_info "Patch application completed in $((end_time - START_TIME)) seconds"

    return $exit_code
}

# Run main function
main "$@"
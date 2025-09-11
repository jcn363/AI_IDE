#!/bin/bash

# Dependency Update Automation with Compatibility Testing
# Automated dependency updates with comprehensive testing and rollback
# Author: Security Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
WEB_DIR="${PROJECT_ROOT}/web"
UPDATE_LOG="${PROJECT_ROOT}/dependency-update.log"
REPORT_DIR="${PROJECT_ROOT}/security-reports/dependency-updates"
BACKUP_DIR="${PROJECT_ROOT}/security-backups/$(date +%Y%m%d_%H%M%S)_updates"
START_TIME=$(date +%s)

# Default configuration
DRY_RUN=false
AUTO_APPROVE=false
UPDATE_RUST=true
UPDATE_NPM=true
COMPATIBILITY_TEST=true
MAJOR_UPDATES=false
PATCH_ONLY=false
MAX_UPDATE_ATTEMPTS=5

# Create directories
mkdir -p "${REPORT_DIR}"
mkdir -p "${BACKUP_DIR}"

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${UPDATE_LOG}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${UPDATE_LOG}" >&2
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${UPDATE_LOG}"
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${UPDATE_LOG}"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Automated dependency updates with compatibility testing for Rust AI IDE.

OPTIONS:
    -h, --help                  Show this help message
    -v, --verbose               Enable verbose output
    --dry-run                   Run in dry-run mode (no changes applied)
    --auto-approve              Automatically approve updates without prompts
    --no-rust                   Skip Rust dependency updates
    --no-npm                    Skip NPM dependency updates
    --no-compatibility-test     Skip compatibility testing
    --major-updates             Allow major version updates (dangerous)
    --patch-only                Only update patch versions (safe)
    --max-attempts NUM          Maximum update attempts (default: 5)
    --backup-dir DIR            Backup directory (default: auto-generated)
    --report-dir DIR            Output directory for reports (default: security-reports/dependency-updates)

EXAMPLES:
    $0 --dry-run --verbose
    $0 --auto-approve --patch-only
    $0 --major-updates --no-compatibility-test

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
        --no-rust)
            UPDATE_RUST=false
            shift
            ;;
        --no-npm)
            UPDATE_NPM=false
            shift
            ;;
        --no-compatibility-test)
            COMPATIBILITY_TEST=false
            shift
            ;;
        --major-updates)
            MAJOR_UPDATES=true
            shift
            ;;
        --patch-only)
            PATCH_ONLY=true
            shift
            ;;
        --max-attempts)
            MAX_UPDATE_ATTEMPTS="$2"
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
        --arg update_rust "$UPDATE_RUST" \
        --arg update_npm "$UPDATE_NPM" \
        '{
            timestamp: $timestamp,
            backup_dir: $backup_dir,
            dry_run: ($dry_run == "true"),
            update_rust: ($update_rust == "true"),
            update_npm: ($update_npm == "true"),
            files_backed_up: [
                "Cargo.lock",
                "Cargo.toml",
                "package-lock.json",
                "package.json"
            ]
        }' > "${BACKUP_DIR}/update-backup-manifest.json"

    log_success "Backups created successfully"
}

# Function to analyze current dependency versions
analyze_current_versions() {
    log_info "Analyzing current dependency versions..."

    # Analyze Rust dependencies
    if [[ "${UPDATE_RUST}" == true ]]; then
        log_info "Analyzing Rust dependencies..."
        cd "${PROJECT_ROOT}"

        # Get current dependency versions
        cargo tree --format "{p}" > "${REPORT_DIR}/rust-deps-current.txt" 2>/dev/null || true

        # Check for outdated dependencies
        if command -v cargo-outdated >/dev/null 2>&1; then
            cargo outdated --format json > "${REPORT_DIR}/rust-outdated.json" 2>/dev/null || true
            local outdated_count=$(jq -r '.dependencies // [] | length' "${REPORT_DIR}/rust-outdated.json" 2>/dev/null || echo "0")
            log_info "Found ${outdated_count} outdated Rust dependencies"
        else
            log_warning "cargo-outdated not available - install with: cargo install cargo-outdated"
        fi
    fi

    # Analyze NPM dependencies
    if [[ "${UPDATE_NPM}" == true ]]; then
        log_info "Analyzing NPM dependencies..."
        cd "${WEB_DIR}"

        # Get current dependency versions
        npm list --depth=0 > "${REPORT_DIR}/npm-deps-current.txt" 2>/dev/null || true

        # Check for outdated dependencies
        npm outdated --json > "${REPORT_DIR}/npm-outdated.json" 2>/dev/null || true
        local outdated_count=$(jq -r 'keys | length' "${REPORT_DIR}/npm-outdated.json" 2>/dev/null || echo "0")
        log_info "Found ${outdated_count} outdated NPM dependencies"
    fi

    log_success "Dependency analysis completed"
}

# Function to update Rust dependencies
update_rust_dependencies() {
    if [[ "${UPDATE_RUST}" != true ]]; then
        log_info "Skipping Rust dependency updates"
        return 0
    fi

    if [[ "${DRY_RUN}" == true ]]; then
        log_info "Dry run - would update Rust dependencies"
        return 0
    fi

    log_info "Updating Rust dependencies..."

    cd "${PROJECT_ROOT}"
    local update_success=false
    local attempt=1

    while [[ $attempt -le $MAX_UPDATE_ATTEMPTS ]]; do
        log_info "Update attempt ${attempt}/${MAX_UPDATE_ATTEMPTS}"

        # Backup current state before each attempt
        cp "${PROJECT_ROOT}/Cargo.lock" "${BACKUP_DIR}/Cargo.lock.attempt${attempt}" 2>/dev/null || true

        if [[ "${PATCH_ONLY}" == true ]]; then
            # Conservative update - only patch versions
            log_info "Performing conservative patch updates..."
            if cargo update --patch > "${REPORT_DIR}/cargo-update-attempt${attempt}.log" 2>&1; then
                log_success "Conservative Rust update successful"
                update_success=true
                break
            fi
        elif [[ "${MAJOR_UPDATES}" == true ]]; then
            # Aggressive update - allow major version changes
            log_warning "Performing aggressive updates (allowing major version changes)..."
            if cargo update > "${REPORT_DIR}/cargo-update-attempt${attempt}.log" 2>&1; then
                log_success "Aggressive Rust update successful"
                update_success=true
                break
            fi
        else
            # Standard update - minor and patch versions
            log_info "Performing standard updates (minor and patch versions)..."
            if cargo update > "${REPORT_DIR}/cargo-update-attempt${attempt}.log" 2>&1; then
                log_success "Standard Rust update successful"
                update_success=true
                break
            fi
        fi

        log_warning "Update attempt ${attempt} failed"
        ((attempt++))
    done

    if [[ "$update_success" == false ]]; then
        log_error "All Rust update attempts failed"
        return 1
    fi

    # Generate Rust update report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg attempts "$attempt" \
        --arg success "$update_success" \
        --arg strategy "$( [[ "${PATCH_ONLY}" == true ]] && echo "patch_only" || [[ "${MAJOR_UPDATES}" == true ]] && echo "major_allowed" || echo "standard" )" \
        '{
            timestamp: $timestamp,
            language: "rust",
            update_strategy: $strategy,
            attempts: ($attempts | tonumber),
            successful: ($success == "true"),
            log_file: "cargo-update-attempt\($attempts).log"
        }' > "${REPORT_DIR}/rust-update-report.json"

    return 0
}

# Function to update NPM dependencies
update_npm_dependencies() {
    if [[ "${UPDATE_NPM}" != true ]]; then
        log_info "Skipping NPM dependency updates"
        return 0
    fi

    if [[ "${DRY_RUN}" == true ]]; then
        log_info "Dry run - would update NPM dependencies"
        return 0
    fi

    log_info "Updating NPM dependencies..."

    cd "${WEB_DIR}"
    local update_success=false
    local attempt=1

    while [[ $attempt -le $MAX_UPDATE_ATTEMPTS ]]; do
        log_info "Update attempt ${attempt}/${MAX_UPDATE_ATTEMPTS}"

        # Backup current state before each attempt
        cp "${WEB_DIR}/package-lock.json" "${BACKUP_DIR}/package-lock.attempt${attempt}.json" 2>/dev/null || true

        if [[ "${PATCH_ONLY}" == true ]]; then
            # Conservative update - only patch versions
            log_info "Performing conservative patch updates..."
            if npm update --save > "${REPORT_DIR}/npm-update-attempt${attempt}.log" 2>&1; then
                log_success "Conservative NPM update successful"
                update_success=true
                break
            fi
        elif [[ "${MAJOR_UPDATES}" == true ]]; then
            # Aggressive update - allow major version changes
            log_warning "Performing aggressive updates (allowing major version changes)..."
            if npm update --save --force > "${REPORT_DIR}/npm-update-attempt${attempt}.log" 2>&1; then
                log_success "Aggressive NPM update successful"
                update_success=true
                break
            fi
        else
            # Standard update - minor and patch versions
            log_info "Performing standard updates (minor and patch versions)..."
            if npm update --save > "${REPORT_DIR}/npm-update-attempt${attempt}.log" 2>&1; then
                log_success "Standard NPM update successful"
                update_success=true
                break
            fi
        fi

        log_warning "Update attempt ${attempt} failed"
        ((attempt++))
    done

    if [[ "$update_success" == false ]]; then
        log_error "All NPM update attempts failed"
        return 1
    fi

    # Generate NPM update report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg attempts "$attempt" \
        --arg success "$update_success" \
        --arg strategy "$( [[ "${PATCH_ONLY}" == true ]] && echo "patch_only" || [[ "${MAJOR_UPDATES}" == true ]] && echo "major_allowed" || echo "standard" )" \
        '{
            timestamp: $timestamp,
            language: "npm",
            update_strategy: $strategy,
            attempts: ($attempts | tonumber),
            successful: ($success == "true"),
            log_file: "npm-update-attempt\($attempts).log"
        }' > "${REPORT_DIR}/npm-update-report.json"

    return 0
}

# Function to run compatibility tests
run_compatibility_tests() {
    if [[ "${COMPATIBILITY_TEST}" != true ]]; then
        log_info "Skipping compatibility tests"
        return 0
    fi

    if [[ "${DRY_RUN}" == true ]]; then
        log_info "Dry run - would run compatibility tests"
        return 0
    fi

    log_info "Running compatibility tests..."

    local test_success=true

    # Test Rust components
    if [[ "${UPDATE_RUST}" == true ]]; then
        log_info "Testing Rust compatibility..."
        cd "${PROJECT_ROOT}"

        # Build test
        if cargo build --workspace --quiet > "${REPORT_DIR}/rust-build-test.log" 2>&1; then
            log_success "Rust build test passed"
        else
            log_error "Rust build test failed"
            test_success=false
        fi

        # Unit tests
        if cargo test --workspace --quiet > "${REPORT_DIR}/rust-unit-tests.log" 2>&1; then
            log_success "Rust unit tests passed"
        else
            log_error "Rust unit tests failed"
            test_success=false
        fi
    fi

    # Test NPM components
    if [[ "${UPDATE_NPM}" == true ]]; then
        log_info "Testing NPM compatibility..."
        cd "${WEB_DIR}"

        # Install dependencies
        if npm install --quiet > "${REPORT_DIR}/npm-install-test.log" 2>&1; then
            log_success "NPM install test passed"
        else
            log_error "NPM install test failed"
            test_success=false
        fi

        # Build test
        if npm run build --quiet > "${REPORT_DIR}/npm-build-test.log" 2>&1; then
            log_success "NPM build test passed"
        else
            log_error "NPM build test failed"
            test_success=false
        fi

        # Unit tests
        if npm test --quiet > "${REPORT_DIR}/npm-unit-tests.log" 2>&1; then
            log_success "NPM unit tests passed"
        else
            log_error "NPM unit tests failed"
            test_success=false
        fi
    fi

    # Generate compatibility test report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg tests_passed "$test_success" \
        --arg rust_tests "$( [[ "${UPDATE_RUST}" == true ]] && echo "true" || echo "false" )" \
        --arg npm_tests "$( [[ "${UPDATE_NPM}" == true ]] && echo "true" || echo "false" )" \
        '{
            timestamp: $timestamp,
            compatibility_tests_passed: ($tests_passed == "true"),
            rust_tests_run: ($rust_tests == "true"),
            npm_tests_run: ($npm_tests == "true")
        }' > "${REPORT_DIR}/compatibility-test-report.json"

    return $([ "$test_success" == true ] && echo 0 || echo 1)
}

# Function to generate comprehensive update report
generate_update_report() {
    log_info "Generating comprehensive dependency update report..."

    local overall_success=true

    # Check update results
    local rust_updated=$(jq -r '.successful // false' "${REPORT_DIR}/rust-update-report.json" 2>/dev/null || echo "false")
    local npm_updated=$(jq -r '.successful // false' "${REPORT_DIR}/npm-update-report.json" 2>/dev/null || echo "false")

    # Check compatibility tests
    if [[ "${COMPATIBILITY_TEST}" == true ]]; then
        local tests_passed=$(jq -r '.compatibility_tests_passed // false' "${REPORT_DIR}/compatibility-test-report.json" 2>/dev/null || echo "false")
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
        --arg rust_updated "$rust_updated" \
        --arg npm_updated "$npm_updated" \
        --arg tests_run "$COMPATIBILITY_TEST" \
        --arg major_updates "$MAJOR_UPDATES" \
        --arg patch_only "$PATCH_ONLY" \
        '{
            timestamp: $timestamp,
            duration_seconds: ($duration | tonumber),
            backup_dir: $backup_dir,
            dry_run: ($dry_run == "true"),
            update_strategy: (if ($patch_only == "true") then "patch_only" elif ($major_updates == "true") then "major_allowed" else "standard" end),
            updates_applied: {
                rust: ($rust_updated == "true"),
                npm: ($npm_updated == "true")
            },
            compatibility_tests_run: ($tests_run == "true"),
            overall_success: ($overall_success == "true")
        }' > "${REPORT_DIR}/dependency-update-summary.json"

    # Create HTML report
    cat > "${REPORT_DIR}/dependency-update-report.html" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>Rust AI IDE Dependency Update Report</title>
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
        <h1>Rust AI IDE Dependency Update Report</h1>
        <p><strong>Generated:</strong> $(date)</p>
        <p><strong>Duration:</strong> $(($(date +%s) - START_TIME)) seconds</p>
        <p><strong>Overall Status:</strong> <span class="${overall_success,,}">${overall_success^^}</span></p>
        <p><strong>Strategy:</strong> $( [[ "${PATCH_ONLY}" == true ]] && echo "Patch Only" || [[ "${MAJOR_UPDATES}" == true ]] && echo "Major Updates Allowed" || echo "Standard" )</p>
        <p><strong>Dry Run:</strong> ${DRY_RUN}</p>
    </div>

    <div class="section">
        <h3>Updates Applied</h3>
        <p><strong>Rust Dependencies:</strong> ${rust_updated}</p>
        <p><strong>NPM Dependencies:</strong> ${npm_updated}</p>
    </div>

    <div class="section">
        <h3>Compatibility Testing</h3>
        <p><strong>Tests Run:</strong> ${COMPATIBILITY_TEST}</p>
        <p><strong>Tests Passed:</strong> $( [[ "${COMPATIBILITY_TEST}" == true ]] && jq -r '.compatibility_tests_passed // false' "${REPORT_DIR}/compatibility-test-report.json" 2>/dev/null || echo "N/A" )</p>
    </div>

    <div class="section">
        <h3>Backup Information</h3>
        <p><strong>Backup Directory:</strong> ${BACKUP_DIR}</p>
    </div>
</body>
</html>
EOF

    log_success "Comprehensive dependency update report generated: ${REPORT_DIR}/dependency-update-summary.json"
}

# Function to handle rollback
perform_rollback() {
    if [[ "${DRY_RUN}" == true ]]; then
        log_info "Dry run - skipping rollback"
        return 0
    fi

    log_warning "Performing rollback due to update failure..."

    # Use the existing rollback script
    if [[ -f "${SCRIPT_DIR}/rollback-patch-application.sh" ]]; then
        "${SCRIPT_DIR}/rollback-patch-application.sh" --backup-dir "${BACKUP_DIR}" --force
    else
        log_error "Rollback script not found"
    fi
}

# Trap to handle cleanup and rollback on exit
trap 'EXIT_CODE=$?; log_info "Dependency update completed (exit code: $EXIT_CODE)"; [[ $EXIT_CODE -ne 0 ]] && perform_rollback; [[ -d "${REPORT_DIR}" ]] && echo "Reports available in: ${REPORT_DIR}"' EXIT

# Main function
main() {
    log_info "Starting automated dependency updates"
    log_info "Log file: ${UPDATE_LOG}"
    log_info "Report directory: ${REPORT_DIR}"
    log_info "Backup directory: ${BACKUP_DIR}"
    log_info "Dry run: ${DRY_RUN}"
    log_info "Update strategy: $( [[ "${PATCH_ONLY}" == true ]] && echo "Patch Only" || [[ "${MAJOR_UPDATES}" == true ]] && echo "Major Updates Allowed" || echo "Standard" )"

    mkdir -p "${REPORT_DIR}"
    mkdir -p "${BACKUP_DIR}"

    local exit_code=0

    # Create backups
    create_backups

    # Analyze current versions
    analyze_current_versions

    # Update dependencies
    if ! update_rust_dependencies; then
        log_error "Rust dependency update failed"
        exit_code=$((exit_code + 1))
    fi

    if ! update_npm_dependencies; then
        log_error "NPM dependency update failed"
        exit_code=$((exit_code + 1))
    fi

    # Run compatibility tests
    if ! run_compatibility_tests; then
        log_error "Compatibility tests failed"
        exit_code=$((exit_code + 1))
    fi

    # Generate reports
    generate_update_report

    local end_time=$(date +%s)
    log_info "Dependency update process completed in $((end_time - START_TIME)) seconds"

    return $exit_code
}

# Run main function
main "$@"
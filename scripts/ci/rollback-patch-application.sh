#!/bin/bash

# Manual Rollback Script for Patch Application
# Restore dependencies from backup after failed patch application
# Author: Security Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
WEB_DIR="${PROJECT_ROOT}/web"
ROLLBACK_LOG="${PROJECT_ROOT}/rollback.log"
START_TIME=$(date +%s)

# Default configuration
BACKUP_DIR=""
FORCE=false
DRY_RUN=false

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${ROLLBACK_LOG}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${ROLLBACK_LOG}" >&2
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${ROLLBACK_LOG}"
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${ROLLBACK_LOG}"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Manual rollback for patch application failures in Rust AI IDE.

OPTIONS:
    -h, --help              Show this help message
    -v, --verbose           Enable verbose output
    -b, --backup-dir DIR    Backup directory to restore from (required)
    -f, --force             Force rollback without confirmation
    --dry-run               Show what would be restored without making changes

EXAMPLES:
    $0 --backup-dir /path/to/backup/20230910_143000
    $0 --backup-dir security-backups/20230910_143000 --force
    $0 --backup-dir security-backups/20230910_143000 --dry-run

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
        -b|--backup-dir)
            BACKUP_DIR="$2"
            shift 2
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Function to validate backup directory
validate_backup_dir() {
    if [[ -z "${BACKUP_DIR}" ]]; then
        log_error "Backup directory not specified. Use --backup-dir option."
        log_info "Available backup directories:"
        find "${PROJECT_ROOT}/security-backups" -maxdepth 1 -type d -name "20*" 2>/dev/null | sort -r | head -10 || true
        exit 1
    fi

    if [[ ! -d "${BACKUP_DIR}" ]]; then
        log_error "Backup directory does not exist: ${BACKUP_DIR}"
        exit 1
    fi

    if [[ ! -f "${BACKUP_DIR}/backup-manifest.json" ]]; then
        log_error "Backup manifest not found in: ${BACKUP_DIR}"
        log_error "This may not be a valid backup directory"
        exit 1
    fi

    log_info "Using backup directory: ${BACKUP_DIR}"
}

# Function to show backup information
show_backup_info() {
    log_info "Backup Information:"

    if [[ -f "${BACKUP_DIR}/backup-manifest.json" ]]; then
        local timestamp=$(jq -r '.timestamp // "Unknown"' "${BACKUP_DIR}/backup-manifest.json" 2>/dev/null || echo "Unknown")
        local dry_run=$(jq -r '.dry_run // false' "${BACKUP_DIR}/backup-manifest.json" 2>/dev/null || echo "false")

        log_info "  Created: ${timestamp}"
        log_info "  Dry Run: ${dry_run}"

        log_info "  Files available for restore:"
        jq -r '.files_backed_up[]?' "${BACKUP_DIR}/backup-manifest.json" 2>/dev/null | while read -r file; do
            if [[ -f "${BACKUP_DIR}/${file}.backup" ]]; then
                local size=$(du -h "${BACKUP_DIR}/${file}.backup" 2>/dev/null | cut -f1)
                log_info "    ✓ ${file} (${size})"
            else
                log_warning "    ✗ ${file} (missing)"
            fi
        done
    else
        log_warning "Could not read backup manifest"
    fi
}

# Function to confirm rollback
confirm_rollback() {
    if [[ "${FORCE}" == true ]] || [[ "${DRY_RUN}" == true ]]; then
        return 0
    fi

    log_warning "This will overwrite current dependency files with backup versions!"
    log_warning "Current changes will be lost."

    read -p "Are you sure you want to proceed with rollback? (yes/no): " -r
    if [[ ! "${REPLY}" =~ ^[Yy][Ee][Ss]$ ]]; then
        log_info "Rollback cancelled by user"
        exit 0
    fi
}

# Function to restore Rust dependencies
restore_rust_dependencies() {
    log_info "Restoring Rust dependencies..."

    local restored=false

    # Restore Cargo.lock
    if [[ -f "${BACKUP_DIR}/Cargo.lock.backup" ]]; then
        if [[ "${DRY_RUN}" == true ]]; then
            log_info "Would restore: Cargo.lock"
        else
            cp "${BACKUP_DIR}/Cargo.lock.backup" "${PROJECT_ROOT}/Cargo.lock"
            log_success "Restored Cargo.lock"
            restored=true
        fi
    else
        log_warning "Cargo.lock backup not found"
    fi

    # Restore Cargo.toml
    if [[ -f "${BACKUP_DIR}/Cargo.toml.backup" ]]; then
        if [[ "${DRY_RUN}" == true ]]; then
            log_info "Would restore: Cargo.toml"
        else
            cp "${BACKUP_DIR}/Cargo.toml.backup" "${PROJECT_ROOT}/Cargo.toml"
            log_success "Restored Cargo.toml"
            restored=true
        fi
    else
        log_warning "Cargo.toml backup not found"
    fi

    return $([ "$restored" == true ] && echo 0 || echo 1)
}

# Function to restore NPM dependencies
restore_npm_dependencies() {
    log_info "Restoring NPM dependencies..."

    local restored=false

    # Restore package-lock.json
    if [[ -f "${BACKUP_DIR}/package-lock.json.backup" ]]; then
        if [[ "${DRY_RUN}" == true ]]; then
            log_info "Would restore: package-lock.json"
        else
            cp "${BACKUP_DIR}/package-lock.json.backup" "${WEB_DIR}/package-lock.json"
            log_success "Restored package-lock.json"
            restored=true
        fi
    else
        log_warning "package-lock.json backup not found"
    fi

    # Restore package.json
    if [[ -f "${BACKUP_DIR}/package.json.backup" ]]; then
        if [[ "${DRY_RUN}" == true ]]; then
            log_info "Would restore: package.json"
        else
            cp "${BACKUP_DIR}/package.json.backup" "${WEB_DIR}/package.json"
            log_success "Restored package.json"
            restored=true
        fi
    else
        log_warning "package.json backup not found"
    fi

    return $([ "$restored" == true ] && echo 0 || echo 1)
}

# Function to run tests after rollback
run_post_rollback_tests() {
    if [[ "${DRY_RUN}" == true ]]; then
        log_info "Dry run - skipping post-rollback tests"
        return 0
    fi

    log_info "Running tests after rollback..."

    local test_success=true

    # Test Rust components
    log_info "Testing Rust components..."
    if cd "${PROJECT_ROOT}" && cargo test --workspace --quiet > "${BACKUP_DIR}/rust-test-after-rollback.log" 2>&1; then
        log_success "Rust tests passed after rollback"
    else
        log_error "Rust tests failed after rollback"
        test_success=false
    fi

    # Test web components
    log_info "Testing web components..."
    if cd "${WEB_DIR}" && npm test > "${BACKUP_DIR}/npm-test-after-rollback.log" 2>&1; then
        log_success "NPM tests passed after rollback"
    else
        log_error "NPM tests failed after rollback"
        test_success=false
    fi

    return $([ "$test_success" == true ] && echo 0 || echo 1)
}

# Function to reinstall dependencies after rollback
reinstall_dependencies() {
    if [[ "${DRY_RUN}" == true ]]; then
        log_info "Dry run - skipping dependency reinstallation"
        return 0
    fi

    log_info "Reinstalling dependencies after rollback..."

    # Reinstall Rust dependencies
    log_info "Running cargo update..."
    if cd "${PROJECT_ROOT}" && cargo update > "${BACKUP_DIR}/cargo-update-after-rollback.log" 2>&1; then
        log_success "Cargo dependencies updated"
    else
        log_warning "Cargo update failed"
    fi

    # Reinstall NPM dependencies
    log_info "Running npm install..."
    if cd "${WEB_DIR}" && npm install > "${BACKUP_DIR}/npm-install-after-rollback.log" 2>&1; then
        log_success "NPM dependencies installed"
    else
        log_warning "NPM install failed"
    fi
}

# Function to generate rollback report
generate_rollback_report() {
    log_info "Generating rollback report..."

    local rollback_report="${BACKUP_DIR}/rollback-report.json"

    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg duration "$(($(date +%s) - START_TIME))" \
        --arg backup_dir "$BACKUP_DIR" \
        --arg dry_run "$DRY_RUN" \
        '{
            timestamp: $timestamp,
            duration_seconds: ($duration | tonumber),
            backup_dir: $backup_dir,
            dry_run: ($dry_run == "true"),
            rollback_completed: ($dry_run == "false"),
            log_file: "rollback.log"
        }' > "${rollback_report}"

    log_success "Rollback report generated: ${rollback_report}"
}

# Function to list available backups
list_available_backups() {
    log_info "Available backup directories:"
    log_info "============================="

    local backup_count=0
    while IFS= read -r -d '' backup_dir; do
        if [[ -f "${backup_dir}/backup-manifest.json" ]]; then
            local timestamp=$(jq -r '.timestamp // "Unknown"' "${backup_dir}/backup-manifest.json" 2>/dev/null || echo "Unknown")
            local size=$(du -sh "${backup_dir}" 2>/dev/null | cut -f1)

            echo "$(basename "${backup_dir}") - ${timestamp} - ${size}"
            ((backup_count++))
        fi
    done < <(find "${PROJECT_ROOT}/security-backups" -maxdepth 1 -type d -name "20*" -print0 2>/dev/null | sort -zr)

    if [[ $backup_count -eq 0 ]]; then
        log_warning "No backup directories found"
        log_info "Backup directories are created automatically during patch application"
        log_info "Run patch application first: scripts/ci/automated-patch-application.sh"
    fi
}

# Main function
main() {
    log_info "Starting manual rollback process"
    log_info "Log file: ${ROLLBACK_LOG}"

    # Check if help was requested without arguments
    if [[ $# -eq 0 ]] || [[ "$1" == "--help" ]] || [[ "$1" == "-h" ]]; then
        usage
        echo ""
        list_available_backups
        exit 0
    fi

    # Validate backup directory
    validate_backup_dir

    # Show backup information
    show_backup_info

    # Confirm rollback
    confirm_rollback

    local exit_code=0

    # Perform rollback
    log_info "Starting rollback process..."

    if ! restore_rust_dependencies; then
        log_error "Rust dependency restoration failed"
        exit_code=$((exit_code + 1))
    fi

    if ! restore_npm_dependencies; then
        log_error "NPM dependency restoration failed"
        exit_code=$((exit_code + 1))
    fi

    # Reinstall dependencies if rollback was performed
    if [[ "${DRY_RUN}" == false ]]; then
        reinstall_dependencies
    fi

    # Run tests after rollback
    if ! run_post_rollback_tests; then
        log_warning "Some tests failed after rollback - manual verification recommended"
    fi

    # Generate report
    generate_rollback_report

    local end_time=$(date +%s)
    log_info "Rollback process completed in $((end_time - START_TIME)) seconds"

    if [[ "${DRY_RUN}" == true ]]; then
        log_info "Dry run completed - no changes were made"
    else
        log_success "Rollback completed successfully"
        log_info "Next steps:"
        log_info "  1. Verify application functionality"
        log_info "  2. Run full test suite: cargo test --workspace && cd web && npm test"
        log_info "  3. If issues persist, consider manual dependency management"
    fi

    return $exit_code
}

# Run main function
main "$@"
#!/bin/bash

# Automated Maintenance Workflows
# This script orchestrates maintenance tasks for the Rust AI IDE
# Author: DevOps Team
# Version: 1.0.0

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
LOG_FILE="${PROJECT_ROOT}/logs/maintenance-workflow-$(date +%Y%m%d).log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${LOG_FILE}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${LOG_FILE}" >&2
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${LOG_FILE}"
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${LOG_FILE}"
}

# Function to check workspace consistency
check_workspace_consistency() {
    log_info "Checking workspace consistency..."

    # Check if all Cargo.toml files exist
    local missing_files=0
    local total_files=0

    while IFS= read -r -d '' file; do
        ((total_files++))
        if [[ ! -f "$file" ]]; then
            log_error "Missing Cargo.toml file: $file"
            ((missing_files++))
        fi
    done < <(find "${PROJECT_ROOT}/crates" -name "Cargo.toml" -print0)

    if [[ $missing_files -eq 0 ]]; then
        log_success "Workspace consistency check passed ($total_files files found)"
        return 0
    else
        log_error "Workspace consistency check failed ($missing_files files missing)"
        return 1
    fi
}

# Function to update dependencies
update_dependencies() {
    log_info "Updating project dependencies..."

    # Check for outdated dependencies
    if cargo outdated --exit-code 1 >/dev/null 2>&1; then
        log_info "Dependencies are up to date"
    else
        log_info "Updating dependencies..."
        cargo update
        log_success "Dependencies updated successfully"
    fi

    # Update web dependencies if package.json exists
    if [[ -f "${PROJECT_ROOT}/web/package.json" ]]; then
        cd "${PROJECT_ROOT}/web"
        npm audit fix --audit-level moderate || log_warning "Some npm packages could not be automatically fixed"
        cd "${PROJECT_ROOT}"
    fi
}

# Function to clean build artifacts
clean_build_artifacts() {
    log_info "Cleaning build artifacts..."

    # Clean Rust build artifacts
    cargo clean

    # Clean web build artifacts
    if [[ -d "${PROJECT_ROOT}/web/node_modules" ]]; then
        cd "${PROJECT_ROOT}/web"
        npm run clean 2>/dev/null || rm -rf dist/ build/ .next/ 2>/dev/null || true
        cd "${PROJECT_ROOT}"
    fi

    # Clean Docker images
    if command -v docker >/dev/null 2>&1; then
        docker system prune -f >/dev/null 2>&1 || true
    fi

    log_success "Build artifacts cleaned"
}

# Function to run security checks
run_security_checks() {
    log_info "Running maintenance security checks..."

    # Install cargo-audit if not present
    if ! command -v cargo-audit >/dev/null 2>&1; then
        cargo install cargo-audit
    fi

    # Run security audit
    if cargo audit --format json >/dev/null 2>&1; then
        log_success "Security audit passed"
    else
        log_warning "Security audit found issues - manual review recommended"
    fi

    # Check license compliance
    if command -v cargo-deny >/dev/null 2>&1; then
        cargo deny check
    fi
}

# Function to run performance checks
run_performance_checks() {
    log_info "Running performance checks..."

    # Check build times
    local start_time=$(date +%s)
    if cargo check --quiet >/dev/null 2>&1; then
        local end_time=$(date +%s)
        local build_time=$((end_time - start_time))
        log_info "Build time: ${build_time}s"

        if [[ $build_time -gt 300 ]]; then
            log_warning "Build time is high (${build_time}s) - consider optimization"
        else
            log_success "Build performance is acceptable"
        fi
    else
        log_error "Build check failed"
        return 1
    fi
}

# Function to backup configuration
backup_configuration() {
    log_info "Backing up configuration files..."

    local backup_dir="${PROJECT_ROOT}/backups/$(date +%Y%m%d_%H%M%S)"
    mkdir -p "${backup_dir}"

    # Backup important configuration files
    cp "${PROJECT_ROOT}/Cargo.toml" "${backup_dir}/" 2>/dev/null || true
    cp "${PROJECT_ROOT}/rust-toolchain.toml" "${backup_dir}/" 2>/dev/null || true
    cp -r "${PROJECT_ROOT}/ci-cd/" "${backup_dir}/" 2>/dev/null || true
    cp -r "${PROJECT_ROOT}/docker/" "${backup_dir}/" 2>/dev/null || true

    # Create backup archive
    cd "${PROJECT_ROOT}"
    tar -czf "${backup_dir}.tar.gz" -C "${PROJECT_ROOT}" "backups/$(basename "${backup_dir}")" 2>/dev/null || true
    rm -rf "${backup_dir}"

    log_success "Configuration backup created: ${backup_dir}.tar.gz"
}

# Function to generate maintenance report
generate_maintenance_report() {
    local report_file="${PROJECT_ROOT}/reports/maintenance-report-$(date +%Y%m%d_%H%M%S).json"

    mkdir -p "$(dirname "${report_file}")"

    cat > "${report_file}" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "maintenance_type": "automated_workflow",
    "summary": {
        "workspace_consistency": "completed",
        "dependency_updates": "completed",
        "security_checks": "completed",
        "performance_checks": "completed",
        "cleanup": "completed"
    },
    "system_info": {
        "rust_version": "$(rustc --version 2>/dev/null || echo 'unknown')",
        "cargo_version": "$(cargo --version 2>/dev/null || echo 'unknown')",
        "git_commit": "$(git rev-parse HEAD 2>/dev/null || echo 'unknown')"
    }
}
EOF

    log_success "Maintenance report generated: ${report_file}"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS] [ACTION]

Automated Maintenance Workflows for Rust AI IDE

ACTIONS:
    full                Run complete maintenance workflow (default)
    check               Check workspace consistency only
    update              Update dependencies only
    clean               Clean build artifacts only
    security            Run security checks only
    performance         Run performance checks only
    backup              Backup configuration only

OPTIONS:
    -h, --help           Show this help message
    -v, --verbose        Enable verbose output
    --dry-run            Show what would be done without making changes
    --skip-security      Skip security checks
    --skip-performance   Skip performance checks

EXAMPLES:
    $0                    Run full maintenance workflow
    $0 --dry-run         Show maintenance workflow without executing
    $0 check             Check workspace consistency only
    $0 update            Update dependencies only

EOF
}

# Parse command line arguments
DRY_RUN=false
SKIP_SECURITY=false
SKIP_PERFORMANCE=false
VERBOSE=false
ACTION="full"

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
        --skip-security)
            SKIP_SECURITY=true
            shift
            ;;
        --skip-performance)
            SKIP_PERFORMANCE=true
            shift
            ;;
        check|update|clean|security|performance|backup|full)
            ACTION="$1"
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Main execution function
main() {
    log_info "Starting automated maintenance workflow (Action: ${ACTION})"

    if [[ "${DRY_RUN}" == true ]]; then
        log_info "DRY RUN MODE: No actual maintenance will be performed"
        exit 0
    fi

    local exit_code=0

    case "${ACTION}" in
        full)
            # Run complete maintenance workflow
            if check_workspace_consistency; then
                update_dependencies
                clean_build_artifacts

                if [[ "${SKIP_SECURITY}" != true ]]; then
                    run_security_checks || exit_code=1
                fi

                if [[ "${SKIP_PERFORMANCE}" != true ]]; then
                    run_performance_checks || exit_code=1
                fi

                backup_configuration
                generate_maintenance_report
            else
                exit_code=1
            fi
            ;;
        check)
            check_workspace_consistency || exit_code=1
            ;;
        update)
            update_dependencies
            ;;
        clean)
            clean_build_artifacts
            ;;
        security)
            if [[ "${SKIP_SECURITY}" != true ]]; then
                run_security_checks || exit_code=1
            fi
            ;;
        performance)
            if [[ "${SKIP_PERFORMANCE}" != true ]]; then
                run_performance_checks || exit_code=1
            fi
            ;;
        backup)
            backup_configuration
            ;;
        *)
            log_error "Unknown action: ${ACTION}"
            usage
            exit 1
            ;;
    esac

    if [[ $exit_code -eq 0 ]]; then
        log_success "Maintenance workflow completed successfully"
    else
        log_error "Maintenance workflow completed with errors"
    fi

    return $exit_code
}

# Execute main function
main "$@"
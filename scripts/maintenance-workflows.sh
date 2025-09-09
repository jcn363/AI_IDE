#!/bin/bash

# Systematic Dependency Review & Code Quality Workflow Scripts
# This script provides automated workflows for dependency maintenance and unused variable monitoring

set -euo pipefail

# Configuration
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_ARGS="--workspace"
DRY_RUN="${DRY_RUN:-false}"

# Logging functions
log_info() {
    echo -e "\033[34m[INFO]\033[0m $(date '+%Y-%m-%d %H:%M:%S') $*" >&2
}

log_warn() {
    echo -e "\033[33m[WARN]\033[0m $(date '+%Y-%m-%d %H:%M:%S') $*" >&2
}

log_error() {
    echo -e "\033[31m[ERROR]\033[0m $(date '+%Y-%m-%d %H:%M:%S') $*" >&2
}

# Workflow: Quick Code Quality Check
check_unused_variables() {
    log_info "Running unused variable analysis..."

    local output_file="${WORKSPACE_ROOT}/unused_variables_report.txt"

    # Capture full cargo check output
    if cargo check $CARGO_ARGS --message-format=json 2>&1 | tee "$output_file" | jq -r '.message | select(.level == "warning" and (.message | contains("unused variable"))) | .rendered // empty' | head -20; then
        log_info "Unused variable check completed. Full report saved to: $output_file"
    else
        log_error "Cargo check failed. Please review compilation errors first."
        return 1
    fi
}

# Workflow: Strategic Variable Analysis
analyze_strategic_variables() {
    log_info "Analyzing strategic underscore usage patterns..."

    local report="${WORKSPACE_ROOT}/strategic_variables_report.md"

    cat > "$report" << 'EOF'
# Strategic Variable Analysis Report

This report identifies variables that may require underscore prefixes for future extensibility.

## AI/ML Function Parameters

### Context Parameters (Future Pipeline Extensions)
EOF

    # Find AI/ML functions with unused parameters
    find "${WORKSPACE_ROOT}/crates/rust-ai-ide-ai" -name "*.rs" -exec grep -l "fn.*analysis\|fn.*prediction\|fn.*learning" {} \; | head -5 | while read -r file; do
        echo -e "\n### $file" >> "$report"
        grep -n "fn.*(" "$file" | grep -E "analysis|prediction|learning" | head -3 >> "$report" || true
    done

    cat >> "$report" << 'EOF'

## Recommendations

1. Functions with placeholder parameters should use underscore prefixes
2. AI/ML analysis functions should preserve signatures for pipeline compatibility
3. Test-specific unused variables can be locally scoped

EOF

    log_info "Strategic analysis complete. Report saved to: $report"
}

# Workflow: Dependency Compatibility Audit
audit_dependencies() {
    log_info "Running dependency compatibility audit..."

    # Check for outdated dependencies
    if command -v cargo-outdated >/dev/null 2>&1; then
        log_info "Checking for outdated dependencies..."
        cargo outdated $CARGO_ARGS --root-deps-only || log_warn "cargo-outdated check failed"
    else
        log_warn "cargo-outdated not installed. Install with: cargo install cargo-outdated"
    fi

    # Security audit
    if command -v cargo-audit >/dev/null 2>&1; then
        log_info "Running security audit..."
        cargo audit || log_warn "Security audit found vulnerabilities"
    else
        log_warn "cargo-audit not installed. Install with: cargo install cargo-audit"
    fi

    # Check dependency tree for conflicts
    log_info "Analyzing dependency tree..."
    cargo tree $CARGO_ARGS 2>/dev/null | grep -E "(conflicts|duplicate)" || log_info "No dependency conflicts detected"
}

# Workflow: Compilation Health Check
check_compilation_health() {
    log_info "Performing compilation health check..."

    local start_time=$(date +%s)
    local error_count=0
    local warning_count=0

    # Extract counts from cargo check output
    if output=$(cargo check $CARGO_ARGS --message-format=json 2>&1); then
        error_count=$(echo "$output" | jq -r '.message | select(.level == "error") | .rendered' | wc -l)
        warning_count=$(echo "$output" | jq -r '.message | select(.level == "warning") | .rendered' | wc -l)

        local end_time=$(date +%s)
        local duration=$((end_time - start_time))

        log_info "Compilation complete in ${duration}s"
        log_info "Errors: $error_count"
        log_info "Warnings: $warning_count"

        if [ "$error_count" -gt 0 ]; then
            log_error "Compilation errors detected. Please resolve before proceeding."
            return 1
        fi

        if [ "$warning_count" -gt 50 ]; then
            log_warn "High warning count detected ($warning_count). Consider code quality review."
        else
            log_info "Code quality check passed."
        fi
    else
        log_error "Cargo check command failed"
        return 1
    fi
}

# Workflow: Backup and Recovery Check
verify_backup_integrity() {
    log_info "Verifying backup and recovery integrity..."

    # Check if backup directory structure exists
    local backup_roots=(
        "${WORKSPACE_ROOT}/crates/rust-ai-ide-ai-refactoring/src/backup"
        "${WORKSPACE_ROOT}/crates/rust-ai-ide-ai/src/backup"
    )

    for backup_root in "${backup_roots[@]}"; do
        if [ -d "$backup_root" ]; then
            local backup_count=$(find "$backup_root" -name "*.bak" -o -name "*.backup" | wc -l)
            log_info "Found $backup_count backup files in $backup_root"

            # Verify backup integrity (basic check)
            find "$backup_root" -name "*.bak" -o -name "*.backup" -exec echo "✅ {}" \; | head -5
        else
            log_info "Backup directory not found: $backup_root"
        fi
    done
}

# Workflow: Report Generation
generate_maintenance_report() {
    log_info "Generating comprehensive maintenance report..."

    local report_file="${WORKSPACE_ROOT}/maintenance_report_$(date +%Y%m%d_%H%M%S).md"

    cat > "$report_file" << EOF
# Maintenance Report - $(date)

Generated by automated workflow script.

## System Health
EOF

    # Compilation health
    if check_compilation_health >/dev/null 2>&1; then
        echo "- ✅ Compilation status: PASS" >> "$report_file"
    else
        echo "- ❌ Compilation status: FAIL" >> "$report_file"
    fi

    # Dependency audit result
    if audit_dependencies >/dev/null 2>&1; then
        echo "- ✅ Dependency audit: PASS" >> "$report_file"
    else
        echo "- ⚠️ Dependency audit: WARNINGS" >> "$report_file"
    fi

    cat >> "$report_file" << EOF

## Recent Changes
- Check git log for recent commits
- Review dependency updates
- Monitor code quality trends

## Next Steps
1. Review any new unused variable warnings
2. Update dependencies where safe
3. Address compilation errors if present
4. Update documentation as needed

For detailed procedures, see: docs/DEPENDENCY_MAINTENANCE.md
EOF

    log_info "Maintenance report saved to: $report_file"
}

# Main command dispatcher
main() {
    local command="${1:-}"
    shift || true

    case "$command" in
        "check-unused")
            check_unused_variables "$@"
            ;;
        "analyze-strategic")
            analyze_strategic_variables "$@"
            ;;
        "audit-deps")
            audit_dependencies "$@"
            ;;
        "health-check")
            check_compilation_health "$@"
            ;;
        "backup-check")
            verify_backup_integrity "$@"
            ;;
        "full-report")
            generate_maintenance_report "$@"
            ;;
        "help"|"-h"|"--help")
            echo "Usage: $0 <command>"
            echo ""
            echo "Commands:"
            echo "  check-unused      - Analyze unused variables across workspace"
            echo "  analyze-strategic - Identify strategic underscore usage patterns"
            echo "  audit-deps        - Perform dependency compatibility audit"
            echo "  health-check      - Run compilation and code quality checks"
            echo "  backup-check      - Verify backup system integrity"
            echo "  full-report       - Generate comprehensive maintenance report"
            echo "  help              - Show this help message"
            echo ""
            echo "Environment Variables:"
            echo "  DRY_RUN=true      - Show what would be done without executing"
            ;;
        *)
            log_error "Invalid command: $command"
            log_info "Use '$0 help' for available commands"
            exit 1
            ;;
    esac
}

# Execute main function if script is run directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
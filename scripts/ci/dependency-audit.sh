#!/bin/bash

# Dependency Audit Script for Rust AI IDE
# Regular dependency audits with license compliance and security scanning
# Enhanced version building upon existing cargo-deny infrastructure
# Author: DevOps Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
AUDIT_LOG="${PROJECT_ROOT}/dependency-audit.log"
REPORT_DIR="${PROJECT_ROOT}/security-reports/dependency-audits/$(date +%Y%m%d_%H%M%S)"
START_TIME=$(date +%s)

# Create directories
mkdir -p "${REPORT_DIR}"

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${AUDIT_LOG}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${AUDIT_LOG}" >&2
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${AUDIT_LOG}"
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${AUDIT_LOG}"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Comprehensive dependency audit for Rust AI IDE with license compliance and security scanning.

OPTIONS:
    -h, --help                  Show this help message
    -v, --verbose               Enable verbose output
    --output-dir DIR            Output directory for reports (default: auto-generated)
    --schedule-daily            Run daily audit checks
    --schedule-weekly           Run comprehensive weekly audit
    --critical-only             Check only critical security issues
    --all-crates                Audit all 67 crates individually
    --security-only             Skip license checks, focus on security

EXAMPLES:
    $0 --verbose --all-crates
    $0 --schedule-weekly --output-dir /tmp/audit-results
    $0 --critical-only --security-only

EOF
}

# Parse command line arguments
VERBOSE=false
SCHEDULE_DAILY=false
SCHEDULE_WEEKLY=false
CRITICAL_ONLY=false
ALL_CRATES=false
SECURITY_ONLY=false
OUTPUT_DIR="${REPORT_DIR}"

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
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --schedule-daily)
            SCHEDULE_DAILY=true
            shift
            ;;
        --schedule-weekly)
            SCHEDULE_WEEKLY=true
            shift
            ;;
        --critical-only)
            CRITICAL_ONLY=true
            shift
            ;;
        --all-crates)
            ALL_CRATES=true
            shift
            ;;
        --security-only)
            SECURITY_ONLY=true
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Function to run comprehensive security audits
run_security_audit() {
    log_info "Running comprehensive security audit..."

    local security_report="${OUTPUT_DIR}/security-audit-report.json"
    local vulnerabilities_found=0

    # Enhanced cargo-audit with nightly toolchain
    if [[ "${SCHEDULE_WEEKLY}" == true ]] || [[ "${CRITICAL_ONLY}" == false ]]; then
        log_info "Running cargo-audit with nightly toolchain..."

        if command -v cargo-audit >/dev/null 2>&1; then
            cd "${PROJECT_ROOT}"

            # Run audit with enhanced options
            if cargo +nightly audit --format json --deny warnings > "${OUTPUT_DIR}/audit-results.json" 2>"${OUTPUT_DIR}/audit-errors.log"; then
                local vuln_count=$(jq -r '.vulnerabilities.count // 0' "${OUTPUT_DIR}/audit-results.json" 2>/dev/null || echo "0")
                local warnings_count=$(jq -r '.warnings // [] | length' "${OUTPUT_DIR}/audit-results.json" 2>/dev/null || echo "0")

                if [[ "${vuln_count}" -gt 0 ]]; then
                    vulnerabilities_found=$((vulnerabilities_found + vuln_count))
                    log_error "Security vulnerabilities found: ${vuln_count}"
                    jq '.vulnerabilities.list[]? | {id: .id, package: .package.name, severity: .severity}' "${OUTPUT_DIR}/audit-results.json" 2>/dev/null || true
                fi

                if [[ "${warnings_count}" -gt 0 ]]; then
                    log_warning "Security warnings found: ${warnings_count}"
                fi
            else
                log_warning "cargo-audit failed - check audit-errors.log"
            fi
        else
            log_warning "cargo-audit not available - install with: cargo install cargo-audit"
        fi
    fi

    # Critical vulnerabilities only mode
    if [[ "${CRITICAL_ONLY}" == true ]]; then
        log_info "Running critical-only security scan..."

        # Use cargo-deny advisories for critical issues
        if cargo deny check advisories --format json > "${OUTPUT_DIR}/critical-advisories.json" 2>&1; then
            local critical_count=$(jq -r '.errors // [] | length' "${OUTPUT_DIR}/critical-advisories.json" 2>/dev/null || echo "0")
            if [[ "${critical_count}" -gt 0 ]]; then
                vulnerabilities_found=$((vulnerabilities_found + critical_count))
                log_error "Critical security advisories found: ${critical_count}"
            fi
        fi
    fi

    # Generate security audit report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg vulnerabilities_found "$vulnerabilities_found" \
        --arg schedule_type "$( [[ "${SCHEDULE_WEEKLY}" == true ]] && echo "weekly" || [[ "${SCHEDULE_DAILY}" == true ]] && echo "daily" || echo "manual" )" \
        --arg critical_only "$CRITICAL_ONLY" \
        '{
            timestamp: $timestamp,
            audit_type: "security",
            schedule_type: $schedule_type,
            critical_only: ($critical_only == "true"),
            vulnerabilities_found: ($vulnerabilities_found | tonumber),
            status: (if ($vulnerabilities_found == "0") then "SECURE" else "VULNERABILITIES_FOUND" end),
            risk_level: (if ($vulnerabilities_found > "10") then "HIGH" elif ($vulnerabilities_found > "0") then "MEDIUM" else "LOW" end)
        }' > "${security_report}"

    log_info "Security audit completed. Report: ${security_report}"
    return $vulnerabilities_found
}

# Function to run license compliance audit
run_license_audit() {
    if [[ "${SECURITY_ONLY}" == true ]]; then
        log_info "Skipping license audit (--security-only mode)"
        return 0
    fi

    log_info "Running license compliance audit..."

    local license_report="${OUTPUT_DIR}/license-audit-report.json"
    local license_issues=0

    cd "${PROJECT_ROOT}"

    # Enhanced cargo-deny license checking
    if command -v cargo-deny >/dev/null 2>&1; then
        # Full license audit
        if [[ "${SCHEDULE_WEEKLY}" == true ]] || [[ "${ALL_CRATES}" == true ]]; then
            log_info "Running comprehensive license audit on all crates..."

            # Get all crate names from workspace
            local crates_list=$(cargo metadata --format-version 1 | jq -r '.packages[].name' 2>/dev/null || echo "")

            for crate in ${crates_list}; do
                log_info "Auditing licenses for crate: ${crate}"

                if cargo deny check licenses --manifest-path "$(find . -name "Cargo.toml" -path "*/${crate}/*" -o -name "Cargo.toml" -exec grep -l "\"${crate}\"" {} \; | head -1)" \
                    --format json > "${OUTPUT_DIR}/license-${crate}.json" 2>&1; then
                    local crate_issues=$(jq -r '.errors // [] | length' "${OUTPUT_DIR}/license-${crate}.json" 2>/dev/null || echo "0")
                    license_issues=$((license_issues + crate_issues))
                fi
            done
        fi

        # Standard license check
        if cargo deny check licenses --format json > "${OUTPUT_DIR}/license-check.json" 2>&1; then
            local standard_issues=$(jq -r '.errors // [] | length' "${OUTPUT_DIR}/license-check.json" 2>/dev/null || echo "0")
            license_issues=$((license_issues + standard_issues))

            if [[ "${standard_issues}" -gt 0 ]]; then
                log_error "License compliance issues found: ${standard_issues}"
                jq '.errors[]? | .message' "${OUTPUT_DIR}/license-check.json" 2>/dev/null || true
            fi
        else
            license_issues=$((license_issues + 1))
            log_error "License compliance check failed"
        fi

        # Check for banned licenses specifically
        if cargo deny check bans --format json > "${OUTPUT_DIR}/bans-check.json" 2>&1; then
            local ban_violations=$(jq -r '.errors // [] | length' "${OUTPUT_DIR}/bans-check.json" 2>/dev/null || echo "0")
            if [[ "${ban_violations}" -gt 0 ]]; then
                license_issues=$((license_issues + ban_violations))
                log_error "License ban violations found: ${ban_violations}"
            fi
        fi
    else
        log_error "cargo-deny not available - license audit cannot proceed"
        return 1
    fi

    # Generate license audit report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg license_issues "$license_issues" \
        --arg all_crates_audited "$ALL_CRATES" \
        '{
            timestamp: $timestamp,
            audit_type: "license",
            all_crates_audited: ($all_crates_audited == "true"),
            license_issues_found: ($license_issues | tonumber),
            status: (if ($license_issues == "0") then "COMPLIANT" else "ISSUES_FOUND" end),
            compliance_level: (if ($license_issues == "0") then "FULL" elif ($license_issues < "5") then "GOOD" else "NEEDS_ATTENTION" end)
        }' > "${license_report}"

    log_info "License audit completed. Report: ${license_report}"
    return $license_issues
}

# Function to audit dependency freshness
run_freshness_audit() {
    log_info "Running dependency freshness audit..."

    local freshness_report="${OUTPUT_DIR}/freshness-audit-report.json"
    local outdated_count=0

    cd "${PROJECT_ROOT}"

    # Use cargo-outdated for freshness analysis
    if command -v cargo-outdated >/dev/null 2>&1; then
        log_info "Checking for outdated dependencies..."

        if cargo outdated --format json > "${OUTPUT_DIR}/outdated-deps.json" 2>&1; then
            outdated_count=$(jq -r '.dependencies // [] | length' "${OUTPUT_DIR}/outdated-deps.json" 2>/dev/null || echo "0")
            log_info "Found ${outdated_count} outdated dependencies"

            if [[ "${outdated_count}" -gt 0 ]]; then
                jq '.dependencies[]? | {name: .name, current: .current, latest: .latest, kind: .kind}' "${OUTPUT_DIR}/outdated-deps.json" 2>/dev/null || true
            fi
        else
            log_warning "cargo-outdated check failed"
        fi
    else
        log_warning "cargo-outdated not available - install with: cargo install cargo-outdated"
    fi

    # Generate freshness audit report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg outdated_count "$outdated_count" \
        '{
            timestamp: $timestamp,
            audit_type: "freshness",
            outdated_dependencies: ($outdated_count | tonumber),
            status: (if ($outdated_count > "50") then "OUTDATED" elif ($outdated_count > "20") then "NEEDS_UPDATE" else "CURRENT" end),
            recommendation: (if ($outdated_count > "20") then "Consider scheduled dependency updates" else "Dependencies are reasonably current" end)
        }' > "${freshness_report}"

    log_info "Freshness audit completed. Report: ${freshness_report}"
    return $outdated_count
}

# Function to generate comprehensive audit report
generate_comprehensive_report() {
    log_info "Generating comprehensive dependency audit report..."

    local comprehensive_report="${OUTPUT_DIR}/comprehensive-audit-report.json"
    local html_report="${OUTPUT_DIR}/audit-report.html"

    # Collect all audit results
    local security_issues=$(jq -r '.vulnerabilities_found // 0' "${OUTPUT_DIR}/security-audit-report.json" 2>/dev/null || echo "0")
    local license_issues=$(jq -r '.license_issues_found // 0' "${OUTPUT_DIR}/license-audit-report.json" 2>/dev/null || echo "0")
    local outdated_deps=$(jq -r '.outdated_dependencies // 0' "${OUTPUT_DIR}/freshness-audit-report.json" 2>/dev/null || echo "0")

    local overall_risk="LOW"
    if [[ "${security_issues}" -gt 0 ]] || [[ "${license_issues}" -gt 5 ]] || [[ "${outdated_deps}" -gt 50 ]]; then
        overall_risk="HIGH"
    elif [[ "${security_issues}" -gt 0 ]] || [[ "${license_issues}" -gt 0 ]] || [[ "${outdated_deps}" -gt 20 ]]; then
        overall_risk="MEDIUM"
    fi

    # Generate comprehensive JSON report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg duration "$(($(date +%s) - START_TIME))" \
        --arg security_issues "$security_issues" \
        --arg license_issues "$license_issues" \
        --arg outdated_deps "$outdated_deps" \
        --arg overall_risk "$overall_risk" \
        --arg schedule_type "$( [[ "${SCHEDULE_WEEKLY}" == true ]] && echo "weekly" || [[ "${SCHEDULE_DAILY}" == true ]] && echo "daily" || echo "manual" )" \
        --arg all_crates "$ALL_CRATES" \
        --arg critical_only "$CRITICAL_ONLY" \
        --arg security_only "$SECURITY_ONLY" \
        '{
            timestamp: $timestamp,
            duration_seconds: ($duration | tonumber),
            audit_type: "comprehensive",
            schedule_type: $schedule_type,
            configuration: {
                all_crates_audited: ($all_crates == "true"),
                critical_only: ($critical_only == "true"),
                security_only: ($security_only == "true")
            },
            results: {
                security_vulnerabilities: ($security_issues | tonumber),
                license_issues: ($license_issues | tonumber),
                outdated_dependencies: ($outdated_deps | tonumber)
            },
            overall_risk: $overall_risk,
            status: (if ($overall_risk == "HIGH") then "ACTION_REQUIRED" elif ($overall_risk == "MEDIUM") then "REVIEW_NEEDED" else "HEALTHY" end),
            recommendations: [
                (if ($security_issues > "0") then "Address security vulnerabilities immediately" else "Security posture is good" end),
                (if ($license_issues > "0") then "Review and resolve license compliance issues" else "License compliance is maintained" end),
                (if ($outdated_deps > "20") then "Consider updating outdated dependencies" else "Dependencies are reasonably current" end)
            ]
        }' > "${comprehensive_report}"

    # Generate HTML report
    cat > "${html_report}" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>Rust AI IDE Dependency Audit Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f0f0f0; padding: 10px; border-radius: 5px; }
        .status-healthy { color: green; }
        .status-review { color: orange; }
        .status-action { color: red; }
        .section { margin: 20px 0; padding: 10px; border: 1px solid #ccc; border-radius: 5px; }
        .metric { display: inline-block; margin: 10px; padding: 10px; background: #f9f9f9; border-radius: 3px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Rust AI IDE Dependency Audit Report</h1>
        <p><strong>Generated:</strong> $(date)</p>
        <p><strong>Duration:</strong> $(($(date +%s) - START_TIME)) seconds</p>
        <p><strong>Overall Risk:</strong> <span class="status-${overall_risk,,}">${overall_risk}</span></p>
        <p><strong>Schedule:</strong> $( [[ "${SCHEDULE_WEEKLY}" == true ]] && echo "Weekly" || [[ "${SCHEDULE_DAILY}" == true ]] && echo "Daily" || echo "Manual" )</p>
    </div>

    <div class="section">
        <h2>Audit Configuration</h2>
        <p><strong>All Crates Audited:</strong> ${ALL_CRATES}</p>
        <p><strong>Critical Only:</strong> ${CRITICAL_ONLY}</p>
        <p><strong>Security Only:</strong> ${SECURITY_ONLY}</p>
    </div>

    <div class="section">
        <h2>Audit Results</h2>
        <div class="metric">
            <strong>Security Vulnerabilities:</strong> ${security_issues}
        </div>
        <div class="metric">
            <strong>License Issues:</strong> ${license_issues}
        </div>
        <div class="metric">
            <strong>Outdated Dependencies:</strong> ${outdated_deps}
        </div>
    </div>

    <div class="section">
        <h2>Recommendations</h2>
        <ul>
            $(if [[ "${security_issues}" -gt 0 ]]; then echo "<li>🔴 Address security vulnerabilities immediately</li>"; else echo "<li>✅ Security posture is good</li>"; fi)
            $(if [[ "${license_issues}" -gt 0 ]]; then echo "<li>🔴 Review and resolve license compliance issues</li>"; else echo "<li>✅ License compliance is maintained</li>"; fi)
            $(if [[ "${outdated_deps}" -gt 20 ]]; then echo "<li>🟡 Consider updating outdated dependencies</li>"; else echo "<li>✅ Dependencies are reasonably current</li>"; fi)
        </ul>
    </div>

    <div class="section">
        <h3>Report Files</h3>
        <ul>
            <li><a href="comprehensive-audit-report.json">Comprehensive JSON Report</a></li>
            <li><a href="security-audit-report.json">Security Audit Report</a></li>
            <li><a href="license-audit-report.json">License Audit Report</a></li>
            <li><a href="freshness-audit-report.json">Freshness Audit Report</a></li>
        </ul>
    </div>
</body>
</html>
EOF

    log_success "Comprehensive audit report generated: ${comprehensive_report}"
    log_success "HTML audit report generated: ${html_report}"
}

# Main function
main() {
    log_info "Starting dependency audit for Rust AI IDE"
    log_info "Log file: ${AUDIT_LOG}"
    log_info "Report directory: ${OUTPUT_DIR}"

    mkdir -p "${OUTPUT_DIR}"

    local exit_code=0

    # Run audits
    run_security_audit || exit_code=$((exit_code + 1))
    run_license_audit || exit_code=$((exit_code + 1))
    run_freshness_audit || exit_code=$((exit_code + 1))

    # Generate comprehensive report
    generate_comprehensive_report

    local end_time=$(date +%s)
    log_info "Dependency audit completed in $((end_time - START_TIME)) seconds"

    return $exit_code
}

# Run main function
main "$@"
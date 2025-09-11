#!/bin/bash

# Snyk Dependency Security Scanning Script
# Automated vulnerability scanning for npm dependencies in web frontend
# Author: Security Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
WEB_DIR="${PROJECT_ROOT}/web"
SECURITY_LOG="${PROJECT_ROOT}/snyk-security.log"
REPORT_DIR="${PROJECT_ROOT}/security-reports/snyk"
START_TIME=$(date +%s)

# Default configuration
SNYK_TOKEN="${SNYK_TOKEN:-}"
SNYK_ORG="${SNYK_ORG:-}"
SEVERITY_THRESHOLD="${SEVERITY_THRESHOLD:-medium}"
FAIL_ON_ISSUES="${FAIL_ON_ISSUES:-true}"

# Create report directory
mkdir -p "${REPORT_DIR}"

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

Snyk dependency security scanning for Rust AI IDE web frontend.

OPTIONS:
    -h, --help              Show this help message
    -v, --verbose           Enable verbose output
    --severity-threshold    Minimum severity to report (low|medium|high) (default: medium)
    --fail-on-issues        Fail pipeline on issues found (default: true)
    --report-dir DIR        Output directory for reports (default: security-reports/snyk)
    --scheduled             Run in scheduled mode (non-interactive)
    --monitor               Enable Snyk monitoring and reporting
    --fix                   Attempt automatic fixes for vulnerabilities

EXAMPLES:
    $0 --severity-threshold high
    $0 --scheduled --monitor
    $0 --fix --verbose

ENVIRONMENT VARIABLES:
    SNYK_TOKEN              Snyk API token (required)
    SNYK_ORG                Snyk organization ID (optional)

EOF
}

# Parse command line arguments
VERBOSE=false
SCHEDULED=false
MONITOR=false
AUTO_FIX=false

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
        --severity-threshold)
            SEVERITY_THRESHOLD="$2"
            shift 2
            ;;
        --fail-on-issues)
            FAIL_ON_ISSUES="$2"
            shift 2
            ;;
        --report-dir)
            REPORT_DIR="$2"
            shift 2
            ;;
        --scheduled)
            SCHEDULED=true
            shift
            ;;
        --monitor)
            MONITOR=true
            shift
            ;;
        --fix)
            AUTO_FIX=true
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Function to check Snyk requirements
check_snyk_requirements() {
    log_info "Checking Snyk requirements..."

    # Check if Node.js and npm are available
    if ! command -v node >/dev/null 2>&1; then
        log_error "Node.js not found"
        return 1
    fi

    if ! command -v npm >/dev/null 2>&1; then
        log_error "npm not found"
        return 1
    fi

    # Check if web directory exists
    if [[ ! -f "${WEB_DIR}/package.json" ]]; then
        log_error "Web directory not found or invalid: ${WEB_DIR}"
        return 1
    fi

    # Check Snyk installation
    if ! command -v snyk >/dev/null 2>&1; then
        log_info "Snyk CLI not found, installing..."
        npm install -g snyk
    fi

    # Check Snyk authentication
    if [[ -z "${SNYK_TOKEN}" ]]; then
        log_error "SNYK_TOKEN environment variable not set"
        log_info "Please set SNYK_TOKEN or run: snyk auth"
        log_info "To get a token, visit: https://app.snyk.io/account"
        return 1
    fi

    # Authenticate with Snyk
    log_info "Authenticating with Snyk..."
    if ! snyk auth "${SNYK_TOKEN}"; then
        log_error "Snyk authentication failed"
        return 1
    fi

    log_success "Snyk requirements check passed"
}

# Function to run Snyk dependency scan
run_snyk_scan() {
    log_info "Running Snyk dependency vulnerability scan..."

    cd "${WEB_DIR}"

    local json_report="${REPORT_DIR}/snyk-scan-report.json"
    local html_report="${REPORT_DIR}/snyk-scan-report.html"

    # Run Snyk test
    log_info "Scanning dependencies for vulnerabilities..."

    local snyk_command="snyk test --json --severity-threshold=${SEVERITY_THRESHOLD}"

    if [[ "${VERBOSE}" == true ]]; then
        snyk_command="${snyk_command} --print-deps"
    fi

    if [[ "${MONITOR}" == true ]]; then
        snyk_command="${snyk_command} --org=${SNYK_ORG:-}"
    fi

    if eval "${snyk_command}" > "${REPORT_DIR}/snyk-raw-output.json" 2>&1; then
        log_success "Snyk scan completed without critical issues"
        local scan_status="PASSED"
    else
        local exit_code=$?
        if [[ $exit_code -eq 1 ]]; then
            log_warning "Snyk scan found vulnerabilities"
            local scan_status="ISSUES_FOUND"
        elif [[ $exit_code -eq 2 ]]; then
            log_error "Snyk scan failed"
            local scan_status="FAILED"
            return 1
        else
            log_warning "Snyk scan completed with warnings"
            local scan_status="WARNING"
        fi
    fi

    # Process the JSON output
    if [[ -f "${REPORT_DIR}/snyk-raw-output.json" ]]; then
        # Generate HTML report
        snyk test --html > "${html_report}" 2>/dev/null || log_warning "Failed to generate HTML report"

        # Analyze results
        analyze_snyk_results "${REPORT_DIR}/snyk-raw-output.json" "${scan_status}"
    else
        log_error "Snyk scan output not found"
        return 1
    fi
}

# Function to analyze Snyk results
analyze_snyk_results() {
    local json_file="$1"
    local scan_status="$2"

    log_info "Analyzing Snyk scan results..."

    # Parse JSON results
    local high_count=$(jq -r '.vulnerabilities // [] | map(select(.severity == "high")) | length' "${json_file}" 2>/dev/null || echo "0")
    local medium_count=$(jq -r '.vulnerabilities // [] | map(select(.severity == "medium")) | length' "${json_file}" 2>/dev/null || echo "0")
    local low_count=$(jq -r '.vulnerabilities // [] | map(select(.severity == "low")) | length' "${json_file}" 2>/dev/null || echo "0")
    local total_count=$((high_count + medium_count + low_count))

    local unique_packages=$(jq -r '.vulnerabilities // [] | map(.packageName) | unique | length' "${json_file}" 2>/dev/null || echo "0")

    log_info "Snyk Scan Results:"
    log_info "  Total vulnerabilities: ${total_count}"
    log_info "  High severity: ${high_count}"
    log_info "  Medium severity: ${medium_count}"
    log_info "  Low severity: ${low_count}"
    log_info "  Affected packages: ${unique_packages}"

    # Generate summary report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg scan_status "$scan_status" \
        --arg severity_threshold "$SEVERITY_THRESHOLD" \
        --arg total_vulnerabilities "$total_count" \
        --arg high_count "$high_count" \
        --arg medium_count "$medium_count" \
        --arg low_count "$low_count" \
        --arg affected_packages "$unique_packages" \
        '{
            timestamp: $timestamp,
            tool: "Snyk",
            scan_type: "dependency",
            severity_threshold: $severity_threshold,
            vulnerabilities: {
                total: ($total_vulnerabilities | tonumber),
                high: ($high_count | tonumber),
                medium: ($medium_count | tonumber),
                low: ($low_count | tonumber),
                affected_packages: ($affected_packages | tonumber)
            },
            status: $scan_status
        }' > "${REPORT_DIR}/snyk-summary-report.json"

    # Check if we should fail based on severity threshold
    local should_fail=false
    case "${SEVERITY_THRESHOLD}" in
        "high")
            [[ "$high_count" -gt 0 ]] && should_fail=true
            ;;
        "medium")
            [[ "$high_count" -gt 0 || "$medium_count" -gt 0 ]] && should_fail=true
            ;;
        "low")
            [[ "$total_count" -gt 0 ]] && should_fail=true
            ;;
    esac

    if [[ "$should_fail" == true && "${FAIL_ON_ISSUES}" == "true" ]]; then
        log_error "Vulnerabilities found exceeding severity threshold (${SEVERITY_THRESHOLD})"
        return 1
    elif [[ "$total_count" -gt 0 ]]; then
        log_warning "${total_count} vulnerabilities found (below severity threshold)"
        return 0
    else
        log_success "No vulnerabilities found"
        return 0
    fi
}

# Function to run Snyk monitor
run_snyk_monitor() {
    if [[ "${MONITOR}" != true ]]; then
        return 0
    fi

    log_info "Setting up Snyk monitoring..."

    cd "${WEB_DIR}"

    # Run Snyk monitor
    if snyk monitor --org="${SNYK_ORG:-}"; then
        log_success "Snyk monitoring enabled"

        # Generate monitor report
        jq -n \
            --arg timestamp "$(date -Iseconds)" \
            --arg org_id "${SNYK_ORG:-}" \
            '{
                timestamp: $timestamp,
                monitoring_enabled: true,
                organization: $org_id,
                status: "ACTIVE"
            }' > "${REPORT_DIR}/snyk-monitor-report.json"
    else
        log_warning "Failed to enable Snyk monitoring"
    fi
}

# Function to attempt automatic fixes
run_snyk_fix() {
    if [[ "${AUTO_FIX}" != true ]]; then
        return 0
    fi

    log_info "Attempting automatic vulnerability fixes..."

    cd "${WEB_DIR}"

    # Create backup of package-lock.json
    if [[ -f "package-lock.json" ]]; then
        cp package-lock.json "${REPORT_DIR}/package-lock-backup.json"
        log_info "Created backup of package-lock.json"
    fi

    # Run Snyk wizard for fixes
    log_info "Running Snyk wizard for automatic fixes..."
    if echo "y" | snyk wizard > "${REPORT_DIR}/snyk-wizard.log" 2>&1; then
        log_success "Snyk wizard completed"
    else
        log_warning "Snyk wizard failed or found no auto-fixable issues"
    fi

    # Run npm audit fix as fallback
    log_info "Running npm audit fix..."
    if npm audit fix > "${REPORT_DIR}/npm-audit-fix.log" 2>&1; then
        log_success "npm audit fix completed"
    else
        log_warning "npm audit fix failed"
    fi
}

# Function to setup scheduled scanning
setup_scheduled_scan() {
    log_info "Setting up scheduled Snyk scanning..."

    local cron_schedule="0 3 * * *"  # Daily at 3 AM
    local cron_command="${SCRIPT_DIR}/snyk-dependency-scan.sh --scheduled --severity-threshold=${SEVERITY_THRESHOLD}"

    # Add to crontab
    if ! crontab -l 2>/dev/null | grep -q "snyk-dependency-scan.sh"; then
        (crontab -l 2>/dev/null; echo "${cron_schedule} ${cron_command}") | crontab -
        log_success "Scheduled Snyk scan added to crontab"
    else
        log_info "Scheduled Snyk scan already exists in crontab"
    fi

    # Generate schedule report
    jq -n \
        --arg schedule "$cron_schedule" \
        --arg command "$cron_command" \
        --arg setup_time "$(date -Iseconds)" \
        '{
            setup_time: $setup_time,
            schedule: $schedule,
            command: $command,
            status: "ACTIVE"
        }' > "${REPORT_DIR}/snyk-schedule.json"
}

# Function to generate comprehensive report
generate_comprehensive_report() {
    log_info "Generating comprehensive Snyk report..."

    local comprehensive_report="${REPORT_DIR}/comprehensive-snyk-report.json"

    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg duration "$(($(date +%s) - START_TIME))" \
        --arg summary "$(cat "${REPORT_DIR}/snyk-summary-report.json" 2>/dev/null || echo '{}')" \
        --arg monitor "$(cat "${REPORT_DIR}/snyk-monitor-report.json" 2>/dev/null || echo '{}')" \
        --arg schedule "$(cat "${REPORT_DIR}/snyk-schedule.json" 2>/dev/null || echo '{}')" \
        '{
            timestamp: $timestamp,
            duration_seconds: ($duration | tonumber),
            tool: "Snyk",
            scan_completed: true,
            reports: {
                summary: ($summary | fromjson),
                monitor: ($monitor | fromjson),
                schedule: ($schedule | fromjson)
            }
        }' > "${comprehensive_report}"

    log_info "Comprehensive Snyk report generated: ${comprehensive_report}"
}

# Main function
main() {
    log_info "Starting Snyk dependency security scan"
    log_info "Log file: ${SECURITY_LOG}"
    log_info "Report directory: ${REPORT_DIR}"
    log_info "Web directory: ${WEB_DIR}"
    log_info "Severity threshold: ${SEVERITY_THRESHOLD}"

    mkdir -p "${REPORT_DIR}"

    # Check requirements
    check_snyk_requirements || exit 1

    local exit_code=0

    # Run scans
    run_snyk_scan || exit_code=$((exit_code + 1))
    run_snyk_monitor
    run_snyk_fix

    # Setup scheduled scanning if requested
    if [[ "${SCHEDULED}" == false ]]; then
        setup_scheduled_scan
    fi

    # Generate comprehensive report
    generate_comprehensive_report

    local end_time=$(date +%s)
    log_info "Snyk scan completed in $((end_time - START_TIME)) seconds"

    return $exit_code
}

# Run main function
main "$@"
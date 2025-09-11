#!/bin/bash
# Code Review Workflow Integration Script
# Automates code review processes with checks, approvals, and oversight

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
LOG_FILE="${PROJECT_ROOT}/logs/review-workflow-$(date +%Y%m%d-%H%M%S).log"
REVIEW_BRANCH="${REVIEW_BRANCH:-main}"
CHANGE_BRANCH="${CHANGE_BRANCH:-}"
REVIEWER_EMAIL="${REVIEWER_EMAIL:-}"
APPROVAL_WEBHOOK="${APPROVAL_WEBHOOK:-}"
NOTIFICATION_WEBHOOK="${NOTIFICATION_WEBHOOK:-}"

# Logging function
log() {
    local level="$1"
    local message="$2"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[${timestamp}] [${level}] ${message}" | tee -a "${LOG_FILE}"
}

# Notification functions
notify_webhook() {
    local payload="$1"
    local webhook="${NOTIFICATION_WEBHOOK}"
    if [[ -n "${webhook}" ]]; then
        curl -s -X POST "${webhook}" \
            -H 'Content-type: application/json' \
            --data "${payload}" || log "WARN" "Failed to send webhook notification"
    fi
}

notify_approval() {
    local payload="$1"
    local webhook="${APPROVAL_WEBHOOK}"
    if [[ -n "${webhook}" ]]; then
        curl -s -X POST "${webhook}" \
            -H 'Content-type: application/json' \
            --data "${payload}" || log "WARN" "Failed to send approval notification"
    fi
}

# Error handling
error_exit() {
    local message="$1"
    log "ERROR" "${message}"
    notify_webhook "{\"status\":\"failed\",\"stage\":\"review\",\"message\":\"${message}\",\"timestamp\":\"$(date -Iseconds)\"}"
    exit 1
}

# Ensure we're in the project root
cd "${PROJECT_ROOT}" || error_exit "Cannot change to project root directory"

# Create logs directory if it doesn't exist
mkdir -p "$(dirname "${LOG_FILE}")"

# Function to get changed files
get_changed_files() {
    local base_branch="$1"
    local head_branch="${2:-HEAD}"

    if git diff --name-only "${base_branch}".."${head_branch}"; then
        return 0
    else
        log "WARN" "Could not get changed files, using all files"
        find . -name "*.rs" -o -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.toml" | head -50
    fi
}

# Function to run automated checks
run_automated_checks() {
    log "INFO" "Running automated code review checks"

    local check_results=""
    local failed_checks=""

    # Rust linting
    log "INFO" "Running Rust linting checks"
    if cargo +nightly clippy --workspace -- -D warnings; then
        check_results="${check_results}rust_linting:passed "
        log "INFO" "Rust linting passed"
    else
        failed_checks="${failed_checks}rust_linting "
        log "ERROR" "Rust linting failed"
    fi

    # Rust formatting check
    log "INFO" "Checking Rust code formatting"
    if cargo +nightly fmt --check; then
        check_results="${check_results}rust_formatting:passed "
        log "INFO" "Rust formatting check passed"
    else
        failed_checks="${failed_checks}rust_formatting "
        log "ERROR" "Rust formatting check failed"
    fi

    # TypeScript type checking
    log "INFO" "Running TypeScript type checking"
    cd web || return 1
    if npm run type-check; then
        check_results="${check_results}typescript_types:passed "
        log "INFO" "TypeScript type checking passed"
    else
        failed_checks="${failed_checks}typescript_types "
        log "ERROR" "TypeScript type checking failed"
    fi
    cd "${PROJECT_ROOT}" || return 1

    # Security scanning
    log "INFO" "Running security checks"
    if [[ -f "scripts/ci/security-checks.sh" ]]; then
        if bash scripts/ci/security-checks.sh; then
            check_results="${check_results}security_checks:passed "
            log "INFO" "Security checks passed"
        else
            failed_checks="${failed_checks}security_checks "
            log "ERROR" "Security checks failed"
        fi
    fi

    # License compliance
    log "INFO" "Running license compliance checks"
    if cargo deny check; then
        check_results="${check_results}license_compliance:passed "
        log "INFO" "License compliance checks passed"
    else
        failed_checks="${failed_checks}license_compliance "
        log "ERROR" "License compliance checks failed"
    fi

    # Output results
    echo "AUTOMATED_CHECKS_RESULTS=${check_results}"
    echo "FAILED_CHECKS=${failed_checks}"

    if [[ -n "${failed_checks}" ]]; then
        return 1
    fi
}

# Function to analyze code changes
analyze_code_changes() {
    local changed_files="$1"

    log "INFO" "Analyzing code changes for review"

    local rust_files=$(echo "${changed_files}" | grep '\.rs$' | wc -l)
    local ts_files=$(echo "${changed_files}" | grep '\.ts\|\.tsx\|\.js$' | wc -l)
    local config_files=$(echo "${changed_files}" | grep '\.toml\|\.json\|\.yml\|\.yaml$' | wc -l)

    log "INFO" "Change analysis: ${rust_files} Rust files, ${ts_files} TypeScript/JavaScript files, ${config_files} config files"

    # Check for sensitive data in changes
    if echo "${changed_files}" | grep -q -E "(secret|password|token|key)"; then
        log "WARN" "Potential sensitive data files detected in changes"
        echo "SENSITIVE_DATA_DETECTED=true"
    else
        echo "SENSITIVE_DATA_DETECTED=false"
    fi

    # Check for breaking changes
    if echo "${changed_files}" | grep -q "Cargo.toml\|package.json"; then
        log "INFO" "Dependency changes detected - potential breaking changes"
        echo "BREAKING_CHANGES_DETECTED=true"
    else
        echo "BREAKING_CHANGES_DETECTED=false"
    fi
}

# Function to run manual oversight checks
run_manual_oversight() {
    log "INFO" "Running manual oversight checks"

    # Check for large changes
    local total_lines=$(git diff --stat "${REVIEW_BRANCH}" | tail -1 | awk '{print $4+$6}')
    if [[ ${total_lines:-0} -gt 1000 ]]; then
        log "WARN" "Large change detected (${total_lines} lines) - manual review recommended"
        echo "LARGE_CHANGE_DETECTED=true"
        echo "MANUAL_REVIEW_RECOMMENDED=true"
    else
        echo "LARGE_CHANGE_DETECTED=false"
        echo "MANUAL_REVIEW_RECOMMENDED=false"
    fi

    # Check for critical file changes
    if git diff --name-only "${REVIEW_BRANCH}" | grep -q -E "(security|auth|crypto)"; then
        log "WARN" "Critical security files changed - manual security review required"
        echo "SECURITY_REVIEW_REQUIRED=true"
    else
        echo "SECURITY_REVIEW_REQUIRED=false"
    fi

    # Check for API changes
    if git diff --name-only "${REVIEW_BRANCH}" | grep -q "src-tauri/src/"; then
        log "INFO" "Tauri API changes detected - API review recommended"
        echo "API_REVIEW_RECOMMENDED=true"
    else
        echo "API_REVIEW_RECOMMENDED=false"
    fi
}

# Function to generate review report
generate_review_report() {
    local automated_results="$1"
    local oversight_results="$2"
    local change_analysis="$3"

    log "INFO" "Generating comprehensive review report"

    local report_file="${PROJECT_ROOT}/review-report-$(date +%Y%m%d-%H%M%S).md"

    cat > "${report_file}" << EOF
# Code Review Report
Generated: $(date)

## Review Details
- Base Branch: ${REVIEW_BRANCH}
- Change Branch: ${CHANGE_BRANCH:-HEAD}
- Reviewer: ${REVIEWER_EMAIL:-Automated}

## Automated Checks Results
${automated_results}

## Manual Oversight Results
${oversight_results}

## Code Change Analysis
${change_analysis}

## Recommendations
EOF

    # Add recommendations based on results
    if echo "${oversight_results}" | grep -q "MANUAL_REVIEW_RECOMMENDED=true"; then
        echo "- **Manual Review Required**: Large change detected" >> "${report_file}"
    fi

    if echo "${oversight_results}" | grep -q "SECURITY_REVIEW_REQUIRED=true"; then
        echo "- **Security Review Required**: Critical security files modified" >> "${report_file}"
    fi

    if echo "${oversight_results}" | grep -q "API_REVIEW_RECOMMENDED=true"; then
        echo "- **API Review Recommended**: Tauri API changes detected" >> "${report_file}"
    fi

    echo "" >> "${report_file}"
    echo "## Next Steps" >> "${report_file}"
    echo "1. Review automated check failures" >> "${report_file}"
    echo "2. Address manual oversight recommendations" >> "${report_file}"
    echo "3. Perform security review if required" >> "${report_file}"
    echo "4. Test changes in staging environment" >> "${report_file}"

    log "INFO" "Review report generated: ${report_file}"
    echo "REVIEW_REPORT_PATH=${report_file}"
}

# Function to request approval
request_approval() {
    local review_report="$1"

    log "INFO" "Requesting approval for code changes"

    local payload="{
        \"action\": \"approval_requested\",
        \"review_branch\": \"${REVIEW_BRANCH}\",
        \"change_branch\": \"${CHANGE_BRANCH}\",
        \"reviewer_email\": \"${REVIEWER_EMAIL}\",
        \"report_path\": \"${review_report}\",
        \"timestamp\": \"$(date -Iseconds)\",
        \"automated_checks_status\": \"completed\"
    }"

    notify_approval "${payload}"
    log "INFO" "Approval request sent"
}

# Main execution
main() {
    local start_time=$(date +%s)

    log "INFO" "Starting code review workflow"
    notify_webhook "{\"status\":\"started\",\"stage\":\"review\",\"timestamp\":\"$(date -Iseconds)\"}"

    # Get changed files
    local changed_files
    changed_files=$(get_changed_files "${REVIEW_BRANCH}" "${CHANGE_BRANCH}")

    # Run automated checks
    local automated_results=""
    if run_automated_checks; then
        automated_results="✅ All automated checks passed"
    else
        automated_results="❌ Some automated checks failed"
    fi

    # Analyze changes
    local change_analysis
    change_analysis=$(analyze_code_changes "${changed_files}")

    # Run manual oversight
    local oversight_results
    oversight_results=$(run_manual_oversight)

    # Generate report
    local review_report
    review_report=$(generate_review_report "${automated_results}" "${oversight_results}" "${change_analysis}")

    # Request approval if reviewer specified
    if [[ -n "${REVIEWER_EMAIL}" ]]; then
        request_approval "${review_report}"
    fi

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    # Success notification
    local success_message="✅ Code review workflow completed in ${duration}s"
    log "INFO" "${success_message}"

    local payload="{
        \"status\": \"completed\",
        \"stage\": \"review\",
        \"duration_seconds\": ${duration},
        \"review_report\": \"${review_report}\",
        \"timestamp\": \"$(date -Iseconds)\"
    }"
    notify_webhook "${payload}"

    # Output for CI/CD integration
    echo "REVIEW_WORKFLOW_STATUS=completed"
    echo "REVIEW_DURATION=${duration}"
    echo "REVIEW_REPORT=${review_report}"
}

# Trap for cleanup
trap 'error_exit "Review workflow interrupted"' INT TERM

# Run main function
main "$@"
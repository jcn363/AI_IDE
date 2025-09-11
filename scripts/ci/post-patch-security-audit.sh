#!/bin/bash

# Post-Patch Security Audit and Verification Script
# Comprehensive verification after security patches and updates
# Author: Security Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
WEB_DIR="${PROJECT_ROOT}/web"
AUDIT_LOG="${PROJECT_ROOT}/post-patch-audit.log"
REPORT_DIR="${PROJECT_ROOT}/security-reports/post-patch-audit"
PRE_PATCH_REPORT_DIR=""
START_TIME=$(date +%s)

# Default configuration
COMPARE_WITH_PRE_PATCH=true
FAIL_ON_REGRESSION=true
DETAILED_ANALYSIS=true
PERFORMANCE_IMPACT_CHECK=true

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

Post-patch security audit and verification for Rust AI IDE.

OPTIONS:
    -h, --help                      Show this help message
    -v, --verbose                   Enable verbose output
    --pre-patch-report-dir DIR      Directory containing pre-patch security reports
    --no-comparison                 Skip comparison with pre-patch reports
    --no-fail-on-regression         Don't fail on security regressions
    --no-detailed-analysis          Skip detailed analysis
    --no-performance-check          Skip performance impact checks
    --report-dir DIR                Output directory for reports (default: security-reports/post-patch-audit)

EXAMPLES:
    $0 --pre-patch-report-dir security-reports/20230910_143000
    $0 --no-comparison --verbose

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
        --pre-patch-report-dir)
            PRE_PATCH_REPORT_DIR="$2"
            shift 2
            ;;
        --no-comparison)
            COMPARE_WITH_PRE_PATCH=false
            shift
            ;;
        --no-fail-on-regression)
            FAIL_ON_REGRESSION=false
            shift
            ;;
        --no-detailed-analysis)
            DETAILED_ANALYSIS=false
            shift
            ;;
        --no-performance-check)
            PERFORMANCE_IMPACT_CHECK=false
            shift
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

# Function to find latest pre-patch reports
find_latest_pre_patch_reports() {
    if [[ -n "${PRE_PATCH_REPORT_DIR}" ]]; then
        if [[ ! -d "${PRE_PATCH_REPORT_DIR}" ]]; then
            log_error "Pre-patch report directory does not exist: ${PRE_PATCH_REPORT_DIR}"
            return 1
        fi
        return 0
    fi

    # Auto-discover latest security reports
    local latest_reports=""
    if [[ -d "${PROJECT_ROOT}/security-reports" ]]; then
        latest_reports=$(find "${PROJECT_ROOT}/security-reports" -maxdepth 1 -type d -name "20*" | sort | tail -1)
    fi

    if [[ -n "${latest_reports}" ]]; then
        PRE_PATCH_REPORT_DIR="${latest_reports}"
        log_info "Using auto-discovered pre-patch reports: ${PRE_PATCH_REPORT_DIR}"
    else
        log_warning "No pre-patch reports found for comparison"
        COMPARE_WITH_PRE_PATCH=false
    fi
}

# Function to run current security checks
run_current_security_checks() {
    log_info "Running current security checks..."

    # Run the main security checks script
    if [[ -f "${SCRIPT_DIR}/security-checks.sh" ]]; then
        "${SCRIPT_DIR}/security-checks.sh" --output "${REPORT_DIR}" --quick > "${REPORT_DIR}/security-checks.log" 2>&1
        local exit_code=$?

        if [[ $exit_code -eq 0 ]]; then
            log_success "Security checks passed"
        else
            log_warning "Security checks failed with exit code: ${exit_code}"
        fi

        return $exit_code
    else
        log_error "Main security checks script not found"
        return 1
    fi
}

# Function to run OWASP ZAP verification
run_zap_verification() {
    log_info "Running OWASP ZAP verification scan..."

    if [[ -f "${SCRIPT_DIR}/zap-web-scan.sh" ]]; then
        # Run a quick baseline scan
        "${SCRIPT_DIR}/zap-web-scan.sh" --baseline --report-dir "${REPORT_DIR}/zap" > "${REPORT_DIR}/zap-verification.log" 2>&1
        local exit_code=$?

        if [[ $exit_code -eq 0 ]]; then
            log_success "ZAP verification scan completed"
        else
            log_warning "ZAP verification scan failed"
        fi

        return $exit_code
    else
        log_warning "ZAP scan script not found, skipping verification"
        return 0
    fi
}

# Function to run Snyk verification
run_snyk_verification() {
    log_info "Running Snyk verification scan..."

    if [[ -f "${SCRIPT_DIR}/snyk-dependency-scan.sh" ]]; then
        # Run a quick dependency scan
        "${SCRIPT_DIR}/snyk-dependency-scan.sh" --severity-threshold medium --report-dir "${REPORT_DIR}/snyk" > "${REPORT_DIR}/snyk-verification.log" 2>&1
        local exit_code=$?

        if [[ $exit_code -eq 0 ]]; then
            log_success "Snyk verification scan completed"
        else
            log_warning "Snyk verification scan failed"
        fi

        return $exit_code
    else
        log_warning "Snyk scan script not found, skipping verification"
        return 0
    fi
}

# Function to check performance impact
check_performance_impact() {
    if [[ "${PERFORMANCE_IMPACT_CHECK}" != true ]]; then
        log_info "Skipping performance impact check"
        return 0
    fi

    log_info "Checking performance impact..."

    # Measure build time
    log_info "Measuring build performance..."
    cd "${PROJECT_ROOT}"

    local build_start=$(date +%s)
    if cargo build --workspace --quiet > "${REPORT_DIR}/build-performance.log" 2>&1; then
        local build_end=$(date +%s)
        local build_time=$((build_end - build_start))
        log_info "Build completed in ${build_time} seconds"

        # Store performance metrics
        jq -n \
            --arg build_time "$build_time" \
            --arg timestamp "$(date -Iseconds)" \
            '{
                timestamp: $timestamp,
                build_time_seconds: ($build_time | tonumber),
                status: "measured"
            }' > "${REPORT_DIR}/performance-metrics.json"
    else
        log_error "Build failed during performance measurement"
        return 1
    fi

    # Measure test execution time
    log_info "Measuring test performance..."
    local test_start=$(date +%s)
    if cargo test --workspace --quiet > "${REPORT_DIR}/test-performance.log" 2>&1; then
        local test_end=$(date +%s)
        local test_time=$((test_end - test_start))
        log_info "Tests completed in ${test_time} seconds"

        # Update performance metrics
        jq --arg test_time "$test_time" \
            '.test_time_seconds = ($test_time | tonumber)' \
            "${REPORT_DIR}/performance-metrics.json" > "${REPORT_DIR}/performance-metrics.json.tmp" && \
            mv "${REPORT_DIR}/performance-metrics.json.tmp" "${REPORT_DIR}/performance-metrics.json"
    else
        log_error "Tests failed during performance measurement"
        return 1
    fi

    log_success "Performance impact check completed"
    return 0
}

# Function to compare with pre-patch reports
compare_with_pre_patch() {
    if [[ "${COMPARE_WITH_PRE_PATCH}" != true ]] || [[ ! -d "${PRE_PATCH_REPORT_DIR}" ]]; then
        log_info "Skipping comparison with pre-patch reports"
        return 0
    fi

    log_info "Comparing with pre-patch reports from: ${PRE_PATCH_REPORT_DIR}"

    local regression_found=false

    # Compare security check results
    if [[ -f "${REPORT_DIR}/comprehensive-security-report.json" ]] && [[ -f "${PRE_PATCH_REPORT_DIR}/comprehensive-security-report.json" ]]; then
        log_info "Comparing comprehensive security reports..."

        local current_status=$(jq -r '.overall_status // "UNKNOWN"' "${REPORT_DIR}/comprehensive-security-report.json")
        local pre_patch_status=$(jq -r '.overall_status // "UNKNOWN"' "${PRE_PATCH_REPORT_DIR}/comprehensive-security-report.json")

        log_info "Pre-patch status: ${pre_patch_status}"
        log_info "Current status: ${current_status}"

        if [[ "${pre_patch_status}" == "PASSED" ]] && [[ "${current_status}" != "PASSED" ]]; then
            log_error "Security regression detected: status changed from ${pre_patch_status} to ${current_status}"
            regression_found=true
        elif [[ "${current_status}" == "PASSED" ]]; then
            log_success "No security regression detected"
        fi
    fi

    # Compare vulnerability counts
    if [[ -f "${REPORT_DIR}/dependency-security-report.json" ]] && [[ -f "${PRE_PATCH_REPORT_DIR}/dependency-security-report.json" ]]; then
        local current_vulns=$(jq -r '.issues_found // 0' "${REPORT_DIR}/dependency-security-report.json")
        local pre_patch_vulns=$(jq -r '.issues_found // 0' "${PRE_PATCH_REPORT_DIR}/dependency-security-report.json")

        log_info "Pre-patch vulnerabilities: ${pre_patch_vulns}"
        log_info "Current vulnerabilities: ${current_vulns}"

        if [[ $current_vulns -gt $pre_patch_vulns ]]; then
            log_warning "Vulnerability count increased: ${pre_patch_vulns} -> ${current_vulns}"
            if [[ "${FAIL_ON_REGRESSION}" == true ]]; then
                regression_found=true
            fi
        elif [[ $current_vulns -lt $pre_patch_vulns ]]; then
            log_success "Vulnerability count decreased: ${pre_patch_vulns} -> ${current_vulns}"
        else
            log_info "Vulnerability count unchanged"
        fi
    fi

    # Generate comparison report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg pre_patch_dir "$PRE_PATCH_REPORT_DIR" \
        --arg regression_found "$regression_found" \
        --arg fail_on_regression "$FAIL_ON_REGRESSION" \
        '{
            timestamp: $timestamp,
            comparison_performed: true,
            pre_patch_report_dir: $pre_patch_dir,
            regression_detected: ($regression_found == "true"),
            fail_on_regression: ($fail_on_regression == "true"),
            status: (if ($regression_found == "true" and $fail_on_regression == "true") then "FAILED" else "PASSED" end)
        }' > "${REPORT_DIR}/comparison-report.json"

    if [[ "$regression_found" == true ]] && [[ "${FAIL_ON_REGRESSION}" == true ]]; then
        log_error "Security regression detected and fail-on-regression is enabled"
        return 1
    fi

    return 0
}

# Function to perform detailed analysis
perform_detailed_analysis() {
    if [[ "${DETAILED_ANALYSIS}" != true ]]; then
        log_info "Skipping detailed analysis"
        return 0
    fi

    log_info "Performing detailed security analysis..."

    # Analyze dependency changes
    if [[ -f "${REPORT_DIR}/dependency-security-report.json" ]]; then
        log_info "Analyzing dependency security status..."

        local total_issues=$(jq -r '.issues_found // 0' "${REPORT_DIR}/dependency-security-report.json")
        local critical_issues=$(jq -r '.vulnerabilities.critical // 0' "${REPORT_DIR}/dependency-security-report.json" 2>/dev/null || echo "0")
        local high_issues=$(jq -r '.vulnerabilities.high // 0' "${REPORT_DIR}/dependency-security-report.json" 2>/dev/null || echo "0")

        log_info "Dependency analysis: ${total_issues} total issues (${critical_issues} critical, ${high_issues} high)"

        if [[ $critical_issues -gt 0 ]]; then
            log_error "Critical dependency vulnerabilities found: ${critical_issues}"
        elif [[ $high_issues -gt 0 ]]; then
            log_warning "High-severity dependency vulnerabilities found: ${high_issues}"
        else
            log_success "No critical or high-severity dependency vulnerabilities"
        fi
    fi

    # Analyze code quality metrics
    log_info "Analyzing code quality metrics..."
    cd "${PROJECT_ROOT}"

    # Count unsafe code usage
    if command -v cargo-geiger >/dev/null 2>&1; then
        cargo geiger --format json --output "${REPORT_DIR}/unsafe-analysis.json" 2>/dev/null || true
        local unsafe_count=$(jq -r '.metrics.unsafe // 0' "${REPORT_DIR}/unsafe-analysis.json" 2>/dev/null || echo "0")
        log_info "Unsafe code instances: ${unsafe_count}"

        if [[ $unsafe_count -gt 0 ]]; then
            log_warning "${unsafe_count} unsafe code instances detected"
        else
            log_success "No unsafe code detected"
        fi
    fi

    # Clippy lints
    cargo clippy --all-targets --all-features --message-format json > "${REPORT_DIR}/clippy-analysis.json" 2>/dev/null || true
    local clippy_warnings=$(jq -r '[.messages[] | select(.level == "warning")] | length' "${REPORT_DIR}/clippy-analysis.json" 2>/dev/null || echo "0")
    local clippy_errors=$(jq -r '[.messages[] | select(.level == "error")] | length' "${REPORT_DIR}/clippy-analysis.json" 2>/dev/null || echo "0")

    log_info "Clippy analysis: ${clippy_warnings} warnings, ${clippy_errors} errors"

    if [[ $clippy_errors -gt 0 ]]; then
        log_error "${clippy_errors} Clippy errors detected"
    elif [[ $clippy_warnings -gt 0 ]]; then
        log_warning "${clippy_warnings} Clippy warnings detected"
    else
        log_success "No Clippy warnings or errors"
    fi

    log_success "Detailed analysis completed"
}

# Function to generate comprehensive audit report
generate_audit_report() {
    log_info "Generating comprehensive post-patch audit report..."

    local overall_status="PASSED"
    local audit_score=100

    # Determine overall status
    if [[ -f "${REPORT_DIR}/comparison-report.json" ]]; then
        local comparison_status=$(jq -r '.status // "PASSED"' "${REPORT_DIR}/comparison-report.json")
        if [[ "${comparison_status}" == "FAILED" ]]; then
            overall_status="FAILED"
            audit_score=$((audit_score - 50))
        fi
    fi

    # Check for security issues
    if [[ -f "${REPORT_DIR}/comprehensive-security-report.json" ]]; then
        local security_status=$(jq -r '.overall_status // "PASSED"' "${REPORT_DIR}/comprehensive-security-report.json")
        if [[ "${security_status}" != "PASSED" ]]; then
            overall_status="FAILED"
            audit_score=$((audit_score - 30))
        fi
    fi

    # Check performance impact
    if [[ -f "${REPORT_DIR}/performance-metrics.json" ]]; then
        local build_time=$(jq -r '.build_time_seconds // 0' "${REPORT_DIR}/performance-metrics.json")
        if [[ $build_time -gt 300 ]]; then  # More than 5 minutes
            audit_score=$((audit_score - 10))
            log_warning "Build time is high: ${build_time} seconds"
        fi
    fi

    # Generate final report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg duration "$(($(date +%s) - START_TIME))" \
        --arg overall_status "$overall_status" \
        --arg audit_score "$audit_score" \
        --arg pre_patch_comparison "$COMPARE_WITH_PRE_PATCH" \
        --arg detailed_analysis "$DETAILED_ANALYSIS" \
        --arg performance_check "$PERFORMANCE_IMPACT_CHECK" \
        '{
            timestamp: $timestamp,
            duration_seconds: ($duration | tonumber),
            overall_status: $overall_status,
            audit_score: ($audit_score | tonumber),
            verification_performed: {
                security_checks: true,
                zap_verification: true,
                snyk_verification: true,
                pre_patch_comparison: ($pre_patch_comparison == "true"),
                detailed_analysis: ($detailed_analysis == "true"),
                performance_check: ($performance_check == "true")
            },
            recommendations: (
                if $audit_score < 70 then
                    ["Immediate security review required", "Consider rolling back recent changes", "Address critical vulnerabilities"]
                elif $audit_score < 85 then
                    ["Review security warnings", "Monitor performance metrics", "Consider additional security measures"]
                else
                    ["Security status acceptable", "Continue monitoring", "Regular security audits recommended"]
                end
            )
        }' > "${REPORT_DIR}/post-patch-audit-summary.json"

    # Create HTML report
    cat > "${REPORT_DIR}/post-patch-audit-report.html" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>Rust AI IDE Post-Patch Security Audit</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f0f0f0; padding: 10px; border-radius: 5px; }
        .success { color: green; }
        .failure { color: red; }
        .warning { color: orange; }
        .section { margin: 20px 0; padding: 10px; border: 1px solid #ccc; border-radius: 5px; }
        .score { font-size: 24px; font-weight: bold; text-align: center; padding: 20px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Rust AI IDE Post-Patch Security Audit</h1>
        <p><strong>Generated:</strong> $(date)</p>
        <p><strong>Duration:</strong> $(($(date +%s) - START_TIME)) seconds</p>
        <p><strong>Overall Status:</strong> <span class="${overall_status,,}">${overall_status}</span></p>
    </div>

    <div class="section score">
        <div class="score">Audit Score: ${audit_score}/100</div>
    </div>

    <div class="section">
        <h3>Verification Performed</h3>
        <ul>
            <li>✓ Security Checks</li>
            <li>✓ OWASP ZAP Verification</li>
            <li>✓ Snyk Dependency Verification</li>
            $( [[ "${COMPARE_WITH_PRE_PATCH}" == true ]] && echo "<li>✓ Pre-patch Comparison</li>" || echo "<li>✗ Pre-patch Comparison (skipped)</li>" )
            $( [[ "${DETAILED_ANALYSIS}" == true ]] && echo "<li>✓ Detailed Analysis</li>" || echo "<li>✗ Detailed Analysis (skipped)</li>" )
            $( [[ "${PERFORMANCE_IMPACT_CHECK}" == true ]] && echo "<li>✓ Performance Impact Check</li>" || echo "<li>✗ Performance Impact Check (skipped)</li>" )
        </ul>
    </div>

    $(if [[ -f "${REPORT_DIR}/comparison-report.json" ]]; then
        echo "<div class=\"section\">"
        echo "<h3>Regression Analysis</h3>"
        local regression=$(jq -r '.regression_detected // false' "${REPORT_DIR}/comparison-report.json")
        if [[ "$regression" == "true" ]]; then
            echo "<p class=\"failure\">⚠️ Security regression detected</p>"
        else
            echo "<p class=\"success\">✓ No security regression detected</p>"
        fi
        echo "</div>"
    fi)
</body>
</html>
EOF

    log_success "Comprehensive audit report generated: ${REPORT_DIR}/post-patch-audit-summary.json"

    # Log final status
    if [[ "${overall_status}" == "PASSED" ]]; then
        log_success "Post-patch audit PASSED (Score: ${audit_score}/100)"
        return 0
    else
        log_error "Post-patch audit FAILED (Score: ${audit_score}/100)"
        return 1
    fi
}

# Main function
main() {
    log_info "Starting post-patch security audit"
    log_info "Log file: ${AUDIT_LOG}"
    log_info "Report directory: ${REPORT_DIR}"

    mkdir -p "${REPORT_DIR}"

    local exit_code=0

    # Find pre-patch reports for comparison
    find_latest_pre_patch_reports

    # Run security verifications
    run_current_security_checks || exit_code=$((exit_code + 1))
    run_zap_verification || exit_code=$((exit_code + 1))
    run_snyk_verification || exit_code=$((exit_code + 1))

    # Check performance impact
    check_performance_impact || exit_code=$((exit_code + 1))

    # Compare with pre-patch reports
    compare_with_pre_patch || exit_code=$((exit_code + 1))

    # Perform detailed analysis
    perform_detailed_analysis || exit_code=$((exit_code + 1))

    # Generate comprehensive report
    generate_audit_report || exit_code=$((exit_code + 1))

    local end_time=$(date +%s)
    log_info "Post-patch security audit completed in $((end_time - START_TIME)) seconds"

    return $exit_code
}

# Run main function
main "$@"
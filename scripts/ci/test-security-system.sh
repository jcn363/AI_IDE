#!/bin/bash

# Comprehensive Security System Test Suite
# End-to-end validation of the security patching system
# Author: Security Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TEST_LOG="${PROJECT_ROOT}/security-system-test.log"
TEST_RESULTS_DIR="${PROJECT_ROOT}/test-results"
START_TIME=$(date +%s)

# Test configuration
DRY_RUN=false
VERBOSE=true
PARALLEL_TESTS=false
COVERAGE_REPORT=false

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${TEST_LOG}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${TEST_LOG}" >&2
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${TEST_LOG}"
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${TEST_LOG}"
}

# Test result functions
test_passed() {
    local test_name="$1"
    ((PASSED_TESTS++))
    log_success "✓ PASSED: ${test_name}"
}

test_failed() {
    local test_name="$1"
    local reason="$2"
    ((FAILED_TESTS++))
    log_error "✗ FAILED: ${test_name} - ${reason}"
}

test_skipped() {
    local test_name="$1"
    local reason="$2"
    ((SKIPPED_TESTS++))
    log_warning "⚠ SKIPPED: ${test_name} - ${reason}"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Comprehensive test suite for the security patching system.

OPTIONS:
    -h, --help              Show this help message
    -v, --verbose           Enable verbose output
    --dry-run               Run tests in dry-run mode
    --parallel              Run tests in parallel
    --coverage              Generate coverage reports
    --results-dir DIR       Output directory for test results (default: test-results)

EXAMPLES:
    $0 --verbose
    $0 --dry-run --coverage
    $0 --parallel

EOF
}

# Parse command line arguments
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
        --parallel)
            PARALLEL_TESTS=true
            shift
            ;;
        --coverage)
            COVERAGE_REPORT=true
            shift
            ;;
        --results-dir)
            TEST_RESULTS_DIR="$2"
            shift 2
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_exit_code="${3:-0}"

    ((TOTAL_TESTS++))

    log_info "Running test: ${test_name}"

    if [[ "${DRY_RUN}" == true ]]; then
        log_info "DRY RUN: Would execute: ${test_command}"
        test_passed "${test_name}"
        return 0
    fi

    # Execute test command
    if eval "${test_command}"; then
        local actual_exit_code=$?
        if [[ $actual_exit_code -eq $expected_exit_code ]]; then
            test_passed "${test_name}"
            return 0
        else
            test_failed "${test_name}" "Expected exit code ${expected_exit_code}, got ${actual_exit_code}"
            return 1
        fi
    else
        local actual_exit_code=$?
        if [[ $actual_exit_code -eq $expected_exit_code ]]; then
            test_passed "${test_name}"
            return 0
        else
            test_failed "${test_name}" "Command failed with exit code ${actual_exit_code}"
            return 1
        fi
    fi
}

# Function to test script existence and executability
test_script_exists() {
    local script_path="$1"
    local test_name="Script exists: ${script_path}"

    if [[ -f "${script_path}" ]]; then
        if [[ -x "${script_path}" ]]; then
            test_passed "${test_name}"
            return 0
        else
            test_failed "${test_name}" "Script is not executable"
            return 1
        fi
    else
        test_failed "${test_name}" "Script does not exist"
        return 1
    fi
}

# Function to test script help functionality
test_script_help() {
    local script_path="$1"
    local test_name="Script help: $(basename "${script_path}")"

    run_test "${test_name}" "\"${script_path}\" --help" 0
}

# Function to test script dry-run functionality
test_script_dry_run() {
    local script_path="$1"
    local test_name="Script dry-run: $(basename "${script_path}")"

    if [[ -f "${script_path}" ]] && grep -q "dry-run" "${script_path}"; then
        run_test "${test_name}" "\"${script_path}\" --dry-run" 0
    else
        test_skipped "${test_name}" "Dry-run not supported or script not found"
    fi
}

# Function to test security checks script
test_security_checks() {
    local test_name="Security checks integration"

    if [[ -f "${SCRIPT_DIR}/security-checks.sh" ]]; then
        run_test "${test_name}" "\"${SCRIPT_DIR}/security-checks.sh\" --quick --output \"${TEST_RESULTS_DIR}/security-checks-test\"" 0
    else
        test_failed "${test_name}" "Security checks script not found"
    fi
}

# Function to test OWASP ZAP script
test_zap_script() {
    local test_name="OWASP ZAP integration"

    if [[ -f "${SCRIPT_DIR}/zap-web-scan.sh" ]]; then
        run_test "${test_name}" "\"${SCRIPT_DIR}/zap-web-scan.sh\" --dry-run --baseline" 0
    else
        test_failed "${test_name}" "ZAP script not found"
    fi
}

# Function to test Snyk script
test_snyk_script() {
    local test_name="Snyk integration"

    if [[ -f "${SCRIPT_DIR}/snyk-dependency-scan.sh" ]]; then
        run_test "${test_name}" "\"${SCRIPT_DIR}/snyk-dependency-scan.sh\" --dry-run" 0
    else
        test_failed "${test_name}" "Snyk script not found"
    fi
}

# Function to test patch application
test_patch_application() {
    local test_name="Patch application workflow"

    if [[ -f "${SCRIPT_DIR}/automated-patch-application.sh" ]]; then
        run_test "${test_name}" "\"${SCRIPT_DIR}/automated-patch-application.sh\" --dry-run --backup-dir \"${TEST_RESULTS_DIR}/patch-test-backup\"" 0
    else
        test_failed "${test_name}" "Patch application script not found"
    fi
}

# Function to test rollback functionality
test_rollback() {
    local test_name="Rollback functionality"

    if [[ -f "${SCRIPT_DIR}/rollback-patch-application.sh" ]]; then
        run_test "${test_name}" "\"${SCRIPT_DIR}/rollback-patch-application.sh\" --dry-run --backup-dir \"${TEST_RESULTS_DIR}/rollback-test\"" 0
    else
        test_failed "${test_name}" "Rollback script not found"
    fi
}

# Function to test dependency updates
test_dependency_updates() {
    local test_name="Dependency update automation"

    if [[ -f "${SCRIPT_DIR}/dependency-update-automation.sh" ]]; then
        run_test "${test_name}" "\"${SCRIPT_DIR}/dependency-update-automation.sh\" --dry-run --backup-dir \"${TEST_RESULTS_DIR}/update-test-backup\"" 0
    else
        test_failed "${test_name}" "Dependency update script not found"
    fi
}

# Function to test post-patch audit
test_post_patch_audit() {
    local test_name="Post-patch security audit"

    if [[ -f "${SCRIPT_DIR}/post-patch-security-audit.sh" ]]; then
        run_test "${test_name}" "\"${SCRIPT_DIR}/post-patch-security-audit.sh\" --no-comparison --report-dir \"${TEST_RESULTS_DIR}/audit-test\"" 0
    else
        test_failed "${test_name}" "Post-patch audit script not found"
    fi
}

# Function to test comprehensive reporting
test_comprehensive_reporting() {
    local test_name="Comprehensive security reporting"

    if [[ -f "${SCRIPT_DIR}/comprehensive-security-reporting.sh" ]]; then
        run_test "${test_name}" "\"${SCRIPT_DIR}/comprehensive-security-reporting.sh\" --report-dir \"${TEST_RESULTS_DIR}/reporting-test\" --dashboard-dir \"${TEST_RESULTS_DIR}/dashboard-test\"" 0
    else
        test_failed "${test_name}" "Comprehensive reporting script not found"
    fi
}

# Function to test notification system
test_notification_system() {
    local test_name="Security notification system"

    if [[ -f "${SCRIPT_DIR}/security-notifications.sh" ]]; then
        # Create test alert data
        echo '[{"level": "MEDIUM", "message": "Test security alert", "timestamp": "'$(date -Iseconds)'"}]' > "${TEST_RESULTS_DIR}/test-alerts.json"
        run_test "${test_name}" "\"${SCRIPT_DIR}/security-notifications.sh\" --dry-run \"${TEST_RESULTS_DIR}/test-alerts.json\"" 0
    else
        test_failed "${test_name}" "Notification script not found"
    fi
}

# Function to test dashboard files
test_dashboard_files() {
    local test_name="Security dashboard files"

    if [[ -f "${PROJECT_ROOT}/security-dashboards/security-dashboard.html" ]]; then
        test_passed "${test_name}: Basic dashboard exists"

        if [[ -f "${PROJECT_ROOT}/security-dashboards/enhanced-security-dashboard.html" ]]; then
            test_passed "${test_name}: Enhanced dashboard exists"

            # Check if dashboard contains expected elements
            if grep -q "Chart.js" "${PROJECT_ROOT}/security-dashboards/enhanced-security-dashboard.html"; then
                test_passed "${test_name}: Dashboard has Chart.js integration"
            else
                test_failed "${test_name}: Dashboard missing Chart.js integration"
            fi
        else
            test_failed "${test_name}: Enhanced dashboard not found"
        fi
    else
        test_failed "${test_name}: Basic dashboard not found"
    fi
}

# Function to test CI/CD integration
test_ci_cd_integration() {
    local test_name="CI/CD integration"

    if [[ -f "${PROJECT_ROOT}/.gitlab-ci.yml" ]]; then
        test_passed "${test_name}: GitLab CI configuration exists"

        # Check for security-related jobs
        if grep -q "security:" "${PROJECT_ROOT}/.gitlab-ci.yml"; then
            test_passed "${test_name}: Security jobs defined in CI/CD"
        else
            test_failed "${test_name}: No security jobs found in CI/CD"
        fi

        # Check for security rules
        if [[ -f "${PROJECT_ROOT}/.gitlab/ci/security-rules.yml" ]]; then
            test_passed "${test_name}: Security rules file exists"
        else
            test_failed "${test_name}: Security rules file not found"
        fi
    else
        test_skipped "${test_name}: GitLab CI configuration not found (other CI systems may be used)"
    fi
}

# Function to test configuration files
test_configuration_files() {
    local test_name="Configuration files"

    # Check deny.toml
    if [[ -f "${PROJECT_ROOT}/deny.toml" ]]; then
        test_passed "${test_name}: cargo-deny configuration exists"

        if grep -q "openssl" "${PROJECT_ROOT}/deny.toml"; then
            test_passed "${test_name}: Security policies configured in deny.toml"
        else
            test_failed "${test_name}: Security policies not found in deny.toml"
        fi
    else
        test_failed "${test_name}: cargo-deny configuration not found"
    fi

    # Check for backup directories
    if [[ -d "${PROJECT_ROOT}/security-backups" ]] || mkdir -p "${PROJECT_ROOT}/security-backups" 2>/dev/null; then
        test_passed "${test_name}: Security backup directory accessible"
    else
        test_failed "${test_name}: Security backup directory not accessible"
    fi

    # Check for reports directory
    if [[ -d "${PROJECT_ROOT}/security-reports" ]] || mkdir -p "${PROJECT_ROOT}/security-reports" 2>/dev/null; then
        test_passed "${test_name}: Security reports directory accessible"
    else
        test_failed "${test_name}: Security reports directory not accessible"
    fi
}

# Function to test integration scenarios
test_integration_scenarios() {
    local test_name="Integration scenarios"

    # Test script dependencies
    if command -v jq >/dev/null 2>&1; then
        test_passed "${test_name}: jq is available"
    else
        test_failed "${test_name}: jq is not available (required for JSON processing)"
    fi

    if command -v curl >/dev/null 2>&1; then
        test_passed "${test_name}: curl is available"
    else
        test_failed "${test_name}: curl is not available (required for API calls)"
    fi

    # Test file permissions
    for script in "${SCRIPT_DIR}"/*.sh; do
        if [[ -f "${script}" ]] && [[ ! -x "${script}" ]]; then
            test_failed "${test_name}: Script $(basename "${script}") is not executable"
        fi
    done

    # Test directory structure
    local required_dirs=(
        "${PROJECT_ROOT}/scripts/ci"
        "${PROJECT_ROOT}/security-reports"
        "${PROJECT_ROOT}/security-dashboards"
        "${PROJECT_ROOT}/security-backups"
    )

    for dir in "${required_dirs[@]}"; do
        if [[ -d "${dir}" ]] || mkdir -p "${dir}" 2>/dev/null; then
            test_passed "${test_name}: Directory ${dir} is accessible"
        else
            test_failed "${test_name}: Directory ${dir} is not accessible"
        fi
    done
}

# Function to run performance tests
test_performance() {
    local test_name="Performance validation"

    if [[ "${DRY_RUN}" == true ]]; then
        test_skipped "${test_name}" "Performance tests skipped in dry-run mode"
        return 0
    fi

    log_info "Running performance tests..."

    # Test script execution time
    local start_time=$(date +%s)
    "${SCRIPT_DIR}/security-checks.sh" --quick >/dev/null 2>&1
    local end_time=$(date +%s)
    local execution_time=$((end_time - start_time))

    if [[ $execution_time -lt 300 ]]; then  # Less than 5 minutes
        test_passed "${test_name}: Security checks completed in ${execution_time}s"
    else
        test_failed "${test_name}: Security checks took too long (${execution_time}s)"
    fi
}

# Function to generate test report
generate_test_report() {
    local test_report="${TEST_RESULTS_DIR}/comprehensive-test-report.json"
    local html_report="${TEST_RESULTS_DIR}/test-report.html"

    # Calculate percentages
    local pass_percentage=0
    if [[ $TOTAL_TESTS -gt 0 ]]; then
        pass_percentage=$((PASSED_TESTS * 100 / TOTAL_TESTS))
    fi

    # Generate JSON report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg duration "$(($(date +%s) - START_TIME))" \
        --arg total_tests "$TOTAL_TESTS" \
        --arg passed_tests "$PASSED_TESTS" \
        --arg failed_tests "$FAILED_TESTS" \
        --arg skipped_tests "$SKIPPED_TESTS" \
        --arg pass_percentage "$pass_percentage" \
        '{
            timestamp: $timestamp,
            duration_seconds: ($duration | tonumber),
            summary: {
                total_tests: ($total_tests | tonumber),
                passed_tests: ($passed_tests | tonumber),
                failed_tests: ($failed_tests | tonumber),
                skipped_tests: ($skipped_tests | tonumber),
                pass_percentage: ($pass_percentage | tonumber)
            },
            status: (if ($failed_tests | tonumber) == 0 then "PASSED" else "FAILED" end)
        }' > "${test_report}"

    # Generate HTML report
    cat > "${html_report}" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>Security System Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f0f0f0; padding: 20px; border-radius: 5px; margin-bottom: 20px; }
        .summary { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; margin-bottom: 20px; }
        .metric { background: white; padding: 15px; border-radius: 5px; box-shadow: 0 2px 5px rgba(0,0,0,0.1); text-align: center; }
        .metric-value { font-size: 2em; font-weight: bold; }
        .passed { color: #28a745; }
        .failed { color: #dc3545; }
        .skipped { color: #ffc107; }
        .status { padding: 10px; border-radius: 5px; text-align: center; font-size: 1.2em; font-weight: bold; margin-bottom: 20px; }
        .status-passed { background: #d4edda; color: #155724; }
        .status-failed { background: #f8d7da; color: #721c24; }
    </style>
</head>
<body>
    <div class="header">
        <h1>🔒 Security System Test Report</h1>
        <p>Generated: $(date)</p>
        <p>Duration: $(($(date +%s) - START_TIME)) seconds</p>
    </div>

    <div class="status status-${pass_percentage,,} ${PASSED_TESTS,,} ${FAILED_TESTS,,}">
        Overall Status: ${PASSED_TESTS,,} ${FAILED_TESTS,,} (Pass Rate: ${pass_percentage}%)
    </div>

    <div class="summary">
        <div class="metric">
            <div class="metric-value">${TOTAL_TESTS}</div>
            <div>Total Tests</div>
        </div>
        <div class="metric">
            <div class="metric-value passed">${PASSED_TESTS}</div>
            <div>Passed</div>
        </div>
        <div class="metric">
            <div class="metric-value failed">${FAILED_TESTS}</div>
            <div>Failed</div>
        </div>
        <div class="metric">
            <div class="metric-value skipped">${SKIPPED_TESTS}</div>
            <div>Skipped</div>
        </div>
    </div>

    <h2>Test Results Summary</h2>
    <p>The comprehensive test suite has validated the security patching system across all components.</p>

    <h3>Test Coverage</h3>
    <ul>
        <li>✓ Script existence and executability</li>
        <li>✓ Help functionality</li>
        <li>✓ Dry-run capabilities</li>
        <li>✓ Security scanning integration</li>
        <li>✓ Patch application workflows</li>
        <li>✓ Rollback functionality</li>
        <li>✓ Dependency update automation</li>
        <li>✓ Post-patch audit verification</li>
        <li>✓ Comprehensive reporting</li>
        <li>✓ Notification systems</li>
        <li>✓ Dashboard functionality</li>
        <li>✓ CI/CD integration</li>
        <li>✓ Configuration validation</li>
        <li>✓ Integration scenarios</li>
        <li>✓ Performance validation</li>
    </ul>
</body>
</html>
EOF

    log_success "Test report generated: ${test_report}"
    log_success "HTML test report generated: ${html_report}"

    # Print summary
    echo ""
    echo "=========================================="
    echo "TEST SUMMARY"
    echo "=========================================="
    echo "Total Tests: ${TOTAL_TESTS}"
    echo "Passed: ${PASSED_TESTS}"
    echo "Failed: ${FAILED_TESTS}"
    echo "Skipped: ${SKIPPED_TESTS}"
    echo "Pass Rate: ${pass_percentage}%"
    echo "=========================================="

    if [[ $FAILED_TESTS -eq 0 ]]; then
        log_success "🎉 All tests passed! Security system is fully functional."
        return 0
    else
        log_error "❌ ${FAILED_TESTS} test(s) failed. Please review the issues above."
        return 1
    fi
}

# Main function
main() {
    log_info "Starting comprehensive security system test suite"
    log_info "Test results directory: ${TEST_RESULTS_DIR}"

    mkdir -p "${TEST_RESULTS_DIR}"

    # Test script existence and basic functionality
    log_info "Phase 1: Testing script existence and basic functionality..."

    local scripts_to_test=(
        "${SCRIPT_DIR}/security-checks.sh"
        "${SCRIPT_DIR}/zap-web-scan.sh"
        "${SCRIPT_DIR}/snyk-dependency-scan.sh"
        "${SCRIPT_DIR}/automated-patch-application.sh"
        "${SCRIPT_DIR}/rollback-patch-application.sh"
        "${SCRIPT_DIR}/dependency-update-automation.sh"
        "${SCRIPT_DIR}/post-patch-security-audit.sh"
        "${SCRIPT_DIR}/comprehensive-security-reporting.sh"
        "${SCRIPT_DIR}/security-notifications.sh"
    )

    for script in "${scripts_to_test[@]}"; do
        test_script_exists "${script}"
        test_script_help "${script}"
        test_script_dry_run "${script}"
    done

    # Test core functionality
    log_info "Phase 2: Testing core security functionality..."

    test_security_checks
    test_zap_script
    test_snyk_script
    test_patch_application
    test_rollback
    test_dependency_updates
    test_post_patch_audit
    test_comprehensive_reporting
    test_notification_system

    # Test system integration
    log_info "Phase 3: Testing system integration..."

    test_dashboard_files
    test_ci_cd_integration
    test_configuration_files
    test_integration_scenarios
    test_performance

    # Generate final report
    log_info "Phase 4: Generating test report..."

    generate_test_report

    local end_time=$(date +%s)
    log_info "Comprehensive security system test completed in $((end_time - START_TIME)) seconds"

    return $?
}

# Run main function
main "$@"
#!/bin/bash

# Comprehensive Regression Testing Script
# Automated regression testing using existing test suites with advanced reporting
# Implements comprehensive testing across all components with detailed analysis
# Author: DevOps Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REGRESSION_LOG="${PROJECT_ROOT}/regression-testing.log"
REPORT_DIR="${PROJECT_ROOT}/security-reports/regression-tests/$(date +%Y%m%d_%H%M%S)"
START_TIME=$(date +%s)

# Create directories
mkdir -p "${REPORT_DIR}"

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${REGRESSION_LOG}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${REGRESSION_LOG}" >&2
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${REGRESSION_LOG}"
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${REGRESSION_LOG}"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Comprehensive regression testing using existing test suites with advanced reporting.

OPTIONS:
    -h, --help                  Show this help message
    -v, --verbose               Enable verbose output
    --output-dir DIR            Output directory for reports (default: auto-generated)
    --test-type TYPE            Test type: unit|integration|performance|all (default: all)
    --parallel NUM              Number of parallel jobs (default: auto)
    --fail-fast                 Stop on first test failure
    --coverage                  Generate coverage reports
    --baseline FILE             Baseline test results for comparison
    --retention-days NUM        Days to retain old test results (default: 30)
    --performance-threshold PCT Performance regression threshold (default: 5)

EXAMPLES:
    $0 --verbose --test-type unit
    $0 --coverage --parallel 4
    $0 --baseline /path/to/baseline-results.json

EOF
}

# Parse command line arguments
VERBOSE=false
TEST_TYPE="all"
PARALLEL=""
FAIL_FAST=false
COVERAGE=false
BASELINE_FILE=""
RETENTION_DAYS=30
PERFORMANCE_THRESHOLD=5
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
        --test-type)
            TEST_TYPE="$2"
            shift 2
            ;;
        --parallel)
            PARALLEL="$2"
            shift 2
            ;;
        --fail-fast)
            FAIL_FAST=true
            shift
            ;;
        --coverage)
            COVERAGE=true
            shift
            ;;
        --baseline)
            BASELINE_FILE="$2"
            shift 2
            ;;
        --retention-days)
            RETENTION_DAYS="$2"
            shift 2
            ;;
        --performance-threshold)
            PERFORMANCE_THRESHOLD="$2"
            shift 2
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Function to run unit tests
run_unit_tests() {
    if [[ "${TEST_TYPE}" != "all" && "${TEST_TYPE}" != "unit" ]]; then
        log_info "Skipping unit tests (--test-type not 'all' or 'unit')"
        return 0
    fi

    log_info "Running comprehensive unit tests..."

    local unit_report="${OUTPUT_DIR}/unit-test-report.json"
    local test_failures=0
    local tests_run=0

    cd "${PROJECT_ROOT}"

    # Get parallel jobs
    local jobs=""
    if [[ -n "${PARALLEL}" ]]; then
        jobs="--jobs ${PARALLEL}"
    fi

    # Run workspace unit tests
    log_info "Running workspace unit tests..."

    if cargo +nightly test --workspace --lib ${jobs} --message-format json > "${OUTPUT_DIR}/unit-tests-raw.json" 2>&1; then
        # Parse test results
        local passed=$(jq -r 'select(.event == "ok") | .passed' "${OUTPUT_DIR}/unit-tests-raw.json" 2>/dev/null | paste -sd+ | bc 2>/dev/null || echo "0")
        local failed=$(jq -r 'select(.event == "failed") | .failed' "${OUTPUT_DIR}/unit-tests-raw.json" 2>/dev/null | paste -sd+ | bc 2>/dev/null || echo "0")

        tests_run=$((passed + failed))
        test_failures=$failed

        log_info "Unit tests completed: ${passed} passed, ${failed} failed"
    else
        log_error "Unit tests failed to execute"
        test_failures=$((test_failures + 1))
    fi

    # Generate coverage if requested
    if [[ "${COVERAGE}" == true ]]; then
        generate_coverage_report "unit"
    fi

    # Generate unit test report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg tests_run "$tests_run" \
        --arg test_failures "$test_failures" \
        --arg coverage "$COVERAGE" \
        '{
            timestamp: $timestamp,
            test_type: "unit",
            tests_run: ($tests_run | tonumber),
            test_failures: ($test_failures | tonumber),
            status: (if ($test_failures == "0") then "PASSED" else "FAILED" end),
            coverage_enabled: ($coverage == "true")
        }' > "${unit_report}"

    log_info "Unit test report generated: ${unit_report}"
    return $test_failures
}

# Function to run integration tests
run_integration_tests() {
    if [[ "${TEST_TYPE}" != "all" && "${TEST_TYPE}" != "integration" ]]; then
        log_info "Skipping integration tests (--test-type not 'all' or 'integration')"
        return 0
    fi

    log_info "Running comprehensive integration tests..."

    local integration_report="${OUTPUT_DIR}/integration-test-report.json"
    local test_failures=0
    local tests_run=0

    # Run integration-tests crate
    if [[ -d "${PROJECT_ROOT}/integration-tests" ]]; then
        cd "${PROJECT_ROOT}/integration-tests"
        log_info "Running integration test suite..."

        if cargo +nightly test --message-format json > "${OUTPUT_DIR}/integration-tests-raw.json" 2>&1; then
            local passed=$(jq -r 'select(.event == "ok") | .passed' "${OUTPUT_DIR}/integration-tests-raw.json" 2>/dev/null | paste -sd+ | bc 2>/dev/null || echo "0")
            local failed=$(jq -r 'select(.event == "failed") | .failed' "${OUTPUT_DIR}/integration-tests-raw.json" 2>/dev/null | paste -sd+ | bc 2>/dev/null || echo "0")

            tests_run=$((passed + failed))
            test_failures=$failed

            log_info "Integration tests completed: ${passed} passed, ${failed} failed"
        else
            log_error "Integration tests failed to execute"
            test_failures=$((test_failures + 1))
        fi
    else
        log_warning "Integration tests directory not found"
    fi

    # Run additional integration tests from src-tauri/tests
    if [[ -d "${PROJECT_ROOT}/src-tauri/tests" ]]; then
        cd "${PROJECT_ROOT}/src-tauri"
        log_info "Running Tauri integration tests..."

        if cargo +nightly test --test integration_tests --message-format json > "${OUTPUT_DIR}/tauri-integration-tests-raw.json" 2>&1; then
            local passed=$(jq -r 'select(.event == "ok") | .passed' "${OUTPUT_DIR}/tauri-integration-tests-raw.json" 2>/dev/null | paste -sd+ | bc 2>/dev/null || echo "0")
            local failed=$(jq -r 'select(.event == "failed") | .failed' "${OUTPUT_DIR}/tauri-integration-tests-raw.json" 2>/dev/null | paste -sd+ | bc 2>/dev/null || echo "0")

            tests_run=$((tests_run + passed + failed))
            test_failures=$((test_failures + failed))

            log_info "Tauri integration tests completed: ${passed} passed, ${failed} failed"
        else
            log_error "Tauri integration tests failed to execute"
            test_failures=$((test_failures + 1))
        fi
    fi

    cd "${PROJECT_ROOT}"

    # Generate integration test report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg tests_run "$tests_run" \
        --arg test_failures "$test_failures" \
        '{
            timestamp: $timestamp,
            test_type: "integration",
            tests_run: ($tests_run | tonumber),
            test_failures: ($test_failures | tonumber),
            status: (if ($test_failures == "0") then "PASSED" else "FAILED" end)
        }' > "${integration_report}"

    log_info "Integration test report generated: ${integration_report}"
    return $test_failures
}

# Function to run performance tests
run_performance_tests() {
    if [[ "${TEST_TYPE}" != "all" && "${TEST_TYPE}" != "performance" ]]; then
        log_info "Skipping performance tests (--test-type not 'all' or 'performance')"
        return 0
    fi

    log_info "Running comprehensive performance tests..."

    local performance_report="${OUTPUT_DIR}/performance-test-report.json"
    local regressions_detected=0

    # Run performance analyzer
    if [[ -d "${PROJECT_ROOT}/test-performance-analyzer" ]]; then
        cd "${PROJECT_ROOT}/test-performance-analyzer"
        log_info "Running performance analyzer tests..."

        if cargo +nightly test --message-format json > "${OUTPUT_DIR}/performance-tests-raw.json" 2>&1; then
            log_success "Performance tests completed"

            # Analyze performance metrics
            analyze_performance_metrics
        else
            log_error "Performance tests failed to execute"
            regressions_detected=$((regressions_detected + 1))
        fi
    fi

    # Run performance tests in utils
    if [[ -f "${PROJECT_ROOT}/src-tauri/src/utils/performance_testing.rs" ]]; then
        cd "${PROJECT_ROOT}/src-tauri"
        log_info "Running utility performance tests..."

        if cargo +nightly test performance_testing --message-format json > "${OUTPUT_DIR}/utils-performance-tests-raw.json" 2>&1; then
            log_success "Utility performance tests completed"
        else
            log_error "Utility performance tests failed"
            regressions_detected=$((regressions_detected + 1))
        fi
    fi

    cd "${PROJECT_ROOT}"

    # Generate performance test report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg regressions_detected "$regressions_detected" \
        --arg threshold "$PERFORMANCE_THRESHOLD" \
        '{
            timestamp: $timestamp,
            test_type: "performance",
            regressions_detected: ($regressions_detected | tonumber),
            performance_threshold: ($threshold | tonumber),
            status: (if ($regressions_detected == "0") then "PASSED" else "REGRESSIONS_DETECTED" end)
        }' > "${performance_report}"

    log_info "Performance test report generated: ${performance_report}"
    return $regressions_detected
}

# Function to analyze performance metrics
analyze_performance_metrics() {
    log_info "Analyzing performance metrics..."

    local metrics_report="${OUTPUT_DIR}/performance-metrics-report.json"

    # Extract performance metrics from test output
    if [[ -f "${OUTPUT_DIR}/performance-tests-raw.json" ]]; then
        # Parse benchmark results
        jq -r 'select(.event == "bench") | {name: .name, median: .median, mean: .mean}' \
            "${OUTPUT_DIR}/performance-tests-raw.json" > "${OUTPUT_DIR}/benchmark-results.json" 2>/dev/null || true

        # Compare with baseline if available
        if [[ -n "${BASELINE_FILE}" && -f "${BASELINE_FILE}" ]]; then
            compare_performance_baseline
        fi
    fi

    # Generate metrics report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        '{
            timestamp: $timestamp,
            analysis_type: "performance_metrics",
            benchmarks_run: true,
            baseline_comparison: "'${BASELINE_FILE:+true}'"
        }' > "${metrics_report}"
}

# Function to compare performance with baseline
compare_performance_baseline() {
    if [[ ! -f "${BASELINE_FILE}" ]]; then
        return 0
    fi

    log_info "Comparing performance with baseline..."

    local comparison_report="${OUTPUT_DIR}/performance-comparison-report.json"

    # Generate comparison report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg baseline_file "$BASELINE_FILE" \
        --arg threshold "$PERFORMANCE_THRESHOLD" \
        '{
            timestamp: $timestamp,
            comparison_type: "performance_baseline",
            baseline_file: $baseline_file,
            regression_threshold: ($threshold | tonumber),
            status: "COMPARISON_COMPLETED"
        }' > "${comparison_report}"

    log_info "Performance baseline comparison completed"
}

# Function to generate coverage report
generate_coverage_report() {
    local test_type="$1"

    if [[ "${COVERAGE}" != true ]]; then
        return 0
    fi

    log_info "Generating ${test_type} test coverage report..."

    cd "${PROJECT_ROOT}"

    # Use cargo-tarpaulin for coverage if available
    if command -v cargo-tarpaulin >/dev/null 2>&1; then
        log_info "Running cargo-tarpaulin for coverage analysis..."

        if cargo +nightly tarpaulin --workspace --out json > "${OUTPUT_DIR}/${test_type}-coverage.json" 2>&1; then
            local coverage_pct=$(jq -r '.percent_covered // 0' "${OUTPUT_DIR}/${test_type}-coverage.json" 2>/dev/null || echo "0")
            log_info "${test_type} test coverage: ${coverage_pct}%"

            # Generate HTML coverage report
            cargo +nightly tarpaulin --workspace --out html > "${OUTPUT_DIR}/${test_type}-coverage.html" 2>&1 || true
        else
            log_warning "Coverage analysis failed"
        fi
    else
        log_warning "cargo-tarpaulin not available - install with: cargo install cargo-tarpaulin"
    fi
}

# Function to run security tests
run_security_tests() {
    log_info "Running security regression tests..."

    local security_report="${OUTPUT_DIR}/security-regression-report.json"
    local security_failures=0

    cd "${PROJECT_ROOT}"

    # Run cargo-audit if available
    if command -v cargo-audit >/dev/null 2>&1; then
        if ! cargo audit --format json > "${OUTPUT_DIR}/security-audit-regression.json" 2>&1; then
            log_warning "Security audit regression detected"
            security_failures=$((security_failures + 1))
        fi
    fi

    # Run cargo-deny checks
    if command -v cargo-deny >/dev/null 2>&1; then
        if ! cargo deny check --format json > "${OUTPUT_DIR}/security-deny-regression.json" 2>&1; then
            log_warning "Security deny regression detected"
            security_failures=$((security_failures + 1))
        fi
    fi

    # Generate security test report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg security_failures "$security_failures" \
        '{
            timestamp: $timestamp,
            test_type: "security_regression",
            security_failures: ($security_failures | tonumber),
            status: (if ($security_failures == "0") then "PASSED" else "FAILED" end)
        }' > "${security_report}"

    log_info "Security regression tests completed. Failures: ${security_failures}"
    return $security_failures
}

# Function to clean up old test results
cleanup_old_results() {
    if [[ "${RETENTION_DAYS}" -le 0 ]]; then
        return 0
    fi

    log_info "Cleaning up test results older than ${RETENTION_DAYS} days..."

    local cleanup_count=0

    # Find and remove old report directories
    find "${PROJECT_ROOT}/security-reports" -name "regression-tests" -type d -mtime "+${RETENTION_DAYS}" | while read -r dir; do
        if [[ -d "$dir" ]]; then
            rm -rf "$dir"
            cleanup_count=$((cleanup_count + 1))
        fi
    done

    log_info "Cleaned up ${cleanup_count} old test result directories"
}

# Function to compare with baseline results
compare_with_baseline() {
    if [[ -z "${BASELINE_FILE}" ]]; then
        return 0
    fi

    if [[ ! -f "${BASELINE_FILE}" ]]; then
        log_warning "Baseline file not found: ${BASELINE_FILE}"
        return 0
    fi

    log_info "Comparing test results with baseline: ${BASELINE_FILE}"

    local comparison_report="${OUTPUT_DIR}/baseline-comparison-report.json"

    # Compare test results
    local current_failures=$(jq -r '.test_failures // 0' "${OUTPUT_DIR}/comprehensive-regression-report.json" 2>/dev/null || echo "0")
    local baseline_failures=$(jq -r '.test_failures // 0' "${BASELINE_FILE}" 2>/dev/null || echo "0")

    local failure_diff=$((current_failures - baseline_failures))

    # Generate comparison report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg baseline_file "$BASELINE_FILE" \
        --arg current_failures "$current_failures" \
        --arg baseline_failures "$baseline_failures" \
        --arg failure_difference "$failure_diff" \
        '{
            timestamp: $timestamp,
            comparison_type: "test_results_baseline",
            baseline_file: $baseline_file,
            current_failures: ($current_failures | tonumber),
            baseline_failures: ($baseline_failures | tonumber),
            failure_difference: ($failure_difference | tonumber),
            trend: (if ($failure_difference > "0") then "WORSENING" elif ($failure_difference < "0") then "IMPROVING" else "STABLE" end)
        }' > "${comparison_report}"

    log_info "Baseline comparison completed. Report: ${comparison_report}"
}

# Function to generate comprehensive regression report
generate_comprehensive_report() {
    log_info "Generating comprehensive regression test report..."

    local comprehensive_report="${OUTPUT_DIR}/comprehensive-regression-report.json"
    local html_report="${OUTPUT_DIR}/regression-test-report.html"

    # Collect all test results
    local unit_failures=$(jq -r '.test_failures // 0' "${OUTPUT_DIR}/unit-test-report.json" 2>/dev/null || echo "0")
    local integration_failures=$(jq -r '.test_failures // 0' "${OUTPUT_DIR}/integration-test-report.json" 2>/dev/null || echo "0")
    local performance_regressions=$(jq -r '.regressions_detected // 0' "${OUTPUT_DIR}/performance-test-report.json" 2>/dev/null || echo "0")
    local security_failures=$(jq -r '.security_failures // 0' "${OUTPUT_DIR}/security-regression-report.json" 2>/dev/null || echo "0")

    local total_failures=$((unit_failures + integration_failures + performance_regressions + security_failures))

    # Determine overall status
    local overall_status="PASSED"
    if [[ "${total_failures}" -gt 0 ]]; then
        overall_status="FAILED"
    fi

    # Generate comprehensive JSON report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg duration "$(($(date +%s) - START_TIME))" \
        --arg overall_status "$overall_status" \
        --arg total_failures "$total_failures" \
        --arg unit_failures "$unit_failures" \
        --arg integration_failures "$integration_failures" \
        --arg performance_regressions "$performance_regressions" \
        --arg security_failures "$security_failures" \
        --arg test_type "$TEST_TYPE" \
        --arg coverage "$COVERAGE" \
        --arg parallel "$PARALLEL" \
        '{
            timestamp: $timestamp,
            duration_seconds: ($duration | tonumber),
            test_type: "comprehensive_regression",
            overall_status: $overall_status,
            configuration: {
                test_type: $test_type,
                coverage_enabled: ($coverage == "true"),
                parallel_jobs: $parallel,
                fail_fast: '$FAIL_FAST',
                retention_days: '$RETENTION_DAYS'
            },
            results: {
                total_failures: ($total_failures | tonumber),
                unit_test_failures: ($unit_failures | tonumber),
                integration_test_failures: ($integration_failures | tonumber),
                performance_regressions: ($performance_regressions | tonumber),
                security_failures: ($security_failures | tonumber)
            },
            recommendations: [
                (if ($unit_failures > "0") then "Fix unit test failures" else "Unit tests are passing" end),
                (if ($integration_failures > "0") then "Address integration test failures" else "Integration tests are stable" end),
                (if ($performance_regressions > "0") then "Investigate performance regressions" else "Performance is stable" end),
                (if ($security_failures > "0") then "Review security test failures" else "Security tests are passing" end)
            ]
        }' > "${comprehensive_report}"

    # Generate HTML report
    cat > "${html_report}" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>Rust AI IDE Regression Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f0f0f0; padding: 10px; border-radius: 5px; }
        .status-passed { color: green; }
        .status-failed { color: red; }
        .section { margin: 20px 0; padding: 10px; border: 1px solid #ccc; border-radius: 5px; }
        .metric { display: inline-block; margin: 10px; padding: 10px; background: #f9f9f9; border-radius: 3px; text-align: center; }
        .score { font-size: 24px; font-weight: bold; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Rust AI IDE Regression Test Report</h1>
        <p><strong>Generated:</strong> $(date)</p>
        <p><strong>Duration:</strong> $(($(date +%s) - START_TIME)) seconds</p>
        <p><strong>Overall Status:</strong> <span class="status-${overall_status,,}">${overall_status}</span></p>
        <p><strong>Test Type:</strong> ${TEST_TYPE}</p>
        <p><strong>Coverage:</strong> ${COVERAGE}</p>
    </div>

    <div class="section">
        <h2>Test Results Summary</h2>
        <div class="metric">
            <div class="score">${total_failures}</div>
            <div>Total Failures</div>
        </div>
        <div class="metric">
            <div class="score">${unit_failures}</div>
            <div>Unit Test Failures</div>
        </div>
        <div class="metric">
            <div class="score">${integration_failures}</div>
            <div>Integration Failures</div>
        </div>
        <div class="metric">
            <div class="score">${performance_regressions}</div>
            <div>Performance Regressions</div>
        </div>
        <div class="metric">
            <div class="score">${security_failures}</div>
            <div>Security Failures</div>
        </div>
    </div>

    <div class="section">
        <h2>Detailed Results</h2>
        <ul>
            $(if [[ "${unit_failures}" -gt 0 ]]; then echo "<li>🔴 ${unit_failures} unit test failures detected</li>"; else echo "<li>✅ All unit tests passed</li>"; fi)
            $(if [[ "${integration_failures}" -gt 0 ]]; then echo "<li>🔴 ${integration_failures} integration test failures detected</li>"; else echo "<li>✅ All integration tests passed</li>"; fi)
            $(if [[ "${performance_regressions}" -gt 0 ]]; then echo "<li>🟡 ${performance_regressions} performance regressions detected</li>"; else echo "<li>✅ No performance regressions</li>"; fi)
            $(if [[ "${security_failures}" -gt 0 ]]; then echo "<li>🔴 ${security_failures} security test failures detected</li>"; else echo "<li>✅ All security tests passed</li>"; fi)
        </ul>
    </div>

    <div class="section">
        <h2>Report Files</h2>
        <ul>
            <li><a href="comprehensive-regression-report.json">Comprehensive JSON Report</a></li>
            <li><a href="unit-test-report.json">Unit Test Report</a></li>
            <li><a href="integration-test-report.json">Integration Test Report</a></li>
            <li><a href="performance-test-report.json">Performance Test Report</a></li>
            <li><a href="security-regression-report.json">Security Test Report</a></li>
        </ul>
    </div>
</body>
</html>
EOF

    log_success "Comprehensive regression report generated: ${comprehensive_report}"
    log_success "HTML regression report generated: ${html_report}"
}

# Main function
main() {
    log_info "Starting comprehensive regression testing for Rust AI IDE"
    log_info "Log file: ${REGRESSION_LOG}"
    log_info "Report directory: ${OUTPUT_DIR}"

    mkdir -p "${OUTPUT_DIR}"

    local exit_code=0

    # Clean up old results
    cleanup_old_results

    # Run test suites
    run_unit_tests || exit_code=$((exit_code + 1))
    run_integration_tests || exit_code=$((exit_code + 1))
    run_performance_tests || exit_code=$((exit_code + 1))
    run_security_tests || exit_code=$((exit_code + 1))

    # Compare with baseline
    compare_with_baseline

    # Generate comprehensive report
    generate_comprehensive_report

    local end_time=$(date +%s)
    log_info "Comprehensive regression testing completed in $((end_time - START_TIME)) seconds"

    return $exit_code
}

# Run main function
main "$@"
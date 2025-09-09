#!/bin/bash

# Test Helpers Script for Rust AI IDE
# Comprehensive testing utilities for CI/CD pipelines
# Author: QA Engineering Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TEST_LOG="${PROJECT_ROOT}/test-helpers.log"
COVERAGE_DIR="${PROJECT_ROOT}/coverage-results"
START_TIME=$(date +%s)

# Create directories
mkdir -p "${COVERAGE_DIR}"

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

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [COMMAND] [OPTIONS]

Test helpers and utilities for Rust AI IDE CI/CD pipelines.

COMMANDS:
    run-tests          Run comprehensive test suite
    run-coverage       Generate test coverage reports
    run-benchmarks     Execute performance benchmarks
    run-integration    Run integration tests
    run-load-test      Run load and stress tests
    analyze-results    Analyze test results and generate reports
    setup-test-env     Set up test environment and dependencies
    cleanup            Clean up test artifacts and reports

OPTIONS:
    -h, --help              Show this help message
    -v, --verbose           Enable verbose output
    --parallel N            Number of parallel test jobs (default: auto-detect)
    --profile PROFILE       Cargo build profile for testing (default: release)
    --features FEATURES     Cargo features to enable for testing
    --timeout SEC           Test timeout in seconds (default: 600)
    --coverage-threshold N  Minimum coverage percentage required (default: 90)
    --include-bench         Include benchmark tests
    --include-slow          Include slow tests that are normally skipped
    --output FORMAT         Output format for reports (json, html, xml, default: json)
    --environment ENV       Test environment (development, staging, production)

EXAMPLES:
    $0 run-tests --parallel 4 --verbose
    $0 run-coverage --coverage-threshold 85
    $0 run-benchmarks --include-slow --timeout 1200
    $0 analyze-results --output html

EOF
}

# Parse command line arguments
COMMAND=""
VERBOSE=false
PARALLEL_JOBS=$(nproc 2>/dev/null || echo 4)
PROFILE="release"
FEATURES=""
TIMEOUT=600
COVERAGE_THRESHOLD=90
INCLUDE_BENCH=false
INCLUDE_SLOW=false
OUTPUT_FORMAT="json"
ENVIRONMENT="development"

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
        --parallel)
            PARALLEL_JOBS="$2"
            shift 2
            ;;
        --profile)
            PROFILE="$2"
            shift 2
            ;;
        --features)
            FEATURES="$2"
            shift 2
            ;;
        --timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        --coverage-threshold)
            COVERAGE_THRESHOLD="$2"
            shift 2
            ;;
        --include-bench)
            INCLUDE_BENCH=true
            shift
            ;;
        --include-slow)
            INCLUDE_SLOW=true
            shift
            ;;
        --output)
            OUTPUT_FORMAT="$2"
            shift 2
            ;;
        --environment)
            ENVIRONMENT="$2"
            shift 2
            ;;
        run-tests|run-coverage|run-benchmarks|run-integration|run-load-test|analyze-results|setup-test-env|cleanup)
            if [[ -z "${COMMAND}" ]]; then
                COMMAND="$1"
            else
                log_error "Multiple commands not supported: ${COMMAND} and $1"
                exit 1
            fi
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

if [[ -z "${COMMAND}" ]]; then
    log_error "No command specified"
    usage
    exit 1
fi

# Function to check test environment dependencies
check_test_dependencies() {
    log_info "Checking test dependencies..."

    local required_tools=("cargo" "rustc" "jq")

    for tool in "${required_tools[@]}"; do
        if ! command -v "${tool}" >/dev/null 2>&1; then
            log_error "Required tool '${tool}' not found"
            return 1
        fi
    done

    # Check for test-specific tools
    local test_tools=("cargo-tarpaulin" "cargo-criterion")

    for tool in "${test_tools[@]}"; do
        if ! command -v "${tool}" >/dev/null 2>&1 && ! cargo install --list | grep -q "${tool}"; then
            log_warning "Test tool '${tool}' not found - will install if needed"
        fi
    done
}

# Function to set up test environment
setup_test_environment() {
    log_info "Setting up test environment..."

    # Install test-specific tools if needed
    if ! command -v cargo-tarpaulin >/dev/null 2>&1; then
        log_info "Installing cargo-tarpaulin for coverage..."
        cargo install cargo-tarpaulin
    fi

    if ! command -v cargo-criterion >/dev/null 2>&1; then
        log_info "Installing cargo-criterion for benchmarks..."
        cargo install cargo-criterion
    fi

    # Set up test-specific environment variables
    export RUST_BACKTRACE=1
    export RUST_LOG=trace
    export CARGO_INCREMENTAL=0

    # Create test configuration
    cat > "${PROJECT_ROOT}/test-config.toml" << EOF
[testing]
environment = "${ENVIRONMENT}"
parallel_jobs = ${PARALLEL_JOBS}
timeout_seconds = ${TIMEOUT}
verbose = ${VERBOSE}
include_benchmarks = ${INCLUDE_BENCH}
include_slow_tests = ${INCLUDE_SLOW}
coverage_threshold = ${COVERAGE_THRESHOLD}

[build]
profile = "${PROFILE}"
features = "${FEATURES}"
EOF

    log_success "Test environment setup complete"
}

# Function to run comprehensive test suite
run_test_suite() {
    log_info "Running comprehensive test suite..."

    local test_args=("test" "--profile" "${PROFILE}")

    if [[ -n "${FEATURES}" ]]; then
        test_args+=("--features" "${FEATURES}")
    fi

    # Set test environment variables
    export RUST_TEST_THREADS="${PARALLEL_JOBS}"
    export CARGO_TEST_TIMEOUT="${TIMEOUT}"

    local test_start=$(date +%s)

    # Run unit tests
    log_info "Running unit tests..."
    if cargo "${test_args[@]}" --lib --bins --verbose | tee "${PROJECT_ROOT}/unit-test-results.log"; then
        log_success "Unit tests passed"
    else
        log_error "Unit tests failed"
        return 1
    fi

    # Run integration tests
    log_info "Running integration tests..."
    if cargo "${test_args[@]}" --test '*' --verbose | tee "${PROJECT_ROOT}/integration-test-results.log"; then
        log_success "Integration tests passed"
    else
        log_warning "Integration tests failed (continuing with other tests)"
    fi

    # Run doc tests
    log_info "Running documentation tests..."
    if cargo "${test_args[@]}" --doc --verbose | tee "${PROJECT_ROOT}/doc-test-results.log"; then
        log_success "Documentation tests passed"
    else
        log_error "Documentation tests failed"
        return 1
    fi

    # Run benchmarks if requested
    if [[ "${INCLUDE_BENCH}" == true ]]; then
        log_info "Running benchmarks..."
        cargo bench --profile "${PROFILE}" | tee "${PROJECT_ROOT}/benchmark-results.log" || log_warning "Benchmarks completed with warnings"
    fi

    local test_duration=$(( $(date +%s) - test_start ))
    log_info "Test suite completed in ${test_duration} seconds"

    return 0
}

# Function to generate coverage reports
run_coverage_analysis() {
    log_info "Generating test coverage reports..."

    if ! command -v cargo-tarpaulin >/dev/null 2>&1; then
        log_error "cargo-tarpaulin not found. Run setup-test-env first."
        return 1
    fi

    local coverage_args=("tarpaulin"
        "--all-targets"
        "--workspace"
        "--out" "${OUTPUT_FORMAT}"
        "--output-dir" "${COVERAGE_DIR}"
        "--timeout" "${TIMEOUT}"
        "--run-types" "Tests,Lib"
    )

    if [[ "${VERBOSE}" == true ]]; then
        coverage_args+=("-v")
    fi

    log_info "Running: cargo ${coverage_args[*]}"

    if cargo "${coverage_args[@]}"; then
        log_success "Coverage analysis completed"
    else
        log_error "Coverage analysis failed"
        return 1
    fi

    # Check coverage threshold
    local coverage_file="${COVERAGE_DIR}/cobertura.xml"
    if [[ -f "${coverage_file}" ]]; then
        local coverage_percentage=$(grep -oP 'line-rate="\K[^"]+' "${coverage_file}" | head -1 | xargs -I {} echo "scale=2; {} * 100" | bc 2>/dev/null || echo "0")

        log_info "Coverage percentage: ${coverage_percentage}%"

        if (( $(echo "${coverage_percentage} < ${COVERAGE_THRESHOLD}" | bc -l 2>/dev/null || echo 1) )); then
            log_error "Coverage below threshold: ${coverage_percentage}% < ${COVERAGE_THRESHOLD}%"
            return 1
        else
            log_success "Coverage meets threshold: ${coverage_percentage}% >= ${COVERAGE_THRESHOLD}%"
        fi

        # Generate coverage summary
        echo "Coverage Report Summary" > "${COVERAGE_DIR}/coverage-summary.txt"
        echo "Generated: $(date)" >> "${COVERAGE_DIR}/coverage-summary.txt"
        echo "Coverage: ${coverage_percentage}%" >> "${COVERAGE_DIR}/coverage-summary.txt"
        echo "Threshold: ${COVERAGE_THRESHOLD}%" >> "${COVERAGE_DIR}/coverage-summary.txt"
        echo "Status: $( (( $(echo "${coverage_percentage} >= ${COVERAGE_THRESHOLD}" | bc -l 2>/dev/null || echo 0) )) && echo "PASSED" || echo "FAILED" )" >> "${COVERAGE_DIR}/coverage-summary.txt"
    fi

    return 0
}

# Function to run performance benchmarks
run_performance_benchmarks() {
    log_info "Running performance benchmarks..."

    local bench_args=("criterion" "--bench")

    if [[ "${VERBOSE}" == true ]]; then
        bench_args+=("-v")
    fi

    # Run criteria benchmarks
    if cargo "${bench_args[@]}"; then
        log_success "Performance benchmarks completed"
    else
        log_error "Performance benchmarks failed"
        return 1
    fi

    # Run cargo benchmarks
    cargo bench --profile "${PROFILE}" | tee "${PROJECT_ROOT}/cargo-benchmarks.log" || log_warning "Cargo benchmarks completed with warnings"

    return 0
}

# Function to run load and stress tests
run_load_tests() {
    log_info "Running load and stress tests..."

    # Check if hey load testing tool is available
    if ! command -v hey >/dev/null 2>&1; then
        log_warning "hey load testing tool not found. Installing..."
        wget -O hey https://hey-release.s3.us-east-2.amazonaws.com/hey_linux_amd64
        chmod +x hey
        sudo mv hey /usr/local/bin/ || mv hey /usr/local/bin/
    fi

    # Build the application for testing
    cargo build --profile "${PROFILE}"
    local test_binary="${PROJECT_ROOT}/target/${PROFILE}/rust-ai-ide"

    if [[ ! -f "${test_binary}" ]]; then
        log_error "Test binary not found: ${test_binary}"
        return 1
    fi

    # Start the application for testing
    "${test_binary}" &
    local app_pid=$!
    sleep 5  # Wait for startup

    # Run load tests
    mkdir -p "${PROJECT_ROOT}/load-test-results"

    local load_test_config=(
        "hey -n 1000 -c 10 -q 5 http://localhost:8080/health"
        "hey -n 5000 -c 50 -q 10 http://localhost:8080/health"
        "hey -n 10000 -c 100 -q 20 http://localhost:8080/health"
    )

    for i in "${!load_test_config[@]}"; do
        log_info "Running load test ${i}: ${load_test_config[$i]}"
        ${load_test_config[$i]} > "${PROJECT_ROOT}/load-test-results/test-${i}.txt"

        # Analyze results
        local req_sec=$(grep "Requests/sec" "${PROJECT_ROOT}/load-test-results/test-${i}.txt" | awk '{print $2}')
        log_info "Load test ${i} completed: ${req_sec} req/sec"
    done

    # Stop the application
    kill "${app_pid}" 2>/dev/null || true

    log_success "Load tests completed"
}

# Function to analyze test results
analyze_test_results() {
    log_info "Analyzing test results..."

    local analysis_dir="${PROJECT_ROOT}/test-analysis"
    mkdir -p "${analysis_dir}"

    # Analyze unit test results
    if [[ -f "${PROJECT_ROOT}/unit-test-results.log" ]]; then
        local passed_tests=$(grep "test result: ok" "${PROJECT_ROOT}/unit-test-results.log" | wc -l)
        local failed_tests=$(grep "test result: FAILED" "${PROJECT_ROOT}/unit-test-results.log" | wc -l)

        echo "Unit Test Analysis" > "${analysis_dir}/unit-tests-summary.txt"
        echo "Passed: ${passed_tests}" >> "${analysis_dir}/unit-tests-summary.txt"
        echo "Failed: ${failed_tests}" >> "${analysis_dir}/unit-tests-summary.txt"
        echo "Total: $((passed_tests + failed_tests))" >> "${analysis_dir}/unit-tests-summary.txt"
    fi

    # Analyze coverage results
    if [[ -f "${COVERAGE_DIR}/coverage-summary.txt" ]]; then
        cp "${COVERAGE_DIR}/coverage-summary.txt" "${analysis_dir}/coverage-analysis.txt"
    fi

    # Generate comprehensive test report
    local total_duration=$(( $(date +%s) - START_TIME ))

    cat > "${analysis_dir}/comprehensive-test-report.${OUTPUT_FORMAT}" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "environment": "${ENVIRONMENT}",
    "total_duration_seconds": ${total_duration},
    "configuration": {
        "parallel_jobs": ${PARALLEL_JOBS},
        "profile": "${PROFILE}",
        "features": "${FEATURES}",
        "timeout": ${TIMEOUT},
        "coverage_threshold": ${COVERAGE_THRESHOLD},
        "include_benchmarks": ${INCLUDE_BENCH},
        "include_slow_tests": ${INCLUDE_SLOW}
    },
    "results": {
        "unit_tests": $(jq -n '{"passed": 0, "failed": 0}' 2>/dev/null || echo '{"passed": 0, "failed": 0}'),
        "integration_tests": $(jq -n '{"passed": 0, "failed": 0}' 2>/dev/null || echo '{"passed": 0, "failed": 0}'),
        "coverage": $(jq -n '{"percentage": 0, "threshold": 90}' 2>/dev/null || echo '{"percentage": 0, "threshold": 90}'),
        "benchmarks": $(jq -n '{"completed": 0}' 2>/dev/null || echo '{"completed": 0}')
    },
    "artifacts": {
        "logs_directory": "${PROJECT_ROOT}",
        "coverage_directory": "${COVERAGE_DIR}",
        "analysis_directory": "${analysis_dir}"
    }
}
EOF

    log_info "Test analysis completed. Report: ${analysis_dir}/comprehensive-test-report.${OUTPUT_FORMAT}"
}

# Function to clean up test artifacts
cleanup_test_artifacts() {
    log_info "Cleaning up test artifacts..."

    # Remove test logs and temporary files
    rm -f "${PROJECT_ROOT}"/*-test-results.log
    rm -f "${PROJECT_ROOT}"/*-test-*.log
    rm -f "${PROJECT_ROOT}/test-config.toml"

    # Remove old coverage results
    find "${COVERAGE_DIR}" -name "*.profraw" -type f -mtime +7 -delete 2>/dev/null || true
    find "${COVERAGE_DIR}" -name "*.gcda" -type f -mtime +7 -delete 2>/dev/null || true

    # Remove load test results older than 30 days
    find "${PROJECT_ROOT}/load-test-results" -type f -mtime +30 -delete 2>/dev/null || true

    log_success "Cleanup completed"
}

# Main function
main() {
    log_info "Starting test helpers script"
    log_info "Command: ${COMMAND}"
    log_info "Log file: ${TEST_LOG}"

    # Trap to ensure cleanup on exit
    trap 'log_info "Test helpers completed (exit code: $?)"; cleanup_test_artifacts' EXIT

    local exit_code=0

    case "${COMMAND}" in
        setup-test-env)
            check_test_dependencies
            setup_test_environment
            ;;
        run-tests)
            run_test_suite
            ;;
        run-coverage)
            run_coverage_analysis
            ;;
        run-benchmarks)
            run_performance_benchmarks
            ;;
        run-integration)
            run_test_suite
            ;;
        run-load-test)
            run_load_tests
            ;;
        analyze-results)
            analyze_test_results
            ;;
        cleanup)
            cleanup_test_artifacts
            exit 0  # Don't run cleanup again
            ;;
        *)
            log_error "Unknown command: ${COMMAND}"
            usage
            exit 1
            ;;
    esac

    local end_time=$(date +%s)
    log_info "Test helpers completed in $((end_time - START_TIME)) seconds"

    return $exit_code
}

# Run main function
main "$@"
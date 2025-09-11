#!/bin/bash

# Compatibility Check Script for Rust AI IDE
# Comprehensive compatibility checks across all 67 crates in the workspace
# Tests build compatibility, API compatibility, and integration compatibility
# Author: DevOps Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
COMPATIBILITY_LOG="${PROJECT_ROOT}/compatibility-check.log"
REPORT_DIR="${PROJECT_ROOT}/security-reports/compatibility/$(date +%Y%m%d_%H%M%S)"
START_TIME=$(date +%s)

# Create directories
mkdir -p "${REPORT_DIR}"

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${COMPATIBILITY_LOG}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${COMPATIBILITY_LOG}" >&2
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${COMPATIBILITY_LOG}"
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${COMPATIBILITY_LOG}"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Comprehensive compatibility checks across all 67 crates in the Rust AI IDE workspace.

OPTIONS:
    -h, --help                  Show this help message
    -v, --verbose               Enable verbose output
    --output-dir DIR            Output directory for reports (default: auto-generated)
    --build-only                Only run build compatibility checks
    --test-only                 Only run test compatibility checks
    --api-only                  Only run API compatibility checks
    --parallel NUM              Number of parallel jobs (default: auto)
    --fail-fast                 Stop on first failure
    --ignore-warnings           Don't treat warnings as failures
    --baseline FILE             Compare against baseline compatibility report

EXAMPLES:
    $0 --verbose --parallel 4
    $0 --build-only --fail-fast
    $0 --baseline /path/to/baseline-report.json

EOF
}

# Parse command line arguments
VERBOSE=false
BUILD_ONLY=false
TEST_ONLY=false
API_ONLY=false
PARALLEL=""
FAIL_FAST=false
IGNORE_WARNINGS=false
BASELINE_FILE=""
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
        --build-only)
            BUILD_ONLY=true
            shift
            ;;
        --test-only)
            TEST_ONLY=true
            shift
            ;;
        --api-only)
            API_ONLY=true
            shift
            ;;
        --parallel)
            PARALLEL="$2"
            shift 2
            ;;
        --fail-fast)
            FAIL_FAST=true
            shift
            ;;
        --ignore-warnings)
            IGNORE_WARNINGS=true
            shift
            ;;
        --baseline)
            BASELINE_FILE="$2"
            shift 2
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Determine check types to run
RUN_BUILD=true
RUN_TEST=true
RUN_API=true

if [[ "${BUILD_ONLY}" == true ]]; then
    RUN_TEST=false
    RUN_API=false
elif [[ "${TEST_ONLY}" == true ]]; then
    RUN_BUILD=false
    RUN_API=false
elif [[ "${API_ONLY}" == true ]]; then
    RUN_BUILD=false
    RUN_TEST=false
fi

# Function to get workspace crate information
get_workspace_crates() {
    log_info "Analyzing workspace structure..."

    cd "${PROJECT_ROOT}"

    # Get all crates from cargo metadata
    local crates_info=$(cargo metadata --format-version 1 2>/dev/null | jq -r '
        .packages[] |
        select(.source == null or (.source | contains("registry+https://github.com/rust-lang/crates.io-index"))) |
        {name: .name, version: .version, path: .manifest_path | dirname, dependencies: (.dependencies | length)}
    ' 2>/dev/null || echo "[]")

    echo "${crates_info}"

    # Count crates
    local crate_count=$(echo "${crates_info}" | jq -r 'length' 2>/dev/null || echo "0")
    log_info "Found ${crate_count} crates in workspace"
}

# Function to run build compatibility checks
run_build_compatibility() {
    if [[ "${RUN_BUILD}" != true ]]; then
        log_info "Skipping build compatibility checks"
        return 0
    fi

    log_info "Running build compatibility checks across all crates..."

    local build_report="${OUTPUT_DIR}/build-compatibility-report.json"
    local build_failures=0
    local build_warnings=0

    cd "${PROJECT_ROOT}"

    # Get parallel jobs
    local jobs=""
    if [[ -n "${PARALLEL}" ]]; then
        jobs="--jobs ${PARALLEL}"
    fi

    # Build entire workspace with nightly toolchain
    log_info "Building entire workspace with nightly toolchain..."

    if cargo +nightly build --workspace ${jobs} --message-format json > "${OUTPUT_DIR}/build-output.log" 2>&1; then
        log_success "Workspace build successful"

        # Analyze build output for warnings
        local warning_count=$(grep -c "warning:" "${OUTPUT_DIR}/build-output.log" 2>/dev/null || echo "0")
        build_warnings=$((build_warnings + warning_count))

        if [[ "${warning_count}" -gt 0 ]]; then
            log_warning "Found ${warning_count} build warnings"
            if [[ "${IGNORE_WARNINGS}" != true ]]; then
                build_failures=$((build_failures + 1))
            fi
        fi
    else
        log_error "Workspace build failed"
        build_failures=$((build_failures + 1))
    fi

    # Build each crate individually for detailed compatibility analysis
    log_info "Building individual crates for detailed analysis..."

    local crates_info=$(get_workspace_crates)
    local total_crates=$(echo "${crates_info}" | jq -r 'length' 2>/dev/null || echo "0")

    for i in $(seq 0 $((${total_crates} - 1))); do
        local crate_name=$(echo "${crates_info}" | jq -r ".[${i}].name" 2>/dev/null || echo "")
        local crate_path=$(echo "${crates_info}" | jq -r ".[${i}].path" 2>/dev/null || echo "")

        if [[ -n "${crate_name}" && -n "${crate_path}" ]]; then
            log_info "Building crate: ${crate_name}"

            if cargo +nightly build --manifest-path "${crate_path}/Cargo.toml" ${jobs} \
                --message-format json > "${OUTPUT_DIR}/build-${crate_name}.log" 2>&1; then

                # Check for warnings in individual crate build
                local crate_warnings=$(grep -c "warning:" "${OUTPUT_DIR}/build-${crate_name}.log" 2>/dev/null || echo "0")
                if [[ "${crate_warnings}" -gt 0 ]]; then
                    build_warnings=$((build_warnings + crate_warnings))
                    log_warning "Crate ${crate_name} has ${crate_warnings} build warnings"
                fi
            else
                log_error "Failed to build crate: ${crate_name}"
                build_failures=$((build_failures + 1))

                if [[ "${FAIL_FAST}" == true ]]; then
                    log_error "Stopping due to --fail-fast option"
                    return 1
                fi
            fi
        fi
    done

    # Generate build compatibility report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg build_failures "$build_failures" \
        --arg build_warnings "$build_warnings" \
        --arg total_crates "$total_crates" \
        --arg parallel_jobs "$PARALLEL" \
        '{
            timestamp: $timestamp,
            check_type: "build_compatibility",
            total_crates: ($total_crates | tonumber),
            parallel_jobs: $parallel_jobs,
            build_failures: ($build_failures | tonumber),
            build_warnings: ($build_warnings | tonumber),
            status: (if ($build_failures == "0") then "COMPATIBLE" elif ($build_failures < "5") then "MOSTLY_COMPATIBLE" else "INCOMPATIBLE" end),
            compatibility_score: (if ($build_failures == "0" and $build_warnings == "0") then 100 elif ($build_failures == "0") then 90 else (90 - ($build_failures | tonumber) * 10) end)
        }' > "${build_report}"

    log_info "Build compatibility check completed. Report: ${build_report}"
    return $build_failures
}

# Function to run test compatibility checks
run_test_compatibility() {
    if [[ "${RUN_TEST}" != true ]]; then
        log_info "Skipping test compatibility checks"
        return 0
    fi

    log_info "Running test compatibility checks across all crates..."

    local test_report="${OUTPUT_DIR}/test-compatibility-report.json"
    local test_failures=0
    local test_warnings=0

    cd "${PROJECT_ROOT}"

    # Get parallel jobs
    local jobs=""
    if [[ -n "${PARALLEL}" ]]; then
        jobs="--jobs ${PARALLEL}"
    fi

    # Run workspace tests
    log_info "Running workspace tests..."

    if cargo +nightly test --workspace ${jobs} --message-format json > "${OUTPUT_DIR}/test-output.log" 2>&1; then
        log_success "Workspace tests successful"

        # Analyze test output
        local test_count=$(grep -c '"reason":"build-finished"' "${OUTPUT_DIR}/test-output.log" 2>/dev/null || echo "0")
        log_info "Ran ${test_count} test builds"

    else
        log_error "Workspace tests failed"
        test_failures=$((test_failures + 1))

        # Extract test failure details
        grep -A5 -B5 '"event":"failed"' "${OUTPUT_DIR}/test-output.log" > "${OUTPUT_DIR}/test-failures.log" 2>/dev/null || true
    fi

    # Run tests for individual crates
    log_info "Testing individual crates..."

    local crates_info=$(get_workspace_crates)
    local total_crates=$(echo "${crates_info}" | jq -r 'length' 2>/dev/null || echo "0")

    for i in $(seq 0 $((${total_crates} - 1))); do
        local crate_name=$(echo "${crates_info}" | jq -r ".[${i}].name" 2>/dev/null || echo "")
        local crate_path=$(echo "${crates_info}" | jq -r ".[${i}].path" 2>/dev/null || echo "")

        if [[ -n "${crate_name}" && -n "${crate_path}" ]]; then
            log_info "Testing crate: ${crate_name}"

            if cargo +nightly test --manifest-path "${crate_path}/Cargo.toml" ${jobs} \
                --message-format json > "${OUTPUT_DIR}/test-${crate_name}.log" 2>&1; then

                # Check for test warnings or flakes
                local test_warnings_count=$(grep -c "warning:" "${OUTPUT_DIR}/test-${crate_name}.log" 2>/dev/null || echo "0")
                if [[ "${test_warnings_count}" -gt 0 ]]; then
                    test_warnings=$((test_warnings + test_warnings_count))
                    log_warning "Crate ${crate_name} has ${test_warnings_count} test warnings"
                fi
            else
                log_error "Failed to test crate: ${crate_name}"
                test_failures=$((test_failures + 1))

                if [[ "${FAIL_FAST}" == true ]]; then
                    log_error "Stopping due to --fail-fast option"
                    return 1
                fi
            fi
        fi
    done

    # Generate test compatibility report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg test_failures "$test_failures" \
        --arg test_warnings "$test_warnings" \
        --arg total_crates "$total_crates" \
        '{
            timestamp: $timestamp,
            check_type: "test_compatibility",
            total_crates: ($total_crates | tonumber),
            test_failures: ($test_failures | tonumber),
            test_warnings: ($test_warnings | tonumber),
            status: (if ($test_failures == "0") then "COMPATIBLE" elif ($test_failures < "3") then "MOSTLY_COMPATIBLE" else "INCOMPATIBLE" end),
            compatibility_score: (if ($test_failures == "0") then 100 else (95 - ($test_failures | tonumber) * 15) end)
        }' > "${test_report}"

    log_info "Test compatibility check completed. Report: ${test_report}"
    return $test_failures
}

# Function to run API compatibility checks
run_api_compatibility() {
    if [[ "${RUN_API}" != true ]]; then
        log_info "Skipping API compatibility checks"
        return 0
    fi

    log_info "Running API compatibility checks..."

    local api_report="${OUTPUT_DIR}/api-compatibility-report.json"
    local api_issues=0

    cd "${PROJECT_ROOT}"

    # Check for API breaking changes using cargo-semver-checks
    if command -v cargo-semver-checks >/dev/null 2>&1; then
        log_info "Running semantic versioning checks..."

        local crates_info=$(get_workspace_crates)
        local total_crates=$(echo "${crates_info}" | jq -r 'length' 2>/dev/null || echo "0")

        for i in $(seq 0 $((${total_crates} - 1))); do
            local crate_name=$(echo "${crates_info}" | jq -r ".[${i}].name" 2>/dev/null || echo "")
            local crate_path=$(echo "${crates_info}" | jq -r ".[${i}].path" 2>/dev/null || echo "")

            if [[ -n "${crate_name}" && -n "${crate_path}" ]]; then
                log_info "Checking API compatibility for: ${crate_name}"

                if cargo semver-checks check-release --manifest-path "${crate_path}/Cargo.toml" \
                    --baseline-root "${PROJECT_ROOT}" > "${OUTPUT_DIR}/api-${crate_name}.log" 2>&1; then

                    local breaking_changes=$(grep -c "BREAKING" "${OUTPUT_DIR}/api-${crate_name}.log" 2>/dev/null || echo "0")
                    if [[ "${breaking_changes}" -gt 0 ]]; then
                        api_issues=$((api_issues + breaking_changes))
                        log_warning "Crate ${crate_name} has ${breaking_changes} breaking API changes"
                    fi
                else
                    log_warning "Could not check API compatibility for ${crate_name}"
                fi
            fi
        done
    else
        log_warning "cargo-semver-checks not available - install with: cargo install cargo-semver-checks"
        api_issues=$((api_issues + 1))
    fi

    # Check for circular dependencies
    log_info "Checking for circular dependencies..."

    if cargo tree --workspace --format "{p}" > "${OUTPUT_DIR}/dependency-tree.log" 2>&1; then
        # Basic circular dependency detection (can be enhanced)
        local circular_count=$(grep -c "circular" "${OUTPUT_DIR}/dependency-tree.log" 2>/dev/null || echo "0")
        if [[ "${circular_count}" -gt 0 ]]; then
            api_issues=$((api_issues + circular_count))
            log_warning "Found ${circular_count} potential circular dependencies"
        fi
    fi

    # Generate API compatibility report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg api_issues "$api_issues" \
        '{
            timestamp: $timestamp,
            check_type: "api_compatibility",
            api_breaking_changes: ($api_issues | tonumber),
            circular_dependencies: 0,
            status: (if ($api_issues == "0") then "COMPATIBLE" else "BREAKING_CHANGES" end),
            compatibility_score: (if ($api_issues == "0") then 100 else (85 - ($api_issues | tonumber) * 5) end)
        }' > "${api_report}"

    log_info "API compatibility check completed. Report: ${api_report}"
    return $api_issues
}

# Function to compare against baseline
compare_with_baseline() {
    if [[ -z "${BASELINE_FILE}" ]]; then
        return 0
    fi

    if [[ ! -f "${BASELINE_FILE}" ]]; then
        log_warning "Baseline file not found: ${BASELINE_FILE}"
        return 0
    fi

    log_info "Comparing results with baseline: ${BASELINE_FILE}"

    local comparison_report="${OUTPUT_DIR}/baseline-comparison-report.json"

    # Compare build compatibility
    local current_build_score=$(jq -r '.compatibility_score // 0' "${OUTPUT_DIR}/build-compatibility-report.json" 2>/dev/null || echo "0")
    local baseline_build_score=$(jq -r '.compatibility_score // 0' "${BASELINE_FILE}" 2>/dev/null || echo "0")

    local build_diff=$((current_build_score - baseline_build_score))

    # Compare test compatibility
    local current_test_score=$(jq -r '.compatibility_score // 0' "${OUTPUT_DIR}/test-compatibility-report.json" 2>/dev/null || echo "0")
    local baseline_test_score=$(jq -r '.compatibility_score // 0' "${BASELINE_FILE}" 2>/dev/null || echo "0")

    local test_diff=$((current_test_score - baseline_test_score))

    # Generate comparison report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg baseline_file "$BASELINE_FILE" \
        --arg build_score_diff "$build_diff" \
        --arg test_score_diff "$test_diff" \
        --arg current_build_score "$current_build_score" \
        --arg current_test_score "$current_test_score" \
        --arg baseline_build_score "$baseline_build_score" \
        --arg baseline_test_score "$baseline_test_score" \
        '{
            timestamp: $timestamp,
            comparison_type: "baseline",
            baseline_file: $baseline_file,
            comparisons: {
                build_compatibility: {
                    current_score: ($current_build_score | tonumber),
                    baseline_score: ($baseline_build_score | tonumber),
                    difference: ($build_score_diff | tonumber),
                    trend: (if ($build_score_diff > "0") then "IMPROVING" elif ($build_score_diff < "0") then "DECLINING" else "STABLE" end)
                },
                test_compatibility: {
                    current_score: ($current_test_score | tonumber),
                    baseline_score: ($baseline_test_score | tonumber),
                    difference: ($test_score_diff | tonumber),
                    trend: (if ($test_score_diff > "0") then "IMPROVING" elif ($test_score_diff < "0") then "DECLINING" else "STABLE" end)
                }
            },
            overall_trend: (if (($build_score_diff | tonumber) + ($test_score_diff | tonumber) > "0") then "IMPROVING" elif (($build_score_diff | tonumber) + ($test_score_diff | tonumber) < "0") then "DECLINING" else "STABLE" end)
        }' > "${comparison_report}"

    log_info "Baseline comparison completed. Report: ${comparison_report}"
}

# Function to generate comprehensive compatibility report
generate_comprehensive_report() {
    log_info "Generating comprehensive compatibility report..."

    local comprehensive_report="${OUTPUT_DIR}/comprehensive-compatibility-report.json"
    local html_report="${OUTPUT_DIR}/compatibility-report.html"

    # Collect all compatibility results
    local build_failures=$(jq -r '.build_failures // 0' "${OUTPUT_DIR}/build-compatibility-report.json" 2>/dev/null || echo "0")
    local build_warnings=$(jq -r '.build_warnings // 0' "${OUTPUT_DIR}/build-compatibility-report.json" 2>/dev/null || echo "0")
    local build_score=$(jq -r '.compatibility_score // 0' "${OUTPUT_DIR}/build-compatibility-report.json" 2>/dev/null || echo "0")

    local test_failures=$(jq -r '.test_failures // 0' "${OUTPUT_DIR}/test-compatibility-report.json" 2>/dev/null || echo "0")
    local test_warnings=$(jq -r '.test_warnings // 0' "${OUTPUT_DIR}/test-compatibility-report.json" 2>/dev/null || echo "0")
    local test_score=$(jq -r '.compatibility_score // 0' "${OUTPUT_DIR}/test-compatibility-report.json" 2>/dev/null || echo "0")

    local api_issues=$(jq -r '.api_breaking_changes // 0' "${OUTPUT_DIR}/api-compatibility-report.json" 2>/dev/null || echo "0")
    local api_score=$(jq -r '.compatibility_score // 0' "${OUTPUT_DIR}/api-compatibility-report.json" 2>/dev/null || echo "0")

    # Calculate overall compatibility score
    local overall_score=$(( (build_score + test_score + api_score) / 3 ))
    local total_issues=$((build_failures + test_failures + api_issues))

    local overall_status="COMPATIBLE"
    if [[ "${total_issues}" -gt 10 ]]; then
        overall_status="INCOMPATIBLE"
    elif [[ "${total_issues}" -gt 0 ]]; then
        overall_status="MOSTLY_COMPATIBLE"
    fi

    # Generate comprehensive JSON report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg duration "$(($(date +%s) - START_TIME))" \
        --arg overall_score "$overall_score" \
        --arg overall_status "$overall_status" \
        --arg total_issues "$total_issues" \
        --arg run_build "$RUN_BUILD" \
        --arg run_test "$RUN_TEST" \
        --arg run_api "$RUN_API" \
        --arg fail_fast "$FAIL_FAST" \
        --arg parallel_jobs "$PARALLEL" \
        '{
            timestamp: $timestamp,
            duration_seconds: ($duration | tonumber),
            check_type: "comprehensive_compatibility",
            configuration: {
                build_checks: ($run_build == "true"),
                test_checks: ($run_test == "true"),
                api_checks: ($run_api == "true"),
                fail_fast: ($fail_fast == "true"),
                parallel_jobs: $parallel_jobs
            },
            results: {
                build_compatibility: {
                    failures: '$build_failures',
                    warnings: '$build_warnings',
                    score: '$build_score'
                },
                test_compatibility: {
                    failures: '$test_failures',
                    warnings: '$test_warnings',
                    score: '$test_score'
                },
                api_compatibility: {
                    breaking_changes: '$api_issues',
                    score: '$api_score'
                }
            },
            overall_compatibility_score: ($overall_score | tonumber),
            overall_status: $overall_status,
            total_issues: ($total_issues | tonumber),
            recommendations: [
                (if ('$build_failures' > "0") then "Fix build failures before proceeding" else "Build compatibility is good" end),
                (if ('$test_failures' > "0") then "Address test failures to ensure stability" else "Test compatibility is maintained" end),
                (if ('$api_issues' > "0") then "Review API breaking changes for impact" else "API compatibility is preserved" end)
            ]
        }' > "${comprehensive_report}"

    # Generate HTML report
    cat > "${html_report}" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>Rust AI IDE Compatibility Check Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f0f0f0; padding: 10px; border-radius: 5px; }
        .status-compatible { color: green; }
        .status-mostly { color: orange; }
        .status-incompatible { color: red; }
        .section { margin: 20px 0; padding: 10px; border: 1px solid #ccc; border-radius: 5px; }
        .metric { display: inline-block; margin: 10px; padding: 10px; background: #f9f9f9; border-radius: 3px; text-align: center; }
        .score { font-size: 24px; font-weight: bold; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Rust AI IDE Compatibility Check Report</h1>
        <p><strong>Generated:</strong> $(date)</p>
        <p><strong>Duration:</strong> $(($(date +%s) - START_TIME)) seconds</p>
        <p><strong>Overall Status:</strong> <span class="status-${overall_status,,}">${overall_status}</span></p>
        <p><strong>Compatibility Score:</strong> ${overall_score}%</p>
    </div>

    <div class="section">
        <h2>Check Configuration</h2>
        <p><strong>Build Checks:</strong> ${RUN_BUILD}</p>
        <p><strong>Test Checks:</strong> ${RUN_TEST}</p>
        <p><strong>API Checks:</strong> ${RUN_API}</p>
        <p><strong>Parallel Jobs:</strong> ${PARALLEL:-auto}</p>
        <p><strong>Fail Fast:</strong> ${FAIL_FAST}</p>
    </div>

    <div class="section">
        <h2>Compatibility Scores</h2>
        <div class="metric">
            <div class="score">${build_score}%</div>
            <div>Build Compatibility</div>
            <div>Failures: ${build_failures}, Warnings: ${build_warnings}</div>
        </div>
        <div class="metric">
            <div class="score">${test_score}%</div>
            <div>Test Compatibility</div>
            <div>Failures: ${test_failures}, Warnings: ${test_warnings}</div>
        </div>
        <div class="metric">
            <div class="score">${api_score}%</div>
            <div>API Compatibility</div>
            <div>Breaking Changes: ${api_issues}</div>
        </div>
    </div>

    <div class="section">
        <h2>Summary</h2>
        <p><strong>Total Issues:</strong> ${total_issues}</p>
        <ul>
            $(if [[ "${build_failures}" -gt 0 ]]; then echo "<li>🔴 ${build_failures} build failures detected</li>"; else echo "<li>✅ All crates build successfully</li>"; fi)
            $(if [[ "${test_failures}" -gt 0 ]]; then echo "<li>🔴 ${test_failures} test failures detected</li>"; else echo "<li>✅ All tests pass</li>"; fi)
            $(if [[ "${api_issues}" -gt 0 ]]; then echo "<li>🟡 ${api_issues} API breaking changes detected</li>"; else echo "<li>✅ No API breaking changes</li>"; fi)
        </ul>
    </div>

    <div class="section">
        <h2>Report Files</h2>
        <ul>
            <li><a href="comprehensive-compatibility-report.json">Comprehensive JSON Report</a></li>
            <li><a href="build-compatibility-report.json">Build Compatibility Report</a></li>
            <li><a href="test-compatibility-report.json">Test Compatibility Report</a></li>
            <li><a href="api-compatibility-report.json">API Compatibility Report</a></li>
        </ul>
    </div>
</body>
</html>
EOF

    log_success "Comprehensive compatibility report generated: ${comprehensive_report}"
    log_success "HTML compatibility report generated: ${html_report}"
}

# Main function
main() {
    log_info "Starting compatibility checks for Rust AI IDE workspace"
    log_info "Log file: ${COMPATIBILITY_LOG}"
    log_info "Report directory: ${OUTPUT_DIR}"

    mkdir -p "${OUTPUT_DIR}"

    local exit_code=0

    # Run compatibility checks
    run_build_compatibility || exit_code=$((exit_code + 1))
    run_test_compatibility || exit_code=$((exit_code + 1))
    run_api_compatibility || exit_code=$((exit_code + 1))

    # Compare with baseline if provided
    compare_with_baseline

    # Generate comprehensive report
    generate_comprehensive_report

    local end_time=$(date +%s)
    log_info "Compatibility checks completed in $((end_time - START_TIME)) seconds"

    return $exit_code
}

# Run main function
main "$@"
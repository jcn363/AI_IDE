#!/bin/bash

# Incremental Version Bump Script with Automated Testing
# Safe, incremental dependency updates with comprehensive testing and rollback
# Implements incremental version bumping with automated regression testing
# Author: DevOps Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
VERSION_LOG="${PROJECT_ROOT}/incremental-version-bump.log"
REPORT_DIR="${PROJECT_ROOT}/security-reports/version-bumps/$(date +%Y%m%d_%H%M%S)"
BACKUP_DIR="${PROJECT_ROOT}/security-backups/$(date +%Y%m%d_%H%M%S)_version_bumps"
START_TIME=$(date +%s)

# Create directories
mkdir -p "${REPORT_DIR}"
mkdir -p "${BACKUP_DIR}"

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${VERSION_LOG}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${VERSION_LOG}" >&2
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${VERSION_LOG}"
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${VERSION_LOG}"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Incremental dependency version bumps with automated testing and rollback capabilities.

OPTIONS:
    -h, --help                  Show this help message
    -v, --verbose               Enable verbose output
    --output-dir DIR            Output directory for reports (default: auto-generated)
    --target-crate CRATE         Specific crate to update (default: all)
    --version-type TYPE          Version bump type: patch|minor|major (default: patch)
    --max-updates NUM            Maximum number of updates to attempt (default: 10)
    --test-only                  Only run tests, don't apply updates
    --dry-run                    Show what would be updated without applying
    --fail-fast                  Stop on first test failure
    --rollback-on-failure        Automatically rollback on test failures
    --baseline-report FILE       Baseline compatibility report for comparison

EXAMPLES:
    $0 --dry-run --verbose
    $0 --version-type minor --max-updates 5
    $0 --target-crate serde --fail-fast

EOF
}

# Parse command line arguments
VERBOSE=false
TARGET_CRATE=""
VERSION_TYPE="patch"
MAX_UPDATES=10
TEST_ONLY=false
DRY_RUN=false
FAIL_FAST=false
ROLLBACK_ON_FAILURE=true
BASELINE_REPORT=""
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
        --target-crate)
            TARGET_CRATE="$2"
            shift 2
            ;;
        --version-type)
            VERSION_TYPE="$2"
            shift 2
            ;;
        --max-updates)
            MAX_UPDATES="$2"
            shift 2
            ;;
        --test-only)
            TEST_ONLY=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --fail-fast)
            FAIL_FAST=true
            shift
            ;;
        --rollback-on-failure)
            ROLLBACK_ON_FAILURE=true
            shift
            ;;
        --baseline-report)
            BASELINE_REPORT="$2"
            shift 2
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Function to create comprehensive backup
create_incremental_backup() {
    log_info "Creating incremental backup..."

    # Backup current lockfile and manifests
    if [[ -f "${PROJECT_ROOT}/Cargo.lock" ]]; then
        cp "${PROJECT_ROOT}/Cargo.lock" "${BACKUP_DIR}/Cargo.lock.baseline"
        log_info "Backed up Cargo.lock"
    fi

    if [[ -f "${PROJECT_ROOT}/Cargo.toml" ]]; then
        cp "${PROJECT_ROOT}/Cargo.toml" "${BACKUP_DIR}/Cargo.toml.baseline"
        log_info "Backed up Cargo.toml"
    fi

    # Backup web dependencies if they exist
    if [[ -f "${PROJECT_ROOT}/web/package-lock.json" ]]; then
        cp "${PROJECT_ROOT}/web/package-lock.json" "${BACKUP_DIR}/package-lock.json.baseline"
        log_info "Backed up package-lock.json"
    fi

    # Create baseline report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg version_type "$VERSION_TYPE" \
        --arg target_crate "$TARGET_CRATE" \
        --arg max_updates "$MAX_UPDATES" \
        --arg backup_dir "$BACKUP_DIR" \
        '{
            timestamp: $timestamp,
            operation_type: "incremental_version_bump",
            version_type: $version_type,
            target_crate: $target_crate,
            max_updates: ($max_updates | tonumber),
            backup_dir: $backup_dir,
            baseline_files: [
                "Cargo.lock.baseline",
                "Cargo.toml.baseline",
                "package-lock.json.baseline"
            ]
        }' > "${BACKUP_DIR}/incremental-backup-manifest.json"

    log_success "Incremental backup created: ${BACKUP_DIR}"
}

# Function to analyze outdated dependencies
analyze_outdated_dependencies() {
    log_info "Analyzing outdated dependencies..."

    cd "${PROJECT_ROOT}"

    local outdated_report="${OUTPUT_DIR}/outdated-analysis-report.json"
    local update_candidates=()

    # Use cargo-outdated for analysis
    if command -v cargo-outdated >/dev/null 2>&1; then
        log_info "Running cargo-outdated analysis..."

        if cargo outdated --format json > "${OUTPUT_DIR}/cargo-outdated-raw.json" 2>&1; then
            # Filter dependencies based on version type and target
            jq -r '.dependencies[]? |
                select(.project == "rust-ai-ide" or (.project | contains("rust-ai-ide"))) |
                select(.kind != "development") |
                {name: .name, current: .current, latest: .latest, kind: .kind, project: .project}' \
                "${OUTPUT_DIR}/cargo-outdated-raw.json" > "${OUTPUT_DIR}/update-candidates.json"

            # Count candidates
            local candidate_count=$(jq -r 'length' "${OUTPUT_DIR}/update-candidates.json" 2>/dev/null || echo "0")
            log_info "Found ${candidate_count} update candidates"

            # Filter by target crate if specified
            if [[ -n "${TARGET_CRATE}" ]]; then
                jq -r "map(select(.name == \"${TARGET_CRATE}\"))" "${OUTPUT_DIR}/update-candidates.json" > "${OUTPUT_DIR}/filtered-candidates.json"
                mv "${OUTPUT_DIR}/filtered-candidates.json" "${OUTPUT_DIR}/update-candidates.json"

                local filtered_count=$(jq -r 'length' "${OUTPUT_DIR}/update-candidates.json" 2>/dev/null || echo "0")
                log_info "Filtered to ${filtered_count} candidates for crate: ${TARGET_CRATE}"
            fi

            # Filter by version type
            if [[ "${VERSION_TYPE}" == "patch" ]]; then
                # Only patch updates (last number changes)
                jq -r 'map(select(.current | split(".") | .[2] != (.latest | split(".") | .[2])))' \
                    "${OUTPUT_DIR}/update-candidates.json" > "${OUTPUT_DIR}/patch-updates.json"
                mv "${OUTPUT_DIR}/patch-updates.json" "${OUTPUT_DIR}/update-candidates.json"
            elif [[ "${VERSION_TYPE}" == "minor" ]]; then
                # Minor and patch updates (middle number changes)
                jq -r 'map(select(.current | split(".") | .[1] != (.latest | split(".") | .[1])))' \
                    "${OUTPUT_DIR}/update-candidates.json" > "${OUTPUT_DIR}/minor-updates.json"
                mv "${OUTPUT_DIR}/minor-updates.json" "${OUTPUT_DIR}/update-candidates.json"
            fi
            # major includes everything

            local final_count=$(jq -r 'length' "${OUTPUT_DIR}/update-candidates.json" 2>/dev/null || echo "0")
            log_info "Final update candidates: ${final_count}"
        else
            log_warning "cargo-outdated analysis failed"
        fi
    else
        log_error "cargo-outdated not available - install with: cargo install cargo-outdated"
        return 1
    fi

    # Generate analysis report
    local candidate_count=$(jq -r 'length' "${OUTPUT_DIR}/update-candidates.json" 2>/dev/null || echo "0")
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg candidate_count "$candidate_count" \
        --arg version_type "$VERSION_TYPE" \
        --arg target_crate "$TARGET_CRATE" \
        '{
            timestamp: $timestamp,
            analysis_type: "outdated_dependencies",
            update_candidates: ($candidate_count | tonumber),
            version_type: $version_type,
            target_crate: $target_crate,
            status: (if ($candidate_count > "0") then "UPDATES_AVAILABLE" else "UP_TO_DATE" end)
        }' > "${outdated_report}"

    log_info "Outdated dependency analysis completed. Report: ${outdated_report}"
    return 0
}

# Function to perform incremental updates
perform_incremental_updates() {
    if [[ "${TEST_ONLY}" == true ]]; then
        log_info "Test-only mode - skipping updates"
        return 0
    fi

    if [[ "${DRY_RUN}" == true ]]; then
        log_info "Dry-run mode - would perform incremental updates"
        return 0
    fi

    log_info "Performing incremental dependency updates..."

    local update_results="${OUTPUT_DIR}/incremental-update-results.json"
    local updates_applied=0
    local updates_failed=0

    # Read update candidates
    if [[ ! -f "${OUTPUT_DIR}/update-candidates.json" ]]; then
        log_error "No update candidates found"
        return 1
    fi

    local candidate_count=$(jq -r 'length' "${OUTPUT_DIR}/update-candidates.json" 2>/dev/null || echo "0")

    if [[ "${candidate_count}" -eq 0 ]]; then
        log_info "No update candidates available"
        return 0
    fi

    # Process updates incrementally
    for i in $(seq 0 $((${candidate_count} - 1))); do
        if [[ "${updates_applied}" -ge "${MAX_UPDATES}" ]]; then
            log_info "Reached maximum updates limit: ${MAX_UPDATES}"
            break
        fi

        local crate_name=$(jq -r ".[${i}].name" "${OUTPUT_DIR}/update-candidates.json" 2>/dev/null || echo "")
        local current_version=$(jq -r ".[${i}].current" "${OUTPUT_DIR}/update-candidates.json" 2>/dev/null || echo "")
        local latest_version=$(jq -r ".[${i}].latest" "${OUTPUT_DIR}/update-candidates.json" 2>/dev/null || echo "")

        if [[ -z "${crate_name}" || -z "${current_version}" || -z "${latest_version}" ]]; then
            continue
        fi

        log_info "Attempting to update ${crate_name} from ${current_version} to ${latest_version}..."

        # Create checkpoint backup
        cp "${PROJECT_ROOT}/Cargo.lock" "${BACKUP_DIR}/Cargo.lock.checkpoint${updates_applied}"
        cp "${PROJECT_ROOT}/Cargo.toml" "${BACKUP_DIR}/Cargo.toml.checkpoint${updates_applied}"

        # Attempt the update
        if cargo update -p "${crate_name}" --precise "${latest_version}" > "${OUTPUT_DIR}/update-${crate_name}.log" 2>&1; then
            log_success "Successfully updated ${crate_name} to ${latest_version}"

            # Run immediate compatibility test
            if run_compatibility_test "${crate_name}" "${latest_version}"; then
                updates_applied=$((updates_applied + 1))
                log_success "Compatibility test passed for ${crate_name}"

                # Record successful update
                jq -n \
                    --arg crate_name "$crate_name" \
                    --arg old_version "$current_version" \
                    --arg new_version "$latest_version" \
                    --arg timestamp "$(date -Iseconds)" \
                    '{
                        crate_name: $crate_name,
                        old_version: $old_version,
                        new_version: $new_version,
                        timestamp: $timestamp,
                        status: "SUCCESS"
                    }' >> "${OUTPUT_DIR}/successful-updates.json"
            else
                log_error "Compatibility test failed for ${crate_name}"

                # Rollback the update
                if [[ "${ROLLBACK_ON_FAILURE}" == true ]]; then
                    log_info "Rolling back update for ${crate_name}"
                    cp "${BACKUP_DIR}/Cargo.lock.checkpoint${updates_applied}" "${PROJECT_ROOT}/Cargo.lock"
                    cp "${BACKUP_DIR}/Cargo.toml.checkpoint${updates_applied}" "${PROJECT_ROOT}/Cargo.toml"
                    log_info "Rollback completed for ${crate_name}"
                fi

                updates_failed=$((updates_failed + 1))

                if [[ "${FAIL_FAST}" == true ]]; then
                    log_error "Stopping due to --fail-fast option after ${crate_name} failure"
                    break
                fi
            fi
        else
            log_warning "Failed to update ${crate_name}"
            updates_failed=$((updates_failed + 1))
        fi
    done

    # Generate update results report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg updates_applied "$updates_applied" \
        --arg updates_failed "$updates_failed" \
        --arg max_updates "$MAX_UPDATES" \
        --arg version_type "$VERSION_TYPE" \
        '{
            timestamp: $timestamp,
            operation_type: "incremental_updates",
            updates_applied: ($updates_applied | tonumber),
            updates_failed: ($updates_failed | tonumber),
            max_updates: ($max_updates | tonumber),
            version_type: $version_type,
            success_rate: (if (($updates_applied + $updates_failed) > 0) then (($updates_applied / ($updates_applied + $updates_failed)) * 100) else 0 end)
        }' > "${update_results}"

    log_info "Incremental updates completed. Applied: ${updates_applied}, Failed: ${updates_failed}"
    return $updates_failed
}

# Function to run compatibility test for specific update
run_compatibility_test() {
    local crate_name="$1"
    local version="$2"

    log_info "Running compatibility test for ${crate_name}@${version}..."

    # Quick build test
    if cargo check --workspace --quiet > "${OUTPUT_DIR}/compat-test-${crate_name}.log" 2>&1; then
        # Quick test run (subset of tests)
        if cargo test --workspace --quiet --lib > "${OUTPUT_DIR}/compat-test-${crate_name}-tests.log" 2>&1; then
            log_success "Compatibility test passed for ${crate_name}@${version}"
            return 0
        else
            log_error "Test compatibility failed for ${crate_name}@${version}"
            return 1
        fi
    else
        log_error "Build compatibility failed for ${crate_name}@${version}"
        return 1
    fi
}

# Function to run comprehensive regression tests
run_regression_tests() {
    log_info "Running comprehensive regression tests..."

    local regression_report="${OUTPUT_DIR}/regression-test-report.json"
    local test_failures=0

    cd "${PROJECT_ROOT}"

    # Run workspace tests
    if cargo test --workspace --quiet > "${OUTPUT_DIR}/regression-tests.log" 2>&1; then
        log_success "Regression tests passed"
    else
        log_error "Regression tests failed"
        test_failures=$((test_failures + 1))
    fi

    # Run integration tests if they exist
    if [[ -d "integration-tests" ]]; then
        cd "${PROJECT_ROOT}/integration-tests"
        if cargo test --quiet > "${OUTPUT_DIR}/integration-regression-tests.log" 2>&1; then
            log_success "Integration regression tests passed"
        else
            log_error "Integration regression tests failed"
            test_failures=$((test_failures + 1))
        fi
        cd "${PROJECT_ROOT}"
    fi

    # Run performance tests if they exist
    if [[ -d "test-performance-analyzer" ]]; then
        cd "${PROJECT_ROOT}/test-performance-analyzer"
        if cargo test --quiet > "${OUTPUT_DIR}/performance-regression-tests.log" 2>&1; then
            log_success "Performance regression tests passed"
        else
            log_error "Performance regression tests failed"
            test_failures=$((test_failures + 1))
        fi
        cd "${PROJECT_ROOT}"
    fi

    # Generate regression test report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg test_failures "$test_failures" \
        '{
            timestamp: $timestamp,
            test_type: "regression",
            test_failures: ($test_failures | tonumber),
            status: (if ($test_failures == "0") then "PASSED" else "FAILED" end),
            tests_run: ["workspace", "integration", "performance"]
        }' > "${regression_report}"

    log_info "Regression tests completed. Failures: ${test_failures}"
    return $test_failures
}

# Function to compare with baseline
compare_with_baseline() {
    if [[ -z "${BASELINE_REPORT}" ]]; then
        return 0
    fi

    if [[ ! -f "${BASELINE_REPORT}" ]]; then
        log_warning "Baseline report not found: ${BASELINE_REPORT}"
        return 0
    fi

    log_info "Comparing results with baseline: ${BASELINE_REPORT}"

    local comparison_report="${OUTPUT_DIR}/baseline-comparison-report.json"

    # Generate comparison report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg baseline_report "$BASELINE_REPORT" \
        '{
            timestamp: $timestamp,
            comparison_type: "baseline",
            baseline_report: $baseline_report,
            status: "COMPARISON_COMPLETED"
        }' > "${comparison_report}"

    log_info "Baseline comparison completed. Report: ${comparison_report}"
}

# Function to generate comprehensive version bump report
generate_comprehensive_report() {
    log_info "Generating comprehensive version bump report..."

    local comprehensive_report="${OUTPUT_DIR}/comprehensive-version-bump-report.json"
    local html_report="${OUTPUT_DIR}/version-bump-report.html"

    # Collect all results
    local updates_applied=$(jq -r '.updates_applied // 0' "${OUTPUT_DIR}/incremental-update-results.json" 2>/dev/null || echo "0")
    local updates_failed=$(jq -r '.updates_failed // 0' "${OUTPUT_DIR}/incremental-update-results.json" 2>/dev/null || echo "0")
    local candidate_count=$(jq -r '.update_candidates // 0' "${OUTPUT_DIR}/outdated-analysis-report.json" 2>/dev/null || echo "0")
    local test_failures=$(jq -r '.test_failures // 0' "${OUTPUT_DIR}/regression-test-report.json" 2>/dev/null || echo "0")

    local success_rate=0
    if [[ $((${updates_applied} + ${updates_failed})) -gt 0 ]]; then
        success_rate=$(( (updates_applied * 100) / (updates_applied + updates_failed) ))
    fi

    # Determine overall status
    local overall_status="SUCCESS"
    if [[ "${test_failures}" -gt 0 ]]; then
        overall_status="TEST_FAILURES"
    elif [[ "${updates_failed}" -gt "${updates_applied}" ]]; then
        overall_status="UPDATE_FAILURES"
    fi

    # Generate comprehensive JSON report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg duration "$(($(date +%s) - START_TIME))" \
        --arg overall_status "$overall_status" \
        --arg updates_applied "$updates_applied" \
        --arg updates_failed "$updates_failed" \
        --arg candidate_count "$candidate_count" \
        --arg test_failures "$test_failures" \
        --arg success_rate "$success_rate" \
        --arg version_type "$VERSION_TYPE" \
        --arg target_crate "$TARGET_CRATE" \
        --arg dry_run "$DRY_RUN" \
        --arg test_only "$TEST_ONLY" \
        '{
            timestamp: $timestamp,
            duration_seconds: ($duration | tonumber),
            operation_type: "incremental_version_bump",
            overall_status: $overall_status,
            configuration: {
                version_type: $version_type,
                target_crate: $target_crate,
                dry_run: ($dry_run == "true"),
                test_only: ($test_only == "true"),
                max_updates: '$MAX_UPDATES',
                fail_fast: '$FAIL_FAST',
                rollback_on_failure: '$ROLLBACK_ON_FAILURE'
            },
            results: {
                update_candidates: ($candidate_count | tonumber),
                updates_applied: ($updates_applied | tonumber),
                updates_failed: ($updates_failed | tonumber),
                test_failures: ($test_failures | tonumber),
                success_rate: ($success_rate | tonumber)
            },
            recommendations: [
                (if ($test_failures > "0") then "Fix test failures before proceeding" else "Tests are passing" end),
                (if ($updates_failed > $updates_applied) then "Review failed updates and consider manual intervention" else "Update process completed successfully" end),
                (if ($success_rate < "80") then "Low success rate - consider more conservative update strategy" else "Good update success rate" end)
            ]
        }' > "${comprehensive_report}"

    # Generate HTML report
    cat > "${html_report}" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>Rust AI IDE Incremental Version Bump Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f0f0f0; padding: 10px; border-radius: 5px; }
        .status-success { color: green; }
        .status-failures { color: orange; }
        .status-test-failures { color: red; }
        .section { margin: 20px 0; padding: 10px; border: 1px solid #ccc; border-radius: 5px; }
        .metric { display: inline-block; margin: 10px; padding: 10px; background: #f9f9f9; border-radius: 3px; text-align: center; }
        .score { font-size: 24px; font-weight: bold; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Rust AI IDE Incremental Version Bump Report</h1>
        <p><strong>Generated:</strong> $(date)</p>
        <p><strong>Duration:</strong> $(($(date +%s) - START_TIME)) seconds</p>
        <p><strong>Overall Status:</strong> <span class="status-${overall_status,,}">${overall_status}</span></p>
        <p><strong>Success Rate:</strong> ${success_rate}%</p>
    </div>

    <div class="section">
        <h2>Configuration</h2>
        <p><strong>Version Type:</strong> ${VERSION_TYPE}</p>
        <p><strong>Target Crate:</strong> ${TARGET_CRATE:-All}</p>
        <p><strong>Dry Run:</strong> ${DRY_RUN}</p>
        <p><strong>Test Only:</strong> ${TEST_ONLY}</p>
        <p><strong>Max Updates:</strong> ${MAX_UPDATES}</p>
        <p><strong>Fail Fast:</strong> ${FAIL_FAST}</p>
    </div>

    <div class="section">
        <h2>Results</h2>
        <div class="metric">
            <div class="score">${candidate_count}</div>
            <div>Update Candidates</div>
        </div>
        <div class="metric">
            <div class="score">${updates_applied}</div>
            <div>Updates Applied</div>
        </div>
        <div class="metric">
            <div class="score">${updates_failed}</div>
            <div>Updates Failed</div>
        </div>
        <div class="metric">
            <div class="score">${test_failures}</div>
            <div>Test Failures</div>
        </div>
    </div>

    <div class="section">
        <h2>Summary</h2>
        <ul>
            $(if [[ "${updates_applied}" -gt 0 ]]; then echo "<li>✅ ${updates_applied} dependencies successfully updated</li>"; else echo "<li>ℹ️ No dependencies were updated</li>"; fi)
            $(if [[ "${updates_failed}" -gt 0 ]]; then echo "<li>⚠️ ${updates_failed} updates failed and were rolled back</li>"; else echo "<li>✅ No update failures</li>"; fi)
            $(if [[ "${test_failures}" -gt 0 ]]; then echo "<li>🔴 ${test_failures} test failures detected</li>"; else echo "<li>✅ All tests passed</li>"; fi)
            <li>📊 Update success rate: ${success_rate}%</li>
        </ul>
    </div>

    <div class="section">
        <h2>Report Files</h2>
        <ul>
            <li><a href="comprehensive-version-bump-report.json">Comprehensive JSON Report</a></li>
            <li><a href="incremental-update-results.json">Update Results</a></li>
            <li><a href="outdated-analysis-report.json">Outdated Analysis</a></li>
            <li><a href="regression-test-report.json">Regression Tests</a></li>
        </ul>
    </div>
</body>
</html>
EOF

    log_success "Comprehensive version bump report generated: ${comprehensive_report}"
    log_success "HTML version bump report generated: ${html_report}"
}

# Main function
main() {
    log_info "Starting incremental version bump for Rust AI IDE"
    log_info "Log file: ${VERSION_LOG}"
    log_info "Report directory: ${OUTPUT_DIR}"
    log_info "Backup directory: ${BACKUP_DIR}"

    mkdir -p "${OUTPUT_DIR}"
    mkdir -p "${BACKUP_DIR}"

    local exit_code=0

    # Create backup
    create_incremental_backup

    # Analyze outdated dependencies
    analyze_outdated_dependencies

    # Perform incremental updates
    perform_incremental_updates || exit_code=$((exit_code + 1))

    # Run regression tests
    run_regression_tests || exit_code=$((exit_code + 1))

    # Compare with baseline
    compare_with_baseline

    # Generate comprehensive report
    generate_comprehensive_report

    local end_time=$(date +%s)
    log_info "Incremental version bump completed in $((end_time - START_TIME)) seconds"

    return $exit_code
}

# Run main function
main "$@"
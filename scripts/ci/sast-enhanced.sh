#!/bin/bash

# Enhanced Static Application Security Testing (SAST) Script
# Advanced security analysis with automated false positive filtering
# Author: Security Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
SAST_LOG="${PROJECT_ROOT}/sast-enhanced.log"
REPORT_DIR="${PROJECT_ROOT}/security-reports"
START_TIME=$(date +%s)

# Create report directory
mkdir -p "${REPORT_DIR}"

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${SAST_LOG}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${SAST_LOG}" >&2
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${SAST_LOG}"
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${SAST_LOG}"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Enhanced Static Application Security Testing with false positive filtering.

OPTIONS:
    -h, --help              Show this help message
    -v, --verbose           Enable verbose output
    -s, --strict            Fail on warnings (stricter mode)
    -o, --output DIR        Output directory for reports (default: security-reports)
    --no-false-positives    Disable false positive filtering
    --severity-threshold LVL Set minimum severity level (info, warning, error)
    --exclude-pattern PAT   Exclude files matching pattern
    --include-pattern PAT   Only include files matching pattern

EXAMPLES:
    $0 --verbose --output /tmp/sast-results
    $0 --strict --severity-threshold warning
    $0 --exclude-pattern "target/*" --include-pattern "*.rs"

EOF
}

# Parse command line arguments
VERBOSE=false
STRICT=false
OUTPUT_DIR="${REPORT_DIR}/sast"
FALSE_POSITIVE_FILTERING=true
SEVERITY_THRESHOLD="info"
EXCLUDE_PATTERN=""
INCLUDE_PATTERN=""

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
        -s|--strict)
            STRICT=true
            shift
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --no-false-positives)
            FALSE_POSITIVE_FILTERING=false
            shift
            ;;
        --severity-threshold)
            SEVERITY_THRESHOLD="$2"
            shift 2
            ;;
        --exclude-pattern)
            EXCLUDE_PATTERN="$2"
            shift 2
            ;;
        --include-pattern)
            INCLUDE_PATTERN="$2"
            shift 2
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Create output directory
mkdir -p "${OUTPUT_DIR}"

# Function to check if nightly toolchain is available
check_nightly_toolchain() {
    if ! cargo +nightly --version >/dev/null 2>&1; then
        log_error "Nightly Rust toolchain required for enhanced SAST"
        return 1
    fi
    log_success "Nightly Rust toolchain available for SAST"
}

# Function to filter false positives
filter_false_positives() {
    local input_file="$1"
    local output_file="$2"

    if [[ "${FALSE_POSITIVE_FILTERING}" != true ]]; then
        cp "${input_file}" "${output_file}"
        return 0
    fi

    log_info "Filtering false positives..."

    # Common false positive patterns to filter out
    jq '
        # Filter out known false positives
        . as $root |
        if .diagnostics then
            .diagnostics |= map(
                select(
                    # Filter out clippy false positives
                    (.code != "clippy::needless_collect" or (.spans[0].file_name // "" | contains("test") | not)) and
                    (.code != "clippy::single_match" or (.spans[0].file_name // "" | contains("test") | not)) and
                    (.code != "clippy::too_many_arguments" or (.spans[0].file_name // "" | contains("test") | not)) and
                    # Filter out async false positives
                    (.code != "clippy::unused_async" or (.spans[0].file_name // "" | contains("async") | not)) and
                    # Filter out test-related warnings
                    (.code != "clippy::unwrap_used" or (.spans[0].file_name // "" | contains("test") | not)) and
                    (.code != "clippy::expect_used" or (.spans[0].file_name // "" | contains("test") | not)) and
                    # Filter out macro-generated code
                    (.spans[0].file_name // "" | contains("macro") | not)
                )
            ) |
            # Recalculate counts after filtering
            .statistics = {
                "error_count": (.diagnostics | map(select(.level == "error")) | length),
                "warning_count": (.diagnostics | map(select(.level == "warning")) | length),
                "info_count": (.diagnostics | map(select(.level == "info")) | length),
                "filtered_count": (($root.diagnostics // []) | length) - (.diagnostics | length)
            }
        else
            .
        end
    ' "${input_file}" > "${output_file}"

    local filtered_count=$(jq -r '.statistics.filtered_count // 0' "${output_file}")
    if [[ "${filtered_count}" -gt 0 ]]; then
        log_info "Filtered ${filtered_count} false positive(s)"
    fi
}

# Function to run enhanced Clippy analysis
run_clippy_analysis() {
    log_info "Running enhanced Clippy security analysis..."

    local clippy_output="${OUTPUT_DIR}/clippy-raw.json"
    local clippy_filtered="${OUTPUT_DIR}/clippy-filtered.json"
    local issues_found=0

    # Run Clippy with comprehensive security checks
    if cargo +nightly clippy --all-targets --all-features \
        --message-format=json \
        -- -W clippy::all \
        -W clippy::pedantic \
        -W clippy::nursery \
        -D clippy::unwrap_used \
        -D clippy::expect_used \
        -D clippy::panic \
        -D clippy::unimplemented \
        -D clippy::todo \
        -D clippy::unreachable \
        -D clippy::exit \
        -D clippy::print_stdout \
        -D clippy::print_stderr \
        -D clippy::dbg_macro \
        -W clippy::mut_mut \
        -W clippy::string_add_assign \
        -W clippy::string_add \
        -W clippy::needless_borrow \
        -W clippy::cast_lossless \
        -W clippy::cast_possible_truncation \
        -W clippy::cast_possible_wrap \
        -W clippy::cast_precision_loss \
        -W clippy::cast_sign_loss \
        -W clippy::implicit_clone \
        -W clippy::if_not_else \
        -W clippy::int_plus_one \
        -W clippy::large_stack_arrays \
        -W clippy::large_types_passed_by_value \
        -W clippy::linkedlist \
        -W clippy::macro_use_imports \
        -W clippy::manual_range_contains \
        -W clippy::manual_strip \
        -W clippy::map_unwrap_or \
        -W clippy::match_same_arms \
        -W clippy::needless_for_each \
        -W clippy::or_fun_call \
        -W clippy::redundant_clone \
        -W clippy::redundant_else \
        -W clippy::single_match_else \
        -W clippy::stable_sort_primitive \
        -W clippy::unnested_or_patterns \
        -W clippy::unused_peekable \
        -W clippy::unused_rounding \
        -W clippy::use_self 2>"${OUTPUT_DIR}/clippy-errors.log" | \
        jq -s '.' > "${clippy_output}"; then

        log_success "Clippy analysis completed"
    else
        log_warning "Clippy analysis completed with errors (see ${OUTPUT_DIR}/clippy-errors.log)"
        issues_found=$((issues_found + 1))
    fi

    # Filter false positives
    filter_false_positives "${clippy_output}" "${clippy_filtered}"

    # Analyze results
    local error_count=$(jq -r '.[] | select(.level == "error") | length' "${clippy_filtered}" 2>/dev/null || echo "0")
    local warning_count=$(jq -r '.[] | select(.level == "warning") | length' "${clippy_filtered}" 2>/dev/null || echo "0")
    local info_count=$(jq -r '.[] | select(.level == "info") | length' "${clippy_filtered}" 2>/dev/null || echo "0")

    if [[ "${error_count}" -gt 0 ]]; then
        issues_found=$((issues_found + error_count))
        log_error "Found ${error_count} Clippy errors"
    fi

    if [[ "${warning_count}" -gt 0 ]]; then
        if [[ "${STRICT}" == true ]]; then
            issues_found=$((issues_found + warning_count))
            log_error "Found ${warning_count} Clippy warnings (strict mode)"
        else
            log_warning "Found ${warning_count} Clippy warnings"
        fi
    fi

    if [[ "${VERBOSE}" == true && "${info_count}" -gt 0 ]]; then
        log_info "Found ${info_count} Clippy info messages"
    fi

    # Generate Clippy summary
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg error_count "$error_count" \
        --arg warning_count "$warning_count" \
        --arg info_count "$info_count" \
        --arg strict_mode "$STRICT" \
        --arg false_positive_filtering "$FALSE_POSITIVE_FILTERING" \
        --arg severity_threshold "$SEVERITY_THRESHOLD" \
        '{
            timestamp: $timestamp,
            tool: "Enhanced Clippy SAST",
            severity_threshold: $severity_threshold,
            strict_mode: ($strict_mode == "true"),
            false_positive_filtering: ($false_positive_filtering == "true"),
            issues_found: {
                errors: ($error_count | tonumber),
                warnings: ($warning_count | tonumber),
                info: ($info_count | tonumber)
            },
            status: (if ($error_count | tonumber) > 0 or (($strict_mode == "true") and ($warning_count | tonumber) > 0) then "FAILED" else "PASSED" end)
        }' > "${OUTPUT_DIR}/clippy-summary.json"

    log_info "Clippy analysis summary saved to ${OUTPUT_DIR}/clippy-summary.json"
    return $issues_found
}

# Function to analyze unsafe code patterns
analyze_unsafe_code() {
    log_info "Analyzing unsafe code patterns..."

    local unsafe_report="${OUTPUT_DIR}/unsafe-analysis.json"
    local unsafe_issues=0

    if command -v cargo-geiger >/dev/null 2>&1; then
        # Run cargo-geiger for unsafe code analysis
        if cargo +nightly geiger --format json --output "${unsafe_report}" 2>/dev/null; then
            local unsafe_functions=$(jq -r '.metrics.functions.unsafe // 0' "${unsafe_report}" 2>/dev/null || echo "0")
            local unsafe_exprs=$(jq -r '.metrics.expressions.unsafe // 0' "${unsafe_report}" 2>/dev/null || echo "0")

            if [[ "${unsafe_functions}" -gt 0 || "${unsafe_exprs}" -gt 0 ]]; then
                unsafe_issues=$((unsafe_functions + unsafe_exprs))
                log_warning "Found ${unsafe_functions} unsafe functions and ${unsafe_exprs} unsafe expressions"

                if [[ "${STRICT}" == true ]]; then
                    log_error "Unsafe code detected in strict mode"
                fi
            else
                log_success "No unsafe code detected"
            fi
        else
            log_warning "cargo-geiger analysis failed"
        fi
    else
        log_warning "cargo-geiger not available for unsafe code analysis"
    fi

    # Manual unsafe code pattern detection
    log_info "Performing manual unsafe code pattern detection..."

    local manual_unsafe="${OUTPUT_DIR}/manual-unsafe-analysis.json"
    local manual_issues=0

    # Find files with unsafe blocks and analyze them
    find "${PROJECT_ROOT}/src" "${PROJECT_ROOT}/crates" -name "*.rs" -type f 2>/dev/null | while read -r file; do
        if grep -q "unsafe" "${file}"; then
            local unsafe_blocks=$(grep -c "unsafe" "${file}")
            local unsafe_lines=$(grep -n "unsafe" "${file}" | wc -l)

            if [[ "${unsafe_blocks}" -gt 0 ]]; then
                manual_issues=$((manual_issues + unsafe_blocks))
                log_info "File ${file} contains ${unsafe_blocks} unsafe blocks"

                if [[ "${VERBOSE}" == true ]]; then
                    grep -n "unsafe" "${file}" | head -5
                fi
            fi
        fi
    done

    # Generate unsafe code summary
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg unsafe_issues "$unsafe_issues" \
        --arg manual_issues "$manual_issues" \
        '{
            timestamp: $timestamp,
            tool: "Unsafe Code Analysis",
            automated_analysis: true,
            manual_analysis: true,
            issues_found: {
                automated: ($unsafe_issues | tonumber),
                manual: ($manual_issues | tonumber)
            },
            status: (if (($unsafe_issues | tonumber) + ($manual_issues | tonumber)) > 0 then "ISSUES_FOUND" else "CLEAN" end)
        }' > "${OUTPUT_DIR}/unsafe-summary.json"

    log_info "Unsafe code analysis summary saved to ${OUTPUT_DIR}/unsafe-summary.json"
    return $((unsafe_issues + manual_issues))
}

# Function to detect security patterns
detect_security_patterns() {
    log_info "Detecting common security vulnerability patterns..."

    local patterns_report="${OUTPUT_DIR}/security-patterns.json"
    local pattern_issues=0

    # Security vulnerability patterns to check
    declare -a patterns=(
        "unwrap|expect"  # Unchecked unwraps
        "panic!"         # Panic calls
        "todo!|unimplemented!"  # Incomplete code
        "unsafe"         # Unsafe code
        "mem::uninitialized|mem::zeroed"  # Uninitialized memory
        "transmute"      # Dangerous type conversions
        "get_unchecked|index_unchecked"  # Bounds checking bypass
        "static mut"     # Mutable statics
        "lazy_static"    # Global state
    )

    local patterns_json="{}"

    for pattern in "${patterns[@]}"; do
        log_info "Checking pattern: ${pattern}"

        local files_found=$(find "${PROJECT_ROOT}/src" "${PROJECT_ROOT}/crates" -name "*.rs" -type f -exec grep -l "${pattern}" {} \; 2>/dev/null | wc -l)
        local total_occurrences=$(find "${PROJECT_ROOT}/src" "${PROJECT_ROOT}/crates" -name "*.rs" -type f -exec grep -c "${pattern}" {} \; 2>/dev/null | awk '{sum += $1} END {print sum}')

        if [[ "${files_found}" -gt 0 ]]; then
            pattern_issues=$((pattern_issues + files_found))
            log_warning "Pattern '${pattern}' found in ${files_found} files (${total_occurrences} occurrences)"

            # Add to JSON
            patterns_json=$(echo "${patterns_json}" | jq --arg pattern "${pattern}" --arg files "${files_found}" --arg occurrences "${total_occurrences}" \
                '. + {($pattern): {files: ($files | tonumber), occurrences: ($occurrences | tonumber)}}')
        fi
    done

    # Generate security patterns summary
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --argjson patterns "$patterns_json" \
        --arg pattern_issues "$pattern_issues" \
        '{
            timestamp: $timestamp,
            tool: "Security Pattern Detection",
            patterns_analyzed: ($patterns | length),
            issues_found: ($pattern_issues | tonumber),
            pattern_details: $patterns,
            status: (if ($pattern_issues | tonumber) > 0 then "PATTERNS_FOUND" else "NO_PATTERNS" end)
        }' > "${patterns_report}"

    log_info "Security pattern detection summary saved to ${patterns_report}"
    return $pattern_issues
}

# Function to generate comprehensive SAST report
generate_sast_report() {
    log_info "Generating comprehensive SAST report..."

    local comprehensive_report="${OUTPUT_DIR}/comprehensive-sast-report.json"
    local html_report="${OUTPUT_DIR}/sast-report.html"

    # Combine all SAST results
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg duration "$(($(date +%s) - START_TIME))" \
        --arg false_positive_filtering "$FALSE_POSITIVE_FILTERING" \
        --arg severity_threshold "$SEVERITY_THRESHOLD" \
        --arg strict_mode "$STRICT" \
        '{
            timestamp: $timestamp,
            duration_seconds: ($duration | tonumber),
            tool: "Enhanced SAST Suite",
            components: ["Clippy Analysis", "Unsafe Code Analysis", "Security Pattern Detection"],
            configuration: {
                false_positive_filtering: ($false_positive_filtering == "true"),
                severity_threshold: $severity_threshold,
                strict_mode: ($strict_mode == "true")
            }
        }' > "${comprehensive_report}"

    # Add individual component results
    if [[ -f "${OUTPUT_DIR}/clippy-summary.json" ]]; then
        local clippy_data=$(cat "${OUTPUT_DIR}/clippy-summary.json")
        jq --argjson clippy "${clippy_data}" '.clippy_analysis = $clippy' "${comprehensive_report}" > "${comprehensive_report}.tmp" && mv "${comprehensive_report}.tmp" "${comprehensive_report}"
    fi

    if [[ -f "${OUTPUT_DIR}/unsafe-summary.json" ]]; then
        local unsafe_data=$(cat "${OUTPUT_DIR}/unsafe-summary.json")
        jq --argjson unsafe "${unsafe_data}" '.unsafe_analysis = $unsafe' "${comprehensive_report}" > "${comprehensive_report}.tmp" && mv "${comprehensive_report}.tmp" "${comprehensive_report}"
    fi

    if [[ -f "${OUTPUT_DIR}/security-patterns.json" ]]; then
        local patterns_data=$(cat "${OUTPUT_DIR}/security-patterns.json")
        jq --argjson patterns "${patterns_data}" '.security_patterns = $patterns' "${comprehensive_report}" > "${comprehensive_report}.tmp" && mv "${comprehensive_report}.tmp" "${comprehensive_report}"
    fi

    # Calculate overall status
    local total_issues=$(jq -r '.clippy_analysis.issues_found.errors // 0 + .clippy_analysis.issues_found.warnings // 0 + .unsafe_analysis.issues_found.manual // 0 + .security_patterns.issues_found // 0' "${comprehensive_report}" 2>/dev/null || echo "0")
    local overall_status="PASSED"

    if [[ "${total_issues}" -gt 0 ]]; then
        overall_status="ISSUES_FOUND"
    fi

    jq --arg status "${overall_status}" --arg issues "${total_issues}" '.overall_status = $status | .total_issues_found = ($issues | tonumber)' "${comprehensive_report}" > "${comprehensive_report}.tmp" && mv "${comprehensive_report}.tmp" "${comprehensive_report}"

    # Generate HTML report
    cat > "${html_report}" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>Enhanced SAST Report - Rust AI IDE</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f0f0f0; padding: 10px; border-radius: 5px; }
        .status-passed { color: green; }
        .status-issues-found { color: orange; }
        .status-failed { color: red; }
        .section { margin: 20px 0; padding: 10px; border: 1px solid #ccc; border-radius: 5px; }
        .metric { display: inline-block; margin: 10px; padding: 10px; background: #f9f9f9; border-radius: 3px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Enhanced Static Application Security Testing (SAST) Report</h1>
        <p><strong>Generated:</strong> $(date)</p>
        <p><strong>Duration:</strong> $(($(date +%s) - START_TIME)) seconds</p>
        <p><strong>Overall Status:</strong> <span class="status-${overall_status,,}">${overall_status}</span></p>
        <p><strong>Total Issues Found:</strong> ${total_issues}</p>
    </div>

    <div class="section">
        <h2>Configuration</h2>
        <ul>
            <li><strong>False Positive Filtering:</strong> ${FALSE_POSITIVE_FILTERING}</li>
            <li><strong>Severity Threshold:</strong> ${SEVERITY_THRESHOLD}</li>
            <li><strong>Strict Mode:</strong> ${STRICT}</li>
        </ul>
    </div>

    <div class="section">
        <h2>Analysis Results</h2>
        <div class="metric">
            <strong>Clippy Analysis</strong><br>
            Errors: $(jq -r '.clippy_analysis.issues_found.errors // 0' "${comprehensive_report}")<br>
            Warnings: $(jq -r '.clippy_analysis.issues_found.warnings // 0' "${comprehensive_report}")<br>
            Info: $(jq -r '.clippy_analysis.issues_found.info // 0' "${comprehensive_report}")
        </div>
        <div class="metric">
            <strong>Unsafe Code</strong><br>
            Automated: $(jq -r '.unsafe_analysis.issues_found.automated // 0' "${comprehensive_report}")<br>
            Manual: $(jq -r '.unsafe_analysis.issues_found.manual // 0' "${comprehensive_report}")
        </div>
        <div class="metric">
            <strong>Security Patterns</strong><br>
            Patterns Found: $(jq -r '.security_patterns.issues_found // 0' "${comprehensive_report}")
        </div>
    </div>

    <div class="section">
        <h3>Report Files</h3>
        <ul>
            <li><a href="comprehensive-sast-report.json">Comprehensive JSON Report</a></li>
            $(ls -1 "${OUTPUT_DIR}" | grep -E '\.(json|log)$' | sed 's/\(.*\)/<li>\1<\/li>/')
        </ul>
    </div>
</body>
</html>
EOF

    log_info "Comprehensive SAST report generated: ${comprehensive_report}"
    log_info "HTML SAST report generated: ${html_report}"

    # Log final status
    if [[ "${overall_status}" == "PASSED" ]]; then
        log_success "SAST completed successfully - no security issues found"
        return 0
    else
        log_warning "SAST completed with ${total_issues} issue(s) found"
        return $total_issues
    fi
}

# Main function
main() {
    log_info "Starting Enhanced Static Application Security Testing"
    log_info "Log file: ${SAST_LOG}"
    log_info "Output directory: ${OUTPUT_DIR}"

    # Trap to ensure cleanup on exit
    trap 'log_info "SAST completed (exit code: $?)"; [[ -d "${OUTPUT_DIR}" ]] && echo "Reports available in: ${OUTPUT_DIR}"' EXIT

    mkdir -p "${OUTPUT_DIR}"

    check_nightly_toolchain

    local exit_code=0
    local total_issues=0

    # Run SAST components
    run_clippy_analysis || exit_code=$((exit_code + 1))
    analyze_unsafe_code || exit_code=$((exit_code + 1))
    detect_security_patterns || exit_code=$((exit_code + 1))

    # Generate comprehensive report
    generate_sast_report || exit_code=$((exit_code + 1))

    local end_time=$(date +%s)
    log_info "Enhanced SAST completed in $((end_time - START_TIME)) seconds"

    return $exit_code
}

# Run main function
main "$@"
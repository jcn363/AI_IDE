#!/bin/bash

# Performance Trends Analysis Script
# Analyzes performance metrics and trends for the Rust AI IDE
# Author: Performance Team
# Version: 1.0.0

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PERFORMANCE_LOG="${PROJECT_ROOT}/logs/performance-trends-$(date +%Y%m%d).log"
REPORTS_DIR="${PROJECT_ROOT}/reports/performance"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${PERFORMANCE_LOG}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${PERFORMANCE_LOG}" >&2
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${PERFORMANCE_LOG}"
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${PERFORMANCE_LOG}"
}

# Function to measure build time
measure_build_time() {
    log_info "Measuring build performance..."

    local start_time=$(date +%s.%3N)
    if cargo build --release --quiet >/dev/null 2>&1; then
        local end_time=$(date +%s.%3N)
        local build_time=$(echo "$end_time - $start_time" | bc)

        log_info "Build completed in ${build_time}s"
        echo "$build_time"
    else
        log_error "Build failed"
        echo "0"
    fi
}

# Function to measure test execution time
measure_test_time() {
    log_info "Measuring test performance..."

    local start_time=$(date +%s.%3N)
    if cargo test --release --quiet >/dev/null 2>&1; then
        local end_time=$(date +%s.%3N)
        local test_time=$(echo "$end_time - $start_time" | bc)

        log_info "Tests completed in ${test_time}s"
        echo "$test_time"
    else
        log_error "Tests failed"
        echo "0"
    fi
}

# Function to analyze binary size
analyze_binary_size() {
    log_info "Analyzing binary size..."

    # Build release binary
    if cargo build --release --quiet >/dev/null 2>&1; then
        local binary_path="${PROJECT_ROOT}/target/release/rust-ai-ide"

        if [[ -f "$binary_path" ]]; then
            local size_kb=$(du -k "$binary_path" | cut -f1)
            log_info "Binary size: ${size_kb}KB"
            echo "$size_kb"
        else
            log_warning "Release binary not found"
            echo "0"
        fi
    else
        log_error "Failed to build release binary"
        echo "0"
    fi
}

# Function to measure memory usage
measure_memory_usage() {
    log_info "Measuring memory usage..."

    # Run a simple cargo check and measure memory
    local mem_usage=""

    if command -v /usr/bin/time >/dev/null 2>&1; then
        # Use GNU time to measure memory
        mem_usage=$(/usr/bin/time -f "%M" cargo check --quiet 2>&1 | tail -1)
        local mem_kb=$((mem_usage))
        log_info "Peak memory usage: ${mem_kb}KB"
        echo "$mem_kb"
    else
        log_warning "GNU time not available, skipping memory measurement"
        echo "0"
    fi
}

# Function to check compilation warnings
check_compilation_warnings() {
    log_info "Checking compilation warnings..."

    local warnings=$(cargo check 2>&1 | grep -c "warning:" || true)
    log_info "Compilation warnings: $warnings"

    if [[ $warnings -gt 50 ]]; then
        log_warning "High number of compilation warnings detected"
    elif [[ $warnings -eq 0 ]]; then
        log_success "No compilation warnings"
    else
        log_info "Some compilation warnings present"
    fi

    echo "$warnings"
}

# Function to analyze dependencies
analyze_dependencies() {
    log_info "Analyzing dependency count..."

    local dep_count=$(cargo tree | grep -c "├──" || true)
    log_info "Total dependencies: $dep_count"

    if [[ $dep_count -gt 200 ]]; then
        log_warning "High dependency count detected"
    else
        log_success "Dependency count is reasonable"
    fi

    echo "$dep_count"
}

# Function to generate performance report
generate_performance_report() {
    local build_time="$1"
    local test_time="$2"
    local binary_size="$3"
    local memory_usage="$4"
    local warnings="$5"
    local dependencies="$6"

    mkdir -p "${REPORTS_DIR}"

    local report_file="${REPORTS_DIR}/performance-report-$(date +%Y%m%d_%H%M%S).json"

    cat > "${report_file}" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "performance_metrics": {
        "build_time_seconds": $build_time,
        "test_time_seconds": $test_time,
        "binary_size_kb": $binary_size,
        "memory_usage_kb": $memory_usage,
        "compilation_warnings": $warnings,
        "dependency_count": $dependencies
    },
    "thresholds": {
        "max_build_time": 300,
        "max_test_time": 600,
        "max_binary_size_mb": 100,
        "max_warnings": 50,
        "max_dependencies": 200
    },
    "recommendations": []
}
EOF

    # Add recommendations based on metrics
    generate_recommendations "$report_file" "$build_time" "$test_time" "$binary_size" "$warnings" "$dependencies"

    log_success "Performance report generated: ${report_file}"
    echo "${report_file}"
}

# Function to generate recommendations
generate_recommendations() {
    local report_file="$1"
    local build_time="$2"
    local test_time="$3"
    local binary_size="$4"
    local warnings="$5"
    local dependencies="$6"

    local recommendations=()

    # Build time recommendations
    if (( $(echo "$build_time > 300" | bc -l) )); then
        recommendations+=("Build time is high (${build_time}s). Consider enabling incremental compilation or optimizing build dependencies.")
    fi

    # Test time recommendations
    if (( $(echo "$test_time > 600" | bc -l) )); then
        recommendations+=("Test execution time is high (${test_time}s). Consider parallelizing tests or optimizing slow test cases.")
    fi

    # Binary size recommendations
    local binary_mb=$(echo "scale=2; $binary_size / 1024" | bc)
    if (( $(echo "$binary_mb > 100" | bc -l) )); then
        recommendations+=("Binary size is large (${binary_mb}MB). Consider enabling size optimizations or removing unused dependencies.")
    fi

    # Compilation warnings
    if [[ $warnings -gt 50 ]]; then
        recommendations+=("High number of compilation warnings ($warnings). Address warnings to improve code quality.")
    fi

    # Dependencies
    if [[ $dependencies -gt 200 ]]; then
        recommendations+=("High dependency count ($dependencies). Consider auditing dependencies for unused crates.")
    fi

    # Update report with recommendations
    if [[ ${#recommendations[@]} -gt 0 ]]; then
        local recs_json=$(printf '%s\n' "${recommendations[@]}" | jq -R . | jq -s .)
        jq --argjson recs "$recs_json" '.recommendations = $recs' "$report_file" > "${report_file}.tmp"
        mv "${report_file}.tmp" "$report_file"
    fi
}

# Function to compare with previous runs
compare_with_previous() {
    local current_report="$1"

    log_info "Comparing with previous performance reports..."

    # Find previous report
    local previous_report=$(find "${REPORTS_DIR}" -name "performance-report-*.json" -not -name "$(basename "$current_report")" | head -1)

    if [[ -n "$previous_report" ]]; then
        log_info "Comparing with previous report: $(basename "$previous_report")"

        # Extract metrics from both reports
        local current_build=$(jq '.performance_metrics.build_time_seconds' "$current_report")
        local previous_build=$(jq '.performance_metrics.build_time_seconds' "$previous_report")

        if (( $(echo "$current_build > $previous_build" | bc -l) )); then
            local diff=$(echo "$current_build - $previous_build" | bc)
            log_warning "Build time increased by ${diff}s compared to previous run")
        elif (( $(echo "$previous_build > $current_build" | bc -l) )); then
            local diff=$(echo "$previous_build - $current_build" | bc)
            log_success "Build time improved by ${diff}s compared to previous run")
        fi
    else
        log_info "No previous performance report found for comparison"
    fi
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS] [ACTION]

Performance Trends Analysis for Rust AI IDE

ACTIONS:
    full                Run complete performance analysis (default)
    build               Measure build time only
    test                Measure test time only
    size                Analyze binary size only
    memory              Measure memory usage only
    warnings            Check compilation warnings only
    dependencies        Analyze dependencies only

OPTIONS:
    -h, --help           Show this help message
    -v, --verbose        Enable verbose output
    --analyze-only       Skip measurements, analyze existing data only
    --compare            Compare with previous runs
    --output FILE        Save results to specific file

EXAMPLES:
    $0                    Run complete performance analysis
    $0 --compare         Run analysis and compare with previous runs
    $0 build             Measure build time only
    $0 --analyze-only    Analyze existing performance data

EOF
}

# Parse command line arguments
ANALYZE_ONLY=false
COMPARE=false
OUTPUT_FILE=""
ACTION="full"

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
        --analyze-only)
            ANALYZE_ONLY=true
            shift
            ;;
        --compare)
            COMPARE=true
            shift
            ;;
        --output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        build|test|size|memory|warnings|dependencies|full)
            ACTION="$1"
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Main execution function
main() {
    mkdir -p "${REPORTS_DIR}"

    log_info "Starting performance trends analysis (Action: ${ACTION})"

    if [[ "${ANALYZE_ONLY}" == true ]]; then
        log_info "ANALYZE-ONLY MODE: Skipping measurements"
        exit 0
    fi

    local build_time="0"
    local test_time="0"
    local binary_size="0"
    local memory_usage="0"
    local warnings="0"
    local dependencies="0"

    case "${ACTION}" in
        full)
            build_time=$(measure_build_time)
            test_time=$(measure_test_time)
            binary_size=$(analyze_binary_size)
            memory_usage=$(measure_memory_usage)
            warnings=$(check_compilation_warnings)
            dependencies=$(analyze_dependencies)
            ;;
        build)
            build_time=$(measure_build_time)
            ;;
        test)
            test_time=$(measure_test_time)
            ;;
        size)
            binary_size=$(analyze_binary_size)
            ;;
        memory)
            memory_usage=$(measure_memory_usage)
            ;;
        warnings)
            warnings=$(check_compilation_warnings)
            ;;
        dependencies)
            dependencies=$(analyze_dependencies)
            ;;
        *)
            log_error "Unknown action: ${ACTION}"
            usage
            exit 1
            ;;
    esac

    # Generate report
    local report_file
    if [[ -n "$OUTPUT_FILE" ]]; then
        report_file="$OUTPUT_FILE"
    else
        report_file=$(generate_performance_report "$build_time" "$test_time" "$binary_size" "$memory_usage" "$warnings" "$dependencies")
    fi

    # Compare with previous runs if requested
    if [[ "${COMPARE}" == true ]]; then
        compare_with_previous "$report_file"
    fi

    log_success "Performance trends analysis completed"
    log_info "Report: $report_file"
}

# Execute main function
main "$@"
#!/bin/bash

# Performance Trend Analysis Script
# Analyzes benchmark data over time to identify performance regressions and improvements

set -euo pipefail

# Configuration
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BENCHMARK_DIR="${WORKSPACE_ROOT}/benchmark-data"
TREND_REPORT="${WORKSPACE_ROOT}/performance-trends-report.md"
THRESHOLD_DEGRADATION_PERCENT=5
THRESHOLD_WARNING_PERCENT=10

# Logging functions
log_info() {
    echo -e "\033[34m[INFO]\033[0m $(date '+%Y-%m-%d %H:%M:%S') $*" >&2
}

log_warn() {
    echo -e "\033[33m[WARN]\033[0m $(date '+%Y-%m-%d %H:%M:%S') $*" >&2
}

log_error() {
    echo -e "\033[31m[ERROR]\033[0m $(date '+%Y-%m-%d %H:%M:%S') $*" >&2
}

# Setup benchmark directory structure
setup_directories() {
    log_info "Setting up benchmark directory structure..."
    mkdir -p "$BENCHMARK_DIR"/{"current","historical","trends"}
}

# Collect current benchmark data from GitHub artifacts
collect_current_benchmarks() {
    log_info "Collecting current benchmark data..."

    # This would typically be done by downloading from GitHub API
    # For now, we'll work with local benchmark files if available

    find . -name "benchmark-results-*.zip" -type f 2>/dev/null | while read -r archive; do
        log_info "Extracting benchmarks from $archive"
        unzip -q "$archive" -d "$BENCHMARK_DIR/current/" 2>/dev/null || \
        tar -xzf "$archive" -C "$BENCHMARK_DIR/current/" 2>/dev/null || \
        log_warn "Could not extract $archive"
    done
}

# Archive historical data
archive_historical_data() {
    log_info "Archiving historical benchmark data..."

    if [ -d "$BENCHMARK_DIR/current" ]; then
        timestamp=$(date +%Y%m%d_%H%M%S)
        cp -r "$BENCHMARK_DIR/current" "$BENCHMARK_DIR/historical/$timestamp"
    fi
}

# Analyze compilation time trends
analyze_compilation_trends() {
    log_info "Analyzing compilation time trends..."

    local current_compilation_file=""
    local historical_compilation_file=""

    # Find latest compilation benchmarks
    if [ -n "$(find "$BENCHMARK_DIR/current" -name "*compilation*.json" 2>/dev/null)" ]; then
        current_compilation_file=$(find "$BENCHMARK_DIR/current" -name "*compilation*.json" | head -1)
    fi

    if [ -n "$(find "$BENCHMARK_DIR/historical" -name "*compilation*.json" 2>/dev/null)" ]; then
        historical_compilation_file=$(find "$BENCHMARK_DIR/historical" -name "*compilation*.json" | sort -r | head -1)
    fi

    if [ -n "$current_compilation_file" ] && [ -n "$historical_compilation_file" ]; then
        current_time=$(jq -r '.compilation_time // 0' "$current_compilation_file" 2>/dev/null || echo "0")
        historical_time=$(jq -r '.compilation_time // 0' "$historical_compilation_file" 2>/dev/null || echo "0")

        if [ "$historical_time" != "0" ] && [ "$current_time" != "0" ]; then
            change_percent=$(echo "scale=2; (($current_time - $historical_time) / $historical_time) * 100" | bc -l 2>/dev/null || echo "0")

            echo "COMPILATION_TREND=$change_percent" >> "$GITHUB_ENV"
            echo "COMPILATION_CHANGE_PERCENT=$change_percent" >> "$GITHUB_ENV"

            if (( $(echo "$change_percent > $THRESHOLD_WARNING_PERCENT" | bc -l) )); then
                log_error "Significant compilation time increase: ${change_percent}% ($historical_time → $current_time)"
                echo "TREND_SEVERITY=high" >> "$GITHUB_ENV"
            elif (( $(echo "$change_percent > $THRESHOLD_DEGRADATION_PERCENT" | bc -l) )); then
                log_warn "Compilation time increase: ${change_percent}% ($historical_time → $current_time)"
                echo "TREND_SEVERITY=medium" >> "$GITHUB_ENV"
            elif (( $(echo "$change_percent < -$THRESHOLD_DEGRADATION_PERCENT" | bc -l) )); then
                log_info "Compilation time improvement: ${change_percent}% ($historical_time → $current_time)"
                echo "TREND_SEVERITY=improvement" >> "$GITHUB_ENV"
            else
                log_info "Compilation time stable: ${change_percent}% change"
                echo "TREND_SEVERITY=stable" >> "$GITHUB_ENV"
            fi
        fi
    fi
}

# Analyze warning trends
analyze_warning_trends() {
    log_info "Analyzing warning count trends..."

    local current_check_file=""
    local historical_check_file=""

    # Find latest cargo check benchmarks
    if [ -n "$(find "$BENCHMARK_DIR/current" -name "*cargo-check*.json" 2>/dev/null)" ]; then
        current_check_file=$(find "$BENCHMARK_DIR/current" -name "*cargo-check*.json" | head -1)
    fi

    if [ -n "$(find "$BENCHMARK_DIR/historical" -name "*cargo-check*.json" 2>/dev/null)" ]; then
        historical_check_file=$(find "$BENCHMARK_DIR/historical" -name "*cargo-check*.json" | sort -r | head -1)
    fi

    if [ -n "$current_check_file" ] && [ -n "$historical_check_file" ]; then
        current_warnings=$(jq -r '.warning_count // 0' "$current_check_file" 2>/dev/null || echo "0")
        historical_warnings=$(jq -r '.warning_count // 0' "$historical_check_file" 2>/dev/null || echo "0")

        if [ "$historical_warnings" != "0" ]; then
            change_percent=$(echo "scale=2; (($current_warnings - $historical_warnings) / $historical_warnings) * 100" | bc -l 2>/dev/null || echo "0")

            echo "WARNING_TREND=$change_percent" >> "$GITHUB_ENV"
            echo "WARNINGS_CHANGE_PERCENT=$change_percent" >> "$GITHUB_ENV"

            if (( $(echo "$change_percent > 20" | bc -l) )); then
                log_error "Significant warning increase: ${change_percent}% ($historical_warnings → $current_warnings)"
                echo "WARNING_SEVERITY=high" >> "$GITHUB_ENV"
            elif (( $(echo "$change_percent > 10" | bc -l) )); then
                log_warn "Warning increase: ${change_percent}% ($historical_warnings → $current_warnings)"
                echo "WARNING_SEVERITY=medium" >> "$GITHUB_ENV"
            elif (( $(echo "$change_percent < -10" | bc -l) )); then
                log_info "Warning reduction: ${change_percent}% ($historical_warnings → $current_warnings)"
                echo "WARNING_SEVERITY=improvement" >> "$GITHUB_ENV"
            else
                log_info "Warning count stable: ${change_percent}% change"
                echo "WARNING_SEVERITY=stable" >> "$GITHUB_ENV"
            fi
        fi
    fi
}

# Generate performance trend report
generate_trend_report() {
    log_info "Generating performance trend analysis report..."

    cat > "$TREND_REPORT" << EOF
# Performance Trend Analysis Report

**Generated:** $(date '+%Y-%m-%d %H:%M:%S UTC')
**Analysis Period:** Last benchmark run vs previous baseline
**Analysis Script:** $0

## Performance Metrics Overview

### Compilation Performance
EOF

    # Add compilation trend data
    if [ -n "${COMPILATION_CHANGE_PERCENT:-}" ]; then
        echo "- **Compilation Time Change:** ${COMPILATION_CHANGE_PERCENT}%"
        if [ "${TREND_SEVERITY:-}" == "improvement" ]; then
            echo "- **Trend:** 🟢 Improvement detected"
        elif [ "${TREND_SEVERITY:-}" == "high" ]; then
            echo "- **Trend:** 🔴 Major performance degradation"
        elif [ "${TREND_SEVERITY:-}" == "medium" ]; then
            echo "- **Trend:** 🟡 Performance degradation"
        else
            echo "- **Trend:** ⚪ Stable performance"
        fi
    else
        echo "- **Compilation Time Change:** No trend data available"
    fi

    cat >> "$TREND_REPORT" << EOF

### Warning Analysis
EOF

    # Add warning trend data
    if [ -n "${WARNINGS_CHANGE_PERCENT:-}" ]; then
        echo "- **Warning Count Change:** ${WARNINGS_CHANGE_PERCENT}%"
        if [ "${WARNING_SEVERITY:-}" == "improvement" ]; then
            echo "- **Trend:** 🟢 Warning reduction"
        elif [ "${WARNING_SEVERITY:-}" == "high" ]; then
            echo "- **Trend:** 🔴 Major warning increase"
        elif [ "${WARNING_SEVERITY:-}" == "medium" ]; then
            echo "- **Trend:** 🟡 Warning increase"
        else
            echo "- **Trend:** ⚪ Stable warning count"
        fi
    else
        echo "- **Warning Count Change:** No trend data available"
    fi

    cat >> "$TREND_REPORT" << EOF

## Threshold Settings
- **Performance Degradation Threshold:** ${THRESHOLD_DEGRADATION_PERCENT}%
- **Warning Increase Threshold:** 10%
- **Critical Degradation Threshold:** ${THRESHOLD_WARNING_PERCENT}%

## Recommendations

EOF

    if [ "${TREND_SEVERITY:-}" == "high" ] || [ "${WARNING_SEVERITY:-}" == "high" ]; then
        echo "### Critical Issues Requiring Immediate Attention
- 🚨 Performance and/or warning thresholds exceeded
- Investigate recent code changes for performance regressions
- Review build optimizations and dependency updates
- Consider rolling back recent changes if regression is confirmed
        " >> "$TREND_REPORT"
    elif [ "${TREND_SEVERITY:-}" == "medium" ] || [ "${WARNING_SEVERITY:-}" == "medium" ]; then
        echo "### Medium Priority Issues
- ⚠️ Performance degradation or warning increase detected
- Monitor trend in upcoming builds
- Review recent changes for potential optimizations
- Consider updating dependencies if they're causing issues
        " >> "$TREND_REPORT"
    elif [ "${TREND_SEVERITY:-}" == "improvement" ] || [ "${WARNING_SEVERITY:-}" == "improvement" ]; then
        echo "### Positive Trends
- ✅ Performance improvements detected
- Continue monitoring to ensure stability
- Consider documenting successful optimizations
        " >> "$TREND_REPORT"
    else
        echo "### Stable Performance
- ✅ No significant performance changes detected
- Continue regular monitoring
- Performance within acceptable thresholds
        " >> "$TREND_REPORT"
    fi

    cat >> "$TREND_REPORT" << EOF

## Data Sources
- Current benchmark data: ${BENCHMARK_DIR}/current/
- Historical benchmark data: ${BENCHMARK_DIR}/historical/

**Note:** This report is automatically generated. For detailed analysis, review the raw benchmark data files.
EOF

    log_info "Performance trend report saved to: $TREND_REPORT"
}

# Send notifications (placeholder for integration with Slack/Teams/etc)
send_notifications() {
    if [ "${TREND_SEVERITY:-}" == "high" ] || [ "${WARNING_SEVERITY:-}" == "high" ]; then
        log_error "Critical performance regression detected!"
        echo "NOTIFICATION_REQUIRED=true" >> "$GITHUB_ENV"
        echo "NOTIFICATION_SEVERITY=critical" >> "$GITHUB_ENV"
    elif [ "${TREND_SEVERITY:-}" == "medium" ] || [ "${WARNING_SEVERITY:-}" == "medium" ]; then
        log_warn "Performance degradation detected"
        echo "NOTIFICATION_REQUIRED=true" >> "$GITHUB_ENV"
        echo "NOTIFICATION_SEVERITY=warning" >> "$GITHUB_ENV"
    elif [ "${TREND_SEVERITY:-}" == "improvement" ] || [ "${WARNING_SEVERITY:-}" == "improvement" ]; then
        log_info "Performance improvement detected"
        echo "NOTIFICATION_REQUIRED=false" >> "$GITHUB_ENV"
        echo "NOTIFICATION_SEVERITY=info" >> "$GITHUB_ENV"
    else
        echo "NOTIFICATION_REQUIRED=false" >> "$GITHUB_ENV"
        echo "NOTIFICATION_SEVERITY=none" >> "$GITHUB_ENV"
    fi
}

# Main execution flow
main() {
    log_info "Starting performance trend analysis..."

    setup_directories
    collect_current_benchmarks
    archive_historical_data

    analyze_compilation_trends
    analyze_warning_trends

    generate_trend_report
    send_notifications

    log_info "Performance trend analysis complete"
    log_info "Check $TREND_REPORT for detailed results"

    # Export final status
    if [ "${TREND_SEVERITY:-}" == "high" ] || [ "${WARNING_SEVERITY:-}" == "high" ]; then
        exit 1
    fi
}

# Execute main function if script is run directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
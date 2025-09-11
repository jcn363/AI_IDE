#!/bin/bash

# Dependency Health Dashboard Script
# Comprehensive dashboard for dependency health tracking and visualization
# Generates reports and charts for dependency status monitoring
# Author: DevOps Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
DASHBOARD_LOG="${PROJECT_ROOT}/dependency-dashboard.log"
DASHBOARD_DIR="${PROJECT_ROOT}/security-dashboards/dependency-health/$(date +%Y%m%d_%H%M%S)"
START_TIME=$(date +%s)

# Create directories
mkdir -p "${DASHBOARD_DIR}"

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${DASHBOARD_LOG}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${DASHBOARD_LOG}" >&2
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${DASHBOARD_LOG}"
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${DASHBOARD_LOG}"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Comprehensive dependency health dashboard and tracking.

OPTIONS:
    -h, --help                  Show this help message
    -v, --verbose               Enable verbose output
    --output-dir DIR            Output directory for dashboard (default: auto-generated)
    --historical-days NUM       Days of historical data to include (default: 30)
    --format FORMAT             Output format: html|json|markdown (default: html)
    --include-trends            Include trend analysis
    --alert-threshold PCT       Alert threshold percentage (default: 80)

EXAMPLES:
    $0 --verbose --historical-days 90
    $0 --format markdown --include-trends
    $0 --alert-threshold 70

EOF
}

# Parse command line arguments
VERBOSE=false
HISTORICAL_DAYS=30
FORMAT="html"
INCLUDE_TRENDS=false
ALERT_THRESHOLD=80
OUTPUT_DIR="${DASHBOARD_DIR}"

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
        --historical-days)
            HISTORICAL_DAYS="$2"
            shift 2
            ;;
        --format)
            FORMAT="$2"
            shift 2
            ;;
        --include-trends)
            INCLUDE_TRENDS=true
            shift
            ;;
        --alert-threshold)
            ALERT_THRESHOLD="$2"
            shift 2
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Function to collect current dependency health metrics
collect_health_metrics() {
    log_info "Collecting current dependency health metrics..."

    cd "${PROJECT_ROOT}"

    local metrics_report="${OUTPUT_DIR}/health-metrics.json"

    # Security health
    local security_score=100
    local security_issues=0

    if command -v cargo-audit >/dev/null 2>&1; then
        if cargo audit --format json > "${OUTPUT_DIR}/security-scan.json" 2>&1; then
            security_issues=$(jq -r '.vulnerabilities.count // 0' "${OUTPUT_DIR}/security-scan.json" 2>/dev/null || echo "0")
            if [[ "${security_issues}" -gt 0 ]]; then
                security_score=$((100 - (security_issues * 10)))
                [[ "${security_score}" -lt 0 ]] && security_score=0
            fi
        fi
    fi

    # License compliance health
    local license_score=100
    local license_issues=0

    if command -v cargo-deny >/dev/null 2>&1; then
        if cargo deny check licenses --format json > "${OUTPUT_DIR}/license-scan.json" 2>&1; then
            license_issues=$(jq -r '.errors // [] | length' "${OUTPUT_DIR}/license-scan.json" 2>/dev/null || echo "0")
            if [[ "${license_issues}" -gt 0 ]]; then
                license_score=$((100 - (license_issues * 5)))
                [[ "${license_score}" -lt 0 ]] && license_score=0
            fi
        fi
    fi

    # Freshness health
    local freshness_score=100
    local outdated_deps=0

    if command -v cargo-outdated >/dev/null 2>&1; then
        if cargo outdated --format json > "${OUTPUT_DIR}/freshness-scan.json" 2>&1; then
            outdated_deps=$(jq -r '.dependencies // [] | length' "${OUTPUT_DIR}/freshness-scan.json" 2>/dev/null || echo "0")
            if [[ "${outdated_deps}" -gt 50 ]]; then
                freshness_score=50
            elif [[ "${outdated_deps}" -gt 20 ]]; then
                freshness_score=75
            elif [[ "${outdated_deps}" -gt 10 ]]; then
                freshness_score=90
            fi
        fi
    fi

    # Build health
    local build_score=100
    local build_issues=0

    if cargo +nightly check --workspace --message-format json > "${OUTPUT_DIR}/build-scan.json" 2>&1; then
        build_issues=$(grep -c '"level":"error"' "${OUTPUT_DIR}/build-scan.json" 2>/dev/null || echo "0")
        if [[ "${build_issues}" -gt 0 ]]; then
            build_score=$((100 - (build_issues * 2)))
            [[ "${build_score}" -lt 0 ]] && build_score=0
        fi
    else
        build_score=0
        build_issues=$((build_issues + 1))
    fi

    # Calculate overall health score
    local overall_score=$(( (security_score + license_score + freshness_score + build_score) / 4 ))

    # Generate metrics report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg overall_score "$overall_score" \
        --arg security_score "$security_score" \
        --arg security_issues "$security_issues" \
        --arg license_score "$license_score" \
        --arg license_issues "$license_issues" \
        --arg freshness_score "$freshness_score" \
        --arg outdated_deps "$outdated_deps" \
        --arg build_score "$build_score" \
        --arg build_issues "$build_issues" \
        '{
            timestamp: $timestamp,
            collection_type: "current_health_metrics",
            overall_health_score: ($overall_score | tonumber),
            component_scores: {
                security: ($security_score | tonumber),
                license: ($license_score | tonumber),
                freshness: ($freshness_score | tonumber),
                build: ($build_score | tonumber)
            },
            issues_found: {
                security: ($security_issues | tonumber),
                license: ($license_issues | tonumber),
                outdated_dependencies: ($outdated_deps | tonumber),
                build_errors: ($build_issues | tonumber)
            },
            health_status: (if ($overall_score >= 90) then "EXCELLENT" elif ($overall_score >= 75) then "GOOD" elif ($overall_score >= 60) then "FAIR" else "POOR" end),
            alerts: [
                (if ($security_score < 80) then "Security vulnerabilities detected" else "Security posture is good" end),
                (if ($license_score < 80) then "License compliance issues found" else "License compliance is maintained" end),
                (if ($freshness_score < 80) then "Many dependencies are outdated" else "Dependencies are reasonably current" end),
                (if ($build_score < 80) then "Build issues detected" else "Build health is good" end)
            ]
        }' > "${metrics_report}"

    log_info "Health metrics collected. Overall score: ${overall_score}%"
    return $overall_score
}

# Function to collect historical data
collect_historical_data() {
    if [[ "${HISTORICAL_DAYS}" -le 0 ]]; then
        log_info "Skipping historical data collection (--historical-days not set)"
        return 0
    fi

    log_info "Collecting historical data for ${HISTORICAL_DAYS} days..."

    local historical_report="${OUTPUT_DIR}/historical-data.json"
    local historical_data="[]"

    # Find historical reports
    local report_dirs=$(find "${PROJECT_ROOT}/security-reports" -name "dependency-audits" -type d -mtime "-${HISTORICAL_DAYS}" 2>/dev/null || true)

    for dir in $report_dirs; do
        if [[ -f "${dir}/comprehensive-audit-report.json" ]]; then
            local timestamp=$(jq -r '.timestamp // empty' "${dir}/comprehensive-audit-report.json" 2>/dev/null || echo "")
            if [[ -n "$timestamp" ]]; then
                local entry=$(jq -n \
                    --arg timestamp "$timestamp" \
                    --arg path "$dir" \
                    --slurpfile report "${dir}/comprehensive-audit-report.json" \
                    '{timestamp: $timestamp, path: $path, data: $report[0]}' 2>/dev/null || echo "{}")

                if [[ "$entry" != "{}" ]]; then
                    historical_data=$(echo "$historical_data" | jq ". + [$entry]" 2>/dev/null || echo "$historical_data")
                fi
            fi
        fi
    done

    echo "$historical_data" > "${historical_report}"
    log_info "Historical data collected from $(echo "$historical_data" | jq 'length' 2>/dev/null || echo "0") reports"
}

# Function to analyze trends
analyze_trends() {
    if [[ "${INCLUDE_TRENDS}" != true ]]; then
        log_info "Skipping trend analysis (--include-trends not set)"
        return 0
    fi

    log_info "Analyzing dependency health trends..."

    local trends_report="${OUTPUT_DIR}/trend-analysis.json"

    if [[ ! -f "${OUTPUT_DIR}/historical-data.json" ]]; then
        log_warning "No historical data available for trend analysis"
        return 0
    fi

    # Analyze trends over time
    local trend_data=$(jq '
        sort_by(.timestamp) |
        {
            periods: length,
            security_trend: (map(.data.results.security_vulnerabilities // 0) | if length > 1 then (.[-1] - .[0]) else 0 end),
            license_trend: (map(.data.results.license_issues // 0) | if length > 1 then (.[-1] - .[0]) else 0 end),
            freshness_trend: (map(.data.results.outdated_dependencies // 0) | if length > 1 then (.[-1] - .[0]) else 0 end),
            overall_trend: "calculated"
        }
    ' "${OUTPUT_DIR}/historical-data.json" 2>/dev/null || echo "{}")

    # Save trend analysis
    echo "$trend_data" > "${trends_report}"

    log_info "Trend analysis completed"
}

# Function to generate alerts
generate_alerts() {
    log_info "Generating dependency health alerts..."

    local alerts_report="${OUTPUT_DIR}/health-alerts.json"

    if [[ ! -f "${OUTPUT_DIR}/health-metrics.json" ]]; then
        log_warning "No health metrics available for alert generation"
        return 0
    fi

    local alerts="[]"
    local overall_score=$(jq -r '.overall_health_score // 100' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null || echo "100")

    # Generate alerts based on scores and thresholds
    if [[ "$overall_score" -lt "$ALERT_THRESHOLD" ]]; then
        alerts=$(echo "$alerts" | jq '. + [{"level": "CRITICAL", "message": "Overall dependency health score below threshold", "score": '$overall_score', "threshold": '$ALERT_THRESHOLD'}]' 2>/dev/null || echo "$alerts")
    fi

    local security_score=$(jq -r '.component_scores.security // 100' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null || echo "100")
    if [[ "$security_score" -lt 80 ]]; then
        alerts=$(echo "$alerts" | jq '. + [{"level": "HIGH", "message": "Security vulnerabilities detected", "score": '$security_score'}]' 2>/dev/null || echo "$alerts")
    fi

    local license_score=$(jq -r '.component_scores.license // 100' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null || echo "100")
    if [[ "$license_score" -lt 80 ]]; then
        alerts=$(echo "$alerts" | jq '. + [{"level": "HIGH", "message": "License compliance issues found", "score": '$license_score'}]' 2>/dev/null || echo "$alerts")
    fi

    local freshness_score=$(jq -r '.component_scores.freshness // 100' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null || echo "100")
    if [[ "$freshness_score" -lt 70 ]]; then
        alerts=$(echo "$alerts" | jq '. + [{"level": "MEDIUM", "message": "Many dependencies are outdated", "score": '$freshness_score'}]' 2>/dev/null || echo "$alerts")
    fi

    local build_score=$(jq -r '.component_scores.build // 100' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null || echo "100")
    if [[ "$build_score" -lt 80 ]]; then
        alerts=$(echo "$alerts" | jq '. + [{"level": "HIGH", "message": "Build issues detected", "score": '$build_score'}]' 2>/dev/null || echo "$alerts")
    fi

    # Generate alerts report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --argjson alerts "$alerts" \
        --arg alert_count "$(echo "$alerts" | jq 'length' 2>/dev/null || echo "0")" \
        '{
            timestamp: $timestamp,
            alert_type: "dependency_health",
            total_alerts: ($alert_count | tonumber),
            alerts: $alerts,
            recommendations: [
                "Review alerts and address critical issues first",
                "Schedule regular dependency updates",
                "Implement automated dependency monitoring",
                "Consider security audits for high-risk alerts"
            ]
        }' > "${alerts_report}"

    local alert_count=$(echo "$alerts" | jq 'length' 2>/dev/null || echo "0")
    log_info "Generated ${alert_count} dependency health alerts"
}

# Function to generate HTML dashboard
generate_html_dashboard() {
    if [[ "${FORMAT}" != "html" ]]; then
        return 0
    fi

    log_info "Generating HTML dependency health dashboard..."

    local dashboard_html="${OUTPUT_DIR}/dependency-health-dashboard.html"

    if [[ ! -f "${OUTPUT_DIR}/health-metrics.json" ]]; then
        log_warning "No health metrics available for dashboard generation"
        return 0
    fi

    local overall_score=$(jq -r '.overall_health_score // 0' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null || echo "0")
    local security_score=$(jq -r '.component_scores.security // 0' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null || echo "0")
    local license_score=$(jq -r '.component_scores.license // 0' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null || echo "0")
    local freshness_score=$(jq -r '.component_scores.freshness // 0' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null || echo "0")
    local build_score=$(jq -r '.component_scores.build // 0' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null || echo "0")

    local health_status=$(jq -r '.health_status // "UNKNOWN"' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null || echo "UNKNOWN")

    # Generate HTML dashboard
    cat > "${dashboard_html}" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>Rust AI IDE Dependency Health Dashboard</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; background: #f5f5f5; }
        .header { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px; border-radius: 10px; text-align: center; }
        .dashboard { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; margin: 20px 0; }
        .card { background: white; padding: 20px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        .score-circle { width: 120px; height: 120px; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 24px; font-weight: bold; margin: 0 auto; }
        .score-excellent { background: linear-gradient(135deg, #4CAF50, #45a049); color: white; }
        .score-good { background: linear-gradient(135deg, #8BC34A, #7cb342); color: white; }
        .score-fair { background: linear-gradient(135deg, #FFC107, #ffb300); color: black; }
        .score-poor { background: linear-gradient(135deg, #F44336, #e53935); color: white; }
        .alert { background: #fff3cd; border: 1px solid #ffeaa7; padding: 10px; border-radius: 5px; margin: 10px 0; }
        .alert-critical { background: #f8d7da; border: 1px solid #f5c6cb; }
        .metric-bar { display: flex; align-items: center; margin: 10px 0; }
        .metric-label { flex: 1; }
        .metric-value { font-weight: bold; }
        .progress-bar { width: 100%; height: 8px; background: #e0e0e0; border-radius: 4px; overflow: hidden; }
        .progress-fill { height: 100%; border-radius: 4px; transition: width 0.3s; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Rust AI IDE Dependency Health Dashboard</h1>
        <p>Generated: $(date) | Status: ${health_status}</p>
    </div>

    <div class="dashboard">
        <div class="card">
            <h3>Overall Health Score</h3>
            <div class="score-circle score-$(echo "${overall_score}" | awk '{if ($1 >= 90) print "excellent"; else if ($1 >= 75) print "good"; else if ($1 >= 60) print "fair"; else print "poor"}')">
                ${overall_score}%
            </div>
        </div>

        <div class="card">
            <h3>Security Health</h3>
            <div class="metric-bar">
                <span class="metric-label">Security Score</span>
                <span class="metric-value">${security_score}%</span>
            </div>
            <div class="progress-bar">
                <div class="progress-fill" style="width: ${security_score}%; background: ${security_score} >= 80 ? '#4CAF50' : security_score >= 60 ? '#FFC107' : '#F44336'}"></div>
            </div>
        </div>

        <div class="card">
            <h3>License Compliance</h3>
            <div class="metric-bar">
                <span class="metric-label">License Score</span>
                <span class="metric-value">${license_score}%</span>
            </div>
            <div class="progress-bar">
                <div class="progress-fill" style="width: ${license_score}%; background: ${license_score} >= 80 ? '#4CAF50' : license_score >= 60 ? '#FFC107' : '#F44336'}"></div>
            </div>
        </div>

        <div class="card">
            <h3>Freshness Score</h3>
            <div class="metric-bar">
                <span class="metric-label">Freshness Score</span>
                <span class="metric-value">${freshness_score}%</span>
            </div>
            <div class="progress-bar">
                <div class="progress-fill" style="width: ${freshness_score}%; background: ${freshness_score} >= 80 ? '#4CAF50' : freshness_score >= 60 ? '#FFC107' : '#F44336'}"></div>
            </div>
        </div>

        <div class="card">
            <h3>Build Health</h3>
            <div class="metric-bar">
                <span class="metric-label">Build Score</span>
                <span class="metric-value">${build_score}%</span>
            </div>
            <div class="progress-bar">
                <div class="progress-fill" style="width: ${build_score}%; background: ${build_score} >= 80 ? '#4CAF50' : build_score >= 60 ? '#FFC107' : '#F44336'}"></div>
            </div>
        </div>
    </div>

    <div class="card">
        <h3>Alerts & Recommendations</h3>
        $(jq -r '.alerts[]' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null | sed 's/.*/<div class="alert">&<\/div>/' || echo '<div class="alert">No alerts at this time</div>')
    </div>

    <div class="card">
        <h3>Report Files</h3>
        <ul>
            <li><a href="health-metrics.json">Current Health Metrics (JSON)</a></li>
            <li><a href="health-alerts.json">Health Alerts (JSON)</a></li>
            $(if [[ -f "${OUTPUT_DIR}/historical-data.json" ]]; then echo '<li><a href="historical-data.json">Historical Data (JSON)</a></li>'; fi)
            $(if [[ -f "${OUTPUT_DIR}/trend-analysis.json" ]]; then echo '<li><a href="trend-analysis.json">Trend Analysis (JSON)</a></li>'; fi)
        </ul>
    </div>
</body>
</html>
EOF

    log_success "HTML dashboard generated: ${dashboard_html}"
}

# Function to generate Markdown report
generate_markdown_report() {
    if [[ "${FORMAT}" != "markdown" ]]; then
        return 0
    fi

    log_info "Generating Markdown dependency health report..."

    local markdown_report="${OUTPUT_DIR}/dependency-health-report.md"

    if [[ ! -f "${OUTPUT_DIR}/health-metrics.json" ]]; then
        log_warning "No health metrics available for markdown report generation"
        return 0
    fi

    local overall_score=$(jq -r '.overall_health_score // 0' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null || echo "0")
    local security_score=$(jq -r '.component_scores.security // 0' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null || echo "0")
    local license_score=$(jq -r '.component_scores.license // 0' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null || echo "0")
    local freshness_score=$(jq -r '.component_scores.freshness // 0' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null || echo "0")
    local build_score=$(jq -r '.component_scores.build // 0' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null || echo "0")

    # Generate Markdown report
    cat > "${markdown_report}" << EOF
# Rust AI IDE Dependency Health Report

**Generated:** $(date)  
**Overall Health Score:** ${overall_score}%  

## Health Scores

| Component | Score | Status |
|-----------|-------|--------|
| Security | ${security_score}% | $([[ "${security_score}" -ge 80 ]] && echo "✅ Good" || [[ "${security_score}" -ge 60 ]] && echo "⚠️ Fair" || echo "❌ Poor") |
| License | ${license_score}% | $([[ "${license_score}" -ge 80 ]] && echo "✅ Good" || [[ "${license_score}" -ge 60 ]] && echo "⚠️ Fair" || echo "❌ Poor") |
| Freshness | ${freshness_score}% | $([[ "${freshness_score}" -ge 80 ]] && echo "✅ Good" || [[ "${freshness_score}" -ge 60 ]] && echo "⚠️ Fair" || echo "❌ Poor") |
| Build | ${build_score}% | $([[ "${build_score}" -ge 80 ]] && echo "✅ Good" || [[ "${build_score}" -ge 60 ]] && echo "⚠️ Fair" || echo "❌ Poor") |

## Issues Found

$(jq -r '.alerts[]' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null | sed 's/.*/- **Alert:** &/' || echo "- No issues detected")

## Recommendations

$(jq -r '.alerts[]' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null | sed 's/.*/- Review: &/' || echo "- All systems operational")

## Report Files

- [Health Metrics (JSON)](health-metrics.json)
- [Health Alerts (JSON)](health-alerts.json)
$(if [[ -f "${OUTPUT_DIR}/historical-data.json" ]]; then echo "- [Historical Data (JSON)](historical-data.json)"; fi)
$(if [[ -f "${OUTPUT_DIR}/trend-analysis.json" ]]; then echo "- [Trend Analysis (JSON)](trend-analysis.json)"; fi)

---
*This report was generated automatically by the dependency health dashboard script.*
EOF

    log_success "Markdown report generated: ${markdown_report}"
}

# Main function
main() {
    log_info "Starting dependency health dashboard generation"
    log_info "Log file: ${DASHBOARD_LOG}"
    log_info "Dashboard directory: ${OUTPUT_DIR}"

    mkdir -p "${OUTPUT_DIR}"

    # Collect current health metrics
    collect_health_metrics

    # Collect historical data if requested
    collect_historical_data

    # Analyze trends if requested
    analyze_trends

    # Generate alerts
    generate_alerts

    # Generate output in requested format
    case "${FORMAT}" in
        "html")
            generate_html_dashboard
            ;;
        "markdown")
            generate_markdown_report
            ;;
        "json")
            log_info "JSON reports already generated"
            ;;
        *)
            log_warning "Unknown format: ${FORMAT}"
            ;;
    esac

    local end_time=$(date +%s)
    log_info "Dependency health dashboard generation completed in $((end_time - START_TIME)) seconds"

    # Print summary
    if [[ -f "${OUTPUT_DIR}/health-metrics.json" ]]; then
        local overall_score=$(jq -r '.overall_health_score // 0' "${OUTPUT_DIR}/health-metrics.json" 2>/dev/null || echo "0")
        log_success "Dashboard generated successfully. Overall health score: ${overall_score}%"
        log_success "Dashboard available at: ${OUTPUT_DIR}"
    fi
}

# Run main function
main "$@"
#!/bin/bash

# Comprehensive Security Reporting and Dashboard System
# Unified security status reporting and trend analysis for Rust AI IDE
# Author: Security Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REPORT_DIR="${PROJECT_ROOT}/security-reports/comprehensive"
DASHBOARD_DIR="${PROJECT_ROOT}/security-dashboards"
TREND_DIR="${PROJECT_ROOT}/security-trends"
START_TIME=$(date +%s)

# Default configuration
GENERATE_DASHBOARD=true
GENERATE_TRENDS=true
SEND_ALERTS=false
RETENTION_DAYS=30
CRITICAL_THRESHOLD=8
HIGH_THRESHOLD=5

# Create directories
mkdir -p "${REPORT_DIR}"
mkdir -p "${DASHBOARD_DIR}"
mkdir -p "${TREND_DIR}"

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${REPORT_DIR}/reporting.log"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${REPORT_DIR}/reporting.log" >&2
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${REPORT_DIR}/reporting.log"
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${REPORT_DIR}/reporting.log"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Comprehensive security reporting and dashboard generation for Rust AI IDE.

OPTIONS:
    -h, --help                      Show this help message
    -v, --verbose                   Enable verbose output
    --no-dashboard                  Skip dashboard generation
    --no-trends                     Skip trend analysis
    --send-alerts                   Send security alerts
    --retention-days DAYS           Report retention period (default: 30)
    --critical-threshold NUM        Critical vulnerability threshold (default: 8)
    --high-threshold NUM            High vulnerability threshold (default: 5)
    --report-dir DIR                Output directory for reports (default: security-reports/comprehensive)

EXAMPLES:
    $0 --send-alerts --verbose
    $0 --critical-threshold 5 --high-threshold 3
    $0 --no-trends --retention-days 60

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
        --no-dashboard)
            GENERATE_DASHBOARD=false
            shift
            ;;
        --no-trends)
            GENERATE_TRENDS=false
            shift
            ;;
        --send-alerts)
            SEND_ALERTS=true
            shift
            ;;
        --retention-days)
            RETENTION_DAYS="$2"
            shift 2
            ;;
        --critical-threshold)
            CRITICAL_THRESHOLD="$2"
            shift 2
            ;;
        --high-threshold)
            HIGH_THRESHOLD="$2"
            shift 2
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

# Function to collect security reports
collect_security_reports() {
    log_info "Collecting security reports from various sources..."

    local reports_found=0
    local report_data="[]"

    # Find all security report directories
    while IFS= read -r -d '' report_path; do
        if [[ -d "${report_path}" ]]; then
            log_info "Processing report directory: ${report_path}"

            # Collect JSON reports
            while IFS= read -r -d '' json_file; do
                if [[ -f "${json_file}" ]]; then
                    local report_type=$(basename "${json_file}" | sed 's/-report\.json$//' | sed 's/\.json$//')
                    local report_content=$(cat "${json_file}")

                    # Add metadata
                    report_content=$(echo "${report_content}" | jq --arg path "${report_path}" --arg type "${report_type}" \
                        '. + {report_path: $path, report_type: $type, collected_at: "'$(date -Iseconds)'"}')

                    # Append to array
                    report_data=$(echo "${report_data}" | jq --argjson content "${report_content}" '. + [$content]')
                    reports_found=$((reports_found + 1))
                fi
            done < <(find "${report_path}" -name "*.json" -print0)
        fi
    done < <(find "${PROJECT_ROOT}/security-reports" -maxdepth 1 -type d -name "*" -not -name "comprehensive" -print0)

    log_info "Collected ${reports_found} security reports"

    # Save collected reports
    echo "${report_data}" > "${REPORT_DIR}/collected-reports.json"

    return 0
}

# Function to analyze security status
analyze_security_status() {
    log_info "Analyzing overall security status..."

    local collected_reports="${REPORT_DIR}/collected-reports.json"

    if [[ ! -f "${collected_reports}" ]]; then
        log_error "No collected reports found"
        return 1
    fi

    # Calculate security metrics
    local total_reports=$(jq '. | length' "${collected_reports}")
    local passed_reports=$(jq '[.[] | select(.status == "PASSED")] | length' "${collected_reports}")
    local failed_reports=$(jq '[.[] | select(.status == "FAILED")] | length' "${collected_reports}")
    local warning_reports=$(jq '[.[] | select(.status == "WARNING")] | length' "${collected_reports}")

    # Vulnerability analysis
    local total_vulnerabilities=$(jq '[.[] | .vulnerabilities?.total // 0] | add' "${collected_reports}")
    local high_vulnerabilities=$(jq '[.[] | .vulnerabilities?.high // 0] | add' "${collected_reports}")
    local critical_vulnerabilities=$(jq '[.[] | .vulnerabilities?.critical // 0] | add' "${collected_reports}")

    # Performance metrics
    local avg_build_time=$(jq '[.[] | .build_time_seconds // 0] | add / (. | length)' "${collected_reports}")
    local avg_test_time=$(jq '[.[] | .test_time_seconds // 0] | add / (. | length)' "${collected_reports}")

    # Compliance status
    local compliance_passed=$(jq '[.[] | select(.compliance_status == "COMPLIANT" or .compliance_status == "PASSED")] | length' "${collected_reports}")
    local compliance_total=$(jq '[.[] | select(.compliance_status)] | length' "${collected_reports}")

    # Calculate security score
    local security_score=100

    # Deduct points for failed reports
    security_score=$((security_score - (failed_reports * 20)))

    # Deduct points for high/critical vulnerabilities
    security_score=$((security_score - (high_vulnerabilities * 5)))
    security_score=$((security_score - (critical_vulnerabilities * 10)))

    # Deduct points for warnings
    security_score=$((security_score - (warning_reports * 2)))

    # Ensure score doesn't go below 0
    [[ $security_score -lt 0 ]] && security_score=0

    # Generate security status report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg total_reports "$total_reports" \
        --arg passed_reports "$passed_reports" \
        --arg failed_reports "$failed_reports" \
        --arg warning_reports "$warning_reports" \
        --arg total_vulnerabilities "$total_vulnerabilities" \
        --arg high_vulnerabilities "$high_vulnerabilities" \
        --arg critical_vulnerabilities "$critical_vulnerabilities" \
        --arg avg_build_time "${avg_build_time:-0}" \
        --arg avg_test_time "${avg_test_time:-0}" \
        --arg compliance_passed "$compliance_passed" \
        --arg compliance_total "$compliance_total" \
        --arg security_score "$security_score" \
        '{
            timestamp: $timestamp,
            summary: {
                total_reports: ($total_reports | tonumber),
                passed_reports: ($passed_reports | tonumber),
                failed_reports: ($failed_reports | tonumber),
                warning_reports: ($warning_reports | tonumber),
                pass_rate: (if ($total_reports | tonumber) > 0 then (($passed_reports | tonumber) / ($total_reports | tonumber) * 100) else 0 end)
            },
            vulnerabilities: {
                total: ($total_vulnerabilities | tonumber),
                high: ($high_vulnerabilities | tonumber),
                critical: ($critical_vulnerabilities | tonumber)
            },
            performance: {
                avg_build_time_seconds: ($avg_build_time | tonumber),
                avg_test_time_seconds: ($avg_test_time | tonumber)
            },
            compliance: {
                passed: ($compliance_passed | tonumber),
                total: ($compliance_total | tonumber),
                rate: (if ($compliance_total | tonumber) > 0 then (($compliance_passed | tonumber) / ($compliance_total | tonumber) * 100) else 0 end)
            },
            security_score: ($security_score | tonumber),
            risk_level: (
                if ($security_score | tonumber) >= 90 then "LOW"
                elif ($security_score | tonumber) >= 70 then "MEDIUM"
                elif ($security_score | tonumber) >= 50 then "HIGH"
                else "CRITICAL"
                end
            )
        }' > "${REPORT_DIR}/security-status.json"

    log_success "Security status analysis completed. Score: ${security_score}/100"
    return 0
}

# Function to generate trend analysis
generate_trend_analysis() {
    if [[ "${GENERATE_TRENDS}" != true ]]; then
        log_info "Skipping trend analysis"
        return 0
    fi

    log_info "Generating security trend analysis..."

    # Collect historical data
    local historical_data="[]"

    # Find all previous security status reports
    while IFS= read -r -d '' status_file; do
        if [[ -f "${status_file}" ]]; then
            local file_content=$(cat "${status_file}")
            historical_data=$(echo "${historical_data}" | jq --argjson content "${file_content}" '. + [$content]')
        fi
    done < <(find "${TREND_DIR}" -name "security-status-*.json" -print0 | sort -z)

    # Add current status to historical data
    if [[ -f "${REPORT_DIR}/security-status.json" ]]; then
        local current_status=$(cat "${REPORT_DIR}/security-status.json")
        historical_data=$(echo "${historical_data}" | jq --argjson content "${current_status}" '. + [$content]')
    fi

    # Sort by timestamp
    historical_data=$(echo "${historical_data}" | jq 'sort_by(.timestamp)')

    # Calculate trends
    local data_points=$(echo "${historical_data}" | jq '. | length')

    if [[ $data_points -gt 1 ]]; then
        # Calculate 7-day trend
        local recent_scores=$(echo "${historical_data}" | jq -r '.[-7:] | map(.security_score) | .[]' 2>/dev/null || echo "")
        local score_trend="stable"

        if [[ -n "${recent_scores}" ]]; then
            local first_score=$(echo "${recent_scores}" | head -1)
            local last_score=$(echo "${recent_scores}" | tail -1)

            if [[ $(echo "${last_score} > ${first_score}" | bc 2>/dev/null) -eq 1 ]]; then
                score_trend="improving"
            elif [[ $(echo "${last_score} < ${first_score}" | bc 2>/dev/null) -eq 1 ]]; then
                score_trend="declining"
            fi
        fi

        # Vulnerability trends
        local vuln_trend=$(echo "${historical_data}" | jq -r '
            if length > 1 then
                (.[-1].vulnerabilities.total // 0) - (.[-2].vulnerabilities.total // 0)
            else
                0
            end
        ')

        # Generate trend report
        jq -n \
            --arg timestamp "$(date -Iseconds)" \
            --arg data_points "$data_points" \
            --arg score_trend "$score_trend" \
            --arg vuln_trend "$vuln_trend" \
            --argjson historical "$historical_data" \
            '{
                timestamp: $timestamp,
                data_points: ($data_points | tonumber),
                trends: {
                    security_score: $score_trend,
                    vulnerabilities: (if ($vuln_trend | tonumber) > 0 then "increasing" elif ($vuln_trend | tonumber) < 0 then "decreasing" else "stable" end)
                },
                historical_data: $historical
            }' > "${REPORT_DIR}/trend-analysis.json"
    else
        # Generate basic trend report for first run
        jq -n \
            --arg timestamp "$(date -Iseconds)" \
            '{
                timestamp: $timestamp,
                data_points: 1,
                trends: {
                    security_score: "baseline",
                    vulnerabilities: "baseline"
                },
                historical_data: []
            }' > "${REPORT_DIR}/trend-analysis.json"
    fi

    # Save current status for future trend analysis
    if [[ -f "${REPORT_DIR}/security-status.json" ]]; then
        cp "${REPORT_DIR}/security-status.json" "${TREND_DIR}/security-status-$(date +%Y%m%d_%H%M%S).json"
    fi

    log_success "Trend analysis completed"
    return 0
}

# Function to generate security dashboard
generate_security_dashboard() {
    if [[ "${GENERATE_DASHBOARD}" != true ]]; then
        log_info "Skipping dashboard generation"
        return 0
    fi

    log_info "Generating security dashboard..."

    # Create HTML dashboard
    cat > "${DASHBOARD_DIR}/security-dashboard.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Rust AI IDE Security Dashboard</title>
    <style>
        body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; }
        .header { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; border-radius: 10px; margin-bottom: 20px; text-align: center; }
        .metrics-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; margin-bottom: 20px; }
        .metric-card { background: white; padding: 20px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        .metric-title { font-size: 14px; color: #666; margin-bottom: 10px; text-transform: uppercase; }
        .metric-value { font-size: 36px; font-weight: bold; margin-bottom: 5px; }
        .metric-trend { font-size: 14px; }
        .status-good { color: #28a745; }
        .status-warning { color: #ffc107; }
        .status-danger { color: #dc3545; }
        .chart-container { background: white; padding: 20px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); margin-bottom: 20px; }
        .alerts { background: white; padding: 20px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        .alert { padding: 10px; margin: 10px 0; border-left: 4px solid; border-radius: 5px; }
        .alert-critical { border-color: #dc3545; background: #f8d7da; }
        .alert-high { border-color: #fd7e14; background: #fff3cd; }
        .alert-medium { border-color: #ffc107; background: #fff3cd; }
        .alert-low { border-color: #17a2b8; background: #d1ecf1; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔒 Rust AI IDE Security Dashboard</h1>
            <p>Last Updated: <span id="last-updated"></span></p>
        </div>
EOF

    # Add JavaScript for dynamic content
    cat >> "${DASHBOARD_DIR}/security-dashboard.html" << 'EOF'
        <div class="metrics-grid" id="metrics-grid">
            <!-- Metrics will be populated by JavaScript -->
        </div>

        <div class="chart-container">
            <h2>Security Trends</h2>
            <canvas id="trendChart" width="400" height="200"></canvas>
        </div>

        <div class="alerts">
            <h2>Active Security Alerts</h2>
            <div id="alerts-container">
                <!-- Alerts will be populated by JavaScript -->
            </div>
        </div>
    </div>

    <script>
        // Load security data
        async function loadSecurityData() {
            try {
                const response = await fetch('./security-dashboard-data.json');
                const data = await response.json();
                updateDashboard(data);
            } catch (error) {
                console.error('Failed to load security data:', error);
                document.getElementById('metrics-grid').innerHTML = '<div class="metric-card"><p>Failed to load security data</p></div>';
            }
        }

        function updateDashboard(data) {
            updateLastUpdated(data.timestamp);
            updateMetrics(data);
            updateAlerts(data);
        }

        function updateLastUpdated(timestamp) {
            document.getElementById('last-updated').textContent = new Date(timestamp).toLocaleString();
        }

        function updateMetrics(data) {
            const metricsGrid = document.getElementById('metrics-grid');
            const metrics = [
                { title: 'Security Score', value: `${data.security_score}/100`, trend: data.risk_level, class: getScoreClass(data.security_score) },
                { title: 'Total Vulnerabilities', value: data.vulnerabilities.total, trend: '', class: getVulnClass(data.vulnerabilities.total) },
                { title: 'High/Critical Vulns', value: `${data.vulnerabilities.high + data.vulnerabilities.critical}`, trend: '', class: getVulnClass(data.vulnerabilities.high + data.vulnerabilities.critical) },
                { title: 'Test Pass Rate', value: `${Math.round(data.summary.pass_rate)}%`, trend: '', class: getPassRateClass(data.summary.pass_rate) },
                { title: 'Compliance Rate', value: `${Math.round(data.compliance.rate)}%`, trend: '', class: getComplianceClass(data.compliance.rate) }
            ];

            metricsGrid.innerHTML = metrics.map(metric => `
                <div class="metric-card">
                    <div class="metric-title">${metric.title}</div>
                    <div class="metric-value ${metric.class}">${metric.value}</div>
                    <div class="metric-trend">${metric.trend}</div>
                </div>
            `).join('');
        }

        function updateAlerts(data) {
            const alertsContainer = document.getElementById('alerts-container');
            const alerts = [];

            if (data.vulnerabilities.critical > 0) {
                alerts.push({ level: 'critical', message: `${data.vulnerabilities.critical} critical vulnerabilities detected` });
            }
            if (data.vulnerabilities.high > 0) {
                alerts.push({ level: 'high', message: `${data.vulnerabilities.high} high-severity vulnerabilities detected` });
            }
            if (data.security_score < 70) {
                alerts.push({ level: 'high', message: `Security score is low: ${data.security_score}/100` });
            }
            if (data.summary.failed_reports > 0) {
                alerts.push({ level: 'medium', message: `${data.summary.failed_reports} security checks failed` });
            }

            if (alerts.length === 0) {
                alertsContainer.innerHTML = '<p class="alert alert-low">✅ No active security alerts</p>';
            } else {
                alertsContainer.innerHTML = alerts.map(alert => `
                    <div class="alert alert-${alert.level}">
                        <strong>${alert.level.toUpperCase()}:</strong> ${alert.message}
                    </div>
                `).join('');
            }
        }

        function getScoreClass(score) {
            if (score >= 90) return 'status-good';
            if (score >= 70) return 'status-warning';
            return 'status-danger';
        }

        function getVulnClass(count) {
            if (count === 0) return 'status-good';
            if (count <= 5) return 'status-warning';
            return 'status-danger';
        }

        function getPassRateClass(rate) {
            if (rate >= 95) return 'status-good';
            if (rate >= 80) return 'status-warning';
            return 'status-danger';
        }

        function getComplianceClass(rate) {
            if (rate >= 90) return 'status-good';
            if (rate >= 70) return 'status-warning';
            return 'status-danger';
        }

        // Load data on page load
        loadSecurityData();
    </script>
</body>
</html>
EOF

    # Generate dashboard data
    if [[ -f "${REPORT_DIR}/security-status.json" ]]; then
        cp "${REPORT_DIR}/security-status.json" "${DASHBOARD_DIR}/security-dashboard-data.json"
        log_success "Security dashboard generated: ${DASHBOARD_DIR}/security-dashboard.html"
    else
        log_error "Security status data not found for dashboard"
    fi

    return 0
}

# Function to generate alerts
generate_security_alerts() {
    log_info "Generating security alerts..."

    local alerts="[]"

    if [[ -f "${REPORT_DIR}/security-status.json" ]]; then
        local security_score=$(jq -r '.security_score // 0' "${REPORT_DIR}/security-status.json")
        local critical_vulns=$(jq -r '.vulnerabilities.critical // 0' "${REPORT_DIR}/security-status.json")
        local high_vulns=$(jq -r '.vulnerabilities.high // 0' "${REPORT_DIR}/security-status.json")
        local failed_reports=$(jq -r '.summary.failed_reports // 0' "${REPORT_DIR}/security-status.json")

        # Generate alerts based on thresholds
        if [[ $critical_vulns -ge $CRITICAL_THRESHOLD ]]; then
            alerts=$(echo "${alerts}" | jq --arg msg "Critical vulnerability threshold exceeded: ${critical_vulns} critical vulnerabilities found" \
                '. + [{level: "CRITICAL", message: $msg, timestamp: "'$(date -Iseconds)'"}]')
        fi

        if [[ $high_vulns -ge $HIGH_THRESHOLD ]]; then
            alerts=$(echo "${alerts}" | jq --arg msg "High vulnerability threshold exceeded: ${high_vulns} high-severity vulnerabilities found" \
                '. + [{level: "HIGH", message: $msg, timestamp: "'$(date -Iseconds)'"}]')
        fi

        if [[ $security_score -lt 70 ]]; then
            alerts=$(echo "${alerts}" | jq --arg score "$security_score" \
                '. + [{level: "HIGH", message: "Security score is critically low: \($score)/100", timestamp: "'$(date -Iseconds)'"}]')
        fi

        if [[ $failed_reports -gt 0 ]]; then
            alerts=$(echo "${alerts}" | jq --arg failed "$failed_reports" \
                '. + [{level: "MEDIUM", message: "\($failed) security reports failed", timestamp: "'$(date -Iseconds)'"}]')
        fi
    fi

    # Save alerts
    echo "${alerts}" > "${REPORT_DIR}/security-alerts.json"

    local alert_count=$(echo "${alerts}" | jq '. | length')
    log_info "Generated ${alert_count} security alerts"

    return 0
}

# Function to send alerts
send_security_alerts() {
    if [[ "${SEND_ALERTS}" != true ]]; then
        log_info "Skipping alert notifications"
        return 0
    fi

    log_info "Sending security alert notifications..."

    local alerts_file="${REPORT_DIR}/security-alerts.json"

    if [[ ! -f "${alerts_file}" ]]; then
        log_info "No alerts to send"
        return 0
    fi

    local alert_count=$(jq '. | length' "${alerts_file}")

    if [[ $alert_count -gt 0 ]]; then
        log_warning "Sending ${alert_count} security alerts"

        # Here you would integrate with notification systems
        # Examples: email, Slack, Teams, PagerDuty, etc.

        # For now, just log the alerts
        jq -r '.[] | "\(.level): \(.message)"' "${alerts_file}" | while read -r alert; do
            log_warning "ALERT: ${alert}"
        done

        log_info "Security alerts sent (simulation)"
    else
        log_info "No alerts to send"
    fi

    return 0
}

# Function to cleanup old reports
cleanup_old_reports() {
    log_info "Cleaning up old reports (retention: ${RETENTION_DAYS} days)..."

    local deleted_files=0

    # Clean up old reports in comprehensive directory
    while IFS= read -r -d '' old_file; do
        if [[ -f "${old_file}" ]]; then
            rm "${old_file}"
            deleted_files=$((deleted_files + 1))
        fi
    done < <(find "${REPORT_DIR}" -name "*.log" -mtime +${RETENTION_DAYS} -print0 2>/dev/null)

    # Clean up old trend data
    while IFS= read -r -d '' old_file; do
        if [[ -f "${old_file}" ]]; then
            rm "${old_file}"
            deleted_files=$((deleted_files + 1))
        fi
    done < <(find "${TREND_DIR}" -name "*.json" -mtime +${RETENTION_DAYS} -print0 2>/dev/null)

    if [[ $deleted_files -gt 0 ]]; then
        log_info "Cleaned up ${deleted_files} old files"
    else
        log_info "No old files to clean up"
    fi

    return 0
}

# Function to generate executive summary
generate_executive_summary() {
    log_info "Generating executive security summary..."

    if [[ ! -f "${REPORT_DIR}/security-status.json" ]]; then
        log_error "Security status data not found"
        return 1
    fi

    local security_data=$(cat "${REPORT_DIR}/security-status.json")

    # Create executive summary
    cat > "${REPORT_DIR}/executive-summary.md" << EOF
# Rust AI IDE Security Executive Summary

**Report Date:** $(date '+%Y-%m-%d %H:%M:%S')
**Generated By:** Automated Security Reporting System

## Executive Overview

### Security Score: $(echo "${security_data}" | jq -r '.security_score')/100
**Risk Level:** $(echo "${security_data}" | jq -r '.risk_level')

## Key Metrics

| Metric | Value | Status |
|--------|-------|--------|
| Security Score | $(echo "${security_data}" | jq -r '.security_score')/100 | $(echo "${security_data}" | jq -r 'if .security_score >= 90 then "Excellent" elif .security_score >= 70 then "Good" elif .security_score >= 50 then "Needs Attention" else "Critical" end') |
| Test Pass Rate | $(printf "%.1f" $(echo "${security_data}" | jq -r '.summary.pass_rate'))% | $(echo "${security_data}" | jq -r 'if .summary.pass_rate >= 95 then "Excellent" elif .summary.pass_rate >= 80 then "Good" else "Needs Improvement" end') |
| Total Vulnerabilities | $(echo "${security_data}" | jq -r '.vulnerabilities.total') | $(echo "${security_data}" | jq -r 'if .vulnerabilities.total == 0 then "Clean" elif .vulnerabilities.total <= 10 then "Low" elif .vulnerabilities.total <= 50 then "Medium" else "High" end') |
| Critical Vulnerabilities | $(echo "${security_data}" | jq -r '.vulnerabilities.critical') | $(echo "${security_data}" | jq -r 'if .vulnerabilities.critical == 0 then "None" else "Action Required" end') |
| Compliance Rate | $(printf "%.1f" $(echo "${security_data}" | jq -r '.compliance.rate'))% | $(echo "${security_data}" | jq -r 'if .compliance.rate >= 90 then "Compliant" else "Review Required" end') |

## Recommendations

EOF

    # Add recommendations based on security status
    local security_score=$(echo "${security_data}" | jq -r '.security_score')
    local critical_vulns=$(echo "${security_data}" | jq -r '.vulnerabilities.critical')
    local high_vulns=$(echo "${security_data}" | jq -r '.vulnerabilities.high')

    if [[ $security_score -lt 70 ]]; then
        cat >> "${REPORT_DIR}/executive-summary.md" << EOF
### 🚨 Critical Actions Required
- **Immediate Security Review:** Security score is critically low
- **Patch Management:** Address all critical and high-severity vulnerabilities
- **Code Review:** Conduct thorough security code review
- **Dependency Audit:** Review and update all dependencies

EOF
    fi

    if [[ $critical_vulns -gt 0 ]]; then
        cat >> "${REPORT_DIR}/executive-summary.md" << EOF
### 🔥 Critical Vulnerabilities
- **${critical_vulns} critical vulnerabilities** require immediate attention
- Schedule emergency patching within 24 hours
- Consider rolling back recent changes if necessary

EOF
    fi

    if [[ $high_vulns -gt 0 ]]; then
        cat >> "${REPORT_DIR}/executive-summary.md" << EOF
### ⚠️ High Priority Issues
- **${high_vulns} high-severity vulnerabilities** need prompt resolution
- Plan patching within the next development cycle
- Update incident response procedures

EOF
    fi

    cat >> "${REPORT_DIR}/executive-summary.md" << EOF
## Detailed Findings

### Security Test Results
- **Total Tests:** $(echo "${security_data}" | jq -r '.summary.total_reports')
- **Passed:** $(echo "${security_data}" | jq -r '.summary.passed_reports')
- **Failed:** $(echo "${security_data}" | jq -r '.summary.failed_reports')
- **Warnings:** $(echo "${security_data}" | jq -r '.summary.warning_reports')

### Vulnerability Breakdown
- **Critical:** $(echo "${security_data}" | jq -r '.vulnerabilities.critical')
- **High:** $(echo "${security_data}" | jq -r '.vulnerabilities.high')
- **Medium:** $(echo "${security_data}" | jq -r '.vulnerabilities.medium // 0')
- **Low:** $(echo "${security_data}" | jq -r '.vulnerabilities.low // 0')

### Performance Metrics
- **Average Build Time:** $(echo "${security_data}" | jq -r '.performance.avg_build_time_seconds') seconds
- **Average Test Time:** $(echo "${security_data}" | jq -r '.performance.avg_test_time_seconds') seconds

## Next Steps

1. **Review Detailed Reports:** Examine individual security reports for specific findings
2. **Prioritize Fixes:** Address critical and high-severity issues first
3. **Schedule Updates:** Plan dependency updates and security patches
4. **Monitor Trends:** Track security metrics over time
5. **Update Policies:** Review and update security policies as needed

---

*This report was generated automatically by the Rust AI IDE Security Reporting System*
EOF

    log_success "Executive summary generated: ${REPORT_DIR}/executive-summary.md"
    return 0
}

# Main function
main() {
    log_info "Starting comprehensive security reporting"
    log_info "Report directory: ${REPORT_DIR}"
    log_info "Dashboard directory: ${DASHBOARD_DIR}"
    log_info "Trend directory: ${TREND_DIR}"

    mkdir -p "${REPORT_DIR}"

    local exit_code=0

    # Collect and analyze security reports
    collect_security_reports || exit_code=$((exit_code + 1))
    analyze_security_status || exit_code=$((exit_code + 1))

    # Generate trends and dashboard
    generate_trend_analysis || exit_code=$((exit_code + 1))
    generate_security_dashboard || exit_code=$((exit_code + 1))

    # Generate alerts and notifications
    generate_security_alerts || exit_code=$((exit_code + 1))
    send_security_alerts || exit_code=$((exit_code + 1))

    # Generate executive summary
    generate_executive_summary || exit_code=$((exit_code + 1))

    # Cleanup old reports
    cleanup_old_reports || exit_code=$((exit_code + 1))

    local end_time=$(date +%s)
    log_info "Comprehensive security reporting completed in $((end_time - START_TIME)) seconds"

    return $exit_code
}

# Run main function
main "$@"
#!/bin/bash

# Automated Monitoring Setup for Rust AI IDE
# Sets up comprehensive cron jobs for security, performance, and accessibility monitoring

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../../" && pwd)"
WEB_DIR="${PROJECT_ROOT}/web"

LOG_FILE="${PROJECT_ROOT}/monitoring-schedule.log"

log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${LOG_FILE}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${LOG_FILE}" >&2
}

# Monitoring schedules configuration
MONITORING_CONFIG="${SCRIPT_DIR}/monitoring-schedule.conf"

setup_monitoring_config() {
    if [[ ! -f "${MONITORING_CONFIG}" ]]; then
        log_info "Creating monitoring schedule configuration..."
        cat > "${MONITORING_CONFIG}" << 'CONF_EOF'
# Monitoring Schedule Configuration

# Security Scans
DAILY_SECURITY_SCAN="0 6 * * *"
WEEKLY_COMPREHENSIVE_SECURITY="0 7 * * 1"
MONTHLY_DEPENDENCY_AUDIT="0 8 1 * *"
CRITICAL_SECURITY_MONITOR="*/15 * * * *"
RUNTIME_SECURITY_CHECKS="0 */4 * * *"

# Performance Monitoring
DAILY_PERFORMANCE_BASELINE="30 6 * * *"
WEEKLY_PERFORMANCE_REGRESSION="0 9 * * 1"
CONTINUOUS_PERFORMANCE_MONITOR="*/30 * * * *"

# Accessibility Monitoring
DAILY_ACCESSIBILITY_SCAN="0 5 * * *"
WEEKLY_ACCESSIBILITY_AUDIT="0 10 * * 1"

# Automated Reporting
DAILY_SECURITY_REPORT="0 18 * * *"
WEEKLY_PERFORMANCE_REPORT="0 19 * * 1"
MONTHLY_COMPREHENSIVE_REPORT="0 20 1 * *"

# Alert Escalation
CRITICAL_ALERT_CHECK="*/5 * * * *"
HIGH_PRIORITY_ALERT="*/15 * * * *"
MEDIUM_PRIORITY_ALERT="0 */2 * * *"

# Cleanup and Maintenance
DAILY_LOG_ROTATION="0 2 * * *"
WEEKLY_TEMP_CLEANUP="0 3 * * 0"
MONTHLY_ARCHIVE_CLEANUP="0 4 1 * *"
WEEKLY_DEPENDENCY_UPDATES="0 5 * * 1"

# Thresholds for Escalation
CRITICAL_VULN_THRESHOLD=1
HIGH_VULN_THRESHOLD=5
PERFORMANCE_DEGRADATION_THRESHOLD=10
ACCESSIBILITY_FAILURE_THRESHOLD=3

# Notification Recipients
SECURITY_TEAM_EMAIL="security@company.com"
DEVOPS_TEAM_EMAIL="devops@company.com"
DEVELOPMENT_TEAM_EMAIL="dev@company.com"

# Enable/Disable Features
ENABLE_EMAIL_NOTIFICATIONS=true
ENABLE_SLACK_NOTIFICATIONS=false
ENABLE_WEBHOOK_NOTIFICATIONS=true
ENABLE_PAGERDUTY_ESCALATION=false
CONF_EOF
        log_info "Monitoring configuration created: ${MONITORING_CONFIG}"
    fi

    source "${MONITORING_CONFIG}"
}

get_cron_jobs() {
    cat << CRON_EOF
# Rust AI IDE Automated Monitoring Schedule
# Generated on $(date)

# Security Scans
${DAILY_SECURITY_SCAN:-0 6 * * *} cd ${PROJECT_ROOT} && ${SCRIPT_DIR}/../security/vulnerability_scanner.sh >> ${LOG_FILE} 2>&1
${WEEKLY_COMPREHENSIVE_SECURITY:-0 7 * * 1} cd ${PROJECT_ROOT} && ${SCRIPT_DIR}/../security/run_owasp_scan.sh >> ${LOG_FILE} 2>&1
${MONTHLY_DEPENDENCY_AUDIT:-0 8 1 * *} cd ${PROJECT_ROOT} && ${SCRIPT_DIR}/../ci/dependency-audit.sh >> ${LOG_FILE} 2>&1
${CRITICAL_SECURITY_MONITOR:-*/15 * * * *} cd ${PROJECT_ROOT} && ${SCRIPT_DIR}/../ci/security-checks.sh >> ${LOG_FILE} 2>&1
${RUNTIME_SECURITY_CHECKS:-0 */4 * * *} cd ${PROJECT_ROOT} && ${SCRIPT_DIR}/../ci/runtime-security-checks.sh >> ${LOG_FILE} 2>&1

# Performance Monitoring
${DAILY_PERFORMANCE_BASELINE:-30 6 * * *} cd ${PROJECT_ROOT} && cargo run --bin performance_baseline_runner >> ${LOG_FILE} 2>&1
${WEEKLY_PERFORMANCE_REGRESSION:-0 9 * * 1} cd ${PROJECT_ROOT} && ${SCRIPT_DIR}/../ci/performance-regression-detection.sh >> ${LOG_FILE} 2>&1
${CONTINUOUS_PERFORMANCE_MONITOR:-*/30 * * * *} cd ${PROJECT_ROOT} && ${SCRIPT_DIR}/../ci/performance-monitoring-integration.sh >> ${LOG_FILE} 2>&1

# Accessibility Monitoring (Web Frontend)
${DAILY_ACCESSIBILITY_SCAN:-0 5 * * *} cd ${WEB_DIR} && npm run accessibility:scan >> ${LOG_FILE} 2>&1
${WEEKLY_ACCESSIBILITY_AUDIT:-0 10 * * 1} cd ${WEB_DIR} && npm run accessibility:audit >> ${LOG_FILE} 2>&1

# Automated Reporting
${DAILY_SECURITY_REPORT:-0 18 * * *} cd ${PROJECT_ROOT} && python3 ${SCRIPT_DIR}/../security/security_notifications.py --daily-report >> ${LOG_FILE} 2>&1
${WEEKLY_PERFORMANCE_REPORT:-0 19 * * 1} cd ${PROJECT_ROOT} && ${SCRIPT_DIR}/../ci/performance-dashboard-generator.sh >> ${LOG_FILE} 2>&1
${MONTHLY_COMPREHENSIVE_REPORT:-0 20 1 * *} cd ${PROJECT_ROOT} && ${SCRIPT_DIR}/../ci/stakeholder-notifications.sh --monthly >> ${LOG_FILE} 2>&1

# Alert Escalation
${CRITICAL_ALERT_CHECK:-*/5 * * * *} cd ${PROJECT_ROOT} && python3 ${SCRIPT_DIR}/../security/security_notifications.py --alert-level critical >> ${LOG_FILE} 2>&1
${HIGH_PRIORITY_ALERT:-*/15 * * * *} cd ${PROJECT_ROOT} && python3 ${SCRIPT_DIR}/../security/security_notifications.py --alert-level high >> ${LOG_FILE} 2>&1
${MEDIUM_PRIORITY_ALERT:-0 */2 * * *} cd ${PROJECT_ROOT} && python3 ${SCRIPT_DIR}/../security/security_notifications.py --alert-level medium >> ${LOG_FILE} 2>&1

# Cleanup and Maintenance
${DAILY_LOG_ROTATION:-0 2 * * *} cd ${PROJECT_ROOT} && find logs/ -name "*.log" -mtime +7 -delete >> ${LOG_FILE} 2>&1
${WEEKLY_TEMP_CLEANUP:-0 3 * * 0} cd ${PROJECT_ROOT} && find /tmp -name "rust-ai-ide-*" -mtime +1 -delete >> ${LOG_FILE} 2>&1
${MONTHLY_ARCHIVE_CLEANUP:-0 4 1 * *} cd ${PROJECT_ROOT} && find reports/ -name "*.json" -mtime +90 -exec gzip {} \; >> ${LOG_FILE} 2>&1
${WEEKLY_DEPENDENCY_UPDATES:-0 5 * * 1} cd ${PROJECT_ROOT} && ${SCRIPT_DIR}/../ci/dependency-update-automation.sh >> ${LOG_FILE} 2>&1
CRON_EOF
}

install_cron_jobs() {
    log_info "Installing automated monitoring cron jobs..."

    # Backup current crontab
    crontab -l > "${PROJECT_ROOT}/crontab.backup.$(date +%Y%m%d_%H%M%S)" 2>/dev/null || true

    # Get current crontab and remove existing Rust AI IDE monitoring jobs
    CURRENT_CRON=$(crontab -l 2>/dev/null | grep -v "rust-ai-ide\|Rust AI IDE" || true)

    # Add new monitoring jobs
    NEW_CRON_JOBS=$(get_cron_jobs)

    # Combine and install
    {
        echo "${CURRENT_CRON}"
        echo ""
        echo "# Rust AI IDE Monitoring Jobs - Installed $(date)"
        echo "${NEW_CRON_JOBS}"
    } | crontab -

    log_info "Cron jobs installed successfully"
}

verify_cron_jobs() {
    log_info "Verifying cron job installation..."
    INSTALLED_JOBS=$(crontab -l | grep -c "rust-ai-ide\|Rust AI IDE\|vulnerability_scanner\|owasp_scan\|dependency-audit" || true)

    if [[ ${INSTALLED_JOBS} -gt 0 ]]; then
        log_info "Found ${INSTALLED_JOBS} monitoring cron jobs installed"
        crontab -l | grep -E "(rust-ai-ide|Rust AI IDE|vulnerability_scanner|owasp_scan|dependency-audit|performance|accessibility|security_notifications)" | head -10
    else
        log_error "No monitoring cron jobs found after installation"
        exit 1
    fi
}

show_schedule() {
    echo "=== Rust AI IDE Automated Monitoring Schedule ==="
    echo ""
    echo "Security Scans:"
    echo "  Daily Security Scan:         ${DAILY_SECURITY_SCAN:-0 6 * * *}"
    echo "  Weekly Comprehensive:        ${WEEKLY_COMPREHENSIVE_SECURITY:-0 7 * * 1}"
    echo "  Monthly Dependency Audit:    ${MONTHLY_DEPENDENCY_AUDIT:-0 8 1 * *}"
    echo "  Critical Security Monitor:   ${CRITICAL_SECURITY_MONITOR:-*/15 * * * *}"
    echo "  Runtime Security Checks:     ${RUNTIME_SECURITY_CHECKS:-0 */4 * * *}"
    echo ""
    echo "Performance Monitoring:"
    echo "  Daily Performance Baseline:  ${DAILY_PERFORMANCE_BASELINE:-30 6 * * *}"
    echo "  Weekly Regression Check:     ${WEEKLY_PERFORMANCE_REGRESSION:-0 9 * * 1}"
    echo "  Continuous Performance:      ${CONTINUOUS_PERFORMANCE_MONITOR:-*/30 * * * *}"
    echo ""
    echo "Accessibility Monitoring:"
    echo "  Daily Accessibility Scan:    ${DAILY_ACCESSIBILITY_SCAN:-0 5 * * *}"
    echo "  Weekly Accessibility Audit:  ${WEEKLY_ACCESSIBILITY_AUDIT:-0 10 * * 1}"
    echo ""
    echo "Automated Reporting:"
    echo "  Daily Security Report:       ${DAILY_SECURITY_REPORT:-0 18 * * *}"
    echo "  Weekly Performance Report:   ${WEEKLY_PERFORMANCE_REPORT:-0 19 * * 1}"
    echo "  Monthly Comprehensive:       ${MONTHLY_COMPREHENSIVE_REPORT:-0 20 1 * *}"
    echo ""
    echo "Alert Escalation:"
    echo "  Critical Alerts:             ${CRITICAL_ALERT_CHECK:-*/5 * * * *}"
    echo "  High Priority:               ${HIGH_PRIORITY_ALERT:-*/15 * * * *}"
    echo "  Medium Priority:             ${MEDIUM_PRIORITY_ALERT:-0 */2 * * *}"
    echo ""
    echo "Cleanup & Maintenance:"
    echo "  Daily Log Rotation:          ${DAILY_LOG_ROTATION:-0 2 * * *}"
    echo "  Weekly Temp Cleanup:         ${WEEKLY_TEMP_CLEANUP:-0 3 * * 0}"
    echo "  Monthly Archive Cleanup:     ${MONTHLY_ARCHIVE_CLEANUP:-0 4 1 * *}"
    echo "  Weekly Dependency Updates:   ${WEEKLY_DEPENDENCY_UPDATES:-0 5 * * 1}"
    echo ""
    echo "Configuration file: ${MONITORING_CONFIG}"
    echo "Log file: ${LOG_FILE}"
}

usage() {
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Automated Monitoring Setup for Rust AI IDE"
    echo ""
    echo "COMMANDS:"
    echo "    setup           Setup configuration and install cron jobs"
    echo "    install         Install/update cron jobs only"
    echo "    verify          Verify cron jobs are installed"
    echo "    show            Show current monitoring schedule"
    echo "    help            Show this help message"
    echo ""
    echo "EXAMPLES:"
    echo "    $0 setup        # Initial setup with configuration"
    echo "    $0 install      # Update cron jobs"
    echo "    $0 show         # Show schedule"
}

main() {
    setup_monitoring_config

    case "${1:-help}" in
        setup)
            log_info "Setting up automated monitoring system..."
            show_schedule
            install_cron_jobs
            verify_cron_jobs
            ;;
        install)
            install_cron_jobs
            verify_cron_jobs
            ;;
        verify)
            verify_cron_jobs
            ;;
        show)
            show_schedule
            ;;
        help|--help|-h)
            usage
            ;;
        *)
            log_error "Unknown command: $1"
            usage
            exit 1
            ;;
    esac
}

main "$@"
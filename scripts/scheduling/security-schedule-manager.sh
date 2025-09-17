#!/bin/bash

# Security Schedule Manager for Rust AI IDE
# Manages automated security scanning schedules

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
WEB_DIR="${PROJECT_ROOT}/web"

# Configuration
LOG_FILE="${PROJECT_ROOT}/security-schedule.log"
CONFIG_FILE="${SCRIPT_DIR}/security-schedule.conf"

log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${LOG_FILE}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${LOG_FILE}" >&2
}

setup_config() {
    if [[ ! -f "${CONFIG_FILE}" ]]; then
        log_info "Creating default security schedule configuration..."
        cat > "${CONFIG_FILE}" << 'CONF_EOF'
# Security Schedule Configuration
DAILY_SECURITY_SCAN="0 6 * * *"
WEEKLY_SECURITY_AUDIT="0 7 * * 1"
MONTHLY_DEPENDENCY_CHECK="0 8 1 * *"
CRITICAL_ALERT_CHECK="*/30 * * * *"

# Thresholds
CRITICAL_VULN_THRESHOLD=1
HIGH_VULN_THRESHOLD=5
EMAIL_RECIPIENTS="security@company.com,devops@company.com"

# Enable/disable features
ENABLE_WEBHOOKS=true
ENABLE_EMAIL_ALERTS=true
ENABLE_SLACK_NOTIFICATIONS=false
CONF_EOF
        log_info "Configuration file created: ${CONFIG_FILE}"
    fi
    
    source "${CONFIG_FILE}"
}

show_schedule() {
    echo "=== Rust AI IDE Security Schedule ==="
    echo ""
    echo "Daily Security Scan:     ${DAILY_SECURITY_SCAN:-Not set}"
    echo "Weekly Security Audit:   ${WEEKLY_SECURITY_AUDIT:-Not set}"
    echo "Monthly Dependency Check: ${MONTHLY_DEPENDENCY_CHECK:-Not set}"
    echo "Critical Alert Monitor:  ${CRITICAL_ALERT_CHECK:-Not set}"
    echo ""
    echo "Configuration file: ${CONFIG_FILE}"
    echo "Log file: ${LOG_FILE}"
}

usage() {
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Security Schedule Manager for Rust AI IDE"
    echo ""
    echo "COMMANDS:"
    echo "    setup           Setup configuration and cron jobs"
    echo "    daily           Run daily security scan"
    echo "    weekly          Run weekly comprehensive audit"
    echo "    monthly         Run monthly dependency check"
    echo "    monitor         Run critical alert monitor"
    echo "    show            Show current schedule configuration"
    echo "    help            Show this help message"
    echo ""
    echo "EXAMPLES:"
    echo "    $0 setup        # Initial setup"
    echo "    $0 daily        # Manual daily scan"
    echo "    $0 show         # Show schedule"
}

main() {
    setup_config
    
    case "${1:-help}" in
        setup)
            log_info "Setting up security automation..."
            show_schedule
            ;;
        daily)
            log_info "Running daily security scan..."
            cd "${WEB_DIR}"
            npm run security:daily || log_error "Daily scan failed"
            ;;
        weekly)
            log_info "Running weekly security audit..."
            cd "${WEB_DIR}"
            npm run security:weekly || log_error "Weekly audit failed"
            ;;
        monthly)
            log_info "Running monthly dependency check..."
            cd "${WEB_DIR}"
            npm run security:license-check || log_error "Monthly check failed"
            ;;
        monitor)
            log_info "Running critical alert monitor..."
            cd "${WEB_DIR}"
            npm run security:audit:check || log_error "Critical monitor failed"
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

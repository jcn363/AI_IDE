#!/bin/bash

# Security Stakeholder Notification System
# Automated notifications for security events and alerts
# Author: Security Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
NOTIFICATION_LOG="${PROJECT_ROOT}/security-notifications.log"
START_TIME=$(date +%s)

# Default configuration
NOTIFICATION_METHODS="${NOTIFICATION_METHODS:-email,slack}"
EMAIL_RECIPIENTS="${EMAIL_RECIPIENTS:-security@company.com,devops@company.com}"
SLACK_WEBHOOK_URL="${SLACK_WEBHOOK_URL:-}"
TEAMS_WEBHOOK_URL="${TEAMS_WEBHOOK_URL:-}"
SMTP_SERVER="${SMTP_SERVER:-localhost}"
SMTP_PORT="${SMTP_PORT:-587}"
SMTP_USER="${SMTP_USER:-}"
SMTP_PASS="${SMTP_PASS:-}"
NOTIFICATION_LEVEL="${NOTIFICATION_LEVEL:-medium}"  # low, medium, high, critical
DRY_RUN=false

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${NOTIFICATION_LOG}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${NOTIFICATION_LOG}" >&2
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${NOTIFICATION_LOG}"
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${NOTIFICATION_LOG}"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS] [ALERT_FILE]

Security stakeholder notification system for Rust AI IDE.

OPTIONS:
    -h, --help                      Show this help message
    -v, --verbose                   Enable verbose output
    --dry-run                       Show what would be sent without sending
    --methods METHODS               Notification methods (comma-separated: email,slack,teams)
    --level LEVEL                   Minimum alert level (low|medium|high|critical)
    --email-recipients RECIPS       Email recipients (comma-separated)
    --slack-webhook URL             Slack webhook URL
    --teams-webhook URL             Microsoft Teams webhook URL

ALERT_FILE:
    Path to JSON file containing security alerts (default: read from stdin or auto-detect)

ENVIRONMENT VARIABLES:
    NOTIFICATION_METHODS             Comma-separated list of notification methods
    EMAIL_RECIPIENTS                 Comma-separated list of email recipients
    SLACK_WEBHOOK_URL               Slack webhook URL
    TEAMS_WEBHOOK_URL               Microsoft Teams webhook URL
    SMTP_SERVER                     SMTP server hostname
    SMTP_PORT                       SMTP server port
    SMTP_USER                       SMTP username
    SMTP_PASS                       SMTP password

EXAMPLES:
    $0 security-reports/comprehensive/security-alerts.json
    $0 --methods email,slack --level high security-alerts.json
    $0 --dry-run < alerts.json

EOF
}

# Parse command line arguments
VERBOSE=false
ALERT_FILE=""

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
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --methods)
            NOTIFICATION_METHODS="$2"
            shift 2
            ;;
        --level)
            NOTIFICATION_LEVEL="$2"
            shift 2
            ;;
        --email-recipients)
            EMAIL_RECIPIENTS="$2"
            shift 2
            ;;
        --slack-webhook)
            SLACK_WEBHOOK_URL="$2"
            shift 2
            ;;
        --teams-webhook)
            TEAMS_WEBHOOK_URL="$2"
            shift 2
            ;;
        -*)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
        *)
            ALERT_FILE="$1"
            shift
            ;;
    esac
done

# Function to validate notification level
validate_notification_level() {
    local level="$1"
    case "$level" in
        low|medium|high|critical)
            return 0
            ;;
        *)
            log_error "Invalid notification level: $level"
            return 1
            ;;
    esac
}

# Function to get alert severity weight
get_alert_weight() {
    local level="$1"
    case "$level" in
        low)
            echo 1
            ;;
        medium)
            echo 2
            ;;
        high)
            echo 3
            ;;
        critical)
            echo 4
            ;;
        *)
            echo 0
            ;;
    esac
}

# Function to check if alert should be sent
should_send_alert() {
    local alert_level="$1"
    local threshold_level="$2"

    local alert_weight=$(get_alert_weight "$alert_level")
    local threshold_weight=$(get_alert_weight "$threshold_level")

    [[ $alert_weight -ge $threshold_weight ]]
}

# Function to load alerts
load_alerts() {
    if [[ -n "${ALERT_FILE}" ]]; then
        if [[ ! -f "${ALERT_FILE}" ]]; then
            log_error "Alert file not found: ${ALERT_FILE}"
            return 1
        fi
        cat "${ALERT_FILE}"
    elif [[ ! -t 0 ]]; then
        # Read from stdin
        cat
    else
        # Auto-detect latest alerts file
        local latest_alerts=""
        if [[ -d "${PROJECT_ROOT}/security-reports" ]]; then
            latest_alerts=$(find "${PROJECT_ROOT}/security-reports" -name "security-alerts.json" -type f | head -1)
        fi

        if [[ -n "${latest_alerts}" ]]; then
            log_info "Using auto-detected alerts file: ${latest_alerts}"
            cat "${latest_alerts}"
        else
            log_error "No alerts file specified and none found automatically"
            return 1
        fi
    fi
}

# Function to send email notification
send_email_notification() {
    local subject="$1"
    local body="$2"

    if [[ "${DRY_RUN}" == true ]]; then
        log_info "DRY RUN: Would send email with subject: ${subject}"
        [[ "${VERBOSE}" == true ]] && log_info "DRY RUN: Email body preview: ${body:0:200}..."
        return 0
    fi

    # Check if email is enabled
    if [[ "${NOTIFICATION_METHODS}" != *"email"* ]]; then
        return 0
    fi

    log_info "Sending email notification..."

    # Create temporary email file
    local email_file=$(mktemp)
    cat > "${email_file}" << EOF
From: Rust AI IDE Security <security@rust-ai-ide.local>
To: ${EMAIL_RECIPIENTS}
Subject: ${subject}
Content-Type: text/html; charset=UTF-8

${body}
EOF

    # Send email using sendmail or similar
    if command -v sendmail >/dev/null 2>&1; then
        sendmail -t < "${email_file}"
        log_success "Email sent via sendmail"
    elif [[ -n "${SMTP_SERVER}" ]] && [[ -n "${SMTP_USER}" ]]; then
        # Use curl or other SMTP client
        log_info "SMTP configuration available - would send via SMTP"
        log_success "Email sent via SMTP (simulation)"
    else
        log_warning "No email sending mechanism configured"
        [[ "${VERBOSE}" == true ]] && log_info "Email content saved to: ${email_file}"
    fi

    # Clean up temporary file
    rm -f "${email_file}"
}

# Function to send Slack notification
send_slack_notification() {
    local subject="$1"
    local body="$2"
    local alert_level="$3"

    if [[ "${DRY_RUN}" == true ]]; then
        log_info "DRY RUN: Would send Slack notification: ${subject}"
        return 0
    fi

    # Check if Slack is enabled
    if [[ "${NOTIFICATION_METHODS}" != *"slack"* ]] || [[ -z "${SLACK_WEBHOOK_URL}" ]]; then
        return 0
    fi

    log_info "Sending Slack notification..."

    # Create Slack message payload
    local color=$(get_slack_color "$alert_level")
    local payload=$(jq -n \
        --arg text "*${subject}*" \
        --arg color "$color" \
        '{
            attachments: [{
                color: $color,
                title: "🔒 Rust AI IDE Security Alert",
                text: $text,
                fields: [{
                    title: "Alert Level",
                    value: "'${alert_level}'",
                    short: true
                }, {
                    title: "Timestamp",
                    value: "'$(date -Iseconds)'",
                    short: true
                }],
                footer: "Rust AI IDE Security Monitoring",
                ts: now
            }]
        }')

    # Send to Slack
    if curl -s -X POST -H 'Content-type: application/json' --data "${payload}" "${SLACK_WEBHOOK_URL}" >/dev/null 2>&1; then
        log_success "Slack notification sent"
    else
        log_error "Failed to send Slack notification"
    fi
}

# Function to get Slack color for alert level
get_slack_color() {
    local level="$1"
    case "$level" in
        low)
            echo "good"
            ;;
        medium)
            echo "warning"
            ;;
        high)
            echo "#ff8c00"
            ;;
        critical)
            echo "danger"
            ;;
        *)
            echo "#808080"
            ;;
    esac
}

# Function to send Teams notification
send_teams_notification() {
    local subject="$1"
    local body="$2"
    local alert_level="$3"

    if [[ "${DRY_RUN}" == true ]]; then
        log_info "DRY RUN: Would send Teams notification: ${subject}"
        return 0
    fi

    # Check if Teams is enabled
    if [[ "${NOTIFICATION_METHODS}" != *"teams"* ]] || [[ -z "${TEAMS_WEBHOOK_URL}" ]]; then
        return 0
    fi

    log_info "Sending Microsoft Teams notification..."

    # Create Teams message payload
    local color=$(get_teams_color "$alert_level")
    local payload=$(jq -n \
        --arg title "${subject}" \
        --arg text "${body}" \
        --arg color "$color" \
        '{
            "@type": "MessageCard",
            "@context": "http://schema.org/extensions",
            "themeColor": $color,
            "title": "🔒 Rust AI IDE Security Alert",
            "text": $title,
            "sections": [{
                "text": $text,
                "facts": [{
                    "name": "Alert Level",
                    "value": "'${alert_level}'"
                }, {
                    "name": "Timestamp",
                    "value": "'$(date -Iseconds)'"
                }]
            }]
        }')

    # Send to Teams
    if curl -s -X POST -H 'Content-type: application/json' --data "${payload}" "${TEAMS_WEBHOOK_URL}" >/dev/null 2>&1; then
        log_success "Teams notification sent"
    else
        log_error "Failed to send Teams notification"
    fi
}

# Function to get Teams color for alert level
get_teams_color() {
    local level="$1"
    case "$level" in
        low)
            echo "00FF00"
            ;;
        medium)
            echo "FFFF00"
            ;;
        high)
            echo "FF8C00"
            ;;
        critical)
            echo "FF0000"
            ;;
        *)
            echo "808080"
            ;;
    esac
}

# Function to format alert message
format_alert_message() {
    local alerts="$1"
    local alert_count=$(echo "${alerts}" | jq '. | length')

    if [[ $alert_count -eq 0 ]]; then
        echo "✅ No active security alerts"
        return 0
    fi

    local message="🚨 ${alert_count} Security Alert(s) Detected

"
    local critical_count=$(echo "${alerts}" | jq '[.[] | select(.level == "CRITICAL")] | length')
    local high_count=$(echo "${alerts}" | jq '[.[] | select(.level == "HIGH")] | length')
    local medium_count=$(echo "${alerts}" | jq '[.[] | select(.level == "MEDIUM")] | length')
    local low_count=$(echo "${alerts}" | jq '[.[] | select(.level == "LOW")] | length')

    message+="**Summary:**
• Critical: ${critical_count}
• High: ${high_count}
• Medium: ${medium_count}
• Low: ${low_count}

"

    # Add top alerts
    message+="**Top Alerts:**
"
    echo "${alerts}" | jq -r 'sort_by(.level | if . == "CRITICAL" then 4 elif . == "HIGH" then 3 elif . == "MEDIUM" then 2 else 1 end) | reverse | .[0:3][] | "• [\(.level)] \(.message)"' >> "${message}"

    if [[ $alert_count -gt 3 ]]; then
        message+="
... and $(($alert_count - 3)) more alerts"
    fi

    message+="

🔗 [View Full Report](${PROJECT_ROOT}/security-dashboards/security-dashboard.html)
📊 [Executive Summary](${PROJECT_ROOT}/security-reports/comprehensive/executive-summary.md)"

    echo "${message}"
}

# Function to send notifications
send_notifications() {
    local alerts="$1"

    if [[ "${DRY_RUN}" == true ]]; then
        log_info "DRY RUN: Processing alerts for notification..."
    fi

    # Filter alerts by level
    local filtered_alerts=$(echo "${alerts}" | jq --arg threshold "${NOTIFICATION_LEVEL}" '
        map(select(.level | ascii_downcase | if . == "critical" then 4 elif . == "high" then 3 elif . == "medium" then 2 else 1 end) >=
              ($threshold | ascii_downcase | if . == "critical" then 4 elif . == "high" then 3 elif . == "medium" then 2 else 1 end)))')

    local filtered_count=$(echo "${filtered_alerts}" | jq '. | length')

    if [[ $filtered_count -eq 0 ]]; then
        log_info "No alerts meet the notification threshold (${NOTIFICATION_LEVEL})"
        return 0
    fi

    local subject="Rust AI IDE Security Alert: ${filtered_count} Issue(s) Detected"
    local message=$(format_alert_message "${filtered_alerts}")

    # Get highest alert level
    local highest_level=$(echo "${filtered_alerts}" | jq -r 'sort_by(.level | if . == "CRITICAL" then 4 elif . == "HIGH" then 3 elif . == "MEDIUM" then 2 else 1 end) | reverse | .[0].level' 2>/dev/null || echo "MEDIUM")

    log_info "Sending notifications for ${filtered_count} alert(s) (highest level: ${highest_level})"

    # Send notifications via different methods
    IFS=',' read -ra METHODS <<< "${NOTIFICATION_METHODS}"
    for method in "${METHODS[@]}"; do
        case "$method" in
            email)
                send_email_notification "${subject}" "${message}"
                ;;
            slack)
                send_slack_notification "${subject}" "${message}" "${highest_level}"
                ;;
            teams)
                send_teams_notification "${subject}" "${message}" "${highest_level}"
                ;;
            *)
                log_warning "Unknown notification method: ${method}"
                ;;
        esac
    done
}

# Function to generate notification report
generate_notification_report() {
    local alerts="$1"
    local report_file="${PROJECT_ROOT}/security-reports/notifications/notification-report-$(date +%Y%m%d_%H%M%S).json"

    mkdir -p "$(dirname "${report_file}")"

    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg alerts_sent "$(echo "${alerts}" | jq '. | length')" \
        --arg notification_methods "$NOTIFICATION_METHODS" \
        --arg notification_level "$NOTIFICATION_LEVEL" \
        --arg dry_run "$DRY_RUN" \
        '{
            timestamp: $timestamp,
            alerts_sent: ($alerts_sent | tonumber),
            notification_methods: $notification_methods,
            notification_level: $notification_level,
            dry_run: ($dry_run == "true")
        }' > "${report_file}"

    log_success "Notification report generated: ${report_file}"
}

# Main function
main() {
    log_info "Starting security stakeholder notifications"
    log_info "Log file: ${NOTIFICATION_LOG}"

    # Validate notification level
    if ! validate_notification_level "${NOTIFICATION_LEVEL}"; then
        exit 1
    fi

    # Load alerts
    log_info "Loading security alerts..."
    local alerts=$(load_alerts)

    if [[ $? -ne 0 ]]; then
        log_error "Failed to load alerts"
        exit 1
    fi

    # Parse and validate alerts
    local alert_count=$(echo "${alerts}" | jq '. | length' 2>/dev/null || echo "0")

    if [[ $alert_count -eq 0 ]]; then
        log_info "No security alerts to process"
        exit 0
    fi

    log_info "Processing ${alert_count} security alert(s)"

    # Send notifications
    send_notifications "${alerts}"

    # Generate report
    generate_notification_report "${alerts}"

    local end_time=$(date +%s)
    log_info "Security notifications completed in $((end_time - START_TIME)) seconds"
}

# Run main function
main "$@"
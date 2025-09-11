#!/bin/bash
# Stakeholder Notification System
# Multi-channel notifications (Slack, Email, Webhooks) with customizable templates

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
LOG_FILE="${PROJECT_ROOT}/logs/notifications-$(date +%Y%m%d-%H%M%S).log"
TEMPLATE_DIR="${SCRIPT_DIR}/notification-templates"

# Notification channels
SLACK_WEBHOOK="${SLACK_WEBHOOK:-}"
EMAIL_SMTP_SERVER="${EMAIL_SMTP_SERVER:-}"
EMAIL_SMTP_PORT="${EMAIL_SMTP_PORT:-587}"
EMAIL_USERNAME="${EMAIL_USERNAME:-}"
EMAIL_PASSWORD="${EMAIL_PASSWORD:-}"
EMAIL_FROM="${EMAIL_FROM:-ci@rust-ai-ide.dev}"
WEBHOOK_URL="${WEBHOOK_URL:-}"

# Recipients
SLACK_CHANNEL="${SLACK_CHANNEL:-#general}"
EMAIL_RECIPIENTS="${EMAIL_RECIPIENTS:-}"

# Logging function
log() {
    local level="$1"
    local message="$2"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[${timestamp}] [${level}] ${message}" | tee -a "${LOG_FILE}"
}

# Error handling
error_exit() {
    local message="$1"
    log "ERROR" "${message}"
    exit 1
}

# Create template directory if it doesn't exist
mkdir -p "${TEMPLATE_DIR}"

# Function to load notification template
load_template() {
    local template_name="$1"
    local template_file="${TEMPLATE_DIR}/${template_name}.json"

    if [[ -f "${template_file}" ]]; then
        cat "${template_file}"
    else
        log "WARN" "Template ${template_name} not found, using default"
        echo "{}"
    fi
}

# Function to render template with variables
render_template() {
    local template="$1"
    local variables="$2"

    # Simple variable replacement
    local result="$template"
    for var in $(echo "$variables" | jq -r 'keys[]' 2>/dev/null); do
        local value
        value=$(echo "$variables" | jq -r ".${var}")
        result=$(echo "$result" | sed "s/{{${var}}}/${value}/g")
    done

    echo "$result"
}

# Function to send Slack notification
send_slack_notification() {
    local message="$1"
    local webhook="${SLACK_WEBHOOK}"

    if [[ -n "${webhook}" ]]; then
        log "INFO" "Sending Slack notification"

        # Format for Slack
        local payload="{\"channel\":\"${SLACK_CHANNEL}\",\"text\":\"${message}\",\"username\":\"Rust AI IDE CI\",\"icon_emoji\":\":robot_face:\"}"

        if curl -s -X POST "${webhook}" \
            -H 'Content-type: application/json' \
            --data "${payload}"; then
            log "INFO" "Slack notification sent successfully"
        else
            log "WARN" "Failed to send Slack notification"
        fi
    else
        log "INFO" "Slack webhook not configured, skipping"
    fi
}

# Function to send email notification
send_email_notification() {
    local subject="$1"
    local body="$2"
    local recipients="${EMAIL_RECIPIENTS}"

    if [[ -n "${EMAIL_SMTP_SERVER}" && -n "${recipients}" ]]; then
        log "INFO" "Sending email notification to: ${recipients}"

        # Create email content
        local email_content="From: ${EMAIL_FROM}
To: ${recipients}
Subject: ${subject}
Content-Type: text/html; charset=UTF-8

${body}"

        # Send email using curl (assuming SMTP server supports it)
        if echo "${email_content}" | curl -s --ssl-reqd \
            --url "smtp://${EMAIL_SMTP_SERVER}:${EMAIL_SMTP_PORT}" \
            --user "${EMAIL_USERNAME}:${EMAIL_PASSWORD}" \
            --mail-from "${EMAIL_FROM}" \
            --mail-rcpt "${recipients}" \
            --upload-file -; then
            log "INFO" "Email notification sent successfully"
        else
            log "WARN" "Failed to send email notification"
        fi
    else
        log "INFO" "Email configuration incomplete, skipping"
    fi
}

# Function to send webhook notification
send_webhook_notification() {
    local payload="$1"
    local webhook="${WEBHOOK_URL}"

    if [[ -n "${webhook}" ]]; then
        log "INFO" "Sending webhook notification"

        if curl -s -X POST "${webhook}" \
            -H 'Content-type: application/json' \
            --data "${payload}"; then
            log "INFO" "Webhook notification sent successfully"
        else
            log "WARN" "Failed to send webhook notification"
        fi
    else
        log "INFO" "Webhook URL not configured, skipping"
    fi
}

# Function to create notification payload
create_notification_payload() {
    local event_type="$1"
    local status="$2"
    local details="$3"

    local payload="{
        \"event_type\": \"${event_type}\",
        \"status\": \"${status}\",
        \"timestamp\": \"$(date -Iseconds)\",
        \"project\": \"rust-ai-ide\",
        \"environment\": \"${CI_ENVIRONMENT:-production}\",
        \"details\": ${details},
        \"metadata\": {
            \"git_commit\": \"${GIT_COMMIT:-unknown}\",
            \"git_branch\": \"${GIT_BRANCH:-unknown}\",
            \"build_number\": \"${BUILD_NUMBER:-unknown}\",
            \"pipeline_url\": \"${CI_PIPELINE_URL:-}\"
        }
    }"

    echo "${payload}"
}

# Function to format message for different channels
format_message() {
    local event_type="$1"
    local status="$2"
    local details="$3"
    local channel="$4"

    case "${channel}" in
        "slack")
            case "${event_type}" in
                "documentation_update")
                    if [[ "${status}" == "success" ]]; then
                        echo "✅ *Documentation Updated Successfully*\n${details}"
                    else
                        echo "❌ *Documentation Update Failed*\n${details}"
                    fi
                    ;;
                "review_complete")
                    if [[ "${status}" == "success" ]]; then
                        echo "✅ *Code Review Completed*\n${details}"
                    else
                        echo "⚠️ *Code Review Issues Found*\n${details}"
                    fi
                    ;;
                "deployment")
                    if [[ "${status}" == "success" ]]; then
                        echo "🚀 *Deployment Successful*\n${details}"
                    else
                        echo "💥 *Deployment Failed*\n${details}"
                    fi
                    ;;
                *)
                    echo "📢 *${event_type}* - ${status}\n${details}"
                    ;;
            esac
            ;;
        "email")
            cat << EOF
<html>
<body>
    <h2>Rust AI IDE - ${event_type}</h2>
    <p><strong>Status:</strong> ${status}</p>
    <p><strong>Time:</strong> $(date)</p>
    <div style="background-color: #f5f5f5; padding: 10px; border-radius: 5px;">
        <pre>${details}</pre>
    </div>
    <hr>
    <p><small>This is an automated notification from the Rust AI IDE CI/CD pipeline.</small></p>
</body>
</html>
EOF
            ;;
        *)
            echo "${details}"
            ;;
    esac
}

# Function to notify stakeholders
notify_stakeholders() {
    local event_type="$1"
    local status="$2"
    local details="$3"
    local custom_template="${4:-}"

    log "INFO" "Sending ${event_type} notification with status: ${status}"

    # Load custom template if provided
    local template="{}"
    if [[ -n "${custom_template}" ]]; then
        template=$(load_template "${custom_template}")
    fi

    # Create notification payload
    local payload
    payload=$(create_notification_payload "${event_type}" "${status}" "${details}")

    # Render template if custom template provided
    if [[ "${template}" != "{}" ]]; then
        payload=$(render_template "${template}" "${payload}")
    fi

    # Send to all configured channels
    local slack_message
    slack_message=$(format_message "${event_type}" "${status}" "${details}" "slack")
    send_slack_notification "${slack_message}"

    local email_subject="Rust AI IDE - ${event_type} (${status})"
    local email_body
    email_body=$(format_message "${event_type}" "${status}" "${details}" "email")
    send_email_notification "${email_subject}" "${email_body}"

    send_webhook_notification "${payload}"

    log "INFO" "Notifications sent to all configured channels"
}

# Function to create default templates
create_default_templates() {
    log "INFO" "Creating default notification templates"

    # Documentation update template
    cat > "${TEMPLATE_DIR}/documentation_update.json" << 'EOF'
{
    "slack_template": "📚 *Documentation Update*\nStatus: {{status}}\nDetails: {{details}}\nTime: {{timestamp}}",
    "email_template": "<h3>Documentation Update</h3><p>Status: {{status}}</p><p>Details: {{details}}</p>",
    "webhook_template": {
        "event": "documentation_update",
        "status": "{{status}}",
        "details": "{{details}}",
        "timestamp": "{{timestamp}}"
    }
}
EOF

    # Review completion template
    cat > "${TEMPLATE_DIR}/review_complete.json" << 'EOF'
{
    "slack_template": "🔍 *Code Review Complete*\nStatus: {{status}}\nReviewer: {{reviewer}}\nDetails: {{details}}\nTime: {{timestamp}}",
    "email_template": "<h3>Code Review Complete</h3><p>Status: {{status}}</p><p>Reviewer: {{reviewer}}</p><p>Details: {{details}}</p>",
    "webhook_template": {
        "event": "review_complete",
        "status": "{{status}}",
        "reviewer": "{{reviewer}}",
        "details": "{{details}}",
        "timestamp": "{{timestamp}}"
    }
}
EOF

    # Deployment template
    cat > "${TEMPLATE_DIR}/deployment.json" << 'EOF'
{
    "slack_template": "🚀 *Deployment {{status}}*\nEnvironment: {{environment}}\nVersion: {{version}}\nDetails: {{details}}\nTime: {{timestamp}}",
    "email_template": "<h3>Deployment {{status}}</h3><p>Environment: {{environment}}</p><p>Version: {{version}}</p><p>Details: {{details}}</p>",
    "webhook_template": {
        "event": "deployment",
        "status": "{{status}}",
        "environment": "{{environment}}",
        "version": "{{version}}",
        "details": "{{details}}",
        "timestamp": "{{timestamp}}"
    }
}
EOF

    log "INFO" "Default templates created in ${TEMPLATE_DIR}"
}

# Function to list available templates
list_templates() {
    log "INFO" "Available notification templates:"
    if [[ -d "${TEMPLATE_DIR}" ]]; then
        find "${TEMPLATE_DIR}" -name "*.json" -exec basename {} \; | sed 's/\.json$//'
    else
        echo "No templates found"
    fi
}

# Function to test notifications
test_notifications() {
    log "INFO" "Testing notification system"

    local test_details="Test notification from Rust AI IDE CI/CD pipeline\nTime: $(date)\nStatus: Test successful"

    notify_stakeholders "test" "success" "${test_details}"

    log "INFO" "Notification test completed"
}

# Main execution
main() {
    local command="${1:-help}"
    shift

    case "${command}" in
        "notify")
            if [[ $# -lt 3 ]]; then
                echo "Usage: $0 notify <event_type> <status> <details> [template]"
                exit 1
            fi
            local event_type="$1"
            local status="$2"
            local details="$3"
            local template="${4:-}"
            notify_stakeholders "${event_type}" "${status}" "${details}" "${template}"
            ;;
        "template")
            local subcommand="${1:-list}"
            case "${subcommand}" in
                "create")
                    create_default_templates
                    ;;
                "list")
                    list_templates
                    ;;
                *)
                    echo "Usage: $0 template {create|list}"
                    exit 1
                    ;;
            esac
            ;;
        "test")
            test_notifications
            ;;
        "help"|*)
            cat << EOF
Usage: $0 <command> [arguments]

Commands:
    notify <event_type> <status> <details> [template]  - Send notification
    template {create|list}                           - Manage templates
    test                                            - Test notification system
    help                                            - Show this help

Event Types:
    documentation_update, review_complete, deployment, test

Status Values:
    success, failure, warning, info

Examples:
    $0 notify documentation_update success "Docs updated successfully"
    $0 notify deployment success "v1.2.3 deployed" deployment
    $0 template create
    $0 test
EOF
            ;;
    esac
}

# Trap for cleanup
trap 'log "INFO" "Notification script interrupted"' INT TERM

# Run main function
main "$@"
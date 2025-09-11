#!/bin/bash
# Main Documentation and Notification Integration Script
# Orchestrates all CI/CD workflows for documentation, reviews, git operations, and notifications

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
LOG_FILE="${PROJECT_ROOT}/logs/main-integration-$(date +%Y%m%d-%H%M%S).log"
WORKFLOW_CONFIG="${SCRIPT_DIR}/workflow-config.json"

# Import environment variables from secret manager or env file
if [[ -f "${PROJECT_ROOT}/.env.ci" ]]; then
    source "${PROJECT_ROOT}/.env.ci"
fi

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

    # Send failure notification
    if [[ -f "${SCRIPT_DIR}/stakeholder-notifications.sh" ]]; then
        bash "${SCRIPT_DIR}/stakeholder-notifications.sh" notify "workflow_failure" "failed" "${message}"
    fi

    exit 1
}

# Success notification
notify_success() {
    local workflow="$1"
    local details="$2"

    if [[ -f "${SCRIPT_DIR}/stakeholder-notifications.sh" ]]; then
        bash "${SCRIPT_DIR}/stakeholder-notifications.sh" notify "${workflow}" "success" "${details}"
    fi
}

# Function to run documentation workflow
run_documentation_workflow() {
    log "INFO" "Starting documentation workflow"

    local start_time=$(date +%s)

    # Run documentation update
    if [[ -f "${SCRIPT_DIR}/documentation-update.sh" ]]; then
        if bash "${SCRIPT_DIR}/documentation-update.sh"; then
            log "INFO" "Documentation update completed successfully"
        else
            error_exit "Documentation update failed"
        fi
    else
        error_exit "Documentation update script not found"
    fi

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    local details="Documentation workflow completed in ${duration}s
- Rust documentation generated
- TypeScript types updated
- Changelog updated"

    notify_success "documentation_update" "${details}"
    log "INFO" "Documentation workflow completed in ${duration}s"
}

# Function to run review workflow
run_review_workflow() {
    log "INFO" "Starting code review workflow"

    local start_time=$(date +%s)

    # Run automated review checks
    if [[ -f "${SCRIPT_DIR}/review-workflow.sh" ]]; then
        if bash "${SCRIPT_DIR}/review-workflow.sh"; then
            log "INFO" "Review workflow completed successfully"
        else
            error_exit "Review workflow failed"
        fi
    else
        error_exit "Review workflow script not found"
    fi

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    local details="Code review workflow completed in ${duration}s
- Automated checks passed
- Manual oversight completed
- Review report generated"

    notify_success "review_complete" "${details}"
    log "INFO" "Review workflow completed in ${duration}s"
}

# Function to run git operations workflow
run_git_workflow() {
    local operation="$1"
    local args="${2:-}"

    log "INFO" "Starting git ${operation} workflow"

    if [[ -f "${SCRIPT_DIR}/git-operations.sh" ]]; then
        if bash "${SCRIPT_DIR}/git-operations.sh" "${operation}" ${args}; then
            log "INFO" "Git ${operation} completed successfully"
            notify_success "git_operation" "Git ${operation} completed successfully"
        else
            error_exit "Git ${operation} failed"
        fi
    else
        error_exit "Git operations script not found"
    fi
}

# Function to run complete CI/CD workflow
run_full_workflow() {
    log "INFO" "Starting complete CI/CD workflow"

    local workflow_start=$(date +%s)

    # 1. Documentation updates
    log "INFO" "Step 1: Running documentation updates"
    run_documentation_workflow

    # 2. Code review and validation
    log "INFO" "Step 2: Running code review"
    run_review_workflow

    # 3. Commit changes
    log "INFO" "Step 3: Committing changes"
    local commit_message="ci: Automated workflow updates

- Updated documentation
- Passed review checks
- Generated reports

Workflow ID: $(date +%Y%m%d-%H%M%S)"
    run_git_workflow "commit" "ci 'Automated workflow updates' '${commit_message}'"

    # 4. Push changes
    log "INFO" "Step 4: Pushing changes"
    run_git_workflow "push"

    # 5. Final notifications
    local workflow_end=$(date +%s)
    local total_duration=$((workflow_end - workflow_start))

    local final_details="Complete CI/CD workflow finished in ${total_duration}s
- Documentation: ✅ Updated
- Review: ✅ Completed
- Git Operations: ✅ Committed and pushed
- Notifications: ✅ Sent"

    notify_success "full_workflow" "${final_details}"
    log "INFO" "Complete CI/CD workflow finished in ${total_duration}s"
}

# Function to check system status
check_system_status() {
    log "INFO" "Checking system status"

    # Check if all required scripts exist
    local required_scripts=(
        "documentation-update.sh"
        "review-workflow.sh"
        "git-operations.sh"
        "stakeholder-notifications.sh"
    )

    for script in "${required_scripts[@]}"; do
        if [[ ! -f "${SCRIPT_DIR}/${script}" ]]; then
            error_exit "Required script ${script} not found in ${SCRIPT_DIR}"
        fi
    done

    # Check if scripts are executable
    for script in "${required_scripts[@]}"; do
        if [[ ! -x "${SCRIPT_DIR}/${script}" ]]; then
            log "WARN" "${script} is not executable, attempting to fix"
            chmod +x "${SCRIPT_DIR}/${script}"
        fi
    done

    # Check git status
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        error_exit "Not in a git repository"
    fi

    # Check required tools
    local required_tools=("cargo" "npm" "git" "curl")
    for tool in "${required_tools[@]}"; do
        if ! command -v "${tool}" &> /dev/null; then
            error_exit "Required tool ${tool} not found"
        fi
    done

    log "INFO" "System status check passed"
}

# Function to create workflow configuration
create_workflow_config() {
    log "INFO" "Creating workflow configuration"

    cat > "${WORKFLOW_CONFIG}" << EOF
{
    "version": "1.0.0",
    "workflows": {
        "documentation": {
            "description": "Automated documentation updates",
            "steps": ["generate_rust_docs", "generate_types", "update_changelog"],
            "notifications": ["slack", "email", "webhook"]
        },
        "review": {
            "description": "Code review and validation",
            "steps": ["automated_checks", "manual_oversight", "generate_report"],
            "notifications": ["slack", "email"]
        },
        "git_operations": {
            "description": "Git operations with logging",
            "steps": ["commit", "push", "rollback"],
            "notifications": ["webhook"]
        },
        "full_pipeline": {
            "description": "Complete CI/CD pipeline",
            "steps": ["documentation", "review", "git_operations"],
            "notifications": ["slack", "email", "webhook"]
        }
    },
    "notifications": {
        "channels": ["slack", "email", "webhook"],
        "templates": ["documentation_update", "review_complete", "deployment"],
        "recipients": {
            "slack": "${SLACK_CHANNEL:-#general}",
            "email": "${EMAIL_RECIPIENTS:-}",
            "webhook": "${WEBHOOK_URL:-}"
        }
    },
    "security": {
        "secret_manager": "${SECRET_MANAGER:-env}",
        "audit_trail": true,
        "compliance_checks": true
    }
}
EOF

    log "INFO" "Workflow configuration created at ${WORKFLOW_CONFIG}"
}

# Function to display help
show_help() {
    cat << EOF
Rust AI IDE - Documentation and Notification Integration System

Usage: $0 <command> [options]

Commands:
    docs           - Run documentation workflow only
    review         - Run code review workflow only
    git <operation> [args] - Run git operations (commit, push, rollback, status)
    full           - Run complete CI/CD workflow
    status         - Check system status
    config         - Create/update workflow configuration
    help           - Show this help

Examples:
    $0 docs                                    # Update documentation
    $0 review                                  # Run code review
    $0 git commit "feat" "Add new feature"     # Commit changes
    $0 git push                               # Push changes
    $0 full                                   # Run complete workflow
    $0 status                                 # Check system status

Environment Variables:
    SLACK_WEBHOOK     - Slack webhook URL
    EMAIL_SMTP_SERVER - SMTP server for email notifications
    EMAIL_RECIPIENTS  - Email recipients for notifications
    WEBHOOK_URL       - Generic webhook URL
    SECRET_MANAGER    - Secret manager (env, vault, aws, etc.)

Configuration:
    Scripts are located in: ${SCRIPT_DIR}
    Logs are written to: ${PROJECT_ROOT}/logs/
    Configuration: ${WORKFLOW_CONFIG}

For more details, see the documentation in docs/ci-cd/
EOF
}

# Main execution
main() {
    local command="${1:-help}"

    # Create logs directory
    mkdir -p "$(dirname "${LOG_FILE}")"

    log "INFO" "Starting main integration workflow: ${command}"

    case "${command}" in
        "docs")
            check_system_status
            run_documentation_workflow
            ;;
        "review")
            check_system_status
            run_review_workflow
            ;;
        "git")
            shift
            check_system_status
            run_git_workflow "$@"
            ;;
        "full")
            check_system_status
            run_full_workflow
            ;;
        "status")
            check_system_status
            echo "✅ System status check passed"
            ;;
        "config")
            create_workflow_config
            ;;
        "help"|*)
            show_help
            ;;
    esac

    log "INFO" "Main integration workflow completed"
}

# Trap for cleanup
trap 'error_exit "Main integration script interrupted"' INT TERM

# Run main function
main "$@"
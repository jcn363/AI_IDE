#!/bin/bash

# Retrieve Snyk credentials from AWS Secrets Manager
# Securely loads SNYK_TOKEN and SNYK_ORG into environment variables
# Author: DevOps Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
SECRET_ID="/rust-ai-ide/snyk/credentials"

# Logging function
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" >&2
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" >&2
}

# Check AWS CLI availability
check_aws_cli() {
    if ! command -v aws >/dev/null 2>&1; then
        log_error "AWS CLI not found"
        return 1
    fi

    if ! aws sts get-caller-identity >/dev/null 2>&1; then
        log_error "AWS CLI not configured or credentials invalid"
        return 1
    fi

    log_info "AWS CLI check passed"
}

# Retrieve secrets from AWS Secrets Manager
retrieve_secrets() {
    local secret_value
    local snyk_token
    local snyk_org

    log_info "Retrieving Snyk secrets from AWS Secrets Manager..."

    # Get secret value
    secret_value=$(aws secretsmanager get-secret-value \
        --secret-id "$SECRET_ID" \
        --query 'SecretString' \
        --output text)

    if [[ -z "$secret_value" ]]; then
        log_error "Failed to retrieve secret value"
        return 1
    fi

    # Parse JSON and extract values
    snyk_token=$(echo "$secret_value" | jq -r '.SNYK_TOKEN')
    snyk_org=$(echo "$secret_value" | jq -r '.SNYK_ORG')

    if [[ "$snyk_token" == "null" || -z "$snyk_token" ]]; then
        log_error "SNYK_TOKEN not found in secret"
        return 1
    fi

    if [[ "$snyk_org" == "null" || -z "$snyk_org" ]]; then
        log_error "SNYK_ORG not found in secret"
        return 1
    fi

    # Export environment variables (these will be available to calling scripts)
    export SNYK_TOKEN="$snyk_token"
    export SNYK_ORG="$snyk_org"

    log_info "Snyk credentials loaded successfully"
    log_info "SNYK_TOKEN: ${#snyk_token} characters"
    log_info "SNYK_ORG: ${snyk_org}"
}

# Main function
main() {
    log_info "Starting Snyk secrets retrieval..."

    check_aws_cli || exit 1
    retrieve_secrets || exit 1

    log_info "Snyk secrets retrieval completed"
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
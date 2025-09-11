#!/bin/bash

# Test Snyk Setup Configuration
# Validates the Snyk integration setup without running full scans
# Author: DevOps Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
WEB_DIR="${PROJECT_ROOT}/web"

# Logging function
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" >&2
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" >&2
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" >&2
}

# Test AWS CLI and secrets access
test_aws_secrets() {
    log_info "Testing AWS CLI and secrets access..."

    if ! command -v aws >/dev/null 2>&1; then
        log_error "AWS CLI not available"
        return 1
    fi

    if ! aws sts get-caller-identity >/dev/null 2>&1; then
        log_error "AWS CLI not configured"
        return 1
    fi

    log_info "AWS CLI configured successfully"

    # Test secrets retrieval script
    if [[ -f "${SCRIPT_DIR}/retrieve-snyk-secrets.sh" ]]; then
        log_info "Testing secrets retrieval script..."
        chmod +x "${SCRIPT_DIR}/retrieve-snyk-secrets.sh"

        # Source the script to test (in a subshell to avoid affecting current environment)
        if (source "${SCRIPT_DIR}/retrieve-snyk-secrets.sh" && [[ -n "${SNYK_TOKEN:-}" ]] && [[ -n "${SNYK_ORG:-}" ]]); then
            log_success "Secrets retrieval test passed"
            log_info "SNYK_TOKEN length: ${#SNYK_TOKEN}"
            log_info "SNYK_ORG: ${SNYK_ORG}"
        else
            log_error "Secrets retrieval test failed"
            return 1
        fi
    else
        log_error "Secrets retrieval script not found"
        return 1
    fi
}

# Test Node.js and npm
test_nodejs_setup() {
    log_info "Testing Node.js and npm setup..."

    if ! command -v node >/dev/null 2>&1; then
        log_error "Node.js not found"
        return 1
    fi

    if ! command -v npm >/dev/null 2>&1; then
        log_error "npm not found"
        return 1
    fi

    local node_version=$(node --version)
    local npm_version=$(npm --version)

    log_info "Node.js version: ${node_version}"
    log_info "npm version: ${npm_version}"

    # Test web directory
    if [[ ! -f "${WEB_DIR}/package.json" ]]; then
        log_error "Web directory or package.json not found"
        return 1
    fi

    log_success "Node.js setup test passed"
}

# Test Snyk CLI
test_snyk_cli() {
    log_info "Testing Snyk CLI..."

    if ! command -v snyk >/dev/null 2>&1; then
        log_info "Snyk CLI not found, attempting to install..."
        if ! npm install -g snyk; then
            log_error "Failed to install Snyk CLI"
            return 1
        fi
    fi

    local snyk_version=$(snyk --version)
    log_info "Snyk CLI version: ${snyk_version}"

    log_success "Snyk CLI test passed"
}

# Test Snyk authentication
test_snyk_auth() {
    log_info "Testing Snyk authentication..."

    # Load credentials
    source "${SCRIPT_DIR}/retrieve-snyk-secrets.sh"

    if [[ -z "${SNYK_TOKEN:-}" ]]; then
        log_error "SNYK_TOKEN not loaded"
        return 1
    fi

    # Test authentication
    if snyk auth "${SNYK_TOKEN}"; then
        log_success "Snyk authentication test passed"
    else
        log_error "Snyk authentication test failed"
        return 1
    fi
}

# Test Snyk dependency scan setup
test_scan_setup() {
    log_info "Testing Snyk dependency scan setup..."

    if [[ ! -f "${SCRIPT_DIR}/snyk-dependency-scan.sh" ]]; then
        log_error "Snyk dependency scan script not found"
        return 1
    fi

    chmod +x "${SCRIPT_DIR}/snyk-dependency-scan.sh"
    log_info "Snyk dependency scan script is executable"

    # Test script help/usage
    if "${SCRIPT_DIR}/snyk-dependency-scan.sh" --help >/dev/null 2>&1; then
        log_info "Snyk dependency scan script help works"
    else
        log_info "Snyk dependency scan script help not available (expected)"
    fi

    log_success "Snyk scan setup test passed"
}

# Generate test report
generate_test_report() {
    local report_file="${PROJECT_ROOT}/snyk-setup-test-report.json"

    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg status "PASSED" \
        '{
            timestamp: $timestamp,
            test: "Snyk Setup Validation",
            status: $status,
            components: {
                aws_cli: "PASSED",
                secrets_retrieval: "PASSED",
                nodejs_setup: "PASSED",
                snyk_cli: "PASSED",
                snyk_auth: "PASSED",
                scan_setup: "PASSED"
            }
        }' > "${report_file}"

    log_info "Test report generated: ${report_file}"
}

# Main function
main() {
    log_info "Starting Snyk setup validation tests..."

    local test_results=()

    # Run tests
    if test_aws_secrets; then
        test_results+=("aws_secrets:PASSED")
    else
        test_results+=("aws_secrets:FAILED")
    fi

    if test_nodejs_setup; then
        test_results+=("nodejs_setup:PASSED")
    else
        test_results+=("nodejs_setup:FAILED")
    fi

    if test_snyk_cli; then
        test_results+=("snyk_cli:PASSED")
    else
        test_results+=("snyk_cli:FAILED")
    fi

    if test_snyk_auth; then
        test_results+=("snyk_auth:PASSED")
    else
        test_results+=("snyk_auth:FAILED")
    fi

    if test_scan_setup; then
        test_results+=("scan_setup:PASSED")
    else
        test_results+=("scan_setup:FAILED")
    fi

    # Check if any tests failed
    local failed_tests=0
    for result in "${test_results[@]}"; do
        if [[ "$result" == *":FAILED" ]]; then
            ((failed_tests++))
        fi
    done

    if [[ $failed_tests -eq 0 ]]; then
        log_success "All Snyk setup tests passed!"
        generate_test_report
        return 0
    else
        log_error "$failed_tests test(s) failed"
        return 1
    fi
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
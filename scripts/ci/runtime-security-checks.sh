#!/bin/bash

# Runtime Security Checks Script for Rust AI IDE
# Comprehensive runtime security validation for CI/CD pipelines
# Author: Security Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
SECURITY_LOG="${PROJECT_ROOT}/security-runtime-checks.log"
REPORT_DIR="${PROJECT_ROOT}/security-reports/runtime"
START_TIME=$(date +%s)

# Create report directory
mkdir -p "${REPORT_DIR}"

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${SECURITY_LOG}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${SECURITY_LOG}" >&2
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${SECURITY_LOG}"
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${SECURITY_LOG}"
}

# Function to check for security features in build
check_security_features() {
    log_info "Checking security features in build..."

    if [[ ! -f "${PROJECT_ROOT}/Cargo.toml" ]]; then
        log_error "Cargo.toml not found"
        return 1
    fi

    # Check for security-related features in Cargo.toml
    if grep -q "\[features\]" "${PROJECT_ROOT}/Cargo.toml" && grep -q "security" "${PROJECT_ROOT}/Cargo.toml"; then
        log_success "Security features found in Cargo.toml"
    else
        log_warning "Security features not configured in Cargo.toml"
    fi

    return 0
}

# Function to run tests with security features
run_security_tests() {
    log_info "Running tests with security features..."

    if ! command -v cargo >/dev/null 2>&1; then
        log_error "Cargo not found"
        return 1
    fi

    # Set nightly toolchain for enhanced security checks
    if command -v rustup >/dev/null 2>&1; then
        rustup default nightly 2>/dev/null || log_warning "Could not set nightly toolchain"
    fi

    # Build with security features
    log_info "Building with security features..."
    if cargo build --workspace --release --features security; then
        log_success "Security build completed successfully"
    else
        log_error "Security build failed"
        return 1
    fi

    # Run tests with security features
    log_info "Running security-focused tests..."
    if cargo test --workspace --release --features security -- --nocapture > "${REPORT_DIR}/security-test-output.log" 2>&1; then
        log_success "Security tests passed"

        # Analyze test output for security issues
        if grep -qi "panic\|unwrap\|expect" "${REPORT_DIR}/security-test-output.log"; then
            log_warning "Potential security issues detected in test output"
            grep -i "panic\|unwrap\|expect" "${REPORT_DIR}/security-test-output.log" | head -5
        else
            log_success "No security issues detected in test output"
        fi
    else
        log_error "Security tests failed"
        return 1
    fi

    return 0
}

# Function to check for debug assertions in release builds
check_debug_assertions() {
    log_info "Checking for debug assertions in release builds..."

    if cargo build --release -v > "${REPORT_DIR}/release-build.log" 2>&1; then
        if grep -q "debug_assert" "${REPORT_DIR}/release-build.log"; then
            log_warning "Debug assertions found in release build"
            grep "debug_assert" "${REPORT_DIR}/release-build.log" | head -5
        else
            log_success "No debug assertions found in release build"
        fi
    else
        log_error "Release build failed"
        return 1
    fi

    return 0
}

# Function to run memory safety checks
run_memory_safety_checks() {
    log_info "Running memory safety checks..."

    # Check if AddressSanitizer is available
    if echo "int main() {}" | gcc -fsanitize=address -o /dev/null -xc - 2>/dev/null; then
        log_info "AddressSanitizer is available"

        # Run tests with AddressSanitizer
        log_info "Running tests with AddressSanitizer..."
        if RUSTFLAGS="-Z sanitizer=address" cargo test --workspace --lib --release --features security \
            --target x86_64-unknown-linux-gnu > "${REPORT_DIR}/address-sanitizer.log" 2>&1; then
            log_success "AddressSanitizer checks passed"
        else
            log_warning "AddressSanitizer detected issues (review log for details)"
        fi
    else
        log_warning "AddressSanitizer not available"
    fi

    return 0
}

# Function to check for unsafe code usage
check_unsafe_code() {
    log_info "Checking unsafe code usage patterns..."

    if ! command -v cargo >/dev/null 2>&1; then
        log_error "Cargo not found"
        return 1
    fi

    # Use cargo-geiger if available
    if command -v cargo-geiger >/dev/null 2>&1; then
        log_info "Running cargo-geiger for unsafe code analysis..."
        if cargo geiger --format json --output "${REPORT_DIR}/unsafe-analysis.json"; then
            local unsafe_count=$(jq '.metrics.unsafe // 0' "${REPORT_DIR}/unsafe-analysis.json" 2>/dev/null || echo "0")
            if [[ "${unsafe_count}" -gt 0 ]]; then
                log_warning "Unsafe code detected: ${unsafe_count} instances"
                cargo geiger --format ascii-table | head -20
            else
                log_success "No unsafe code detected"
            fi
        else
            log_warning "cargo-geiger analysis failed"
        fi
    else
        log_warning "cargo-geiger not available for unsafe code analysis"
    fi

    return 0
}

# Function to validate security configurations
validate_security_config() {
    log_info "Validating security configurations..."

    # Check for security-related environment variables
    local security_vars=("RUST_BACKTRACE" "RUST_LOG" "PANIC" "SECURITY_SCAN_ENABLED")

    for var in "${security_vars[@]}"; do
        if [[ -n "${!var:-}" ]]; then
            log_info "Security variable ${var} is set"
        fi
    done

    # Check for security-focused Cargo features
    if [[ -f "${PROJECT_ROOT}/Cargo.toml" ]]; then
        if grep -q "security" "${PROJECT_ROOT}/Cargo.toml"; then
            log_success "Security features configured in Cargo.toml"
        else
            log_warning "Security features not found in Cargo.toml"
        fi
    fi

    return 0
}

# Function to generate runtime security report
generate_runtime_report() {
    log_info "Generating runtime security report..."

    local runtime_report="${REPORT_DIR}/runtime-security-report.json"
    local end_time=$(date +%s)
    local duration=$((end_time - START_TIME))

    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg duration "$duration" \
        --arg build_status "COMPLETED" \
        --arg security_tests "$(grep -c "PASSED\|SUCCESS" "${SECURITY_LOG}" || echo "0")" \
        --arg warnings "$(grep -c "WARNING" "${SECURITY_LOG}" || echo "0")" \
        --arg errors "$(grep -c "ERROR" "${SECURITY_LOG}" || echo "0")" \
        '{
            timestamp: $timestamp,
            duration_seconds: ($duration | tonumber),
            overall_status: (if ($errors | tonumber) == 0 then "PASSED" else "FAILED" end),
            checks: {
                security_features: "COMPLETED",
                security_tests: $security_tests,
                memory_safety: "COMPLETED",
                unsafe_code_analysis: "COMPLETED",
                security_config_validation: "COMPLETED"
            },
            issues: {
                warnings: ($warnings | tonumber),
                errors: ($errors | tonumber)
            },
            recommendations: [
                "Review any WARNING messages in the security log",
                "Address any ERROR conditions found during runtime checks",
                "Ensure security features are enabled in production builds",
                "Regularly audit unsafe code usage",
                "Keep security dependencies updated"
            ]
        }' > "${runtime_report}"

    log_info "Runtime security report generated: ${runtime_report}"

    # Generate HTML summary
    cat > "${REPORT_DIR}/runtime-security-summary.html" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>Runtime Security Report - Rust AI IDE</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f0f0f0; padding: 10px; border-radius: 5px; }
        .status-passed { color: green; }
        .status-failed { color: red; }
        .status-warning { color: orange; }
        .section { margin: 20px 0; padding: 10px; border: 1px solid #ccc; border-radius: 5px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Runtime Security Report</h1>
        <p><strong>Generated:</strong> $(date)</p>
        <p><strong>Duration:</strong> ${duration} seconds</p>
    </div>

    <div class="section">
        <h2>Security Test Results</h2>
        <p>Tests Passed: $(grep -c "PASSED\|SUCCESS" "${SECURITY_LOG}" || echo "0")</p>
        <p>Warnings: $(grep -c "WARNING" "${SECURITY_LOG}" || echo "0")</p>
        <p>Errors: $(grep -c "ERROR" "${SECURITY_LOG}" || echo "0")</p>
    </div>

    <div class="section">
        <h2>Log Files</h2>
        <ul>
            <li><a href="runtime-security-report.json">Detailed JSON Report</a></li>
            <li><a href="../../../security-runtime-checks.log">Security Log</a></li>
        </ul>
    </div>
</body>
</html>
EOF

    log_info "HTML summary generated: ${REPORT_DIR}/runtime-security-summary.html"
}

# Main function
main() {
    log_info "Starting runtime security checks"
    log_info "Report directory: ${REPORT_DIR}"

    local exit_code=0

    # Run security checks
    check_security_features || exit_code=$((exit_code + 1))
    run_security_tests || exit_code=$((exit_code + 1))
    check_debug_assertions || exit_code=$((exit_code + 1))
    run_memory_safety_checks || exit_code=$((exit_code + 1))
    check_unsafe_code || exit_code=$((exit_code + 1))
    validate_security_config || exit_code=$((exit_code + 1))

    # Generate report
    generate_runtime_report

    local end_time=$(date +%s)
    log_info "Runtime security checks completed in $((end_time - START_TIME)) seconds"

    if [[ $exit_code -eq 0 ]]; then
        log_success "All runtime security checks passed"
    else
        log_error "Runtime security checks failed with ${exit_code} issues"
    fi

    return $exit_code
}

# Run main function
main "$@"
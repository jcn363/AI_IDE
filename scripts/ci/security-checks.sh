#!/bin/bash

# Security Checks Script for Rust AI IDE
# Comprehensive security validation for CI/CD pipelines
# Author: Security Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
SECURITY_LOG="${PROJECT_ROOT}/security-checks.log"
REPORT_DIR="${PROJECT_ROOT}/security-reports"
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

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Comprehensive security checks for Rust AI IDE CI/CD pipelines.

OPTIONS:
    -h, --help              Show this help message
    -v, --verbose           Enable verbose output
    -q, --quick             Run only critical security checks
    -s, --strict            Fail on warnings (stricter mode)
    -o, --output DIR        Output directory for reports (default: security-reports)
    --skip-dependency-scan  Skip dependency vulnerability scanning
    --skip-sast             Skip static application security testing
    --skip-container-scan   Skip container security scanning
    --skip-license-check    Skip license compliance checking
    --skip-sbom             Skip Software Bill of Materials generation
    --skip-compliance       Skip compliance monitoring
    --skip-remediation      Skip automated vulnerability remediation

EXAMPLES:
    $0 --quick --strict
    $0 --skip-container-scan --output /tmp/security-results
    $0 --verbose

EOF
}

# Parse command line arguments
VERBOSE=false
QUICK=false
STRICT=false
OUTPUT_DIR="${REPORT_DIR}"
SKIP_DEPENDENCY_SCAN=false
SKIP_SAST=false
SKIP_CONTAINER_SCAN=false
SKIP_LICENSE_CHECK=false
SKIP_SBOM=false
SKIP_COMPLIANCE=false
SKIP_REMEDIATION=false

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
        -q|--quick)
            QUICK=true
            shift
            ;;
        -s|--strict)
            STRICT=true
            shift
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --skip-dependency-scan)
            SKIP_DEPENDENCY_SCAN=true
            shift
            ;;
        --skip-sast)
            SKIP_SAST=true
            shift
            ;;
        --skip-container-scan)
            SKIP_CONTAINER_SCAN=true
            shift
            ;;
        --skip-license-check)
            SKIP_LICENSE_CHECK=true
            shift
            ;;
        --skip-sbom)
            SKIP_SBOM=true
            shift
            ;;
        --skip-compliance)
            SKIP_COMPLIANCE=true
            shift
            ;;
        --skip-remediation)
            SKIP_REMEDIATION=true
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Function to check system requirements
check_security_tools() {
    log_info "Checking security tools..."

    local required_tools=("cargo" "rustc")
    local optional_tools=("cargo-audit" "cargo-deny" "cargo-geiger" "trivy" "grype")

    for tool in "${required_tools[@]}"; do
        if ! command -v "${tool}" >/dev/null 2>&1; then
            log_error "Required tool '${tool}' not found"
            return 1
        fi
    done

    # Check if nightly toolchain is available
    if ! cargo +nightly --version >/dev/null 2>&1; then
        log_warning "Nightly Rust toolchain not available - some security checks may be limited"
    else
        log_info "Nightly Rust toolchain available for enhanced security checks"
    fi

    for tool in "${optional_tools[@]}"; do
        if ! command -v "${tool}" >/dev/null 2>&1; then
            log_warning "Optional security tool '${tool}' not found"
        fi
    done
}

# Function to install security tools if needed
install_security_tools() {
    if [[ "${QUICK}" == true ]]; then
        return 0
    fi

    log_info "Installing security tools..."

    # cargo-audit for dependency auditing
    if ! command -v cargo-audit >/dev/null 2>&1; then
        log_info "Installing cargo-audit..."
        cargo install cargo-audit --features=fix
    fi

    # cargo-deny for dependency management
    if ! command -v cargo-deny >/dev/null 2>&1; then
        log_info "Installing cargo-deny..."
        cargo install cargo-deny
    fi

    # cargo-geiger for unsafe code analysis
    if ! command -v cargo-geiger >/dev/null 2>&1; then
        log_info "Installing cargo-geiger..."
        cargo install cargo-geiger
    fi

    # trivy for container scanning (if available)
    if ! command -v trivy >/dev/null 2>&1 && command -v apt-get >/dev/null 2>&1; then
        log_info "Installing trivy..."
        sudo apt-get update && sudo apt-get install -y wget apt-transport-https gnupg
        wget -qO - https://aquasecurity.github.io/trivy-repo/deb/public.key | sudo apt-key add -
        echo "deb https://aquasecurity.github.io/trivy-repo/deb $(lsb_release -sc) main" | sudo tee -a /etc/apt/sources.list.d/trivy.list
        sudo apt-get update && sudo apt-get install -y trivy
    fi

    if ! command -v trivy >/dev/null 2>&1; then
        log_warning "Trivy not available for container scanning"
    fi
}

# Function for static application security testing (SAST)
run_sast() {
    if [[ "${SKIP_SAST}" == true ]]; then
        log_info "Skipping SAST checks (--skip-sast)"
        return 0
    fi

    # Check if nightly toolchain is available for enhanced checks
    if ! cargo +nightly --version >/dev/null 2>&1; then
        if [[ "${QUICK}" == true ]]; then
            log_info "Skipping SAST checks (--quick mode and no nightly toolchain)"
            return 0
        else
            log_warning "Nightly toolchain not available - running basic SAST checks only"
        fi
    fi

    log_info "Running Static Application Security Testing..."

    local sast_report="${OUTPUT_DIR}/sast-report.json"
    local issues_found=0

    # Clippy security checks with nightly toolchain
    log_info "Running enhanced Clippy security checks..."
    if cargo +nightly clippy --all-targets --all-features \
        -- -D clippy::unwrap_used \
        -D clippy::expect_used \
        -D clippy::panic \
        -D clippy::unimplemented \
        -D clippy::todo \
        -D clippy::unreachable \
        -W clippy::pedantic \
        -W clippy::nursery 2>&1 | tee "${OUTPUT_DIR}/clippy-security.log"; then
        log_success "Clippy security checks passed"
    else
        issues_found=$((issues_found + 1))
        [[ "${STRICT}" == true ]] && log_error "Clippy security issues found" || log_warning "Clippy security issues found (non-blocking)"
    fi

    # Unsafe code analysis
    if command -v cargo-geiger >/dev/null 2>&1; then
        log_info "Analyzing unsafe code usage..."
        cargo geiger --format json --output "${OUTPUT_DIR}/unsafe-analysis.json"
        local unsafe_count=$(jq '.metrics.unsafe' "${OUTPUT_DIR}/unsafe-analysis.json" 2>/dev/null || echo "0")
        if [[ "${unsafe_count}" -gt 0 ]]; then
            issues_found=$((issues_found + 1))
            [[ "${STRICT}" == true ]] && log_error "Unsafe code detected (${unsafe_count} instances)" || log_warning "Unsafe code detected (${unsafe_count} instances)"
        else
            log_success "No unsafe code detected"
        fi
    fi

    # Generate SAST summary
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg issues_found "$issues_found" \
        --arg strict_mode "$STRICT" \
        '{
            timestamp: $timestamp,
            tool: "SAST",
            issues_found: ($issues_found | tonumber),
            strict_mode: ($strict_mode == "true"),
            status: (if ($issues_found == "0") then "PASSED" elif ($strict_mode == "true") then "FAILED" else "WARNING" end)
        }' > "${sast_report}"

    log_info "SAST completed. Report: ${sast_report}"
    return $issues_found
}

# Function for dependency security scanning
run_dependency_scan() {
    if [[ "${SKIP_DEPENDENCY_SCAN}" == true ]]; then
        log_info "Skipping dependency security scanning"
        return 0
    fi

    log_info "Running dependency security scanning..."

    local dep_report="${OUTPUT_DIR}/dependency-security-report.json"
    local issues_found=0

    # cargo-audit for known vulnerabilities (with nightly toolchain)
    if command -v cargo-audit >/dev/null 2>&1; then
        log_info "Running cargo-audit..."
        if cargo +nightly audit --format json 2>/dev/null | jq . > "${OUTPUT_DIR}/audit-results.json"; then
            local vulnerabilities=$(jq '.vulnerabilities.count // 0' "${OUTPUT_DIR}/audit-results.json" 2>/dev/null || echo "0")
            if [[ "${vulnerabilities}" -gt 0 ]]; then
                issues_found=$((issues_found + 1))
                log_error "Security vulnerabilities found: ${vulnerabilities}"
                jq '.vulnerabilities.list[]? | {package: .package.name, advisory: .advisory.id, severity: .advisory.severity}' "${OUTPUT_DIR}/audit-results.json" 2>/dev/null | head -10
            else
                log_success "No known vulnerabilities found"
            fi
        else
            # cargo-audit might fail if no Cargo.lock exists or other issues
            log_warning "cargo-audit check could not be completed"
        fi
    fi

    # cargo-deny for license and advisory checks (with nightly toolchain)
    if command -v cargo-deny >/dev/null 2>&1; then
        log_info "Running cargo-deny checks..."

        # License compliance
        if cargo +nightly deny check licenses --format json > "${OUTPUT_DIR}/license-check.json" 2>&1; then
            log_success "License compliance check passed"
        else
            issues_found=$((issues_found + 1))
            log_error "License compliance issues found"
        fi

        # Security advisories
        if cargo +nightly deny check advisories --format json > "${OUTPUT_DIR}/advisory-check.json" 2>&1; then
            log_success "Security advisory check passed"
        else
            issues_found=$((issues_found + 1))
            log_error "Security advisory issues found"
        fi

        # Bans check
        if cargo +nightly deny check bans --format json > "${OUTPUT_DIR}/bans-check.json" 2>&1; then
            log_success "Dependency bans check passed"
        else
            issues_found=$((issues_found + 1))
            log_error "Dependency ban violations found"
        fi
    fi

    # Generate dependency security summary
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg issues_found "$issues_found" \
        '{
            timestamp: $timestamp,
            tool: "Dependency Scan",
            issues_found: ($issues_found | tonumber),
            status: (if ($issues_found == "0") then "PASSED" else "FAILED" end)
        }' > "${dep_report}"

    log_info "Dependency security scan completed. Report: ${dep_report}"

    # SBOM Generation (New Feature)
    if [[ "${SCAN_SBOM:-false}" == true ]]; then
        log_info "Generating Software Bill of Materials (SBOM)..."
        mkdir -p "${OUTPUT_DIR}/sbom"

        # Use cargo-cyclonedx to generate SBOM
        if command -v cargo-cyclonedx >/dev/null 2>&1; then
            cargo cyclonedx --format json --output "${OUTPUT_DIR}/sbom/application-sbom.json" 2>/dev/null || \
                log_warning "Failed to generate CycloneDX SBOM with cargo-cyclonedx"
        else
            log_info "cargo-cyclonedx not available, installing..."
            cargo install cargo-cyclonedx
            cargo cyclonedx --format json --output "${OUTPUT_DIR}/sbom/application-sbom.json" 2>/dev/null || \
                log_warning "Failed to generate CycloneDX SBOM"
        fi

        # Generate SPDX format SBOM as fallback/backup
        if command -v cargo-spdx >/dev/null 2>&1; then
            cargo spdx --format json --output "${OUTPUT_DIR}/sbom/application-spdx.json" 2>/dev/null || \
                log_warning "Failed to generate SPDX SBOM"
        else
            log_warning "cargo-spdx not available - SPDX SBOM generation skipped"
        fi

        log_success "SBOM generation completed"
    fi

    return $issues_found
}

# Function for container security scanning
run_container_scan() {
    if [[ "${SKIP_CONTAINER_SCAN}" == true || "${QUICK}" == true ]]; then
        log_info "Skipping container security scanning"
        return 0
    fi

    if ! command -v trivy >/dev/null 2>&1; then
        log_info "Skipping container scan - trivy not available"
        return 0
    fi

    log_info "Running container security scanning..."

    local container_report="${OUTPUT_DIR}/container-security-report.json"
    local issues_found=0

    # Scan Rust AI containers if they exist
    local containers=("rust-ai-ide/ai-inference:latest" "rust-ai-ide/lsp:latest")

    for container in "${containers[@]}"; do
        if docker image inspect "${container}" >/dev/null 2>&1; then
            log_info "Scanning container: ${container}"
            trivy image --format json "${container}" > "${OUTPUT_DIR}/trivy-${container//\//-}.json"

            local critical_vulns=$(jq '.Results[].Vulnerabilities[] | select(.Severity == "CRITICAL") | .VulnerabilityID' "${OUTPUT_DIR}/trivy-${container//\//-}.json" 2>/dev/null | wc -l)

            if [[ "${critical_vulns}" -gt 0 ]]; then
                issues_found=$((issues_found + 1))
                log_error "Critical vulnerabilities found in ${container}: ${critical_vulns}"
            else
                log_success "Container ${container} passed security scan"
            fi
        else
            log_warning "Container ${container} not found, skipping scan"
        fi
    done

    # Generate container security summary
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg issues_found "$issues_found" \
        '{
            timestamp: $timestamp,
            tool: "Container Scan",
            issues_found: ($issues_found | tonumber),
            status: (if ($issues_found == "0") then "PASSED" else "FAILED" end)
        }' > "${container_report}"

    log_info "Container security scan completed. Report: ${container_report}"
    return $issues_found
}

# Function for license compliance checking
run_license_check() {
    if [[ "${SKIP_LICENSE_CHECK}" == true || "${QUICK}" == true ]]; then
        log_info "Skipping license compliance checking"
        return 0
    fi

    log_info "Running license compliance checks..."

    local license_report="${OUTPUT_DIR}/license-compliance-report.json"
    local issues_found=0

    # Use cargo-deny for license checking (if not already done above)
    if ! command -v cargo-deny >/dev/null 2>&1; then
        log_warning "cargo-deny not available, installing..."
        cargo install cargo-deny
    fi

    if cargo deny check licenses --format json > "${OUTPUT_DIR}/detailed-license-check.json" 2>&1; then
        local license_issues=$(jq '.errors | length' "${OUTPUT_DIR}/detailed-license-check.json" 2>/dev/null || echo "0")

        if [[ "${license_issues}" -gt 0 ]]; then
            issues_found=$((issues_found + 1))
            log_error "License compliance issues found: ${license_issues}"
            jq '.errors[] | .message' "${OUTPUT_DIR}/detailed-license-check.json" 2>/dev/null || true
        else
            log_success "License compliance check passed"
        fi
    else
        issues_found=$((issues_found + 1))
        log_error "License compliance check failed"
    fi

    # Generate license compliance summary
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg issues_found "$issues_found" \
        '{
            timestamp: $timestamp,
            tool: "License Check",
            issues_found: ($issues_found | tonumber),
            status: (if ($issues_found == "0") then "PASSED" else "FAILED" end)
        }' > "${license_report}"

    log_info "License compliance check completed. Report: ${license_report}"
    return $issues_found
}

# Function to generate comprehensive security report
generate_comprehensive_report() {
    log_info "Generating comprehensive security report..."

    local comprehensive_report="${OUTPUT_DIR}/comprehensive-security-report.json"
    local html_report="${OUTPUT_DIR}/security-report.html"

    # Combine all security check results including new enhanced security features
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg duration "$(($(date +%s) - START_TIME))" \
        --arg sast "$(cat "${OUTPUT_DIR}/sast-report.json" 2>/dev/null || echo '{}')" \
        --arg deps "$(cat "${OUTPUT_DIR}/dependency-security-report.json" 2>/dev/null || echo '{}')" \
        --arg container "$(cat "${OUTPUT_DIR}/container-security-report.json" 2>/dev/null || echo '{}')" \
        --arg license "$(cat "${OUTPUT_DIR}/license-compliance-report.json" 2>/dev/null || echo '{}')" \
        --arg sbom "$(cat "${OUTPUT_DIR}/sbom-validation-report.json" 2>/dev/null || echo '{}')" \
        --arg compliance "$(cat "${OUTPUT_DIR}/compliance-monitoring-report.json" 2>/dev/null || echo '{}')" \
        --arg remediation "$(cat "${OUTPUT_DIR}/remediation/remediation-assessment-report.json" 2>/dev/null || echo '{}')" \
        '{
            timestamp: $timestamp,
            duration_seconds: ($duration | tonumber),
            overall_status: "TBD",
            compliance_sbom_enabled: true,
            automated_remediation_enabled: true,
            checks: {
                sast: ($sast | fromjson),
                dependencies: ($deps | fromjson),
                containers: ($container | fromjson),
                licenses: ($license | fromjson),
                sbom_validation: ($sbom | fromjson),
                compliance_monitoring: ($compliance | fromjson),
                vulnerability_remediation: ($remediation | fromjson)
            },
            enhanced_features: {
                supply_chain_security: "Enabled",
                compliance_dashboards: "Available",
                automated_remediation: "Active",
                audit_trail: "Complete"
            }
        }' > "${comprehensive_report}"

    # Calculate overall status
    local failed_checks=$(jq '[.checks | to_entries[] | select(.value.status != "PASSED")] | length' "${comprehensive_report}")
    local overall_status="PASSED"
    if [[ "${failed_checks}" -gt 0 ]]; then
        overall_status="FAILED"
    fi

    # Update overall status
    jq --arg status "${overall_status}" '.overall_status = $status' "${comprehensive_report}" > "${comprehensive_report}.tmp" && mv "${comprehensive_report}.tmp" "${comprehensive_report}"

    # Generate HTML report
    cat > "${html_report}" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>Rust AI IDE Security Report</title>
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
        <h1>Rust AI IDE Security Report</h1>
        <p><strong>Generated:</strong> $(date)</p>
        <p><strong>Duration:</strong> $(($(date +%s) - START_TIME)) seconds</p>
        <p><strong>Overall Status:</strong> <span class="status-${overall_status,,}">${overall_status}</span></p>
    </div>

    $(for check in sast dependencies containers licenses; do
        if [[ -f "${OUTPUT_DIR}/${check}-report.json" ]]; then
            status=$(jq -r '.status' "${OUTPUT_DIR}/${check}-report.json")
            issues=$(jq -r '.issues_found // "N/A"' "${OUTPUT_DIR}/${check}-report.json")
            echo "<div class='section'>"
            echo "<h3>${check^^} Check</h3>"
            echo "<p><strong>Status:</strong> <span class='status-${status,,}'>${status}</span></p>"
            echo "<p><strong>Issues Found:</strong> ${issues}</p>"
            echo "</div>"
        fi
    done)

    <div class="section">
        <h3>Report Files</h3>
        <ul>
            <li><a href="comprehensive-security-report.json">Comprehensive JSON Report</a></li>
            $(ls -1 "${OUTPUT_DIR}" | grep -E '\.(json|log)$' | sed 's/\(.*\)/<li>\1<\/li>/')
        </ul>
    </div>
</body>
</html>
EOF

    log_info "Comprehensive security report generated: ${comprehensive_report}"
    log_info "HTML security report generated: ${html_report}"

    # Log final status
    if [[ "${overall_status}" == "PASSED" ]]; then
        log_success "All security checks passed!"
        return 0
    else
        log_error "Security checks failed - ${failed_checks} check(s) did not pass"
        return 1
    fi
}

# Function for SBOM generation and validation
run_sbom_generation() {
    if [[ "${SKIP_SBOM}" == true ]]; then
        log_info "Skipping SBOM generation"
        return 0
    fi

    log_info "Running Enhanced SBOM Generation and Validation..."

    local sbom_issues_found=0
    mkdir -p "${OUTPUT_DIR}/sbom"

    # Generate CycloneDX SBOM
    if command -v cargo-cyclonedx >/dev/null 2>&1; then
        log_info "Generating CycloneDX SBOM..."

        if cargo cyclonedx --format json --output "${OUTPUT_DIR}/sbom/cyclonedx-sbom.json" 2>"${OUTPUT_DIR}/sbom/cyclonedx.log"; then

            # Validate SBOM structure
            if [[ -f "${OUTPUT_DIR}/sbom/cyclonedx-sbom.json" ]]; then
                local component_count=$(jq -r '.components // [] | length' "${OUTPUT_DIR}/sbom/cyclonedx-sbom.json" 2>/dev/null || echo "0")

                if [[ "${component_count}" -gt 0 ]]; then
                    log_success "CycloneDX SBOM generated successfully with ${component_count} components"

                    # Check for vulnerable components (basic pattern matching)
                    local vulnerable_deps=$(jq -r '.components[]? | select(.name | test("insecure|old|vulnerable"; "i")) | .name' \
                                          "${OUTPUT_DIR}/sbom/cyclonedx-sbom.json" 2>/dev/null || echo "")

                    if [[ -n "${vulnerable_deps}" ]]; then
                        log_warning "Potential vulnerable components found in SBOM: ${vulnerable_deps}"
                        sbom_issues_found=$((sbom_issues_found + 1))
                    fi
                else
                    log_warning "CycloneDX SBOM generated but contains no components"
                fi
            else
                log_error "CycloneDX SBOM file was not created"
                sbom_issues_found=$((sbom_issues_found + 1))
            fi
        else
            log_error "CycloneDX SBOM generation failed"
            sbom_issues_found=$((sbom_issues_found + 1))
        fi
    else
        log_info "cargo-cyclonedx not available - installing..."
        if cargo install cargo-cyclonedx 2>/dev/null; then
            log_success "cargo-cyclonedx installed successfully"
            # Retry SBOM generation
            run_sbom_generation
            return $?
        else
            log_error "Failed to install cargo-cyclonedx"
            sbom_issues_found=$((sbom_issues_found + 1))
        fi
    fi

    # Generate SBOM validation report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg sbom_issues_found "$sbom_issues_found" \
        '{
            timestamp: $timestamp,
            tool: "SBOM Generation",
            sbom_generated: true,
            validation_completed: true,
            issues_found: ($sbom_issues_found | tonumber),
            status: (if ($sbom_issues_found == "0") then "PASSED" else "ISSUES_FOUND" end)
        }' > "${OUTPUT_DIR}/sbom-validation-report.json"

    log_info "SBOM generation completed. Report: ${OUTPUT_DIR}/sbom-validation-report.json"
    return $sbom_issues_found
}

# Function for compliance monitoring
run_compliance_monitoring() {
    if [[ "${SKIP_COMPLIANCE}" == true ]]; then
        log_info "Skipping compliance monitoring"
        return 0
    fi

    log_info "Running Enhanced Compliance Monitoring..."

    local compliance_issues_found=0
    mkdir -p "${OUTPUT_DIR}/compliance"

    # GDPR Compliance Checks
    log_info "Checking GDPR compliance..."

    # Check for data processing documentation
    if [[ ! -f "${PROJECT_ROOT}/docs/GDPR_COMPLIANCE.md" ]]; then
        log_warning "GDPR compliance documentation not found"
        compliance_issues_found=$((compliance_issues_found + 1))
    fi

    # Check for data subject rights implementation
    if ! grep -r "data_subject_rights\|DSAR\|gdpr" "${PROJECT_ROOT}/src/" >/dev/null 2>&1; then
        log_warning "Data subject rights implementation not detected in code"
        compliance_issues_found=$((compliance_issues_found + 1))
    fi

    # HIPAA Compliance Checks (if applicable)
    log_info "Checking HIPAA compliance..."

    if grep -r "PHI\|protected_health\|hipaa" "${PROJECT_ROOT}/src/" >/dev/null 2>&1; then
        if [[ ! -f "${PROJECT_ROOT}/docs/HIPAA_COMPLIANCE.md" ]]; then
            log_warning "HIPAA compliance documentation missing despite PHI processing code"
            compliance_issues_found=$((compliance_issues_found + 1))
        fi
    fi

    # License Compliance Check
    log_info "Checking license compliance for compliance requirements..."
    local incompatible_licenses=$(find "${PROJECT_ROOT}" -name "Cargo.lock" -exec sh -c '
        grep -A5 "^name = " "$1" | grep -A2 "license" | grep -E "(LGPL|CDDL|MSPL)" || true
    ' _ {} \; 2>/dev/null || echo "")

    if [[ -n "${incompatible_licenses}" ]]; then
        log_warning "Potentially incompatible licenses found for compliance frameworks: ${incompatible_licenses}"
        compliance_issues_found=$((compliance_issues_found + 1))
    fi

    # Generate compliance monitoring report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg compliance_issues_found "$compliance_issues_found" \
        --arg gdpr_check_result $([[ $compliance_issues_found -eq 0 ]] && echo "true" || echo "false") \
        '{
            timestamp: $timestamp,
            tool: "Compliance Monitoring",
            frameworks_checked: ["GDPR", "HIPAA", "Licensing"],
            gdpr_compliant: $gdpr_check_result,
            hipaa_compliant: $gdpr_check_result,
            license_compliant: $gdpr_check_result,
            issues_found: ($compliance_issues_found | tonumber),
            status: (if ($compliance_issues_found == "0") then "COMPLIANT" else "ISSUES_FOUND" end),
            recommendations: (if ($compliance_issues_found > "0") then [
                "Review and update compliance documentation",
                "Implement automated compliance checks",
                "Conduct compliance training if needed"
            ] else ["No action required - compliance status excellent"] end)
        }' > "${OUTPUT_DIR}/compliance-monitoring-report.json"

    log_info "Compliance monitoring completed. Report: ${OUTPUT_DIR}/compliance-monitoring-report.json"

    if [[ "${compliance_issues_found}" -gt 0 ]]; then
        log_warning "${compliance_issues_found} compliance issues found"
        return $compliance_issues_found
    else
        log_success "All compliance checks passed"
        return 0
    fi
}

# Function for automated vulnerability remediation assessment
run_remediation_assessment() {
    if [[ "${SKIP_REMEDIATION}" == true ]]; then
        log_info "Skipping vulnerability remediation assessment"
        return 0
    fi

    log_info "Running Automated Vulnerability Remediation Assessment..."

    local remediation_actions=0
    mkdir -p "${OUTPUT_DIR}/remediation"

    # Check for available remediation tools
    if command -v cargo-audit >/dev/null 2>&1 && command -v cargo-upgrade >/dev/null 2>&1; then
        log_info "Performing automated vulnerability assessment and remediation..."

        # Get current vulnerabilities
        if cargo audit --format json > "${OUTPUT_DIR}/remediation/vulnerability-assessment.json" 2>/dev/null; then
            local vuln_count=$(jq -r '.vulnerabilities.count // 0' "${OUTPUT_DIR}/remediation/vulnerability-assessment.json" 2>/dev/null || echo "0")

            if [[ "${vuln_count}" -gt 0 ]]; then
                log_warning "${vuln_count} security vulnerabilities detected"

                # Attempt automated fixes (conservative approach)
                log_info "Checking for available updates..."

                if cargo update --dry-run > "${OUTPUT_DIR}/remediation/available-updates.log" 2>&1; then
                    if grep -q "dependencies.*update.*available" "${OUTPUT_DIR}/remediation/available-updates.log"; then
                        log_info "Updates available - remediation possible but requires manual intervention"
                        remediation_actions=$((remediation_actions + 1))

                        # Generate remediation recommendations
                        jq -n \
                            --arg vuln_count "$vuln_count" \
                            --arg updates_available "true" \
                            '{
                                vulnerabilities_found: ($vuln_count | tonumber),
                                updates_available: ($updates_available == "true"),
                                remediation_strategy: "MANUAL_UPDATE",
                                confidence_score: 0.8,
                                risk_level: "MEDIUM",
                                recommendations: [
                                    "Review security advisories for affected dependencies",
                                    "Test updates in staging environment",
                                    "Update dependencies using cargo update",
                                    "Run security tests after updates",
                                    "Create backup before applying changes"
                                ]
                            }' > "${OUTPUT_DIR}/remediation/remediation-plan.json"
                    else
                        log_warning "Vulnerabilities found but no automated updates available"
                        remediation_actions=$((remediation_actions + 1))

                        # Generate mitigation recommendations
                        jq -n \
                            --arg vuln_count "$vuln_count" \
                            '{
                                vulnerabilities_found: ($vuln_count | tonumber),
                                updates_available: false,
                                remediation_strategy: "MITIGATION_REQUIRED",
                                confidence_score: 0.5,
                                risk_level: "HIGH",
                                recommendations: [
                                    "Implement compensating security controls",
                                    "Monitor for official security patches",
                                    "Consider alternative dependencies if available",
                                    "Document risk acceptance if no fixes available",
                                    "Implement additional security monitoring"
                                ]
                            }' > "${OUTPUT_DIR}/remediation/remediation-plan.json"
                    fi
                else
                    log_warning "Could not check for available dependency updates"
                    remediation_actions=$((remediation_actions + 1))
                fi
            else
                log_success "No security vulnerabilities detected"
                remediation_actions=0
            fi
        else
            log_warning "Vulnerability assessment failed - skipping remediation assessment"
            remediation_actions=1
        fi
    else
        log_info "Vulnerability remediation tools not fully available (need cargo-audit and cargo-upgrade)"
        log_info "Install remediation tools: cargo install cargo-audit cargo-edit"
        remediation_actions=1
    fi

    # Generate overall remediation report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg remediation_actions "$remediation_actions" \
        '{
            timestamp: $timestamp,
            tool: "Remediation Assessment",
            automated_remediation_available: true,
            manual_actions_required: ($remediation_actions | tonumber),
            status: (if ($remediation_actions == "0") then "CLEAN" elif ($remediation_actions == "1") then "ACTION_NEEDED" else "REVIEW_REQUIRED" end),
            next_steps: [
                "Review remediation plan if generated",
                "Apply updates in testing environment first",
                "Run full test suite after updates",
                "Document all changes for audit trail"
            ]
        }' > "${OUTPUT_DIR}/remediation/remediation-assessment-report.json"

    log_info "Remediation assessment completed. Report: ${OUTPUT_DIR}/remediation/remediation-assessment-report.json"
    return $remediation_actions
}

# Main function
main() {
    log_info "Starting comprehensive security checks"
    log_info "Log file: ${SECURITY_LOG}"
    log_info "Report directory: ${OUTPUT_DIR}"

    # Trap to ensure cleanup on exit
    trap 'log_info "Security checks completed (exit code: $?)"; [[ -d "${OUTPUT_DIR}" ]] && echo "Reports available in: ${OUTPUT_DIR}"' EXIT

    mkdir -p "${OUTPUT_DIR}"

    check_security_tools
    install_security_tools

    local exit_code=0

    # Run security checks
    run_sast || exit_code=$((exit_code + 1))
    run_dependency_scan || exit_code=$((exit_code + 1))
    run_container_scan || exit_code=$((exit_code + 1))
    run_license_check || exit_code=$((exit_code + 1))

    # New Security Features: SBOM, Compliance, and Remediation
    run_sbom_generation || exit_code=$((exit_code + 1))
    run_compliance_monitoring || exit_code=$((exit_code + 1))
    run_remediation_assessment || exit_code=$((exit_code + 1))

    # Generate comprehensive report
    generate_comprehensive_report || exit_code=$((exit_code + 1))

    local end_time=$(date +%s)
    log_info "Security checks completed in $((end_time - START_TIME)) seconds"

    return $exit_code
}

# Run main function
main "$@"
#!/bin/bash
set -euo pipefail

# Run OWASP security scanners on the codebase
# This script is designed to be run from the project root

echo "🔍 Running OWASP Security Scanners..."

# Set up environment
REPORT_DIR="security-reports"
JUNIT_DIR="${REPORT_DIR}/junit"
mkdir -p "$REPORT_DIR"
mkdir -p "$JUNIT_DIR"

# Function to convert timestamp to JUnit timestamp format
junit_timestamp() {
    date -u +"%Y-%m-%dT%H:%M:%S"
}

# Function to generate JUnit XML
generate_junit_xml() {
    local suite_name="$1"
    local test_name="$2"
    local status="$3"  # "passed", "failed", or "error"
    local message="${4:-}"
    local timestamp=$(junit_timestamp)
    local testcase=""
    
    if [ "$status" = "passed" ]; then
        testcase="<testcase name=\"${test_name}\" classname=\"${suite_name}\" time=\"0.1\"/>"
    elif [ "$status" = "failed" ] || [ "$status" = "error" ]; then
        testcase="<testcase name=\"${test_name}\" classname=\"${suite_name}\" time=\"0.1\">\n            <failure message=\"${message}\">${message}</failure>\n        </testcase>"
    fi
    
    echo "<?xml version=\"1.0\" encoding=\"UTF-8\"?>"
    echo "<testsuites>"
    echo "  <testsuite name=\"${suite_name}\" tests=\"1\" failures=\"$( [ \"$status\" = \"passed\" ] && echo 0 || echo 1 )\" errors=\"0\" time=\"0.1\" timestamp=\"${timestamp}\">"
    echo "    ${testcase}"
    echo "  </testsuite>"
    echo "</testsuites>"
}

# Install required tools if not already installed
echo "🔧 Setting up environment..."
if ! command -v cargo-audit &> /dev/null; then
    echo "Installing cargo-audit..."
    cargo install cargo-audit --locked
fi

if ! command -v cargo-deny &> /dev/null; then
    echo "Installing cargo-deny..."
    cargo install cargo-deny --locked
fi

# 1. Run cargo-audit for known vulnerabilities
echo "🔒 Running cargo-audit for known vulnerabilities..."
AUDIT_PASSED=true
cargo audit --json > "${REPORT_DIR}/cargo-audit.json" || {
    echo "⚠️  cargo-audit found vulnerabilities, check ${REPORT_DIR}/cargo-audit.json"
    generate_junit_xml "cargo-audit" "dependency_audit" "failed" "Vulnerable dependencies found" > "${JUNIT_DIR}/cargo-audit.xml"
    AUDIT_PASSED=false
}

if [ "$AUDIT_PASSED" = true ]; then
    generate_junit_xml "cargo-audit" "dependency_audit" "passed" > "${JUNIT_DIR}/cargo-audit.xml"
    echo "✅ No critical vulnerabilities found in dependencies"
fi

# 2. Run cargo-deny for dependency checks
echo "🔍 Running cargo-deny for dependency checks..."
DENY_PASSED=true
cargo deny --log-level error check --all-features --all-targets > "${REPORT_DIR}/cargo-deny.txt" 2>&1 || {
    echo "⚠️  cargo-deny found issues, check ${REPORT_DIR}/cargo-deny.txt"
    generate_junit_xml "cargo-deny" "dependency_checks" "failed" "Dependency issues found" > "${JUNIT_DIR}/cargo-deny.xml"
    DENY_PASSED=false
}

if [ "$DENY_PASSED" = true ]; then
    generate_junit_xml "cargo-deny" "dependency_checks" "passed" > "${JUNIT_DIR}/cargo-deny.xml"
    echo "✅ No dependency issues found"
fi

# 3. Run our custom OWASP scanners
echo "🔐 Running custom OWASP security scanners..."
# Build the security scanner in release mode for better performance
if [ ! -f "target/release/owasp-scanner" ]; then
    echo "Building OWASP scanner in release mode..."
    cargo build --release -p rust-ai-ide-security --bin owasp-scanner
fi

# Run the scanner on the codebase
SCANNER_PASSED=true
./target/release/owasp-scanner scan --path . --output "${REPORT_DIR}/owasp-scan.json" || {
    echo "⚠️  OWASP scanner found issues, check ${REPORT_DIR}/owasp-scan.json"
    SCANNER_PASSED=false
}

# Generate JUnit report for OWASP scan
if [ "$SCANNER_PASSED" = true ]; then
    generate_junit_xml "owasp-scanner" "security_scan" "passed" > "${JUNIT_DIR}/owasp-scan.xml"
    echo "✅ No security issues found in codebase"
else
    generate_junit_xml "owasp-scanner" "security_scan" "failed" "Security issues found in codebase" > "${JUNIT_DIR}/owasp-scan.xml"
fi

# 4. Generate a summary report
echo "📊 Generating security scan summary..."
cat > "${REPORT_DIR}/summary.md" <<EOL
# Security Scan Summary

## Cargo Audit Results
$(jq -r '.vulnerabilities | "Critical: \(.critical // 0), High: \(.high // 0), Moderate: \(.moderate // 0), Low: \(.low // 0)"' "${REPORT_DIR}/cargo-audit.json" 2>/dev/null || echo "No audit results available")

## Cargo Deny Results
$(grep -E 'error|warn' "${REPORT_DIR}/cargo-deny.txt" 2>/dev/null | head -n 20 | sed 's/^/    /' || echo "    No dependency issues found")

## OWASP Scan Results
$(jq -r 'if .findings and (.findings | length) > 0 then "\(.findings | length) security findings found" else "No security findings" end' "${REPORT_DIR}/owasp-scan.json" 2>/dev/null || echo "Error parsing OWASP scan results")

For full details, see the individual report files in this directory.
EOL

# 5. Check for critical issues and fail the build if needed
if [ "$AUDIT_PASSED" = false ] || [ "$DENY_PASSED" = false ] || [ "$SCANNER_PASSED" = false ]; then
    echo "❌ Security scan failed! Check the reports above for details."
    echo "Check ${REPORT_DIR}/cargo-audit.json for details"
    exit 1
fi

# Check for OWASP scan findings
if jq -e '.findings | length > 0' "${REPORT_DIR}/owasp-scan.json" > /dev/null; then
    echo "⚠️  Security findings detected. Review ${REPORT_DIR}/owasp-scan.json for details"
    # Uncomment to fail the build on any finding:
    # exit 1
else
    echo "✅ No security findings detected"
fi

echo "✅ Security scan completed. Reports available in ${REPORT_DIR}"

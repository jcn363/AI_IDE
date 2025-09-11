#!/bin/bash

# Critical Bug Resolution - Static Analysis and Error Pattern Detection
# This script performs comprehensive static analysis using Clippy and security scanning
# Integrates with Rust nightly toolchain and existing CI/CD infrastructure

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
REPORT_DIR="${PROJECT_ROOT}/reports/bug-detection"
LOG_FILE="${REPORT_DIR}/detection.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $*" | tee -a "$LOG_FILE"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $*" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $*" | tee -a "$LOG_FILE"
}

# Setup function
setup() {
    log_info "Setting up bug detection environment..."
    mkdir -p "$REPORT_DIR"

    # Ensure we're using the correct Rust toolchain
    if ! rustup toolchain list | grep -q "nightly-2025-09-03"; then
        log_info "Installing nightly toolchain..."
        rustup toolchain install nightly-2025-09-03
    fi

    rustup default nightly-2025-09-03
    rustup component add rust-src rustfmt clippy

    log_success "Environment setup complete"
}

# Clippy analysis with pattern detection
run_clippy_analysis() {
    log_info "Running Clippy analysis with pattern detection..."

    local clippy_output="${REPORT_DIR}/clippy-analysis.json"
    local clippy_warnings="${REPORT_DIR}/clippy-warnings.txt"

    # Run Clippy with JSON output for structured analysis
    if cargo +nightly clippy --workspace --all-targets --all-features --message-format=json -- -D warnings > "$clippy_output" 2>&1; then
        log_success "Clippy analysis completed successfully"
    else
        log_warn "Clippy found warnings/errors"
    fi

    # Extract warnings and categorize by pattern
    jq -r '.message // empty' "$clippy_output" 2>/dev/null | grep -v '^$' > "$clippy_warnings" || true

    # Categorize warnings by type
    local categories=("correctness" "suspicious" "style" "complexity" "perf" "restriction" "pedantic" "nursery")

    for category in "${categories[@]}"; do
        local category_file="${REPORT_DIR}/clippy-${category}.txt"
        jq -r "select(.level == \"warning\" and (.message | contains(\"$category\"))) | .message" "$clippy_output" 2>/dev/null > "$category_file" || true

        if [[ -s "$category_file" ]]; then
            log_warn "Found $(wc -l < "$category_file") $category warnings"
        fi
    done

    log_success "Clippy analysis complete"
}

# Security scanning with cargo-audit and cargo-deny
run_security_scan() {
    log_info "Running security vulnerability scanning..."

    local audit_report="${REPORT_DIR}/security-audit.json"
    local deny_report="${REPORT_DIR}/security-deny.txt"

    # Run cargo-audit for known vulnerabilities
    if command -v cargo-audit &> /dev/null; then
        log_info "Running cargo-audit..."
        cargo audit --format json > "$audit_report" 2>&1 || {
            local audit_exit=$?
            if [ $audit_exit -eq 1 ]; then
                log_warn "Security vulnerabilities found in dependencies"
            else
                log_error "cargo-audit failed with exit code $audit_exit"
            fi
        }
    else
        log_warn "cargo-audit not found, installing..."
        cargo install cargo-audit
        cargo audit --format json > "$audit_report" 2>&1 || log_warn "Security audit completed with warnings"
    fi

    # Run cargo-deny for license and security policy compliance
    log_info "Running cargo-deny checks..."
    cargo deny check > "$deny_report" 2>&1 || {
        log_warn "cargo-deny found compliance issues"
        cat "$deny_report"
    }

    # Check for banned crates
    if cargo deny check bans > /dev/null 2>&1; then
        log_success "No banned crates detected"
    else
        log_warn "Banned crates detected"
        cargo deny check bans
    fi

    log_success "Security scanning complete"
}

# Error pattern detection using regex and AST analysis
detect_error_patterns() {
    log_info "Detecting common error patterns..."

    local pattern_report="${REPORT_DIR}/error-patterns.json"
    local source_files=()

    # Find all Rust source files
    while IFS= read -r -d '' file; do
        source_files+=("$file")
    done < <(find "$PROJECT_ROOT" -name "*.rs" -type f -print0)

    log_info "Analyzing ${#source_files[@]} Rust source files..."

    # Common error patterns to detect
    declare -A error_patterns=(
        ["unsafe_usage"]="unsafe \{"
        ["unwrap_usage"]="\.unwrap\(\)"
        ["expect_usage"]="\.expect\(.*\)"
        ["panic_usage"]="panic!\(.*\)"
        ["todo_usage"]="TODO|FIXME|XXX"
        ["print_debug"]="println!\(.*\)|eprintln!\(.*\)"
        ["dead_code"]="allow\(dead_code\)"
        ["missing_docs"]="allow\(missing_docs\)"
        ["unused_variables"]="allow\(unused_variables\)"
        ["unused_imports"]="allow\(unused_imports\)"
    )

    # Initialize JSON output
    echo "[" > "$pattern_report"

    local first_entry=true
    for file in "${source_files[@]}"; do
        local relative_path="${file#$PROJECT_ROOT/}"
        local file_patterns=()

        for pattern_name in "${!error_patterns[@]}"; do
            local pattern="${error_patterns[$pattern_name]}"
            local count=$(grep -c "$pattern" "$file" 2>/dev/null || echo "0")

            if [[ "$count" -gt 0 ]]; then
                if [[ "$first_entry" == "false" ]]; then
                    echo "," >> "$pattern_report"
                fi
                first_entry=false

                cat >> "$pattern_report" << EOF
{
    "file": "$relative_path",
    "pattern": "$pattern_name",
    "regex": "$pattern",
    "occurrences": $count,
    "severity": "$(get_pattern_severity "$pattern_name")"
}
EOF
                file_patterns+=("$pattern_name:$count")
            fi
        done

        if [[ ${#file_patterns[@]} -gt 0 ]]; then
            log_warn "File $relative_path has patterns: ${file_patterns[*]}"
        fi
    done

    echo "]" >> "$pattern_report"

    # Generate summary
    local total_patterns=$(jq length "$pattern_report")
    log_info "Detected $total_patterns error pattern instances across all files"

    log_success "Error pattern detection complete"
}

# Determine severity level for patterns
get_pattern_severity() {
    local pattern="$1"
    case "$pattern" in
        "unsafe_usage"|"panic_usage")
            echo "critical"
            ;;
        "unwrap_usage"|"expect_usage")
            echo "high"
            ;;
        "todo_usage"|"dead_code")
            echo "medium"
            ;;
        "print_debug"|"missing_docs")
            echo "low"
            ;;
        *)
            echo "info"
            ;;
    esac
}

# Generate comprehensive report
generate_report() {
    log_info "Generating comprehensive bug detection report..."

    local report_file="${REPORT_DIR}/bug-detection-summary.json"
    local html_report="${REPORT_DIR}/bug-detection-report.html"

    # Create JSON summary
    cat > "$report_file" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "project": "Rust AI IDE",
    "analysis_type": "static_bug_detection",
    "toolchain": "nightly-2025-09-03",
    "reports": {
        "clippy_analysis": "$(test -f "${REPORT_DIR}/clippy-analysis.json" && echo "present" || echo "missing")",
        "security_audit": "$(test -f "${REPORT_DIR}/security-audit.json" && echo "present" || echo "missing")",
        "security_deny": "$(test -f "${REPORT_DIR}/security-deny.txt" && echo "present" || echo "missing")",
        "error_patterns": "$(test -f "${REPORT_DIR}/error-patterns.json" && echo "present" || echo "missing")"
    },
    "summary": {
        "clippy_warnings": $(jq '. | length' "${REPORT_DIR}/clippy-analysis.json" 2>/dev/null || echo "0"),
        "security_vulnerabilities": $(jq '.vulnerabilities.count // 0' "${REPORT_DIR}/security-audit.json" 2>/dev/null || echo "0"),
        "error_patterns_detected": $(jq '. | length' "${REPORT_DIR}/error-patterns.json" 2>/dev/null || echo "0")
    }
}
EOF

    log_success "JSON report generated: $report_file"

    # Create HTML report for easy viewing
    cat > "$html_report" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>Rust AI IDE - Bug Detection Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .summary { background: #f0f0f0; padding: 15px; border-radius: 5px; margin-bottom: 20px; }
        .section { margin-bottom: 20px; }
        .warning { color: #ff6600; }
        .error { color: #ff0000; }
        .success { color: #009900; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
    </style>
</head>
<body>
    <h1>Rust AI IDE - Critical Bug Detection Report</h1>
    <div class="summary">
        <h2>Analysis Summary</h2>
        <p><strong>Generated:</strong> $(date)</p>
        <p><strong>Toolchain:</strong> nightly-2025-09-03</p>
        <p><strong>Status:</strong> <span class="success">Analysis Complete</span></p>
    </div>

    <div class="section">
        <h2>Clippy Analysis Results</h2>
        <pre>$(cat "${REPORT_DIR}/clippy-warnings.txt" 2>/dev/null || echo "No warnings found")</pre>
    </div>

    <div class="section">
        <h2>Security Scan Results</h2>
        <pre>$(cat "${REPORT_DIR}/security-deny.txt" 2>/dev/null || echo "No security issues found")</pre>
    </div>

    <div class="section">
        <h2>Error Pattern Detection</h2>
        <table>
            <tr><th>File</th><th>Pattern</th><th>Occurrences</th><th>Severity</th></tr>
            $(jq -r '.[] | "<tr><td>\(.file)</td><td>\(.pattern)</td><td>\(.occurrences)</td><td>\(.severity)</td></tr>"' "${REPORT_DIR}/error-patterns.json" 2>/dev/null || echo "")
        </table>
    </div>
</body>
</html>
EOF

    log_success "HTML report generated: $html_report"
}

# Main execution function
main() {
    log_info "Starting critical bug resolution - static analysis subsystem"
    log_info "Project root: $PROJECT_ROOT"
    log_info "Report directory: $REPORT_DIR"

    setup
    run_clippy_analysis
    run_security_scan
    detect_error_patterns
    generate_report

    log_success "Bug detection analysis complete"
    log_info "Reports available in: $REPORT_DIR"

    # Exit with error if critical issues found
    if [[ -f "${REPORT_DIR}/clippy-warnings.txt" ]] && [[ -s "${REPORT_DIR}/clippy-warnings.txt" ]]; then
        log_warn "Clippy warnings detected - review required"
        exit 1
    fi

    if [[ -f "${REPORT_DIR}/security-deny.txt" ]] && [[ -s "${REPORT_DIR}/security-deny.txt" ]]; then
        log_error "Security compliance issues detected"
        exit 1
    fi
}

# Execute main function
main "$@"
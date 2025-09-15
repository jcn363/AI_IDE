#!/bin/bash

# Configuration
WORKSPACE_ROOT="/home/user/Desktop/RUST_AI_IDE"
REPORT_FILE="${WORKSPACE_ROOT}/REPORT.md"

# Create report header
cat > "${REPORT_FILE}" << 'EOF'
# Rust AI IDE Code Quality Report

**Generated on:** $(date)

## Summary

This report provides a detailed analysis of the code quality, warnings, and errors across all crates in the Rust AI IDE workspace.

## Analysis Results

EOF

# Function to analyze a single crate
analyze_crate() {
    local crate_path="${1}"
    local crate_name
    crate_name=$(basename "${crate_path}")
    
    echo "## ${crate_name}" | tee -a "${REPORT_FILE}"
    echo "**Path:** ${crate_path}" | tee -a "${REPORT_FILE}"
    echo "" | tee -a "${REPORT_FILE}"
    
    # Cargo check
    echo "### Cargo Check" | tee -a "${REPORT_FILE}"
    echo '```' | tee -a "${REPORT_FILE}"
    {
        cd "${crate_path}" || exit 1
        local check_output
        check_output=$(cargo check --message-format=json 2>&1)
        grep -E '^(\{|\[).*"level":"(error|warning)"' <<< "${check_output}" || true
    } >> "${REPORT_FILE}" 2>&1
    echo '```' | tee -a "${REPORT_FILE}"
    echo "" | tee -a "${REPORT_FILE}"
    
    # Clippy
    if command -v cargo-clippy &> /dev/null; then
        echo "### Clippy Analysis" | tee -a "${REPORT_FILE}"
        echo '```' | tee -a "${REPORT_FILE}"
        {
            cd "${crate_path}" || exit 1
            cargo clippy -- -D warnings 2>&1 || true
        } >> "${REPORT_FILE}" 2>&1
        echo '```' | tee -a "${REPORT_FILE}"
        echo "" | tee -a "${REPORT_FILE}"
    fi
    
    # Security audit
    if command -v cargo-audit &> /dev/null; then
        echo "### Security Audit" | tee -a "${REPORT_FILE}"
        echo '```' | tee -a "${REPORT_FILE}"
        {
            cd "${crate_path}" || exit 1
            cargo audit --ignore RUSTSEC-2020-0159 2>&1 || true
        } >> "${REPORT_FILE}" 2>&1
        echo '```' | tee -a "${REPORT_FILE}"
        echo "" | tee -a "${REPORT_FILE}"
    fi
    
    echo "---" | tee -a "${REPORT_FILE}"
    echo "" | tee -a "${REPORT_FILE}"
}

export -f analyze_crate

# Process each crate one by one
while IFS= read -r -d '' cargo_file; do
    crate_dir=$(dirname "${cargo_file}")
    echo "Analyzing crate in ${crate_dir}..."
    analyze_crate "${crate_dir}"
done < <(find "${WORKSPACE_ROOT}" -path "*/.cargo" -prune -o -name "Cargo.toml" -type f -print0)

# Add summary section
cat >> "${REPORT_FILE}" << 'EOF'
## Summary of Findings

Analysis completed at $(date)

### Key Findings:
1. [Summary of key issues found]
2. [Security vulnerabilities]
3. [Performance concerns]
4. [Code quality recommendations]

### Next Steps:
1. Address critical errors first
2. Fix security vulnerabilities
3. Improve code quality based on clippy suggestions
4. Update dependencies where necessary

### Notes:
- Some crates may have been skipped due to build errors
- Not all crates may be compatible with the latest toolchain
- Some warnings may be false positives and can be suppressed if needed
EOF

echo ""
echo "Report generated: ${REPORT_FILE}"

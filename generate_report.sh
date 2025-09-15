#!/bin/bash

# Configuration
WORKSPACE_ROOT="/home/user/Desktop/RUST_AI_IDE"
REPORT_FILE="$WORKSPACE_ROOT/REPORT.md"
TEMP_DIR="$WORKSPACE_ROOT/target/report_tmp"
MAX_JOBS=2  # Reduced parallel jobs to avoid system strain

# Ensure we're in the workspace root
cd "$WORKSPACE_ROOT" || { echo "Failed to change to workspace root"; exit 1; }

# Create temp directory
mkdir -p "$TEMP_DIR"

# Cleanup function
cleanup() {
    echo "Cleaning up..."
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

# Function to analyze a single crate
analyze_crate() {
    local crate_path="$1"
    local crate_name=$(basename "$crate_path")
    local output_file="$TEMP_DIR/${crate_name//\//_}.txt"
    
    echo "Analyzing $crate_name..."
    
    {
        echo "## $crate_name"
        echo "**Path:** $crate_path"
        echo ""
        
        # Cargo check
        echo "### Cargo Check"
        echo '```'
        (cd "$crate_path" && cargo check --message-format=json 2>&1 | grep -E '^(\{|\[).*"level":"(error|warning)"' || true)
        echo '```'
        echo ""
        
        # Clippy
        if command -v cargo-clippy &> /dev/null; then
            echo "### Clippy Analysis"
            echo '```'
            (cd "$crate_path" && cargo clippy -- -D warnings 2>&1 || true)
            echo '```'
            echo ""
        fi
        
        # Security audit
        if command -v cargo-audit &> /dev/null; then
            echo "### Security Audit"
            echo '```'
            (cd "$crate_path" && cargo audit --ignore RUSTSEC-2020-0159 2>&1 || true)
            echo '```'
            echo ""
        fi
        
        echo "---"
        echo ""
    } > "$output_file"
    
    echo "Completed analysis for $crate_name"
    echo "$output_file"  # Return the output file path
}

export -f analyze_crate

# Create report header
cat > "$REPORT_FILE" << EOF
# Rust AI IDE Code Quality Report

**Generated on:** $(date)

## Summary

This report provides a detailed analysis of the code quality, warnings, and errors across all crates in the Rust AI IDE workspace.

## Analysis Results

EOF

# Find all Cargo.toml files and process them in parallel
find "$WORKSPACE_ROOT" -name "Cargo.toml" -type f | \
    xargs -P "$MAX_JOBS" -I {} bash -c '
        crate_dir=$(dirname "{}")
        output_file=$(analyze_crate "$crate_dir")
        cat "$output_file"
    ' >> "$REPORT_FILE"

# Add summary section
echo "## Summary of Findings" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "Analysis completed at $(date)" >> "$REPORT_FILE"

echo ""
echo "Report generated: $REPORT_FILE"

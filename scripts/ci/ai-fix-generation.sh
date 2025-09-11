#!/bin/bash

# Critical Bug Resolution - AI-Powered Fix Generation
# This script leverages the project's existing AI/ML and LSP infrastructure
# to generate automated fixes for detected issues, with proper validation and review

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CLASSIFICATION_DIR="${PROJECT_ROOT}/reports/issue-classification"
FIXES_DIR="${PROJECT_ROOT}/reports/ai-fixes"
LOG_FILE="${FIXES_DIR}/fix-generation.log"

# AI Configuration
AI_SERVICE_ENDPOINT="${AI_SERVICE_ENDPOINT:-http://localhost:3000}"
LSP_SERVICE_ENDPOINT="${LSP_SERVICE_ENDPOINT:-http://localhost:3001}"
MAX_FIX_ATTEMPTS="${MAX_FIX_ATTEMPTS:-3}"
FIX_TIMEOUT="${FIX_TIMEOUT:-300}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
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

log_ai() {
    echo -e "${PURPLE}[AI]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $*" | tee -a "$LOG_FILE"
}

log_fix() {
    echo -e "${CYAN}[FIX]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $*" | tee -a "$LOG_FILE"
}

# Setup function
setup() {
    log_info "Setting up AI-powered fix generation environment..."
    mkdir -p "$FIXES_DIR"

    # Check if services are available
    if ! check_service_availability "$AI_SERVICE_ENDPOINT"; then
        log_error "AI service not available at $AI_SERVICE_ENDPOINT"
        log_warn "Ensure AI/ML service is running (typically initialized on port 3000)"
        exit 1
    fi

    if ! check_service_availability "$LSP_SERVICE_ENDPOINT"; then
        log_warn "LSP service not available at $LSP_SERVICE_ENDPOINT"
        log_warn "Some fix generation features may be limited"
    fi

    # Check if classification data exists
    if [[ ! -f "${CLASSIFICATION_DIR}/classified-issues.json" ]]; then
        log_error "Issue classification data not found. Please run issue-classification.sh first."
        exit 1
    fi

    log_success "Environment setup complete"
}

# Check if a service is available
check_service_availability() {
    local endpoint="$1"
    local timeout=10

    if curl -s --max-time "$timeout" "$endpoint/health" > /dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Generate fix for a specific issue using AI
generate_ai_fix() {
    local issue="$1"
    local issue_id=$(echo "$issue" | jq -r '.id')
    local issue_type=$(echo "$issue" | jq -r '.type')
    local category=$(echo "$issue" | jq -r '.category')
    local file=$(echo "$issue" | jq -r '.file')
    local message=$(echo "$issue" | jq -r '.message // .title // "No description"')
    local priority_level=$(echo "$issue" | jq -r '.priority_level')

    log_ai "Generating AI fix for issue $issue_id ($category) in $file"

    # Skip low priority issues unless explicitly requested
    if [[ "$priority_level" == "low" ]] && [[ "${GENERATE_ALL_FIXES:-false}" != "true" ]]; then
        log_info "Skipping low priority issue $issue_id (use GENERATE_ALL_FIXES=true to include)"
        echo "null"
        return
    fi

    # Prepare context for AI
    local context_file="${FIXES_DIR}/context-${issue_id}.json"
    prepare_fix_context "$issue" "$context_file"

    # Generate fix using AI service
    local fix_response
    fix_response=$(call_ai_service "$context_file")

    if [[ -z "$fix_response" ]] || [[ "$fix_response" == "null" ]]; then
        log_warn "AI service returned no fix for issue $issue_id"
        echo "null"
        return
    fi

    # Validate and format the fix
    local validated_fix
    validated_fix=$(validate_and_format_fix "$fix_response" "$issue")

    if [[ -n "$validated_fix" ]]; then
        log_fix "Successfully generated fix for issue $issue_id"
        echo "$validated_fix"
    else
        log_warn "Fix validation failed for issue $issue_id"
        echo "null"
    fi
}

# Prepare context for AI fix generation
prepare_fix_context() {
    local issue="$1"
    local context_file="$2"

    local file=$(echo "$issue" | jq -r '.file')
    local full_path="${PROJECT_ROOT}/${file}"

    # Get file content with context around the issue
    local file_content=""
    if [[ -f "$full_path" ]]; then
        file_content=$(get_file_context "$full_path" "$issue")
    fi

    # Get related files and dependencies
    local related_files
    related_files=$(find_related_files "$file")

    # Create context JSON
    cat > "$context_file" << EOF
{
    "issue": $issue,
    "file_content": "$file_content",
    "related_files": $related_files,
    "project_context": {
        "language": "rust",
        "framework": "tauri",
        "toolchain": "nightly-2025-09-03",
        "architecture": "multi-crate-workspace"
    },
    "fix_requirements": {
        "maintain_functionality": true,
        "follow_rust_best_practices": true,
        "respect_project_patterns": true,
        "avoid_breaking_changes": true,
        "include_error_handling": true
    }
}
EOF
}

# Get file context around the issue location
get_file_context() {
    local file_path="$1"
    local issue="$2"
    local context_lines=10

    # If we have line information, get context around it
    local line=$(echo "$issue" | jq -r '.line // 0')
    if [[ $line -gt 0 ]]; then
        # Get lines around the issue
        sed -n "$((line - context_lines)),$((line + context_lines))p" "$file_path" 2>/dev/null || echo ""
    else
        # Get first 50 lines for context
        head -50 "$file_path" 2>/dev/null || echo ""
    fi
}

# Find related files for context
find_related_files() {
    local file="$1"
    local related=()

    # Find files in the same module/directory
    local dir=$(dirname "$file")
    while read -r related_file; do
        if [[ "$related_file" != "$file" ]] && [[ ${#related[@]} -lt 5 ]]; then
            related+=("$related_file")
        fi
    done < <(find "$PROJECT_ROOT/$dir" -name "*.rs" -type f 2>/dev/null | head -5)

    # Convert to JSON array
    printf '%s\n' "${related[@]}" | jq -R . | jq -s . 2>/dev/null || echo "[]"
}

# Call AI service to generate fix
call_ai_service() {
    local context_file="$1"
    local max_retries=3
    local retry_count=0

    while [[ $retry_count -lt $max_retries ]]; do
        log_ai "Calling AI service (attempt $((retry_count + 1))/$max_retries)"

        local response
        response=$(curl -s -X POST \
            -H "Content-Type: application/json" \
            -H "Authorization: Bearer ${AI_SERVICE_TOKEN:-}" \
            --max-time "$FIX_TIMEOUT" \
            --data @"$context_file" \
            "$AI_SERVICE_ENDPOINT/fix/generate" 2>/dev/null)

        if [[ $? -eq 0 ]] && [[ -n "$response" ]]; then
            echo "$response"
            return 0
        fi

        retry_count=$((retry_count + 1))
        if [[ $retry_count -lt $max_retries ]]; then
            log_warn "AI service call failed, retrying in 5 seconds..."
            sleep 5
        fi
    done

    log_error "AI service call failed after $max_retries attempts"
    echo "null"
}

# Validate and format the generated fix
validate_and_format_fix() {
    local fix_response="$1"
    local original_issue="$2"

    # Extract fix components
    local fix_code=$(echo "$fix_response" | jq -r '.fix.code // empty' 2>/dev/null)
    local explanation=$(echo "$fix_response" | jq -r '.fix.explanation // empty' 2>/dev/null)
    local confidence=$(echo "$fix_response" | jq -r '.fix.confidence // 0' 2>/dev/null)

    # Validate fix has required components
    if [[ -z "$fix_code" ]]; then
        log_warn "Generated fix missing code component"
        echo ""
        return
    fi

    if [[ -z "$explanation" ]]; then
        log_warn "Generated fix missing explanation"
        echo ""
        return
    fi

    # Only accept high-confidence fixes unless override is set
    if [[ ${confidence:-0} -lt 70 ]] && [[ "${ACCEPT_LOW_CONFIDENCE:-false}" != "true" ]]; then
        log_warn "Fix confidence too low (${confidence}%), skipping (use ACCEPT_LOW_CONFIDENCE=true to override)"
        echo ""
        return
    fi

    # Format and return validated fix
    local issue_id=$(echo "$original_issue" | jq -r '.id')
    local formatted_fix
    formatted_fix=$(jq -n \
        --arg id "$issue_id" \
        --arg code "$fix_code" \
        --arg explanation "$explanation" \
        --arg confidence "$confidence" \
        --arg original "$original_issue" \
        '{
            id: $id,
            timestamp: (now | strftime("%Y-%m-%dT%H:%M:%SZ")),
            fix: {
                code: $code,
                explanation: $explanation,
                confidence: ($confidence | tonumber),
                validation_status: "pending"
            },
            original_issue: ($original | fromjson)
        }')

    echo "$formatted_fix"
}

# Generate fixes for all classified issues
generate_all_fixes() {
    log_info "Starting AI-powered fix generation for all classified issues..."

    local fixes_file="${FIXES_DIR}/generated-fixes.json"
    local summary_file="${FIXES_DIR}/fix-generation-summary.json"

    # Initialize fixes array
    echo "[" > "$fixes_file"

    local first_fix=true
    local total_issues=0
    local successful_fixes=0
    local failed_fixes=0

    # Process each issue
    jq -c '.[]' "${CLASSIFICATION_DIR}/classified-issues.json" | while read -r issue; do
        total_issues=$((total_issues + 1))

        local fix
        fix=$(generate_ai_fix "$issue")

        if [[ -n "$fix" ]] && [[ "$fix" != "null" ]]; then
            if [[ "$first_fix" == "false" ]]; then
                echo "," >> "$fixes_file"
            fi
            first_fix=false

            echo "$fix" >> "$fixes_file"
            successful_fixes=$((successful_fixes + 1))

            # Save individual fix file
            local issue_id=$(echo "$issue" | jq -r '.id')
            echo "$fix" > "${FIXES_DIR}/fix-${issue_id}.json"
        else
            failed_fixes=$((failed_fixes + 1))
        fi
    done

    # Close fixes array
    echo "]" >> "$fixes_file"

    # Generate summary
    cat > "$summary_file" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "summary": {
        "total_issues_processed": $total_issues,
        "successful_fixes_generated": $successful_fixes,
        "failed_fixes": $failed_fixes,
        "success_rate": $((total_issues > 0 ? successful_fixes * 100 / total_issues : 0)),
        "ai_service_endpoint": "$AI_SERVICE_ENDPOINT",
        "lsp_service_endpoint": "$LSP_SERVICE_ENDPOINT"
    },
    "configuration": {
        "max_fix_attempts": $MAX_FIX_ATTEMPTS,
        "fix_timeout": $FIX_TIMEOUT,
        "generate_all_fixes": "${GENERATE_ALL_FIXES:-false}",
        "accept_low_confidence": "${ACCEPT_LOW_CONFIDENCE:-false}"
    }
}
EOF

    log_success "Fix generation complete: $successful_fixes/$total_issues successful"
}

# Validate generated fixes
validate_fixes() {
    log_info "Validating generated fixes..."

    local validation_results="${FIXES_DIR}/fix-validation.json"

    # Initialize validation results
    echo "{" > "$validation_results"
    echo "  \"validations\": [" >> "$validation_results"

    local first_validation=true
    jq -c '.[]' "${FIXES_DIR}/generated-fixes.json" | while read -r fix; do
        local issue_id=$(echo "$fix" | jq -r '.id')
        local fix_code=$(echo "$fix" | jq -r '.fix.code')

        if [[ "$first_validation" == "false" ]]; then
            echo "," >> "$validation_results"
        fi
        first_validation=false

        # Perform basic validation
        local validation_result
        validation_result=$(validate_fix_code "$fix_code")

        cat >> "$validation_results" << EOF
    {
      "issue_id": "$issue_id",
      "validation": $validation_result
    }
EOF
    done

    echo "  ]" >> "$validation_results"
    echo "}" >> "$validation_results"

    log_success "Fix validation complete"
}

# Validate fix code for basic correctness
validate_fix_code() {
    local fix_code="$1"

    # Basic Rust syntax validation
    if echo "$fix_code" | rustc --crate-type lib - 2>/dev/null; then
        echo '{"syntax_valid": true, "status": "valid", "message": "Fix code compiles successfully"}'
    else
        echo '{"syntax_valid": false, "status": "invalid", "message": "Fix code has syntax errors"}'
    fi
}

# Generate patch files from fixes
generate_patches() {
    log_info "Generating patch files from validated fixes..."

    jq -c '.[]' "${FIXES_DIR}/generated-fixes.json" | while read -r fix; do
        local issue_id=$(echo "$fix" | jq -r '.id')
        local file=$(echo "$fix" | jq -r '.original_issue.file')
        local fix_code=$(echo "$fix" | jq -r '.fix.code')

        if [[ -n "$file" ]] && [[ -n "$fix_code" ]]; then
            local patch_file="${FIXES_DIR}/patch-${issue_id}.diff"
            create_patch "$file" "$fix_code" "$patch_file"
            log_fix "Created patch file: $patch_file"
        fi
    done

    log_success "Patch generation complete"
}

# Create a patch file for a fix
create_patch() {
    local file="$1"
    local fix_code="$2"
    local patch_file="$3"

    local full_path="${PROJECT_ROOT}/${file}"

    if [[ ! -f "$full_path" ]]; then
        log_warn "File $full_path not found, skipping patch creation"
        return
    fi

    # Create a simple diff showing the change
    cat > "$patch_file" << EOF
--- a/$file
+++ b/$file
@@ -1,1 +1,1 @@
$(head -1 "$full_path")
$(echo "$fix_code" | head -1)
EOF

    # Note: This is a simplified patch. In a real implementation,
    # you would use proper diff tools and context lines
}

# Generate HTML report
generate_html_report() {
    log_info "Generating HTML fix generation report..."

    local html_report="${FIXES_DIR}/fix-generation-report.html"

    cat > "$html_report" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>Rust AI IDE - AI Fix Generation Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; background: #f5f5f5; }
        .header { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px; border-radius: 5px; margin-bottom: 20px; }
        .summary { background: #3498db; color: white; padding: 15px; border-radius: 5px; margin-bottom: 20px; }
        .fix { border: 1px solid #ddd; margin: 10px 0; padding: 15px; border-radius: 5px; background: white; }
        .fix.valid { border-left: 5px solid #27ae60; }
        .fix.invalid { border-left: 5px solid #e74c3c; }
        .code { background: #f8f9fa; padding: 10px; border-radius: 3px; font-family: monospace; margin: 10px 0; }
        .confidence { color: #f39c12; font-weight: bold; }
        .explanation { background: #ecf0f1; padding: 10px; margin: 10px 0; border-radius: 3px; }
        table { border-collapse: collapse; width: 100%; background: white; margin: 20px 0; }
        th, td { border: 1px solid #ddd; padding: 12px; text-align: left; }
        th { background-color: #2c3e50; color: white; }
        .metric { text-align: center; font-weight: bold; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Rust AI IDE - AI-Powered Fix Generation Report</h1>
        <p>Generated: $(date)</p>
    </div>

    <div class="summary">
        <h2>Generation Summary</h2>
        $(jq -r '"<p><strong>Total Issues Processed:</strong> \(.summary.total_issues_processed)</p><p><strong>Successful Fixes:</strong> \(.summary.successful_fixes_generated)</p><p><strong>Failed Fixes:</strong> \(.summary.failed_fixes)</p><p><strong>Success Rate:</strong> \(.summary.success_rate)%</p>"' "${FIXES_DIR}/fix-generation-summary.json")
    </div>

    <h2>Generated Fixes</h2>
    $(jq -r '.[] | "<div class=\"fix valid\"><h3>Issue: \(.id)</h3><p><strong>File:</strong> \(.original_issue.file)</p><p><strong>Category:</strong> \(.original_issue.category)</p><div class=\"confidence\">Confidence: \(.fix.confidence)%</div><div class=\"explanation\"><strong>Explanation:</strong><br>\(.fix.explanation)</div><div class=\"code\"><strong>Fix Code:</strong><br><pre>\(.fix.code)</pre></div></div>"' "${FIXES_DIR}/generated-fixes.json")

    <h2>Configuration</h2>
    $(jq -r '"<table><tr><th>Setting</th><th>Value</th></tr><tr><td>AI Service Endpoint</td><td>\(.summary.ai_service_endpoint)</td></tr><tr><td>LSP Service Endpoint</td><td>\(.summary.lsp_service_endpoint)</td></tr><tr><td>Max Fix Attempts</td><td>\(.configuration.max_fix_attempts)</td></tr><tr><td>Fix Timeout</td><td>\(.configuration.fix_timeout)s</td></tr><tr><td>Generate All Fixes</td><td>\(.configuration.generate_all_fixes)</td></tr><tr><td>Accept Low Confidence</td><td>\(.configuration.accept_low_confidence)</td></tr></table>"' "${FIXES_DIR}/fix-generation-summary.json")
</body>
</html>
EOF

    log_success "HTML report generated: $html_report"
}

# Main execution function
main() {
    log_info "Starting AI-powered fix generation subsystem"
    log_info "Project root: $PROJECT_ROOT"
    log_info "AI service: $AI_SERVICE_ENDPOINT"
    log_info "LSP service: $LSP_SERVICE_ENDPOINT"
    log_info "Fixes directory: $FIXES_DIR"

    setup
    generate_all_fixes
    validate_fixes
    generate_patches
    generate_html_report

    log_success "AI-powered fix generation complete"
    log_info "Reports and patches available in: $FIXES_DIR"

    # Summary
    local successful_fixes=$(jq '.summary.successful_fixes_generated // 0' "${FIXES_DIR}/fix-generation-summary.json" 2>/dev/null || echo "0")
    if [[ $successful_fixes -gt 0 ]]; then
        log_ai "Generated $successful_fixes AI-powered fixes ready for review and application"
    else
        log_warn "No fixes were generated - check AI service status and issue classification"
    fi
}

# Execute main function
main "$@"
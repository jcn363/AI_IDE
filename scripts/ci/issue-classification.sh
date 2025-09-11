#!/bin/bash

# Critical Bug Resolution - Automated Issue Classification and Prioritization
# This script analyzes detected issues and assigns priority scores based on severity,
# impact, and architectural considerations for the Rust AI IDE project

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
REPORT_DIR="${PROJECT_ROOT}/reports/bug-detection"
CLASSIFICATION_DIR="${PROJECT_ROOT}/reports/issue-classification"
LOG_FILE="${CLASSIFICATION_DIR}/classification.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Priority thresholds
CRITICAL_THRESHOLD=90
HIGH_THRESHOLD=70
MEDIUM_THRESHOLD=40
LOW_THRESHOLD=20

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

log_priority() {
    echo -e "${PURPLE}[PRIORITY]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $*" | tee -a "$LOG_FILE"
}

# Setup function
setup() {
    log_info "Setting up issue classification environment..."
    mkdir -p "$CLASSIFICATION_DIR"

    # Check if bug detection reports exist
    if [[ ! -f "${REPORT_DIR}/bug-detection-summary.json" ]]; then
        log_error "Bug detection reports not found. Please run bug-detection.sh first."
        exit 1
    fi

    log_success "Environment setup complete"
}

# Calculate severity score based on issue type and context
calculate_severity_score() {
    local issue_type="$1"
    local file_path="$2"
    local context="$3"
    local base_score=0

    case "$issue_type" in
        "security_vulnerability")
            base_score=95
            ;;
        "unsafe_usage")
            base_score=85
            ;;
        "panic_usage")
            base_score=80
            ;;
        "unwrap_usage"|"expect_usage")
            base_score=70
            ;;
        "clippy_correctness")
            base_score=75
            ;;
        "clippy_suspicious")
            base_score=60
            ;;
        "dead_code")
            base_score=30
            ;;
        "missing_docs")
            base_score=25
            ;;
        "print_debug")
            base_score=20
            ;;
        "unused_variables"|"unused_imports")
            base_score=15
            ;;
        "clippy_style"|"clippy_pedantic")
            base_score=10
            ;;
        *)
            base_score=20
            ;;
    esac

    # Adjust score based on file location (more critical if in core modules)
    if [[ "$file_path" =~ ^(src-tauri|src/|crates/.*core|crates/.*lsp) ]]; then
        base_score=$((base_score + 10))
    fi

    # Adjust for context keywords
    if [[ "$context" =~ (security|unsafe|panic|crash|memory|thread) ]]; then
        base_score=$((base_score + 15))
    fi

    # Cap at 100
    echo $((base_score > 100 ? 100 : base_score))
}

# Calculate impact score based on potential reach and consequences
calculate_impact_score() {
    local issue_type="$1"
    local file_path="$2"
    local context="$3"
    local base_score=0

    # Impact based on module type
    if [[ "$file_path" =~ (web/|src-tauri/src/main.rs) ]]; then
        base_score=80  # Frontend/UI impact
    elif [[ "$file_path" =~ (crates/.*lsp|crates/.*ai) ]]; then
        base_score=90  # Core AI/LSP functionality
    elif [[ "$file_path" =~ (crates/.*infra|crates/.*security) ]]; then
        base_score=85  # Infrastructure/Security impact
    elif [[ "$file_path" =~ (crates/.*common|crates/.*types) ]]; then
        base_score=70  # Shared components
    else
        base_score=50  # Other components
    fi

    # Adjust for issue type impact
    case "$issue_type" in
        "security_vulnerability"|"unsafe_usage")
            base_score=$((base_score + 20))
            ;;
        "panic_usage"|"clippy_correctness")
            base_score=$((base_score + 15))
            ;;
        "unwrap_usage"|"expect_usage")
            base_score=$((base_score + 10))
            ;;
    esac

    # Cap at 100
    echo $((base_score > 100 ? 100 : base_score))
}

# Classify issues from bug detection reports
classify_issues() {
    log_info "Starting automated issue classification..."

    local classified_issues="${CLASSIFICATION_DIR}/classified-issues.json"
    local priority_matrix="${CLASSIFICATION_DIR}/priority-matrix.json"

    # Initialize JSON output
    echo "[" > "$classified_issues"
    echo "{" > "$priority_matrix"

    local first_issue=true
    local total_issues=0
    local critical_count=0
    local high_count=0
    local medium_count=0
    local low_count=0

    # Process error patterns from bug detection
    if [[ -f "${REPORT_DIR}/error-patterns.json" ]]; then
        log_info "Processing error patterns..."
        jq -c '.[]' "${REPORT_DIR}/error-patterns.json" | while read -r issue; do
            local file=$(echo "$issue" | jq -r '.file')
            local pattern=$(echo "$issue" | jq -r '.pattern')
            local occurrences=$(echo "$issue" | jq -r '.occurrences')
            local severity=$(echo "$issue" | jq -r '.severity')

            local severity_score=$(calculate_severity_score "$pattern" "$file" "$pattern")
            local impact_score=$(calculate_impact_score "$pattern" "$file" "$pattern")
            local priority_score=$(((severity_score + impact_score) / 2))

            # Determine priority level
            local priority_level
            if [[ $priority_score -ge $CRITICAL_THRESHOLD ]]; then
                priority_level="critical"
                critical_count=$((critical_count + 1))
            elif [[ $priority_score -ge $HIGH_THRESHOLD ]]; then
                priority_level="high"
                high_count=$((high_count + 1))
            elif [[ $priority_score -ge $MEDIUM_THRESHOLD ]]; then
                priority_level="medium"
                medium_count=$((medium_count + 1))
            else
                priority_level="low"
                low_count=$((low_count + 1))
            fi

            # Add to classified issues
            if [[ "$first_issue" == "false" ]]; then
                echo "," >> "$classified_issues"
            fi
            first_issue=false

            cat >> "$classified_issues" << EOF
{
    "id": "pattern-${total_issues}",
    "type": "error_pattern",
    "category": "$pattern",
    "file": "$file",
    "occurrences": $occurrences,
    "severity_score": $severity_score,
    "impact_score": $impact_score,
    "priority_score": $priority_score,
    "priority_level": "$priority_level",
    "classification": {
        "technical_debt": $([[ "$pattern" =~ (dead_code|missing_docs|unused) ]] && echo "true" || echo "false"),
        "security_risk": $([[ "$pattern" =~ (unsafe|security) ]] && echo "true" || echo "false"),
        "performance_impact": $([[ "$pattern" =~ (unwrap|expect|panic) ]] && echo "true" || echo "false"),
        "maintainability_issue": $([[ "$pattern" =~ (style|pedantic|complexity) ]] && echo "true" || echo "false")
    },
    "estimated_effort": "$(get_effort_estimate "$pattern" "$occurrences")",
    "tags": ["$(get_issue_tags "$pattern" "$file")"]
}
EOF

            total_issues=$((total_issues + 1))
        done
    fi

    # Process Clippy warnings
    if [[ -f "${REPORT_DIR}/clippy-analysis.json" ]]; then
        log_info "Processing Clippy warnings..."
        jq -c '.[] | select(.level == "warning")' "${REPORT_DIR}/clippy-analysis.json" 2>/dev/null | while read -r warning; do
            local file=$(echo "$warning" | jq -r '.target.src_path // "unknown"' | sed 's|.*/src/|src/|')
            local message=$(echo "$warning" | jq -r '.message.rendered // .message.message // "Unknown warning"')
            local category=$(echo "$warning" | jq -r '.code // "unknown"')

            # Extract issue type from category
            local issue_type="clippy_$category"

            local severity_score=$(calculate_severity_score "$issue_type" "$file" "$message")
            local impact_score=$(calculate_impact_score "$issue_type" "$file" "$message")
            local priority_score=$(((severity_score + impact_score) / 2))

            # Determine priority level
            local priority_level
            if [[ $priority_score -ge $CRITICAL_THRESHOLD ]]; then
                priority_level="critical"
                critical_count=$((critical_count + 1))
            elif [[ $priority_score -ge $HIGH_THRESHOLD ]]; then
                priority_level="high"
                high_count=$((high_count + 1))
            elif [[ $priority_score -ge $MEDIUM_THRESHOLD ]]; then
                priority_level="medium"
                medium_count=$((medium_count + 1))
            else
                priority_level="low"
                low_count=$((low_count + 1))
            fi

            # Add to classified issues
            if [[ "$first_issue" == "false" ]]; then
                echo "," >> "$classified_issues"
            fi
            first_issue=false

            cat >> "$classified_issues" << EOF
{
    "id": "clippy-${total_issues}",
    "type": "clippy_warning",
    "category": "$category",
    "file": "$file",
    "message": "$message",
    "severity_score": $severity_score,
    "impact_score": $impact_score,
    "priority_score": $priority_score,
    "priority_level": "$priority_level",
    "classification": {
        "technical_debt": $([[ "$category" =~ (style|pedantic) ]] && echo "true" || echo "false"),
        "security_risk": $([[ "$message" =~ (unsafe|security|overflow) ]] && echo "true" || echo "false"),
        "performance_impact": $([[ "$message" =~ (perf|memory|alloc) ]] && echo "true" || echo "false"),
        "maintainability_issue": $([[ "$category" =~ (complexity|nursery) ]] && echo "true" || echo "false")
    },
    "estimated_effort": "small",
    "tags": ["clippy", "lint", "$category"]
}
EOF

            total_issues=$((total_issues + 1))
        done
    fi

    # Process security vulnerabilities
    if [[ -f "${REPORT_DIR}/security-audit.json" ]]; then
        log_info "Processing security vulnerabilities..."
        jq -c '.vulnerabilities.list[]?' "${REPORT_DIR}/security-audit.json" 2>/dev/null | while read -r vuln; do
            local package=$(echo "$vuln" | jq -r '.package.name // "unknown"')
            local severity=$(echo "$vuln" | jq -r '.advisory.severity // "unknown"')
            local title=$(echo "$vuln" | jq -r '.advisory.title // "Unknown vulnerability"')

            local severity_score=$(calculate_severity_score "security_vulnerability" "Cargo.toml" "$severity")
            local impact_score=$(calculate_impact_score "security_vulnerability" "Cargo.toml" "$title")
            local priority_score=$(((severity_score + impact_score) / 2))

            local priority_level="critical"
            critical_count=$((critical_count + 1))

            # Add to classified issues
            if [[ "$first_issue" == "false" ]]; then
                echo "," >> "$classified_issues"
            fi
            first_issue=false

            cat >> "$classified_issues" << EOF
{
    "id": "security-${total_issues}",
    "type": "security_vulnerability",
    "category": "security",
    "package": "$package",
    "severity": "$severity",
    "title": "$title",
    "severity_score": $severity_score,
    "impact_score": $impact_score,
    "priority_score": $priority_score,
    "priority_level": "$priority_level",
    "classification": {
        "technical_debt": false,
        "security_risk": true,
        "performance_impact": false,
        "maintainability_issue": false
    },
    "estimated_effort": "medium",
    "tags": ["security", "vulnerability", "$severity"]
}
EOF

            total_issues=$((total_issues + 1))
        done
    fi

    # Close JSON arrays
    echo "]" >> "$classified_issues"

    # Generate priority matrix
    cat >> "$priority_matrix" << EOF
    "total_issues": $total_issues,
    "critical_count": $critical_count,
    "high_count": $high_count,
    "medium_count": $medium_count,
    "low_count": $low_count,
    "distribution": {
        "critical_percentage": $((total_issues > 0 ? critical_count * 100 / total_issues : 0)),
        "high_percentage": $((total_issues > 0 ? high_count * 100 / total_issues : 0)),
        "medium_percentage": $((total_issues > 0 ? medium_count * 100 / total_issues : 0)),
        "low_percentage": $((total_issues > 0 ? low_count * 100 / total_issues : 0))
    },
    "thresholds": {
        "critical": $CRITICAL_THRESHOLD,
        "high": $HIGH_THRESHOLD,
        "medium": $MEDIUM_THRESHOLD,
        "low": $LOW_THRESHOLD
    }
}
EOF

    log_success "Issue classification complete"
    log_info "Classified $total_issues issues: $critical_count critical, $high_count high, $medium_count medium, $low_count low"
}

# Get effort estimate based on issue type and complexity
get_effort_estimate() {
    local pattern="$1"
    local occurrences="$2"

    case "$pattern" in
        "security_vulnerability")
            echo "large"
            ;;
        "unsafe_usage"|"panic_usage")
            echo "medium"
            ;;
        "unwrap_usage"|"expect_usage")
            if [[ $occurrences -gt 10 ]]; then
                echo "large"
            else
                echo "medium"
            fi
            ;;
        "dead_code"|"missing_docs")
            if [[ $occurrences -gt 50 ]]; then
                echo "large"
            else
                echo "small"
            fi
            ;;
        *)
            echo "small"
            ;;
    esac
}

# Get relevant tags for issue categorization
get_issue_tags() {
    local pattern="$1"
    local file="$2"
    local tags=""

    # Add architectural tags
    if [[ "$file" =~ (web/) ]]; then
        tags="${tags}frontend,"
    elif [[ "$file" =~ (src-tauri/) ]]; then
        tags="${tags}desktop,"
    elif [[ "$file" =~ (crates/.*lsp) ]]; then
        tags="${tags}lsp,"
    elif [[ "$file" =~ (crates/.*ai) ]]; then
        tags="${tags}ai,"
    fi

    # Add pattern-specific tags
    case "$pattern" in
        "security_vulnerability")
            tags="${tags}security,critical"
            ;;
        "unsafe_usage")
            tags="${tags}unsafe,memory"
            ;;
        "panic_usage")
            tags="${tags}error-handling,crash"
            ;;
        "unwrap_usage"|"expect_usage")
            tags="${tags}error-handling,runtime"
            ;;
        "dead_code")
            tags="${tags}maintenance,refactor"
            ;;
        "missing_docs")
            tags="${tags}documentation,maintainability"
            ;;
        *)
            tags="${tags}$pattern"
            ;;
    esac

    echo "${tags%,}"
}

# Generate prioritized action plan
generate_action_plan() {
    log_info "Generating prioritized action plan..."

    local action_plan="${CLASSIFICATION_DIR}/action-plan.json"
    local sorted_issues="${CLASSIFICATION_DIR}/issues-sorted-by-priority.json"

    # Sort issues by priority score (descending)
    jq 'sort_by(.priority_score) | reverse' "$CLASSIFICATION_DIR/classified-issues.json" > "$sorted_issues"

    # Create action plan with phases
    cat > "$action_plan" << EOF
{
    "generated_at": "$(date -Iseconds)",
    "action_plan": {
        "phase_1_critical": {
            "description": "Address critical security and safety issues immediately",
            "timeframe": "1-2 days",
            "issues": $(jq '[.[] | select(.priority_level == "critical")]' "$sorted_issues")
        },
        "phase_2_high": {
            "description": "Fix high-priority issues that affect core functionality",
            "timeframe": "1 week",
            "issues": $(jq '[.[] | select(.priority_level == "high")]' "$sorted_issues")
        },
        "phase_3_medium": {
            "description": "Address medium-priority issues for improved reliability",
            "timeframe": "2-3 weeks",
            "issues": $(jq '[.[] | select(.priority_level == "medium")]' "$sorted_issues")
        },
        "phase_4_low": {
            "description": "Clean up low-priority issues and technical debt",
            "timeframe": "1 month",
            "issues": $(jq '[.[] | select(.priority_level == "low")]' "$sorted_issues")
        }
    },
    "summary": {
        "total_issues": $(jq length "$sorted_issues"),
        "critical_count": $(jq '[.[] | select(.priority_level == "critical")] | length' "$sorted_issues"),
        "estimated_total_effort": "$(calculate_total_effort "$sorted_issues")"
    }
}
EOF

    log_success "Action plan generated"
    log_priority "Critical issues requiring immediate attention: $(jq '[.[] | select(.priority_level == "critical")] | length' "$sorted_issues")"
}

# Calculate total effort estimate
calculate_total_effort() {
    local issues_file="$1"
    local small=0
    local medium=0
    local large=0

    while read -r effort; do
        case "$effort" in
            "small") small=$((small + 1)) ;;
            "medium") medium=$((medium + 1)) ;;
            "large") large=$((large + 1)) ;;
        esac
    done < <(jq -r '.[] | .estimated_effort' "$issues_file")

    echo "${small}s + ${medium}m + ${large}l"
}

# Generate HTML report
generate_html_report() {
    log_info "Generating HTML classification report..."

    local html_report="${CLASSIFICATION_DIR}/classification-report.html"

    cat > "$html_report" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>Rust AI IDE - Issue Classification Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; background: #f5f5f5; }
        .header { background: #2c3e50; color: white; padding: 20px; border-radius: 5px; margin-bottom: 20px; }
        .summary { background: #3498db; color: white; padding: 15px; border-radius: 5px; margin-bottom: 20px; }
        .critical { background: #e74c3c; color: white; }
        .high { background: #f39c12; color: black; }
        .medium { background: #f1c40f; color: black; }
        .low { background: #27ae60; color: white; }
        .issue { border: 1px solid #ddd; margin: 10px 0; padding: 15px; border-radius: 5px; background: white; }
        .priority-badge { padding: 3px 8px; border-radius: 3px; font-size: 12px; font-weight: bold; }
        table { border-collapse: collapse; width: 100%; background: white; margin: 20px 0; }
        th, td { border: 1px solid #ddd; padding: 12px; text-align: left; }
        th { background-color: #2c3e50; color: white; }
        .phase { background: #ecf0f1; padding: 15px; margin: 10px 0; border-radius: 5px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Rust AI IDE - Critical Bug Resolution: Issue Classification</h1>
        <p>Generated: $(date)</p>
    </div>

    <div class="summary">
        <h2>Priority Distribution</h2>
        <p>Total Issues: $(jq '.total_issues' "${CLASSIFICATION_DIR}/priority-matrix.json")</p>
        <p>Critical: $(jq '.critical_count' "${CLASSIFICATION_DIR}/priority-matrix.json") |
           High: $(jq '.high_count' "${CLASSIFICATION_DIR}/priority-matrix.json") |
           Medium: $(jq '.medium_count' "${CLASSIFICATION_DIR}/priority-matrix.json") |
           Low: $(jq '.low_count' "${CLASSIFICATION_DIR}/priority-matrix.json")</p>
    </div>

    <h2>Action Plan</h2>
    <div class="phase critical">
        <h3>Phase 1: Critical Issues (Immediate - 1-2 days)</h3>
        <p>Security vulnerabilities and safety-critical issues</p>
        $(jq -r '.action_plan.phase_1_critical.issues[]? | "<div class=\"issue\"><strong>\(.category)</strong> in \(.file)<br><small>\(.message // .title // "No details")</small></div>"' "${CLASSIFICATION_DIR}/action-plan.json")
    </div>

    <div class="phase high">
        <h3>Phase 2: High Priority (1 week)</h3>
        <p>Core functionality issues</p>
        $(jq -r '.action_plan.phase_2_high.issues[]? | "<div class=\"issue\"><strong>\(.category)</strong> in \(.file)<br><small>\(.message // .title // "No details")</small></div>"' "${CLASSIFICATION_DIR}/action-plan.json")
    </div>

    <div class="phase medium">
        <h3>Phase 3: Medium Priority (2-3 weeks)</h3>
        <p>Reliability improvements</p>
        $(jq -r '.action_plan.phase_3_medium.issues[]? | "<div class=\"issue\"><strong>\(.category)</strong> in \(.file)<br><small>\(.message // .title // "No details")</small></div>"' "${CLASSIFICATION_DIR}/action-plan.json")
    </div>

    <div class="phase low">
        <h3>Phase 4: Low Priority (1 month)</h3>
        <p>Technical debt cleanup</p>
        $(jq -r '.action_plan.phase_4_low.issues[]? | "<div class=\"issue\"><strong>\(.category)</strong> in \(.file)<br><small>\(.message // .title // "No details")</small></div>"' "${CLASSIFICATION_DIR}/action-plan.json")
    </div>
</body>
</html>
EOF

    log_success "HTML report generated: $html_report"
}

# Main execution function
main() {
    log_info "Starting automated issue classification and prioritization subsystem"
    log_info "Project root: $PROJECT_ROOT"
    log_info "Classification directory: $CLASSIFICATION_DIR"

    setup
    classify_issues
    generate_action_plan
    generate_html_report

    log_success "Issue classification and prioritization complete"
    log_info "Reports available in: $CLASSIFICATION_DIR"

    # Check for critical issues
    local critical_count=$(jq '.critical_count // 0' "${CLASSIFICATION_DIR}/priority-matrix.json" 2>/dev/null || echo "0")
    if [[ $critical_count -gt 0 ]]; then
        log_priority "ALERT: $critical_count critical issues detected - immediate action required"
        exit 1
    fi
}

# Execute main function
main "$@"
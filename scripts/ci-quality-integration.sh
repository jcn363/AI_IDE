#!/bin/bash

# CI/CD Quality Integration Script
#
# This script integrates quality gates into CI/CD pipelines
# Supports GitHub Actions, Jenkins, Azure DevOps, and other CI systems

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# CI system detection
detect_ci_system() {
    if [ -n "$GITHUB_ACTIONS" ]; then
        echo "github_actions"
    elif [ -n "$JENKINS_HOME" ]; then
        echo "jenkins"
    elif [ -n "$TF_BUILD" ]; then
        echo "azure_devops"
    elif [ -n "$GITLAB_CI" ]; then
        echo "gitlab_ci"
    elif [ -n "$CIRCLECI" ]; then
        echo "circle_ci"
    else
        echo "generic"
    fi
}

# Detect environment
CI_SYSTEM=$(detect_ci_system)
echo "CI_SYSTEM: $CI_SYSTEM"

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_header() {
    echo "=================================================="
    echo " $1"
    echo "=================================================="
}

# Setup CI environment
setup_ci_environment() {
    log_info "Setting up CI environment for $CI_SYSTEM..."

    # Get current commit information
    CURRENT_COMMIT="${GITHUB_SHA:-${GIT_COMMIT:-${BUILD_VCS_NUMBER:-$(git rev-parse HEAD 2>/dev/null)}}}:-unknown"
    CURRENT_BRANCH="${GITHUB_HEAD_REF:-${GITHUB_BASE_REF:-${GIT_BRANCH:-${BRANCH_NAME:-$(git rev-parse --abbrev-ref HEAD 2>/dev/null)}}}:-unknown}"
    BUILD_NUMBER="${GITHUB_RUN_NUMBER:-${BUILD_NUMBER:-${CI_BUILD_NUMBER:-1}}}"

    export CURRENT_COMMIT
    export CURRENT_BRANCH
    export BUILD_NUMBER

    log_info "Commit: $CURRENT_COMMIT"
    log_info "Branch: $CURRENT_BRANCH"
    log_info "Build: $BUILD_NUMBER"

    # Set up test output directories
    export TEST_RESULTS_DIR="test-results-${CI_SYSTEM}-${BUILD_NUMBER}"
    mkdir -p "$TEST_RESULTS_DIR"

    # Export CI-specific environment variables
    case $CI_SYSTEM in
        "github_actions")
            echo "test_results_dir=$TEST_RESULTS_DIR" >> $GITHUB_OUTPUT
            echo "quality_gates_output=$TEST_RESULTS_DIR/gate-results/consolidated_report.json" >> $GITHUB_OUTPUT
            ;;
        "jenkins")
            echo "TEST_RESULTS_DIR=$TEST_RESULTS_DIR"
            echo "QUALITY_GATES_OUTPUT=$TEST_RESULTS_DIR/gate-results/consolidated_report.json"
            ;;
        "azure_devops")
            echo "##vso[task.setvariable variable=testResultsDir]$TEST_RESULTS_DIR"
            echo "##vso[task.setvariable variable=qualityGatesOutput]$TEST_RESULTS_DIR/gate-results/consolidated_report.json"
            ;;
    esac

    log_success "CI environment setup completed"
}

# Pre-build checks
run_pre_build_checks() {
    log_header "Running Pre-Build Checks"

    # Check Rust version
    if command -v rustc >/dev/null 2>&1; then
        RUST_VERSION=$(rustc --version)
        log_info "Rust version: $RUST_VERSION"
    else
        log_error "Rust not found"
        exit 1
    fi

    # Check cargo
    if command -v cargo >/dev/null 2>&1; then
        CARGO_VERSION=$(cargo --version)
        log_info "Cargo version: $CARGO_VERSION"
    else
        log_error "Cargo not found"
        exit 1
    fi

    # Check for required tools
    local required_tools=("gcc" "git")
    for tool in "${required_tools[@]}"; do
        if command -v "$tool" >/dev/null 2>&1; then
            log_info "$tool found: $($tool --version 2>&1 | head -1)"
        else
            log_warning "$tool not found (may be required)"
        fi
    done

    # Check for optional tools used in quality gates
    local optional_tools=("cargo-tarpaulin" "cargo-audit" "node" "docker")
    for tool in "${optional_tools[@]}"; do
        if command -v "$tool" >/dev/null 2>&1; then
            local version=$($tool --version 2>&1 | head -1)
            log_info "$tool available: $version"
        else
            log_info "$tool not available (quality gates may be limited)"
        fi
    done

    # Verify workspace consistency
    if [ -f "scripts/check-workspace-consistency.sh" ]; then
        log_info "Running workspace consistency check..."
        if bash scripts/check-workspace-consistency.sh; then
            log_success "Workspace consistency check passed"
        else
            log_warning "Workspace consistency issues found"
        fi
    fi

    log_success "Pre-build checks completed"
}

# Run quality gates with CI-optimized settings
run_quality_gates() {
    log_header "Running Quality Gates"

    local quality_gates_script="$SCRIPT_DIR/run-quality-gates.sh"
    local quality_gates_args=()

    # Configure for CI environment
    quality_gates_args+=(--ci)
    quality_gates_args+=(--output-dir "$TEST_RESULTS_DIR")

    # Add gates to run (can be configured via environment)
    local gates_to_run="${QUALITY_GATES:-unit,integration,performance,coverage,security}"
    if [ -n "$gates_to_run" ]; then
        quality_gates_args+=(--gates "$gates_to_run")
    fi

    # Enable fail-fast for CI
    quality_gates_args+=(--fail-fast)

    # Enable verbose output for CI logs
    quality_gates_args+=(--verbose)

    # Handle UI/E2E test skipping in CI environments where they're not configured
    if [ "$ENABLE_UI_TESTS" != "true" ]; then
        quality_gates_args+=(--skip-ui-tests)
    fi

    if [ "$ENABLE_E2E_TESTS" != "true" ]; then
        quality_gates_args+=(--skip-e2e-tests)
    fi

    log_info "Executing quality gates: ${quality_gates_args[@]}"

    if bash "$quality_gates_script" "${quality_gates_args[@]}"; then
        QUALITY_GATES_PASSED=true
        log_success "Quality gates passed"
    else
        QUALITY_GATES_PASSED=false
        log_error "Quality gates failed"
    fi

    # Export result for CI system
    case $CI_SYSTEM in
        "github_actions")
            echo "quality_gates_passed=$QUALITY_GATES_PASSED" >> $GITHUB_OUTPUT
            ;;
        "jenkins")
            echo "QUALITY_GATES_PASSED=$QUALITY_GATES_PASSED"
            ;;
        "azure_devops")
            echo "##vso[task.setvariable variable=qualityGatesPassed]$QUALITY_GATES_PASSED"
            ;;
    esac

    return $($QUALITY_GATES_PASSED && echo 0 || echo 1)
}

# Upload artifacts and reports
upload_artifacts() {
    log_header "Uploading Artifacts and Reports"

    local artifacts_dir="$TEST_RESULTS_DIR"

    if [ ! -d "$artifacts_dir" ]; then
        log_warning "No artifacts directory found: $artifacts_dir"
        return 0
    fi

    case $CI_SYSTEM in
        "github_actions")
            # Upload artifacts to GitHub Actions
            if command -v actions/upload-artifact@v3 >/dev/null 2>&1; then
                log_info "Uploading artifacts to GitHub Actions..."
                echo "artifact_name=test-results-$BUILD_NUMBER"
                echo "artifact_path=$artifacts_dir"
                echo "::set-output name=artifact_name::test-results-$BUILD_NUMBER"
                echo "::set-output name=artifact_path::$artifacts_dir"
                # In practice, you'd use the actions/upload-artifact action here
                log_success "Artifacts configured for upload"
            fi
            ;;
        "jenkins")
            # Move artifacts to Jenkins artifacts directory
            if [ -n "$WORKSPACE" ]; then
                cp -r "$artifacts_dir" "$WORKSPACE/artifacts-$BUILD_NUMBER/"
                log_success "Artifacts copied to Jenkins workspace"
            fi
            ;;
        "azure_devops")
            # Azure DevOps artifact upload
            log_info "Artifacts ready for Azure DevOps upload"
            ;;
    esac

    log_success "Artifact upload configuration completed"
}

# Post-build processing
post_build_processing() {
    log_header "Post-Build Processing"

    # Generate summary report for CI dashboard
    local summary_file="$TEST_RESULTS_DIR/gate-results/consolidated_report.json"
    local readable_report="$TEST_RESULTS_DIR/QUALITY_SUMMARY.md"

    if [ -f "$summary_file" ]; then
        log_info "Generating CI summary report..."

        cat > "$readable_report" << EOF
# Quality Gates Summary - Build $BUILD_NUMBER

## Build Information
- **Commit**: \`$CURRENT_COMMIT\`
- **Branch**: \`$CURRENT_BRANCH\`
- **Timestamp**: \`$(date -u +%Y-%m-%dT%H:%M:%SZ)\`
- **CI System**: \`$CI_SYSTEM\`
EOF

        # Extract and add quality gate results
        if command -v jq >/dev/null 2>&1; then
            local total_gates=$(jq -r '.summary.total_gates' "$summary_file" 2>/dev/null || echo "N/A")
            local passed_gates=$(jq -r '.summary.passed_gates' "$summary_file" 2>/dev/null || echo "N/A")
            local failed_gates=$(jq -r '.summary.failed_gates' "$summary_file" 2>/dev/null || echo "N/A")
            local success_rate=$(jq -r '.summary.success_rate' "$summary_file" 2>/dev/null || echo "0.0")

            cat >> "$readable_report" << EOF

## Quality Gate Results
- **Total Gates**: $total_gates
- **Passed**: $passed_gates
- **Failed**: $failed_gates
- **Success Rate**: ${success_rate}%

### Detailed Gate Status
EOF

            # Add individual gate details
            jq -r '.gates[] | "- **\(.gate)**: \(.status)"' "$summary_file" 2>/dev/null | head -10 >> "$readable_report" || echo "- No detailed gate information available" >> "$readable_report"
        else
            echo -e "\n## Detailed Results\nSee: $summary_file" >> "$readable_report"
        fi

        log_success "Summary report generated: $readable_report"

        # Display summary in CI logs
        if [ -f "$readable_report" ]; then
            log_header "Quality Gates Summary"
            cat "$readable_report"
        fi
    else
        log_warning "No quality gate results found to summarize"
    fi
}

# Send notifications
send_notifications() {
    if [ "$QUALITY_GATES_PASSED" = "true" ]; then
        log_success "Quality gates passed - sending success notifications"
        # Send success notifications (email, Slack, etc.)
    else
        log_error "Quality gates failed - sending failure notifications"
        # Send failure notifications with details

        # In practice, you would integrate with notification services:
        # - Slack webhooks
        # - Email notifications
        # - Teams/Slack webhooks
        # - Jira issue creation

        local failed_gates_file="$TEST_RESULTS_DIR/gate-results/failed_gates.txt"
        if [ -n "$GITHUB_FAILURE_NOTIFICATION_WEBHOOK" ]; then
            log_info "Sending failure notification to configured webhook"
            # curl -X POST $GITHUB_FAILURE_NOTIFICATION_WEBHOOK \
            #     -H 'Content-Type: application/json' \
            #     -d "{\"text\":\"Quality gates failed for build $BUILD_NUMBER on $CURRENT_BRANCH\", \"results\":\"$summary_file\"}"
        fi
    fi
}

# Environment setup for different CI systems
setup_environment_specific() {
    case $CI_SYSTEM in
        "github_actions")
            log_info "Configuring for GitHub Actions"

            # Set up paths for GitHub Actions
            echo "$HOME/.cargo/bin" >> $GITHUB_PATH

            # Export results for subsequent steps
            echo "ci_system=github_actions" >> $GITHUB_OUTPUT
            ;;
        "jenkins")
            log_info "Configuring for Jenkins"

            # Jenkins-specific setup
            export JAVA_OPTS="${JAVA_OPTS:--Xmx1g}"

            # Make sure we have access to workspace
            if [ -n "$WORKSPACE" ]; then
                cd "$WORKSPACE"
            fi
            ;;
        "azure_devops")
            log_info "Configuring for Azure DevOps"

            # Azure DevOps specific setup
            # Set agent temp directory
            export AGENT_TEMPDIRECTORY="${AGENT_TEMPDIRECTORY:-/tmp}"
            ;;
        "gitlab_ci")
            log_info "Configuring for GitLab CI"

            # GitLab CI specific setup
            # Use artifacts for test results
            if [ -n "$CI_PROJECT_DIR" ]; then
                export TEST_RESULTS_DIR="${CI_PROJECT_DIR}/test-results"
                mkdir -p "$TEST_RESULTS_DIR"
            fi
            ;;
        "circle_ci")
            log_info "Configuring for CircleCI"

            # CircleCI specific setup
            # Store test results in proper location
            if [ -n "$CIRCLE_WORKING_DIRECTORY" ]; then
                export TEST_RESULTS_DIR="${CIRCLE_TEST_REPORTS}/quality-gates"
                mkdir -p "$TEST_RESULTS_DIR"
            fi
            ;;
        "generic")
            log_info "Running in generic CI environment"

            # Generic setup
            export TEST_RESULTS_DIR="${TEST_RESULTS_DIR:-test-results}"
            ;;
    esac
}

# Main execution
main() {
    log_header "CI/CD Quality Integration"

    # Setup environment specific configuration
    setup_environment_specific

    # Setup CI environment
    setup_ci_environment

    # Pre-build checks
    run_pre_build_checks

    # Run quality gates
    if run_quality_gates; then
        log_success "Quality gates executed successfully"
    else
        log_error "Quality gates execution failed"
    fi

    # Upload artifacts
    upload_artifacts

    # Post-build processing
    post_build_processing

    # Send notifications (if configured)
    if [ "${ENABLE_NOTIFICATIONS:-false}" = "true" ]; then
        send_notifications
    fi

    log_header "CI/CD Quality Integration Complete"

    # Return final status
    if [ "$QUALITY_GATES_PASSED" = "true" ]; then
        log_success "✅ All quality checks passed"
        exit 0
    else
        log_error "❌ Quality checks failed"
        exit 1
    fi
}

# Ensure we're in the project root
cd "$PROJECT_ROOT" 2>/dev/null || {
    log_error "Could not change to project root directory: $PROJECT_ROOT"
    exit 1
}

# Create output directories with proper permissions
mkdir -p "test-results" 2>/dev/null || true
chmod 755 "scripts" 2>/dev/null || true

# Run main function with all arguments
main "$@"
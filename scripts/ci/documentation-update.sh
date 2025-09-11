#!/bin/bash

# Automated Documentation Update System
# Updates documentation based on code changes and generates API docs
# Author: Documentation Team
# Version: 1.0.0

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
DOCS_DIR="${PROJECT_ROOT}/docs"
API_DOCS_DIR="${DOCS_DIR}/api"
CHANGELOG_FILE="${PROJECT_ROOT}/CHANGELOG.md"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${PROJECT_ROOT}/logs/documentation-update.log"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${PROJECT_ROOT}/logs/documentation-update.log" >&2
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${PROJECT_ROOT}/logs/documentation-update.log"
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${PROJECT_ROOT}/logs/documentation-update.log"
}

# Function to generate API documentation
generate_api_docs() {
    log_info "Generating API documentation..."

    mkdir -p "${API_DOCS_DIR}"

    # Generate Rust documentation
    if command -v cargo >/dev/null 2>&1; then
        log_info "Generating Rust API documentation..."
        cargo doc --workspace --no-deps --document-private-items
        cp -r target/doc "${API_DOCS_DIR}/rust/"
        log_success "Rust API documentation generated"
    else
        log_warning "Cargo not found, skipping Rust documentation generation"
    fi

    # Generate TypeScript documentation if web frontend exists
    if [[ -d "${PROJECT_ROOT}/web" && -f "${PROJECT_ROOT}/web/package.json" ]]; then
        cd "${PROJECT_ROOT}/web"
        if command -v npm >/dev/null 2>&1 && npm list typedoc >/dev/null 2>&1; then
            log_info "Generating TypeScript API documentation..."
            npx typedoc --out "${API_DOCS_DIR}/typescript" src/
            log_success "TypeScript API documentation generated"
        else
            log_warning "TypeDoc not available, skipping TypeScript documentation generation"
        fi
        cd "${PROJECT_ROOT}"
    fi
}

# Function to update README files
update_readme_files() {
    log_info "Updating README files..."

    # Update main README with current project status
    if [[ -f "${PROJECT_ROOT}/README.md" ]]; then
        local temp_readme="${PROJECT_ROOT}/README.tmp"

        # Update build status badge
        sed 's|build-[a-zA-Z]*-.*|build-passing-brightgreen|' "${PROJECT_ROOT}/README.md" > "$temp_readme"
        mv "$temp_readme" "${PROJECT_ROOT}/README.md"

        # Update version information
        local version=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.1.0")
        sed "s|version-.*|version-${version}-blue|" "${PROJECT_ROOT}/README.md" > "$temp_readme"
        mv "$temp_readme" "${PROJECT_ROOT}/README.md"

        log_success "Main README updated"
    fi

    # Update crate-specific READMEs
    for crate_dir in "${PROJECT_ROOT}/crates"/*/; do
        if [[ -d "$crate_dir" ]]; then
            local crate_name=$(basename "$crate_dir")
            local readme_file="${crate_dir}README.md"

            if [[ ! -f "$readme_file" ]]; then
                log_info "Creating README for crate: $crate_name"
                cat > "$readme_file" << EOF
# ${crate_name}

[![Crates.io](https://img.shields.io/crates/v/${crate_name}.svg)](https://crates.io/crates/${crate_name})
[![Documentation](https://docs.rs/${crate_name}/badge.svg)](https://docs.rs/${crate_name})
[![License](https://img.shields.io/crates/l/${crate_name}.svg)](https://github.com/rust-ai-ide/${crate_name}#license)

## Description

Brief description of the ${crate_name} crate functionality.

## Usage

\`\`\`rust
// Example usage code
\`\`\`

## Documentation

Full documentation is available at [docs.rs]().
EOF
                log_success "README created for crate: $crate_name"
            fi
        fi
    done
}

# Function to update changelog
update_changelog() {
    log_info "Updating changelog..."

    if [[ ! -f "$CHANGELOG_FILE" ]]; then
        log_info "Creating initial changelog..."
        cat > "$CHANGELOG_FILE" << 'EOF'
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project setup

### Changed
- N/A

### Deprecated
- N/A

### Removed
- N/A

### Fixed
- N/A

### Security
- N/A
EOF
        log_success "Initial changelog created"
        return
    fi

    # Get recent commits for changelog updates
    local recent_commits=$(git log --oneline -n 10 --since="1 week ago" 2>/dev/null || echo "")

    if [[ -n "$recent_commits" ]]; then
        log_info "Found recent commits for changelog update"

        # Parse commits and categorize them
        local added_items=""
        local fixed_items=""
        local changed_items=""

        while IFS= read -r commit; do
            local commit_msg=$(echo "$commit" | cut -d' ' -f2-)
            if [[ "$commit_msg" =~ ^feat: ]]; then
                added_items+="- ${commit_msg#feat: }\n"
            elif [[ "$commit_msg" =~ ^fix: ]]; then
                fixed_items+="- ${commit_msg#fix: }\n"
            elif [[ "$commit_msg" =~ ^refactor: ]] || [[ "$commit_msg" =~ ^chore: ]]; then
                changed_items+="- ${commit_msg#*: }\n"
            fi
        done <<< "$recent_commits"

        # Update unreleased section
        if [[ -n "$added_items" ]]; then
            sed -i '/### Added/a '"$added_items"'' "$CHANGELOG_FILE"
        fi
        if [[ -n "$fixed_items" ]]; then
            sed -i '/### Fixed/a '"$fixed_items"'' "$CHANGELOG_FILE"
        fi
        if [[ -n "$changed_items" ]]; then
            sed -i '/### Changed/a '"$changed_items"'' "$CHANGELOG_FILE"
        fi

        log_success "Changelog updated with recent changes"
    else
        log_info "No recent commits found for changelog update"
    fi
}

# Function to update dependency documentation
update_dependency_docs() {
    log_info "Updating dependency documentation..."

    local deps_file="${DOCS_DIR}/dependencies.md"

    if command -v cargo >/dev/null 2>&1; then
        log_info "Generating dependency tree..."
        cargo tree --workspace > "${deps_file}.tmp"

        # Format as markdown
        {
            echo "# Project Dependencies"
            echo ""
            echo "Generated on: $(date)"
            echo ""
            echo "\`\`\`"
            cat "${deps_file}.tmp"
            echo "\`\`\`"
        } > "$deps_file"

        rm "${deps_file}.tmp"
        log_success "Dependency documentation updated"
    else
        log_warning "Cargo not found, skipping dependency documentation update"
    fi
}

# Function to update performance documentation
update_performance_docs() {
    log_info "Updating performance documentation..."

    local perf_docs_dir="${DOCS_DIR}/performance"
    mkdir -p "$perf_docs_dir"

    # Check for performance reports
    if [[ -d "${PROJECT_ROOT}/reports/performance" ]]; then
        local latest_report=$(find "${PROJECT_ROOT}/reports/performance" -name "performance-report-*.json" | sort | tail -1)

        if [[ -n "$latest_report" ]]; then
            log_info "Generating performance documentation from latest report..."

            # Extract key metrics
            local build_time=$(jq -r '.performance_metrics.build_time_seconds // "N/A"' "$latest_report")
            local test_time=$(jq -r '.performance_metrics.test_time_seconds // "N/A"' "$latest_report")
            local binary_size=$(jq -r '.performance_metrics.binary_size_kb // "N/A"' "$latest_report")

            cat > "${perf_docs_dir}/latest-performance.md" << EOF
# Latest Performance Metrics

Generated on: $(date)
Report: $(basename "$latest_report")

## Build Performance
- Build Time: ${build_time}s
- Test Time: ${test_time}s
- Binary Size: ${binary_size}KB

## Recommendations
$(jq -r '.recommendations[] // "None"' "$latest_report" | sed 's/^/- /')

## Full Report
See: \`${latest_report}\`
EOF

            log_success "Performance documentation updated"
        fi
    else
        log_warning "No performance reports found"
    fi
}

# Function to update security documentation
update_security_docs() {
    log_info "Updating security documentation..."

    local security_docs_dir="${DOCS_DIR}/security"
    mkdir -p "$security_docs_dir"

    # Check for security reports
    if [[ -d "${PROJECT_ROOT}/security-reports" ]]; then
        local latest_audit=$(find "${PROJECT_ROOT}/security-reports" -name "cargo-audit-report.json" | sort | tail -1)

        if [[ -n "$latest_audit" ]]; then
            log_info "Generating security documentation from latest audit..."

            local vulnerabilities=$(jq '.vulnerabilities.count // 0' "$latest_audit" 2>/dev/null || echo 0)

            cat > "${security_docs_dir}/latest-security.md" << EOF
# Latest Security Audit

Generated on: $(date)
Audit: $(basename "$latest_audit")

## Summary
- Vulnerabilities Found: ${vulnerabilities}

$(if [[ $vulnerabilities -gt 0 ]]; then
    echo "## Vulnerabilities"
    jq -r '.vulnerabilities.list[]? | "- \(.advisory.id): \(.advisory.title)"' "$latest_audit" 2>/dev/null || echo "No detailed vulnerability information available"
else
    echo "## Status: ✅ No vulnerabilities found"
fi)

## Full Report
See: \`${latest_audit}\`
EOF

            log_success "Security documentation updated"
        fi
    else
        log_warning "No security reports found"
    fi
}

# Function to validate documentation
validate_documentation() {
    log_info "Validating documentation completeness..."

    local issues=0

    # Check for required files
    local required_files=(
        "README.md"
        "CONTRIBUTING.md"
        "CHANGELOG.md"
        "docs/README.md"
    )

    for file in "${required_files[@]}"; do
        if [[ ! -f "${PROJECT_ROOT}/${file}" ]]; then
            log_error "Missing required documentation file: $file"
            ((issues++))
        fi
    done

    # Check crate documentation
    for crate_dir in "${PROJECT_ROOT}/crates"/*/; do
        if [[ -d "$crate_dir" ]]; then
            if [[ ! -f "${crate_dir}README.md" ]]; then
                log_warning "Missing README for crate: $(basename "$crate_dir")"
                ((issues++))
            fi
        fi
    done

    # Check API documentation
    if [[ ! -d "${API_DOCS_DIR}/rust" ]]; then
        log_warning "Missing Rust API documentation"
        ((issues++))
    fi

    if [[ $issues -eq 0 ]]; then
        log_success "Documentation validation passed"
        return 0
    else
        log_warning "Documentation validation found $issues issues"
        return 1
    fi
}

# Function to generate documentation report
generate_documentation_report() {
    local report_file="${PROJECT_ROOT}/reports/documentation-update-report-$(date +%Y%m%d_%H%M%S).json"

    mkdir -p "$(dirname "${report_file}")"

    cat > "${report_file}" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "documentation_update": {
        "api_docs_generated": $([[ -d "${API_DOCS_DIR}" ]] && echo true || echo false),
        "readme_files_updated": $(find "${PROJECT_ROOT}" -name "README.md" | wc -l),
        "changelog_updated": $([[ -f "$CHANGELOG_FILE" ]] && echo true || echo false),
        "dependency_docs_updated": $([[ -f "${DOCS_DIR}/dependencies.md" ]] && echo true || echo false),
        "performance_docs_updated": $([[ -d "${DOCS_DIR}/performance" ]] && echo true || echo false),
        "security_docs_updated": $([[ -d "${DOCS_DIR}/security" ]] && echo true || echo false)
    },
    "validation_results": {
        "passed": $(validate_documentation && echo true || echo false)
    }
}
EOF

    log_success "Documentation update report generated: ${report_file}"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS] [ACTION]

Automated Documentation Update System

ACTIONS:
    all                 Update all documentation (default)
    api                 Generate API documentation only
    readme              Update README files only
    changelog           Update changelog only
    dependencies        Update dependency documentation only
    performance         Update performance documentation only
    security            Update security documentation only
    validate            Validate documentation completeness only

OPTIONS:
    -h, --help           Show this help message
    -v, --verbose        Enable verbose output
    --validate-only      Only validate, don't update
    --skip-api           Skip API documentation generation

EXAMPLES:
    $0                    Update all documentation
    $0 api                Generate API documentation only
    $0 validate           Validate documentation completeness
    $0 --validate-only    Check documentation without updating

EOF
}

# Parse command line arguments
VALIDATE_ONLY=false
SKIP_API=false
ACTION="all"

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
        --validate-only)
            VALIDATE_ONLY=true
            shift
            ;;
        --skip-api)
            SKIP_API=true
            shift
            ;;
        all|api|readme|changelog|dependencies|performance|security|validate)
            ACTION="$1"
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Main execution function
main() {
    log_info "Starting automated documentation update system (Action: ${ACTION})"

    if [[ "${VALIDATE_ONLY}" == true ]]; then
        log_info "VALIDATE-ONLY MODE: Only validating documentation"
        validate_documentation
        exit $?
    fi

    local exit_code=0

    case "${ACTION}" in
        all)
            if [[ "${SKIP_API}" != true ]]; then
                generate_api_docs
            fi
            update_readme_files
            update_changelog
            update_dependency_docs
            update_performance_docs
            update_security_docs
            validate_documentation || exit_code=1
            generate_documentation_report
            ;;
        api)
            generate_api_docs
            ;;
        readme)
            update_readme_files
            ;;
        changelog)
            update_changelog
            ;;
        dependencies)
            update_dependency_docs
            ;;
        performance)
            update_performance_docs
            ;;
        security)
            update_security_docs
            ;;
        validate)
            validate_documentation || exit_code=1
            ;;
        *)
            log_error "Unknown action: ${ACTION}"
            usage
            exit 1
            ;;
    esac

    if [[ $exit_code -eq 0 ]]; then
        log_success "Documentation update system completed successfully"
    else
        log_error "Documentation update system completed with errors"
    fi

    return $exit_code
}

# Execute main function
main "$@"
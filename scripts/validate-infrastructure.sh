#!/bin/bash

# Infrastructure Validation Script for Rust AI IDE Enterprise
# Validates all components, configurations, and security settings

set -euo pipefail

# Configuration
VALIDATION_LOG="/tmp/infrastructure-validation-$(date +%s).log"
PASS_COUNT=0
FAIL_COUNT=0
WARN_COUNT=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log() {
    echo "$*" | tee -a "$VALIDATION_LOG"
}

pass() {
    echo -e "${GREEN}✓ PASS${NC}: $*" | tee -a "$VALIDATION_LOG"
    ((PASS_COUNT++))
}

fail() {
    echo -e "${RED}✗ FAIL${NC}: $*" | tee -a "$VALIDATION_LOG"
    ((FAIL_COUNT++))
}

warn() {
    echo -e "${YELLOW}⚠ WARN${NC}: $*" | tee -a "$VALIDATION_LOG"
    ((WARN_COUNT++))
}

# Validation functions
validate_docker_setup() {
    log "=== Validating Docker Setup ==="

    # Check Docker installation
    if ! command -v docker &> /dev/null; then
        fail "Docker CLI not installed"
        return
    fi
    pass "Docker CLI available"

    # Check Docker daemon
    if ! docker info &> /dev/null; then
        fail "Docker daemon not running or not accessible"
        return
    fi
    pass "Docker daemon accessible"

    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        fail "Docker Compose not installed"
        return
    fi
    pass "Docker Compose available"

    # Check for required images in air-gapped environments
    if [[ "${AIRGAPPED:-false}" == "true" ]]; then
        local required_images=("postgres:14-alpine" "redis:7-alpine" "nginx:alpine")
        for image in "${required_images[@]}"; do
            if ! docker image inspect "$image" &> /dev/null; then
                fail "Required image $image not available in cache"
            else
                pass "Required image $image available in cache"
            fi
        done
    fi
}

validate_project_structure() {
    log "=== Validating Project Structure ==="

    # Check required directories
    local required_dirs=("docker" "disaster-recovery" "ci-cd" "scripts" "src-tauri" "web")
    for dir in "${required_dirs[@]}"; do
        if [[ ! -d "$dir" ]]; then
            fail "Required directory $dir missing"
        else
            pass "Directory $dir exists"
        fi
    done

    # Check configuration files
    local required_files=("Cargo.toml" "docker/docker-compose.yml" "docker/Dockerfile.rust")
    for file in "${required_files[@]}"; do
        if [[ ! -f "$file" ]]; then
            fail "Required configuration file $file missing"
        else
            pass "Configuration file $file exists"
        fi
    done

    # Check Rust toolchain
    if [[ -f "rust-toolchain.toml" ]]; then
        local toolchain=$(grep 'channel' rust-toolchain.toml | cut -d'"' -f2)
        if [[ "$toolchain" == "nightly"* ]]; then
            pass "Rust nightly toolchain configured"
        else
            warn "Non-nightly Rust toolchain detected: $toolchain"
        fi
    else
        warn "rust-toolchain.toml not found"
    fi
}

validate_security_config() {
    log "=== Validating Security Configuration ==="

    # Check deny.toml for security policies
    if [[ -f "deny.toml" ]]; then
        if grep -q "openssl" deny.toml && grep -q "md5" deny.toml; then
            pass "Security policies configured in deny.toml"
        else
            fail "Critical security policies missing from deny.toml"
        fi
    else
        fail "deny.toml not found - security policies undefined"
    fi

    # Check .gitignore security
    if [[ -f ".gitignore" ]]; then
        local sensitive_patterns=("secrets" "*.key" "*.pem")
        for pattern in "${sensitive_patterns[@]}"; do
            if grep -q "$pattern" .gitignore; then
                pass "Sensitive pattern $pattern ignored in git"
            else
                warn "Sensitive pattern $pattern not found in .gitignore"
            fi
        done
    fi

    # Check Docker security
    if [[ -f "docker/.dockerignore" ]]; then
        pass "Docker ignore file exists"
    else
        warn "Docker ignore file missing"
    fi
}

validate_backup_systems() {
    log "=== Validating Backup Systems ==="

    # Check backup scripts
    if [[ -x "disaster-recovery/scripts/automated-backup.sh" ]]; then
        pass "Automated backup script exists and executable"
    else
        fail "Automated backup script missing or not executable"
    fi

    # Check backup directories setup
    if [[ -d "disaster-recovery" ]]; then
        pass "Disaster recovery directory exists"
    else
        fail "Disaster recovery directory missing"
    fi

    # Validate backup configuration
    if [[ -f "disaster-recovery/README.md" ]]; then
        pass "Disaster recovery documentation exists"
    else
        fail "Disaster recovery documentation missing"
    fi
}

validate_ci_cd_configuration() {
    log "=== Validating CI/CD Configuration ==="

    # Check CI/CD configurations
    local ci_files=("ci-cd/Jenkinsfile" "ci-cd/.gitlab-ci.yml" "ci-cd/azure-pipelines.yml")
    local existing_ci=0

    for file in "${ci_files[@]}"; do
        if [[ -f "$file" ]]; then
            pass "CI/CD configuration $file exists"
            ((existing_ci++))
        fi
    done

    if [[ $existing_ci -eq 0 ]]; then
        fail "No CI/CD configurations found"
    fi

    # Check Dockerfile configurations
    local dockerfiles=("docker/Dockerfile.rust" "docker/Dockerfile.web" "docker/Dockerfile.tauri")
    for dockerfile in "${dockerfiles[@]}"; do
        if [[ -f "$dockerfile" ]]; then
            pass "Dockerfile $dockerfile exists"
        else
            fail "Required Dockerfile $dockerfile missing"
        fi
    done
}

validate_container_configuration() {
    log "=== Validating Container Configuration ==="

    # Check Docker Compose files
    local compose_files=("docker/docker-compose.yml" "docker/docker-compose.air-gapped.yml")
    for compose_file in "${compose_files[@]}"; do
        if [[ -f "$compose_file" ]]; then
            pass "Docker Compose file $compose_file exists"

            # Basic syntax validation
            if docker-compose -f "$compose_file" config --quiet 2>/dev/null; then
                pass "Docker Compose file $compose_file syntax valid"
            else
                fail "Docker Compose file $compose_file syntax invalid"
            fi
        else
            fail "Docker Compose file $compose_file missing"
        fi
    done

    # Check for secrets management
    local secrets_file="docker/docker-compose.yml"
    if grep -q "secrets:" "$secrets_file" 2>/dev/null; then
        pass "Docker Compose uses secrets management"
    else
        fail "Docker Compose does not use secrets management"
    fi
}

generate_validation_report() {
    log ""
    log "=== VALIDATION SUMMARY ==="
    log "Passed: $PASS_COUNT"
    log "Failed: $FAIL_COUNT"
    log "Warnings: $WARN_COUNT"
    log "Total Checks: $((PASS_COUNT + FAIL_COUNT + WARN_COUNT))"
    log ""
    log "Detailed log: $VALIDATION_LOG"

    # Exit with appropriate code
    if [[ $FAIL_COUNT -gt 0 ]]; then
        log "❌ Validation failed with $FAIL_COUNT critical issues"
        return 1
    elif [[ $WARN_COUNT -gt 0 ]]; then
        log "⚠️  Validation passed with $WARN_COUNT warnings"
        return 0
    else
        log "✅ All validations passed successfully"
        return 0
    fi
}

# Main validation routine
main() {
    log "Starting Infrastructure Validation for Rust AI IDE"
    log "Timestamp: $(date)"
    log "Environment: ${ENVIRONMENT:-unknown}"
    log ""

    # Run all validations
    validate_docker_setup
    validate_project_structure
    validate_security_config
    validate_backup_systems
    validate_ci_cd_configuration
    validate_container_configuration

    # Generate final report
    generate_validation_report
}

# Run main function
main "$@"
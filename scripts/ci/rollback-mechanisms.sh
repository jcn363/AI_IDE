#!/bin/bash

# Critical Bug Resolution - Automated Rollback Mechanisms
# This script provides automated rollback capabilities for failed deployments
# Implements blue-green deployment strategies and recovery procedures

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
ROLLBACK_DIR="${PROJECT_ROOT}/reports/rollbacks"
LOG_FILE="${ROLLBACK_DIR}/rollback.log"

# Deployment configuration
DOCKER_REGISTRY="${DOCKER_REGISTRY:-registry.local:5000}"
DOCKER_IMAGE_PREFIX="${DOCKER_IMAGE_PREFIX:-$DOCKER_REGISTRY/rust-ai-ide}"
DEPLOYMENT_ENV="${DEPLOYMENT_ENV:-production}"
ROLLBACK_TIMEOUT="${ROLLBACK_TIMEOUT:-600}"

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

log_rollback() {
    echo -e "${PURPLE}[ROLLBACK]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $*" | tee -a "$LOG_FILE"
}

log_deployment() {
    echo -e "${CYAN}[DEPLOYMENT]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $*" | tee -a "$LOG_FILE"
}

# Setup function
setup() {
    log_info "Setting up automated rollback mechanisms..."
    mkdir -p "$ROLLBACK_DIR"

    # Ensure docker is available
    if ! command -v docker &> /dev/null; then
        log_error "Docker is required for rollback operations"
        exit 1
    fi

    # Check docker registry access
    if ! docker login "$DOCKER_REGISTRY" --username "${CI_REGISTRY_USER:-}" --password "${CI_REGISTRY_PASSWORD:-}" 2>/dev/null; then
        log_warn "Docker registry login failed - ensure credentials are available"
    fi

    log_success "Rollback environment setup complete"
}

# Create deployment snapshot for rollback
create_deployment_snapshot() {
    log_info "Creating deployment snapshot for rollback protection..."

    local snapshot_file="${ROLLBACK_DIR}/deployment-snapshot-$(date +%Y%m%d-%H%M%S).json"
    local current_commit=$(git rev-parse HEAD)
    local current_images=$(docker images --format "table {{.Repository}}:{{.Tag}}" | grep "$DOCKER_IMAGE_PREFIX" || echo "")

    cat > "$snapshot_file" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "commit_sha": "$current_commit",
    "deployment_environment": "$DEPLOYMENT_ENV",
    "docker_images": $(echo "$current_images" | jq -R -s 'split("\n") | map(select(. != ""))'),
    "active_containers": $(docker ps --format 'json' | jq -s '.'),
    "docker_compose_files": ["docker/docker-compose.yml"],
    "rollback_available": true
}
EOF

    log_success "Deployment snapshot created: $snapshot_file"
    echo "$snapshot_file"
}

# Perform immediate rollback to previous version
perform_immediate_rollback() {
    local reason="$1"
    local snapshot_file="$2"

    log_rollback "Initiating immediate rollback due to: $reason"
    log_rollback "Using snapshot: $snapshot_file"

    if [[ ! -f "$snapshot_file" ]]; then
        log_error "Snapshot file not found: $snapshot_file"
        return 1
    fi

    # Extract rollback information
    local previous_commit=$(jq -r '.commit_sha' "$snapshot_file")
    local previous_images=$(jq -r '.docker_images[]' "$snapshot_file" 2>/dev/null || echo "")

    log_rollback "Rolling back to commit: $previous_commit"

    # Stop current deployment
    log_rollback "Stopping current deployment..."
    cd docker
    docker-compose down --timeout 30 || log_warn "Failed to stop current containers gracefully"

    # Restore previous images
    if [[ -n "$previous_images" ]]; then
        log_rollback "Restoring previous Docker images..."
        echo "$previous_images" | while read -r image; do
            if [[ -n "$image" ]]; then
                docker pull "$image" 2>/dev/null || log_warn "Failed to pull image: $image"
            fi
        done
    fi

    # Checkout previous commit
    cd "$PROJECT_ROOT"
    git checkout "$previous_commit" --force

    # Restart with previous version
    log_rollback "Restarting with previous version..."
    cd docker
    docker-compose up -d --build --force-recreate

    # Verify rollback
    if verify_deployment_health; then
        log_success "Rollback completed successfully"
        notify_rollback_success "$reason" "$previous_commit"
        return 0
    else
        log_error "Rollback verification failed"
        notify_rollback_failure "$reason" "$previous_commit"
        return 1
    fi
}

# Blue-green deployment rollback
perform_blue_green_rollback() {
    local reason="$1"

    log_deployment "Performing blue-green deployment rollback due to: $reason"

    cd docker

    # Get current active environment
    local active_env
    if docker-compose ps | grep -q "Up"; then
        active_env="blue"
        local inactive_env="green"
    else
        active_env="green"
        local inactive_env="blue"
    fi

    log_deployment "Active environment: $active_env, Rolling back to: $inactive_env"

    # Switch traffic to inactive environment
    log_deployment "Switching traffic to $inactive_env environment..."
    docker-compose --project-name "rust-ai-ide-$inactive_env" up -d

    # Verify inactive environment health
    if verify_deployment_health "$inactive_env"; then
        # Stop active environment
        log_deployment "Stopping $active_env environment..."
        docker-compose --project-name "rust-ai-ide-$active_env" down --timeout 60

        log_success "Blue-green rollback completed successfully"
        notify_rollback_success "$reason" "$inactive_env"
        return 0
    else
        log_error "Blue-green rollback verification failed - keeping active environment"
        notify_rollback_failure "$reason" "$inactive_env"
        return 1
    fi
}

# Gradual rollback with canary strategy
perform_canary_rollback() {
    local reason="$1"
    local rollback_percentage="${ROLLBACK_PERCENTAGE:-25}"

    log_deployment "Performing canary rollback due to: $reason"
    log_deployment "Rolling back $rollback_percentage% of traffic"

    cd docker

    # Scale down current deployment gradually
    local current_scale
    current_scale=$(docker-compose ps | grep "rust-backend" | wc -l)

    if [[ $current_scale -gt 1 ]]; then
        local rollback_instances=$((current_scale * rollback_percentage / 100))
        rollback_instances=$((rollback_instances > 0 ? rollback_instances : 1))

        log_deployment "Scaling down to $((current_scale - rollback_instances)) instances..."

        docker-compose up -d --scale rust-backend=$((current_scale - rollback_instances)) --no-recreate

        # Monitor for 5 minutes
        local monitor_time=300
        local healthy=true

        for ((i=0; i<monitor_time; i+=30)); do
            if ! verify_deployment_health; then
                healthy=false
                break
            fi
            sleep 30
        done

        if [[ "$healthy" == "true" ]]; then
            log_success "Canary rollback successful - scaling down complete"
            notify_rollback_success "$reason" "canary-$rollback_percentage%"
            return 0
        else
            log_warn "Canary rollback detected issues - rolling back completely"
            perform_immediate_rollback "Canary rollback health check failed" "$(create_deployment_snapshot)"
            return 1
        fi
    else
        log_warn "Not enough instances for canary rollback, performing immediate rollback"
        perform_immediate_rollback "$reason" "$(create_deployment_snapshot)"
    fi
}

# Verify deployment health after rollback
verify_deployment_health() {
    local env_name="${1:-}"

    log_info "Verifying deployment health..."

    # Health check endpoints
    local health_endpoints=("http://localhost:8080/health" "http://localhost:3000/health")

    if [[ -n "$env_name" ]]; then
        health_endpoints=("http://localhost:8080/health" "http://localhost:3000/health")
    fi

    local max_attempts=30
    local attempt=1

    while [[ $attempt -le $max_attempts ]]; do
        local all_healthy=true

        for endpoint in "${health_endpoints[@]}"; do
            if ! curl -f --max-time 10 "$endpoint" > /dev/null 2>&1; then
                all_healthy=false
                break
            fi
        done

        if [[ "$all_healthy" == "true" ]]; then
            log_success "Deployment health verification passed"
            return 0
        fi

        log_info "Health check attempt $attempt/$max_attempts failed, waiting 10 seconds..."
        sleep 10
        ((attempt++))
    done

    log_error "Deployment health verification failed after $max_attempts attempts"
    return 1
}

# Monitor deployment and trigger automatic rollback if needed
monitor_and_auto_rollback() {
    local monitor_duration="${MONITOR_DURATION:-3600}"  # 1 hour default
    local health_check_interval="${HEALTH_CHECK_INTERVAL:-60}"  # 1 minute
    local failure_threshold="${FAILURE_THRESHOLD:-3}"

    log_info "Starting automatic rollback monitoring for ${monitor_duration}s"
    log_info "Health check interval: ${health_check_interval}s, Failure threshold: $failure_threshold"

    local consecutive_failures=0
    local start_time=$(date +%s)

    while true; do
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))

        if [[ $elapsed -ge $monitor_duration ]]; then
            log_success "Monitoring period completed without requiring rollback"
            return 0
        fi

        if ! verify_deployment_health > /dev/null 2>&1; then
            ((consecutive_failures++))
            log_warn "Health check failed ($consecutive_failures/$failure_threshold)"

            if [[ $consecutive_failures -ge $failure_threshold ]]; then
                log_error "Failure threshold reached, triggering automatic rollback"
                local snapshot=$(create_deployment_snapshot)
                perform_immediate_rollback "Automatic rollback due to health check failures" "$snapshot"
                return $?
            fi
        else
            if [[ $consecutive_failures -gt 0 ]]; then
                log_info "Health check recovered (failures reset to 0)"
                consecutive_failures=0
            fi
        fi

        sleep "$health_check_interval"
    done
}

# Notify about rollback success
notify_rollback_success() {
    local reason="$1"
    local target="$2"

    log_success "ROLLBACK SUCCESSFUL: $reason -> $target"

    # Send notifications (implement based on your notification system)
    if [[ -n "${SLACK_WEBHOOK_URL:-}" ]]; then
        curl -X POST -H 'Content-type: application/json' \
            --data "{\"text\":\"🚨 Rust AI IDE Rollback Successful\\nReason: $reason\\nTarget: $target\\nTime: $(date)\"}" \
            "$SLACK_WEBHOOK_URL" || log_warn "Slack notification failed"
    fi
}

# Notify about rollback failure
notify_rollback_failure() {
    local reason="$1"
    local target="$2"

    log_error "ROLLBACK FAILED: $reason -> $target"

    # Send critical notifications
    if [[ -n "${SLACK_WEBHOOK_URL:-}" ]]; then
        curl -X POST -H 'Content-type: application/json' \
            --data "{\"text\":\"💥 CRITICAL: Rust AI IDE Rollback Failed\\nReason: $reason\\nTarget: $target\\nTime: $(date)\\nManual intervention required!\"}" \
            "$SLACK_WEBHOOK_URL" || log_warn "Slack notification failed"
    fi
}

# Generate rollback report
generate_rollback_report() {
    local rollback_type="$1"
    local success="$2"
    local reason="$3"
    local target="$4"

    local report_file="${ROLLBACK_DIR}/rollback-report-$(date +%Y%m%d-%H%M%S).json"

    cat > "$report_file" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "rollback_type": "$rollback_type",
    "success": $success,
    "reason": "$reason",
    "target": "$target",
    "environment": "$DEPLOYMENT_ENV",
    "docker_registry": "$DOCKER_REGISTRY",
    "execution_details": {
        "timeout": $ROLLBACK_TIMEOUT,
        "user": "${USER:-unknown}",
        "working_directory": "$PWD"
    },
    "system_info": {
        "docker_version": "$(docker --version 2>/dev/null || echo 'unknown')",
        "git_commit": "$(git rev-parse HEAD 2>/dev/null || echo 'unknown')",
        "uname": "$(uname -a)"
    }
}
EOF

    log_info "Rollback report generated: $report_file"
}

# List available rollback snapshots
list_rollback_snapshots() {
    log_info "Available rollback snapshots:"

    find "$ROLLBACK_DIR" -name "deployment-snapshot-*.json" -type f | while read -r snapshot; do
        local timestamp=$(jq -r '.timestamp' "$snapshot" 2>/dev/null || echo "unknown")
        local commit=$(jq -r '.commit_sha' "$snapshot" 2>/dev/null || echo "unknown")
        echo "  $(basename "$snapshot"): $timestamp (commit: ${commit:0:8})"
    done
}

# Main execution function
main() {
    local action="${1:-help}"

    log_info "Starting automated rollback mechanisms subsystem"
    log_info "Project root: $PROJECT_ROOT"
    log_info "Deployment environment: $DEPLOYMENT_ENV"
    log_info "Docker registry: $DOCKER_REGISTRY"

    setup

    case "$action" in
        "snapshot")
            create_deployment_snapshot
            ;;
        "immediate")
            local reason="${2:-Manual rollback request}"
            local snapshot="${3:-$(find "$ROLLBACK_DIR" -name "deployment-snapshot-*.json" -type f | head -1)}"
            perform_immediate_rollback "$reason" "$snapshot"
            generate_rollback_report "immediate" $? "$reason" "$snapshot"
            ;;
        "blue-green")
            local reason="${2:-Blue-green rollback request}"
            perform_blue_green_rollback "$reason"
            generate_rollback_report "blue-green" $? "$reason" "environment-switch"
            ;;
        "canary")
            local reason="${2:-Canary rollback request}"
            perform_canary_rollback "$reason"
            generate_rollback_report "canary" $? "$reason" "${ROLLBACK_PERCENTAGE:-25}%"
            ;;
        "monitor")
            monitor_and_auto_rollback
            ;;
        "list")
            list_rollback_snapshots
            ;;
        "help"|*)
            echo "Usage: $0 <action> [options]"
            echo ""
            echo "Actions:"
            echo "  snapshot                    Create deployment snapshot"
            echo "  immediate <reason> [snapshot] Perform immediate rollback"
            echo "  blue-green <reason>          Perform blue-green rollback"
            echo "  canary <reason>              Perform canary rollback"
            echo "  monitor                      Start automatic monitoring"
            echo "  list                         List available snapshots"
            echo "  help                         Show this help"
            echo ""
            echo "Environment variables:"
            echo "  DEPLOYMENT_ENV               Deployment environment (default: production)"
            echo "  DOCKER_REGISTRY              Docker registry URL"
            echo "  ROLLBACK_TIMEOUT             Rollback timeout in seconds (default: 600)"
            echo "  MONITOR_DURATION             Monitoring duration in seconds (default: 3600)"
            echo "  HEALTH_CHECK_INTERVAL        Health check interval in seconds (default: 60)"
            echo "  FAILURE_THRESHOLD            Consecutive failures before rollback (default: 3)"
            echo "  ROLLBACK_PERCENTAGE          Percentage for canary rollback (default: 25)"
            exit 0
            ;;
    esac

    log_info "Automated rollback mechanisms operation complete"
}

# Execute main function
main "$@"
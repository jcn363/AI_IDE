#!/bin/bash

# Deployment Helpers Script for Rust AI IDE
# Comprehensive deployment automation utilities for CI/CD pipelines
# Author: DevOps Engineering Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
DEPLOY_LOG="${PROJECT_ROOT}/deployment-helpers.log"
DEPLOYMENT_DIR="${PROJECT_ROOT}/cloud-deployment"
HELM_DIR="${DEPLOYMENT_DIR}/helm/rust-ai-ide"
START_TIME=$(date +%s)

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${DEPLOY_LOG}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${DEPLOY_LOG}" >&2
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${DEPLOY_LOG}"
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${DEPLOY_LOG}"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [COMMAND] [OPTIONS]

Deployment helpers and automation utilities for Rust AI IDE CI/CD pipelines.

COMMANDS:
    validate-deployment   Validate deployment configuration and prerequisites
    prepare-helm-charts   Prepare and validate Helm charts
    deploy-staging        Deploy to staging environment
    deploy-production     Deploy to production environment with blue-green
    rollback              Perform automated rollback to previous version
    monitor-deployment    Monitor deployment status and health checks
    scale-services        Scale deployment services up/down
    update-config         Update deployment configuration dynamically
    cleanup-old-deployments  Clean up old deployment artifacts

OPTIONS:
    -h, --help              Show this help message
    -v, --verbose           Enable verbose output
    -n, --namespace NS      Kubernetes namespace (default: rust-ai-ide-staging)
    --environment ENV       Target environment (staging, production)
    --strategy STRATEGY     Deployment strategy (blue-green, canary, rolling)
    --image-tag TAG         Docker image tag to deploy
    --wait-timeout SEC      Wait timeout for deployment operations (default: 600)
    --health-check-endpoint Endpoint for health checks
    --rollback-tag TAG      Specific tag to rollback to
    --scale-factor N        Scale factor for service scaling (1.0 = no change)
    --dry-run              Perform dry-run operations only

EXAMPLES:
    $0 validate-deployment --environment staging
    $0 deploy-staging --image-tag v1.2.3 --wait-timeout 1200
    $0 deploy-production --strategy blue-green --verbose
    $0 rollback --rollback-tag v1.1.0
    $0 monitor-deployment --namespace rust-ai-ide-prod
    $0 scale-services --scale-factor 1.5

EOF
}

# Parse command line arguments
COMMAND=""
VERBOSE=false
NAMESPACE="rust-ai-ide-staging"
ENVIRONMENT="staging"
STRATEGY="blue-green"
IMAGE_TAG=""
WAIT_TIMEOUT=600
HEALTH_CHECK_ENDPOINT="/health"
ROLLBACK_TAG=""
SCALE_FACTOR="1.0"
DRY_RUN=false

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
        -n|--namespace)
            NAMESPACE="$2"
            shift 2
            ;;
        --environment)
            ENVIRONMENT="$2"
            shift 2
            ;;
        --strategy)
            STRATEGY="$2"
            shift 2
            ;;
        --image-tag)
            IMAGE_TAG="$2"
            shift 2
            ;;
        --wait-timeout)
            WAIT_TIMEOUT="$2"
            shift 2
            ;;
        --health-check-endpoint)
            HEALTH_CHECK_ENDPOINT="$2"
            shift 2
            ;;
        --rollback-tag)
            ROLLBACK_TAG="$2"
            shift 2
            ;;
        --scale-factor)
            SCALE_FACTOR="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        validate-deployment|prepare-helm-charts|deploy-staging|deploy-production|rollback|monitor-deployment|scale-services|update-config|cleanup-old-deployments)
            if [[ -z "${COMMAND}" ]]; then
                COMMAND="$1"
            else
                log_error "Multiple commands not supported: ${COMMAND} and $1"
                exit 1
            fi
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

if [[ -z "${COMMAND}" ]]; then
    log_error "No command specified"
    usage
    exit 1
fi

# Function to check deployment prerequisites
check_deployment_prerequisites() {
    log_info "Checking deployment prerequisites..."

    local required_tools=("helm" "kubectl" "docker")

    for tool in "${required_tools[@]}"; do
        if ! command -v "${tool}" >/dev/null 2>&1; then
            log_error "Required tool '${tool}' not found"
            return 1
        fi
    done

    # Check Kubernetes connectivity
    if ! kubectl cluster-info >/dev/null 2>&1; then
        log_error "Cannot connect to Kubernetes cluster"
        return 1
    fi

    # Check namespace exists
    if ! kubectl get namespace "${NAMESPACE}" >/dev/null 2>&1; then
        log_warning "Namespace '${NAMESPACE}' does not exist. Creating..."
        kubectl create namespace "${NAMESPACE}" || log_error "Failed to create namespace ${NAMESPACE}"
    fi

    # Check Helm chart exists
    if [[ ! -d "${HELM_DIR}" ]]; then
        log_error "Helm chart directory not found: ${HELM_DIR}"
        return 1
    fi

    log_success "Deployment prerequisites check passed"
}

# Function to validate deployment configuration
validate_deployment_config() {
    log_info "Validating deployment configuration..."

    local validation_errors=0

    # Validate Helm chart
    if ! helm template "${HELM_DIR}" --dry-run >/dev/null 2>&1; then
        log_error "Helm chart validation failed"
        validation_errors=$((validation_errors + 1))
    fi

    # Validate Docker images exist
    if [[ -n "${IMAGE_TAG}" ]]; then
        local images=("rust-ai-ide/ai-inference:${IMAGE_TAG}" "rust-ai-ide/lsp:${IMAGE_TAG}")

        for image in "${images[@]}"; do
            if ! docker image inspect "${image}" >/dev/null 2>&1; then
                log_error "Docker image not found: ${image}"
                validation_errors=$((validation_errors + 1))
            fi
        done
    fi

    # Validate values.yaml
    local values_file="${HELM_DIR}/values.yaml"
    if [[ ! -f "${values_file}" ]]; then
        log_error "Helm values file not found: ${values_file}"
        validation_errors=$((validation_errors + 1))
    else
        # Check for required values
        if ! grep -q "image:" "${values_file}"; then
            log_error "Required image configuration missing in values.yaml"
            validation_errors=$((validation_errors + 1))
        fi
    fi

    # Validate environment-specific configurations
    case "${ENVIRONMENT}" in
        staging)
            NAMESPACE="rust-ai-ide-staging"
            ;;
        production)
            NAMESPACE="rust-ai-ide-prod"
            ;;
        *)
            log_error "Unknown environment: ${ENVIRONMENT}"
            validation_errors=$((validation_errors + 1))
            ;;
    esac

    if [[ ${validation_errors} -gt 0 ]]; then
        log_error "${validation_errors} validation errors found"
        return 1
    fi

    log_success "Deployment configuration validation passed"
}

# Function to prepare Helm charts
prepare_helm_charts() {
    log_info "Preparing Helm charts..."

    cd "${HELM_DIR}"

    # Update Helm dependencies
    helm dependency update

    # Lint the chart
    if ! helm lint .; then
        log_error "Helm chart linting failed"
        return 1
    fi

    # Create environment-specific values override
    local env_values="${PROJECT_ROOT}/deployment-values-${ENVIRONMENT}.yaml"

    cat > "${env_values}" << EOF
environment: ${ENVIRONMENT}
namespace: ${NAMESPACE}
deployment:
  strategy: ${STRATEGY}
  imageTag: ${IMAGE_TAG:-latest}
  timestamp: $(date +%s)

resources:
  limits:
    cpu: ${ENVIRONMENT_CPU_LIMIT:-1000m}
    memory: ${ENVIRONMENT_MEMORY_LIMIT:-2Gi}
  requests:
    cpu: ${ENVIRONMENT_CPU_REQUEST:-500m}
    memory: ${ENVIRONMENT_MEMORY_REQUEST:-1Gi}

ingress:
  enabled: ${ENVIRONMENT_INGRESS_ENABLED:-true}
  className: ${ENVIRONMENT_INGRESS_CLASS:-nginx}
  host: ${ENVIRONMENT_INGRESS_HOST:-rust-ai-ide.${ENVIRONMENT}.example.com}
EOF

    log_info "Environment values created: ${env_values}"

    # Dry-run template generation
    helm template . --values "${env_values}" --dry-run > "${PROJECT_ROOT}/helm-template-${ENVIRONMENT}.yaml"

    log_success "Helm charts prepared successfully"
}

# Function to perform blue-green deployment
perform_blue_green_deployment() {
    log_info "Performing blue-green deployment..."

    local release_name="rust-ai-ide-${ENVIRONMENT}"
    local green_release="${release_name}-green"
    local blue_release="${release_name}-blue"

    # Determine which color is currently active
    local active_release
    if kubectl get service "${release_name}" -n "${NAMESPACE}" >/dev/null 2>&1; then
        active_release=$(kubectl get service "${release_name}" -n "${NAMESPACE}" -o jsonpath='{.spec.selector.color}' 2>/dev/null || echo "blue")
    else
        active_release="blue"
    fi

    local inactive_color="blue"
    if [[ "${active_release}" == "blue" ]]; then
        inactive_color="green"
    fi

    local inactive_release="${release_name}-${inactive_color}"

    log_info "Active color: ${active_release}, deploying to: ${inactive_color}"

    # Deploy to inactive environment
    local deploy_values="${PROJECT_ROOT}/deployment-values-${ENVIRONMENT}.yaml"

    if [[ "${DRY_RUN}" == true ]]; then
        log_info "DRY RUN: Would deploy ${inactive_release}"
        helm template "${inactive_release}" "${HELM_DIR}" \
            --values "${deploy_values}" \
            --set color="${inactive_color}" \
            --namespace "${NAMESPACE}"
        return 0
    fi

    # Install/upgrade the inactive environment
    helm upgrade --install "${inactive_release}" "${HELM_DIR}" \
        --values "${deploy_values}" \
        --set color="${inactive_color}" \
        --namespace "${NAMESPACE}" \
        --wait \
        --timeout "${WAIT_TIMEOUT}s"

    # Wait for the deployment to become ready
    if ! kubectl rollout status deployment "${inactive_release}" -n "${NAMESPACE}" --timeout="${WAIT_TIMEOUT}s"; then
        log_error "Deployment rollout failed"
        return 1
    fi

    # Perform health checks on the new deployment
    if ! perform_health_checks "${inactive_release}"; then
        log_error "Health checks failed for ${inactive_release}"
        return 1
    fi

    # Switch traffic to the new deployment
    kubectl patch service "${release_name}" -n "${NAMESPACE}" \
        --type='json' \
        -p="[{\"op\": \"replace\", \"path\": \"/spec/selector/color\", \"value\": \"${inactive_color}\"}]"

    log_success "Traffic switched to ${inactive_color} environment"

    # Clean up old deployment after some time (async)
    {
        sleep 300  # Wait 5 minutes for traffic to stabilize
        helm uninstall "${release_name}-${active_release}" -n "${NAMESPACE}" || log_warning "Failed to cleanup old deployment"
    } &

    return 0
}

# Function to perform health checks
perform_health_checks() {
    local release_name="$1"
    log_info "Performing health checks for ${release_name}..."

    local service_url
    service_url=$(kubectl get svc "${release_name}" -n "${NAMESPACE}" -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>/dev/null || kubectl get svc "${release_name}" -n "${NAMESPACE}" -o jsonpath='{.spec.clusterIP}')

    if [[ -z "${service_url}" ]]; then
        log_error "Could not determine service URL for health checks"
        return 1
    fi

    local health_url="http://${service_url}${HEALTH_CHECK_ENDPOINT}"
    local max_attempts=30
    local attempt=1

    while [[ ${attempt} -le ${max_attempts} ]]; do
        log_info "Health check attempt ${attempt}/${max_attempts}: ${health_url}"

        if curl -f "${health_url}" --max-time 10 >/dev/null 2>&1; then
            log_success "Health check passed"
            return 0
        fi

        sleep 10
        attempt=$((attempt + 1))
    done

    log_error "Health checks failed after ${max_attempts} attempts"
    return 1
}

# Function to perform rollback
perform_rollback() {
    log_info "Performing rollback..."

    local release_name="rust-ai-ide-${ENVIRONMENT}"
    local current_color
    current_color=$(kubectl get service "${release_name}" -n "${NAMESPACE}" -o jsonpath='{.spec.selector.color}' 2>/dev/null || echo "")

    if [[ -z "${current_color}" ]]; then
        log_error "Cannot determine current active color for rollback"
        return 1
    fi

    local previous_color="blue"
    if [[ "${current_color}" == "blue" ]]; then
        previous_color="green"
    fi

    local previous_release="${release_name}-${previous_color}"

    # Check if previous deployment exists
    if ! kubectl get deployment "${previous_release}" -n "${NAMESPACE}" >/dev/null 2>&1; then
        log_error "Previous deployment ${previous_release} not found"
        return 1
    fi

    # Check if previous deployment is healthy
    if ! perform_health_checks "${previous_release}"; then
        log_error "Previous deployment is not healthy"
        return 1
    fi

    # Switch traffic back
    kubectl patch service "${release_name}" -n "${NAMESPACE}" \
        --type='json' \
        -p="[{\"op\": \"replace\", \"path\": \"/spec/selector/color\", \"value\": \"${previous_color}\"}]"

    log_success "Rollback completed - traffic switched back to ${previous_color}"
}

# Function to monitor deployment
monitor_deployment() {
    log_info "Monitoring deployment status..."

    local release_name="rust-ai-ide-${ENVIRONMENT}"

    # Get deployment status
    kubectl get deployments -n "${NAMESPACE}" -l app="${release_name}"

    # Get pod status
    kubectl get pods -n "${NAMESPACE}" -l app="${release_name}"

    # Get service status
    kubectl get services -n "${NAMESPACE}" -l app="${release_name}"

    # Check for any issues
    local failed_pods
    failed_pods=$(kubectl get pods -n "${NAMESPACE}" -l app="${release_name}" --no-headers | grep -v Running | wc -l)

    if [[ ${failed_pods} -gt 0 ]]; then
        log_warning "Found ${failed_pods} pods not in Running state"
        kubectl get pods -n "${NAMESPACE}" -l app="${release_name}" --no-headers | grep -v Running
    fi

    # Check resource usage
    kubectl top pods -n "${NAMESPACE}" -l app="${release_name}" || log_warning "Cannot get resource usage"

    log_success "Deployment monitoring completed"
}

# Function to scale services
scale_services() {
    log_info "Scaling services by factor: ${SCALE_FACTOR}"

    local deployments
    deployments=$(kubectl get deployments -n "${NAMESPACE}" -l app="rust-ai-ide-${ENVIRONMENT}" -o jsonpath='{.items[*].metadata.name}')

    for deployment in ${deployments}; do
        local current_replicas
        current_replicas=$(kubectl get deployment "${deployment}" -n "${NAMESPACE}" -o jsonpath='{.spec.replicas}')

        local new_replicas
        new_replicas=$(echo "scale=0; ${current_replicas} * ${SCALE_FACTOR}" | bc 2>/dev/null || echo "${current_replicas}")

        # Ensure minimum of 1 replica
        if [[ $(echo "${new_replicas} < 1" | bc 2>/dev/null) -eq 1 ]]; then
            new_replicas=1
        fi

        log_info "Scaling ${deployment} from ${current_replicas} to ${new_replicas} replicas"

        if [[ "${DRY_RUN}" != true ]]; then
            kubectl scale deployment "${deployment}" -n "${NAMESPACE}" --replicas="${new_replicas}"
        fi
    done

    log_success "Service scaling completed"
}

# Function to cleanup old deployments
cleanup_old_deployments() {
    log_info "Cleaning up old deployments..."

    local deployments
    deployments=$(kubectl get deployments -n "${NAMESPACE}" -o jsonpath='{.items[*].metadata.name}')

    for deployment in ${deployments}; do
        # Skip active deployments
        if [[ "${deployment}" == "rust-ai-ide-${ENVIRONMENT}" ]]; then
            continue
        fi

        # Remove deployments older than 7 days
        local creation_time
        creation_time=$(kubectl get deployment "${deployment}" -n "${NAMESPACE}" -o jsonpath='{.metadata.creationTimestamp}')
        local age_days=$(( ( $(date +%s) - $(date -d "${creation_time}" +%s) ) / 86400 ))

        if [[ ${age_days} -gt 7 ]]; then
            log_info "Removing old deployment: ${deployment} (${age_days} days old)"
            kubectl delete deployment "${deployment}" -n "${NAMESPACE}"
        fi
    done

    log_success "Cleanup completed"
}

# Main function
main() {
    log_info "Starting deployment helpers script"
    log_info "Command: ${COMMAND}"
    log_info "Environment: ${ENVIRONMENT}"
    log_info "Namespace: ${NAMESPACE}"
    log_info "Log file: ${DEPLOY_LOG}"

    # Trap to ensure cleanup on exit
    trap 'log_info "Deployment helpers completed (exit code: $?)"; [[ -d "${PROJECT_ROOT}" ]] && echo "Logs available at: ${DEPLOY_LOG}"' EXIT

    local exit_code=0

    case "${COMMAND}" in
        validate-deployment)
            check_deployment_prerequisites
            validate_deployment_config
            ;;
        prepare-helm-charts)
            prepare_helm_charts
            ;;
        deploy-staging)
            ENVIRONMENT="staging"
            NAMESPACE="rust-ai-ide-staging"
            perform_blue_green_deployment
            ;;
        deploy-production)
            ENVIRONMENT="production"
            NAMESPACE="rust-ai-ide-prod"
            perform_blue_green_deployment
            ;;
        rollback)
            perform_rollback
            ;;
        monitor-deployment)
            monitor_deployment
            ;;
        scale-services)
            scale_services
            ;;
        update-config)
            log_error "Update config not yet implemented"
            exit_code=1
            ;;
        cleanup-old-deployments)
            cleanup_old_deployments
            ;;
        *)
            log_error "Unknown command: ${COMMAND}"
            usage
            exit_code=1
            ;;
    esac

    local end_time=$(date +%s)
    log_info "Deployment helpers completed in $((end_time - START_TIME)) seconds"

    return $exit_code
}

# Run main function
main "$@"
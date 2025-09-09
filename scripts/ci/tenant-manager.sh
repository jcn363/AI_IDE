#!/bin/bash

# Enterprise Tenant Manager for Rust AI IDE
# Manages multi-tenant deployments with isolation and resource controls
# Author: DevOps Engineering Team
# Version: 1.0.0

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TENANT_CONFIG="${PROJECT_ROOT}/cloud-deployment/tenants.yaml"
TEMPLATE_FILE="${PROJECT_ROOT}/cloud-deployment/k8s/tenant-template.yaml"
DEPLOY_LOG="${PROJECT_ROOT}/tenant-deploy.log"
START_TIME=$(date +%s)

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${DEPLOY_LOG}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${DEPLOY_LOG}" >&2
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${DEPLOY_LOG}"
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${DEPLOY_LOG}"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [COMMAND] [TENANT_NAME] [OPTIONS]

Enterprise tenant management for Rust AI IDE multi-tenant deployments.

COMMANDS:
    list                List all configured tenants
    deploy TNAME        Deploy a specific tenant
    remove TNAME        Remove a tenant from the cluster
    update TNAME        Update tenant configuration
    status TNAME        Show tenant status
    secrets TNAME       Setup tenant secrets (database, SMTP)
    backup TNAME        Create tenant backup
    restore TNAME       Restore tenant from backup
    monitoring TNAME    Setup tenant-specific monitoring

TENANT_NAME:
    The name of the tenant (must match configuration)

OPTIONS:
    -h, --help          Show this help message
    -v, --verbose       Enable verbose output
    -d, --dry-run       Perform dry-run operations only
    --secrets-only      Only setup secrets for tenant

EXAMPLES:
    $0 deploy acme-corp
    $0 list
    $0 secrets startup-xyz
    $0 monitoring university
    $0 remove startup-xyz --dry-run

EOF
}

# Parse command line arguments
COMMAND=""
TENANT_NAME=""
VERBOSE=false
DRY_RUN=false
SECRETS_ONLY=false

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
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        --secrets-only)
            SECRETS_ONLY=true
            shift
            ;;
        list|deploy|remove|update|status|secrets|backup|restore|monitoring)
            if [[ -z "${COMMAND}" ]]; then
                COMMAND="$1"
            else
                log_error "Multiple commands not supported: ${COMMAND} and $1"
                exit 1
            fi
            if [[ "$1" != "list" ]]; then
                shift
                TENANT_NAME="$1"
            fi
            ;;
        *)
            if [[ -z "${TENANT_NAME}" && "${COMMAND}" != "list" ]]; then
                TENANT_NAME="$1"
            else
                log_error "Unknown option: $1"
                usage
                exit 1
            fi
            ;;
    esac
done

if [[ -z "${COMMAND}" ]]; then
    log_error "No command specified"
    usage
    exit 1
fi

# Function to load tenant configuration
load_tenant_config() {
    local tenant_key="$1"

    if ! command -v yq >/dev/null 2>&1; then
        log_error "yq is required for parsing YAML configuration"
        exit 1
    fi

    if [[ ! -f "${TENANT_CONFIG}" ]]; then
        log_error "Tenant configuration file not found: ${TENANT_CONFIG}"
        exit 1
    fi

    # Extract tenant configuration as shell variables
    eval "$(yq -r ".tenants.${tenant_key} | to_entries | map (\"export \(.key | upcase)=\"\(.value | @sh)\" \") | .[]" "${TENANT_CONFIG}")"

    # Handle nested objects
    eval "$(yq -r ".tenants.${tenant_key}.quota | to_entries | map (\"export QUOTA_\(.key | upcase)=\"\(.value | @sh)\" \") | .[]" "${TENANT_CONFIG}")"
    eval "$(yq -r ".tenants.${tenant_key}.resources | to_entries | map (\"export RESOURCES_\(.key | upcase)=\"\(.value | @sh)\" \") | .[]" "${TENANT_CONFIG}")"
    eval "$(yq -r ".tenants.${tenant_key}.database | to_entries | map (\"export DATABASE_\(.key | upcase)=\"\(.value | @sh)\" \") | .[]" "${TENANT_CONFIG}")"
    eval "$(yq -r ".tenants.${tenant_key}.smtp | to_entries | map (\"export SMTP_\(.key | upcase)=\"\(.value | @sh)\" \") | .[]" "${TENANT_CONFIG}")"

    log_info "Loaded configuration for tenant: ${tenant_key}"
}

# Function to substitute variables in template
substitute_template() {
    local tenant_key="$1"
    local output_file="$2"

    if [[ ! -f "${TEMPLATE_FILE}" ]]; then
        log_error "Template file not found: ${TEMPLATE_FILE}"
        exit 1
    fi

    # Create a temporary file for variable substitution
    local temp_file="/tmp/tenant-template-${TENANT_ID}.yaml"
    cp "${TEMPLATE_FILE}" "${temp_file}"

    # Use sed to substitute variables
    sed -i "s|\$TENANT_ID|${ID}|g" "${temp_file}"
    sed -i "s|\$TENANT_NAME|${NAME}|g" "${temp_file}"
    sed -i "s|\$TENANT_DOMAIN|${DOMAIN}|g" "${temp_file}"
    sed -i "s|\$TENANT_USER_EMAIL|${TENANT_USER_EMAIL:-admin@${DOMAIN}}|g" "${temp_file}"
    sed -i "s|\$SLA_LEVEL|${SLA_LEVEL}|g" "${temp_file}"
    sed -i "s|\$POD_QUOTA|${QUOTA_PODS}|g" "${temp_file}"
    sed -i "s|\$SERVICE_QUOTA|${QUOTA_SERVICES}|g" "${temp_file}"
    sed -i "s|\$PVC_QUOTA|${QUOTA_PVCS}|g" "${temp_file}"
    sed -i "s|\$CPU_REQUEST_QUOTA|${QUOTA_CPU_REQUEST}|g" "${temp_file}"
    sed -i "s|\$CPU_LIMIT_QUOTA|${QUOTA_CPU_LIMIT}|g" "${temp_file}"
    sed -i "s|\$MEMORY_REQUEST_QUOTA|${QUOTA_MEMORY_REQUEST}|g" "${temp_file}"
    sed -i "s|\$MEMORY_LIMIT_QUOTA|${QUOTA_MEMORY_LIMIT}|g" "${temp_file}"
    sed -i "s|\$REPLICA_COUNT|${REPLICAS}|g" "${temp_file}"
    sed -i "s|\$CPU_LIMIT|${RESOURCES_CPU_LIMIT}|g" "${temp_file}"
    sed -i "s|\$CPU_REQUEST|${RESOURCES_CPU_REQUEST}|g" "${temp_file}"
    sed -i "s|\$MEMORY_LIMIT|${RESOURCES_MEMORY_LIMIT}|g" "${temp_file}"
    sed -i "s|\$MEMORY_REQUEST|${RESOURCES_MEMORY_REQUEST}|g" "${temp_file}"
    sed -i "s|\$TENANT_DB_HOST|${DATABASE_HOST}|g" "${temp_file}"
    sed -i "s|\$IMAGE_TAG|${IMAGE_TAG:-latest}|g" "${temp_file}"
    sed -i "s|\$CERT_ISSUER|letsencrypt-prod|g" "${temp_file}"

    mv "${temp_file}" "${output_file}"
    log_info "Template substituted and saved to: ${output_file}"
}

# Function to deploy tenant
deploy_tenant() {
    local tenant_key="$1"

    log_info "Deploying tenant: ${tenant_key}"

    # Load configuration
    load_tenant_config "${tenant_key}"

    # Check if namespace already exists
    if kubectl get namespace "rust-ai-ide-tenant-${ID}" >/dev/null 2>&1; then
        if [[ "${SECRETS_ONLY}" == false ]]; then
            log_warning "Tenant namespace already exists. Use 'update' command to modify."
            return 1
        fi
    fi

    # Create output directory
    local output_dir="/tmp/tenant-${ID}"
    mkdir -p "${output_dir}"

    # Substitute template
    local rendered_file="${output_dir}/tenant-${ID}.yaml"
    substitute_template "${tenant_key}" "${rendered_file}"

    if [[ "${DRY_RUN}" == true ]]; then
        log_info "DRY RUN: Would deploy tenant ${tenant_key}"
        log_info "Rendered manifest:"
        cat "${rendered_file}"
        return 0
    fi

    # Deploy the tenant
    if [[ "${SECRETS_ONLY}" == false ]]; then
        log_info "Applying tenant manifests..."
        kubectl apply -f "${rendered_file}"
    fi

    # Setup secrets
    setup_tenant_secrets "${tenant_key}"

    # Wait for deployments to be ready
    if [[ "${SECRETS_ONLY}" == false ]]; then
        log_info "Waiting for tenant deployments to be ready..."
        kubectl wait --for=condition=available --timeout=600s deployment/ai-inference-tenant-${ID} -n "rust-ai-ide-tenant-${ID}"
        kubectl wait --for=condition=available --timeout=600s deployment/lsp-server-tenant-${ID} -n "rust-ai-ide-tenant-${ID}"
    fi

    log_success "Tenant ${tenant_key} deployed successfully"
}

# Function to setup tenant secrets
setup_tenant_secrets() {
    local tenant_key="$1"

    log_info "Setting up secrets for tenant: ${tenant_key}"

    # Create namespace if it doesn't exist
    kubectl create namespace "rust-ai-ide-tenant-${ID}" --dry-run=client -o yaml | kubectl apply -f -

    # Database secret
    if [[ -n "${DATABASE_DB_NAME}" && -n "${DATABASE_HOST}" ]]; then
        cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Secret
metadata:
  name: tenant-db-secret-${ID}
  namespace: rust-ai-ide-tenant-${ID}
type: Opaque
data:
  DATABASE_URL: $(echo "postgresql://user:password@${DATABASE_HOST}:5432/${DATABASE_DB_NAME}" | base64 -w 0)
EOF
        if [[ "${VERBOSE}" == true ]]; then
            log_info "Database secret configured for ${DATABASE_DB_NAME}"
        fi
    fi

    # SMTP secret (if enabled)
    if [[ "${SMTP_ENABLED}" == "true" ]]; then
        cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Secret
metadata:
  name: tenant-smtp-secret-${ID}
  namespace: rust-ai-ide-tenant-${ID}
type: Opaque
data:
  SMTP_CONFIG: $(echo "smtp://${SMTP_RELAY}:587?auth=plain" | base64 -w 0)
EOF
        if [[ "${VERBOSE}" == true ]]; then
            log_info "SMTP secret configured for ${SMTP_RELAY}"
        fi
    fi

    log_success "Secrets setup completed for tenant ${tenant_key}"
}

# Function to list tenants
list_tenants() {
    log_info "Listing all configured tenants"

    if ! command -v yq >/dev/null 2>&1; then
        log_error "yq is required for parsing YAML configuration"
        exit 1
    fi

    echo "Configured Tenants:"
    echo "===================="
    yq -r '.tenants | keys[]' "${TENANT_CONFIG}" | while read -r tenant; do
        local id name domain sla
        id=$(yq -r ".tenants.${tenant}.id" "${TENANT_CONFIG}")
        name=$(yq -r ".tenants.${tenant}.name" "${TENANT_CONFIG}")
        domain=$(yq -r ".tenants.${tenant}.domain" "${TENANT_CONFIG}")
        sla=$(yq -r ".tenants.${tenant}.sla_level" "${TENANT_CONFIG}")

        # Check if tenant is deployed
        local status="Not Deployed"
        if kubectl get namespace "rust-ai-ide-tenant-${id}" >/dev/null 2>&1; then
            local ready_pods
            ready_pods=$(kubectl get pods -n "rust-ai-ide-tenant-${id}" --no-headers | grep -c "Running" || echo "0")
            status="${ready_pods} pods ready"
        fi

        echo "  ${tenant}:"
        echo "    ID: ${id}"
        echo "    Name: ${name}"
        echo "    Domain: ${domain}"
        echo "    SLA: ${sla}"
        echo "    Status: ${status}"
        echo ""
    done
}

# Function to remove tenant
remove_tenant() {
    local tenant_key="$1"

    log_info "Removing tenant: ${tenant_key}"

    load_tenant_config "${tenant_key}"

    if [[ "${DRY_RUN}" == true ]]; then
        log_info "DRY RUN: Would remove tenant ${tenant_key} (namespace: rust-ai-ide-tenant-${ID})"
        return 0
    fi

    # Delete namespace (which removes all resources)
    kubectl delete namespace "rust-ai-ide-tenant-${ID}" --ignore-not-found=true

    # Wait for deletion to complete
    kubectl wait --for=delete namespace "rust-ai-ide-tenant-${ID}" --timeout=300s || log_warning "Namespace deletion may still be in progress"

    log_success "Tenant ${tenant_key} removed successfully"
}

# Function to show tenant status
show_tenant_status() {
    local tenant_key="$1"

    log_info "Showing status for tenant: ${tenant_key}"

    load_tenant_config "${tenant_key}"

    local namespace="rust-ai-ide-tenant-${ID}"

    if ! kubectl get namespace "${namespace}" >/dev/null 2>&1; then
        log_warning "Tenant ${tenant_key} is not deployed"
        return 1
    fi

    echo "Tenant Status: ${tenant_key} (${ID})"
    echo "========================================"
    echo "Namespace: ${namespace}"
    echo "Domain: ${DOMAIN}"
    echo ""
    echo "Resources:"
    kubectl get all -n "${namespace}" --no-headers | wc -l | xargs echo "Total resources: "
    echo ""
    echo "Pods:"
    kubectl get pods -n "${namespace}" -o wide
    echo ""
    echo "Services:"
    kubectl get services -n "${namespace}"
    echo ""
    echo "Ingress:"
    kubectl get ingress -n "${namespace}" -o wide
    echo ""
    echo "Resource Usage:"
    kubectl top pods -n "${namespace}" || log_warning "Resource metrics not available"
}

# Main function
main() {
    log_info "Starting tenant manager script"
    log_info "Command: ${COMMAND}"
    if [[ -n "${TENANT_NAME}" ]]; then
        log_info "Tenant: ${TENANT_NAME}"
    fi
    log_info "Log file: ${DEPLOY_LOG}"

    # Trap to ensure cleanup on exit
    trap 'log_info "Tenant manager completed (exit code: $?)"; rm -rf /tmp/tenant-*' EXIT

    local exit_code=0

    case "${COMMAND}" in
        list)
            list_tenants
            ;;
        deploy)
            deploy_tenant "${TENANT_NAME}"
            ;;
        remove)
            remove_tenant "${TENANT_NAME}"
            ;;
        status)
            show_tenant_status "${TENANT_NAME}"
            ;;
        secrets)
            load_tenant_config "${TENANT_NAME}"
            setup_tenant_secrets "${TENANT_NAME}"
            ;;
        update|backup|restore|monitoring)
            log_error "Command '${COMMAND}' not yet implemented"
            exit_code=1
            ;;
        *)
            log_error "Unknown command: ${COMMAND}"
            usage
            exit_code=1
            ;;
    esac

    local end_time=$(date +%s)
    log_info "Tenant manager completed in $((end_time - START_TIME)) seconds"

    return $exit_code
}

# Run main function
main "$@"
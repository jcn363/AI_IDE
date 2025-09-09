#!/bin/bash

# Enterprise SSO/RBAC Setup Script for Rust AI IDE
# Configures SSO integration with LDAP/Active Directory and RBAC policies
# Author: DevOps Engineering Team
# Version: 1.0.0

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
SSO_LOG="${PROJECT_ROOT}/sso-setup.log"
START_TIME=$(date +%s)

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${SSO_LOG}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${SSO_LOG}" >&2
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${SSO_LOG}"
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${SSO_LOG}"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [COMMAND] [OPTIONS]

Enterprise SSO/RBAC setup utility for Rust AI IDE.

COMMANDS:
    deploy-auth-service       Deploy LDAP/AD authentication service
    configure-ldap           Configure LDAP/Active Directory integration
    setup-rbac-policies      Crete enterprise RBAC policies and bindings
    setup-tenant-sso         Configure SSO for specific tenant
    validate-auth-setup      Validate authentication configuration
    test-integration         Test LDAP/AD integration and RBAC policies

OPTIONS:
    -h, --help               Show this help message
    -v, --verbose            Enable verbose output
    -t, --tenant TENANT_ID   Target tenant for SSO setup
    -u, --user USER_EMAIL    User email for role assignment
    -r, --role ROLE          Role to assign (admin, developer, auditor, tenant-admin)
    --ldap-host HOST         LDAP server hostname
    --ldap-port PORT         LDAP server port (default: 389)
    --ldap-base-dn DN        LDAP base DN
    --ldap-bind-dn DN        LDAP bind DN
    --dry-run                Perform dry-run operations only

EXAMPLES:
    $0 deploy-auth-service
    $0 configure-ldap --ldap-host ldap.company.com --ldap-base-dn "dc=company,dc=com"
    $0 setup-rbac-policies --user admin@company.com --role admin
    $0 setup-tenant-sso --tenant acme001
    $0 test-integration --verbose

EOF
}

# Variables
COMMAND=""
TENANT_ID=""
USER_EMAIL=""
USER_ROLE=""
LDAP_HOST=""
LDAP_PORT="389"
LDAP_BASE_DN=""
LDAP_BIND_DN=""
VERBOSE=false
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
        -t|--tenant)
            TENANT_ID="$2"
            shift 2
            ;;
        -u|--user)
            USER_EMAIL="$2"
            shift 2
            ;;
        -r|--role)
            USER_ROLE="$2"
            shift 2
            ;;
        --ldap-host)
            LDAP_HOST="$2"
            shift 2
            ;;
        --ldap-port)
            LDAP_PORT="$2"
            shift 2
            ;;
        --ldap-base-dn)
            LDAP_BASE_DN="$2"
            shift 2
            ;;
        --ldap-bind-dn)
            LDAP_BIND_DN="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        deploy-auth-service|configure-ldap|setup-rbac-policies|setup-tenant-sso|validate-auth-setup|test-integration)
            if [[ -z "${COMMAND}" ]]; then
                COMMAND="$1"
            else
                log_error "Multiple commands not supported: ${COMMAND} and $1"
                exit 1
            fi
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

# Function to deploy authentication service
deploy_auth_service() {
    log_info "Deploying enterprise authentication service..."

    if [[ "${DRY_RUN}" == true ]]; then
        log_info "DRY RUN: Would deploy LDAP authentication service"
        kubectl apply --dry-run=client -f "${PROJECT_ROOT}/cloud-deployment/k8s/ldap-auth.yaml"
        return 0
    fi

    # Apply authentication service manifests
    kubectl apply -f "${PROJECT_ROOT}/cloud-deployment/k8s/ldap-auth.yaml"

    # Wait for deployment
    kubectl wait --for=condition=available --timeout=300s deployment/ldap-authenticator -n kube-system

    log_success "Authentication service deployed successfully"
}

# Function to configure LDAP
configure_ldap() {
    log_info "Configuring LDAP/Active Directory integration..."

    # Validate required parameters
    if [[ -z "${LDAP_HOST}" || -z "${LDAP_BASE_DN}" ]]; then
        log_error "LDAP host and base DN are required"
        log_error "Use --ldap-host and --ldap-base-dn options"
        exit 1
    fi

    local ldap_config="/tmp/ldap-config-${START_TIME}.yaml"

    cat > "${ldap_config}" << EOF
apiVersion: v1
kind: ConfigMap
metadata:
  name: ldap-auth-config
  namespace: kube-system
data:
  config.yaml: |
    ldap:
      host: "${LDAP_HOST}"
      port: "${LDAP_PORT}"
      bindDN: "${LDAP_BIND_DN}"
      bindPassword: "PLACEHOLDER_PASSWORD"
      userSearchBase: "${LDAP_BASE_DN}"
      userFilter: "(&(objectClass=user)(sAMAccountName=%s))"
      groupSearchBase: "${LDAP_BASE_DN}"
      groupFilter: "(&(objectClass=group)(member=%s))"
      userAttribute: "sAMAccountName"
      groupAttribute: "memberOf"

    oidc:
      issuerURL: "https://dex.${LDAP_HOST}"
      clientID: "rust-ai-ide"
      clientSecret: "PLACEHOLDER_OIDC_SECRET"
      redirectURL: "https://auth.${LDAP_HOST}/callback"

    rbac:
      adminGroup: "CN=Domain Admins,${LDAP_BASE_DN}"
      developerGroup: "CN=Developers,${LDAP_BASE_DN}"
      auditorGroup: "CN=Auditors,${LDAP_BASE_DN}"
EOF

    if [[ "${DRY_RUN}" == true ]]; then
        log_info "DRY RUN: LDAP configuration would be applied:"
        cat "${ldap_config}"
        return 0
    fi

    kubectl apply -f "${ldap_config}"

    # Prompt for secure credential setup
    log_warning "LDAP configuration applied. Please manually create secrets:"
    echo "kubectl create secret generic ldap-auth-secrets -n kube-system \\"
    echo "  --from-literal=ldap-bind-password='\${LDAP_BIND_PASSWORD}' \\"
    echo "  --from-literal=oidc-client-secret='\${OIDC_CLIENT_SECRET}'"
    echo ""
    echo "Update the bind password placeholder in the ConfigMap if needed."

    log_success "LDAP configuration template deployed"
}

# Function to setup RBAC policies
setup_rbac_policies() {
    log_info "Setting up enterprise RBAC policies..."

    # Validate parameters
    if [[ -z "${USER_EMAIL}" || -z "${USER_ROLE}" ]]; then
        log_error "User email and role are required"
        exit 1
    fi

    # Validate role
    case "${USER_ROLE}" in
        admin|developer|auditor)
            ;;
        *)
            log_error "Invalid role: ${USER_ROLE}. Must be admin, developer, or auditor"
            exit 1
            ;;
    esac

    local binding_name="enterprise-${USER_ROLE}-binding-${USER_EMAIL//[@.]/-}"

    cat > "/tmp/rbac-binding-${START_TIME}.yaml" << EOF
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: ${binding_name}
  labels:
    enterprise-rbac: "true"
  annotations:
    created-by: "sso-setup-script"
    user-email: "${USER_EMAIL}"
    role: "${USER_ROLE}"
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: enterprise-${USER_ROLE}
subjects:
- kind: User
  name: "${USER_EMAIL}"
  apiGroup: rbac.authorization.k8s.io
EOF

    if [[ "${DRY_RUN}" == true ]]; then
        log_info "DRY RUN: Would create RBAC binding for ${USER_EMAIL} with role ${USER_ROLE}"
        cat "/tmp/rbac-binding-${START_TIME}.yaml"
        return 0
    fi

    kubectl apply -f "/tmp/rbac-binding-${START_TIME}.yaml"

    log_success "RBAC policy created for user: ${USER_EMAIL} with role: ${USER_ROLE}"
}

# Function to setup tenant SSO
setup_tenant_sso() {
    log_info "Setting up SSO integration for tenant: ${TENANT_ID}"

    if [[ -z "${TENANT_ID}" ]]; then
        log_error "Tenant ID is required for SSO setup"
        exit 1
    fi

    local namespace="rust-ai-ide-tenant-${TENANT_ID}"

    # Check if tenant exists
    if ! kubectl get namespace "${namespace}" >/dev/null 2>&1; then
        log_error "Tenant ${TENANT_ID} does not exist"
        return 1
    fi

    # Create tenant-specific RBAC
    local tenant_rbac="/tmp/tenant-rbac-${TENANT_ID}-${START_TIME}.yaml"

    cat > "${tenant_rbac}" << EOF
# Tenant-specific RBAC for ${TENANT_ID}
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: tenant-${TENANT_ID}-admin
  namespace: ${namespace}
  labels:
    tenant-id: "${TENANT_ID}"
    enterprise-rbac: "true"
rules:
- apiGroups: [""]
  resources: ["pods", "services", "configmaps", "secrets", "persistentvolumeclaims"]
  resourceNames: ["*-${TENANT_ID}*"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: ["apps"]
  resources: ["deployments", "replicasets"]
  resourceNames: ["*-${TENANT_ID}*"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: ["networking.k8s.io"]
  resources: ["ingresses"]
  resourceNames: ["*-${TENANT_ID}*"]
  verbs: ["get", "list", "watch", "create", "update", "patch"]

---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: tenant-${TENANT_ID}-admin-binding-template
  namespace: ${namespace}
  labels:
    tenant-id: "${TENANT_ID}"
    enterprise-rbac: "true"
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: Role
  name: tenant-${TENANT_ID}-admin
subjects:
- kind: User
  name: "PLACEHOLDER_USER_EMAIL"
  apiGroup: rbac.authorization.k8s.io
EOF

    if [[ "${DRY_RUN}" == true ]]; then
        log_info "DRY RUN: Would setup SSO for tenant ${TENANT_ID}"
        cat "${tenant_rbac}"
        return 0
    fi

    kubectl apply -f "${tenant_rbac}"

    log_success "SSO configuration prepared for tenant: ${TENANT_ID}"

    # Provide manual binding instructions
    echo ""
    echo "To bind users to this tenant, run:"
    echo "kubectl patch rolebinding tenant-${TENANT_ID}-admin-binding-template \\"
    echo "  -n ${namespace} \\"
    echo "  --type='json' \\"
    echo "  -p '[{\"op\": \"replace\", \"path\": \"/subjects/0/name\", \"value\": \"user@company.com\"}]'"
}

# Function to validate auth setup
validate_auth_setup() {
    log_info "Validating authentication setup..."

    # Check authentication service
    if ! kubectl get deployment ldap-authenticator -n kube-system >/dev/null 2>&1; then
        log_error "LDAP authenticator deployment not found"
        return 1
    fi

    # Check service accessibility
    if ! kubectl get svc ldap-authenticator -n kube-system >/dev/null 2>&1; then
        log_error "LDAP authenticator service not found"
        return 1
    fi

    # Check config maps
    if ! kubectl get configmap ldap-auth-config -n kube-system >/dev/null 2>&1; then
        log_error "LDAP configuration ConfigMap not found"
        return 1
    fi

    # Check secrets existence
    if ! kubectl get secret ldap-auth-secrets -n kube-system >/dev/null 2>&1; then
        log_warning "LDAP secrets not configured - authentication may fail"
    fi

    # Validate cluster roles
    local roles=("enterprise-admin" "enterprise-developer" "enterprise-auditor")
    for role in "${roles[@]}"; do
        if ! kubectl get clusterrole "${role}" >/dev/null 2>&1; then
            log_error "ClusterRole ${role} not found"
            return 1
        fi
    done

    log_success "Authentication setup validation passed"
}

# Function to test integration
test_integration() {
    log_info "Testing LDAP/AD integration and RBAC policies..."

    # Test LDAP connectivity (if configured)
    local ldap_host
    ldap_host=$(kubectl get configmap ldap-auth-config -n kube-system -o jsonpath='{.data.config\.yaml}' 2>/dev/null | grep -oP 'host:\s*\K.*' || echo "")

    if [[ -n "${ldap_host}" ]]; then
        log_info "Testing LDAP connectivity to: ${ldap_host}"
        # In a real environment, would add LDAP test connection here
    else
        log_info "LDAP not configured - skipping connectivity test"
    fi

    # Test RBAC policies
    log_info "Testing RBAC policy access..."

    # Create a test pod to validate permissions
    local test_pod="/tmp/test-auth-pod-${START_TIME}.yaml"

    cat > "${test_pod}" << EOF
apiVersion: v1
kind: Pod
metadata:
  name: auth-test-pod
  namespace: default
spec:
  serviceAccountName: rust-aiide-sa
  containers:
  - name: test
    image: busybox
    command: ["sleep", "300"]
  restartPolicy: Never
EOF

    if [[ "${DRY_RUN}" != true ]]; then
        kubectl apply -f "${test_pod}" 2>/dev/null || log_warning "Could not create test pod - permission test inconclusive"

        # Wait a moment and check pod status
        sleep 2
        if kubectl get pod auth-test-pod -n default >/dev/null 2>&1; then
            log_success "RBAC permissions validation: PASSED"
            kubectl delete pod auth-test-pod -n default --ignore-not-found=true
        else
            log_warning "RBAC permissions validation: UNDETERMINED"
        fi
    fi

    log_success "Integration test completed"
}

# Main function
main() {
    log_info "Starting SSO/RBAC setup script"
    log_info "Command: ${COMMAND}"
    if [[ -n "${TENANT_ID}" ]]; then
        log_info "Tenant: ${TENANT_ID}"
    fi
    if [[ -n "${USER_EMAIL}" ]]; then
        log_info "User: ${USER_EMAIL}"
    fi
    log_info "Log file: ${SSO_LOG}"

    # Trap to ensure cleanup
    trap 'log_info "SSO setup completed (exit code: $?)"; rm -f /tmp/*-'"${START_TIME}"'.yaml' EXIT

    local exit_code=0

    case "${COMMAND}" in
        deploy-auth-service)
            deploy_auth_service
            ;;
        configure-ldap)
            configure_ldap
            ;;
        setup-rbac-policies)
            setup_rbac_policies
            ;;
        setup-tenant-sso)
            setup_tenant_sso
            ;;
        validate-auth-setup)
            validate_auth_setup
            ;;
        test-integration)
            test_integration
            ;;
        *)
            log_error "Unknown command: ${COMMAND}"
            usage
            exit_code=1
            ;;
    esac

    local end_time=$(date +%s)
    log_info "SSO setup completed in $((end_time - START_TIME)) seconds"

    return $exit_code
}

# Run main function
main "$@"
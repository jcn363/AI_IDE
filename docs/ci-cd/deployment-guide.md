# Deployment Guide

## Overview

This guide provides comprehensive procedures for deploying the Rust AI IDE across different environments (development, staging, production) using our automated CI/CD pipelines.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Environment Setup](#environment-setup)
- [Deployment Strategies](#deployment-strategies)
- [Manual Deployment](#manual-deployment)
- [Automated Deployment](#automated-deployment)
- [Monitoring and Troubleshooting](#monitoring-and-troubleshooting)
- [Rollback Procedures](#rollback-procedures)

## Prerequisites

### Required Tools

- **Kubernetes** 1.24+
- **Helm** 3.8+
- **kubectl** configured with cluster access
- **Docker** 20.10+
- **Git** for repository access

### Required Credentials

- Docker Hub access token
- Kubernetes cluster certificates
- Cloud provider credentials (if applicable)

### Network Requirements

- HTTPS endpoints for all environments
- Load balancer configuration
- DNS configuration for ingress

## Environment Setup

### Development Environment

```bash
# Namespace creation
kubectl create namespace rust-ai-ide-dev

# RBAC setup
kubectl apply -f k8s/rbac/dev-rbac.yaml

# ConfigMaps and Secrets
kubectl apply -f k8s/config/dev-config.yaml
```

### Staging Environment

```bash
# Namespace creation
kubectl create namespace rust-ai-ide-staging

# RBAC setup
kubectl apply -f k8s/rbac/staging-rbac.yaml

# ConfigMaps and Secrets
kubectl apply -f k8s/config/staging-config.yaml

# Network policies
kubectl apply -f k8s/network-policies/staging-network-policies.yaml
```

### Production Environment

```bash
# Namespace creation
kubectl create namespace rust-ai-ide-prod

# RBAC setup
kubectl apply -f k8s/rbac/prod-rbac.yaml

# ConfigMaps and Secrets
kubectl apply -f k8s/config/prod-config.yaml

# Network policies
kubectl apply -f k8s/network-policies/prod-network-policies.yaml

# Resource quotas
kubectl apply -f k8s/resource-quotas/prod-quotas.yaml
```

## Deployment Strategies

### Blue-Green Deployment

The default strategy for production deployments, providing zero-downtime updates:

1. **Deploy to inactive environment** (green if blue is active)
2. **Run health checks** on new deployment
3. **Gradual traffic shifting** (if canary is enabled)
4. **Full traffic switch** after validation
5. **Monitor and cleanup** old deployment

#### Manual Blue-Green Deployment

```bash
# Using deployment helpers script
./scripts/ci/deployment-helpers.sh deploy-production \
    --strategy blue-green \
    --image-tag v1.2.3 \
    --verbose
```

### Canary Deployment

Gradual rollout with traffic splitting:

```bash
# Deploy 20% to canary
helm upgrade rust-ai-ide-canary ./helm/rust-ai-ide \
    --set canary.enabled=true \
    --set canary.weight=20 \
    --namespace rust-ai-ide-prod
```

### Rolling Update

Traditional rolling deployment:

```bash
helm upgrade rust-ai-ide ./helm/rust-ai-ide \
    --set deployment.strategy=rolling \
    --namespace rust-ai-ide-prod
```

## Manual Deployment

### Using Helm Charts

```bash
# Add Helm repository (if applicable)
helm repo add rust-ai-ide https://charts.rust-ai-ide.dev
helm repo update

# Install/upgrade with custom values
helm upgrade --install rust-ai-ide ./cloud-deployment/helm/rust-ai-ide \
    --values ./cloud-deployment/helm/rust-ai-ide/values-prod.yaml \
    --namespace rust-ai-ide-prod \
    --wait \
    --timeout 600s
```

### Using Deployment Scripts

```bash
# Prepare deployment
./scripts/ci/deployment-helpers.sh prepare-helm-charts \
    --environment production

# Validate deployment configuration
./scripts/ci/deployment-helpers.sh validate-deployment \
    --environment production \
    --image-tag v1.2.3

# Execute deployment
./scripts/ci/deployment-helpers.sh deploy-production \
    --image-tag v1.2.3 \
    --wait-timeout 1200
```

## Automated Deployment

### GitHub Actions Workflow

#### Manual Trigger

```yaml
# Trigger deployment from GitHub Actions
gh workflow run deployment.yml \
    -f environment=staging \
    -f image_tag=v1.2.3
```

#### Scheduled Deployments

```yaml
# Nightly staging deployments
on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM UTC
  workflow_dispatch:
```

#### Auto-deployment on Release

```yaml
# Deploy on release creation
on:
  release:
    types: [published]
```

### Deployment Pipeline Stages

1. **Build & Test**
   - Run CI pipeline
   - Generate artifacts
   - Security scanning

2. **Container Build**
   - Multi-stage Docker builds
   - Security scanning
   - Push to registry

3. **Infrastructure Provisioning**
   - Helm chart validation
   - Kubernetes resource creation
   - Configuration management

4. **Deployment Execution**
   - Blue-green deployment
   - Health checks
   - Traffic switching

5. **Post-deployment**
   - Monitoring setup
   - Performance validation
   - Documentation update

## Monitoring and Troubleshooting

### Health Checks

#### Automatic Health Checks

- **Readiness probes**: `/health/ready`
- **Liveness probes**: `/health/live`
- **Application metrics**: `/metrics`

#### Manual Health Verification

```bash
# Check pod health
kubectl get pods -n rust-ai-ide-prod

# Check service endpoints
kubectl get services -n rust-ai-ide-prod

# Test application endpoints
curl -f https://api.rust-ai-ide.com/health

# Check logs
kubectl logs -f deployment/rust-ai-ide-prod -n rust-ai-ide-prod
```

### Common Issues and Solutions

#### Deployment Failures

**Issue**: Pods stuck in `Pending` state
```bash
# Check resource constraints
kubectl describe pod <pod-name> -n rust-ai-ide-prod

# Check node capacity
kubectl get nodes --show-labels
```

**Issue**: Image pull failures
```bash
# Verify image exists
docker pull rust-ai-ide/ai-inference:v1.2.3

# Check registry credentials
kubectl get secrets -n rust-ai-ide-prod
```

#### Performance Issues

**Issue**: High CPU/memory usage
```bash
# Check resource usage
kubectl top pods -n rust-ai-ide-prod

# Adjust resource limits
helm upgrade rust-ai-ide ./helm/rust-ai-ide \
    --set resources.limits.cpu=2000m \
    --namespace rust-ai-ide-prod
```

**Issue**: Slow response times
```bash
# Check network policies
kubectl get networkpolicies -n rust-ai-ide-prod

# Verify service mesh configuration
kubectl get virtualservices -n rust-ai-ide-prod
```

### Logging and Debugging

#### Application Logs

```bash
# View pod logs
kubectl logs -f deployment/rust-ai-ide -n rust-ai-ide-prod

# Logs with timestamps
kubectl logs -f deployment/rust-ai-ide -n rust-ai-ide-prod --timestamps

# Previous container logs
kubectl logs -f deployment/rust-ai-ide -n rust-ai-ide-prod --previous
```

#### System Logs

```bash
# Kubernetes events
kubectl get events -n rust-ai-ide-prod --sort-by=.metadata.creationTimestamp

# Node logs (if accessible)
kubectl logs -f node/<node-name> -n kube-system
```

#### Debug Containers

```bash
# Ephemeral debug container
kubectl debug deployment/rust-ai-ide -n rust-ai-ide-prod --image=busybox

# Network debugging
kubectl run debug-pod --image=busybox -n rust-ai-ide-prod --rm -it -- sh
```

## Rollback Procedures

### Automatic Rollback

Automatic rollbacks are triggered when:
- Health checks fail for 5+ minutes
- Error rate exceeds 10%
- Performance degradation detected

```bash
# Trigger automatic rollback
./scripts/ci/deployment-helpers.sh rollback \
    --environment production \
    --verbose
```

### Manual Rollback

#### To Previous Version

```bash
# Rollback using Helm
helm rollback rust-ai-ide 1 -n rust-ai-ide-prod

# Or using deployment script
./scripts/ci/deployment-helpers.sh rollback \
    --environment production \
    --rollback-tag v1.1.0
```

#### Emergency Rollback

```bash
# Immediate rollback (no health checks)
kubectl patch service rust-ai-ide -n rust-ai-ide-prod \
    --type='json' \
    -p='[{"op": "replace", "path": "/spec/selector/color", "value": "blue"}]'
```

### Rollback Validation

```bash
# Verify rollback success
kubectl get pods -n rust-ai-ide-prod
kubectl get services -n rust-ai-ide-prod

# Test functionality
curl -f https://api.rust-ai-ide.com/health
```

## Security Considerations

### TLS Configuration

```yaml
# Ingress with TLS
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: rust-ai-ide-ingress
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  tls:
  - hosts:
    - api.rust-ai-ide.com
    secretName: rust-ai-ide-tls
  rules:
  - host: api.rust-ai-ide.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: rust-ai-ide
            port:
              number: 443
```

### Network Policies

```yaml
# Restrict traffic
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: rust-ai-ide-network-policy
spec:
  podSelector:
    matchLabels:
      app: rust-ai-ide
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          trusted: "true"
    ports:
    - protocol: TCP
      port: 8080
```

## Performance Optimization

### Resource Optimization

```yaml
# Optimized resource allocation
resources:
  limits:
    cpu: 2000m
    memory: 4Gi
  requests:
    cpu: 1000m
    memory: 2Gi
```

### Horizontal Pod Autoscaling

```yaml
# CPU-based scaling
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: rust-ai-ide-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: rust-ai-ide
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

## Operational Best Practices

### Backup and Recovery

```bash
# Database backups (if applicable)
kubectl exec -it deployment/postgres -- pg_dump > backup.sql

# Configuration backups
kubectl get configmaps -n rust-ai-ide-prod -o yaml > config-backup.yaml

# Secrets backup (with caution)
kubectl get secrets -n rust-ai-ide-prod -o yaml > secrets-backup.yaml
```

### Maintenance Windows

```bash
# Schedule maintenance
kubectl annotate deployment rust-ai-ide \
    maintenance-scheduled="true" \
    -n rust-ai-ide-prod

# Drain nodes for maintenance
kubectl drain <node-name> --ignore-daemonsets --delete-local-data
```

### Cost Optimization

```bash
# Scale down during low-traffic periods
kubectl scale deployment rust-ai-ide --replicas=1 -n rust-ai-ide-prod

# Use spot instances for non-critical workloads
# Configure in Helm values.yaml
nodeSelector:
  node-type: spot
```

## Support and Escalation

### Monitoring Dashboards

- **Grafana**: `https://grafana.rust-ai-ide.com`
- **Kibana**: `https://kibana.rust-ai-ide.com`
- **Prometheus**: `https://prometheus.rust-ai-ide.com`

### Alert Channels

- **Critical Issues**: #alerts-critical Slack channel
- **General Support**: #devops-support Slack channel
- **On-call Rotation**: PagerDuty integration

### Contact Information

- **DevOps Team**: devops@rust-ai-ide.com
- **Security Team**: security@rust-ai-ide.com
- **Emergency**: +1-555-0123 (24/7)

---

## Quick Reference Commands

```bash
# Status checks
kubectl get all -n rust-ai-ide-prod
kubectl top pods -n rust-ai-ide-prod

# Logs
kubectl logs -f deployment/rust-ai-ide -n rust-ai-ide-prod

# Scaling
kubectl scale deployment rust-ai-ide --replicas=5 -n rust-ai-ide-prod

# Rolling restart
kubectl rollout restart deployment/rust-ai-ide -n rust-ai-ide-prod

# Port forwarding for debugging
kubectl port-forward svc/rust-ai-ide 8080:8080 -n rust-ai-ide-prod
```

*Last updated: $(date)*
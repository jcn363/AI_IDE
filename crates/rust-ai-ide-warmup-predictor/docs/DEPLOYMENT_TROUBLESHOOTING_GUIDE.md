# Model Warmup Prediction System - Deployment Troubleshooting Guide

This guide addresses deployment-specific issues for the Model Warmup Prediction System, covering containerization, orchestration, environment setup, scaling, and production deployment challenges across all supported platforms.

## Deployment Architecture

### Supported Platforms

- **Docker**: Containerized deployment
- **Kubernetes**: Orchestrated deployment
- **AWS ECS/Fargate**: Cloud-native deployment
- **Azure Container Instances**: Cloud deployment
- **Google Cloud Run**: Serverless deployment
- **Bare Metal**: Traditional server deployment

### Deployment Components

```yaml
# Complete deployment stack
deployment:
  application:
    - warmup-predictor-service
    - lsp-service
    - multi-model-orchestrator
  infrastructure:
    - postgresql/redis (external)
    - object storage (models/data)
    - monitoring stack
  networking:
    - load balancers
    - service mesh
    - ingress controllers
```

## Container Deployment Issues

### Docker Build Failures

#### Issue: Build Context Too Large

**Symptoms:**
- Docker build fails with "context too large"
- Build times >30 minutes
- Out of disk space during build

**Resolution:**
```dockerfile
# Optimize Dockerfile for smaller context
FROM rust:1.75-slim as builder

# Use .dockerignore to exclude unnecessary files
# .dockerignore contents:
# target/
# .git/
# docs/
# *.md
# docker/

# Multi-stage build to reduce final image size
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/warmup-predictor /usr/local/bin/
```

#### Issue: Rust Compilation Failures in Docker

**Symptoms:**
- "cargo build" fails in container
- Missing dependencies or toolchains
- Compilation errors not seen locally

**Resolution:**
```dockerfile
FROM rustlang/rust:nightly-bullseye-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    sqlite3 \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates sqlite3 && rm -rf /var/lib/apt/lists/*
COPY --from=0 /app/target/release/warmup-predictor /usr/local/bin/
```

### Container Runtime Issues

#### Issue: Container Startup Failures

**Symptoms:**
- Container exits immediately after start
- "CrashLoopBackOff" in Kubernetes
- Health check failures

**Resolution:**
```yaml
# Kubernetes deployment with proper health checks
apiVersion: apps/v1
kind: Deployment
metadata:
  name: warmup-predictor
spec:
  template:
    spec:
      containers:
      - name: warmup-predictor
        image: warmup-predictor:latest
        ports:
        - containerPort: 8080
        # Startup probe for slow-starting applications
        startupProbe:
          httpGet:
            path: /health/startup
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 30
        # Liveness probe
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8080
          initialDelaySeconds: 60
          periodSeconds: 30
        # Readiness probe
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
```

#### Issue: Resource Constraints in Containers

**Symptoms:**
- OOMKilled by Kubernetes
- CPU throttling
- Performance degradation

**Resolution:**
```yaml
# Proper resource limits
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: warmup-predictor
    resources:
      requests:
        memory: "1Gi"
        cpu: "500m"
      limits:
        memory: "2Gi"
        cpu: "1000m"
    env:
    - name: RUST_MIN_STACK
      value: "8388608"  # 8MB minimum stack
```

## Orchestration Issues

### Kubernetes Deployment Problems

#### Issue: Pod Scheduling Failures

**Symptoms:**
- Pods stuck in "Pending" state
- "Insufficient cpu/memory" errors
- Node affinity issues

**Resolution:**
```yaml
# Node selector and affinity rules
apiVersion: apps/v1
kind: Deployment
metadata:
  name: warmup-predictor
spec:
  template:
    spec:
      nodeSelector:
        accelerator: nvidia-tesla-k80  # For GPU workloads
      affinity:
        nodeAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
            nodeSelectorTerms:
            - matchExpressions:
              - key: kubernetes.io/os
                operator: In
                values:
                - linux
      tolerations:
      - key: "dedicated"
        operator: "Equal"
        value: "gpu"
        effect: "NoSchedule"
```

#### Issue: Service Discovery Failures

**Symptoms:**
- Inter-service communication failures
- DNS resolution errors
- Connection timeouts between services

**Resolution:**
```yaml
# Service configuration with proper selectors
apiVersion: v1
kind: Service
metadata:
  name: warmup-predictor-service
spec:
  selector:
    app: warmup-predictor
    version: v1.2.0  # Match deployment labels
  ports:
  - name: http
    port: 80
    targetPort: 8080
  type: ClusterIP

# Network policies for security
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: warmup-predictor-network-policy
spec:
  podSelector:
    matchLabels:
      app: warmup-predictor
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: frontend
    ports:
    - protocol: TCP
      port: 8080
```

### Service Mesh Issues

#### Issue: Istio Sidecar Injection Problems

**Symptoms:**
- Sidecar containers not injected
- Traffic routing failures
- mTLS certificate issues

**Resolution:**
```yaml
# Enable Istio injection
apiVersion: apps/v1
kind: Deployment
metadata:
  name: warmup-predictor
  annotations:
    kubectl.kubernetes.io/default-container: warmup-predictor
    kubectl.kubernetes.io/default-logs-container: warmup-predictor
spec:
  template:
    metadata:
      annotations:
        kubectl.kubernetes.io/default-container: warmup-predictor
        kubectl.kubernetes.io/default-logs-container: warmup-predictor
        sidecar.istio.io/status: '{"version":"abc123","initContainers":["istio-init"],"containers":["istio-proxy"],"volumes":["istiod-ca-cert","istio-data","istio-podinfo","istio-token","istiod-ca-cert"]}'
      labels:
        security.istio.io/tlsMode: istio
        service.istio.io/canonical-name: warmup-predictor
        service.istio.io/canonical-revision: latest
    spec:
      containers:
      - name: warmup-predictor
        image: warmup-predictor:latest
```

## Cloud Platform Issues

### AWS ECS Deployment

#### Issue: Task Definition Problems

**Symptoms:**
- Tasks fail to start
- Resource allocation errors
- Container health check failures

**Resolution:**
```json
{
  "family": "warmup-predictor",
  "taskRoleArn": "arn:aws:iam::123456789012:role/ecsTaskExecutionRole",
  "executionRoleArn": "arn:aws:iam::123456789012:role/ecsTaskExecutionRole",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "1024",
  "memory": "2048",
  "containerDefinitions": [
    {
      "name": "warmup-predictor",
      "image": "123456789012.dkr.ecr.us-east-1.amazonaws.com/warmup-predictor:latest",
      "essential": true,
      "portMappings": [
        {
          "containerPort": 8080,
          "hostPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {"name": "ENVIRONMENT", "value": "production"},
        {"name": "AWS_REGION", "value": "us-east-1"}
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/warmup-predictor",
          "awslogs-region": "us-east-1",
          "awslogs-stream-prefix": "ecs"
        }
      },
      "healthCheck": {
        "command": ["CMD-SHELL", "curl -f http://localhost:8080/health || exit 1"],
        "interval": 30,
        "timeout": 5,
        "retries": 3,
        "startPeriod": 60
      }
    }
  ]
}
```

### Azure Container Instances

#### Issue: Container Group Deployment Failures

**Symptoms:**
- Container groups fail to provision
- Resource quota exceeded
- Network configuration errors

**Resolution:**
```bash
# Azure CLI deployment with proper networking
az container create \
  --resource-group myResourceGroup \
  --name warmup-predictor \
  --image myregistry.azurecr.io/warmup-predictor:latest \
  --cpu 1 \
  --memory 2 \
  --registry-login-server myregistry.azurecr.io \
  --registry-username $ACR_USERNAME \
  --registry-password $ACR_PASSWORD \
  --environment-variables ENVIRONMENT=production \
  --ports 80 \
  --protocol TCP \
  --ip-address public \
  --dns-name-label warmup-predictor \
  --os-type Linux
```

## Scaling Issues

### Horizontal Scaling Problems

#### Issue: Auto-scaling Failures

**Symptoms:**
- HPA not triggering scaling
- Scaling events not working
- Resource thresholds not met

**Resolution:**
```yaml
# HorizontalPodAutoscaler configuration
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: warmup-predictor-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: warmup-predictor
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  - type: Pods
    pods:
      metric:
        name: requests_per_second
      target:
        type: AverageValue
        averageValue: 1000m
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
      - type: Pods
        value: 2
        periodSeconds: 60
      selectPolicy: Max
```

### Load Balancing Issues

#### Issue: Uneven Load Distribution

**Symptoms:**
- Some instances overloaded
- Others underutilized
- Session affinity problems

**Resolution:**
```yaml
# Load balancer configuration
apiVersion: v1
kind: Service
metadata:
  name: warmup-predictor-lb
  annotations:
    service.beta.kubernetes.io/aws-load-balancer-type: nlb
    service.beta.kubernetes.io/aws-load-balancer-cross-zone-load-balancing-enabled: "true"
    service.beta.kubernetes.io/aws-load-balancer-healthcheck-path: "/health"
spec:
  type: LoadBalancer
  selector:
    app: warmup-predictor
  ports:
  - name: http
    port: 80
    targetPort: 8080
  sessionAffinity: None  # Disable session affinity for better load distribution
```

## Environment-Specific Issues

### Development Environment

#### Issue: Local Development Setup Problems

**Symptoms:**
- Port conflicts
- Database connection issues
- File permission problems

**Resolution:**
```bash
# Development docker-compose setup
version: '3.8'
services:
  warmup-predictor:
    build: .
    ports:
      - "8080:8080"
    environment:
      - ENVIRONMENT=development
      - DATABASE_URL=postgres://user:password@db:5432/warmup_dev
    depends_on:
      - db
    volumes:
      - .:/app
      - /app/target
    command: cargo run --bin warmup-predictor

  db:
    image: postgres:13
    environment:
      POSTGRES_DB: warmup_dev
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

### Production Environment

#### Issue: Production Configuration Issues

**Symptoms:**
- Security settings too permissive
- Performance not optimized
- Monitoring not configured

**Resolution:**
```yaml
# Production deployment configuration
apiVersion: apps/v1
kind: Deployment
metadata:
  name: warmup-predictor-prod
spec:
  replicas: 3
  template:
    spec:
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 2000
      containers:
      - name: warmup-predictor
        image: registry.example.com/warmup-predictor:v1.2.0
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          runAsNonRoot: true
          capabilities:
            drop:
            - ALL
        env:
        - name: ENVIRONMENT
          value: "production"
        - name: RUST_LOG
          value: "info"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-secret
              key: url
        resources:
          requests:
            cpu: 500m
            memory: 1Gi
          limits:
            cpu: 1000m
            memory: 2Gi
```

## Monitoring and Observability

### Deployment Monitoring

```yaml
# Prometheus monitoring
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: warmup-predictor-monitor
spec:
  selector:
    matchLabels:
      app: warmup-predictor
  endpoints:
  - port: metrics
    path: /metrics
    interval: 30s
```

### Log Aggregation

```yaml
# Fluentd log collection
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: fluentd
spec:
  template:
    spec:
      containers:
      - name: fluentd
        image: fluent/fluentd-kubernetes-daemonset:v1.14-debian-elasticsearch7-1
        env:
        - name: FLUENT_UID
          value: "0"
        volumeMounts:
        - name: varlogcontainers
          mountPath: /var/log/containers
        - name: varlogpods
          mountPath: /var/log/pods
        - name: varlibdockercontainers
          mountPath: /var/lib/docker/containers
      volumes:
      - name: varlogcontainers
        hostPath:
          path: /var/log/containers
      - name: varlogpods
        hostPath:
          path: /var/log/pods
      - name: varlibdockercontainers
        hostPath:
          path: /var/lib/docker/containers
```

## Rollback Procedures

### Emergency Rollback

```bash
# Kubernetes rollback
kubectl rollout undo deployment/warmup-predictor

# Docker rollback
docker tag myapp:v1 myapp:rollback
docker push myapp:rollback

# Blue-green rollback
kubectl patch service warmup-predictor -p '{"spec":{"selector":{"version":"v1.1.0"}}}'
```

### Gradual Rollback

```yaml
# Canaries deployment for safe rollbacks
apiVersion: flagger.app/v1beta1
kind: Canary
metadata:
  name: warmup-predictor
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: warmup-predictor
  progressDeadlineSeconds: 60
  service:
    port: 80
    targetPort: 8080
  analysis:
    interval: 30s
    threshold: 5
    maxWeight: 50
    stepWeight: 10
    metrics:
    - name: request-success-rate
      thresholdRange:
        min: 99
      interval: 1m
    - name: request-duration
      thresholdRange:
        max: 500
      interval: 1m
```

This deployment troubleshooting guide provides comprehensive procedures for diagnosing and resolving deployment issues across various platforms and environments. Always test deployments in staging environments before production rollout.
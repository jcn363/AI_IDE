# Enterprise SQL LSP API Reference

## Overview

This document provides comprehensive API reference for the Enterprise SQL LSP Server, including REST/GraphQL endpoints, monitoring APIs, security configurations, and client integration guidelines.

## Health Check Endpoints

### GET /health
- **Description**: Comprehensive system health check endpoint
- **Port**: 9090
- **Authentication**: None (internal monitoring)
- **Response Format**:
```json
{
  "status": "Healthy|Degraded|Unhealthy|Critical",
  "components": {
    "cache": "Healthy",
    "memory": "Healthy",
    "security": "Healthy",
    "compliance": "Healthy",
    "scaling": "Healthy"
  },
  "uptime_seconds": 3600,
  "compliance_status": true,
  "security_incidents": 0,
  "active_instances": 3,
  "metrics": {
    "cache_hit_rate": 87.5,
    "memory_usage_percent": 72.4,
    "total_queries": 12547,
    "error_rate": 0.001,
    "current_load_factor": 0.85
  }
}
```

### GET /ready
- **Description**: Readiness probe for load balancer
- **Port**: 9090
- **Response**: `200 OK` when service is ready, `503 Service Unavailable` otherwise

### GET /startup
- **Description**: Startup probe for Kubernetes
- **Port**: 9090
- **Response**: `200 OK` when service has fully started

### GET /metrics
- **Description**: Prometheus-style metrics endpoint
- **Port**: 9090
- **Authentication**: Required (Bearer token)
- **Content-Type**: `text/plain; version=0.0.4; charset=utf-8`

### POST /shutdown
- **Description**: Graceful shutdown endpoint (admin only)
- **Port**: 9090
- **Authentication**: Required (Admin JWT token)
- **Method**: POST to initiate graceful shutdown

## Enterprise API Endpoints

### POST /api/v1/query/analyze
- **Description**: Process and analyze SQL queries with enterprise monitoring
- **Port**: 8443 (HTTPS)
- **Authentication**: Required (JWT/MFA)
- **Request Body**:
```json
{
  "query": "SELECT * FROM users WHERE id = $1",
  "dialect": "postgresql",
  "context": {
    "user_id": "user123",
    "session_id": "sess456",
    "client_ip": "192.168.1.100"
  },
  "options": {
    "performance_analysis": true,
    "security_validation": true,
    "optimization_suggestions": true
  }
}
```
- **Response**:
```json
{
  "query_id": "q_123456",
  "syntax_valid": true,
  "performance_metrics": {
    "execution_time_us": 125000,
    "memory_usage_bytes": 2048000,
    "complexity_score": 8,
    "bottleneck": "io"
  },
  "security_warnings": [],
  "optimizations": [
    {
      "type": "index_suggestion",
      "description": "Consider adding index on user_id column",
      "confidence": 0.95,
      "impact": "Major"
    }
  ],
  "compliance_checks": {
    "gdpr_compliant": true,
    "data_encryption": "AES-256",
    "audit_logged": true
  },
  "processing_time_ms": 45
}
```

### GET /api/v1/monitoring/dashboard
- **Description**: Enterprise monitoring dashboard data
- **Port**: 8443 (HTTPS)
- **Authentication**: Required (Admin role)
- **Response**:
```json
{
  "cache_performance": {
    "metrics_hit_rate": 87.5,
    "schema_hit_rate": 92.1,
    "optimization_hit_rate": 83.7,
    "error_cache_hit_rate": 99.2,
    "overall_hit_rate": 89.8,
    "target_achievement": true
  },
  "memory_usage": {
    "current_usage_percent": 72.4,
    "trend": "Stable",
    "pressure_level": "Low",
    "high_water_mark_bytes": 2147483648,
    "leak_indicators": []
  },
  "performance_scores": {
    "memory_efficiency": 85.2,
    "cache_efficiency": 91.5,
    "throughput_efficiency": 94.7,
    "overall_score": 90.5
  },
  "security_events": [
    {
      "timestamp": "2025-09-10T19:45:00Z",
      "event_type": "QueryValidation",
      "severity": "Low",
      "description": "500 queries validated successfully",
      "incidents_today": 0
    }
  ],
  "compliance_metrics": {
    "soc2_compliance_score": 98.5,
    "gdpr_compliance_score": 96.7,
    "last_audit_date": "2025-09-01",
    "open_incidents": 0,
    "audit_logs_30_days": 12547
  },
  "system_load": {
    "cpu_usage_percent": 45.2,
    "memory_usage_percent": 72.4,
    "active_connections": 1247,
    "queued_requests": 15,
    "error_rate_per_sec": 0.002
  }
}
```

### GET /api/v1/security/audit-logs
- **Description**: Retrieve security audit logs
- **Port**: 8443 (HTTPS)
- **Authentication**: Required (Security Admin role)
- **Parameters**:
  - `start_date`: ISO 8601 date string
  - `end_date`: ISO 8601 date string
  - `event_type`: Filter by event type
  - `severity`: Filter by severity level
  - `user_id`: Filter by user
- **Response**:
```json
{
  "total_records": 12547,
  "records": [
    {
      "id": "audit_123456",
      "timestamp": "2025-09-10T19:45:00Z",
      "event_type": "SQL_INJECTION_ATTEMPT",
      "user_id": "user789",
      "client_ip": "192.168.1.100",
      "query_hash": "abc123...",
      "severity": "HIGH",
      "details": {
        "attack_vector": "UNION SELECT",
        "confidence": 0.95,
        "response": "QUERY_BLOCKED"
      }
    }
  ]
}
```

### POST /api/v1/scaling/instances
- **Description**: Scale instances up/down (cluster admin)
- **Port**: 8443 (HTTPS)
- **Authentication**: Required (Cluster Admin role)
- **Request Body**:
```json
{
  "action": "scale_up",
  "instances": 5,
  "reason": "Traffic spike detected"
}
```
- **Response**:
```json
{
  "result": "initiated",
  "target_instances": 5,
  "current_instances": 3,
  "estimated_completion_seconds": 120,
  "job_id": "scale_456789"
}
```

### GET /api/v1/compliance/report
- **Description**: Generate compliance reports
- **Port**: 8443 (HTTPS)
- **Authentication**: Required (Compliance role)
- **Parameters**:
  - `framework`: SOC2, GDPR, or BOTH
  - `period`: 30, 90, or 365 days
  - `format`: pdf, json, or xml
- **Response**: Compliance report in requested format

### POST /api/v1/backup/create
- **Description**: Initiate system backup
- **Port**: 8443 (HTTPS)
- **Authentication**: Required (Backup Admin role)
- **Request Body**:
```json
{
  "backup_type": "full_system",
  "include_compliance_logs": true,
  "encryption_key_id": "bkup_key_001"
}
```
- **Response**:
```json
{
  "backup_id": "backup_20250910_194500",
  "status": "initiated",
  "estimated_size_gb": 15.7,
  "estimated_duration_minutes": 45
}
```

## GraphQL API

### Endpoint: /graphql
- **Port**: 8443 (HTTPS)
- **Authentication**: Required (JWT/MFA)
- **Content-Type**: `application/json`

### Schema Highlights

```graphql
type Query {
  healthStatus: HealthStatus!
  systemMetrics: SystemMetrics!
  securityEvents(
    filter: SecurityEventFilter
    limit: Int
    offset: Int
  ): SecurityEventConnection!
  complianceReport(framework: ComplianceFramework!): ComplianceReport!
  auditLogs(timeRange: TimeRange!): AuditLogConnection!
  performanceDashboard: PerformanceDashboard!
  cacheMetrics(layer: CacheLayer): CacheMetrics!
  scalingStatus: ScalingStatus!
}

type Mutation {
  executeQuery(
    input: QueryExecutionInput!
  ): QueryExecutionResult!

  scaleInstances(
    input: ScalingInput!
  ): ScalingResult!

  performSecurityScan(
    input: SecurityScanInput!
  ): SecurityScanResult!

  generateComplianceReport(
    input: ComplianceReportInput!
  ): ComplianceReport!
}

type Subscription {
  cacheHitRateUpdates: CacheHitRateUpdate!
  securityEventStream: SecurityEvent!
  systemLoadUpdates: SystemLoadUpdate!
  complianceViolationAlerts: ComplianceViolationAlert!
}
```

### Example GraphQL Query

```graphql
query SystemHealthCheck {
  healthStatus {
    status
    components {
      cache
      memory
      security
      compliance
      scaling
    }
    uptimeSeconds
    metrics {
      cacheHitRate
      memoryUsagePercent
    }
  }

  securityEvents(filter: { severity: HIGH }, limit: 10) {
    edges {
      node {
        timestamp
        eventType
        severity
        description
      }
    }
  }

  performanceDashboard {
    cachePerformance {
      overallHitRate
      targetAchievement
    }
    memoryUsage {
      currentUsagePercent
      trend
      pressureLevel
    }
  }
}
```

## Authentication & Security

### JWT Authentication
- **Token Format**: `Bearer <jwt_token>`
- **Claims Required**:
  - `sub`: User ID
  - `exp`: Expiration timestamp
  - `roles`: Array of user roles
  - `client_id`: Client identifier
  - `mfa_complete`: Multi-factor auth completion flag

### Multi-Factor Authentication (MFA)
- **Supported Methods**: TOTP (Time-based OTP), SMS, Hardware Security Keys
- **MFA Endpoint**: `POST /api/v1/auth/mfa/verify`
- **MFA Flow**:
  1. Initial login with username/password
  2. Server responds with MFA challenge
  3. Client completes MFA verification
  4. Server issues JWT token

### Role-Based Access Control (RBAC)

#### Available Roles
- **USER**: Basic query analysis and optimization
- **ADMIN**: System management and monitoring
- **SECURITY_ADMIN**: Security controls and incident response
- **COMPLIANCE_OFFICER**: Compliance reporting and audits
- **CLUSTER_ADMIN**: Horizontal scaling and cluster management
- **BACKUP_ADMIN**: Backup and recovery operations

#### Permission Matrix
| Endpoint | USER | ADMIN | SECURITY_ADMIN | COMPLIANCE_OFFICER | CLUSTER_ADMIN | BACKUP_ADMIN |
|----------|------|-------|----------------|-------------------|---------------|-------------|
| `/api/v1/query/analyze` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `/api/v1/monitoring/dashboard` | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ |
| `/api/v1/security/audit-logs` | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ |
| `/api/v1/scaling/instances` | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ |
| `/api/v1/compliance/report` | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ |
| `/api/v1/backup/create` | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ |

### TLS 1.3 Configuration

- **Minimum Version**: TLS 1.3 enforced
- **Cipher Suites**:
  - TLS_AES_256_GCM_SHA384 (mandatory)
  - TLS_AES_128_GCM_SHA256
  - TLS_CHACHA20_POLY1305_SHA256
- **Certificate Rotation**: Automatic renewal via Let's Encrypt
- **HSTS**: Enabled with 31536000 seconds (1 year)
- **Server Name Indication (SNI)**: Required for multi-tenant deployments

## Client Integration Examples

### JavaScript/TypeScript Client

```typescript
import { EnterpriseSqlLspClient } from '@rust-ai-ide/enterprise-lsp-client';

const client = new EnterpriseSqlLspClient({
  endpoint: 'https://lsp.enterprise.company.com',
  apiKey: 'your-jwt-token'
});

// Analyze a SQL query
const result = await client.analyzeQuery({
  query: 'SELECT * FROM users WHERE active = ?',
  dialect: 'postgresql',
  options: {
    performanceAnalysis: true,
    optimizationSuggestions: true
  }
});

console.log('Query healthy:', result.syntaxValid);
console.log('Performance score:', result.performanceMetrics.complexityScore);

// Monitor system health
const health = await client.getHealthStatus();
console.log('System status:', health.status);
console.log('Cache hit rate:', health.metrics.cacheHitRate);
```

### Python Client

```python
from rustai_enterprise_lsp import EnterpriseClient
import asyncio

async def main():
    client = EnterpriseClient(
        endpoint="https://lsp.enterprise.company.com",
        auth_token="your-jwt-token"
    )

    # Execute and analyze query
    result = await client.analyze_query(
        query="SELECT u.*, p.* FROM users u LEFT JOIN profiles p ON u.id = p.user_id",
        dialect="postgresql",
        context={
            "user_id": "user123",
            "session_id": "sess456"
        }
    )

    print(f"Query complexity: {result.performance_metrics.complexity_score}")
    print(f"Optimizations found: {len(result.optimizations)}")

    # Get monitoring dashboard (admin only)
    dashboard = await client.get_monitoring_dashboard()
    print(f"Overall performance score: {dashboard.performance_scores.overall_score}")

if __name__ == "__main__":
    asyncio.run(main())
```

### curl Examples

```bash
# Health check
curl -i https://lsp.enterprise.company.com/health

# Analyze query with authentication
curl -X POST https://lsp.enterprise.company.com/api/v1/query/analyze \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "SELECT * FROM users WHERE status = $1",
    "dialect": "postgresql",
    "options": {
      "performance_analysis": true,
      "security_validation": true
    }
  }'

# Get monitoring dashboard (admin role required)
curl https://lsp.enterprise.company.com/api/v1/monitoring/dashboard \
  -H "Authorization: Bearer $ADMIN_JWT_TOKEN"

# Scale instances (cluster admin required)
curl -X POST https://lsp.enterprise.company.com/api/v1/scaling/instances \
  -H "Authorization: Bearer $CLUSTER_ADMIN_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"action":"scale_up","instances":5}'
```

## Configuration Files

### Enterprise Configuration (TOML)

```toml
title = "Enterprise SQL LSP Configuration"

[server]
port = 8080
https_port = 8443
metrics_port = 9090
log_level = "info"

[performance]
max_concurrent_tasks = 100
batch_size = 50
enable_parallel_processing = true
enable_horizontal_scaling = true

[cache]
max_memory_per_layer_mb = 2048
cache_hit_rate_target = 0.85
enable_distributed_cache = true
redis_endpoints = ["redis-cluster:6379"]

[security]
enable_security_hardening = true
enable_mfa = false
tls_version = "1.3"
audit_log_retention_days = 365
enable_rate_limiting = true
max_concurrent_requests = 1000

[compliance]
soc2_enabled = true
gdpr_enabled = true
audit_trail_enabled = true
data_encryption_enabled = true
never_log_pii = true

[monitoring]
monitoring_enabled = true
memory_profiling_interval_seconds = 30
performance_benchmark_interval_days = 90
alert_notification_channels = ["email", "slack", "pagerduty"]
log_aggregation_enabled = true

[scaling]
min_instances = 1
max_instances = 10
scaling_policy = "cpu_and_memory"
session_stickiness_ttl_minutes = 30
health_check_interval_seconds = 30

[backups]
enabled = true
interval_hours = 6
retention_days = 30
compression_level = 9
encryption_enabled = true

[external_integrations]
elasticsearch_endpoint = "http://elasticsearch:9200"
logstash_endpoint = "http://logstash:5400"
redis_cluster_endpoints = "redis-cluster:6379"
```

### Kubernetes ConfigMap Patch

```yaml
apiVersion: 3.3.0
kind: ConfigMap
metadata:
  name: enterprise-lsp-config
  namespace: sql-lsp-enterprise
data:
  enterprise-config.toml: |
    # Enterprise configuration content as above
```

## Error Codes and Handling

| Error Code | Description | Suggested Action |
|------------|-------------|------------------|
| `SQL_INJECTION_DETECTED` | Potential SQL injection attack | Sanitize user input and use parameterized queries |
| `INVALID_SYNTAX` | Query syntax is invalid | Correct query syntax using LSP suggestions |
| `RATE_LIMIT_EXCEEDED` | Too many requests in short time | Implement exponential backoff |
| `COMPLIANCE_VIOLATION` | Query violates compliance rules | Review data handling policies |
| `RESOURCE_EXHAUSTED` | System resources depleted | Reduce query complexity or add resources |
| `AUTHORIZATION_FAILED` | Insufficient permissions | Check user roles and permissions |
| `MFA_REQUIRED` | Multi-factor authentication needed | Complete MFA verification |

## Deployment Best Practices

### Environment Variables

```bash
# Production environment variables
export RUST_LSP_LOG_LEVEL=info
export RUST_LSP_CONFIG_FILE=/app/config/enterprise-config.toml
export RUST_LSP_DATABASE_URL=postgresql://user:password@postgres-cluster:5432/sql_lsp_prod
export RUST_LSP_REDIS_URL=redis://redis-cluster:6379
export RUST_LSP_JWT_SECRET_KEY='your-256-bit-secret-key'
export RUST_LSP_AUDIT_ENCRYPTION_KEY='your-audit-encryption-key'

# Performance tuning
export RUST_LSP_MAX_CONCURRENT_TASKS=100
export RUST_LSP_MEMORY_LIMIT_MB=4096
export RUST_LSP_CACHE_HIT_RATE_TARGET=0.85

# Security settings
export RUST_LSP_TLS_CERT_PATH=/app/ssl/cert.pem
export RUST_LSP_TLS_KEY_PATH=/app/ssl/key.pem
export RUST_LSP_MFA_ENABLED=true
```

### Docker Production Image

```dockerfile
FROM rust:1.75-slim as builder

# Install required system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Build the application
WORKDIR /app
COPY . .
RUN cargo build --release --features enterprise-monitoring,enterprise-sql-lsp

FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false sql-lsp

# Copy application binary
COPY --from=builder /app/target/release/sql-lsp-enterprise /usr/local/bin/
COPY --from=builder /app/config/ /etc/sql-lsp/

# Set ownership
RUN chown -R sql-lsp:sql-lsp /etc/sql-lsp

# Switch to non-root user
USER sql-lsp

# Expose ports
EXPOSE 8080 8443 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
  CMD curl -f http://localhost:9090/health || exit 1

# Run the application
CMD ["sql-lsp-enterprise"]
```

## Monitoring and Alerting

### Prometheus Metrics

```prometheus
# Cache performance metrics
sql_lsp_cache_hit_rate{layer="metrics"}
sql_lsp_cache_miss_count_total
sql_lsp_cache_eviction_count_total

# Memory usage metrics
sql_lsp_memory_usage_bytes
sql_lsp_memory_allocation_rate_per_second
sql_lsp_gc_collections_total

# Performance metrics
sql_lsp_query_execution_time_microseconds
sql_lsp_query_complexity_score
sql_lsp_active_connections_count

# Security metrics
sql_lsp_sql_injection_attempts_total
sql_lsp_rate_limit_exceeded_total
sql_lsp_authentication_failures_total

# Compliance metrics
sql_lsp_compliance_violations_total
sql_lsp_audit_log_entries_total
```

### Alert Rules

```yaml
groups:
- name: sql-lsp-enterprise
  rules:
  - alert: CacheHitRateLow
    expr: sql_lsp_cache_hit_rate < 0.85
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "Cache hit rate below target"
      description: "Cache hit rate is {{ $value }}%, below 85% target for 5 minutes"

  - alert: MemoryUsageHigh
    expr: sql_lsp_memory_usage_percent > 80
    for: 3m
    labels:
      severity: warning
    annotations:
      summary: "Memory usage above threshold"
      description: "Memory usage is {{ $value }}%, above 80% threshold for 3 minutes"

  - alert: SQLInjectionDetected
    expr: rate(sql_lsp_security_incidents_total[5m]) > 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "SQL injection attack detected"
      description: "Potential SQL injection attack detected"

  - alert: ComplianceViolation
    expr: sql_lsp_compliance_violations_total > 0
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "Compliance violation detected"
      description: "Security or compliance violation detected"
```

## Performance Benchmarks

### Target Performance Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Cache Hit Rate | ≥85% | Rolling average over 1 hour |
| Query Response Time | <500ms | P95 percentile |
| Memory Usage | ≤80% | Peak usage under normal load |
| Error Rate | <0.1% | Per minute average |
| Throughput | ≥1000 QPS | Sustained concurrent connections |

### Quarterly Regression Testing

1. **Performance Baseline**: Establish metrics with current dataset
2. **Load Testing**: Simulate production traffic patterns
3. **Memory Profiling**: Long-running memory leak detection
4. **Security Stress Testing**: Attack surface resilience testing
5. **Compliance Validation**: SOC2/GDPR compliance verification

### Scaling Thresholds

- **CPU Usage > 70%**: Add instance via HPA
- **Memory Usage > 80%**: Trigger emergency cache shedding
- **Active Connections > 1000**: Scale out horizontally
- **Response Time > 1s P95**: Add instances or optimize queries
- **Error Rate > 0.1%**: Investigate and mitigate root causes

This comprehensive API reference provides everything needed to deploy, configure, and operate the Enterprise SQL LSP Server in production environments. For additional support or specific deployment scenarios, consult the deployment guide or contact the platform engineering team.
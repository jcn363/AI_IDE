# Deployment Monitoring Setup

## Overview

Comprehensive monitoring setup for the Rust AI IDE deployment system, ensuring observability, alerting, and performance tracking across all environments.

## Table of Contents

- [Monitoring Architecture](#monitoring-architecture)
- [Metrics Collection](#metrics-collection)
- [Logging Setup](#logging-setup)
- [Alerting Configuration](#alerting-configuration)
- [Dashboards](#dashboards)
- [Health Checks](#health-checks)
- [Performance Monitoring](#performance-monitoring)
- [Troubleshooting](#troubleshooting)

## Monitoring Architecture

### Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Application   │    │   Prometheus    │    │     Grafana     │
│                 │    │                 │    │                 │
│ • Business      │◄──►│ • Metrics       │◄──►│ • Dashboards    │
│   Metrics       │    │   Collection    │    │ • Alerts        │
│ • Performance   │    │ • Service       │    │ • Analytics     │
│ • Health        │    │   Discovery     │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         ▲                       ▲                       ▲
         │                       │                       │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Fluent Bit     │    │   Alertmanager  │    │   Elasticsearch │
│                 │    │                 │    │                 │
│ • Log           │◄──►│ • Alert         │◄──►│ • Log Storage    │
│   Collection    │    │   Routing       │    │ • Search        │
│ • Processing    │    │ • Grouping      │    │ • Analytics     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Data Flow

1. **Metrics**: Application → Prometheus → Grafana
2. **Logs**: Application → Fluent Bit → Elasticsearch → Kibana
3. **Alerts**: Prometheus → Alertmanager → Slack/Email

## Metrics Collection

### Application Metrics

#### Rust-Specific Metrics

```rust
use prometheus::{Encoder, TextEncoder, register_counter, register_histogram};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REQUEST_COUNTER: prometheus::Counter =
        register_counter!("http_requests_total", "Total number of HTTP requests")
            .expect("Can't create metrics");

    pub static ref REQUEST_DURATION: prometheus::Histogram =
        register_histogram!("http_request_duration_seconds", "HTTP request duration")
            .expect("Can't create metrics");
}

// Usage in handlers
REQUEST_COUNTER.inc();
let _timer = REQUEST_DURATION.start_timer();
```

#### Custom Business Metrics

```rust
lazy_static! {
    pub static ref AI_INFERENCE_COUNTER: prometheus::Counter =
        register_counter!("ai_inference_requests_total", "AI inference requests")
            .expect("Can't create metrics");

    pub static ref MODEL_LOAD_TIME: prometheus::Histogram =
        register_histogram!("model_load_duration_seconds", "Model loading time")
            .expect("Can't create metrics");

    pub static ref MEMORY_USAGE: prometheus::Gauge =
        register_gauge!("memory_usage_bytes", "Current memory usage")
            .expect("Can't create metrics");
}
```

### System Metrics

#### Kubernetes Metrics

```yaml
# Prometheus ServiceMonitor
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: rust-ai-ide-monitor
  namespace: monitoring
spec:
  selector:
    matchLabels:
      app: rust-ai-ide
  endpoints:
  - port: metrics
    interval: 30s
    path: /metrics
```

#### Node and Pod Metrics

```yaml
# Kube-state-metrics integration
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: kube-state-metrics
spec:
  selector:
    matchLabels:
      app.kubernetes.io/name: kube-state-metrics
  endpoints:
  - port: http-metrics
  - port: telemetry
```

### Database Metrics (if applicable)

```yaml
# PostgreSQL exporter
apiVersion: v1
kind: ConfigMap
metadata:
  name: postgres-exporter-config
data:
  queries.yaml: |
    pg_stat_database:
      query: "SELECT * FROM pg_stat_database"
      metrics:
      - datname:
          usage: "LABEL"
          description: "Name of the database"
```

## Logging Setup

### Application Logging

#### Structured Logging in Rust

```rust
use tracing::{info, error, warn, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[instrument]
pub async fn handle_request(req: Request) -> Result<Response, Error> {
    info!(method = %req.method(), uri = %req.uri(), "Processing request");

    let result = process_request(req).await;

    match result {
        Ok(response) => {
            info!("Request processed successfully");
            Ok(response)
        }
        Err(err) => {
            error!(error = %err, "Request processing failed");
            Err(err)
        }
    }
}
```

#### Log Levels

```yaml
# Helm values configuration
logging:
  level: info  # debug, info, warn, error
  format: json  # json, text
  outputs:
    - stdout
    - file: /var/log/app.log
    - fluent-bit
```

### Fluent Bit Configuration

```yaml
# Fluent Bit ConfigMap
apiVersion: v1
kind: ConfigMap
metadata:
  name: fluent-bit-config
data:
  fluent-bit.conf: |
    [SERVICE]
        Flush         5
        Log_Level     info
        Daemon        off

    [INPUT]
        Name              tail
        Path              /var/log/containers/rust-ai-ide*.log
        Parser            docker
        Tag               kube.*
        Mem_Buf_Limit     5MB

    [FILTER]
        Name              kubernetes
        Match             kube.*
        Kube_URL          https://kubernetes.default.svc:443
        Kube_CA_File      /var/run/secrets/kubernetes.io/serviceaccount/ca.crt
        Kube_Token_File   /var/run/secrets/kubernetes.io/serviceaccount/token

    [OUTPUT]
        Name              es
        Match             *
        Host              elasticsearch.logging.svc.cluster.local
        Port              9200
        Index             rust-ai-ide
        Type              flb_type
```

## Alerting Configuration

### Prometheus Rules

```yaml
# PrometheusRule for Rust AI IDE
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: rust-ai-ide-alerts
  namespace: monitoring
spec:
  groups:
  - name: rust-ai-ide
    rules:
    - alert: RustAIIdeDown
      expr: up{job="rust-ai-ide"} == 0
      for: 5m
      labels:
        severity: critical
      annotations:
        summary: "Rust AI IDE is down"
        description: "Rust AI IDE has been down for more than 5 minutes"

    - alert: RustAIIdeHighErrorRate
      expr: rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m]) > 0.1
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "High error rate detected"
        description: "Error rate is {{ $value }}% over the last 5 minutes"

    - alert: RustAIIdeHighLatency
      expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 5
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "High latency detected"
        description: "95th percentile latency is {{ $value }}s"
```

### Alertmanager Configuration

```yaml
# Alertmanager ConfigMap
apiVersion: v1
kind: ConfigMap
metadata:
  name: alertmanager-config
data:
  alertmanager.yml: |
    global:
      smtp_smarthost: 'smtp.gmail.com:587'
      smtp_from: 'alerts@rust-ai-ide.com'
      smtp_auth_username: 'alerts@rust-ai-ide.com'
      smtp_auth_password: 'password'

    route:
      group_by: ['alertname']
      group_wait: 10s
      group_interval: 10s
      repeat_interval: 1h
      receiver: 'devops-team'
      routes:
      - match:
          severity: critical
        receiver: 'devops-pager'

    receivers:
    - name: 'devops-team'
      slack_configs:
      - api_url: 'https://hooks.slack.com/services/...'
        channel: '#alerts'
        title: '{{ .GroupLabels.alertname }}'
        text: '{{ .CommonAnnotations.description }}'

    - name: 'devops-pager'
      pagerduty_configs:
      - service_key: 'your-pagerduty-key'
```

## Dashboards

### Grafana Dashboards

#### Main Application Dashboard

```json
{
  "dashboard": {
    "title": "Rust AI IDE - Main",
    "tags": ["rust", "ai", "ide"],
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])",
            "legendFormat": "{{method}} {{uri}}"
          }
        ]
      },
      {
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total{status=~\"5..\"}[5m])",
            "legendFormat": "Errors"
          }
        ]
      },
      {
        "title": "Response Time",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          }
        ]
      }
    ]
  }
}
```

### Custom Panels

#### AI Inference Performance

```json
{
  "title": "AI Inference Performance",
  "type": "graph",
  "targets": [
    {
      "expr": "rate(ai_inference_requests_total[5m])",
      "legendFormat": "Inference Requests/sec"
    },
    {
      "expr": "histogram_quantile(0.95, rate(ai_inference_duration_seconds_bucket[5m]))",
      "legendFormat": "95th percentile latency"
    }
  ]
}
```

#### System Resources

```json
{
  "title": "System Resources",
  "type": "bargauge",
  "targets": [
    {
      "expr": "1 - (avg(irate(node_cpu_seconds_total{mode=\"idle\"}[5m])) by (instance))",
      "legendFormat": "CPU Usage"
    },
    {
      "expr": "(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes))",
      "legendFormat": "Memory Usage"
    }
  ]
}
```

## Health Checks

### Kubernetes Health Checks

```yaml
# Deployment with health checks
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rust-ai-ide
spec:
  template:
    spec:
      containers:
      - name: rust-ai-ide
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
        startupProbe:
          httpGet:
            path: /health/startup
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 30
```

### Application Health Endpoints

#### /health/live - Liveness Check

```rust
#[get("/health/live")]
pub async fn liveness() -> impl Responder {
    // Quick check: process is running and can respond
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().timestamp(),
        "service": "rust-ai-ide"
    }))
}
```

#### /health/ready - Readiness Check

```rust
#[get("/health/ready")]
pub async fn readiness(
    db_pool: web::Data<DbPool>,
    ai_service: web::Data<AiService>,
) -> impl Responder {
    // Check database connectivity
    let db_healthy = db_pool.check_connection().await.is_ok();

    // Check AI service availability
    let ai_healthy = ai_service.health_check().await.is_ok();

    if db_healthy && ai_healthy {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "ready",
            "checks": {
                "database": "ok",
                "ai_service": "ok"
            }
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "not_ready",
            "checks": {
                "database": if db_healthy { "ok" } else { "error" },
                "ai_service": if ai_healthy { "ok" } else { "error" }
            }
        }))
    }
}
```

#### /health/startup - Startup Check

```rust
#[get("/health/startup")]
pub async fn startup(
    cache: web::Data<Cache>,
    config: web::Data<AppConfig>,
) -> impl Responder {
    // Check if all required components are initialized
    let cache_ready = cache.is_ready().await;
    let config_loaded = config.is_loaded();

    if cache_ready && config_loaded {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "started",
            "components": {
                "cache": "ready",
                "configuration": "loaded"
            }
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "starting",
            "components": {
                "cache": if cache_ready { "ready" } else { "initializing" },
                "configuration": if config_loaded { "loaded" } else { "loading" }
            }
        }))
    }
}
```

### Synthetic Monitoring

```bash
# External health check script
#!/bin/bash

ENDPOINT="https://api.rust-ai-ide.com/health"
TIMEOUT=30

if curl -f --max-time $TIMEOUT "$ENDPOINT" >/dev/null 2>&1; then
    echo "Health check passed"
    exit 0
else
    echo "Health check failed"
    exit 1
fi
```

## Performance Monitoring

### Application Performance Monitoring (APM)

#### Custom Performance Metrics

```rust
pub struct PerformanceMonitor {
    request_count: prometheus::Counter,
    request_duration: prometheus::Histogram,
    memory_usage: prometheus::Gauge,
    cpu_usage: prometheus::Gauge,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            request_count: register_counter!("http_requests_total", "Total HTTP requests").unwrap(),
            request_duration: register_histogram!("http_request_duration_seconds", "Request duration").unwrap(),
            memory_usage: register_gauge!("memory_usage_bytes", "Memory usage").unwrap(),
            cpu_usage: register_gauge!("cpu_usage_percent", "CPU usage percentage").unwrap(),
        }
    }

    pub fn record_request(&self, duration: std::time::Duration) {
        self.request_count.inc();
        self.request_duration.observe(duration.as_secs_f64());
    }

    pub fn update_system_metrics(&self) {
        // Update memory and CPU usage
        // Implementation depends on system monitoring libraries
    }
}
```

### Distributed Tracing

#### OpenTelemetry Integration

```rust
use opentelemetry::{global, trace::Tracer};
use tracing_opentelemetry::OpenTelemetryLayer;

pub fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    let tracer = global::tracer("rust-ai-ide");

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(OpenTelemetryLayer::new(tracer))
        .try_init()?;

    Ok(())
}
```

```yaml
# OpenTelemetry collector configuration
receivers:
  otlp:
    protocols:
      grpc:
      http:

processors:
  batch:

exporters:
  jaeger:
    endpoint: "jaeger:14268/api/traces"

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [jaeger]
```

### Profiling

#### CPU and Memory Profiling

```rust
#[cfg(feature = "profiling")]
use pprof::ProfilerGuard;

#[cfg(feature = "profiling")]
pub struct Profiler {
    guard: ProfilerGuard<'static>,
}

#[cfg(feature = "profiling")]
impl Profiler {
    pub fn new() -> Self {
        let guard = pprof::ProfilerGuard::new(100).unwrap();
        Self { guard }
    }

    pub fn report(&self) -> pprof::Report {
        self.guard.report().build().unwrap()
    }
}
```

## Troubleshooting

### Common Monitoring Issues

#### Metrics Not Appearing

1. **Check Prometheus configuration**
   ```bash
   kubectl get prometheusrules -n monitoring
   kubectl describe prometheusrules rust-ai-ide-alerts
   ```

2. **Verify service discovery**
   ```bash
   kubectl get servicemonitors -n monitoring
   kubectl describe servicemonitor rust-ai-ide-monitor
   ```

3. **Check application metrics endpoint**
   ```bash
   curl http://rust-ai-ide:9090/metrics
   ```

#### Alerts Not Firing

1. **Validate Prometheus rules**
   ```bash
   kubectl exec -it prometheus-0 -n monitoring -- promtool check rules /etc/prometheus/prometheus.yml
   ```

2. **Test alert expressions**
   ```bash
   kubectl exec -it prometheus-0 -n monitoring -- promtool query instant 'up{job="rust-ai-ide"}'
   ```

### Log Analysis

#### Log Queries in Kibana

```
# Successful requests
kubernetes.labels.app: rust-ai-ide AND level: info AND message: "Request processed successfully"

# Errors
kubernetes.labels.app: rust-ai-ide AND level: error

# AI inference requests
kubernetes.labels.app: rust-ai-ide AND ai.inference.request
```

#### Log Visualization

```bash
# Extract error patterns
kubectl logs -f deployment/rust-ai-ide -n rust-ai-ide-prod | grep -E "ERROR|WARN" | jq '.'

# Monitor request patterns
kubectl logs -f deployment/rust-ai-ide -n rust-ai-ide-prod | grep "Processing request" | awk '{print $1, $7}'
```

### Performance Analysis

#### Memory Leak Detection

```bash
# Monitor memory usage over time
kubectl top pods -n rust-ai-ide-prod --containers

# Check for memory leaks in logs
kubectl logs deployment/rust-ai-ide -n rust-ai-ide-prod | grep -i "memory\|leak"
```

#### CPU Profiling

```bash
# Generate flame graph
kubectl exec -it deployment/rust-ai-ide -n rust-ai-ide-prod -- /usr/local/bin/perf record -F 99 -a -- sleep 60
kubectl exec -it deployment/rust-ai-ide -n rust-ai-ide-prod -- /usr/local/bin/perf script | /usr/local/bin/stackcollapse-perf.pl | /usr/local/bin/flamegraph.pl > flame-graph.svg
```

### Automated Diagnostics

```bash
# Comprehensive health check script
#!/bin/bash

echo "=== Rust AI IDE Health Check ==="
echo "Timestamp: $(date)"
echo

echo "1. Application Status:"
kubectl get pods -n rust-ai-ide-prod -l app=rust-ai-ide
echo

echo "2. Service Endpoints:"
kubectl get endpoints -n rust-ai-ide-prod
echo

echo "3. Recent Events:"
kubectl get events -n rust-ai-ide-prod --sort-by=.metadata.creationTimestamp | tail -10
echo

echo "4. Resource Usage:"
kubectl top pods -n rust-ai-ide-prod --containers
echo

echo "5. Application Health:"
curl -s https://api.rust-ai-ide.com/health | jq .
echo

echo "6. Error Logs (last 5 minutes):"
kubectl logs --since=5m deployment/rust-ai-ide -n rust-ai-ide-prod | grep -i error | tail -10
```

### Escalation Procedures

1. **Warning Level**: Slack notification to devops channel
2. **Critical Level**: PagerDuty alert + SMS to on-call engineer
3. **Emergency**: Multiple channels + executive notification

### Monitoring Alerts Contact Matrix

| Alert Level | Channels | Response Time | Team |
|-------------|----------|--------------|------|
| Warning | Slack | < 30 minutes | DevOps |
| Critical | Slack + PagerDuty | < 15 minutes | DevOps + Dev |
| Emergency | All channels + SMS | < 5 minutes | DevOps + Dev + Management |

---

## Quick Commands

```bash
# Check application status
kubectl get pods -n rust-ai-ide-prod -l app=rust-ai-ide

# View metrics
kubectl port-forward svc/prometheus 9090:9090 -n monitoring

# View dashboards
kubectl port-forward svc/grafana 3000:3000 -n monitoring

# Check logs
kubectl logs -f deployment/rust-ai-ide -n rust-ai-ide-prod

# Monitor resources
kubectl top pods -n rust-ai-ide-prod --containers

# Health check
curl -f https://api.rust-ai-ide.com/health
```

*Last updated: $(date)*
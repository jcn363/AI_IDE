# Model Warmup Prediction System - Diagnostic Tools Guide

[![Rust Nightly](https://img.shields.io/badge/rust-nightly-orange.svg)](https://rust-lang.github.io/rustup/concepts/channels.html)
[![Documentation](https://docs.rs/rust-ai-ide-warmup-predictor/badge.svg)](https://docs.rs/rust-ai-ide-warmup-predictor)

This guide provides comprehensive diagnostic tools and procedures for troubleshooting the Model Warmup Prediction System. It includes interactive diagnostic tools, log analysis techniques, automated health checks, and step-by-step debugging workflows for all 7 core components.

## Table of Contents

- [Quick Start](#quick-start)
- [Interactive Diagnostic Tools](#interactive-diagnostic-tools)
- [Log Analysis Guide](#log-analysis-guide)
- [Automated Health Checks](#automated-health-checks)
- [Component-Specific Diagnostics](#component-specific-diagnostics)
- [Integration Diagnostics](#integration-diagnostics)
- [Performance Profiling](#performance-profiling)
- [Debug Commands Reference](#debug-commands-reference)

## Quick Start

### One-Click System Diagnosis

```bash
# Run complete system diagnostic
curl -X POST http://localhost:3000/api/warmup/diagnostics/full-system-scan \
  -H "Content-Type: application/json" \
  -d '{"include_performance": true, "include_logs": true}'
```

### Real-time Component Health Dashboard

```bash
# Start interactive health dashboard
cargo run --bin warmup_health_dashboard
# Opens web interface at http://localhost:8080
```

## Interactive Diagnostic Tools

### System Health Scanner

```rust
use rust_ai_ide_warmup_predictor::diagnostics::SystemHealthScanner;

let scanner = SystemHealthScanner::new().await?;
let health_report = scanner.scan_all_components().await?;

println!("System Health: {:.1}%", health_report.overall_health_percentage);
for component in health_report.component_status {
    println!("{}: {}", component.name, component.status);
}
```

### Interactive Component Inspector

```rust
use rust_ai_ide_warmup_predictor::diagnostics::ComponentInspector;

let inspector = ComponentInspector::new().await?;

// Inspect specific component
let usage_analyzer_status = inspector.inspect_component("UsagePatternAnalyzer").await?;
println!("Component Status: {}", usage_analyzer_status.health_status);

// Interactive inspection
inspector.start_interactive_mode().await?;
```

### Real-time Metrics Monitor

```rust
use rust_ai_ide_warmup_predictor::diagnostics::RealtimeMetricsMonitor;

let monitor = RealtimeMetricsMonitor::new().await?;
monitor.start_monitoring().await?;

// Monitor specific metrics
monitor.watch_metric("prediction_accuracy", |value| {
    if value < 0.7 {
        println!("WARNING: Prediction accuracy dropped to {:.2}%", value * 100.0);
    }
}).await?;
```

## Log Analysis Guide

### Log Collection and Parsing

#### Automated Log Collection

```bash
# Collect logs from all components
./scripts/collect_warmup_logs.sh --since "1 hour ago" --output /tmp/warmup_logs.tar.gz

# Parse and analyze logs
./scripts/analyze_warmup_logs.sh /tmp/warmup_logs.tar.gz
```

#### Log Pattern Analysis

```rust
use rust_ai_ide_warmup_predictor::diagnostics::LogAnalyzer;

let analyzer = LogAnalyzer::new().await?;

// Analyze error patterns
let error_patterns = analyzer.analyze_error_patterns(log_files).await?;
for pattern in error_patterns.frequent_patterns {
    println!("Pattern: {} (occurrences: {})", pattern.signature, pattern.count);
}

// Detect anomalies
let anomalies = analyzer.detect_anomalies(log_files).await?;
for anomaly in anomalies.significant_anomalies {
    println!("Anomaly: {} at {}", anomaly.description, anomaly.timestamp);
}
```

### Component-Specific Log Analysis

#### UsagePatternAnalyzer Logs

```bash
# Search for pattern analysis logs
grep "PATTERN_ANALYSIS" /var/log/warmup-system/*.log | tail -20

# Analyze pattern detection effectiveness
grep "PATTERN_DETECTED\|PATTERN_FAILED" /var/log/warmup-system/usage_analyzer.log \
  | awk '{print $1, $2, $8}' \
  | sort | uniq -c
```

#### PredictionEngine Logs

```bash
# Monitor prediction accuracy
grep "PREDICTION_ACCURACY" /var/log/warmup-system/prediction_engine.log \
  | awk '{print $1, $2, $NF}' \
  | tail -10

# Check for model loading issues
grep "MODEL_LOAD\|MODEL_ERROR" /var/log/warmup-system/prediction_engine.log
```

#### ResourceManager Logs

```bash
# Monitor resource usage
grep "RESOURCE_USAGE" /var/log/warmup-system/resource_manager.log \
  | awk '{print $1, $2, $4, $6}' \
  | tail -20

# Check for allocation failures
grep "ALLOCATION_FAILED\|RESOURCE_EXHAUSTED" /var/log/warmup-system/resource_manager.log
```

## Automated Health Checks

### Health Check Configuration

```rust
use rust_ai_ide_warmup_predictor::health::HealthChecker;

let mut checker = HealthChecker::new();

// Configure component health checks
checker.add_component_check("UsagePatternAnalyzer", |component| async move {
    // Check if component is responding
    let response_time = component.ping().await?;
    if response_time > Duration::from_millis(500) {
        return Err(HealthCheckError::SlowResponse(response_time));
    }

    // Check data freshness
    let last_update = component.last_data_update().await?;
    if last_update.elapsed() > Duration::from_secs(300) {
        return Err(HealthCheckError::StaleData(last_update.elapsed()));
    }

    Ok(())
});

// Configure system-wide checks
checker.add_system_check("memory_usage", |system| async move {
    let mem_usage = system.get_memory_usage().await?;
    if mem_usage.percentage > 90.0 {
        return Err(HealthCheckError::HighMemoryUsage(mem_usage.percentage));
    }
    Ok(())
});
```

### Scheduled Health Monitoring

```rust
use rust_ai_ide_warmup_predictor::health::ScheduledHealthMonitor;

let monitor = ScheduledHealthMonitor::new(checker);

// Schedule regular health checks
monitor.schedule_check(Duration::from_secs(60)).await?; // Every minute
monitor.schedule_check(Duration::from_secs(3600)).await?; // Every hour

// Enable alerting
monitor.set_alert_handler(|alert| async move {
    match alert.severity {
        Severity::Critical => {
            // Send immediate alert
            send_critical_alert(alert).await?;
        }
        Severity::Warning => {
            // Log warning
            log_warning(alert).await?;
        }
        _ => {}
    }
    Ok(())
}).await?;
```

### Health Check Endpoints

```rust
// Tauri command for health status
#[tauri::command]
async fn get_system_health(state: State<ModelWarmupPredictor>) -> Result<SystemHealth, String> {
    let health = state.health_checker.check_all().await
        .map_err(|e| e.to_string())?;
    Ok(health)
}

// REST API endpoint
async fn health_endpoint() -> impl Responder {
    let health = global_health_checker.check_all().await;
    HttpResponse::Ok().json(health)
}
```

## Component-Specific Diagnostics

### UsagePatternAnalyzer Diagnostics

```rust
use rust_ai_ide_warmup_predictor::diagnostics::UsagePatternAnalyzerDiagnostic;

let diagnostic = UsagePatternAnalyzerDiagnostic::new();

// Check data collection
let data_health = diagnostic.check_data_collection().await?;
println!("Data Collection: {}", data_health.status);

// Analyze pattern quality
let pattern_quality = diagnostic.analyze_pattern_quality().await?;
println!("Pattern Quality Score: {:.2}", pattern_quality.score);

// Validate pattern detection
let validation = diagnostic.validate_pattern_detection().await?;
for issue in validation.issues {
    println!("Issue: {}", issue.description);
}
```

### PredictionEngine Diagnostics

```rust
use rust_ai_ide_warmup_predictor::diagnostics::PredictionEngineDiagnostic;

let diagnostic = PredictionEngineDiagnostic::new();

// Test prediction accuracy
let accuracy_test = diagnostic.test_prediction_accuracy(test_data).await?;
println!("Accuracy: {:.2}%", accuracy_test.accuracy * 100.0);

// Check model health
let model_health = diagnostic.check_model_health().await?;
println!("Model Health: {}", model_health.status);

// Validate prediction latency
let latency_test = diagnostic.test_prediction_latency().await?;
println!("P95 Latency: {:.2}ms", latency_test.p95_latency_ms);
```

### ResourceManager Diagnostics

```rust
use rust_ai_ide_warmup_predictor::diagnostics::ResourceManagerDiagnostic;

let diagnostic = ResourceManagerDiagnostic::new();

// Monitor resource utilization
let utilization = diagnostic.monitor_resource_utilization().await?;
println!("CPU: {:.1}%, Memory: {:.1}%", utilization.cpu_percent, utilization.memory_percent);

// Check allocation efficiency
let efficiency = diagnostic.analyze_allocation_efficiency().await?;
println!("Allocation Efficiency: {:.2}%", efficiency.efficiency * 100.0);

// Detect resource leaks
let leaks = diagnostic.detect_resource_leaks().await?;
for leak in leaks.detected_leaks {
    println!("Leak detected: {} - {} bytes", leak.resource_type, leak.size_bytes);
}
```

## Integration Diagnostics

### Tauri Integration Diagnostics

```rust
use rust_ai_ide_warmup_predictor::diagnostics::TauriIntegrationDiagnostic;

let diagnostic = TauriIntegrationDiagnostic::new();

// Test IPC communication
let ipc_test = diagnostic.test_ipc_communication().await?;
println!("IPC Health: {}", ipc_test.status);

// Check command handlers
let command_test = diagnostic.test_command_handlers().await?;
for command in command_test.commands {
    println!("Command {}: {}", command.name, command.status);
}

// Validate state management
let state_test = diagnostic.validate_state_management().await?;
println!("State Management: {}", state_test.status);
```

### EventBus Integration Diagnostics

```rust
use rust_ai_ide_warmup_predictor::diagnostics::EventBusDiagnostic;

let diagnostic = EventBusDiagnostic::new();

// Test event publishing
let publish_test = diagnostic.test_event_publishing().await?;
println!("Event Publishing: {}", publish_test.status);

// Check event subscriptions
let subscription_test = diagnostic.test_event_subscriptions().await?;
for subscription in subscription_test.subscriptions {
    println!("Subscription {}: {}", subscription.event_type, subscription.status);
}

// Validate event routing
let routing_test = diagnostic.validate_event_routing().await?;
println!("Event Routing: {} messages processed", routing_test.messages_processed);
```

### LSP Service Integration Diagnostics

```rust
use rust_ai_ide_warmup_predictor::diagnostics::LSPIntegrationDiagnostic;

let diagnostic = LSPIntegrationDiagnostic::new();

// Test LSP communication
let lsp_test = diagnostic.test_lsp_communication().await?;
println!("LSP Communication: {}", lsp_test.status);

// Check model loading coordination
let model_test = diagnostic.test_model_loading_coordination().await?;
for model in model_test.models {
    println!("Model {}: {}", model.name, model.status);
}

// Validate service discovery
let discovery_test = diagnostic.validate_service_discovery().await?;
println!("Service Discovery: {} services found", discovery_test.services_found);
```

## Performance Profiling

### Component Performance Profiling

```rust
use rust_ai_ide_warmup_predictor::diagnostics::PerformanceProfiler;

let profiler = PerformanceProfiler::new();

// Profile component operations
let profile = profiler.profile_component("PredictionEngine").await?;
println!("Average Latency: {:.2}ms", profile.average_latency_ms);
println!("Throughput: {:.0} req/sec", profile.throughput_req_per_sec);
println!("Memory Usage: {:.1} MB", profile.memory_usage_mb);

// Compare performance baselines
let comparison = profiler.compare_with_baseline(profile).await?;
println!("Performance vs Baseline: {:.1}%", comparison.performance_change_percent);
```

### Memory Profiling

```rust
use rust_ai_ide_warmup_predictor::diagnostics::MemoryProfiler;

let profiler = MemoryProfiler::new();

// Profile memory usage
let memory_profile = profiler.profile_memory_usage().await?;
println!("Heap Usage: {:.1} MB", memory_profile.heap_usage_mb);
println!("Stack Usage: {:.1} MB", memory_profile.stack_usage_mb);

// Detect memory leaks
let leak_detection = profiler.detect_memory_leaks().await?;
for leak in leak_detection.leaks {
    println!("Memory leak: {} bytes at {}", leak.size_bytes, leak.location);
}
```

### Concurrency Profiling

```rust
use rust_ai_ide_warmup_predictor::diagnostics::ConcurrencyProfiler;

let profiler = ConcurrencyProfiler::new();

// Profile thread utilization
let thread_profile = profiler.profile_thread_utilization().await?;
for thread in thread_profile.threads {
    println!("Thread {}: {:.1}% utilization", thread.name, thread.utilization_percent);
}

// Analyze lock contention
let contention = profiler.analyze_lock_contention().await?;
for lock in contention.hotspots {
    println!("Lock contention: {} waits", lock.wait_count);
}
```

## Debug Commands Reference

### System-Level Debug Commands

```bash
# Enable debug logging for all components
curl -X POST http://localhost:3000/api/warmup/debug/enable-all \
  -H "Content-Type: application/json" \
  -d '{"log_level": "debug"}'

# Disable debug logging
curl -X POST http://localhost:3000/api/warmup/debug/disable-all

# Reset all components to default state
curl -X POST http://localhost:3000/api/warmup/debug/reset-all
```

### Component-Specific Debug Commands

```bash
# Debug UsagePatternAnalyzer
curl -X POST http://localhost:3000/api/warmup/debug/usage-pattern-analyzer \
  -H "Content-Type: application/json" \
  -d '{"action": "enable_pattern_tracing"}'

# Debug PredictionEngine
curl -X POST http://localhost:3000/api/warmup/debug/prediction-engine \
  -H "Content-Type: application/json" \
  -d '{"action": "enable_prediction_logging", "model_id": "default"}'

# Debug ResourceManager
curl -X POST http://localhost:3000/api/warmup/debug/resource-manager \
  -H "Content-Type: application/json" \
  -d '{"action": "enable_resource_tracing"}'
```

### Diagnostic Report Generation

```bash
# Generate full diagnostic report
curl -X GET http://localhost:3000/api/warmup/diagnostics/report \
  -H "Accept: application/json" \
  > diagnostic_report.json

# Generate component-specific report
curl -X GET "http://localhost:3000/api/warmup/diagnostics/report?component=PredictionEngine" \
  > prediction_engine_report.json

# Generate performance report
curl -X GET http://localhost:3000/api/warmup/diagnostics/performance-report \
  -H "Accept: application/json" \
  > performance_report.json
```

### Automated Diagnostic Scripts

```bash
#!/bin/bash
# comprehensive_system_diagnostic.sh

echo "Starting comprehensive system diagnostic..."

# Check system health
echo "1. Checking system health..."
curl -s http://localhost:3000/api/warmup/health | jq '.'

# Run component diagnostics
echo "2. Running component diagnostics..."
for component in UsagePatternAnalyzer PredictionEngine WarmupScheduler ResourceManager WarmupQueue PerformancePredictor ModelWarmupMetrics; do
    echo "Diagnosing $component..."
    curl -s "http://localhost:3000/api/warmup/diagnostics/component?component=$component" | jq '.'
done

# Check integration health
echo "3. Checking integration health..."
curl -s http://localhost:3000/api/warmup/diagnostics/integration | jq '.'

# Generate performance report
echo "4. Generating performance report..."
curl -s http://localhost:3000/api/warmup/diagnostics/performance > performance_report.json

echo "Diagnostic complete. Check output files for details."
```

### Log Analysis Scripts

```bash
#!/bin/bash
# analyze_warmup_logs.sh

LOG_DIR="/var/log/warmup-system"
OUTPUT_DIR="./log_analysis"

mkdir -p "$OUTPUT_DIR"

echo "Analyzing warmup system logs..."

# Extract error patterns
echo "1. Extracting error patterns..."
grep -r "ERROR\|CRITICAL" "$LOG_DIR" > "$OUTPUT_DIR/error_patterns.log"

# Analyze performance metrics
echo "2. Analyzing performance metrics..."
grep -r "PERFORMANCE\|LATENCY" "$LOG_DIR" > "$OUTPUT_DIR/performance_metrics.log"

# Check for anomalies
echo "3. Checking for anomalies..."
grep -r "ANOMALY\|UNEXPECTED" "$LOG_DIR" > "$OUTPUT_DIR/anomalies.log"

# Generate summary report
echo "4. Generating summary report..."
cat > "$OUTPUT_DIR/summary_report.txt" << EOF
Log Analysis Summary
====================

Total log files analyzed: $(find "$LOG_DIR" -name "*.log" | wc -l)
Total error entries: $(wc -l < "$OUTPUT_DIR/error_patterns.log")
Total performance entries: $(wc -l < "$OUTPUT_DIR/performance_metrics.log")
Total anomalies detected: $(wc -l < "$OUTPUT_DIR/anomalies.log")

Top error patterns:
$(cut -d' ' -f4- "$OUTPUT_DIR/error_patterns.log" | sort | uniq -c | sort -nr | head -10)

EOF

echo "Log analysis complete. Results in $OUTPUT_DIR/"
```

This diagnostic tools guide provides comprehensive tools and procedures for diagnosing issues in the Model Warmup Prediction System. Use these tools proactively to maintain system health and quickly resolve issues when they occur.
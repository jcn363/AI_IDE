# Comprehensive Performance Testing Guide

This guide demonstrates how to use the advanced performance testing capabilities for the Model Warmup Prediction System's 7 core components.

## 🚀 Quick Start

```rust
use rust_ai_ide_warmup_predictor::comprehensive_performance_tests::*;

// Create comprehensive test suite
let suite = ComprehensivePerformanceTestSuite::new()?;

// Run all component tests
let results = suite.run_all_component_tests().await?;

// Generate comprehensive report
println!("{}", results.generate_comprehensive_report());

Ok(())
```

## 📊 Testing All 7 Core Components

The system provides comprehensive testing for all core components:

### 1. UsagePatternAnalyzer
```rust
// Test usage pattern analysis
let analyzer_results = suite.test_usage_pattern_analyzer().await?;
```

### 2. PredictionEngine
```rust
// Test prediction accuracy and performance
let prediction_results = suite.test_prediction_engine().await?;
```

### 3. WarmupScheduler
```rust
// Test scheduling algorithms
let scheduler_results = suite.test_warmup_scheduler().await?;
```

### 4. ResourceManager
```rust
// Test resource monitoring and allocation
let resource_results = suite.test_resource_manager().await?;
```

### 5. WarmupQueue
```rust
// Test queue throughput and prioritization
let queue_results = suite.test_warmup_queue().await?;
```

### 6. PerformancePredictor
```rust
// Test performance impact prediction
let perf_predictor_results = suite.test_performance_predictor().await?;
```

### 7. ModelWarmupMetrics
```rust
// Test metrics collection and analysis
let metrics_results = suite.test_model_warmup_metrics().await?;
```

## 🧪 Advanced Testing Capabilities

### Micro-Benchmarks
Test individual algorithms with precise timing:
```rust
let micro_result = suite.suite.benchmarker.micro_benchmark_components(
    "pattern_matching_algorithm",
    || {
        // Your algorithm implementation
        Ok(())
    }
).await?;
```

### Latency Analysis with Percentiles
```rust
let latencies = vec![
    Duration::from_millis(10),
    Duration::from_millis(15),
    Duration::from_millis(12),
    // ... more latency measurements
];

let analysis = suite.suite.benchmarker.analyze_latency_distribution(&latencies)?;
println!("P95 Latency: {:?}", analysis.p95);
println!("P99 Latency: {:?}", analysis.p99);
```

### Memory Profiling
```rust
let memory_profile = suite.suite.benchmarker.memory_profile(
    "memory_intensive_operation",
    async {
        // Operation that may have memory leaks
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(())
    }
).await?;

if !memory_profile.potential_leaks.is_empty() {
    println!("⚠️  Memory leaks detected: {:?}", memory_profile.potential_leaks);
}
```

### Accuracy Validation
```rust
let validator = suite.suite.accuracy_validator.write().await;
validator.add_ground_truth(AccuracyTestCase {
    input: sample_request,
    expected_result: Some(expected_model_id),
    expected_confidence: 0.85,
});

// Validate accuracy
let accuracy_results = validator.validate_accuracy(predictor_function).await?;
println!("Overall Accuracy: {:.2}%", accuracy_results.overall_accuracy * 100.0);
```

### Load Testing
```rust
let load_tester = suite.suite.load_tester.write().await;
load_tester.add_scenario(LoadTestScenario {
    name: "high_load_scenario".to_string(),
    patterns: vec![RequestPattern {
        weight: 1.0,
        request_template: sample_request,
    }],
    target_throughput: 500.0, // requests per second
    duration: Duration::from_secs(300), // 5 minutes
});

let load_results = load_tester.run_scenario("high_load_scenario", request_handler).await?;
```

### Stress Testing
```rust
let mut stress_tester = suite.suite.stress_tester.write().await;
stress_tester.find_saturation_point(
    "saturation_test",
    1000, // max concurrent users
    || async {
        // Simulate request processing
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }
).await?;
```

### Statistical Analysis
```rust
let samples = vec![120.5, 115.2, 118.8, 122.1, 119.9]; // latency samples in ms
let statistical_analysis = suite.suite.benchmarker.statistical_analysis(
    "latency_distribution",
    &samples
).await?;

println!("Mean: {:.2}ms", statistical_analysis.mean);
println!("Std Dev: {:.2}ms", statistical_analysis.std_dev);
println!("95% CI: [{:.2}, {:.2}]", statistical_analysis.confidence_interval_95.0, statistical_analysis.confidence_interval_95.1);
```

### Automated Monitoring
```rust
let monitor = suite.suite.automated_monitor.write().await;
monitor.set_alert_threshold("cpu_percent", 80.0);
monitor.set_alert_threshold("memory_mb", 4000.0);

// Record performance snapshot
let snapshot = PerformanceSnapshot {
    timestamp: Utc::now(),
    system_metrics: SystemMetrics {
        cpu_percent: 45.2,
        memory_mb: 2048.0,
        disk_io_bytes: 1024,
        network_io_bytes: 512,
    },
    application_metrics: ApplicationMetrics {
        active_connections: 15,
        request_rate: 120.5,
        error_rate: 0.005,
        avg_latency_ms: 45.2,
    },
};

let alerts = monitor.record_snapshot(snapshot).await?;
for alert in alerts {
    println!("🚨 {}", alert);
}
```

### Regression Detection
```rust
// Set baseline
suite.suite.benchmarker.set_baseline("throughput", baseline_result).await?;

// Detect regression
let current_throughput = 850.0;
let has_regression = suite.suite.benchmarker.detect_regression("throughput", current_throughput, 0.95).await?;
if has_regression {
    println!("⚠️  Performance regression detected!");
}
```

## 📈 Test Configuration

Customize test behavior with `PerformanceTestConfig`:

```rust
let config = PerformanceTestConfig {
    iterations: 2000,           // Number of iterations per test
    warmup_iterations: 200,     // Warmup iterations
    max_duration: Duration::from_secs(600), // Max test duration
    memory_profiling: true,     // Enable memory profiling
    detailed_latency: true,     // Enable detailed latency analysis
    concurrent_users: 100,      // Load test concurrent users
    max_load: 500,              // Stress test max load
};

let suite = ComprehensivePerformanceTestSuite { config, ..Default::default() };
```

## 📋 Test Scenarios

### Real-World Scenarios

1. **IDE Startup Scenario**
   ```rust
   // Test warmup prediction during IDE cold start
   let startup_results = suite.test_integrated_workflows().await?;
   ```

2. **Heavy Development Session**
   ```rust
   // Simulate intensive coding session with frequent AI interactions
   let session_config = LoadTestConfig {
       target_throughput: 50.0,
       duration: Duration::from_secs(1800), // 30 minutes
       // ... other config
   };
   ```

3. **Large Project Context**
   ```rust
   // Test with large codebase context
   let large_project_request = WarmupRequest {
       project_context: ProjectContext {
           language: "rust".to_string(),
           size_lines: 50000, // 50K lines
           complexity_score: 0.9,
           // ... other fields
       },
       // ... other fields
   };
   ```

### Edge Cases

1. **Resource Constraints**
   ```rust
   // Test behavior under memory pressure
   let constrained_config = WarmupConfig {
       max_memory_mb: 512, // Very limited memory
       max_cpu_percent: 20.0, // Limited CPU
       // ... other constraints
   };
   ```

2. **High Contention**
   ```rust
   // Test with many concurrent requests
   let high_contention_config = BenchmarkConfig {
       concurrent_requests: 200,
       // ... other settings
   };
   ```

3. **Network Issues**
   ```rust
   // Simulate network latency/degradation
   // (Implement custom network simulation layer)
   ```

## 📊 Performance Metrics Dashboard

The system generates comprehensive reports including:

- **Latency Analysis**: P50, P90, P95, P99, P99.9 percentiles
- **Throughput Metrics**: Requests per second, error rates
- **Memory Usage**: Heap size, peak usage, leak detection
- **Resource Utilization**: CPU, memory, network, disk I/O
- **Accuracy Metrics**: Precision, recall, F1-score, confidence intervals
- **Statistical Analysis**: Mean, variance, confidence intervals, outlier detection
- **Regression Detection**: Performance trend analysis and alerts

### Sample Report Output
```
# Comprehensive Performance Test Report

## Executive Summary
This report contains comprehensive performance analysis of all 7 core components
of the Model Warmup Prediction System.

## Component Performance Details

### UsagePatternAnalyzer
#### Micro-Benchmarks
- usage_pattern_analysis: 5.23ms avg latency, 191.23 req/sec throughput

#### Memory Usage
- Heap: 45MB, Peak: 67MB

### PredictionEngine
#### Micro-Benchmarks
- prediction_accuracy: 12.45ms avg latency, 80.32 req/sec throughput

#### Latency Analysis
- P95 Latency: 18.5ms
- P99 Latency: 25.2ms

## Recommendations
1. Performance baselines met across all components
2. Memory usage within acceptable limits
3. Scalability verified up to tested load levels
```

## 🔧 Integration with CI/CD

### Automated Performance Regression Testing

```bash
# Run performance tests in CI
cargo test -p rust-ai-ide-warmup-predictor --test performance_tests

# Generate performance report
cargo run --bin performance_report_generator > performance_report.md

# Compare against baselines
cargo run --bin regression_detector -- --baseline main_branch.json --current feature_branch.json
```

### Performance Baselines

```rust
// Store performance baselines
let baseline_results = suite.run_all_component_tests().await?;
suite.suite.benchmarker.export_results("performance_baselines.json").await?;
```

### Alert Integration

```rust
// Send alerts to monitoring system
for alert in alerts {
    // Send to Slack, email, or monitoring dashboard
    send_alert(&alert).await?;
}
```

## 🎯 Best Practices

### Test Design
1. **Warmup Period**: Always include warmup iterations before measurement
2. **Statistical Significance**: Run sufficient iterations for reliable results
3. **Realistic Scenarios**: Use production-like data and request patterns
4. **Resource Monitoring**: Track system resources during tests
5. **Baseline Comparison**: Compare against known good baselines

### Performance Targets
- **Latency**: P95 < 100ms for interactive operations
- **Throughput**: > 100 req/sec for typical workloads
- **Memory**: < 500MB per component under normal load
- **CPU**: < 50% utilization under moderate load
- **Error Rate**: < 1% under normal operating conditions

### Monitoring
- Track performance trends over time
- Set up alerts for performance regressions
- Monitor resource usage patterns
- Validate accuracy metrics regularly

## 🚨 Troubleshooting

### Common Issues

1. **Inconsistent Results**
   - Ensure system is idle during testing
   - Use sufficient warmup iterations
   - Control for external factors (network, disk I/O)

2. **Memory Leaks**
   - Check memory profiling output
   - Look for growing heap usage over time
   - Validate garbage collection efficiency

3. **Performance Regressions**
   - Compare against recent baselines
   - Check for changes in dependencies
   - Profile CPU and memory usage

4. **High Latency Variance**
   - Analyze latency distribution
   - Check for GC pauses or I/O blocking
   - Validate thread contention

## 📚 API Reference

### Core Types
- `ComprehensivePerformanceTestSuite` - Main test suite
- `ComponentTestResults` - Individual component results
- `BenchmarkResult` - Single benchmark result
- `LatencyAnalysis` - Detailed latency statistics
- `MemoryProfile` - Memory usage analysis
- `AccuracyValidation` - Prediction accuracy metrics

### Advanced Features
- `StatisticalAnalyzer` - Statistical analysis tools
- `LoadTester` - Load testing framework
- `StressTester` - Stress testing capabilities
- `AutomatedMonitor` - Continuous monitoring
- `RegressionDetector` - Performance regression detection

This comprehensive testing system ensures the Model Warmup Prediction System maintains high performance, accuracy, and reliability across all 7 core components.
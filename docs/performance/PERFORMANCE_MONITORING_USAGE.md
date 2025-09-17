# Performance Monitoring Usage Guide

This guide explains how to use the performance monitoring system to track, baseline, and detect regressions in the Rust AI IDE project.

## Overview

The performance monitoring system provides comprehensive tracking of:

- CPU-bound workload performance (synchronous operations)
- I/O-bound workload performance (asynchronous operations)
- Memory usage patterns
- Build time metrics
- Regression detection and alerting

## Architecture

The system consists of several components:

- **EnhancedPerformanceAnalyzer**: Core performance testing engine
- **WorkspaceMetricsCollector**: Multi-crate performance metrics collection
- **Automated baseline tracking**: Continuous performance monitoring
- **Regression detection**: Threshold-based alerting system

## Configuration Files

### `performance_baseline_config.json`

Main configuration file containing:

```json
{
  "monitoring_settings": {
    "enabled": true,
    "monitoring_integration": true,
    "alert_on_regression": true,
    "regression_threshold_percent": 5.0,
    "enable_baseline_comparison": true
  },
  "performance_targets": {
    "sync_workload_ops_per_second": {
      "target": 10000,
      "warning_threshold": 8000,
      "critical_threshold": 5000
    }
  }
}
```

### `performance_baseline_data.json`

Contains current baseline data and historical performance metrics:

```json
{
  "performance_baselines": {
    "sync_workload": {
      "avg_ops_per_second": 12500.0,
      "last_updated": "2025-09-16T15:28:52.212Z",
      "sample_count": 1,
      "baseline_history": [...]
    }
  }
}
```

## Running Performance Tests

### Manual Testing

Run individual performance tests using the Enhanced Performance Analyzer:

```bash
# Run from src-tauri directory
cd src-tauri
cargo +nightly test test_enhanced_performance_analyzer_creation -- --nocapture
```

### Automated Baseline Updates

Use the automated script to update baselines:

```bash
# From project root
./scripts/update_performance_baselines.sh
```

This script will:
1. Run performance tests
2. Update baseline data using moving averages
3. Check for regressions
4. Generate reports
5. Store historical data

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Performance Monitoring
on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  performance-test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Setup Rust
      uses: dtolnay/rust-toolchain@nightly
    - name: Install dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y jq bc
    - name: Run performance baseline update
      run: ./scripts/update_performance_baselines.sh
    - name: Upload performance report
      uses: actions/upload-artifact@v3
      with:
        name: performance-report
        path: performance_report_*.md
    - name: Check for regressions
      run: |
        if jq -e '.regression_alerts.alert_history[-1]' performance_baseline_data.json > /dev/null; then
          echo "Performance regression detected!"
          exit 1
        fi
```

### Jenkins Pipeline Example

```groovy
pipeline {
    agent any
    stages {
        stage('Performance Test') {
            steps {
                sh './scripts/update_performance_baselines.sh'
            }
            post {
                always {
                    archiveArtifacts artifacts: 'performance_report_*.md', fingerprint: true
                }
                failure {
                    script {
                        def regressionData = readJSON file: 'performance_baseline_data.json'
                        if (regressionData.regression_alerts?.alert_history?.size() > 0) {
                            echo "Performance regression detected!"
                            currentBuild.result = 'UNSTABLE'
                        }
                    }
                }
            }
        }
    }
}
```

## Interpreting Results

### Performance Metrics

- **Operations/second**: Higher is better
- **Memory usage**: Lower is better
- **Build time**: Lower is better
- **Regression threshold**: Configurable percentage (default 5%)

### Baseline Comparison

The system uses moving averages to track baseline performance:

```
New Baseline = Old Baseline × (1 - α) + New Measurement × α
```

Where α (learning rate) defaults to 0.1.

### Regression Detection

Regressions are detected when current performance falls below:

```
Baseline × (1 - threshold/100)
```

## Alert Configuration

### Thresholds

Configure alert thresholds in `performance_baseline_config.json`:

```json
{
  "performance_targets": {
    "sync_workload_ops_per_second": {
      "target": 10000,
      "warning_threshold": 8000,
      "critical_threshold": 5000
    }
  }
}
```

### Alert Channels

Configure notification channels:

```json
{
  "alert_configuration": {
    "email_alerts": {
      "enabled": true,
      "recipients": ["dev-team@company.com"]
    },
    "slack_alerts": {
      "enabled": true,
      "webhook_url": "https://hooks.slack.com/...",
      "channel": "#performance-alerts"
    }
  }
}
```

## Troubleshooting

### Common Issues

1. **Compilation failures in performance tests**
   - Ensure nightly Rust toolchain is installed
   - Check for missing dependencies
   - Verify workspace configuration

2. **Baseline file not found**
   - Run initial baseline establishment
   - Check file permissions
   - Verify JSON syntax

3. **False positive regressions**
   - Adjust regression threshold
   - Increase sample count for stable baselines
   - Review system load during tests

### Debug Mode

Enable debug logging:

```bash
export RUST_LOG=debug
./scripts/update_performance_baselines.sh
```

## Best Practices

### Test Environment Consistency

- Use dedicated performance testing machines
- Maintain consistent system load
- Control environmental variables
- Use same hardware configurations

### Baseline Management

- Establish baselines on clean systems
- Update baselines gradually
- Review baseline changes manually
- Archive historical data

### Monitoring Strategy

- Run performance tests daily
- Monitor trends over time
- Set appropriate alert thresholds
- Balance sensitivity with false positives

## Advanced Usage

### Custom Performance Tests

Create custom performance tests by extending `EnhancedPerformanceAnalyzer`:

```rust
impl EnhancedPerformanceAnalyzer {
    pub async fn run_custom_workload(&mut self, workload_fn: fn() -> u64) -> anyhow::Result<EnhancedPerformanceTestResult> {
        // Custom workload implementation
    }
}
```

### Integration with External Monitoring

Integrate with external monitoring systems:

```rust
// Send metrics to monitoring dashboard
pub async fn export_to_monitoring_dashboard(&self, metrics: &PerformanceMetrics) -> anyhow::Result<()> {
    // Implementation for external monitoring integration
}
```

## API Reference

### EnhancedPerformanceAnalyzer

- `run_enhanced_performance_test_suite()` - Run full performance test suite
- `collect_workspace_metrics()` - Collect metrics across workspace
- `update_baseline()` - Update baseline data
- `check_and_alert_regressions()` - Check for and alert on regressions

### Test Utilities

- `run_sync_performance_workload()` - Run CPU-bound performance test
- `run_async_performance_workload()` - Run I/O-bound performance test
- `calculate_ops_per_second()` - Calculate performance metric

## Contributing

When adding new performance tests:

1. Follow the existing test patterns
2. Update baseline configuration
3. Add appropriate thresholds
4. Document new metrics
5. Update CI/CD pipelines

## Support

For issues with performance monitoring:

1. Check the troubleshooting guide
2. Review recent baseline changes
3. Examine system resource usage
4. Contact the development team

---

*This documentation is maintained alongside the performance monitoring codebase. Please update both when making changes.*
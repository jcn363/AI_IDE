#!/bin/bash

# Performance Monitoring CI/CD Integration Script
#
# This script integrates continuous performance monitoring into CI/CD pipelines
# Supports Prometheus metrics collection, alerting, and iterative tuning

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PERFORMANCE_CRATE_DIR="$PROJECT_ROOT/crates/rust-ai-ide-performance"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Default configuration
METRICS_SERVER_PORT="${METRICS_SERVER_PORT:-9090}"
METRICS_SERVER_HOST="${METRICS_SERVER_HOST:-127.0.0.1}"
COLLECTION_INTERVAL="${COLLECTION_INTERVAL:-30}"
ENABLE_PROMETHEUS="${ENABLE_PROMETHEUS:-true}"
ENABLE_ALERTING="${ENABLE_ALERTING:-true}"
ENABLE_INSTRUMENTATION="${ENABLE_INSTRUMENTATION:-true}"

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Setup performance monitoring environment
setup_performance_monitoring() {
    log_info "Setting up performance monitoring environment..."

    # Create necessary directories
    mkdir -p "$PROJECT_ROOT/performance-data"
    mkdir -p "$PROJECT_ROOT/performance-reports"

    # Export environment variables for the performance monitoring system
    export RUST_AI_IDE_METRICS_PORT="$METRICS_SERVER_PORT"
    export RUST_AI_IDE_METRICS_HOST="$METRICS_SERVER_HOST"
    export RUST_AI_IDE_COLLECTION_INTERVAL="$COLLECTION_INTERVAL"
    export RUST_AI_IDE_ENABLE_PROMETHEUS="$ENABLE_PROMETHEUS"
    export RUST_AI_IDE_ENABLE_ALERTING="$ENABLE_ALERTING"
    export RUST_AI_IDE_ENABLE_INSTRUMENTATION="$ENABLE_INSTRUMENTATION"

    # Set up Prometheus configuration if enabled
    if [ "$ENABLE_PROMETHEUS" = "true" ]; then
        setup_prometheus_config
    fi

    log_success "Performance monitoring environment setup completed"
}

# Setup Prometheus configuration
setup_prometheus_config() {
    log_info "Setting up Prometheus configuration..."

    local prometheus_config_dir="$PROJECT_ROOT/prometheus-config"
    mkdir -p "$prometheus_config_dir"

    # Create prometheus.yml configuration
    cat > "$prometheus_config_dir/prometheus.yml" << EOF
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  # - "first_rules.yml"
  # - "second_rules.yml"

scrape_configs:
  - job_name: 'rust-ai-ide'
    static_configs:
      - targets: ['$METRICS_SERVER_HOST:$METRICS_SERVER_PORT']
    scrape_interval: ${COLLECTION_INTERVAL}s
    metrics_path: '/metrics'
EOF

    log_success "Prometheus configuration created at: $prometheus_config_dir/prometheus.yml"
}

# Build performance monitoring components
build_performance_monitoring() {
    log_info "Building performance monitoring components..."

    # Build the performance crate with all features
    cd "$PROJECT_ROOT"
    cargo build --release -p rust-ai-ide-performance

    if [ $? -eq 0 ]; then
        log_success "Performance monitoring components built successfully"
    else
        log_error "Failed to build performance monitoring components"
        exit 1
    fi
}

# Start metrics server
start_metrics_server() {
    log_info "Starting metrics server..."

    # Build and run the metrics server (in background)
    cd "$PERFORMANCE_CRATE_DIR"

    # Create a simple test program to start the metrics server
    cat > "$PROJECT_ROOT/start_metrics_server.rs" << 'EOF'
use rust_ai_ide_performance::{
    collector::{CollectorBuilder, UnifiedPerformanceCollector},
    metrics::{MetricsRegistry, PrometheusExporter},
    metrics_server::{MetricsServerBuilder, MetricsServerConfig},
    instrumentation::{PerformanceInstrumentor, InstrumentationConfig},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Rust AI IDE Performance Monitoring Server...");

    // Create collector
    let collector = Arc::new(CollectorBuilder::new().build());

    // Create metrics registry
    let registry = Arc::new(MetricsRegistry::new());

    // Create Prometheus exporter
    let exporter = Arc::new(PrometheusExporter::new(Arc::clone(&collector)));

    // Initialize default metrics
    exporter.initialize_default_metrics().await?;

    // Create metrics server
    let server_config = MetricsServerConfig {
        address: std::env::var("RUST_AI_IDE_METRICS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
        port: std::env::var("RUST_AI_IDE_METRICS_PORT")
            .unwrap_or_else(|_| "9090".to_string())
            .parse()
            .unwrap_or(9090),
        metrics_path: "/metrics".to_string(),
        enable_health_check: true,
        health_check_path: "/health".to_string(),
    };

    let server = MetricsServerBuilder::new()
        .with_config(server_config)
        .build(exporter);

    println!("Metrics server starting on {}:{}", server_config.address, server_config.port);
    println!("Metrics endpoint: http://{}:{}{}", server_config.address, server_config.port, server_config.metrics_path);
    println!("Health endpoint: http://{}:{}{}", server_config.address, server_config.port, server_config.health_check_path);

    // Start the server
    server.start().await?;

    Ok(())
}
EOF

    # Build and run the server
    rustc --edition 2021 -L "$PROJECT_ROOT/target/release/deps" \
        --extern rust_ai_ide_performance="$PROJECT_ROOT/target/release/librust_ai_ide_performance.rlib" \
        -L "$PROJECT_ROOT/target/release/deps" \
        --extern tokio="$PROJECT_ROOT/target/release/deps/libtokio-*.rlib" \
        --extern serde="$PROJECT_ROOT/target/release/deps/libserde-*.rlib" \
        --extern serde_json="$PROJECT_ROOT/target/release/deps/libserde_json-*.rlib" \
        --extern chrono="$PROJECT_ROOT/target/release/deps/libchrono-*.rlib" \
        "$PROJECT_ROOT/start_metrics_server.rs" -o "$PROJECT_ROOT/metrics_server"

    # Start server in background
    nohup "$PROJECT_ROOT/metrics_server" > "$PROJECT_ROOT/performance-data/server.log" 2>&1 &
    METRICS_SERVER_PID=$!

    # Wait a bit for server to start
    sleep 3

    # Check if server is running
    if kill -0 $METRICS_SERVER_PID 2>/dev/null; then
        log_success "Metrics server started successfully (PID: $METRICS_SERVER_PID)"
        echo $METRICS_SERVER_PID > "$PROJECT_ROOT/performance-data/server.pid"
    else
        log_error "Failed to start metrics server"
        cat "$PROJECT_ROOT/performance-data/server.log"
        exit 1
    fi
}

# Run performance tests with monitoring
run_performance_tests_with_monitoring() {
    log_info "Running performance tests with monitoring..."

    local test_results="$PROJECT_ROOT/performance-data/test-results.json"

    # Run existing performance tests
    if [ -f "$PROJECT_ROOT/scripts/run-performance-tests.js" ]; then
        log_info "Running JavaScript performance tests..."
        node "$PROJECT_ROOT/scripts/run-performance-tests.js" \
            --output-dir "$PROJECT_ROOT/performance-data" \
            --verbose
    fi

    # Run Rust performance tests
    if [ -f "$PROJECT_ROOT/test-performance-analysis.rs" ]; then
        log_info "Running Rust performance analysis..."
        rustc --edition 2021 "$PROJECT_ROOT/test-performance-analysis.rs" -o "$PROJECT_ROOT/perf_analysis"
        "$PROJECT_ROOT/perf_analysis"
    fi

    log_success "Performance tests completed"
}

# Generate performance reports
generate_performance_reports() {
    log_info "Generating performance reports..."

    local reports_dir="$PROJECT_ROOT/performance-reports"
    local timestamp=$(date +%Y%m%d_%H%M%S)

    # Create performance summary report
    cat > "$reports_dir/performance_summary_$timestamp.md" << EOF
# Performance Monitoring Report

Generated: $(date)
CI Build: ${CI_BUILD_NUMBER:-N/A}
Commit: ${GITHUB_SHA:-${GIT_COMMIT:-$(git rev-parse HEAD 2>/dev/null)}}

## Configuration
- Metrics Server: $METRICS_SERVER_HOST:$METRICS_SERVER_PORT
- Collection Interval: ${COLLECTION_INTERVAL}s
- Prometheus Enabled: $ENABLE_PROMETHEUS
- Alerting Enabled: $ENABLE_ALERTING
- Instrumentation Enabled: $ENABLE_INSTRUMENTATION

## Metrics Endpoints
- Metrics: http://$METRICS_SERVER_HOST:$METRICS_SERVER_PORT/metrics
- Health: http://$METRICS_SERVER_HOST:$METRICS_SERVER_PORT/health

## Test Results

### Performance Tests
$(if [ -f "$PROJECT_ROOT/performance-data/test-results.json" ]; then
    echo "✅ Performance tests executed"
    echo "📊 Results: $PROJECT_ROOT/performance-data/test-results.json"
else
    echo "❌ No performance test results found"
fi)

### Server Status
$(if [ -f "$PROJECT_ROOT/performance-data/server.pid" ]; then
    local pid=$(cat "$PROJECT_ROOT/performance-data/server.pid")
    if kill -0 $pid 2>/dev/null; then
        echo "✅ Metrics server running (PID: $pid)"
    else
        echo "❌ Metrics server not running"
    fi
else
    echo "❌ Metrics server not started"
fi)

## Recommendations
1. Monitor the metrics endpoint for real-time performance data
2. Set up Prometheus scraping for historical analysis
3. Configure alerts based on performance thresholds
4. Review instrumentation data for optimization opportunities

EOF

    log_success "Performance report generated: $reports_dir/performance_summary_$timestamp.md"
}

# Cleanup performance monitoring
cleanup_performance_monitoring() {
    log_info "Cleaning up performance monitoring..."

    # Stop metrics server if running
    if [ -f "$PROJECT_ROOT/performance-data/server.pid" ]; then
        local pid=$(cat "$PROJECT_ROOT/performance-data/server.pid")
        if kill -0 $pid 2>/dev/null; then
            log_info "Stopping metrics server (PID: $pid)..."
            kill $pid
            sleep 2
            if kill -0 $pid 2>/dev/null; then
                kill -9 $pid
            fi
        fi
        rm -f "$PROJECT_ROOT/performance-data/server.pid"
    fi

    # Clean up temporary files
    rm -f "$PROJECT_ROOT/start_metrics_server.rs"
    rm -f "$PROJECT_ROOT/metrics_server"

    log_success "Performance monitoring cleanup completed"
}

# Main execution
main() {
    log_info "=== Rust AI IDE Performance Monitoring CI/CD Integration ==="

    # Setup
    setup_performance_monitoring

    # Build components
    build_performance_monitoring

    # Start monitoring
    start_metrics_server

    # Run tests with monitoring
    run_performance_tests_with_monitoring

    # Generate reports
    generate_performance_reports

    # Export results for CI systems
    export_ci_results

    log_success "Performance monitoring integration completed"

    # Keep server running for CI systems that may need it
    if [ "${KEEP_SERVER_RUNNING:-false}" = "true" ]; then
        log_info "Keeping metrics server running for CI pipeline..."
        log_info "Server PID: $(cat $PROJECT_ROOT/performance-data/server.pid 2>/dev/null || echo 'N/A')"
        log_info "Metrics URL: http://$METRICS_SERVER_HOST:$METRICS_SERVER_PORT/metrics"
        # Don't call cleanup, let CI system handle it
        exit 0
    fi

    # Cleanup (unless explicitly disabled)
    if [ "${SKIP_CLEANUP:-false}" != "true" ]; then
        cleanup_performance_monitoring
    fi
}

# Export results for CI systems
export_ci_results() {
    # GitHub Actions
    if [ -n "$GITHUB_ACTIONS" ]; then
        echo "performance_report=$PROJECT_ROOT/performance-reports/performance_summary_$(date +%Y%m%d_%H%M%S).md" >> $GITHUB_OUTPUT
        echo "metrics_endpoint=http://$METRICS_SERVER_HOST:$METRICS_SERVER_PORT/metrics" >> $GITHUB_OUTPUT
    fi

    # Jenkins
    if [ -n "$JENKINS_HOME" ]; then
        echo "PERFORMANCE_REPORT=$PROJECT_ROOT/performance-reports/performance_summary_$(date +%Y%m%d_%H%M%S).md"
        echo "METRICS_ENDPOINT=http://$METRICS_SERVER_HOST:$METRICS_SERVER_PORT/metrics"
    fi

    # Azure DevOps
    if [ -n "$TF_BUILD" ]; then
        echo "##vso[task.setvariable variable=performanceReport]$PROJECT_ROOT/performance-reports/performance_summary_$(date +%Y%m%d_%H%M%S).md"
        echo "##vso[task.setvariable variable=metricsEndpoint]http://$METRICS_SERVER_HOST:$METRICS_SERVER_PORT/metrics"
    fi
}

# Error handling
trap 'log_error "Performance monitoring failed"; cleanup_performance_monitoring; exit 1' ERR

# Run main function
main "$@"
#!/bin/bash

# Performance Monitoring Dashboard Setup Script
#
# This script sets up comprehensive dashboards for performance monitoring
# Supports Grafana and custom dashboards with real-time metrics visualization

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DASHBOARD_DIR="$PROJECT_ROOT/performance-dashboards"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Default configuration
GRAFANA_ENABLED="${GRAFANA_ENABLED:-true}"
PROMETHEUS_URL="${PROMETHEUS_URL:-http://localhost:9090}"
METRICS_URL="${METRICS_URL:-http://localhost:9090/metrics}"
DASHBOARD_PORT="${DASHBOARD_PORT:-3001}"

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

# Create dashboard directory structure
create_dashboard_structure() {
    log_info "Creating dashboard directory structure..."

    mkdir -p "$DASHBOARD_DIR"
    mkdir -p "$DASHBOARD_DIR/grafana"
    mkdir -p "$DASHBOARD_DIR/grafana/dashboards"
    mkdir -p "$DASHBOARD_DIR/grafana/datasources"
    mkdir -p "$DASHBOARD_DIR/custom"
    mkdir -p "$DASHBOARD_DIR/custom/css"
    mkdir -p "$DASHBOARD_DIR/custom/js"
    mkdir -p "$DASHBOARD_DIR/custom/templates"

    log_success "Dashboard structure created at: $DASHBOARD_DIR"
}

# Setup Grafana configuration
setup_grafana_config() {
    if [ "$GRAFANA_ENABLED" != "true" ]; then
        log_info "Grafana integration disabled"
        return 0
    fi

    log_info "Setting up Grafana configuration..."

    # Create Grafana datasource configuration
    cat > "$DASHBOARD_DIR/grafana/datasources/prometheus.yml" << EOF
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: $PROMETHEUS_URL
    isDefault: true
    editable: true
EOF

    # Create comprehensive Rust AI IDE performance dashboard
    create_grafana_dashboard

    log_success "Grafana configuration created"
}

# Create Grafana dashboard
create_grafana_dashboard() {
    cat > "$DASHBOARD_DIR/grafana/dashboards/rust-ai-ide-performance.json" << 'EOF'
{
  "dashboard": {
    "id": null,
    "title": "Rust AI IDE Performance Monitoring",
    "tags": ["rust", "ai", "ide", "performance"],
    "timezone": "browser",
    "panels": [
      {
        "id": 1,
        "title": "CPU Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(rust_ai_ide_cpu_usage_percent[5m])",
            "legendFormat": "CPU Usage %"
          }
        ],
        "gridPos": {
          "h": 8,
          "w": 12,
          "x": 0,
          "y": 0
        }
      },
      {
        "id": 2,
        "title": "Memory Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "rust_ai_ide_memory_usage_bytes",
            "legendFormat": "Memory Usage (bytes)"
          }
        ],
        "gridPos": {
          "h": 8,
          "w": 12,
          "x": 12,
          "y": 0
        }
      },
      {
        "id": 3,
        "title": "Response Time",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(rust_ai_ide_response_time_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          },
          {
            "expr": "histogram_quantile(0.50, rate(rust_ai_ide_response_time_seconds_bucket[5m]))",
            "legendFormat": "50th percentile"
          }
        ],
        "gridPos": {
          "h": 8,
          "w": 12,
          "x": 0,
          "y": 8
        }
      },
      {
        "id": 4,
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(rust_ai_ide_requests_total[5m])",
            "legendFormat": "Requests/second"
          }
        ],
        "gridPos": {
          "h": 8,
          "w": 12,
          "x": 12,
          "y": 8
        }
      },
      {
        "id": 5,
        "title": "Performance Alerts",
        "type": "table",
        "targets": [
          {
            "expr": "ALERTS{alertname=~\"rust_ai_ide.*\"}",
            "legendFormat": "{{alertname}}"
          }
        ],
        "gridPos": {
          "h": 8,
          "w": 24,
          "x": 0,
          "y": 16
        }
      }
    ],
    "time": {
      "from": "now-1h",
      "to": "now"
    },
    "timepicker": {},
    "templating": {
      "list": []
    },
    "annotations": {
      "list": []
    },
    "refresh": "5s",
    "schemaVersion": 27,
    "version": 0,
    "links": []
  }
}
EOF
}

# Create custom dashboard
create_custom_dashboard() {
    log_info "Creating custom performance dashboard..."

    # Create HTML dashboard
    cat > "$DASHBOARD_DIR/custom/index.html" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rust AI IDE Performance Dashboard</title>
    <link rel="stylesheet" href="css/dashboard.css">
</head>
<body>
    <header>
        <h1>Rust AI IDE Performance Dashboard</h1>
        <div id="status">Connecting...</div>
    </header>

    <main>
        <section id="metrics-overview">
            <h2>System Metrics</h2>
            <div class="metric-grid">
                <div class="metric-card" id="cpu-card">
                    <h3>CPU Usage</h3>
                    <div class="metric-value" id="cpu-value">--</div>
                    <canvas id="cpu-chart" width="300" height="150"></canvas>
                </div>
                <div class="metric-card" id="memory-card">
                    <h3>Memory Usage</h3>
                    <div class="metric-value" id="memory-value">--</div>
                    <canvas id="memory-chart" width="300" height="150"></canvas>
                </div>
                <div class="metric-card" id="requests-card">
                    <h3>Request Rate</h3>
                    <div class="metric-value" id="requests-value">--</div>
                    <canvas id="requests-chart" width="300" height="150"></canvas>
                </div>
            </div>
        </section>

        <section id="performance-analysis">
            <h2>Performance Analysis</h2>
            <div class="analysis-grid">
                <div class="analysis-card">
                    <h3>Response Time Distribution</h3>
                    <canvas id="response-time-chart" width="400" height="200"></canvas>
                </div>
                <div class="analysis-card">
                    <h3>Recent Alerts</h3>
                    <div id="alerts-list">No recent alerts</div>
                </div>
            </div>
        </section>

        <section id="instrumentation-data">
            <h2>Instrumentation Data</h2>
            <div id="instrumentation-table">
                <table>
                    <thead>
                        <tr>
                            <th>Operation</th>
                            <th>Count</th>
                            <th>Avg Duration</th>
                            <th>Min Duration</th>
                            <th>Max Duration</th>
                        </tr>
                    </thead>
                    <tbody id="instrumentation-body">
                        <tr><td colspan="5">Loading...</td></tr>
                    </tbody>
                </table>
            </div>
        </section>
    </main>

    <script src="js/dashboard.js"></script>
</body>
</html>
EOF

    # Create CSS
    cat > "$DASHBOARD_DIR/custom/css/dashboard.css" << 'EOF'
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: #1a1a1a;
    color: #e0e0e0;
    line-height: 1.6;
}

header {
    background: #2d2d2d;
    padding: 1rem 2rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid #404040;
}

header h1 {
    color: #00d4aa;
}

#status {
    padding: 0.5rem 1rem;
    border-radius: 4px;
    font-weight: bold;
}

#status.connecting { background: #ffa500; color: black; }
#status.connected { background: #00d4aa; color: black; }
#status.error { background: #ff4444; color: white; }

main {
    padding: 2rem;
    max-width: 1400px;
    margin: 0 auto;
}

section {
    margin-bottom: 3rem;
}

h2 {
    color: #00d4aa;
    margin-bottom: 1.5rem;
    border-bottom: 2px solid #00d4aa;
    padding-bottom: 0.5rem;
}

.metric-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 2rem;
}

.metric-card {
    background: #2d2d2d;
    border-radius: 8px;
    padding: 1.5rem;
    border: 1px solid #404040;
}

.metric-card h3 {
    color: #00d4aa;
    margin-bottom: 1rem;
}

.metric-value {
    font-size: 2.5rem;
    font-weight: bold;
    text-align: center;
    margin: 1rem 0;
    color: #ffffff;
}

.analysis-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2rem;
}

.analysis-card {
    background: #2d2d2d;
    border-radius: 8px;
    padding: 1.5rem;
    border: 1px solid #404040;
}

.analysis-card h3 {
    color: #00d4aa;
    margin-bottom: 1rem;
}

#instrumentation-table {
    background: #2d2d2d;
    border-radius: 8px;
    overflow: hidden;
    border: 1px solid #404040;
}

#instrumentation-table table {
    width: 100%;
    border-collapse: collapse;
}

#instrumentation-table th,
#instrumentation-table td {
    padding: 1rem;
    text-align: left;
    border-bottom: 1px solid #404040;
}

#instrumentation-table th {
    background: #404040;
    color: #00d4aa;
    font-weight: bold;
}

#instrumentation-table tr:hover {
    background: #333333;
}
EOF

    # Create JavaScript
    cat > "$DASHBOARD_DIR/custom/js/dashboard.js" << EOF
// Rust AI IDE Performance Dashboard JavaScript
const METRICS_URL = '$METRICS_URL';
const UPDATE_INTERVAL = 5000; // 5 seconds

class PerformanceDashboard {
    constructor() {
        this.charts = {};
        this.data = {
            cpu: [],
            memory: [],
            requests: [],
            responseTime: []
        };
        this.maxDataPoints = 50;
        this.statusElement = document.getElementById('status');
        this.initializeCharts();
        this.startUpdates();
    }

    initializeCharts() {
        // Initialize Chart.js charts
        const chartOptions = {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                y: {
                    beginAtZero: true,
                    grid: {
                        color: '#404040'
                    },
                    ticks: {
                        color: '#e0e0e0'
                    }
                },
                x: {
                    grid: {
                        color: '#404040'
                    },
                    ticks: {
                        color: '#e0e0e0'
                    }
                }
            },
            plugins: {
                legend: {
                    labels: {
                        color: '#e0e0e0'
                    }
                }
            }
        };

        this.charts.cpu = this.createChart('cpu-chart', 'CPU Usage %');
        this.charts.memory = this.createChart('memory-chart', 'Memory (MB)');
        this.charts.requests = this.createChart('requests-chart', 'Requests/sec');
        this.charts.responseTime = this.createChart('response-time-chart', 'Response Time (ms)');
    }

    createChart(canvasId, label) {
        const ctx = document.getElementById(canvasId).getContext('2d');
        return new Chart(ctx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [{
                    label: label,
                    data: [],
                    borderColor: '#00d4aa',
                    backgroundColor: 'rgba(0, 212, 170, 0.1)',
                    tension: 0.4
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true,
                        grid: {
                            color: '#404040'
                        },
                        ticks: {
                            color: '#e0e0e0'
                        }
                    },
                    x: {
                        grid: {
                            color: '#404040'
                        },
                        ticks: {
                            color: '#e0e0e0'
                        }
                    }
                },
                plugins: {
                    legend: {
                        labels: {
                            color: '#e0e0e0'
                        }
                    }
                }
            }
        });
    }

    async updateMetrics() {
        try {
            const response = await fetch(METRICS_URL);
            if (!response.ok) throw new Error('Failed to fetch metrics');
            
            const metricsText = await response.text();
            const metrics = this.parsePrometheusMetrics(metricsText);
            
            this.updateCharts(metrics);
            this.updateValues(metrics);
            
            this.statusElement.textContent = 'Connected';
            this.statusElement.className = 'connected';
            
        } catch (error) {
            console.error('Failed to update metrics:', error);
            this.statusElement.textContent = 'Error: ' + error.message;
            this.statusElement.className = 'error';
        }
    }

    parsePrometheusMetrics(metricsText) {
        const metrics = {};
        const lines = metricsText.split('\n');
        
        for (const line of lines) {
            if (line.startsWith('#') || line.trim() === '') continue;
            
            const [nameValue, timestamp] = line.split(' ');
            const [name, value] = nameValue.split(' ');
            
            if (name && value) {
                metrics[name] = parseFloat(value);
            }
        }
        
        return metrics;
    }

    updateCharts(metrics) {
        const timestamp = new Date().toLocaleTimeString();
        
        // Update CPU chart
        if (metrics.rust_ai_ide_cpu_usage_percent !== undefined) {
            this.updateChartData(this.charts.cpu, timestamp, metrics.rust_ai_ide_cpu_usage_percent);
        }
        
        // Update Memory chart
        if (metrics.rust_ai_ide_memory_usage_bytes !== undefined) {
            const memoryMB = metrics.rust_ai_ide_memory_usage_bytes / (1024 * 1024);
            this.updateChartData(this.charts.memory, timestamp, memoryMB);
        }
        
        // Update Requests chart (simplified)
        if (metrics.rust_ai_ide_requests_total !== undefined) {
            this.updateChartData(this.charts.requests, timestamp, metrics.rust_ai_ide_requests_total);
        }
    }

    updateChartData(chart, label, value) {
        chart.data.labels.push(label);
        chart.data.datasets[0].data.push(value);
        
        if (chart.data.labels.length > this.maxDataPoints) {
            chart.data.labels.shift();
            chart.data.datasets[0].data.shift();
        }
        
        chart.update();
    }

    updateValues(metrics) {
        // Update current values
        if (metrics.rust_ai_ide_cpu_usage_percent !== undefined) {
            document.getElementById('cpu-value').textContent = 
                metrics.rust_ai_ide_cpu_usage_percent.toFixed(1) + '%';
        }
        
        if (metrics.rust_ai_ide_memory_usage_bytes !== undefined) {
            const memoryMB = (metrics.rust_ai_ide_memory_usage_bytes / (1024 * 1024)).toFixed(1);
            document.getElementById('memory-value').textContent = memoryMB + ' MB';
        }
        
        if (metrics.rust_ai_ide_requests_total !== undefined) {
            document.getElementById('requests-value').textContent = 
                metrics.rust_ai_ide_requests_total.toFixed(0);
        }
    }

    startUpdates() {
        this.updateMetrics(); // Initial update
        setInterval(() => this.updateMetrics(), UPDATE_INTERVAL);
    }
}

// Initialize dashboard when page loads
document.addEventListener('DOMContentLoaded', () => {
    new PerformanceDashboard();
});
EOF

    log_success "Custom dashboard created"
}

# Start dashboard server
start_dashboard_server() {
    log_info "Starting dashboard server..."

    # Create a simple HTTP server for the custom dashboard
    cat > "$DASHBOARD_DIR/custom/server.js" << EOF
const express = require('express');
const path = require('path');
const app = express();
const port = $DASHBOARD_PORT;

// Serve static files
app.use(express.static(path.join(__dirname, '.')));

// Start server
app.listen(port, () => {
    console.log(\`Performance Dashboard running at http://localhost:\${port}\`);
});
EOF

    # Check if Node.js is available
    if command -v node >/dev/null 2>&1; then
        cd "$DASHBOARD_DIR/custom"
        nohup node server.js > "$DASHBOARD_DIR/dashboard-server.log" 2>&1 &
        DASHBOARD_SERVER_PID=$!

        # Wait a bit for server to start
        sleep 2

        if kill -0 $DASHBOARD_SERVER_PID 2>/dev/null; then
            log_success "Dashboard server started (PID: $DASHBOARD_SERVER_PID)"
            echo $DASHBOARD_SERVER_PID > "$DASHBOARD_DIR/dashboard-server.pid"
        else
            log_warning "Dashboard server failed to start"
        fi
    else
        log_warning "Node.js not found - dashboard server not started"
    fi
}

# Generate dashboard configuration summary
generate_dashboard_summary() {
    log_info "Generating dashboard configuration summary..."

    local summary_file="$DASHBOARD_DIR/dashboard-summary.md"

    cat > "$summary_file" << EOF
# Performance Dashboard Configuration

Generated: $(date)
Project: Rust AI IDE

## Dashboard URLs

### Custom Dashboard
- URL: http://localhost:$DASHBOARD_PORT
- Status: $(if [ -f "$DASHBOARD_DIR/dashboard-server.pid" ]; then echo "Running"; else echo "Not started"; fi)
- Files: $DASHBOARD_DIR/custom/

### Grafana Dashboard
- Configuration: $DASHBOARD_DIR/grafana/
- Datasource: $PROMETHEUS_URL
- Dashboard: $DASHBOARD_DIR/grafana/dashboards/rust-ai-ide-performance.json

### Metrics Endpoints
- Prometheus: $PROMETHEUS_URL
- Direct Metrics: $METRICS_URL

## Configuration Options

### Grafana Integration
- Enabled: $GRAFANA_ENABLED
- Import the dashboard JSON file into your Grafana instance
- Configure Prometheus datasource to point to: $PROMETHEUS_URL

### Custom Dashboard
- Served on port: $DASHBOARD_PORT
- Auto-refreshes every 5 seconds
- Real-time metrics visualization

## Setup Instructions

### For Grafana:
1. Import the dashboard JSON file
2. Configure Prometheus datasource
3. Adjust refresh intervals as needed

### For Custom Dashboard:
1. Ensure Node.js is installed
2. Dashboard server starts automatically
3. Access via web browser

## Metrics Available

- CPU Usage Percentage
- Memory Usage (bytes)
- Request Rate
- Response Time Distribution
- Performance Alerts
- Instrumentation Data

EOF

    log_success "Dashboard summary generated: $summary_file"
}

# Main execution
main() {
    log_info "=== Performance Dashboard Setup ==="

    # Create structure
    create_dashboard_structure

    # Setup Grafana
    setup_grafana_config

    # Create custom dashboard
    create_custom_dashboard

    # Start dashboard server
    start_dashboard_server

    # Generate summary
    generate_dashboard_summary

    log_success "Dashboard setup completed"

    # Display access information
    echo ""
    echo "=== Dashboard Access Information ==="
    echo "Custom Dashboard: http://localhost:$DASHBOARD_PORT"
    echo "Grafana Config: $DASHBOARD_DIR/grafana/"
    echo "Summary: $DASHBOARD_DIR/dashboard-summary.md"
    echo ""
}

# Run main function
main "\$@"
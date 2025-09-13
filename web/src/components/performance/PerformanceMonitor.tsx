import React from 'react';
import { invoke } from '@tauri-apps/api/core';

interface PerformanceSnapshot {
  timestamp: number;
  memoryUsage: number;
  memoryGrowth: number;
  activeOperations: number;
  cpuUsage: number;
  networkRequests: number;
  cacheHitRate: number;
  averageResponseTime: number;
}

interface PerformanceMetrics {
  totalUpTime: number;
  averageMemoryUsage: number;
  peakMemoryUsage: number;
  totalOperationsExecuted: number;
  averageOperationDuration: number;
  cacheEffectiveness: number;
  resourceUtilization: number;
  bottleneckDetection: string[];
}

interface OptimizationRecommendation {
  id: string;
  title: string;
  description: string;
  impact: 'high' | 'medium' | 'low';
  confidence: number;
  estimatedImprovement: number;
  implementationEffort: 'easy' | 'medium' | 'hard';
  affectedComponents: string[];
  action: () => Promise<void>;
}

interface PerformanceMonitorState {
  currentSnapshot: PerformanceSnapshot | null;
  historicalData: PerformanceSnapshot[];
  metrics: PerformanceMetrics | null;
  recommendations: OptimizationRecommendation[];
  alerts: Array<{
    id: string;
    level: 'critical' | 'warning' | 'info';
    message: string;
    timestamp: number;
    resolved?: boolean;
  }>;
  isMonitoring: boolean;
  analysisTimeframe: number; // hours
  alertThresholds: {
    highMemoryUsage: number;
    slowOperations: number;
    highErrorRate: number;
    cacheMissRate: number;
  };
}

class PerformanceMonitor extends React.Component<{}, PerformanceMonitorState> {
  private monitoringInterval: NodeJS.Timeout | null = null;
  private alertCheckInterval: NodeJS.Timeout | null = null;

  constructor(props: {}) {
    super(props);

    this.state = {
      currentSnapshot: null,
      historicalData: [],
      metrics: null,
      recommendations: [],
      alerts: [],
      isMonitoring: false,
      analysisTimeframe: 24, // Last 24 hours
      alertThresholds: {
        highMemoryUsage: 500, // MB
        slowOperations: 5000, // ms
        highErrorRate: 0.15, // 15%
        cacheMissRate: 0.4, // 40% miss rate
      },
    };
  }

  componentDidMount() {
    this.startMonitoring();
    this.loadHistoricalData();
    this.generateInitialRecommendations();
  }

  componentWillUnmount() {
    this.stopMonitoring();
  }

  private startMonitoring() {
    if (this.state.isMonitoring) return;

    this.setState({ isMonitoring: true });

    // Update metrics every 5 seconds
    this.monitoringInterval = setInterval(() => {
      this.collectPerformanceSnapshot();
    }, 5000);

    // Check for alerts every 30 seconds
    this.alertCheckInterval = setInterval(() => {
      this.checkPerformanceAlerts();
    }, 30000);
  }

  private stopMonitoring() {
    this.setState({ isMonitoring: false });

    if (this.monitoringInterval) {
      clearInterval(this.monitoringInterval);
      this.monitoringInterval = null;
    }
    if (this.alertCheckInterval) {
      clearInterval(this.alertCheckInterval);
      this.alertCheckInterval = null;
    }
  }

  private async loadHistoricalData() {
    try {
      const historical = await invoke<PerformanceSnapshot[]>('get_performance_history', {
        hours: this.state.analysisTimeframe,
      });
      this.setState({ historicalData: historical });
      this.calculateMetrics(historical);
    } catch (error) {
      console.error('Failed to load historical data:', error);
    }
  }

  private async collectPerformanceSnapshot() {
    try {
      const snapshot = await invoke<PerformanceSnapshot>('get_performance_snapshot');
      this.setState((prevState) => ({
        currentSnapshot: snapshot,
        historicalData: [...prevState.historicalData.slice(-100), snapshot], // Keep last 100 snapshots
      }));
    } catch (error) {
      console.error('Failed to collect performance snapshot:', error);
    }
  }

  private calculateMetrics(data: PerformanceSnapshot[]): PerformanceMetrics {
    if (data.length === 0) {
      return this.getDefaultMetrics();
    }

    const totalTime = data.length > 1 ? data[data.length - 1].timestamp - data[0].timestamp : 0;
    const memoryUsages = data.map((d) => d.memoryUsage);
    const responseTimes = data.map((d) => d.averageResponseTime);

    const metrics: PerformanceMetrics = {
      totalUpTime: totalTime,
      averageMemoryUsage: memoryUsages.reduce((a, b) => a + b, 0) / memoryUsages.length,
      peakMemoryUsage: Math.max(...memoryUsages),
      totalOperationsExecuted: data.reduce((sum, d) => sum + d.activeOperations, 0),
      averageOperationDuration: responseTimes.reduce((a, b) => a + b, 0) / responseTimes.length,
      cacheEffectiveness: data[data.length - 1]?.cacheHitRate || 0,
      resourceUtilization: Math.min(100, (memoryUsages[memoryUsages.length - 1] / 1000) * 100), // Rough estimate
      bottleneckDetection: this.detectBottlenecks(data),
    };

    this.setState({ metrics });
    return metrics;
  }

  private getDefaultMetrics(): PerformanceMetrics {
    return {
      totalUpTime: 0,
      averageMemoryUsage: 0,
      peakMemoryUsage: 0,
      totalOperationsExecuted: 0,
      averageOperationDuration: 0,
      cacheEffectiveness: 0,
      resourceUtilization: 0,
      bottleneckDetection: [],
    };
  }

  private detectBottlenecks(data: PerformanceSnapshot[]): string[] {
    const bottlenecks: string[] = [];
    const recentData = data.slice(-10); // Last 10 snapshots

    if (recentData.length < 2) return bottlenecks;

    const avgMemory = recentData.reduce((sum, d) => sum + d.memoryUsage, 0) / recentData.length;
    const avgResponseTime =
      recentData.reduce((sum, d) => sum + d.averageResponseTime, 0) / recentData.length;
    const avgCacheHitRate =
      recentData.reduce((sum, d) => sum + d.cacheHitRate, 0) / recentData.length;

    if (avgMemory > this.state.alertThresholds.highMemoryUsage) {
      bottlenecks.push('High Memory Utilization');
    }

    if (avgResponseTime > this.state.alertThresholds.slowOperations) {
      bottlenecks.push('Slow Operation Response Times');
    }

    if (avgCacheHitRate < 1 - this.state.alertThresholds.cacheMissRate) {
      bottlenecks.push('Low Cache Effectiveness');
    }

    // Detect memory leaks
    const memoryGrowth = recentData[recentData.length - 1].memoryUsage - recentData[0].memoryUsage;
    if (memoryGrowth > 50) {
      // Growing by more than 50MB
      bottlenecks.push('Potential Memory Leak');
    }

    return bottlenecks;
  }

  private async checkPerformanceAlerts() {
    if (!this.state.currentSnapshot || !this.state.metrics) return;

    const snapshot = this.state.currentSnapshot;
    const thresholds = this.state.alertThresholds;

    const newAlerts = [];

    // Memory usage alert
    if (snapshot.memoryUsage > thresholds.highMemoryUsage) {
      newAlerts.push({
        id: `memory_${Date.now()}`,
        level: 'warning' as const,
        message: `High memory usage: ${snapshot.memoryUsage.toFixed(1)} MB (threshold: ${thresholds.highMemoryUsage} MB)`,
        timestamp: Date.now(),
      });
    }

    // Response time alert
    if (snapshot.averageResponseTime > thresholds.slowOperations) {
      newAlerts.push({
        id: `response_${Date.now()}`,
        level: 'warning' as const,
        message: `Slow response times: ${snapshot.averageResponseTime.toFixed(0)}ms (threshold: ${thresholds.slowOperations}ms)`,
        timestamp: Date.now(),
      });
    }

    // Cache effectiveness alert
    if (snapshot.cacheHitRate < 1 - thresholds.cacheMissRate) {
      newAlerts.push({
        id: `cache_${Date.now()}`,
        level: 'info' as const,
        message: `Low cache hit rate: ${(snapshot.cacheHitRate * 100).toFixed(1)}%`,
        timestamp: Date.now(),
      });
    }

    // Add to existing alerts (keep resolved ones but mark as resolved if condition cleared)
    this.setState((prevState) => ({
      alerts: [...prevState.alerts, ...newAlerts].slice(-50), // Keep last 50 alerts
    }));
  }

  private generateInitialRecommendations(): void {
    const recommendations: OptimizationRecommendation[] = [
      {
        id: 'memory_pooling',
        title: 'Enable Memory Pooling',
        description: 'Implement memory pooling for frequent allocations to reduce GC pressure',
        impact: 'high' as const,
        confidence: 0.85,
        estimatedImprovement: 25,
        implementationEffort: 'medium' as const,
        affectedComponents: ['refactoring-engine', 'ast-processor'],
        action: async () => {
          await invoke('enable_memory_pooling');
          this.forceUpdate();
        },
      },
      {
        id: 'cache_optimization',
        title: 'Optimize Cache Strategy',
        description: 'Implement hierarchical caching with TTL optimization for improved hit rates',
        impact: 'medium' as const,
        confidence: 0.75,
        estimatedImprovement: 15,
        implementationEffort: 'easy' as const,
        affectedComponents: ['lsp-cache', 'analysis-engine'],
        action: async () => {
          await invoke('optimize_cache_strategy');
          this.forceUpdate();
        },
      },
      {
        id: 'parallel_processing',
        title: 'Enable Parallel Processing',
        description: 'Process independent refactoring operations in parallel to improve throughput',
        impact: 'high' as const,
        confidence: 0.9,
        estimatedImprovement: 40,
        implementationEffort: 'medium' as const,
        affectedComponents: ['batch-orchestrator', 'operation-scheduler'],
        action: async () => {
          await invoke('enable_parallel_processing');
          this.forceUpdate();
        },
      },
      {
        id: 'precompiling',
        title: 'Implement Precompiling',
        description: 'Precompile frequently used patterns and templates for faster response times',
        impact: 'medium' as const,
        confidence: 0.7,
        estimatedImprovement: 18,
        implementationEffort: 'hard' as const,
        affectedComponents: ['template-manager', 'pattern-recognizer'],
        action: async () => {
          await invoke('implement_precompiling');
          this.forceUpdate();
        },
      },
      {
        id: 'lazy_loading',
        title: 'Implement Lazy Loading',
        description: 'Load components and resources on-demand to improve startup performance',
        impact: 'low' as const,
        confidence: 0.65,
        estimatedImprovement: 12,
        implementationEffort: 'easy' as const,
        affectedComponents: ['ui-loader', 'component-initializer'],
        action: async () => {
          await invoke('implement_lazy_loading');
          this.forceUpdate();
        },
      },
    ];

    this.setState({ recommendations });
  }

  private resolveAlert(alertId: string) {
    this.setState((prevState) => ({
      alerts: prevState.alerts.map((alert) =>
        alert.id === alertId ? { ...alert, resolved: true } : alert
      ),
    }));
  }

  render() {
    const {
      currentSnapshot,
      historicalData,
      metrics,
      recommendations,
      alerts,
      isMonitoring,
      alertThresholds,
    } = this.state;

    return (
      <div className="performance-monitor">
        <div className="performance-header">
          <h3>Performance Monitor</h3>
          <div className="monitor-controls">
            <button
              className={`btn ${isMonitoring ? 'btn-danger' : 'btn-success'}`}
              onClick={() => (isMonitoring ? this.stopMonitoring() : this.startMonitoring())}
            >
              {isMonitoring ? 'Stop Monitoring' : 'Start Monitoring'}
            </button>
            <button className="btn btn-outline" onClick={() => this.loadHistoricalData()}>
              Refresh Data
            </button>
          </div>
        </div>

        <div className="performance-content">
          {this.renderCurrentMetrics()}
          {this.renderHistoricalChart()}
          {this.renderAlerts()}
          {this.renderRecommendations()}
          {this.renderPerformanceSettings()}
        </div>
      </div>
    );
  }

  private renderCurrentMetrics() {
    const { currentSnapshot, metrics } = this.state;

    if (!currentSnapshot || !metrics) {
      return (
        <div className="metrics-section">
          <h4>Current Performance Metrics</h4>
          <p>Loading metrics...</p>
        </div>
      );
    }

    return (
      <div className="metrics-section">
        <h4>Current Performance Metrics</h4>
        <div className="metrics-grid">
          <div className="metric-card">
            <div className="metric-icon">🧠</div>
            <div className="metric-label">Memory Usage</div>
            <div className="metric-value">{currentSnapshot.memoryUsage.toFixed(1)} MB</div>
            <div className="metric-subvalue">Peak: {metrics.peakMemoryUsage.toFixed(1)} MB</div>
          </div>

          <div className="metric-card">
            <div className="metric-icon">⚡</div>
            <div className="metric-label">Response Time</div>
            <div className="metric-value">{currentSnapshot.averageResponseTime.toFixed(0)} ms</div>
            <div className="metric-subvalue">
              Average: {metrics.averageOperationDuration.toFixed(0)} ms
            </div>
          </div>

          <div className="metric-card">
            <div className="metric-icon">📊</div>
            <div className="metric-label">Cache Hit Rate</div>
            <div className="metric-value">{(currentSnapshot.cacheHitRate * 100).toFixed(1)}%</div>
            <div className="metric-subvalue">
              Effective: {(metrics.cacheEffectiveness * 100).toFixed(1)}%
            </div>
          </div>

          <div className="metric-card">
            <div className="metric-icon">🚀</div>
            <div className="metric-label">Active Operations</div>
            <div className="metric-value">{currentSnapshot.activeOperations}</div>
            <div className="metric-subvalue">Total: {metrics.totalOperationsExecuted}</div>
          </div>

          <div className="metric-card">
            <div className="metric-icon">🕐</div>
            <div className="metric-label">Uptime</div>
            <div className="metric-value">{(metrics.totalUpTime / 3600000).toFixed(1)}h</div>
            <div className="metric-subvalue">
              Health: {(metrics.resourceUtilization * 100).toFixed(1)}%
            </div>
          </div>

          <div className="metric-card">
            <div className="metric-icon">🌐</div>
            <div className="metric-label">Network Load</div>
            <div className="metric-value">{currentSnapshot.networkRequests}</div>
            <div className="metric-subvalue">Requests/sec</div>
          </div>
        </div>

        {metrics.bottleneckDetection.length > 0 && (
          <div className="bottlenecks-section">
            <h5>Detected Performance Issues</h5>
            {metrics.bottleneckDetection.map((bottleneck, index) => (
              <div key={index} className="bottleneck-item warning">
                ⚠️ {bottleneck}
              </div>
            ))}
          </div>
        )}
      </div>
    );
  }

  private renderHistoricalChart() {
    const { historicalData } = this.state;

    if (historicalData.length === 0) {
      return (
        <div className="chart-section">
          <h4>Performance Trends</h4>
          <p>No historical data available</p>
        </div>
      );
    }

    // Simple visual representation - in a real implementation you'd use a charting library
    const latest = historicalData.slice(-10);

    return (
      <div className="chart-section">
        <h4>Performance Trends (Last {latest.length} samples)</h4>
        <div className="chart-container">
          <div className="chart-metric">
            <label>Memory Usage (MB)</label>
            {latest.map((snapshot, index) => (
              <div
                key={index}
                className="chart-bar"
                style={{
                  height: Math.min(snapshot.memoryUsage / 10, 100) + '%',
                }}
              >
                {snapshot.memoryUsage.toFixed(0)}
              </div>
            ))}
          </div>
          <div className="chart-metric">
            <label>Response Time (ms)</label>
            {latest.map((snapshot, index) => (
              <div
                key={index}
                className="chart-bar"
                style={{
                  height: Math.min(snapshot.averageResponseTime / 50, 100) + '%',
                }}
              >
                {snapshot.averageResponseTime.toFixed(0)}
              </div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  private renderAlerts() {
    const { alerts } = this.state;

    return (
      <div className="alerts-section">
        <h4>Performance Alerts</h4>
        {alerts.length === 0 ? (
          <p>No active alerts</p>
        ) : (
          <div className="alerts-list">
            {alerts
              .filter((alert) => !alert.resolved)
              .map((alert) => (
                <div key={alert.id} className={`alert-item ${alert.level}`}>
                  <div className="alert-content">
                    <span className="alert-level">{alert.level.toUpperCase()}</span>
                    <span className="alert-message">{alert.message}</span>
                    <span className="alert-time">
                      {new Date(alert.timestamp).toLocaleTimeString()}
                    </span>
                  </div>
                  <button className="btn-close" onClick={() => this.resolveAlert(alert.id)}>
                    ×
                  </button>
                </div>
              ))}
          </div>
        )}
      </div>
    );
  }

  private renderRecommendations() {
    const { recommendations } = this.state;

    return (
      <div className="recommendations-section">
        <h4>Optimization Recommendations</h4>
        <div className="recommendations-list">
          {recommendations.map((rec) => (
            <div key={rec.id} className={`recommendation-item ${rec.impact}`}>
              <div className="recommendation-header">
                <h5>{rec.title}</h5>
                <div className="recommendation-meta">
                  <span className={`impact ${rec.impact}`}>{rec.impact.toUpperCase()} IMPACT</span>
                  <span className={`effort ${rec.implementationEffort}`}>
                    {rec.implementationEffort.toUpperCase()}
                  </span>
                  <span className="improvement">+{rec.estimatedImprovement}% improvement</span>
                </div>
              </div>
              <p>{rec.description}</p>
              <div className="recommendation-confidence">
                Confidence: {Math.round(rec.confidence * 100)}%
              </div>
              <div className="recommendation-components">
                <small>Affects: {rec.affectedComponents.join(', ')}</small>
              </div>
              <button className="btn btn-primary btn-sm" onClick={() => rec.action()}>
                Apply Optimization
              </button>
            </div>
          ))}
        </div>
      </div>
    );
  }

  private renderPerformanceSettings() {
    return (
      <div className="settings-section">
        <h4>Alert Thresholds</h4>
        <div className="thresholds-settings">
          <div className="setting-group">
            <label htmlFor="memory-threshold">High Memory Usage (MB)</label>
            <input
              id="memory-threshold"
              type="number"
              value={this.state.alertThresholds.highMemoryUsage}
              onChange={(e) => this.updateThreshold('highMemoryUsage', parseInt(e.target.value))}
            />
          </div>

          <div className="setting-group">
            <label htmlFor="response-threshold">Slow Response Time (ms)</label>
            <input
              id="response-threshold"
              type="number"
              value={this.state.alertThresholds.slowOperations}
              onChange={(e) => this.updateThreshold('slowOperations', parseInt(e.target.value))}
            />
          </div>

          <div className="setting-group">
            <label htmlFor="error-threshold">High Error Rate (%)</label>
            <input
              id="error-threshold"
              type="number"
              value={this.state.alertThresholds.highErrorRate * 100}
              onChange={(e) =>
                this.updateThreshold('highErrorRate', parseFloat(e.target.value) / 100)
              }
            />
          </div>

          <div className="setting-group">
            <label htmlFor="cache-threshold">Cache Miss Rate (%)</label>
            <input
              id="cache-threshold"
              type="number"
              value={this.state.alertThresholds.cacheMissRate * 100}
              onChange={(e) =>
                this.updateThreshold('cacheMissRate', parseFloat(e.target.value) / 100)
              }
            />
          </div>
        </div>
      </div>
    );
  }

  private updateThreshold(key: keyof typeof this.state.alertThresholds, value: number) {
    this.setState((prevState) => ({
      alertThresholds: {
        ...prevState.alertThresholds,
        [key]: value,
      },
    }));
  }
}

export default PerformanceMonitor;

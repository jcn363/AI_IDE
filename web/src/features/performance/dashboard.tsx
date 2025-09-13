import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface PerformanceMetrics {
  cpuUsage: number;
  memoryUsage: number;
  responseTime: number;
  activeUsers: number;
  sessionDuration: number;
  concurrentSessions: number;
  networkLatency: number;
  errorRate: number;
}

interface SessionData {
  sessionId: string;
  userId: string;
  startTime: string;
  activity: string[];
  performance: {
    cpu: number;
    memory: number;
  };
}

interface CollaborativeMetrics {
  totalActiveSessions: number;
  averageResponseTime: number;
  peakConcurrentUsers: number;
  performanceTrends: {
    timestamp: string;
    cpu: number;
    memory: number;
  }[];
}

const PerformanceDashboard: React.FC = () => {
  const [metrics, setMetrics] = useState<PerformanceMetrics | null>(null);
  const [sessions, setSessions] = useState<SessionData[]>([]);
  const [collaborativeMetrics, setCollaborativeMetrics] = useState<CollaborativeMetrics | null>(
    null
  );
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchPerformanceData();

    // Real-time updates every 5 seconds
    const interval = setInterval(fetchPerformanceData, 5000);

    return () => clearInterval(interval);
  }, []);

  const fetchPerformanceData = async () => {
    try {
      setLoading(true);

      // Fetch collaborative performance metrics
      const collabMetrics: CollaborativeMetrics = await invoke(
        'get_collaborative_performance_metrics'
      );
      setCollaborativeMetrics(collabMetrics);

      // Fetch individual performance metrics
      const perfMetrics: PerformanceMetrics = await invoke('get_performance_metrics');
      setMetrics(perfMetrics);

      // Fetch active session data
      const sessionData: SessionData[] = await invoke('get_active_sessions');
      setSessions(sessionData);

      setError(null);
    } catch (err) {
      setError('Failed to fetch performance data');
      console.error('Performance data fetch error:', err);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="performance-dashboard loading">
        <div className="loading-spinner">Loading performance data...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="performance-dashboard error">
        <div className="error-message">{error}</div>
        <button onClick={fetchPerformanceData}>Retry</button>
      </div>
    );
  }

  return (
    <div className="performance-dashboard">
      <header className="dashboard-header">
        <h1>Collaborative Performance Dashboard</h1>
        <p>Real-time monitoring for multi-user sessions</p>
      </header>

      {/* System Performance Metrics */}
      <section className="metrics-section">
        <h2>System Performance</h2>
        <div className="metrics-grid">
          {metrics && (
            <>
              <div className="metric-card">
                <h3>CPU Usage</h3>
                <div className="metric-value">{metrics.cpuUsage.toFixed(1)}%</div>
                <div className="metric-bar">
                  <div className="metric-fill" style={{ width: `${metrics.cpuUsage}%` }} />
                </div>
              </div>

              <div className="metric-card">
                <h3>Memory Usage</h3>
                <div className="metric-value">{metrics.memoryUsage.toFixed(1)}%</div>
                <div className="metric-bar">
                  <div className="metric-fill" style={{ width: `${metrics.memoryUsage}%` }} />
                </div>
              </div>

              <div className="metric-card">
                <h3>Response Time</h3>
                <div className="metric-value">{metrics.responseTime}ms</div>
              </div>

              <div className="metric-card">
                <h3>Network Latency</h3>
                <div className="metric-value">{metrics.networkLatency}ms</div>
              </div>

              <div className="metric-card">
                <h3>Error Rate</h3>
                <div className="metric-value">{metrics.errorRate.toFixed(2)}%</div>
              </div>
            </>
          )}
        </div>
      </section>

      {/* Collaborative Metrics */}
      <section className="collaborative-section">
        <h2>Collaborative Metrics</h2>
        <div className="collaborative-grid">
          {collaborativeMetrics && (
            <>
              <div className="metric-card">
                <h3>Active Sessions</h3>
                <div className="metric-value">{collaborativeMetrics.totalActiveSessions}</div>
              </div>

              <div className="metric-card">
                <h3>Avg Response Time</h3>
                <div className="metric-value">{collaborativeMetrics.averageResponseTime}ms</div>
              </div>

              <div className="metric-card">
                <h3>Peak Concurrent Users</h3>
                <div className="metric-value">{collaborativeMetrics.peakConcurrentUsers}</div>
              </div>
            </>
          )}
        </div>
      </section>

      {/* Session Activity */}
      <section className="sessions-section">
        <h2>Active Sessions</h2>
        <div className="sessions-table">
          <table>
            <thead>
              <tr>
                <th>Session ID</th>
                <th>User ID</th>
                <th>Start Time</th>
                <th>CPU Usage</th>
                <th>Memory Usage</th>
                <th>Recent Activity</th>
              </tr>
            </thead>
            <tbody>
              {sessions.map((session) => (
                <tr key={session.sessionId}>
                  <td>{session.sessionId}</td>
                  <td>{session.userId}</td>
                  <td>{new Date(session.startTime).toLocaleTimeString()}</td>
                  <td>{session.performance.cpu.toFixed(1)}%</td>
                  <td>{session.performance.memory.toFixed(1)}%</td>
                  <td>
                    <ul>
                      {session.activity.slice(-3).map((act, idx) => (
                        <li key={idx}>{act}</li>
                      ))}
                    </ul>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Performance Trends */}
      <section className="trends-section">
        <h2>Performance Trends</h2>
        <div className="trends-chart">
          {collaborativeMetrics?.performanceTrends && (
            <div className="chart-placeholder">
              <p>Performance Trends Chart</p>
              <div className="trend-data">
                {collaborativeMetrics.performanceTrends.map((point, idx) => (
                  <div key={idx} className="trend-point">
                    <span>{new Date(point.timestamp).toLocaleTimeString()}</span>
                    <span>CPU: {point.cpu.toFixed(1)}%</span>
                    <span>Memory: {point.memory.toFixed(1)}%</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </section>

      <style jsx>{`
        .performance-dashboard {
          padding: 20px;
          font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
          background-color: #f5f5f5;
          min-height: 100vh;
        }

        .dashboard-header {
          text-align: center;
          margin-bottom: 30px;
        }

        .dashboard-header h1 {
          color: #2c3e50;
          margin-bottom: 10px;
        }

        .dashboard-header p {
          color: #7f8c8d;
          font-size: 14px;
        }

        .metrics-section,
        .collaborative-section,
        .sessions-section,
        .trends-section {
          background: white;
          border-radius: 8px;
          padding: 20px;
          margin-bottom: 20px;
          box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
        }

        .metrics-section h2,
        .collaborative-section h2,
        .sessions-section h2,
        .trends-section h2 {
          color: #34495e;
          margin-bottom: 20px;
          border-bottom: 2px solid #ecf0f1;
          padding-bottom: 10px;
        }

        .metrics-grid,
        .collaborative-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 20px;
        }

        .metric-card {
          background: #f8f9fa;
          border-radius: 6px;
          padding: 15px;
          text-align: center;
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        }

        .metric-card h3 {
          color: #495057;
          margin-bottom: 10px;
          font-size: 14px;
          font-weight: 600;
        }

        .metric-value {
          font-size: 24px;
          font-weight: bold;
          color: #28a745;
          margin-bottom: 10px;
        }

        .metric-bar {
          width: 100%;
          height: 8px;
          background: #e9ecef;
          border-radius: 4px;
          overflow: hidden;
        }

        .metric-fill {
          height: 100%;
          background: linear-gradient(90deg, #28a745, #20c997);
          transition: width 0.3s ease;
        }

        .metric-fill[style*='width: 70%'],
        .metric-fill[style*='width: 80%'],
        .metric-fill[style*='width: 90%'],
        .metric-fill[style*='width: 100%'] {
          background: linear-gradient(90deg, #fd7e14, #dc3545);
        }

        .sessions-table {
          overflow-x: auto;
        }

        .sessions-table table {
          width: 100%;
          border-collapse: collapse;
        }

        .sessions-table th,
        .sessions-table td {
          padding: 12px;
          text-align: left;
          border-bottom: 1px solid #dee2e6;
        }

        .sessions-table th {
          background: #f8f9fa;
          font-weight: 600;
          color: #495057;
        }

        .sessions-table td ul {
          list-style: none;
          padding: 0;
          margin: 0;
        }

        .sessions-table td li {
          font-size: 12px;
          color: #6c757d;
          margin-bottom: 2px;
        }

        .chart-placeholder {
          padding: 20px;
          text-align: center;
          background: #f8f9fa;
          border-radius: 6px;
        }

        .trend-data {
          margin-top: 20px;
        }

        .trend-point {
          display: flex;
          justify-content: space-between;
          padding: 8px 0;
          border-bottom: 1px solid #e9ecef;
        }

        .trend-point span {
          font-size: 12px;
          color: #6c757d;
        }

        .loading,
        .error {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 50vh;
        }

        .loading-spinner {
          font-size: 18px;
          color: #6c757d;
        }

        .error-message {
          color: #dc3545;
          margin-bottom: 10px;
        }

        .error button {
          padding: 8px 16px;
          background: #dc3545;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        }

        .error button:hover {
          background: #c82333;
        }
      `}</style>
    </div>
  );
};

export default PerformanceDashboard;

import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { FineTuneJobInfo, TrainingProgress, TrainingMetrics } from '../../features/ai/types';

interface JobProgressTrackerProps {
  job: FineTuneJobInfo;
  expanded?: boolean;
  onExpand?: () => void;
  onJobCancel?: (jobId: string) => void;
  onJobPause?: (jobId: string) => void;
  onJobResume?: (jobId: string) => void;
}

interface RealTimeMetrics {
  gpuUtilization: number;
  memoryUsage: number;
  loss: number;
  learningRate: number;
  samplesProcessed: number;
  eta?: string;
}

export const JobProgressTracker: React.FC<JobProgressTrackerProps> = ({
  job,
  expanded = false,
  onExpand,
  onJobCancel,
  onJobPause,
  onJobResume,
}) => {
  const [realTimeMetrics, setRealTimeMetrics] = useState<RealTimeMetrics | null>(null);
  const [isPolling, setIsPolling] = useState(false);
  const [pollInterval, setPollInterval] = useState<NodeJS.Timeout | null>(null);

  useEffect(() => {
    if ((job.status === 'Training' || job.status === 'Initializing') && expanded) {
      startPolling();
    } else {
      stopPolling();
    }

    return () => {
      stopPolling();
    };
  }, [job.status, expanded]);

  const startPolling = () => {
    if (isPolling) return;

    setIsPolling(true);
    // Poll every 5 seconds for real-time metrics
    const interval = setInterval(() => {
      fetchRealTimeMetrics();
    }, 5000);

    setPollInterval(interval);
  };

  const stopPolling = () => {
    if (pollInterval) {
      clearInterval(pollInterval);
      setPollInterval(null);
    }
    setIsPolling(false);
  };

  const fetchRealTimeMetrics = async () => {
    try {
      const metrics = await invoke<RealTimeMetrics>('get_finetune_job_metrics', {
        jobId: job.jobId,
      });
      setRealTimeMetrics(metrics);
    } catch (error) {
      console.error('Failed to fetch real-time metrics:', error);
    }
  };

  const handleCancel = () => {
    if (window.confirm('Are you sure you want to cancel this training job?')) {
      onJobCancel?.(job.jobId);
    }
  };

  const handlePause = () => {
    onJobPause?.(job.jobId);
  };

  const handleResume = () => {
    onJobResume?.(job.jobId);
  };

  const getProgressPercentage = (progress: TrainingProgress) => {
    return (progress.epoch / progress.totalEpochs) * 100;
  };

  const renderProgressBar = (progress: TrainingProgress) => {
    const percentage = getProgressPercentage(progress);
    const color =
      percentage < 25
        ? '#e53e3e'
        : percentage < 75
          ? '#dd6b20'
          : percentage === 100
            ? '#38a169'
            : '#3182ce';

    return (
      <div className="progress-container">
        <div className="progress-bar">
          <div
            className="progress-fill"
            style={{
              width: `${percentage}%`,
              backgroundColor: color,
            }}
          />
        </div>
        <div className="progress-text">
          <span>
            Epoch {progress.epoch}/{progress.totalEpochs}
          </span>
          <span>({percentage.toFixed(1)}%)</span>
        </div>
      </div>
    );
  };

  const renderMetrics = (metrics?: TrainingMetrics) => {
    if (!metrics) return null;

    return (
      <div className="metrics-grid">
        <div className="metric-item">
          <span className="metric-label">Final Loss:</span>
          <span className="metric-value">{metrics.finalLoss.toFixed(6)}</span>
        </div>
        <div className="metric-item">
          <span className="metric-label">Training Time:</span>
          <span className="metric-value">{formatDuration(metrics.trainingTimeSeconds)}</span>
        </div>
        <div className="metric-item">
          <span className="metric-label">Peak Memory:</span>
          <span className="metric-value">{(metrics.peakMemoryUsageMb / 1024).toFixed(1)} GB</span>
        </div>
        <div className="metric-item">
          <span className="metric-label">Samples/sec:</span>
          <span className="metric-value">{metrics.samplesPerSecond?.toFixed(1) || 'N/A'}</span>
        </div>
      </div>
    );
  };

  const renderRealTimeMetrics = () => {
    if (!realTimeMetrics) return null;

    return (
      <div className="real-time-metrics">
        <h4>Real-Time Metrics</h4>
        <div className="metrics-grid">
          <div className="metric-item">
            <span className="metric-label">GPU Usage:</span>
            <span className="metric-value">{realTimeMetrics.gpuUtilization}%</span>
          </div>
          <div className="metric-item">
            <span className="metric-label">Memory:</span>
            <span className="metric-value">
              {(realTimeMetrics.memoryUsage / 1024).toFixed(1)} GB
            </span>
          </div>
          <div className="metric-item">
            <span className="metric-label">Current Loss:</span>
            <span className="metric-value">{realTimeMetrics.loss.toFixed(6)}</span>
          </div>
          <div className="metric-item">
            <span className="metric-label">Learning Rate:</span>
            <span className="metric-value">{realTimeMetrics.learningRate.toExponential(3)}</span>
          </div>
          <div className="metric-item">
            <span className="metric-label">Samples Processed:</span>
            <span className="metric-value">
              {realTimeMetrics.samplesProcessed.toLocaleString()}
            </span>
          </div>
          {realTimeMetrics.eta && (
            <div className="metric-item">
              <span className="metric-label">ETA:</span>
              <span className="metric-value">{realTimeMetrics.eta}</span>
            </div>
          )}
        </div>
      </div>
    );
  };

  const renderControls = () => {
    return (
      <div className="job-controls">
        {job.status === 'Training' && (
          <>
            <button onClick={handlePause} className="control-btn warning">
              Pause Training
            </button>
            <button onClick={handleCancel} className="control-btn danger">
              Cancel Training
            </button>
          </>
        )}
        {job.status === 'Paused' && (
          <button onClick={handleResume} className="control-btn primary">
            Resume Training
          </button>
        )}
        <button onClick={onExpand} className="control-btn secondary">
          {expanded ? 'Collapse' : 'Expand Details'}
        </button>
      </div>
    );
  };

  const formatDuration = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const remainingSeconds = Math.floor(seconds % 60);
    return `${hours}h ${minutes}m ${remainingSeconds}s`;
  };

  return (
    <div className="job-progress-tracker">
      <div className="progress-header">
        <h3>{job.name} - Training Progress</h3>
        <span className={`status-badge status-${job.status.toLowerCase()}`}>{job.status}</span>
      </div>

      {job.progress && renderProgressBar(job.progress)}

      {expanded && (
        <div className="details-section">
          {renderRealTimeMetrics()}
          {job.progress && (
            <div className="training-details">
              <div className="details-grid">
                <div className="detail-item">
                  <span className="detail-label">Epoch:</span>
                  <span className="detail-value">
                    {job.progress.epoch}/{job.progress.totalEpochs}
                  </span>
                </div>
                <div className="detail-item">
                  <span className="detail-label">Step:</span>
                  <span className="detail-value">
                    {job.progress.step}/{job.progress.totalSteps}
                  </span>
                </div>
                <div className="detail-item">
                  <span className="detail-label">Loss:</span>
                  <span className="detail-value">{job.progress.loss?.toFixed(6) || 'N/A'}</span>
                </div>
                <div className="detail-item">
                  <span className="detail-label">ETA:</span>
                  <span className="detail-value">
                    {job.progress.estimatedTimeRemaining
                      ? formatDuration(job.progress.estimatedTimeRemaining)
                      : 'N/A'}
                  </span>
                </div>
              </div>
            </div>
          )}

          {renderMetrics(job.metrics)}
        </div>
      )}

      {renderControls()}

      {job.errorMessage && (
        <div className="error-message">
          <strong>Error:</strong> {job.errorMessage}
        </div>
      )}

      <style jsx>{`
        .job-progress-tracker {
          padding: 16px;
          border: 1px solid #e1e5e9;
          border-radius: 8px;
          background: white;
          margin-bottom: 16px;
        }

        .progress-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 16px;
        }

        .progress-header h3 {
          margin: 0;
          font-size: 18px;
          color: #2d3748;
        }

        .status-badge {
          padding: 4px 12px;
          border-radius: 12px;
          font-size: 12px;
          font-weight: 500;
          color: white;
          text-transform: uppercase;
        }

        .status-completed {
          background: #38a169;
        }
        .status-training {
          background: #3182ce;
        }
        .status-failed {
          background: #e53e3e;
        }
        .status-paused {
          background: #dd6b20;
        }

        .progress-container {
          margin-bottom: 16px;
        }

        .progress-bar {
          height: 8px;
          background: #e1e5e9;
          border-radius: 4px;
          overflow: hidden;
        }

        .progress-fill {
          height: 100%;
          transition:
            width 0.3s ease,
            background-color 0.3s ease;
        }

        .progress-text {
          display: flex;
          justify-content: space-between;
          margin-top: 8px;
          font-size: 14px;
          color: #4a5568;
        }

        .details-section {
          margin: 20px 0;
          padding-top: 16px;
          border-top: 1px solid #e1e5e9;
        }

        .real-time-metrics h4 {
          margin: 0 0 12px 0;
          font-size: 16px;
          color: #2d3748;
        }

        .metrics-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 12px;
          margin-bottom: 16px;
        }

        .metric-item,
        .detail-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 8px 12px;
          border: 1px solid #f1f5f9;
          border-radius: 4px;
          background: #f8fafc;
        }

        .metric-label,
        .detail-label {
          font-weight: 500;
          color: #4a5568;
        }

        .metric-value,
        .detail-value {
          color: #2d3748;
          font-family: monospace;
        }

        .training-details {
          margin-top: 20px;
        }

        .details-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 16px;
        }

        .job-controls {
          display: flex;
          gap: 12px;
          margin-top: 16px;
          flex-wrap: wrap;
        }

        .control-btn {
          padding: 8px 16px;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-weight: 500;
          transition: background-color 0.2s;
        }

        .control-btn.primary {
          background: #3182ce;
          color: white;
        }

        .control-btn.warning {
          background: #dd6b20;
          color: white;
        }

        .control-btn.danger {
          background: #e53e3e;
          color: white;
        }

        .control-btn.secondary {
          background: #e2e8f0;
          color: #4a5568;
        }

        .error-message {
          margin-top: 16px;
          padding: 12px;
          background: #fed7d7;
          border: 1px solid #fc8181;
          border-radius: 4px;
          color: #c53030;
        }
      `}</style>
    </div>
  );
};

export default JobProgressTracker;

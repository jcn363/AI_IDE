import React, { useEffect, useState } from 'react';
import { LearningMetrics } from '../../types/ai';

interface LearningProgressIndicatorProps {
  metrics: LearningMetrics;
  showDetails?: boolean;
  compact?: boolean;
}

const LearningProgressIndicator: React.FC<LearningProgressIndicatorProps> = ({
  metrics,
  showDetails = false,
  compact = false,
}) => {
  const [animatedProgress, setAnimatedProgress] = useState(metrics.averageRating * 20);

  useEffect(() => {
    // Animate the progress bar
    const targetProgress = metrics.averageRating * 20;
    const animationDuration = 1000;
    const startTime = Date.now();

    const animate = () => {
      const elapsed = Date.now() - startTime;
      const progress = Math.min(elapsed / animationDuration, 1);
      const easeOutProgress = 1 - Math.pow(1 - progress, 3); // Cubic ease out

      setAnimatedProgress(metrics.averageRating * 20 * easeOutProgress);

      if (progress < 1) {
        requestAnimationFrame(animate);
      }
    };

    requestAnimationFrame(animate);
  }, [metrics.averageRating]);

  const getImprovementColor = (improvement: number): string => {
    if (improvement > 0.1) return '#10b981'; // Green for significant improvement
    if (improvement > 0.05) return '#f59e0b'; // Yellow for moderate improvement
    if (improvement < -0.05) return '#ef4444'; // Red for decline
    return '#6b7280'; // Gray for stable
  };

  const getImprovementIcon = (improvement: number): string => {
    if (improvement > 0.1) return '📈';
    if (improvement > 0.05) return '📊';
    if (improvement < -0.05) return '📉';
    return '➡️';
  };

  if (compact) {
    return (
      <div className="learning-progress-compact">
        <div className="progress-bar">
          <div
            className="progress-fill"
            style={{
              width: `${animatedProgress}%`,
              backgroundColor: getImprovementColor(metrics.improvementRate),
            }}
          />
        </div>
        <span className="rating">{metrics.averageRating.toFixed(1)}⭐</span>
      </div>
    );
  }

  return (
    <div className="learning-progress-indicator">
      <div className="progress-header">
        <h4>AI Learning Progress</h4>
        <div className="improvement-indicator">
          <span
            className="improvement-icon"
            style={{ color: getImprovementColor(metrics.improvementRate) }}
          >
            {getImprovementIcon(metrics.improvementRate)}
          </span>
          <span className="improvement-text">
            {metrics.improvementRate > 0
              ? `Improving at ${(metrics.improvementRate * 100).toFixed(1)}%`
              : metrics.improvementRate < 0
                ? `Declining at ${Math.abs(metrics.improvementRate * 100).toFixed(1)}%`
                : 'Stable performance'}
          </span>
        </div>
      </div>

      <div className="progress-metrics">
        <div className="metric-item">
          <div className="metric-label">Generations</div>
          <div className="metric-value">{metrics.totalGenerations}</div>
        </div>

        <div className="metric-item">
          <div className="metric-label">Average Rating</div>
          <div className="metric-value">
            <span className="rating-stars">{'⭐'.repeat(Math.round(metrics.averageRating))}</span>
            <span className="rating-number">{metrics.averageRating.toFixed(1)}</span>
          </div>
        </div>

        <div className="metric-item">
          <div className="metric-label">Pattern Accuracy</div>
          <div className="metric-value">{(metrics.patternAccuracy * 100).toFixed(1)}%</div>
        </div>

        <div className="metric-item">
          <div className="metric-label">Context Improvement</div>
          <div className="metric-value">{(metrics.contextImprovement * 100).toFixed(1)}%</div>
        </div>
      </div>

      {showDetails && (
        <div className="progress-details">
          <div className="progress-bar-container">
            <div className="progress-labels">
              <span>Learning Progress</span>
              <span>{animatedProgress.toFixed(1)}%</span>
            </div>
            <div className="progress-bar">
              <div
                className="progress-fill"
                style={{
                  width: `${animatedProgress}%`,
                  backgroundColor: getImprovementColor(metrics.improvementRate),
                }}
              />
            </div>
          </div>

          <div className="last-updated">
            Last updated: {new Date(metrics.lastUpdated).toLocaleDateString()}
          </div>
        </div>
      )}

      <style jsx>{`
        .learning-progress-indicator {
          background: white;
          border-radius: 12px;
          padding: 20px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
          border: 1px solid #e5e7eb;
        }

        .learning-progress-compact {
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 8px 12px;
          background: #f9fafb;
          border-radius: 8px;
          border: 1px solid #e5e7eb;
        }

        .progress-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 16px;
        }

        .progress-header h4 {
          margin: 0;
          color: #111827;
          font-size: 18px;
        }

        .improvement-indicator {
          display: flex;
          align-items: center;
          gap: 8px;
          font-size: 14px;
          color: #6b7280;
        }

        .improvement-icon {
          font-size: 16px;
        }

        .progress-metrics {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
          gap: 16px;
          margin-bottom: 20px;
        }

        .metric-item {
          text-align: center;
          padding: 12px;
          background: #f9fafb;
          border-radius: 8px;
        }

        .metric-label {
          font-size: 12px;
          color: #6b7280;
          margin-bottom: 4px;
        }

        .metric-value {
          font-size: 18px;
          font-weight: 600;
          color: #111827;
        }

        .rating-stars {
          margin-right: 4px;
        }

        .rating-number {
          font-size: 14px;
          color: #6b7280;
        }

        .progress-details {
          border-top: 1px solid #e5e7eb;
          padding-top: 16px;
        }

        .progress-bar-container {
          margin-bottom: 12px;
        }

        .progress-labels {
          display: flex;
          justify-content: space-between;
          margin-bottom: 8px;
          font-size: 14px;
          color: #6b7280;
        }

        .progress-bar {
          height: 8px;
          background: #e5e7eb;
          border-radius: 4px;
          overflow: hidden;
        }

        .progress-fill {
          height: 100%;
          border-radius: 4px;
          transition: width 0.3s ease;
        }

        .compact .progress-bar {
          height: 6px;
        }

        .compact .rating {
          font-size: 14px;
          font-weight: 600;
          color: #111827;
        }

        .last-updated {
          font-size: 12px;
          color: #9ca3af;
          text-align: center;
        }

        @media (max-width: 640px) {
          .progress-metrics {
            grid-template-columns: repeat(2, 1fr);
          }

          .progress-header {
            flex-direction: column;
            align-items: flex-start;
            gap: 8px;
          }
        }
      `}</style>
    </div>
  );
};

export default LearningProgressIndicator;

import React from 'react';

interface ProgressBarProps {
  value: number;
  max?: number;
  label?: string;
  color?: 'primary' | 'success' | 'warning' | 'error' | 'info';
  size?: 'small' | 'medium' | 'large';
  showPercentage?: boolean;
  animated?: boolean;
  className?: string;
}

const PROGRESS_COLORS = {
  primary: '#3182ce',
  success: '#38a169',
  warning: '#dd6b20',
  error: '#e53e3e',
  info: '#3182ce',
};

export const ProgressBar: React.FC<ProgressBarProps> = ({
  value,
  max = 100,
  label,
  color = 'primary',
  size = 'medium',
  showPercentage = false,
  animated = false,
  className = '',
}) => {
  const percentage = Math.min(Math.max((value / max) * 100, 0), 100);

  const getSizeStyles = () => {
    switch (size) {
      case 'small':
        return { height: '4px', fontSize: '12px' };
      case 'large':
        return { height: '12px', fontSize: '16px' };
      default:
        return { height: '8px', fontSize: '14px' };
    }
  };

  const sizeStyles = getSizeStyles();

  return (
    <div className={`progress-container ${className}`}>
      {(label || showPercentage) && (
        <div className="progress-header">
          {label && <span className="progress-label">{label}</span>}
          {showPercentage && (
            <span className="progress-percentage">{percentage.toFixed(1)}%</span>
          )}
        </div>
      )}

      <div className="progress-bar" style={{ height: sizeStyles.height }}>
        <div
          className="progress-fill"
          style={{
            width: `${percentage}%`,
            backgroundColor: PROGRESS_COLORS[color],
            transition: animated ? 'width 0.3s ease' : 'none',
          }}
        />
      </div>

      <style jsx>{`
        .progress-container {
          width: 100%;
        }

        .progress-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 6px;
        }

        .progress-label {
          font-weight: 500;
          color: #4a5568;
          font-size: ${sizeStyles.fontSize};
        }

        .progress-percentage {
          font-size: ${parseInt(sizeStyles.fontSize) - 2}px;
          color: #718096;
          font-weight: 500;
        }

        .progress-bar {
          width: 100%;
          background: #e1e5e9;
          border-radius: ${parseInt(sizeStyles.height) / 2}px;
          overflow: hidden;
        }

        .progress-fill {
          height: 100%;
          border-radius: ${parseInt(sizeStyles.height) / 2}px;
        }

        @media (prefers-reduced-motion: reduce) {
          .progress-fill {
            transition: none !important;
          }
        }
      `}</style>
    </div>
  );
};

export default ProgressBar;
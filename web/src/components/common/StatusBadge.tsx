import React from 'react';

interface StatusBadgeProps {
  status: string;
  size?: 'small' | 'medium' | 'large';
  className?: string;
}

const STATUS_COLORS: Record<string, { bg: string; text: string }> = {
  // Cargo operations
  completed: { bg: '#c6f6d5', text: '#276749' },
  failed: { bg: '#fed7d7', text: '#c53030' },
  pending: { bg: '#e2e8f0', text: '#4a5568' },
  running: { bg: '#c6f6d5', text: '#276749' },
  initializing: { bg: '#fff5f5', text: '#d69e2e' },
  checking: { bg: '#bee3f8', text: '#2c5282' },
  building: { bg: '#c6f6d5', text: '#276749' },
  testing: { bg: '#c6f6d5', text: '#276749' },
  clippy: { bg: '#d6bcfa', text: '#553c9a' },
  fmt: { bg: '#d6bcfa', text: '#553c9a' },

  // AI Fine-tuning
  training: { bg: '#c6f6d5', text: '#276749' },
  evaluating: { bg: '#bee3f8', text: '#2c3338' },
  saving: { bg: '#fef5e7', text: '#7b341e' },
  created: { bg: '#e2e8f0', text: '#4a5568' },
  cancelled: { bg: '#e2e8f0', text: '#4a5568' },

  // Model states
  loaded: { bg: '#c6f6d5', text: '#276749' },
  loading: { bg: '#bee3f8', text: '#2c3338' },
  unloaded: { bg: '#e2e8f0', text: '#4a5568' },
  error: { bg: '#fed7d7', text: '#c53030' },
};

export const StatusBadge: React.FC<StatusBadgeProps> = ({
  status,
  size = 'medium',
  className = '',
}) => {
  const getColors = (statusKey: string) => {
    return STATUS_COLORS[statusKey.toLowerCase()] || STATUS_COLORS.pending;
  };

  const { bg, text } = getColors(status);

  const getSizeStyles = () => {
    switch (size) {
      case 'small':
        return { padding: '2px 6px', fontSize: '11px' };
      case 'large':
        return { padding: '6px 12px', fontSize: '14px' };
      default:
        return { padding: '4px 8px', fontSize: '12px' };
    }
  };

  const sizeStyles = getSizeStyles();

  return (
    <span
      className={`status-badge ${className}`}
      style={{
        backgroundColor: bg,
        color: text,
        padding: sizeStyles.padding,
        fontSize: sizeStyles.fontSize,
      }}
    >
      {status.charAt(0).toUpperCase() + status.slice(1)}
    </span>
  );
};

export default StatusBadge;

// CSS-in-JS styles
const badgeStyles = `
  .status-badge {
    border-radius: 12px;
    font-weight: 500;
    text-transform: uppercase;
    display: inline-block;
    white-space: nowrap;
    text-align: center;
    user-select: none;
  }

  .status-badge.small {
    padding: 2px 6px !important;
    font-size: 11px !important;
  }

  .status-badge.medium {
    padding: 4px 8px !important;
    font-size: 12px !important;
  }

  .status-badge.large {
    padding: 6px 12px !important;
    font-size: 14px !important;
  }
`;

// Inject styles (in a real app, you'd use a proper CSS-in-JS solution)
// This is a simple implementation for demonstration
if (typeof document !== 'undefined') {
  const existingStyle = document.getElementById('status-badge-styles');
  if (!existingStyle) {
    const styleSheet = document.createElement('style');
    styleSheet.id = 'status-badge-styles';
    styleSheet.textContent = badgeStyles;
    document.head.appendChild(styleSheet);
  }
}
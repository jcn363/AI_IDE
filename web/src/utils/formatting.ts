/**
 * Common formatting utilities for display and data presentation
 */

/**
 * Formats file sizes in human-readable format
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

/**
 * Formats numbers with appropriate separators
 */
export function formatNumber(num: number): string {
  return new Intl.NumberFormat('en-US').format(num);
}

/**
 * Formats duration in milliseconds to human-readable format
 */
export function formatDuration(ms: number): string {
  const seconds = Math.floor(ms / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days > 0) {
    return `${days}d ${hours % 24}h ${minutes % 60}m`;
  } else if (hours > 0) {
    return `${hours}h ${minutes % 60}m ${seconds % 60}s`;
  } else if (minutes > 0) {
    return `${minutes}m ${seconds % 60}s`;
  } else {
    return `${seconds}s`;
  }
}

/**
 * Formats a timestamp to a human-readable date string
 */
export function formatTimestamp(timestamp: number | string | Date): string {
  const date = new Date(timestamp);
  return date.toLocaleString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

/**
 * Formats dependency version string with appropriate styling
 */
export function formatVersion(version: string, isLatest = false): string {
  if (isLatest) {
    return version + ' (latest)';
  }
  return version;
}

/**
 * Formats package name with namespace
 */
export function formatPackageName(name: string, namespace?: string): string {
  if (namespace) {
    return `${namespace}/${name}`;
  }
  return name;
}

/**
 * Truncates text to a specified length with ellipsis
 */
export function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text;
  return text.slice(0, maxLength - 3) + '...';
}

/**
 * Formats error messages with standardized format
 */
export function formatErrorMessage(error: string | Error): string {
  if (typeof error === 'string') return error;
  return error.message || 'An unknown error occurred';
}

/**
 * Capitalizes the first letter of a string
 */
export function capitalizeFirst(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

/**
 * Converts camelCase to readable format
 */
export function camelCaseToReadable(str: string): string {
  return capitalizeFirst(str.replace(/([A-Z])/g, ' $1').trim());
}

/**
 * Formats learning rate with appropriate precision
 */
export function formatLearningRate(rate: number): string {
  if (rate >= 1e-3) {
    return rate.toFixed(4);
  } else if (rate >= 1e-6) {
    return rate.toExponential(2);
  } else {
    return rate.toExponential(1);
  }
}

/**
 * Formats training status with appropriate icons/emoji
 */
export function formatTrainingStatus(status: string): string {
  const statusMap: Record<string, string> = {
    running: '🔄 Running',
    completed: '✅ Completed',
    failed: '❌ Failed',
    stopped: '⏹️ Stopped',
    queued: '⏳ Queued',
    pending: '🟡 Pending',
  };
  return statusMap[status] || capitalizeFirst(status);
}

/**
 * Formats job progress as percentage
 */
export function formatJobProgress(current: number, total: number): string {
  if (total === 0) return '0%';
  const percentage = Math.round((current / total) * 100);
  return `${percentage}%`;
}

/**
 * Formats array elements with comma separation
 */
export function formatArrayList(items: string[], maxItems = 3): string {
  if (items.length <= maxItems) {
    return items.join(', ');
  }
  const displayed = items.slice(0, maxItems).join(', ');
  const remaining = items.length - maxItems;
  return `${displayed} (+${remaining} more)`;
}

export default {
  formatFileSize,
  formatNumber,
  formatDuration,
  formatTimestamp,
  formatVersion,
  formatPackageName,
  truncateText,
  formatErrorMessage,
  capitalizeFirst,
  camelCaseToReadable,
  formatLearningRate,
  formatTrainingStatus,
  formatJobProgress,
  formatArrayList,
};

export type UpdateType = 'major' | 'minor' | 'patch' | 'prerelease';

export interface VersionCompatibility {
  isCompatible: boolean;
  breakingChanges: string[];
  recommendedVersion?: string;
  compatibilityScore: number; // 0-100
}

export interface ChangelogData {
  version: string;
  date: string;
  changes: {
    type: 'added' | 'changed' | 'deprecated' | 'removed' | 'fixed' | 'security';
    description: string;
  }[];
}

export interface DependencyUpdate {
  name: string;
  currentVersion: string;
  latestVersion: string;
  updateType: UpdateType;
  usedIn: Array<{ member: string; version: string }>;
  changelogUrl?: string;
  isUpdating: boolean;
  updateError?: string;
  compatibility?: VersionCompatibility;
  changelog?: ChangelogData;
  lastUpdated?: string;
  downloadStats?: {
    downloads: number;
    version: string;
  };
  isDirectDependency: boolean;
}

// Helper function to determine update type
export const getUpdateType = (current: string, latest: string): UpdateType => {
  if (!current || !latest || current === latest) return 'patch';
  
  // Handle pre-release versions
  if (latest.includes('-')) return 'prerelease';
  
  const currentParts = current.split('.').map(Number);
  const latestParts = latest.split('.').map(Number);
  
  if (currentParts[0] < latestParts[0]) return 'major';
  if (currentParts[1] < latestParts[1]) return 'minor';
  if (currentParts[2] < latestParts[2]) return 'patch';
  
  return 'patch';
};

export const getUpdateTypeLabel = (type: UpdateType): string => {
  switch (type) {
    case 'major': return 'Major Update';
    case 'minor': return 'Minor Update';
    case 'patch': return 'Patch Update';
    case 'prerelease': return 'Pre-release';
    default: return 'Update';
  }
};

export const getUpdateTypeColor = (type: UpdateType): string => {
  switch (type) {
    case 'major': return 'red';
    case 'minor': return 'orange';
    case 'patch': return 'green';
    case 'prerelease': return 'purple';
    default: return 'blue';
  }
};

export type Severity = 'low' | 'medium' | 'high';

export interface VersionAlignment {
  id: string;
  dependencyName: string;
  currentVersions: Record<string, string>; // packageName -> version
  suggestedVersion: string;
  severity: Severity;
  affectedPackages: string[];
  description?: string;
  lastUpdated?: string;
  isIgnored?: boolean;
}

export interface VersionAlignmentFilter {
  severity?: Severity | 'all';
  searchTerm?: string;
  showIgnored?: boolean;
  sortBy?: 'severity' | 'dependencyName' | 'suggestedVersion';
  sortOrder?: 'asc' | 'desc';
}

export interface WorkspaceStats {
  totalDependencies: number;
  alignedDependencies: number;
  conflicts: number;
  highSeverity: number;
  mediumSeverity: number;
  lowSeverity: number;
}

export interface WorkspaceAlignmentResult {
  alignments: VersionAlignment[];
  stats: WorkspaceStats;
}

export interface VersionAlignmentState {
  alignments: VersionAlignment[];
  filteredAlignments: VersionAlignment[];
  selectedIds: string[];
  filter: VersionAlignmentFilter;
  status: 'idle' | 'loading' | 'succeeded' | 'failed';
  error: string | null;
  lastUpdated?: string;
  workspaceStats: WorkspaceStats;
}

export interface BulkActionResponse {
  success: boolean;
  message: string;
  updatedCount: number;
  failedCount: number;
}

export interface DependencyState {
  versionAlignment: VersionAlignmentState;
  // Add other dependency-related state here
}

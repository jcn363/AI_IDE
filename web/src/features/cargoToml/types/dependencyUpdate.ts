export interface DependencyUpdateInfo {
  name: string;
  currentVersion: string;
  latestVersion: string;
  updateType: 'major' | 'minor' | 'patch' | 'unknown';
  changelogUrl?: string;
  isDirect: boolean;
  usedIn: string[];
}

export interface DependencyUpdatesResponse {
  updates: DependencyUpdateInfo[];
  timestamp: string;
}

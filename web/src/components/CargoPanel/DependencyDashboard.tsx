/**
 * Main dashboard component for dependency management
 * Contains sub-components for different aspects of dependency management
 */

import React, { useState } from 'react';
import { Box, Tabs, Tab, Alert } from '@mui/material';
import { DependencyGraph } from './DependencyGraph';
import { LockfileViewer } from './LockfileViewer';
import { FeaturesManager } from './FeaturesManager';
import { ConflictResolver } from './ConflictResolver';
import type { CargoManifest, CargoDependency } from '../../types/cargo';

// Represents the structure of dependency data from Cargo
interface DependencyData {
  package?: {
    name: string;
    version: string;
    edition?: string;
    [key: string]: unknown;
  };
  dependencies?: Record<string, string | { version?: string; [key: string]: unknown }>;
  'dev-dependencies'?: Record<string, string | { version?: string; [key: string]: unknown }>;
  'build-dependencies'?: Record<string, string | { version?: string; [key: string]: unknown }>;
  features?: Record<string, string[]>;
  [key: string]: unknown; // Index signature to allow other properties
}

// Type guard to check if an object is a DependencyData
function isDependencyData(obj: unknown): obj is DependencyData {
  return obj !== null && typeof obj === 'object' && (obj as DependencyData).package !== undefined;
}

// Converts DependencyData to CargoManifest
function toCargoManifest(data: DependencyData | null): CargoManifest {
  if (!data) {
    return {
      package: {
        name: 'unknown',
        version: '0.0.0',
        edition: '2021',
      },
      dependencies: {},
      'dev-dependencies': {},
      'build-dependencies': {},
      features: {},
    };
  }

  // Helper function to convert dependency entries to CargoDependency format
  const convertDeps = (
    deps: Record<string, string | { version?: string; [key: string]: unknown }> = {}
  ): Record<string, CargoDependency> => {
    return Object.entries(deps).reduce(
      (acc, [name, dep]) => {
        if (typeof dep === 'string') {
          acc[name] = { version: dep } as CargoDependency;
        } else if (dep && typeof dep === 'object') {
          // Ensure we have at least a version or path for a valid CargoDependency
          if ('version' in dep || 'path' in dep || 'git' in dep) {
            acc[name] = { ...dep } as CargoDependency;
          }
        }
        return acc;
      },
      {} as Record<string, CargoDependency>
    );
  };

  return {
    package: data.package
      ? {
          name: data.package.name || 'unknown',
          version: data.package.version || '0.0.0',
          edition: data.package.edition || '2021',
          ...(typeof data.package === 'object' ? data.package : {}),
        }
      : undefined,
    dependencies: convertDeps(data.dependencies),
    'dev-dependencies': convertDeps(data['dev-dependencies']),
    'build-dependencies': convertDeps(data['build-dependencies']),
    features: data.features || {},
  };
}

interface ConflictData {
  name: string;
  versions: string[];
}

interface DependencyDashboardProps {
  projectPath: string | null;
  manifest: DependencyData | null;
  lockfile: DependencyData | null;
  features: Record<string, string[]> | null;
  conflicts: ConflictData[];
  error: string;
  loading: boolean;
  activeTab: number;
  onTabChange: (tab: number) => void;
  onLoadManifest: () => void;
  onLoadLockfile: () => void;
  onLoadFeatures: () => void;
  onLoadConflicts: () => void;
  onUpdateDependencies: (packageName?: string) => void;
  onError: (error: string) => void;
  onLoading: (loading: boolean) => void;
  onFeaturesUpdate: (features: Record<string, string[]> | null) => void;
  onConflictsUpdate: (conflicts: ConflictData[]) => void;
}

/**
 * Dashboard for managing Rust dependencies with multiple views
 */
export const DependencyDashboard: React.FC<DependencyDashboardProps> = ({
  projectPath,
  manifest,
  lockfile,
  features,
  conflicts,
  error,
  loading,
  activeTab,
  onTabChange,
  onLoadManifest,
  onLoadLockfile,
  onLoadFeatures,
  onLoadConflicts,
  onUpdateDependencies,
  onError,
}) => {
  return (
    <div className="dependency-dashboard">
      <Box sx={{ borderBottom: 1, borderColor: 'divider', mb: 2 }}>
        <Tabs value={activeTab} onChange={(_, v) => onTabChange(v)} aria-label="dependency tabs">
          <Tab label="Graph" />
          <Tab label="Cargo.lock" />
          <Tab label="Features" />
          <Tab label="Conflicts" />
          <Tab label="Update" />
        </Tabs>
      </Box>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => onError('')}>
          {error}
        </Alert>
      )}

      {/* Graph Tab */}
      {activeTab === 0 && <DependencyGraph manifest={toCargoManifest(manifest)} />}

      {/* Cargo.lock Tab */}
      {activeTab === 1 && <LockfileViewer projectPath={projectPath} onError={onError} />}

      {/* Features Tab */}
      {activeTab === 2 && (
        <FeaturesManager
          projectPath={projectPath}
          features={features}
          onError={onError}
          onLoading={(loading) => {}} // TODO: handle loading state
          onFeatureUpdate={(features) => {}} // TODO: handle features update
        />
      )}

      {/* Conflicts Tab */}
      {activeTab === 3 && (
        <ConflictResolver
          projectPath={projectPath}
          conflicts={conflicts}
          onError={onError}
          onLoading={(loading) => {}} // TODO: handle loading state
          onConflictsUpdate={(conflicts) => {}} // TODO: handle conflicts update
          onUpdateDependencies={onUpdateDependencies}
        />
      )}

      {/* Update Tab */}
      {activeTab === 4 && (
        <div>
          <Box sx={{ mb: 2, display: 'flex', gap: 1, alignItems: 'center' }}>
            <button onClick={() => onUpdateDependencies(undefined)}>Update All</button>
            <input
              placeholder="Package name (optional)"
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  const val = (e.target as any).value?.trim();
                  onUpdateDependencies(val || undefined);
                }
              }}
            />
          </Box>
          <p>Run cargo update for all deps or a specific package using -p.</p>
        </div>
      )}
    </div>
  );
};

export default DependencyDashboard;

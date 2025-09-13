/**
 * Component for managing and displaying features from Cargo.toml
 * Handles feature flag visualization and modification
 */

import React from 'react';
import { Button, List, ListItem, ListItemText, Typography } from '@mui/material';
import { invoke } from '@tauri-apps/api/core';

interface FeaturesManagerProps {
  projectPath: string | null;
  features: Record<string, string[]> | null;
  onError: (error: string) => void;
  onLoading: (loading: boolean) => void;
  onFeatureUpdate: (features: Record<string, string[]> | null) => void;
}

/**
 * Manages the display and modification of Cargo features
 */
export const FeaturesManager: React.FC<FeaturesManagerProps> = ({
  projectPath,
  features,
  onError,
  onLoading,
  onFeatureUpdate,
}) => {
  const loadFeatures = async () => {
    if (!projectPath) {
      onError('No project selected');
      return;
    }

    onLoading(true);
    try {
      // Load features from cargo metadata
      const result = await invoke('run_cargo_command', {
        command: 'metadata',
        args: '--format-version=1 --no-deps',
        cwd: projectPath,
      });

      if (result && typeof result === 'string') {
        const metadata = JSON.parse(result);
        const featureMap: Record<string, string[]> = {};

        // Extract features from metadata
        if (metadata.packages) {
          metadata.packages.forEach((pkg: any) => {
            if (pkg.features) {
              featureMap[pkg.name] = Object.keys(pkg.features);
            }
          });
        }

        onFeatureUpdate(featureMap);
      }
    } catch (error) {
      console.error('Error loading features:', error);
      onError(`Failed to load features: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      onLoading(false);
    }
  };

  return (
    <div className="features-manager">
      <Button variant="contained" onClick={loadFeatures} disabled={!projectPath} sx={{ mb: 2 }}>
        Load Features
      </Button>

      {features ? (
        <List dense>
          {Object.entries(features).map(([pkgName, pkgFeatures]) => (
            <ListItem key={pkgName}>
              <ListItemText primary={pkgName} secondary={pkgFeatures.join(', ')} />
            </ListItem>
          ))}
        </List>
      ) : (
        <Typography variant="body2">No features loaded.</Typography>
      )}
    </div>
  );
};

export default FeaturesManager;

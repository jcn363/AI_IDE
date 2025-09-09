/**
 * Component for detecting and resolving dependency conflicts
 * Handles version conflicts in Cargo dependencies
 */

import React from 'react';
import { Button, List, ListItem, ListItemText, ListItemSecondaryAction, Typography } from '@mui/material';
import { invoke } from '@tauri-apps/api/core';

interface ConflictData {
  name: string;
  versions: string[];
}

interface ConflictResolverProps {
  projectPath: string | null;
  conflicts: ConflictData[];
  onError: (error: string) => void;
  onLoading: (loading: boolean) => void;
  onConflictsUpdate: (conflicts: ConflictData[]) => void;
  onUpdateDependencies: (packageName?: string) => void;
}

/**
 * Resolves version conflicts in Cargo dependencies
 */
export const ConflictResolver: React.FC<ConflictResolverProps> = ({
  projectPath,
  conflicts,
  onError,
  onLoading,
  onConflictsUpdate,
  onUpdateDependencies,
}) => {
  const loadConflicts = async () => {
    if (!projectPath) return;

    onLoading(true);
    try {
      const result = await invoke<string>('execute_command', {
        command: 'tree',
        args: '--duplicates',
        cwd: projectPath,
      });

      if (result) {
        const conflictLines = result.split('\n').filter(line => line.includes('(*)'));
        const conflictMap = new Map<string, Set<string>>();

        conflictLines.forEach(line => {
          const match = line.match(/^([^\s]+) v([\d.]+)/);
          if (match) {
            const [, name, version] = match;
            if (!conflictMap.has(name)) {
              conflictMap.set(name, new Set());
            }
            conflictMap.get(name)?.add(version);
          }
        });

        const processedConflicts: ConflictData[] = Array.from(conflictMap.entries())
          .filter(([, versions]) => versions.size > 1)
          .map(([name, versions]) => ({
            name,
            versions: Array.from(versions)
          }));

        onConflictsUpdate(processedConflicts);
      }
    } catch (error) {
      console.error('Error loading conflicts:', error);
      onError(`Failed to load conflicts: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      onLoading(false);
    }
  };

  return (
    <div className="conflict-resolver">
      <Button
        variant="contained"
        onClick={loadConflicts}
        disabled={!projectPath}
        sx={{ mb: 2 }}
      >
        Scan Conflicts
      </Button>

      {conflicts.length > 0 ? (
        <List dense>
          {conflicts.map(c => (
            <ListItem key={c.name} secondaryAction={
              <Button
                size="small"
                variant="outlined"
                onClick={() => onUpdateDependencies(c.name)}
              >
                Update -p {c.name}
              </Button>
            }>
              <ListItemText
                primary={c.name}
                secondary={`versions: ${c.versions.join(', ')}`}
              />
            </ListItem>
          ))}
        </List>
      ) : (
        <Typography variant="body2">
          No conflicts found or not scanned.
        </Typography>
      )}
    </div>
  );
};

export default ConflictResolver;
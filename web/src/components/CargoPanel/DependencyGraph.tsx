/**
 * Component for visualizing dependency graphs in the Cargo Panel
 * Wraps the DependencyGraphRenderer with cargo-specific functionality
 */

import React from 'react';
import { DependencyGraphRenderer } from '../../features/cargoToml/dependencyGraph';
import type { CargoManifest } from '../../types/cargo';

interface DependencyGraphProps {
  manifest: CargoManifest;
  width?: number;
  height?: number;
}

/**
 * Renders a dependency graph for the given Cargo manifest
 */
export const DependencyGraph: React.FC<DependencyGraphProps> = ({
  manifest,
  width = 800,
  height = 600
}) => {
  return (
    <DependencyGraphRenderer
      manifest={manifest}
      width={width}
      height={height}
    />
  );
};

export default DependencyGraph;
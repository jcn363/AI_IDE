import { useMemo } from 'react';
import { CargoManifest } from '../../../types/cargo';
import {
  DependencyLink,
  DependencyNode,
  FeatureInfo,
  resolveFeatureDependencies,
} from './FeatureResolver';

export interface CrateInfo {
  version: string;
  features: Record<string, FeatureInfo>;
  defaultFeatures: string[];
  dependencies: Record<string, string>;
  buildDependencies: Record<string, string>;
  devDependencies: Record<string, string>;
}

export interface DependencyGraphOptions {
  includeDevDeps?: boolean;
  includeBuildDeps?: boolean;
  includeFeatures?: boolean;
  resolveFeatures?: boolean;
}

export interface DependencyGraphResult {
  nodes: DependencyNode[];
  links: DependencyLink[];
  crates: DependencyNode[];
  features: DependencyNode[];
}

/**
 * Hook that processes Cargo manifest dependencies into a graph structure
 */
export function useDependencyGraph(
  manifest: CargoManifest,
  options: DependencyGraphOptions = {}
): DependencyGraphResult {
  const {
    includeDevDeps = true,
    includeBuildDeps = true,
    includeFeatures = true,
    resolveFeatures = true,
  } = options;

  return useMemo(() => {
    const nodes: DependencyNode[] = [];
    const links: DependencyLink[] = [];
    const addedNodes = new Set<string>();
    const featureNodes = new Map<string, DependencyNode>();
    const crateNodes = new Map<string, DependencyNode>();

    // Add root package
    const rootId = manifest.package?.name || 'root';
    const rootNode: DependencyNode = {
      id: rootId,
      name: manifest.package?.name || 'root',
      type: 'workspace',
      version: manifest.package?.version,
      isRoot: true,
      isDirect: true,
      description: manifest.package?.description,
      homepage: manifest.package?.homepage,
      repository: manifest.package?.repository,
      documentation: manifest.package?.documentation,
      license: manifest.package?.license,
    };

    nodes.push(rootNode);
    addedNodes.add(rootId);
    crateNodes.set(rootId, rootNode);

    // Process dependencies
    type DependencyInfo =
      | string
      | {
          version?: string;
          features?: string[];
          optional?: boolean;
          default_features?: boolean;
        };

    type Dependencies = Record<string, DependencyInfo>;

    interface ProcessDepsOptions {
      isDev?: boolean;
      isBuild?: boolean;
      isOptional?: boolean;
      isDirect?: boolean;
    }

    const processDeps = (
      deps: Dependencies | undefined,
      parentId: string,
      options: ProcessDepsOptions = {}
    ) => {
      if (!deps) return;

      Object.entries(deps).forEach(([name, depInfo]) => {
        const depData = typeof depInfo === 'string' ? { version: depInfo } : depInfo;

        const depVersion = depData.version || '*'; // Use wildcard if version not specified
        const depId = `${name}@${depVersion}`;
        const isDirect = !parentId.includes('@');

        // Create or update crate node
        let crateNode = crateNodes.get(depId);
        if (!crateNode) {
          crateNode = {
            id: depId,
            name,
            type: 'crate',
            version: depData.version,
            isDirect,
            isDev: options.isDev,
            isBuild: options.isBuild,
            isOptional: depData.optional,
            features: [],
            resolvedFeatures: {},
          };

          nodes.push(crateNode);
          addedNodes.add(depId);
          crateNodes.set(depId, crateNode);
        } else {
          // Update existing node with additional metadata
          if (isDirect) crateNode.isDirect = true;
          if (options.isDev) crateNode.isDev = true;
          if (options.isBuild) crateNode.isBuild = true;
          if (depData.optional) crateNode.isOptional = true;
        }

        // Process features if enabled
        if (
          crateNode &&
          includeFeatures &&
          Array.isArray(depData.features) &&
          depData.features.length > 0
        ) {
          if (!crateNode.features) crateNode.features = [];

          // Add feature nodes and links
          depData.features.forEach((featureName) => {
            const featureId = `${depId}#${featureName}`;
            let featureNode = featureNodes.get(featureId);

            if (!featureNode) {
              featureNode = {
                id: featureId,
                name: featureName,
                type: 'feature',
                isDefault: depData.default_features === true,
                isOptional: depData.optional || false,
              };

              nodes.push(featureNode);
              addedNodes.add(featureId);
              featureNodes.set(featureId, featureNode);

              // Add feature to crate's features list if not already present
              if (crateNode?.features && !crateNode.features.includes(featureName)) {
                (crateNode.features as string[]).push(featureName);
              }
            }

            // Link feature to crate
            links.push({
              source: depId,
              target: featureId,
              type: 'feature',
              feature: featureName,
              optional: depData.optional,
            });

            // Link parent to feature if this is a feature dependency
            if (parentId.includes('#')) {
              links.push({
                source: parentId,
                target: featureId,
                type: 'depends',
                required: !depData.optional,
              });
            }
          });
        }

        // Link parent to crate
        links.push({
          source: parentId,
          target: depId,
          type: depData.optional ? 'optional' : 'depends',
          optional: depData.optional,
          label: options.isDev ? 'dev' : options.isBuild ? 'build' : undefined,
        });

        // Process default features if enabled
        if (includeFeatures && depData.default_features !== false) {
          const defaultFeatureId = `${depId}#default`;
          if (!featureNodes.has(defaultFeatureId)) {
            const defaultFeatureNode: DependencyNode = {
              id: defaultFeatureId,
              name: 'default',
              type: 'feature',
              isDefault: true,
              isOptional: true,
            };

            nodes.push(defaultFeatureNode);
            addedNodes.add(defaultFeatureId);
            featureNodes.set(defaultFeatureId, defaultFeatureNode);

            // Link default feature to crate
            links.push({
              source: depId,
              target: defaultFeatureId,
              type: 'default',
              optional: true,
            });
          }
        }
      });
    };

    // Process all dependencies
    const processDependencies = () => {
      // Process regular dependencies
      if (manifest.dependencies) {
        processDeps(manifest.dependencies, rootId, { isDirect: true });
      }

      // Process dev dependencies if enabled
      if (includeDevDeps && manifest['dev-dependencies']) {
        processDeps(manifest['dev-dependencies'], rootId, { isDev: true, isDirect: true });
      }

      // Process build dependencies if enabled
      if (includeBuildDeps && manifest['build-dependencies']) {
        processDeps(manifest['build-dependencies'], rootId, { isBuild: true, isDirect: true });
      }

      // Process workspace dependencies
      if (manifest.workspace?.dependencies) {
        processDeps(manifest.workspace.dependencies, rootId, { isDirect: true });
      }
    };

    // Process all dependencies
    processDependencies();

    // Resolve feature dependencies if enabled
    if (resolveFeatures && featureNodes.size > 0) {
      resolveFeatureDependencies(nodes, links, featureNodes, crateNodes);
    }

    return {
      nodes,
      links,
      crates: Array.from(crateNodes.values()),
      features: Array.from(featureNodes.values()),
    };
  }, [manifest, includeDevDeps, includeBuildDeps, includeFeatures, resolveFeatures]);
}

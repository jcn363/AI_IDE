export interface FeatureInfo {
  name: string;
  optional: boolean;
  default: boolean;
  requires: string[];
  description?: string;
}

export interface FeatureDefinition {
  name: string;
  default?: boolean;
  requires?: string[];
}

export type Feature = string | FeatureDefinition;

export interface DependencyNode {
  id: string;
  name: string;
  type: 'crate' | 'feature' | 'workspace';
  version?: string;
  features?: Feature[];
  resolvedFeatures?: Record<string, boolean>;
  isRoot?: boolean;
  isDirect?: boolean;
  isDev?: boolean;
  isBuild?: boolean;
  isOptional?: boolean;
  isDefault?: boolean;
  description?: string;
  homepage?: string;
  repository?: string;
  documentation?: string;
  license?: string;
}

export interface DependencyLink {
  source: string;
  target: string;
  type: 'depends' | 'feature' | 'optional' | 'default';
  label?: string;
  required?: boolean;
  optional?: boolean;
  feature?: string;
}

/**
 * Resolves feature dependencies within a dependency graph
 */
export const resolveFeatureDependencies = (
  nodes: DependencyNode[],
  links: DependencyLink[],
  featureNodes: Map<string, DependencyNode>,
  crateNodes: Map<string, DependencyNode>
) => {
  // First pass: Build a map of feature requirements
  const featureRequirements = new Map<string, Set<string>>();

  // Process each feature node to build requirements
  featureNodes.forEach((featureNode, featureId) => {
    if (!featureNode.name) return;

    const crateId = featureId.split('#')[0];
    const crateNode = crateNodes.get(crateId);
    if (!crateNode) return;

    // Define the feature type based on Cargo.toml structure
    type FeatureEntry = string | { name: string; requires?: string[] };

    // Get feature requirements from Cargo.toml
    const featureDef = crateNode.features?.find((f) => {
      if (typeof f === 'string') {
        return f === featureNode.name;
      }
      return f?.name === featureNode.name;
    });

    if (
      featureDef &&
      typeof featureDef === 'object' &&
      'requires' in featureDef &&
      Array.isArray(featureDef.requires)
    ) {
      featureRequirements.set(featureId, new Set(featureDef.requires));
    }
  });

  // Second pass: Resolve feature dependencies
  featureRequirements.forEach((requiredFeatures, featureId) => {
    const [crateId] = featureId.split('#');
    const crateNode = crateNodes.get(crateId);
    if (!crateNode) return;

    requiredFeatures.forEach((requiredFeature) => {
      // Check if the required feature is a direct feature of the same crate
      const targetFeatureId = `${crateId}#${requiredFeature}`;
      if (featureNodes.has(targetFeatureId)) {
        links.push({
          source: featureId,
          target: targetFeatureId,
          type: 'feature',
          required: true,
          label: 'requires',
        });
        return;
      }

      // Check if it's a feature from a dependency
      const depLink = links.find(
        (link) =>
          link.source === crateId &&
          link.type === 'depends' &&
          (link.target as string).startsWith(requiredFeature.split('=')[0])
      );

      if (depLink) {
        const depCrateId = depLink.target as string;
        const depFeatureName = requiredFeature.split('/').pop() || '';
        const depFeatureId = `${depCrateId}#${depFeatureName}`;

        if (featureNodes.has(depFeatureId)) {
          links.push({
            source: featureId,
            target: depFeatureId,
            type: 'feature',
            required: true,
            label: 'requires',
          });
        }
      }
    });
  });

  // Third pass: Resolve default features
  nodes.forEach((node) => {
    if (node.type === 'crate' && node.features?.length) {
      const defaultFeatures = node.features.filter((f): f is Feature => {
        if (typeof f === 'string') return false;
        return f !== null && typeof f === 'object' && 'default' in f && f.default === true;
      });

      defaultFeatures.forEach((feature) => {
        const featureName = typeof feature === 'string' ? feature : feature.name || '';
        const featureId = `${node.id}#${featureName}`;

        if (featureNodes.has(featureId)) {
          links.push({
            source: node.id,
            target: featureId,
            type: 'default',
            required: false,
            label: 'default',
          });
        }
      });
    }
  });
};

/**
 * Converts a feature array to resolved features record
 */
export const resolveFeaturesToRecord = (features: Feature[]): Record<string, boolean> => {
  const resolved: Record<string, boolean> = {};
  features.forEach((feature) => {
    if (typeof feature === 'string') {
      resolved[feature] = true;
    } else if (typeof feature === 'object' && feature.name) {
      resolved[feature.name] = feature.default !== false;
    }
  });
  return resolved;
};

/**
 * Checks if a feature is enabled based on feature definitions and requirements
 */
export const isFeatureEnabled = (
  featureName: string,
  enabledFeatures: string[],
  featureDefinitions: Record<string, Feature>
): boolean => {
  if (!featureDefinitions[featureName]) {
    return enabledFeatures.includes(featureName);
  }

  if (typeof featureDefinitions[featureName] === 'string') {
    return enabledFeatures.includes(featureName);
  }

  const featureDef = featureDefinitions[featureName] as FeatureDefinition;
  if (featureDef.requires) {
    // Feature is enabled only if all required features are also enabled
    return featureDef.requires.every((dep) =>
      isFeatureEnabled(dep, enabledFeatures, featureDefinitions)
    );
  }

  return enabledFeatures.includes(featureName);
};

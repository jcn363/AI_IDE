import { CargoManifest } from '../../types/cargo';

interface FeatureUsage {
  name: string;
  usedBy: string[];
  enabledByDefault: boolean;
  isUsed: boolean;
}

export async function analyzeFeatureFlags(manifest: CargoManifest): Promise<FeatureUsage[]> {
  const features = manifest.features || {};
  const dependencies = {
    ...(manifest.dependencies || {}),
    ...(manifest['dev-dependencies'] || {}),
  };
  const featureUsages: FeatureUsage[] = [];

  // Analyze each feature
  for (const [featureName, featureDeps] of Object.entries(features)) {
    const usedBy: string[] = [];

    // Check if feature is used by any dependency
    for (const [depName, depInfo] of Object.entries(dependencies)) {
      const depFeatures = typeof depInfo === 'object' ? depInfo.features || [] : [];

      // Handle different feature specification formats
      if (Array.isArray(depFeatures) && depFeatures.includes(featureName)) {
        usedBy.push(depName);
      } else if (typeof depFeatures === 'string' && depFeatures === featureName) {
        usedBy.push(depName);
      } else if (
        typeof depFeatures === 'boolean' &&
        depFeatures === true &&
        featureName === 'default'
      ) {
        usedBy.push(depName);
      }
    }

    // Check if feature is enabled by default
    const defaultFeatures = manifest['package']?.defaultFeatures;
    const isDefault = Array.isArray(defaultFeatures) && defaultFeatures.includes(featureName);

    featureUsages.push({
      name: featureName,
      usedBy,
      enabledByDefault: isDefault,
      isUsed: usedBy.length > 0 || isDefault,
    });
  }

  return featureUsages;
}

export function optimizeFeatureFlags(
  manifest: CargoManifest,
  features: FeatureUsage[]
): CargoManifest {
  const optimizedManifest: CargoManifest = {
    ...manifest,
    features: manifest.features ? { ...manifest.features } : manifest.features,
    package: manifest.package ? { ...manifest.package } : manifest.package,
  };

  // Remove unused features
  const unusedFeatures = features.filter((f) => !f.isUsed);
  unusedFeatures.forEach((feature) => {
    if (optimizedManifest.features) {
      delete optimizedManifest.features[feature.name];
    }
  });

  // Optimize default features
  const defaultFeatures = features.filter((f) => f.enabledByDefault && !f.isUsed);
  if (defaultFeatures.length > 0 && optimizedManifest.package) {
    optimizedManifest.package.defaultFeatures = (
      optimizedManifest.package.defaultFeatures || []
    ).filter((f: string) => !defaultFeatures.some((df) => df.name === f));
  }

  return optimizedManifest;
}

export function getFeatureFlagSuggestions(features: FeatureUsage[] | undefined): string[] {
  const suggestions: string[] = [];

  if (!features || !Array.isArray(features)) {
    return suggestions;
  }

  features.forEach((feature) => {
    if (!feature.isUsed && !feature.enabledByDefault) {
      suggestions.push(`Unused feature "${feature.name}" can be safely removed`);
    } else if (feature.enabledByDefault && feature.usedBy.length === 0) {
      suggestions.push(
        `Default feature "${feature.name}" is not used by any dependency and can be disabled by default`
      );
    }
  });

  return suggestions;
}

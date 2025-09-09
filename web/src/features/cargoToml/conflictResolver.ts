import semver, { Range } from 'semver';
import { CargoManifest, CargoDependency } from '../../types/cargo';

export interface VersionConflict {
  package: string;
  requestedVersions: {
    version: string;
    by: string[];
  }[];
  resolution?: {
    version: string;
    reason: string;
  };
}

export interface ResolutionStrategy {
  preferHighest?: boolean;
  preferLowest?: boolean;
  preferLatestPatch?: boolean;
  preferLatestMinor?: boolean;
  preferStable?: boolean;
}

export class DependencyConflictResolver {
  private manifest: CargoManifest;
  private lockfile: any;
  private strategy: ResolutionStrategy;

  constructor(manifest: CargoManifest, lockfile: any = {}, strategy: ResolutionStrategy = {}) {
    this.manifest = manifest;
    this.lockfile = lockfile;
    this.strategy = {
      preferStable: true,
      ...strategy,
    };
  }

  /**
   * Find all version conflicts in the dependency tree
   */
  findConflicts(): VersionConflict[] {
    const deps = this.collectDependencies();
    const conflicts: VersionConflict[] = [];

    for (const [name, versions] of Object.entries(deps)) {
      if (versions.size > 1) {
        conflicts.push({
          package: name,
          requestedVersions: Array.from(versions).map(([version, requesters]) => ({
            version,
            by: Array.from(requesters),
          })),
        });
      }
    }

    return conflicts;
  }

  /**
   * Resolve all conflicts using the current strategy
   */
  resolveConflicts(conflicts: VersionConflict[]): VersionConflict[] {
    return conflicts.map(conflict => ({
      ...conflict,
      resolution: this.resolveConflict(conflict),
    }));
  }

  /**
   * Resolve a single conflict using the current strategy
   */
  private resolveConflict(conflict: VersionConflict): { version: string; reason: string } {
    const versions = conflict.requestedVersions.map(v => v.version);
    
    // Try to find a version that satisfies all constraints
    const allRanges = versions.map(v => new semver.Range(v));
    const allVersions = this.getAllVersions(conflict.package);
    
    // Sort versions according to strategy
    const sortedVersions = [...allVersions].sort((a, b) => {
      if (this.strategy.preferStable) {
        if (semver.prerelease(a) && !semver.prerelease(b)) return 1;
        if (!semver.prerelease(a) && semver.prerelease(b)) return -1;
      }
      
      if (this.strategy.preferHighest) {
        return semver.rcompare(a, b);
      }
      
      if (this.strategy.preferLowest) {
        return semver.compare(a, b);
      }
      
      // Default: prefer highest version that satisfies all constraints
      return semver.rcompare(a, b);
    });
    
    // Find the first version that satisfies all constraints
    for (const version of sortedVersions) {
      if (allRanges.every(range => semver.satisfies(version, range))) {
        return {
          version,
          reason: `Version ${version} satisfies all constraints and is ${this.strategy.preferStable ? 'stable' : 'the latest'}`,
        };
      }
    }
    
    // If no version satisfies all constraints, use the highest version
    return {
      version: sortedVersions[0],
      reason: 'No version satisfies all constraints, using highest version',
    };
  }

  /**
   * Collect all dependencies and their requested versions
   */
  private collectDependencies(): Record<string, Map<string, Set<string>>> {
    const deps: Record<string, Map<string, Set<string>>> = {};
    
    const addDependency = (name: string, version: string, requester: string) => {
      if (!deps[name]) {
        deps[name] = new Map();
      }
      
      if (!deps[name].has(version)) {
        deps[name].set(version, new Set());
      }
      
      deps[name].get(version)?.add(requester);
    };
    
    // Process direct dependencies
    this.processDependencySection(this.manifest.dependencies, 'root', addDependency);
    this.processDependencySection(this.manifest['dev-dependencies'], 'root', addDependency);
    this.processDependencySection(this.manifest['build-dependencies'], 'root', addDependency);
    
    // Process lockfile if available
    if (this.lockfile.package) {
      for (const pkg of this.lockfile.package) {
        if (pkg.dependencies) {
          for (const dep of pkg.dependencies) {
            addDependency(dep.name, dep.version, pkg.name);
          }
        }
      }
    }
    
    return deps;
  }
  
  private processDependencySection(
    deps: Record<string, CargoDependency> | undefined,
    requester: string,
    callback: (name: string, version: string, requester: string) => void,
  ) {
    if (!deps) return;
    
    for (const [name, dep] of Object.entries(deps)) {
      if (typeof dep === 'string') {
        callback(name, dep, requester);
      } else if (dep && typeof dep === 'object' && 'version' in dep && dep.version) {
        // Only call the callback if version is defined
        callback(name, dep.version, requester);
      }
    }
  }
  
  /**
   * Get all available versions for a package from the lockfile
   */
  private getAllVersions(packageName: string): string[] {
    if (!this.lockfile.package) return [];
    
    return this.lockfile.package
      .filter((pkg: any) => pkg.name === packageName)
      .map((pkg: any) => pkg.version)
      .filter((v: string | null): v is string => v !== null && semver.valid(v) !== null);
  }
}

/**
 * Helper function to detect and resolve conflicts
 */
export function detectAndResolveConflicts(
  manifest: CargoManifest,
  lockfile: any = {},
  strategy: ResolutionStrategy = {}
): VersionConflict[] {
  const resolver = new DependencyConflictResolver(manifest, lockfile, strategy);
  const conflicts = resolver.findConflicts();
  return resolver.resolveConflicts(conflicts);
}

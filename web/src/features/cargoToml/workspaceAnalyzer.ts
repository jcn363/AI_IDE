import { CargoDependency, CargoManifest } from '../../types/cargo';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export interface WorkspaceMember {
  name: string;
  path: string;
  manifest: CargoManifest;
  dependencies: string[];
  inheritedDependencies: Record<string, string>; // name -> version
  directDependencies: Record<string, string>; // name -> version
}

export interface WorkspaceAnalysis {
  root: CargoManifest;
  members: WorkspaceMember[];
  workspaceDependencies: Record<string, string>; // name -> version
  inheritanceGraph: Record<string, string[]>; // member -> inherited members
}

/**
 * Parse a Cargo.toml file and return its content as a JavaScript object
 */
async function parseCargoToml(filePath: string): Promise<CargoManifest> {
  try {
    // In a real implementation, you would use a TOML parser here
    // This is a simplified version
    const { stdout } = await execAsync(`cat "${filePath}"`);
    // @ts-ignore - Assume we have a TOML parser available
    return window.toml.parse(stdout);
  } catch (error) {
    console.error(`Error parsing ${filePath}:`, error);
    throw error;
  }
}

/**
 * Get the list of workspace members by parsing the Cargo.toml files
 */
async function getWorkspaceMembers(
  rootPath: string,
  rootManifest: CargoManifest
): Promise<WorkspaceMember[]> {
  if (!rootManifest.workspace?.members) {
    return [];
  }

  const members: WorkspaceMember[] = [];

  for (const memberPath of rootManifest.workspace.members) {
    try {
      const fullPath = `${rootPath}/${memberPath}/Cargo.toml`;
      const manifest = await parseCargoToml(fullPath);

      members.push({
        name: manifest.package?.name || memberPath.split('/').pop() || memberPath,
        path: memberPath,
        manifest,
        dependencies: [],
        inheritedDependencies: {},
        directDependencies: { ...getDependencies(manifest) },
      });
    } catch (error) {
      console.error(`Error processing workspace member ${memberPath}:`, error);
    }
  }

  return members;
}

/**
 * Extract dependencies from a manifest
 */
function getDependencies(manifest: CargoManifest): Record<string, string> {
  const deps: Record<string, string> = {};

  const addDeps = (depsObj: Record<string, CargoDependency> | undefined) => {
    if (!depsObj) return;

    Object.entries(depsObj).forEach(([name, dep]) => {
      if (typeof dep === 'string') {
        deps[name] = dep;
      } else if (dep.version) {
        deps[name] = dep.version;
      }
    });
  };

  addDeps(manifest.dependencies);
  addDeps(manifest['dev-dependencies']);
  addDeps(manifest['build-dependencies']);

  return deps;
}

/**
 * Analyze workspace inheritance and dependencies
 */
async function analyzeWorkspaceInheritance(
  rootPath: string,
  rootManifest: CargoManifest
): Promise<WorkspaceAnalysis> {
  const members = await getWorkspaceMembers(rootPath, rootManifest);
  const inheritanceGraph: Record<string, string[]> = {};

  // First pass: build inheritance graph and collect workspace dependencies
  const workspaceDependencies: Record<string, string> = {};

  // Add root workspace dependencies
  if (rootManifest.dependencies) {
    Object.entries(rootManifest.dependencies).forEach(([name, dep]) => {
      if (typeof dep === 'string') {
        workspaceDependencies[name] = dep;
      } else if (dep.version) {
        workspaceDependencies[name] = dep.version;
      }
    });
  }

  // Process each member
  for (const member of members) {
    inheritanceGraph[member.name] = [];

    // Check for workspace inheritance
    if (member.manifest.workspace) {
      // This member inherits from the workspace
      inheritanceGraph[member.name].push('workspace');

      // Copy workspace dependencies to inheritedDependencies
      member.inheritedDependencies = { ...workspaceDependencies };
    }

    // Check for member-to-member dependencies
    for (const depName of Object.keys(member.directDependencies)) {
      const depMember = members.find((m) => m.name === depName);
      if (depMember) {
        inheritanceGraph[member.name].push(depName);
      }
    }

    // Merge direct and inherited dependencies
    member.dependencies = [
      ...new Set([
        ...Object.keys(member.directDependencies),
        ...Object.keys(member.inheritedDependencies),
      ]),
    ];
  }

  return {
    root: rootManifest,
    members,
    workspaceDependencies,
    inheritanceGraph,
  };
}

/**
 * Get the latest version of a crate from crates.io
 */
async function getLatestVersion(crateName: string): Promise<string> {
  try {
    const response = await fetch(`https://crates.io/api/v1/crates/${crateName}`);
    if (!response.ok) {
      throw new Error(`Failed to fetch crate info for ${crateName}`);
    }

    const data = await response.json();
    return data.crate?.newest_version || 'unknown';
  } catch (error) {
    console.error(`Error fetching latest version for ${crateName}:`, error);
    return 'unknown';
  }
}

/**
 * Check for outdated dependencies across the workspace
 */
async function checkForOutdatedDependencies(analysis: WorkspaceAnalysis) {
  const allDeps = new Set<string>();

  // Collect all unique dependencies
  analysis.members.forEach((member) => {
    Object.keys(member.directDependencies).forEach((dep) => allDeps.add(dep));
  });

  // Check each dependency
  const updates: Array<{
    name: string;
    currentVersion: string;
    latestVersion: string;
    usedIn: Array<{ member: string; version: string }>;
  }> = [];

  for (const dep of Array.from(allDeps)) {
    // Skip workspace members
    if (analysis.members.some((m) => m.name === dep)) continue;

    // Find all versions used in the workspace
    const usages: Array<{ member: string; version: string }> = [];

    for (const member of analysis.members) {
      const version = member.directDependencies[dep];
      if (version) {
        usages.push({ member: member.name, version });
      }
    }

    if (usages.length > 0) {
      const latestVersion = await getLatestVersion(dep);
      updates.push({
        name: dep,
        currentVersion: usages[0].version, // Just use the first version found
        latestVersion,
        usedIn: usages,
      });
    }
  }

  return updates;
}

export {
  parseCargoToml,
  getWorkspaceMembers,
  analyzeWorkspaceInheritance,
  checkForOutdatedDependencies,
  getLatestVersion,
};

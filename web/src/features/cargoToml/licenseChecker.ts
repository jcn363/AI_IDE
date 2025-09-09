import { CargoDependency, CargoManifest } from '../../types/cargo';

export interface LicenseInfo {
  package: string;
  version: string;
  license?: string;
  license_file?: string;
  repository?: string;
  homepage?: string;
  isApproved: boolean;
  isCopyleft: boolean;
  isBanned: boolean;
  notes?: string;
}

// Common open source licenses grouped by category
const LICENSE_CATEGORIES = {
  // Permissive licenses (generally safe)
  PERMISSIVE: [
    'MIT', 'Apache-2.0', 'BSD-2-Clause', 'BSD-3-Clause', 'ISC', 
    'Unlicense', '0BSD', 'Apache 2.0', 'MIT/Apache-2.0', 'Zlib',
    'CC0-1.0', 'BSL-1.0', 'Unlicense OR MIT', 'Apache-2.0 OR MIT',
  ],
  
  // Copyleft licenses (may have implications for commercial use)
  COPYLEFT: [
    'GPL-2.0', 'GPL-3.0', 'AGPL-3.0', 'LGPL-2.1', 'LGPL-3.0',
    'MPL-2.0', 'EPL-1.0', 'EPL-2.0', 'CDDL-1.0', 'CPAL-1.0',
  ],
  
  // Problematic or banned licenses
  BANNED: [
    'SSPL-1.0', 'JSON', 'CC-BY-SA-4.0', 'CC-BY-NC-*', 'CC-BY-ND-*',
    'GPL-1.0', 'AGPL-1.0', 'Sleepycat', 'Jabber-1.0', 'Netscape-1.1',
    'Artistic-1.0', 'Artistic-2.0', 'NPL-1.0', 'QPL-1.0', 'WTFPL',
  ],
};

// License compatibility matrix (simplified)
const LICENSE_COMPATIBILITY: Record<string, string[]> = {
  'MIT': ['MIT', 'Apache-2.0', 'BSD-2-Clause', 'BSD-3-Clause', 'ISC', 'Unlicense'],
  'Apache-2.0': ['Apache-2.0', 'MIT', 'BSD-3-Clause'],
  'GPL-3.0': ['GPL-3.0', 'LGPL-3.0', 'AGPL-3.0'],
  'MPL-2.0': ['MPL-2.0', 'MIT', 'Apache-2.0', 'GPL-2.0', 'LGPL-2.1'],
};

// Cache for license information
const licenseCache = new Map<string, Promise<LicenseInfo>>();

async function fetchLicenseInfo(packageName: string, version: string): Promise<LicenseInfo> {
  const cacheKey = `${packageName}@${version}`;
  
  if (licenseCache.has(cacheKey)) {
    return licenseCache.get(cacheKey)!;
  }
  
  const fetchPromise = (async () => {
    try {
      const response = await fetch(`https://crates.io/api/v1/crates/${packageName}/${version}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch crate info for ${packageName}@${version}`);
      }
      
      const data = await response.json();
      const versionInfo = data.versions.find((v: any) => v.num === version) || data.versions[0];
      
      const license = versionInfo.license || data.crate?.license || '';
      const licenseNormalized = license.replace(/\s*OR\s*/g, '/').split('/')[0].trim();
      
      const isApproved = LICENSE_CATEGORIES.PERMISSIVE.includes(licenseNormalized);
      const isCopyleft = LICENSE_CATEGORIES.COPYLEFT.some(l => license.includes(l));
      const isBanned = LICENSE_CATEGORIES.BANNED.some(l => 
        (l.endsWith('*') ? license.startsWith(l.slice(0, -1)) : license === l),
      );
      
      return {
        package: packageName,
        version,
        license: versionInfo.license || data.crate?.license,
        repository: data.crate?.repository || data.crate?.homepage,
        homepage: data.crate?.homepage || data.crate?.documentation,
        isApproved,
        isCopyleft,
        isBanned,
        notes: isBanned ? 'This license may have legal implications' : 
               isCopyleft ? 'Copyleft license - check compatibility' : '',
      };
    } catch (error) {
      console.error(`Error fetching license info for ${packageName}@${version}:`, error);
      return {
        package: packageName,
        version,
        isApproved: false,
        isCopyleft: false,
        isBanned: false,
        notes: 'Could not determine license information',
      };
    }
  })();
  
  licenseCache.set(cacheKey, fetchPromise);
  return fetchPromise;
}

export async function checkLicenseCompliance(manifest: CargoManifest): Promise<LicenseInfo[]> {
  const results: LicenseInfo[] = [];
  
  const checkDependencies = async (deps: Record<string, CargoDependency> | undefined) => {
    if (!deps) return;
    
    await Promise.all(
      Object.entries(deps).map(async ([name, depInfo]) => {
        const version = typeof depInfo === 'string' ? depInfo : depInfo.version;
        if (!version) return;
        
        const info = await fetchLicenseInfo(name, version);
        results.push(info);
      }),
    );
  };
  
  // Check all dependency types
  await checkDependencies(manifest.dependencies);
  await checkDependencies(manifest['dev-dependencies']);
  await checkDependencies(manifest['build-dependencies']);
  
  return results.sort((a, b) => {
    // Sort by: banned > copyleft > unapproved > approved
    if (a.isBanned !== b.isBanned) return a.isBanned ? -1 : 1;
    if (a.isCopyleft !== b.isCopyleft) return a.isCopyleft ? -1 : 1;
    if (a.isApproved !== b.isApproved) return a.isApproved ? 1 : -1;
    return a.package.localeCompare(b.package);
  });
}

export function getLicenseSummary(licenses: LicenseInfo[]): {
  total: number;
  approved: number;
  copyleft: number;
  banned: number;
  unknown: number;
} {
  return licenses.reduce((acc, license) => {
    acc.total+=1;
    if (license.isBanned) acc.banned+=1;
    else if (license.isCopyleft) acc.copyleft+=1;
    else if (license.isApproved) acc.approved+=1;
    else acc.unknown+=1;
    return acc;
  }, { total: 0, approved: 0, copyleft: 0, banned: 0, unknown: 0 });
}

export function checkLicenseCompatibility(
  projectLicense: string, 
  dependencyLicenses: LicenseInfo[],
): { compatible: boolean; conflicts: Array<{package: string; license: string}> } {
  const projectLicenses = projectLicense.split('/').map(l => l.trim());
  const conflicts: Array<{package: string; license: string}> = [];
  
  for (const dep of dependencyLicenses) {
    if (!dep.license) continue;
    
    const depLicenses = dep.license.split('/').map(l => l.trim());
    let isCompatible = false;
    
    for (const projLicense of projectLicenses) {
      const compatibleLicenses = LICENSE_COMPATIBILITY[projLicense] || [];
      if (depLicenses.some(dl => compatibleLicenses.includes(dl))) {
        isCompatible = true;
        break;
      }
    }
    
    if (!isCompatible) {
      conflicts.push({
        package: dep.package,
        license: dep.license,
      });
    }
  }
  
  return {
    compatible: conflicts.length === 0,
    conflicts,
  };
}

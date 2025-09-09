import { CargoDependency, CargoManifest } from '../../types/cargo';

export interface Vulnerability {
  id: string;
  package: string;
  version: string;
  severity: 'critical' | 'high' | 'medium' | 'low';
  title: string;
  description: string;
  patched_versions?: string;
  url: string;
}

const RUSTSEC_API_URL = 'https://rustsec.org/advisory-db';
const CACHE_DURATION = 24 * 60 * 60 * 1000; // 24 hours

interface VulnerabilityCache {
  timestamp: number;
  data: Record<string, Vulnerability[]>;
}

let vulnerabilityCache: VulnerabilityCache = {
  timestamp: 0,
  data: {},
};

async function fetchVulnerabilities(): Promise<Record<string, Vulnerability[]>> {
  const now = Date.now();
  
  // Return cached data if still valid
  if (now - vulnerabilityCache.timestamp < CACHE_DURATION) {
    return vulnerabilityCache.data;
  }

  try {
    const response = await fetch(`${RUSTSEC_API_URL}/vulns.json`);
    if (!response.ok) throw new Error('Failed to fetch vulnerability data');
    
    const vulnerabilities: Vulnerability[] = await response.json();
    const vulnMap: Record<string, Vulnerability[]> = {};
    
    // Group vulnerabilities by package name
    for (const vuln of vulnerabilities) {
      if (!vulnMap[vuln.package]) {
        vulnMap[vuln.package] = [];
      }
      vulnMap[vuln.package].push(vuln);
    }
    
    // Update cache
    vulnerabilityCache = {
      timestamp: now,
      data: vulnMap,
    };
    
    return vulnMap;
  } catch (error) {
    console.error('Error fetching vulnerability data:', error);
    return vulnerabilityCache.data; // Return stale data if available
  }
}

function isVulnerable(version: string, vuln: Vulnerability): boolean {
  try {
    // Simple version check - in a real app, use proper semver comparison
    return version === vuln.version || 
           version === '*' || 
           version.startsWith(`^${  vuln.version}`) ||
           version.startsWith(`~${  vuln.version}`) ||
           (version.startsWith('>=') && vuln.version >= version.slice(2));
  } catch (e) {
    console.warn(`Error comparing versions ${version} and ${vuln.version}:`, e);
    return false;
  }
}

export async function scanForVulnerabilities(manifest: CargoManifest): Promise<Vulnerability[]> {
  const vulnerabilities: Vulnerability[] = [];
  const vulnMap = await fetchVulnerabilities();
  
  const checkDependencies = (deps: Record<string, CargoDependency> | undefined) => {
    if (!deps) return;
    
    Object.entries(deps).forEach(([name, depInfo]) => {
      const version = typeof depInfo === 'string' ? depInfo : depInfo.version;
      if (!version) return;
      
      const pkgVulns = vulnMap[name];
      if (!pkgVulns) return;
      
      pkgVulns.forEach(vuln => {
        if (isVulnerable(version, vuln)) {
          vulnerabilities.push(vuln);
        }
      });
    });
  };
  
  // Check all dependency types
  checkDependencies(manifest.dependencies);
  checkDependencies(manifest['dev-dependencies']);
  checkDependencies(manifest['build-dependencies']);
  
  return vulnerabilities.sort((a, b) => {
    const severityOrder: Record<string, number> = {
      'critical': 0, 'high': 1, 'medium': 2, 'low': 3,
    };
    return severityOrder[a.severity] - severityOrder[b.severity];
  });
}

export function getVulnerabilitySummary(vulnerabilities: Vulnerability[]): string {
  if (vulnerabilities.length === 0) {
    return 'No known security vulnerabilities found.';
  }
  
  const counts = vulnerabilities.reduce((acc, vuln) => {
    acc[vuln.severity] = (acc[vuln.severity] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);
  
  const summary = Object.entries(counts)
    .map(([severity, count]) => `${count} ${severity}`)
    .join(', ');
    
  return `Found ${vulnerabilities.length} vulnerabilities (${summary})`;
}

export function getFixSuggestion(vuln: Vulnerability): string {
  if (vuln.patched_versions) {
    return `Update to version ${vuln.patched_versions} or later.`;
  }
  return 'No specific fix version available. Consider removing or replacing this dependency.';
}

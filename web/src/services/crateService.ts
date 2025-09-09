interface CrateInfo {
  name: string;
  max_version: string;
  description: string;
  documentation: string;
  repository: string;
  downloads: number;
  recent_downloads: number;
  versions: CrateVersion[];
  keywords: string[];
  categories: string[];
  features: Record<string, string[]>;
}

interface CrateVersion {
  num: string;
  features: Record<string, string[]>;
  yanked: boolean;
}

import { ApiClient, ApiResponse } from '../shared/services/api';

const apiClient = new ApiClient({
  baseUrl: 'https://crates.io',
});

export class CrateService {
  private static instance: CrateService;
  private cache: Map<string, CrateInfo> = new Map();

  private constructor() {}

  public static getInstance(): CrateService {
    if (!CrateService.instance) {
      CrateService.instance = new CrateService();
    }
    return CrateService.instance;
  }

  public async getCrateInfo(name: string): Promise<CrateInfo | null> {
    // Check cache first
    if (this.cache.has(name)) {
      return this.cache.get(name) || null;
    }

    try {
      const response: ApiResponse<{ crate: CrateInfo }> = await apiClient.get(`api/v1/crates/${name}`);
      const crateInfo: CrateInfo = response.data.crate;
      
      // Cache the result
      this.cache.set(name, crateInfo);
      
      return crateInfo;
    } catch (error) {
      console.error('Error fetching crate info:', error);
      return null;
    }
  }

  public async getCrateVersions(name: string): Promise<CrateVersion[]> {
    const info = await this.getCrateInfo(name);
    return info?.versions || [];
  }

  public async getCrateFeatures(name: string, version?: string): Promise<Record<string, string[]>> {
    const info = await this.getCrateInfo(name);
    if (!info) return {};

    const targetVersion = version || info.max_version;
    const versionInfo = info.versions.find(v => v.num === targetVersion);
    
    return versionInfo?.features || info.features || {};
  }

  public async searchCrates(query: string): Promise<Array<{ name: string; description: string }>> {
    try {
      const response: ApiResponse<{ crates: any[] }> = await apiClient.get(`api/v1/crates`, {
        params: { q: query, per_page: '10' }
      });

      return response.data.crates.map((crate: any) => ({
        name: crate.name,
        description: crate.description || 'No description available',
      }));
    } catch (error) {
      console.error('Error searching crates:', error);
      return [];
    }
  }
}

export const crateService = CrateService.getInstance();

import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface CrateMetrics {
  build_time: number; // in milliseconds
  codegen_time: number; // in milliseconds
  codegen_units: number;
  incremental: boolean;
  dependencies: string[];
  features: string[];
}

export interface PerformanceMetrics {
  total_time: number; // in milliseconds
  crates: Record<string, CrateMetrics>;
  dependencies: Record<string, number>; // in milliseconds
  features: Record<string, string[]>; // Map of crate names to their enabled features
}

interface BackendCrateMetrics {
  build_time: { secs: number; nanos: number };
  codegen_time: { secs: number; nanos: number };
  codegen_units: number;
  incremental: boolean;
  dependencies: string[];
  features: string[];
}

interface BackendPerformanceMetrics {
  total_time: { secs: number; nanos: number };
  crates: Record<string, BackendCrateMetrics>;
  dependencies: Record<string, { secs: number; nanos: number }>;
  features: Record<string, string[]>;
}

const durationToMs = (duration: { secs: number; nanos: number }): number => {
  return duration.secs * 1000 + duration.nanos / 1_000_000;
};

const convertBackendMetrics = (data: BackendPerformanceMetrics): PerformanceMetrics => {
  return {
    total_time: durationToMs(data.total_time),
    crates: Object.fromEntries(
      Object.entries(data.crates).map(([name, crateData]) => [
        name,
        {
          build_time: durationToMs(crateData.build_time),
          codegen_time: durationToMs(crateData.codegen_time),
          codegen_units: crateData.codegen_units,
          incremental: crateData.incremental,
          dependencies: crateData.dependencies,
          features: crateData.features,
        },
      ])
    ),
    dependencies: Object.fromEntries(
      Object.entries(data.dependencies).map(([name, duration]) => [name, durationToMs(duration)])
    ),
    features: data.features,
  };
};

export const usePerformanceAnalysis = (projectPath: string) => {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [metrics, setMetrics] = useState<PerformanceMetrics | null>(null);

  const analyzePerformance = useCallback(
    async (path: string, release: boolean, incremental: boolean) => {
      if (!path) {
        setError('No project path provided');
        return null;
      }

      console.log('Analyzing performance with:', { path, release, incremental });

      setIsLoading(true);
      setError(null);

      try {
        console.log('Invoking cargo_analyze_performance...');
        const result = await invoke<BackendPerformanceMetrics>('cargo_analyze_performance', {
          projectPath: path,
          release,
          incremental,
        });
        console.log('Received performance metrics:', result);

        const convertedMetrics = convertBackendMetrics(result);
        setMetrics(convertedMetrics);
        return convertedMetrics;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Failed to analyze performance';
        setError(errorMessage);
        console.error('Performance analysis failed:', err);
        return null;
      } finally {
        setIsLoading(false);
      }
    },
    [projectPath]
  );

  return {
    isLoading,
    error,
    metrics,
    analyzePerformance,
  };
};

export default usePerformanceAnalysis;

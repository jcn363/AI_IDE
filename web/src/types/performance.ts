export interface CrateMetrics {
  build_time: number;
  codegen_time: number;
  codegen_units: number;
  incremental: boolean;
  features: string[];
  dependencies: string[];
  [key: string]: any;
}

export interface PerformanceMetrics {
  total_time: number;
  crates: Record<string, CrateMetrics>;
  dependencies: Record<string, number>;
  target: string;
  release: boolean;
  incremental: boolean;
  [key: string]: any;
}

export interface OptimizationSuggestion {
  type: string;
  message: string;
  details?: string;
  severity: 'info' | 'warning' | 'error';
}

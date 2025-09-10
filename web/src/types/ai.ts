// TypeScript definitions for AI/ML services
// These types mirror the Rust shared types for type safety

export interface VectorSearchRequest {
  query_vector: number[];
  top_k: number;
  filters?: SearchFilter[];
  config: VectorSearchConfig;
}

export interface VectorSearchConfig {
  algorithm: SearchAlgorithm;
  include_content: boolean;
  min_similarity?: number;
  collections: string[];
}

export enum SearchAlgorithm {
  Similarity = 'Similarity',
  Hybrid = 'Hybrid',
  Sparse = 'Sparse',
}

export interface SearchFilter {
  field: string;
  operator: FilterOperator;
  value: any;
}

export enum FilterOperator {
  Equal = 'Equal',
  NotEqual = 'NotEqual',
  GreaterThan = 'GreaterThan',
  LessThan = 'LessThan',
  Contains = 'Contains',
}

export interface VectorSearchResult {
  id: string;
  score: number;
  content?: string;
  file_path?: string;
  line_number?: number;
  metadata: Record<string, any>;
}

export interface InferenceRequest {
  model_name: string;
  input: any;
  ab_test_name?: string;
  config: InferenceConfig;
}

export interface InferenceConfig {
  temperature?: number;
  max_tokens?: number;
  top_p?: number;
  frequency_penalty?: number;
  presence_penalty?: number;
  stream: boolean;
  custom_params: Record<string, any>;
}

export interface InferenceResult {
  output: any;
  inference_time_ms: number;
  model_used: string;
  confidence_score?: number;
}

export interface CodeSearchRequest {
  query: string;
  languages: string[];
  file_patterns: string[];
  max_code_chunk_length: number;
  include_docs: boolean;
  ranking: SearchRanking;
}

export interface SearchRanking {
  semantic_weight: number;
  exact_match_weight: number;
  proximity_weight: number;
  recency_factor: number;
}

export enum CodeResultType {
  Function = 'Function',
  Class = 'Class',
  Struct = 'Struct',
  Method = 'Method',
  Variable = 'Variable',
  Import = 'Import',
  Comment = 'Comment',
  Documentation = 'Documentation',
  Other = 'Other',
}

export interface CodeSearchResult {
  id: string;
  code_snippet: string;
  file_path: string;
  line_number: number;
  language: string;
  score: number;
  result_type: CodeResultType;
  context: ContextLine[];
  highlights: HighlightSpan[];
}

export interface ContextLine {
  line_number: number;
  content: string;
  is_highlighted: boolean;
}

export interface HighlightSpan {
  start: number;
  end: number;
  match_type: MatchType;
}

export enum MatchType {
  Exact = 'Exact',
  Fuzzy = 'Fuzzy',
  Semantic = 'Semantic',
}

export interface ABTestConfiguration {
  model_a: string;
  model_b: string;
  traffic_split: number;
  enabled: boolean;
  start_time?: number;
  end_time?: number;
}

export interface ABTestResults {
  config: ABTestConfiguration;
  traffic_stats: Record<string, TrafficStatistics>;
  performance_stats: Record<string, PerformanceStatistics>;
  winner_confidence: number;
  recommended_winner?: string;
}

export interface TrafficStatistics {
  total_requests: number;
  error_count: number;
  avg_response_time_ms: number;
}

export interface PerformanceStatistics {
  avg_inference_time_ms: number;
  avg_confidence: number;
  total_tokens_processed: number;
  error_rate: number;
}

export enum WorkerTaskStatus {
  Pending = 'Pending',
  Running = 'Running',
  Completed = 'Completed',
  Failed = 'Failed',
  Cancelled = 'Cancelled',
}

export enum TaskType {
  Inference = 'Inference',
  VectorIndexing = 'VectorIndexing',
  CodeAnalysis = 'CodeAnalysis',
  SearchIndexing = 'SearchIndexing',
  Training = 'Training',
  Other = 'Other',
}

export interface TaskHandle {
  id: string;
  task_type: TaskType;
  status: WorkerTaskStatus;
  progress: number;
  start_time?: number;
  end_time?: number;
  error_message?: string;
  metadata: Record<string, any>;
}

export interface GPUMetrics {
  device: string;
  gpu_utilization: number;
  memory_used: number;
  memory_total: number;
  temperature?: number;
  power_usage?: number;
  inference_time_ms?: number;
  batch_size?: number;
  active_model?: string;
  timestamp: number;
}

export interface PerformanceMetrics {
  system_health: number;
  memory_stats: MemoryMetrics;
  gpu_stats: GPUMetrics[];
  model_stats: Record<string, ModelPerformance>;
  cache_stats: CacheMetrics;
  active_tasks: TaskHandle[];
  timestamp: number;
}

export interface MemoryMetrics {
  used_mb: number;
  total_mb: number;
  page_faults: number;
  cache_hits: number;
  cache_misses: number;
}

export interface ModelPerformance {
  inference_count: number;
  avg_inference_time_ms: number;
  error_rate: number;
  last_used: number;
}

export interface CacheMetrics {
  total_requests: number;
  hit_rate: number;
  size_mb: number;
  entries_count: number;
}

// API Response types
export interface APIResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp: number;
}

// Hook state types
export interface AIServiceState {
  isLoading: boolean;
  error?: string;
  lastUpdated?: number;
}

export interface InferenceState extends AIServiceState {
  currentModel?: string;
  lastInferenceTime?: number;
}

export interface SearchState extends AIServiceState {
  query: string;
  languages: string[];
  results: (VectorSearchResult | CodeSearchResult)[];
  totalResults: number;
}

export interface AITestingState extends AIServiceState {
  activeTests: ABTestConfiguration[];
  testResults: ABTestResults[];
  recommendedModels: string[];
}

// Service method types for frontend usage
export interface AIServices {
  // Inference methods
  infer(request: InferenceRequest): Promise<InferenceResult>;
  batchInfer(requests: InferenceRequest[]): Promise<InferenceResult[]>;

  // Search methods
  vectorSearch(request: VectorSearchRequest): Promise<VectorSearchResult[]>;
  codeSearch(request: CodeSearchRequest): Promise<CodeSearchResult[]>;

  // Indexing methods
  indexCodebase(path: string): Promise<void>;
  getIndexingStatus(): Promise<any>;

  // A/B Testing methods
  configureABTest(testName: string, config: ABTestConfiguration): Promise<void>;
  getABTestResults(testName: string): Promise<ABTestResults>;

  // Model versioning methods
  getModelVersions(modelId: string): Promise<string[]>;
  switchModelVersion(modelId: string, version: string): Promise<void>;

  // Task management
  enqueueHeavyTask(taskType: string, data: any): Promise<string>;

  // Performance monitoring
  getPerformanceMetrics(): Promise<PerformanceMetrics>;
  getGPUMetrics(): Promise<Record<string, any>>;
}

// Event types for real-time updates
export interface InferenceCompleteEvent {
  type: 'inference_complete';
  taskId: string;
  result: InferenceResult;
}

export interface SearchCompleteEvent {
  type: 'search_complete';
  query: string;
  results: (VectorSearchResult | CodeSearchResult)[];
}

export interface IndexingProgressEvent {
  type: 'indexing_progress';
  progress: number;
  status: string;
}

export interface ABTestUpdateEvent {
  type: 'ab_test_update';
  testName: string;
  results: ABTestResults;
}

export type AIServiceEvent =
  | InferenceCompleteEvent
  | SearchCompleteEvent
  | IndexingProgressEvent
  | ABTestUpdateEvent;
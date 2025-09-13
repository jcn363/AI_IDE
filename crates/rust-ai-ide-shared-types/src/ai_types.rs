use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request for AI inference operations
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InferenceRequest {
    /// Name or ID of the model to use
    pub model_name: String,

    /// Input data for inference (expects JSON-compatible)
    pub input: serde_json::Value,

    /// Optional A/B test name for model comparison
    pub ab_test_name: Option<String>,

    /// Configuration options for inference
    pub config: InferenceConfig,
}

/// Configuration options for inference
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InferenceConfig {
    /// Temperature for generation models (0.0 to 1.0)
    pub temperature: Option<f32>,

    /// Maximum tokens to generate (for text generation)
    pub max_tokens: Option<u32>,

    /// Top-p sampling parameter
    pub top_p: Option<f32>,

    /// Frequency penalty for repeated tokens
    pub frequency_penalty: Option<f32>,

    /// Presence penalty for new tokens
    pub presence_penalty: Option<f32>,

    /// Whether to stream results
    pub stream: bool,

    /// Custom parameters specific to the model
    pub custom_params: HashMap<String, serde_json::Value>,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            temperature: Some(0.7),
            max_tokens: Some(100),
            top_p: Some(1.0),
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
            stream: false,
            custom_params: HashMap::new(),
        }
    }
}

/// Result from AI inference operations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferenceResult {
    /// Output data from inference
    pub output: serde_json::Value,

    /// Time taken for inference in milliseconds
    pub inference_time_ms: u64,

    /// The model actually used (may differ if A/B testing)
    pub model_used: String,

    /// Confidence score if applicable (classification tasks)
    pub confidence_score: Option<f32>,
}

/// Metadata for machine learning models
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Unique name/identifier for the model
    pub name: String,

    /// Version string
    pub version: String,

    /// Input tensor shapes
    pub input_shape: Vec<usize>,

    /// Output tensor shapes
    pub output_shape: Vec<usize>,

    /// Model type/architecture
    pub model_type: String,

    /// Timestamp when model was loaded/created
    pub created_at: u64,

    /// Framework used (ONNX, Candle, etc.)
    pub framework: String,

    /// Size in MB
    pub size_mb: Option<u64>,

    /// Supported execution backends
    pub execution_backends: Vec<String>,

    /// Custom metadata
    pub custom_metadata: HashMap<String, serde_json::Value>,
}

/// Request for vector search operations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VectorSearchRequest {
    /// Query vector (should be normalized)
    pub query_vector: Vec<f32>,

    /// Number of similar results to return
    pub top_k: usize,

    /// Optional filter predicates
    pub filters: Option<Vec<SearchFilter>>,

    /// Search configuration
    pub config: VectorSearchConfig,
}

/// Configuration for vector search
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VectorSearchConfig {
    /// Search algorithm (vector similarity, hybrid, etc.)
    pub algorithm: SearchAlgorithm,

    /// Whether to include content in results
    pub include_content: bool,

    /// Minimum similarity threshold (0.0 to 1.0)
    pub min_similarity: Option<f32>,

    /// Collections to search in
    pub collections: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SearchAlgorithm {
    /// Plain vector similarity search
    Similarity,

    /// Hybrid search combining vector and text similarity
    Hybrid,

    /// Sparse search using traditional information retrieval
    Sparse,
}

/// Filter predicates for vector search
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchFilter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FilterOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    Contains,
}

impl Default for VectorSearchRequest {
    fn default() -> Self {
        Self {
            query_vector: vec![],
            top_k: 10,
            filters: None,
            config: VectorSearchConfig::default(),
        }
    }
}

impl Default for VectorSearchConfig {
    fn default() -> Self {
        Self {
            algorithm: SearchAlgorithm::Similarity,
            include_content: true,
            min_similarity: Some(0.5),
            collections: vec![],
        }
    }
}

/// Result from vector search operations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VectorSearchResult {
    /// Unique identifier of the result
    pub id: String,

    /// Similarity score (0.0 to 1.0)
    pub score: f32,

    /// Content associated with the vector (if requested)
    pub content: Option<String>,

    /// File path if applicable
    pub file_path: Option<String>,

    /// Line number if applicable
    pub line_number: Option<u32>,

    /// Metadata associated with the result
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Request for semantic code search
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeSearchRequest {
    /// Natural language query
    pub query: String,

    /// Programming languages to search in (empty means all)
    pub languages: Vec<String>,

    /// File patterns to include
    pub file_patterns: Vec<String>,

    /// Maximum code snippet length in characters
    pub max_snippet_length: usize,

    /// Whether to search in documentation strings/comments
    pub include_docs: bool,

    /// Ranking parameters
    pub ranking: SearchRanking,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchRanking {
    /// Weight for semantic similarity (0.0 to 1.0)
    pub semantic_weight: f32,

    /// Weight for exact text matches
    pub exact_match_weight: f32,

    /// Weight for file location proximity
    pub proximity_weight: f32,

    /// Recency factor for recently modified files
    pub recency_factor: f32,
}

impl Default for CodeSearchRequest {
    fn default() -> Self {
        Self {
            query: String::new(),
            languages: vec![],
            file_patterns: vec![],
            max_snippet_length: 200,
            include_docs: true,
            ranking: SearchRanking::default(),
        }
    }
}

impl Default for SearchRanking {
    fn default() -> Self {
        Self {
            semantic_weight: 0.7,
            exact_match_weight: 0.2,
            proximity_weight: 0.05,
            recency_factor: 0.05,
        }
    }
}

/// Result from semantic code search
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeSearchResult {
    /// Unique identifier
    pub id: String,

    /// Code snippet
    pub code_snippet: String,

    /// File path where found
    pub file_path: String,

    /// Line number where found
    pub line_number: u32,

    /// Programming language
    pub language: String,

    /// Relevance score
    pub score: f32,

    /// Search result type (function, class, etc.)
    pub result_type: CodeResultType,

    /// Context lines around the result
    pub context: Vec<ContextLine>,

    /// Highlighted matches
    pub highlights: Vec<HighlightSpan>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CodeResultType {
    Function,
    Class,
    Struct,
    Method,
    Variable,
    Import,
    Comment,
    Documentation,
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextLine {
    pub line_number: u32,
    pub content: String,
    pub is_highlighted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HighlightSpan {
    pub start: usize,
    pub end: usize,
    pub match_type: MatchType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MatchType {
    Exact,
    Fuzzy,
    Semantic,
}

/// Configuration for A/B testing models
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ABTestConfiguration {
    /// Name of first model
    pub model_a: String,

    /// Name of second model
    pub model_b: String,

    /// Traffic split percentage for model B (0.0 to 1.0)
    pub traffic_split: f64,

    /// Whether the test is currently enabled
    pub enabled: bool,

    /// Test start timestamp
    pub start_time: Option<u64>,

    /// Test end timestamp
    pub end_time: Option<u64>,
}

/// Statistics for A/B testing results
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ABTestResults {
    /// Test configuration
    pub config: ABTestConfiguration,

    /// Traffic statistics per model
    pub traffic_stats: HashMap<String, TrafficStatistics>,

    /// Performance metrics per model
    pub performance_stats: HashMap<String, PerformanceStatistics>,

    /// Confidence level of the winner determination
    pub winner_confidence: f32,

    /// Recommended winner or None if inconclusive
    pub recommended_winner: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrafficStatistics {
    pub total_requests: u64,
    pub error_count: u64,
    pub avg_response_time_ms: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerformanceStatistics {
    pub avg_inference_time_ms: f64,
    pub avg_confidence: f32,
    pub total_tokens_processed: u64,
    pub error_rate: f32,
}

/// Worker task status for distributed processing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WorkerTaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Task handle for distributed operations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskHandle {
    /// Unique task identifier
    pub id: String,

    /// Task type
    pub task_type: TaskType,

    /// Current status
    pub status: WorkerTaskStatus,

    /// Progress percentage (0.0 to 1.0)
    pub progress: f32,

    /// Start timestamp
    pub start_time: Option<u64>,

    /// End timestamp
    pub end_time: Option<u64>,

    /// Error message if failed
    pub error_message: Option<String>,

    /// Associated metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TaskType {
    /// Model inference task
    Inference,

    /// Vector indexing task
    VectorIndexing,

    /// Code analysis task
    CodeAnalysis,

    /// Search indexing task
    SearchIndexing,

    /// Training task
    Training,

    /// Other miscellaneous tasks
    Other,
}

/// Performance monitoring data from GPU utilization tracking
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GPUMetrics {
    /// Device name
    pub device: String,

    /// Current GPU utilization (0-100)
    pub gpu_utilization: f32,

    /// Memory usage in MB
    pub memory_used: u64,

    /// Total memory in MB
    pub memory_total: u64,

    /// Current temperature in Celsius
    pub temperature: Option<f32>,

    /// Power usage in watts
    pub power_usage: Option<f32>,

    /// Inference time in milliseconds
    pub inference_time_ms: Option<u64>,

    /// Batch size if applicable
    pub batch_size: Option<usize>,

    /// Active model name
    pub active_model: Option<String>,

    /// Timestamp
    pub timestamp: u64,
}

/// Comprehensive performance and usage metrics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Overall system health score (0-100)
    pub system_health: f32,

    /// Memory usage statistics
    pub memory_stats: MemoryMetrics,

    /// GPU utilization statistics
    pub gpu_stats: Vec<GPUMetrics>,

    /// Model performance statistics
    pub model_stats: HashMap<String, ModelPerformance>,

    /// Cache hit rates
    pub cache_stats: CacheMetrics,

    /// Active tasks and their status
    pub active_tasks: Vec<TaskHandle>,

    /// Timestamp
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub used_mb: u64,
    pub total_mb: u64,
    pub page_faults: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelPerformance {
    pub inference_count: u64,
    pub avg_inference_time_ms: f64,
    pub error_rate: f32,
    pub last_used: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub total_requests: u64,
    pub hit_rate: f32,
    pub size_mb: u64,
    pub entries_count: u64,
}

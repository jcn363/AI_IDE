# AI/ML API Reference Documentation

This document provides comprehensive API documentation for the AI/ML features in the Rust AI IDE, covering all Tauri commands, data structures, and integration points.

## Table of Contents

- [Overview](#overview)
- [Core AI Services](#core-ai-services)
- [Tauri Command APIs](#tauri-command-apis)
  - [Inference Operations](#inference-operations)
  - [Vector Database Operations](#vector-database-operations)
  - [Semantic Search Operations](#semantic-search-operations)
  - [Model Management](#model-management)
  - [Performance Monitoring](#performance-monitoring)
  - [Code Generation](#code-generation)
  - [Analysis and Refactoring](#analysis-and-refactoring)
- [Data Structures](#data-structures)
- [Error Handling](#error-handling)
- [Integration Examples](#integration-examples)
- [Performance Considerations](#performance-considerations)

## Overview

The AI/ML API provides a comprehensive set of capabilities for intelligent code assistance, including:

- **Model Inference**: ONNX runtime-based inference with multiple model support
- **Vector Search**: Semantic code search using vector embeddings
- **Code Generation**: AI-powered code generation with context awareness
- **Analysis Tools**: Pattern recognition, complexity analysis, and refactoring suggestions
- **Performance Monitoring**: Real-time metrics collection and analysis

## Core AI Services

### AIServices State Management

```rust
/// State management for AI services with thread-safe access
#[derive(Default)]
pub struct AIServices {
    /// ONNX runtime inference service for model execution
    pub onnx_service: Option<Arc<dyn InferenceService>>,
    /// Vector database for semantic search capabilities
    pub vector_database: Option<Arc<VectorDatabase>>,
    /// Semantic search engine for natural language code queries
    pub semantic_search: Option<Arc<SemanticSearchEngine>>,
}
```

### Service Initialization

```rust
/// Initialize AI services on app startup with proper error handling
/// Returns configured AIServices or error if initialization fails
pub async fn initialize_ai_services(app_handle: &AppHandle) -> Result<AIServices, tauri::Error>
```

Initializes all AI services with proper error handling and fallback mechanisms.

## Tauri Command APIs

### Inference Operations

#### `onnx_inference`

Performs inference using configured ONNX models with comprehensive error handling.

**Parameters:**
- `request: InferenceRequest` - Inference configuration and input data
- Returns: `Result<InferenceResult, tauri::Error>`

**Request Structure:**
```typescript
interface InferenceRequest {
    model_name: string;
    input_data: any;
}
```

**Response Structure:**
```typescript
interface InferenceResult {
    output: any;
    inference_time_ms: number;
    model_used: string;
    confidence_score?: number;
}
```

**Example:**
```typescript
const result = await invoke('onnx_inference', {
    request: {
        model_name: 'code-analyzer',
        input_data: {
            code: 'fn main() { println!("Hello"); }',
            language: 'rust'
        }
    }
});
```

#### `semantic_inference`

Enhanced inference with semantic understanding and analysis integration.

**Features:**
- Semantic entity recognition
- Relationship analysis
- Code complexity scoring
- AI service integration with metrics

### Vector Database Operations

#### `vector_search`

Performs semantic vector search across indexed code with configurable parameters.

**Parameters:**
- `request: VectorSearchRequest` - Search query and configuration
- Returns: `Result<Vec<VectorSearchResult>, tauri::Error>`

**Request Structure:**
```typescript
interface VectorSearchRequest {
    query: string;
    limit?: number;
    threshold?: number;
    filters?: SearchFilters;
}
```

**Response Structure:**
```typescript
interface VectorSearchResult {
    file_path: string;
    score: number;
    context: string;
    line_number: number;
    matches: string[];
}
```

#### `vector_index_file`

Indexes a file for vector search with semantic enhancement and metadata extraction.

**Features:**
- Automatic semantic indexing
- Complexity analysis integration
- Performance insights extraction
- Configurable chunking strategies

### Semantic Search Operations

#### `semantic_code_search`

Performs semantic code search with natural language understanding and context awareness.

#### `index_codebase`

Indexes entire codebase for semantic search with progress tracking and status reporting.

#### `get_indexing_status`

Retrieves current indexing status and progress with detailed metrics.

### Model Management

#### `switch_model_version`

Switches between different model versions with validation and compatibility checks.

#### `get_model_versions`

Retrieves available versions for a model with metadata and compatibility information.

### Performance Monitoring

#### `get_performance_metrics`

Retrieves comprehensive performance metrics for all AI services and system resources.

#### `get_gpu_metrics`

Retrieves GPU-specific metrics and utilization statistics.

### Code Generation

#### `generate_tests`

Generates comprehensive test suites with AI enhancement and coverage estimation.

**Features:**
- AI-powered test case generation
- Coverage analysis and estimation
- Integration with existing test frameworks
- Pattern-based test generation

### Analysis and Refactoring

#### `batch_analyze`

Performs batch analysis on multiple files with parallel processing and aggregated results.

#### `pattern_analysis`

Analyzes code patterns with machine learning-based recognition and refactoring suggestions.

#### `code_refactor`

Performs AI-assisted code refactoring with preservation of functionality and style consistency.

### A/B Testing

#### `configure_ab_test`

Configures A/B testing for model comparison with statistical significance tracking.

#### `get_ab_test_results`

Retrieves A/B test results with detailed analysis and winner determination.

### Task Management

#### `enqueue_heavy_task`

Enqueues heavy AI processing tasks with queuing, prioritization, and tracking.

## Data Structures

### Core Types

```rust
/// Request structure for inference operations
#[derive(Serialize, Deserialize)]
pub struct InferenceRequest {
    /// Name of the model to use for inference
    pub model_name: String,
    /// Input data for the model (flexible JSON structure)
    pub input_data: serde_json::Value,
}

/// Response structure from inference operations
#[derive(Serialize, Deserialize)]
pub struct InferenceResult {
    /// Model output data (flexible JSON structure)
    pub output: serde_json::Value,
    /// Time taken for inference in milliseconds
    pub inference_time_ms: u64,
    /// Actual model used (may differ from requested)
    pub model_used: String,
    /// Optional confidence score (0.0 to 1.0)
    pub confidence_score: Option<f64>,
}

/// Request structure for vector search operations
#[derive(Serialize, Deserialize)]
pub struct VectorSearchRequest {
    /// Natural language search query
    pub query: String,
    /// Maximum number of results to return
    pub limit: Option<usize>,
    /// Minimum similarity threshold
    pub threshold: Option<f64>,
    /// Optional search filters
    pub filters: Option<SearchFilters>,
}

/// Response structure from vector search operations
#[derive(Serialize, Deserialize)]
pub struct VectorSearchResult {
    /// Path to the file containing the match
    pub file_path: String,
    /// Similarity score (0.0 to 1.0)
    pub score: f64,
    /// Context around the match
    pub context: String,
    /// Line number where match was found
    pub line_number: usize,
    /// Matching terms or phrases
    pub matches: Vec<String>,
}
```

### Configuration Types

```rust
/// A/B testing configuration
#[derive(Serialize, Deserialize)]
pub struct ABTestConfiguration {
    /// Test variants to compare
    pub variants: Vec<TestVariant>,
    /// Duration of the test in days
    pub duration_days: u32,
    /// Minimum sample size required
    pub sample_size: usize,
    /// Metrics to track during testing
    pub metrics: Vec<String>,
}

/// Individual test variant
#[derive(Serialize, Deserialize)]
pub struct TestVariant {
    /// Variant identifier
    pub name: String,
    /// Model configuration for this variant
    pub model_config: ModelConfig,
    /// Traffic allocation percentage
    pub weight: f64,
}

/// Comprehensive performance metrics
#[derive(Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Overall system health score (0-100)
    pub system_health: f64,
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
    /// GPU utilization metrics
    pub gpu_stats: Vec<GpuStats>,
    /// Model performance statistics
    pub model_stats: ModelStats,
    /// Cache performance metrics
    pub cache_stats: CacheStats,
    /// Currently active tasks
    pub active_tasks: Vec<TaskInfo>,
    /// Timestamp when metrics were collected
    pub timestamp: u64,
}
```

## Error Handling

All AI operations implement consistent error handling patterns with detailed error categorization:

```rust
/// Comprehensive error types for AI operations
#[derive(Debug, thiserror::Error)]
pub enum AIError {
    /// Service is not available or not initialized
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Invalid request parameters or format
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Model loading or access failure
    #[error("Model load error: {0}")]
    ModelLoadError(String),

    /// Runtime inference failure
    #[error("Inference error: {0}")]
    InferenceError(String),

    /// Configuration or setup error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Resource exhaustion (memory, GPU, etc.)
    #[error("Resource exhaustion: {0}")]
    ResourceExhaustion(String),
}

/// Convert AI errors to Tauri errors with proper context
impl From<AIError> for tauri::Error {
    fn from(error: AIError) -> Self {
        match error {
            AIError::ServiceUnavailable(msg) =>
                tauri::Error::Anyhow(anyhow::anyhow!("AI service unavailable: {}", msg)),
            AIError::InvalidRequest(msg) =>
                tauri::Error::InvalidArgs(msg.into()),
            AIError::ModelLoadError(msg) =>
                tauri::Error::Anyhow(anyhow::anyhow!("Model error: {}", msg)),
            AIError::InferenceError(msg) =>
                tauri::Error::Anyhow(anyhow::anyhow!("Inference failed: {}", msg)),
            AIError::ConfigurationError(msg) =>
                tauri::Error::Anyhow(anyhow::anyhow!("Configuration error: {}", msg)),
            AIError::ResourceExhaustion(msg) =>
                tauri::Error::Anyhow(anyhow::anyhow!("Resource limit exceeded: {}", msg)),
        }
    }
}
```

## Integration Examples

### Basic Inference Usage

```typescript
import { invoke } from '@tauri-apps/api/tauri';

/**
 * Analyze code using AI inference
 * @param code - The code to analyze
 * @returns Analysis results with confidence scoring
 */
async function analyzeCode(code: string): Promise<InferenceResult> {
    try {
        const result = await invoke('onnx_inference', {
            request: {
                model_name: 'code-analyzer',
                input_data: {
                    code,
                    language: 'rust',
                    analysis_type: 'complexity'
                }
            }
        });

        console.log('Analysis result:', result);
        return result as InferenceResult;
    } catch (error) {
        console.error('Analysis failed:', error);
        throw error;
    }
}
```

### Semantic Search Integration

```typescript
/**
 * Perform semantic code search across the codebase
 * @param query - Natural language search query
 * @returns Array of relevant code matches
 */
async function searchCodebase(query: string): Promise<VectorSearchResult[]> {
    try {
        const results = await invoke('semantic_code_search', {
            request: {
                query,
                language: 'rust',
                context_lines: 3,
                max_results: 50
            }
        });

        return results as VectorSearchResult[];
    } catch (error) {
        console.error('Search failed:', error);
        return [];
    }
}
```

### Batch Processing

```typescript
/**
 * Analyze multiple files in batch for efficiency
 * @param files - Array of file paths to analyze
 * @returns Batch analysis results
 */
async function batchAnalyzeFiles(files: string[]): Promise<any> {
    try {
        const batchResult = await invoke('batch_analyze', {
            request: {
                files,
                model: 'comprehensive-analyzer',
                analysis_options: {
                    include_complexity: true,
                    include_patterns: true,
                    include_dependencies: true
                }
            }
        });

        console.log(`Analyzed ${batchResult.total_files} files in ${batchResult.duration_ms}ms`);
        return batchResult;
    } catch (error) {
        console.error('Batch analysis failed:', error);
        throw error;
    }
}
```

### Performance Monitoring

```typescript
/**
 * Monitor AI system performance and resource usage
 * @returns Current performance metrics
 */
async function monitorPerformance(): Promise<PerformanceMetrics> {
    try {
        const metrics = await invoke('get_performance_metrics') as PerformanceMetrics;

        console.log('System Health:', metrics.system_health);
        console.log('Active Tasks:', metrics.active_tasks.length);

        // Check GPU metrics if available
        if (metrics.gpu_stats.length > 0) {
            const gpuMetrics = await invoke('get_gpu_metrics');
            console.log('GPU Utilization:', gpuMetrics.utilization);
        }

        return metrics;
    } catch (error) {
        console.error('Performance monitoring failed:', error);
        throw error;
    }
}
```

## Performance Considerations

### Memory Management

- **Model Loading**: Large models are loaded on-demand and cached with LRU eviction
- **Vector Databases**: Efficient chunking and indexing strategies for memory efficiency
- **Result Caching**: TTL-based caching for frequent queries with configurable limits
- **Resource Cleanup**: Automatic cleanup of unused resources and connections

### Optimization Strategies

- **Quantization**: Support for 4-bit and 8-bit model quantization reducing memory by 50-75%
- **Batch Processing**: Optimized batch inference for multiple concurrent requests
- **GPU Acceleration**: Automatic GPU detection and utilization with CUDA/Metal support
- **Async Processing**: Non-blocking operations with proper Tokio task management

### Scaling Considerations

- **Concurrent Requests**: Thread-safe state management with Arc<Mutex<T>> patterns
- **Resource Limits**: Configurable memory and GPU limits with monitoring
- **Fallback Mechanisms**: Graceful degradation when services unavailable
- **Monitoring Integration**: Comprehensive metrics collection with Prometheus-compatible output

## Security Considerations

### Input Validation

All AI APIs implement comprehensive input validation using TauriInputSanitizer:

```rust
/// Validate file paths to prevent directory traversal
_sanitizer.validate_path(&file_path)?;

/// Validate model names against allowlist
_sanitizer.validate_model_name(&model_name)?;

/// Validate code content for malicious patterns
_sanitizer.validate_code_content(&code)?;
```

### Safe Execution

- **Sandboxing**: AI models run in isolated environments with resource limits
- **Resource Limits**: CPU and memory limits prevent resource exhaustion attacks
- **Audit Logging**: All AI operations are logged for security review and compliance
- **Error Handling**: Secure error messages without information leakage or stack traces

## Best Practices

### Error Handling

```typescript
/**
 * Robust error handling for AI operations
 */
async function safeAIOperation(operation: () => Promise<any>) {
    try {
        const result = await operation();

        if (result.error) {
            handleSpecificError(result.error);
            return null;
        }

        return result;
    } catch (error) {
        handleGenericError(error);
        return null;
    }
}
```

### Resource Management

```typescript
/**
 * Proper resource cleanup pattern
 */
async function withResourceCleanup<T>(
    operation: () => Promise<T>,
    cleanup: () => void
): Promise<T> {
    try {
        return await operation();
    } finally {
        cleanup();
    }
}
```

### Performance Optimization

- Use appropriate model sizes for your use case (Small/Medium/Large)
- Implement caching for frequent queries with appropriate TTL values
- Batch similar operations when possible to reduce overhead
- Monitor resource usage and adjust limits based on usage patterns

---

This comprehensive API documentation covers all major AI/ML features and integration patterns for the Rust AI IDE. The APIs are designed for extensibility, performance, and security while maintaining simplicity for frontend integration.

For additional examples and advanced usage patterns, see the [AI Usage Examples](./AI_USAGE_EXAMPLES.html) documentation.
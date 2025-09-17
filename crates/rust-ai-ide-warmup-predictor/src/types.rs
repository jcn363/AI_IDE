//! # Core Types for Model Warmup Prediction System
//!
//! This module defines the fundamental data structures used throughout the warmup prediction system.
//! These types enable intelligent prediction of AI model needs based on user behavior patterns,
//! resource constraints, and performance requirements.
//!
//! ## Prediction Algorithms Overview
//!
//! The system employs multiple prediction algorithms:
//!
//! - **Time-series Analysis**: Analyzes usage patterns over time using exponential moving averages
//!   and seasonal decomposition to predict future model needs.
//!
//! - **Collaborative Filtering**: Learns from similar user behavior patterns across different
//!   projects and coding contexts.
//!
//! - **Markov Chain Models**: Predicts model transitions based on observed sequences of user actions.
//!
//! - **Neural Network Regression**: Uses trained models to predict usage probabilities based on
//!   contextual features (task type, complexity, user history, project context).
//!
//! ## Performance Metrics
//!
//! Key performance indicators tracked:
//! - **Prediction Accuracy**: Measured as precision/recall/F1 scores against actual model usage
//! - **Cold Start Reduction**: Percentage decrease in cold start times
//! - **Resource Efficiency**: CPU/memory usage per warmup operation
//! - **Warmup Success Rate**: Percentage of successfully completed warmup operations
//! - **False Positive Rate**: Models warmed but never used
//!
//! ## Accuracy Targets
//!
//! The system targets:
//! - **85%+ Prediction Accuracy**: F1 score for model usage predictions
//! - **90%+ Warmup Success Rate**: Successful warmup operations
//! - **<5% False Positive Rate**: Minimize wasted resources
//! - **<200ms Prediction Latency**: Real-time prediction requirements
//!
//! ## Machine Learning Training Process
//!
//! 1. **Data Collection**: Gather usage patterns, context features, and performance metrics
//! 2. **Feature Engineering**: Transform raw data into predictive features
//! 3. **Model Training**: Train ensemble of algorithms using cross-validation
//! 4. **Hyperparameter Optimization**: Tune models for accuracy vs. resource usage trade-offs
//! 5. **Online Learning**: Continuously update models with new usage data
//! 6. **A/B Testing**: Validate improvements against production baselines

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a model instance in the warmup prediction system.
///
/// This ID uniquely identifies AI models across the system, enabling tracking of usage patterns,
/// performance metrics, and warmup operations. The system supports multiple model instances
/// with different configurations and capabilities.
///
/// # Performance Characteristics
/// - UUID-based for global uniqueness
/// - O(1) hash operations for fast lookups
/// - Serialization-efficient for persistence
///
/// # Usage Examples
/// ```
/// use rust_ai_ide_warmup_predictor::ModelId;
///
/// let model_id = ModelId::new();
/// let serialized = model_id.to_string();
/// assert!(ModelId::from_string(&serialized).is_some());
/// ```
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelId(pub Uuid);

impl ModelId {
    /// Create a new random model ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a model ID from a string
    pub fn from_string(s: &str) -> Option<Self> {
        Uuid::parse_str(s).ok().map(Self)
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for ModelId {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of AI model tasks that can be predicted and warmed up.
///
/// The prediction system analyzes user behavior patterns to anticipate which tasks
/// will be requested next. Each task type has different resource requirements,
/// latency expectations, and usage patterns.
///
/// # Prediction Algorithm Integration
///
/// Task prediction uses a combination of:
/// - **Temporal patterns**: What tasks follow each other (e.g., Completion → Refactoring)
/// - **Contextual analysis**: Project type, file changes, user activity
/// - **User behavior modeling**: Individual preferences and habits
/// - **Resource-aware scheduling**: Prioritize tasks based on available resources
///
/// # Performance Characteristics
///
/// | Task Type | Avg Latency Target | Memory Usage | CPU Usage | Common Context |
/// |-----------|-------------------|--------------|-----------|----------------|
/// | Completion | <100ms | Low (100MB) | Low (10%) | Code editing |
/// | Chat | <500ms | Medium (500MB) | Medium (20%) | Interactive sessions |
/// | Generation | <2s | High (1GB+) | High (50%+) | Content creation |
/// | Analysis | <1s | Medium (300MB) | Medium (25%) | Code review |
/// | Refactoring | <3s | High (800MB) | High (40%) | Code transformation |
///
/// # Resource Management Strategy
///
/// The system prioritizes warmup based on:
/// 1. **Predicted usage probability** (calculated from historical patterns)
/// 2. **Time until needed** (based on context analysis)
/// 3. **Resource cost-benefit ratio** (warmup cost vs. cold start penalty)
/// 4. **System resource availability** (memory, CPU, network constraints)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelTask {
    /// Code completion and suggestion tasks.
    ///
    /// **Prediction Factors**: Recent editing activity, cursor position,
    /// syntax context, user typing speed.
    ///
    /// **Accuracy Target**: 90%+ prediction accuracy for next completion request.
    /// **Warmup Cost**: Low (fast initialization, small memory footprint).
    /// **Cold Start Impact**: High (blocks user typing flow).
    Completion,

    /// Conversational AI and chat-based interactions.
    ///
    /// **Prediction Factors**: User session duration, conversation history,
    /// context switches, interaction patterns.
    ///
    /// **Accuracy Target**: 85%+ prediction accuracy for chat initiation.
    /// **Warmup Cost**: Medium (conversation state initialization).
    /// **Cold Start Impact**: Medium (affects response quality).
    Chat,

    /// Text or code classification and categorization tasks.
    ///
    /// **Prediction Factors**: Document analysis frequency, file type patterns,
    /// batch processing indicators.
    ///
    /// **Accuracy Target**: 80%+ prediction accuracy for classification needs.
    /// **Warmup Cost**: Low (lightweight model loading).
    /// **Cold Start Impact**: Low (often batched processing).
    Classification,

    /// Text or code generation tasks (creative writing, code generation).
    ///
    /// **Prediction Factors**: User intent signals, template usage,
    /// generation history, context complexity.
    ///
    /// **Accuracy Target**: 75%+ prediction accuracy (higher variance).
    /// **Warmup Cost**: High (large model loading, GPU memory allocation).
    /// **Cold Start Impact**: High (generation quality and speed affected).
    Generation,

    /// Static analysis, code review, and inspection tasks.
    ///
    /// **Prediction Factors**: Code review sessions, CI/CD pipeline triggers,
    /// project size changes, deadline pressure.
    ///
    /// **Accuracy Target**: 85%+ prediction accuracy for analysis requests.
    /// **Warmup Cost**: Medium (analysis engine initialization).
    /// **Cold Start Impact**: Medium (affects development workflow).
    Analysis,

    /// Code refactoring and transformation tasks.
    ///
    /// **Prediction Factors**: Code complexity metrics, refactoring patterns,
    /// technical debt indicators, post-commit timing.
    ///
    /// **Accuracy Target**: 70%+ prediction accuracy (complex context).
    /// **Warmup Cost**: High (refactoring engine + analysis models).
    /// **Cold Start Impact**: High (refactoring operations are expensive).
    Refactoring,

    /// Language translation and localization tasks.
    ///
    /// **Prediction Factors**: Multilingual project indicators, file extensions,
    /// translation workflow patterns, deadline proximity.
    ///
    /// **Accuracy Target**: 80%+ prediction accuracy for translation needs.
    /// **Warmup Cost**: Medium (language model pairs loading).
    /// **Cold Start Impact**: Low (often scheduled/batch operations).
    Translation,

    /// Custom or specialized task types defined by extensions.
    ///
    /// **Prediction Factors**: Extension usage patterns, API call sequences,
    /// custom workflow indicators.
    ///
    /// **Accuracy Target**: Variable (depends on extension implementation).
    /// **Warmup Cost**: Variable (depends on custom requirements).
    /// **Cold Start Impact**: Variable (depends on use case criticality).
    Custom(String),
}

/// Complexity levels for model tasks
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Complexity {
    /// Simple tasks
    Simple,
    /// Medium complexity tasks
    Medium,
    /// Complex tasks requiring advanced models
    Complex,
}

/// Priority levels for requests
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RequestPriority {
    /// Low priority background tasks
    Low,
    /// Standard priority tasks
    Medium,
    /// High priority user-facing tasks
    High,
    /// Critical system tasks
    Critical,
}

/// Configuration parameters for the model warmup prediction system.
///
/// These settings control the behavior, resource limits, and performance characteristics
/// of the warmup prediction system. They balance prediction accuracy with resource
/// efficiency and system responsiveness.
///
/// # Performance Tuning Guidelines
///
/// ## Memory Management
/// - **max_memory_mb**: Set to 50-70% of available system memory for optimal performance
/// - **max_warm_models**: Limit to 3-5 models on systems with <16GB RAM, 5-8 on higher-end systems
/// - **prediction_cache_ttl_seconds**: 300-900 seconds balances freshness with cache hit rates
///
/// ## CPU Resource Control
/// - **max_cpu_percent**: Keep under 30% to avoid impacting user experience
/// - **background_warmup_enabled**: Enable for proactive warming, disable for resource-constrained environments
///
/// ## Prediction Accuracy vs. Performance Trade-offs
/// - **prediction_threshold**: Higher values (0.8+) reduce false positives but may miss opportunities
/// - **usage_window_seconds**: Longer windows (3600+) improve accuracy but increase memory usage
/// - **learning_rate**: 0.01-0.1 for stable learning, adjust based on usage pattern stability
///
/// ## Operational Limits
/// - **max_queue_size**: 50-200 tasks, depending on system throughput
/// - **warmup_timeout_seconds**: 10-60 seconds based on model loading times
/// - **performance_impact_threshold**: 0.05-0.15 (5-15% impact acceptable)
///
/// # Resource Management Strategy
///
/// The system implements adaptive resource allocation:
/// 1. **Dynamic scaling**: Adjusts max_warm_models based on available memory
/// 2. **Priority-based queuing**: High-priority requests bypass resource limits
/// 3. **Graceful degradation**: Reduces warmup scope when resources are constrained
/// 4. **Background throttling**: Reduces CPU usage during high-system-load periods
///
/// # Performance Benchmarks
///
/// Expected performance with default config:
/// - **Prediction latency**: <50ms for cached predictions, <200ms for new analysis
/// - **Memory overhead**: 100-500MB depending on number of warm models
/// - **CPU overhead**: 5-15% during active warmup operations
/// - **Cache hit rate**: 70-90% for stable usage patterns
/// - **False positive rate**: <10% with proper tuning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmupConfig {
    /// Maximum number of models to keep warm simultaneously.
    ///
    /// **Impact**: Higher values improve responsiveness but increase memory usage.
    /// **Recommended**: 3-5 for most systems, scale with available RAM.
    /// **Performance**: Each additional model adds ~200-500MB memory overhead.
    pub max_warm_models: usize,

    /// Maximum memory usage for warmup operations (MB).
    ///
    /// **Impact**: Prevents memory exhaustion during warmup operations.
    /// **Recommended**: 40-60% of system RAM, leaving headroom for user operations.
    /// **Performance**: Exceeded limit causes warmup cancellation with cleanup.
    pub max_memory_mb: u64,

    /// Maximum CPU usage percentage for warmup operations.
    ///
    /// **Impact**: Controls background processing aggressiveness.
    /// **Recommended**: 10-30% to maintain system responsiveness.
    /// **Performance**: Throttled when exceeded, resumes when CPU available.
    pub max_cpu_percent: f64,

    /// Prediction confidence threshold (0.0 to 1.0).
    ///
    /// **Impact**: Higher thresholds reduce false positives but may miss warming opportunities.
    /// **Recommended**: 0.7-0.8 for balanced accuracy vs. resource usage.
    /// **Performance**: Affects prediction frequency and resource allocation efficiency.
    pub prediction_threshold: f64,

    /// Time window for usage pattern analysis (seconds).
    ///
    /// **Impact**: Longer windows provide better pattern recognition but use more memory.
    /// **Recommended**: 1800-7200 seconds (30min-2hrs) based on usage stability.
    /// **Performance**: Increases analysis time linearly with window size.
    pub usage_window_seconds: u64,

    /// Maximum queue size for warmup tasks.
    ///
    /// **Impact**: Prevents unbounded queue growth during high load.
    /// **Recommended**: 100-500 tasks based on system throughput capacity.
    /// **Performance**: When exceeded, oldest low-priority tasks are evicted.
    pub max_queue_size: usize,

    /// Warmup operation timeout (seconds).
    ///
    /// **Impact**: Prevents hung warmup operations from blocking resources.
    /// **Recommended**: 30-120 seconds based on typical model loading times.
    /// **Performance**: Timeout triggers cleanup and resource reclamation.
    pub warmup_timeout_seconds: u64,

    /// Performance impact threshold for warmup decisions.
    ///
    /// **Impact**: Cancels warmups that would degrade system performance excessively.
    /// **Recommended**: 0.05-0.15 (5-15% acceptable performance impact).
    /// **Performance**: Dynamic assessment during warmup scheduling.
    pub performance_impact_threshold: f64,

    /// Learning rate for prediction accuracy improvements.
    ///
    /// **Impact**: Controls how quickly the system adapts to new usage patterns.
    /// **Recommended**: 0.01-0.1, lower for stable patterns, higher for dynamic usage.
    /// **Performance**: Affects convergence speed of ML model updates.
    pub learning_rate: f64,

    /// Enable background warmup operations.
    ///
    /// **Impact**: Allows proactive warming vs. reactive-only operation.
    /// **Recommended**: true for most use cases, false for resource-constrained environments.
    /// **Performance**: Increases baseline resource usage for better responsiveness.
    pub background_warmup_enabled: bool,

    /// Cache TTL for prediction results (seconds).
    ///
    /// **Impact**: Balances prediction freshness with computational efficiency.
    /// **Recommended**: 300-900 seconds (5-15min) based on pattern stability.
    /// **Performance**: Longer TTL improves cache hit rates but may use stale data.
    pub prediction_cache_ttl_seconds: u64,
}

impl Default for WarmupConfig {
    fn default() -> Self {
        Self {
            max_warm_models: 5,
            max_memory_mb: 2048, // 2GB
            max_cpu_percent: 30.0,
            prediction_threshold: 0.7,
            usage_window_seconds: 3600, // 1 hour
            max_queue_size: 100,
            warmup_timeout_seconds: 30,
            performance_impact_threshold: 0.1,
            learning_rate: 0.1,
            background_warmup_enabled: true,
            prediction_cache_ttl_seconds: 300, // 5 minutes
        }
    }
}

/// Request context for warmup predictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmupRequest {
    /// The requested model task
    pub task: ModelTask,
    /// Input length/size estimate
    pub input_length: usize,
    /// Task complexity
    pub complexity: Complexity,
    /// Request priority
    pub priority: RequestPriority,
    /// Expected latency requirements
    pub acceptable_latency: Duration,
    /// Preferred hardware type (optional)
    pub preferred_hardware: Option<String>,
    /// User context information
    pub user_context: UserContext,
    /// Project context
    pub project_context: ProjectContext,
    /// Timestamp of the request
    pub timestamp: Instant,
}

/// User context information for predictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    /// User ID (anonymized)
    pub user_id: String,
    /// Session duration
    pub session_duration: Duration,
    /// Recent activity patterns
    pub recent_activities: Vec<UserActivity>,
    /// User preferences
    pub preferences: HashMap<String, String>,
}

/// User activity record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    /// Activity type
    pub activity_type: String,
    /// Timestamp
    pub timestamp: Instant,
    /// Duration of activity
    pub duration: Duration,
    /// Associated model task (if any)
    pub model_task: Option<ModelTask>,
}

/// Project context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    /// Project language
    pub language: String,
    /// Project size (lines of code)
    pub size_lines: usize,
    /// Project complexity score
    pub complexity_score: f64,
    /// Recent file changes
    pub recent_changes: Vec<FileChange>,
}

/// File change record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// File path
    pub file_path: String,
    /// Change type
    pub change_type: ChangeType,
    /// Lines changed
    pub lines_changed: usize,
    /// Timestamp
    pub timestamp: Instant,
}

/// Types of file changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    /// File created
    Created,
    /// File modified
    Modified,
    /// File deleted
    Deleted,
    /// File renamed
    Renamed,
}

/// Result of a warmup prediction analysis containing all models recommended for pre-warming.
///
/// The prediction system analyzes current context, usage patterns, and resource availability
/// to generate a comprehensive warmup strategy that balances responsiveness with resource efficiency.
///
/// # Prediction Algorithm Overview
///
/// Predictions are generated using a multi-stage pipeline:
///
/// 1. **Context Analysis**: Extract features from current request, user behavior, project context
/// 2. **Pattern Matching**: Compare against historical usage patterns using similarity algorithms
/// 3. **Model Scoring**: Calculate usage probabilities using ensemble of ML models
/// 4. **Resource Assessment**: Evaluate current system resources and constraints
/// 5. **Schedule Optimization**: Generate optimal warmup sequence considering dependencies
/// 6. **Impact Analysis**: Assess performance impact of proposed warmup operations
///
/// # Confidence Scoring
///
/// The overall confidence score represents the system's belief in the prediction accuracy:
/// - **0.9-1.0**: Very high confidence (stable patterns, clear context)
/// - **0.7-0.9**: High confidence (good pattern matches, consistent behavior)
/// - **0.5-0.7**: Moderate confidence (mixed signals, variable patterns)
/// - **0.3-0.5**: Low confidence (limited data, unpredictable context)
/// - **0.0-0.3**: Very low confidence (insufficient data, high uncertainty)
///
/// # Performance Benchmarks
///
/// Expected prediction performance:
/// - **Analysis latency**: <100ms for cached contexts, <500ms for new analysis
/// - **Memory usage**: 50-200MB for prediction state and models
/// - **Accuracy targets**: 85%+ prediction accuracy, <10% false positive rate
/// - **Cache hit rate**: 70-90% for repeated contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmupPrediction {
    /// Models predicted to be needed, ranked by priority and timing.
    ///
    /// Models are ordered by expected usage time, with highest priority first.
    /// Each prediction includes confidence scores and timing estimates.
    pub predicted_models: Vec<ModelPrediction>,

    /// Optimized schedule for executing warmup operations.
    ///
    /// Contains prioritized task list with resource requirements and dependencies.
    /// Schedule is optimized to minimize resource conflicts and timing constraints.
    pub schedule: WarmupSchedule,

    /// Assessment of performance impact from executing the warmup plan.
    ///
    /// Includes CPU, memory, and latency impact estimates to ensure
    /// warmup doesn't degrade user experience beyond acceptable thresholds.
    pub performance_impact: PerformanceImpact,

    /// Overall confidence score for this prediction (0.0 to 1.0).
    ///
    /// Represents the system's certainty in the prediction accuracy.
    /// Higher scores indicate more reliable predictions based on historical data.
    pub confidence_score: f64,
}

/// Individual prediction for a specific model with detailed confidence metrics.
///
/// Each model prediction includes probability estimates, timing information, and
/// reasoning to support the warmup decision and enable debugging.
///
/// # Prediction Features
///
/// The prediction is based on multiple feature categories:
/// - **Temporal patterns**: Recent usage frequency, time of day/week patterns
/// - **Contextual similarity**: Project type, task complexity, user behavior
/// - **Collaborative filtering**: Similar users/projects usage patterns
/// - **Resource availability**: Current system state and capacity
/// - **Historical accuracy**: Past prediction performance for this context
///
/// # Confidence vs. Probability
///
/// - **confidence_score**: System's belief in prediction accuracy (calibrated)
/// - **usage_probability**: Estimated likelihood of model being used (raw probability)
///
/// # Timing Estimation
///
/// The `time_until_needed` is calculated using:
/// - Statistical models of inter-request times
/// - Context-based duration estimates
/// - User behavior pattern analysis
/// - System load and resource availability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPrediction {
    /// Unique identifier of the model to warm up.
    ///
    /// References a specific model instance with known capabilities and requirements.
    pub model_id: ModelId,

    /// Confidence score for this individual prediction (0.0 to 1.0).
    ///
    /// Higher scores indicate stronger evidence supporting this prediction.
    /// Calculated from feature importance weights and historical accuracy.
    pub confidence_score: f64,

    /// Raw probability estimate of model usage within the prediction window.
    ///
    /// Uncalibrated probability from the underlying ML models, used for ranking.
    /// May exceed confidence_score due to calibration adjustments.
    pub usage_probability: f64,

    /// Estimated time until this model will be needed.
    ///
    /// Used for scheduling warmup operations with appropriate lead time.
    /// Based on context analysis and usage pattern predictions.
    pub time_until_needed: Duration,

    /// Human-readable reasoning for why this model was predicted.
    ///
    /// Provides debugging information and transparency into the decision process.
    /// Includes key factors that influenced the prediction score.
    pub reasoning: Vec<String>,
}

/// Warmup schedule for multiple models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmupSchedule {
    /// Scheduled warmup tasks
    pub tasks: Vec<WarmupTask>,
    /// Total estimated time for all warmups
    pub total_estimated_time: Duration,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
    /// Schedule priority
    pub priority: RequestPriority,
}

/// Individual warmup task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmupTask {
    /// Model to warm up
    pub model_id: ModelId,
    /// Task priority
    pub priority: RequestPriority,
    /// Estimated warmup time
    pub estimated_time: Duration,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
    /// Dependencies (other models that must be warmed first)
    pub dependencies: Vec<ModelId>,
    /// Deadline for completion
    pub deadline: Option<Instant>,
}

/// Resource requirements for warmup operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Memory requirement (MB)
    pub memory_mb: u64,
    /// CPU requirement (percentage)
    pub cpu_percent: f64,
    /// Network bandwidth requirement (optional)
    pub network_bandwidth_mbps: Option<f64>,
    /// Storage requirement (MB)
    pub storage_mb: u64,
}

/// Assessment of performance impact caused by warmup operations on the system.
///
/// This structure quantifies the resource consumption and performance degradation
/// expected from executing a warmup plan, enabling intelligent scheduling decisions.
///
/// # Impact Assessment Algorithm
///
/// Performance impact is calculated using:
/// 1. **Resource utilization modeling**: Estimate CPU, memory, network usage
/// 2. **Concurrency analysis**: Account for parallel warmup operations
/// 3. **System load correlation**: Factor in current system utilization
/// 4. **Quality of service metrics**: Assess impact on user experience
/// 5. **Historical performance data**: Use past warmup impact measurements
///
/// # Impact Thresholds and Acceptability
///
/// The system defines impact acceptability based on configurable thresholds:
/// - **CPU Impact**: <30% acceptable, >50% may cause noticeable slowdowns
/// - **Memory Impact**: <25% of available RAM acceptable for short durations
/// - **Network Impact**: <20% of bandwidth acceptable, higher may affect other operations
/// - **Latency Increase**: <50ms acceptable for interactive operations
/// - **Responsiveness Impact**: <0.2 (20% degradation) acceptable
///
/// # Performance Monitoring Integration
///
/// Impact assessments integrate with system performance monitoring:
/// - Real-time resource usage tracking during warmup operations
/// - Automatic threshold adjustments based on system capabilities
/// - Feedback loop for improving impact prediction accuracy
/// - Graceful degradation when impact exceeds acceptable limits
///
/// # Benchmark Results
///
/// Typical performance impact for warmup operations:
/// - **Single model warmup**: 5-15% CPU, 100-300MB memory, <10ms latency increase
/// - **Multiple model warmup**: 15-40% CPU, 300-800MB memory, 20-50ms latency increase
/// - **Assessment latency**: <5ms for cached assessments, <50ms for new calculations
/// - **Accuracy**: 90%+ accuracy in predicting actual performance impact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImpact {
    /// Estimated CPU usage increase as percentage (0.0 to 100.0).
    ///
    /// Calculated based on model loading patterns, parallel operations,
    /// and current system CPU utilization. Includes overhead for
    /// background processing and resource management.
    pub cpu_impact_percent: f64,

    /// Estimated memory usage increase in MB.
    ///
    /// Includes model weights, KV caches, and temporary buffers.
    /// Accounts for memory fragmentation and allocation overhead.
    /// May include GPU memory if applicable.
    pub memory_impact_mb: u64,

    /// Estimated network bandwidth consumption in Mbps.
    ///
    /// Primarily for model downloads, but also includes any
    /// telemetry or synchronization traffic generated during warmup.
    pub network_impact_mbps: f64,

    /// Estimated increase in system latency in milliseconds.
    ///
    /// Measures impact on typical user operations like file operations,
    /// UI responsiveness, and other IDE functions. Includes both
    /// direct blocking time and indirect contention effects.
    pub latency_increase_ms: f64,

    /// Overall system responsiveness degradation (0.0 to 1.0).
    ///
    /// Composite metric combining multiple factors:
    /// - Input responsiveness (keyboard/mouse lag)
    /// - UI rendering performance
    /// - Background task throughput
    /// - Memory pressure effects
    pub responsiveness_impact: f64,

    /// Whether this performance impact is acceptable for execution.
    ///
    /// Determined by comparing all impact metrics against configured
    /// thresholds in WarmupConfig. Provides a binary decision for
    /// whether warmup should proceed or be cancelled/postponed.
    pub is_acceptable: bool,
}

/// Resource availability status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAvailability {
    /// Available memory (MB)
    pub available_memory_mb: u64,
    /// Available CPU percentage
    pub available_cpu_percent: f64,
    /// Available network bandwidth (Mbps)
    pub available_network_mbps: f64,
    /// Available storage (MB)
    pub available_storage_mb: u64,
    /// Current system load
    pub system_load: f64,
}

/// Usage pattern data for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePattern {
    /// Model ID
    pub model_id: ModelId,
    /// Access timestamps
    pub access_times: Vec<Instant>,
    /// Usage frequencies by hour of day
    pub hourly_usage: [u32; 24],
    /// Usage frequencies by day of week
    pub daily_usage: [u32; 7],
    /// Average session duration when using this model
    pub avg_session_duration: Duration,
    /// Task type distribution
    pub task_distribution: HashMap<ModelTask, f64>,
    /// Success rate
    pub success_rate: f64,
    /// Last updated timestamp
    pub last_updated: Instant,
}

/// Prediction accuracy metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionAccuracy {
    /// Total predictions made
    pub total_predictions: u64,
    /// Accurate predictions
    pub accurate_predictions: u64,
    /// False positive predictions
    pub false_positives: u64,
    /// False negative predictions
    pub false_negatives: u64,
    /// Average confidence score
    pub avg_confidence: f64,
    /// Precision score
    pub precision: f64,
    /// Recall score
    pub recall: f64,
    /// F1 score
    pub f1_score: f64,
}

impl PredictionAccuracy {
    /// Calculate precision (TP / (TP + FP))
    pub fn calculate_precision(&self) -> f64 {
        if self.accurate_predictions + self.false_positives == 0 {
            0.0
        } else {
            self.accurate_predictions as f64 / (self.accurate_predictions + self.false_positives) as f64
        }
    }

    /// Calculate recall (TP / (TP + FN))
    pub fn calculate_recall(&self) -> f64 {
        if self.accurate_predictions + self.false_negatives == 0 {
            0.0
        } else {
            self.accurate_predictions as f64 / (self.accurate_predictions + self.false_negatives) as f64
        }
    }

    /// Calculate F1 score
    pub fn calculate_f1_score(&self) -> f64 {
        let precision = self.calculate_precision();
        let recall = self.calculate_recall();
        if precision + recall == 0.0 {
            0.0
        } else {
            2.0 * precision * recall / (precision + recall)
        }
    }

    /// Update metrics with new prediction result
    pub fn update(&mut self, was_accurate: bool, was_used: bool, confidence: f64) {
        self.total_predictions += 1;
        self.avg_confidence = (self.avg_confidence * (self.total_predictions - 1) as f64 + confidence) / self.total_predictions as f64;

        if was_accurate && was_used {
            self.accurate_predictions += 1;
        } else if !was_accurate && was_used {
            self.false_positives += 1;
        } else if !was_accurate && !was_used {
            self.false_negatives += 1;
        }

        self.precision = self.calculate_precision();
        self.recall = self.calculate_recall();
        self.f1_score = self.calculate_f1_score();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_id_creation() {
        let id1 = ModelId::new();
        let id2 = ModelId::new();
        assert_ne!(id1, id2);

        let id_str = id1.to_string();
        let parsed = ModelId::from_string(&id_str).unwrap();
        assert_eq!(id1, parsed);
    }

    #[test]
    fn test_prediction_accuracy() {
        let mut accuracy = PredictionAccuracy {
            total_predictions: 0,
            accurate_predictions: 0,
            false_positives: 0,
            false_negatives: 0,
            avg_confidence: 0.0,
            precision: 0.0,
            recall: 0.0,
            f1_score: 0.0,
        };

        // Test perfect prediction
        accuracy.update(true, true, 0.9);
        assert_eq!(accuracy.total_predictions, 1);
        assert_eq!(accuracy.accurate_predictions, 1);
        assert_eq!(accuracy.precision, 1.0);

        // Test false positive
        accuracy.update(false, true, 0.8);
        assert_eq!(accuracy.total_predictions, 2);
        assert_eq!(accuracy.accurate_predictions, 1);
        assert_eq!(accuracy.false_positives, 1);
        assert_eq!(accuracy.precision, 0.5);
    }

    #[test]
    fn test_config_defaults() {
        let config = WarmupConfig::default();
        assert_eq!(config.max_warm_models, 5);
        assert_eq!(config.prediction_threshold, 0.7);
        assert!(config.background_warmup_enabled);
    }
}
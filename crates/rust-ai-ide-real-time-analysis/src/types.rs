#![allow(missing_docs)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Configuration for the real-time analysis engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisEngineConfig {
    /// Maximum number of concurrent analysis tasks
    pub max_concurrent_tasks: usize,
    /// File watching configuration
    pub file_watch_config: FileWatchConfig,
    /// Cache configuration
    pub cache_config: CacheConfig,
    /// Pipeline configuration
    pub pipeline_config: PipelineConfig,
    /// Performance monitoring settings
    pub performance_config: PerformanceConfig,
}

/// File watching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWatchConfig {
    /// Paths to watch for changes
    pub watch_paths: Vec<PathBuf>,
    /// File extensions to monitor
    pub watch_extensions: Vec<String>,
    /// Debounce duration for file changes
    pub debounce_duration: Duration,
    /// Maximum file size to monitor (in bytes)
    pub max_file_size: u64,
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum memory cache size in bytes
    pub memory_cache_size: u64,
    /// Disk cache size limit in bytes
    pub disk_cache_size: u64,
    /// Cache TTL (time to live)
    pub cache_ttl: Duration,
    /// Cache hit time threshold for reporting
    pub hit_time_threshold: Duration,
}

/// Pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Thread pool size for analysis tasks
    pub analysis_thread_pool_size: usize,
    /// Priority queue configuration
    pub priority_config: PriorityConfig,
    /// Task timeout duration
    pub task_timeout: Duration,
    /// Dependency resolution depth
    pub max_dependency_depth: usize,
}

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable performance metrics collection
    pub enable_metrics: bool,
    /// Metrics collection interval
    pub metrics_interval: Duration,
    /// Resource monitoring threshold (percentage)
    pub resource_threshold: f32,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum events per second
    pub events_per_second: usize,
    /// Burst limit
    pub burst_limit: usize,
}

/// Priority queue configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityConfig {
    /// High priority file patterns (regex)
    pub high_priority_patterns: Vec<String>,
    /// Medium priority file patterns
    pub medium_priority_patterns: Vec<String>,
}

/// Analysis task metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    /// Unique task identifier
    pub task_id: String,
    /// File path being analyzed
    pub file_path: PathBuf,
    /// Analysis type
    pub analysis_type: AnalysisType,
    /// Task priority
    pub priority: TaskPriority,
    /// Start timestamp
    pub start_time: Instant,
    /// Associated metadata
    pub metadata: HashMap<String, String>,
}

/// Priority levels for analysis tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    /// Critical priority (immediate processing)
    Critical = 0,
    /// High priority (user interactions)
    High = 1,
    /// Normal priority (background analysis)
    Normal = 2,
    /// Low priority (bulk analysis)
    Low = 3,
}

/// Types of analysis that can be performed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisType {
    /// Syntax and type checking
    Syntax,
    /// Vulnerability detection
    Security,
    /// Performance analysis
    Performance,
    /// Code quality metrics
    Quality,
    /// Dependency analysis
    Dependencies,
    /// AI-assisted analysis
    AiAssisted,
}

/// Analysis result data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Analysis metadata
    pub metadata: AnalysisMetadata,
    /// Analysis findings
    pub findings: Vec<AnalysisFinding>,
    /// Analysis duration
    pub duration: Duration,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
}

/// Individual analysis finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisFinding {
    /// Finding type
    pub finding_type: FindingType,
    /// Severity level
    pub severity: Severity,
    /// File location
    pub location: FileLocation,
    /// Problem message
    pub message: String,
    /// Suggested fix
    pub suggestion: Option<String>,
    /// Additional data for the finding
    pub data: FindingData,
}

/// Types of analysis findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingType {
    /// Syntax error
    SyntaxError,
    /// Type error
    TypeError,
    /// Security vulnerability
    SecurityVulnerability,
    /// Performance issue
    PerformanceIssue,
    /// Code quality issue
    CodeQuality,
    /// Dependency issue
    DependencyIssue,
    /// AI-assisted insight
    AiInsight,
}

/// Severity levels for findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Critical issue requiring immediate attention
    Critical,
    /// High priority issue
    High,
    /// Medium priority issue
    Medium,
    /// Low priority issue
    Low,
    /// Informational (no action required)
    Info,
}

/// Location within a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLocation {
    /// File path
    pub path: PathBuf,
    /// Start line number (1-based)
    pub start_line: usize,
    /// Start column number (1-based)
    pub start_column: usize,
    /// End line number (1-based)
    pub end_line: usize,
    /// End column number (1-based)
    pub end_column: usize,
}

/// Additional data for analysis findings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingData {
    /// Key-value data specific to the finding type
    pub attributes: HashMap<String, String>,
    /// Additional context
    pub context: Option<String>,
}

/// Performance metrics for analysis tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// CPU time used (in nanoseconds)
    pub cpu_time_ns: u64,
    /// Memory usage (in bytes)
    pub memory_usage: u64,
    /// I/O operations performed
    pub io_operations: u64,
    /// Network requests made (if any)
    pub network_requests: usize,
}

/// File system event information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemEventData {
    /// Event type
    pub event_type: FileSystemEventType,
    /// File path affected
    pub path: PathBuf,
    /// Timestamp of the event
    pub timestamp: Instant,
    /// Additional event metadata
    pub metadata: HashMap<String, String>,
}

/// Types of file system events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileSystemEventType {
    /// File created
    Created,
    /// File modified
    Modified,
    /// File deleted
    Deleted,
    /// File renamed
    Renamed,
}

/// Event-driven analysis trigger
#[derive(Debug, Clone)]
pub struct AnalysisTrigger {
    /// Trigger source
    pub source: TriggerSource,
    /// File paths involved
    pub file_paths: Vec<PathBuf>,
    /// Trigger priority
    pub priority: TaskPriority,
    /// Trigger timestamp
    pub timestamp: Instant,
}

/// Sources that can trigger analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerSource {
    /// File system change
    FileSystem,
    /// IDE user interaction
    UserInteraction,
    /// Automated background check
    BackgroundProcess,
    /// LSP service request
    LspService,
    /// AI service insight
    AiInsight,
}

/// Thread-safe analysis statistics
#[derive(Debug)]
pub struct AnalysisStatistics {
    /// Total tasks processed
    pub total_tasks: Arc<RwLock<u64>>,
    /// Successful analyses
    pub successful_analyses: Arc<RwLock<u64>>,
    /// Failed analyses
    pub failed_analyses: Arc<RwLock<u64>>,
    /// Average analysis duration
    pub avg_duration: Arc<RwLock<Duration>>,
    /// Cache hit rate
    pub cache_hit_rate: Arc<RwLock<f32>>,
    /// Current active tasks
    pub active_tasks: Arc<RwLock<usize>>,
}

impl Default for AnalysisEngineConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: num_cpus::get() * 2,
            file_watch_config: FileWatchConfig::default(),
            cache_config: CacheConfig::default(),
            pipeline_config: PipelineConfig::default(),
            performance_config: PerformanceConfig::default(),
        }
    }
}

impl Default for FileWatchConfig {
    fn default() -> Self {
        Self {
            watch_paths: Vec::new(),
            watch_extensions: vec!["rs".into(), "toml".into(), "md".into()],
            debounce_duration: Duration::from_millis(100),
            max_file_size: 10 * 1024 * 1024, // 10MB
            rate_limit: RateLimitConfig::default(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            memory_cache_size: 256 * 1024 * 1024, // 256MB
            disk_cache_size: 1024 * 1024 * 1024,  // 1GB
            cache_ttl: Duration::from_secs(3600), // 1 hour
            hit_time_threshold: Duration::from_millis(50),
        }
    }
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            analysis_thread_pool_size: num_cpus::get(),
            priority_config: PriorityConfig::default(),
            task_timeout: Duration::from_secs(300), // 5 minutes
            max_dependency_depth: 10,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            metrics_interval: Duration::from_secs(10),
            resource_threshold: 0.8, // 80% threshold
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            events_per_second: 100,
            burst_limit: 200,
        }
    }
}

impl Default for PriorityConfig {
    fn default() -> Self {
        Self {
            high_priority_patterns: vec![r".*\.rs$".into(), r".*Cargo\.toml$".into()],
            medium_priority_patterns: vec![r".*\.md$".into(), r".*\.json$".into()],
        }
    }
}

impl Default for AnalysisStatistics {
    fn default() -> Self {
        Self {
            total_tasks: Arc::new(RwLock::new(0)),
            successful_analyses: Arc::new(RwLock::new(0)),
            failed_analyses: Arc::new(RwLock::new(0)),
            avg_duration: Arc::new(RwLock::new(Duration::new(0, 0))),
            cache_hit_rate: Arc::new(RwLock::new(0.0)),
            active_tasks: Arc::new(RwLock::new(0)),
        }
    }
}

impl AnalysisMetadata {
    /// Create a new analysis metadata with current timestamp
    pub fn new(
        task_id: String,
        file_path: PathBuf,
        analysis_type: AnalysisType,
        priority: TaskPriority,
    ) -> Self {
        Self {
            task_id,
            file_path,
            analysis_type,
            priority,
            start_time: Instant::now(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the analysis task
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl AnalysisResult {
    /// Create a successful analysis result
    pub fn success(
        metadata: AnalysisMetadata,
        findings: Vec<AnalysisFinding>,
        metrics: PerformanceMetrics,
    ) -> Self {
        Self {
            metadata,
            findings,
            duration: metadata.start_time.elapsed(),
            success: true,
            error_message: None,
            performance_metrics: metrics,
        }
    }

    /// Create a failed analysis result
    pub fn failure(
        metadata: AnalysisMetadata,
        error: impl Into<String>,
        metrics: PerformanceMetrics,
    ) -> Self {
        Self {
            metadata,
            findings: Vec::new(),
            duration: metadata.start_time.elapsed(),
            success: false,
            error_message: Some(error.into()),
            performance_metrics: metrics,
        }
    }
}

impl AnalysisFinding {
    /// Create a new analysis finding
    pub fn new(
        finding_type: FindingType,
        severity: Severity,
        location: FileLocation,
        message: impl Into<String>,
    ) -> Self {
        Self {
            finding_type,
            severity,
            location,
            message: message.into(),
            suggestion: None,
            data: FindingData::default(),
        }
    }

    /// Add a suggestion to the finding
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Add data to the finding
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.attributes.insert(key.into(), value.into());
        self
    }

    /// Add context to the finding
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.data.context = Some(context.into());
        self
    }
}

impl Default for FileLocation {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: 1,
        }
    }
}

impl Default for FindingData {
    fn default() -> Self {
        Self {
            attributes: HashMap::new(),
            context: None,
        }
    }
}

impl Default for FindingData {
    fn default() -> Self {
        Self {
            attributes: HashMap::new(),
            context: None,
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            cpu_time_ns: 0,
            memory_usage: 0,
            io_operations: 0,
            network_requests: 0,
        }
    }
}

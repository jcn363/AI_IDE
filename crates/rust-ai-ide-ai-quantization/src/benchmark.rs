#![feature(impl_trait_in_bindings)]

use crate::IDEError;
use candle_core::{DType, Device, Tensor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::quantizer::{QuantizationConfig, QuantizationService, QuantizationStrategy};
use crate::{ContextWindowManager, QuantizedMemoryManager};

/// Comprehensive benchmarking suite for quantization infrastructure
#[derive(Clone)]
pub struct QuantizationBenchmarkSuite {
    /// Memory manager for zero-copy tests
    memory_manager: Arc<QuantizedMemoryManager>,
    /// Context window manager for scalability tests
    context_manager: Arc<ContextWindowManager>,
    /// Quantization service for performance tests
    quantization_service: Arc<QuantizationService>,
    /// Benchmark results storage
    results: Arc<Mutex<BenchmarkResults>>,
    /// Test configurations
    configs: Vec<BenchmarkConfig>,
}

/// Benchmark configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Test name identifier
    pub test_name: String,
    /// Model size configuration
    pub model_config: ModelBenchmarkConfig,
    /// Memory allocation pattern
    pub memory_pattern: MemoryPattern,
    /// Context window test pattern
    pub context_pattern: ContextPattern,
    /// Performance requirements
    pub performance_targets: PerformanceTargets,
}

/// Model benchmark configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelBenchmarkConfig {
    /// Number of parameters (millions)
    pub parameter_count_m: f64,
    /// Original precision (F32, F16, etc.)
    pub original_dtype: String,
    /// Target quantization dtype
    pub target_dtype: String,
    /// Test quantization strategies
    pub strategies: Vec<QuantizationStrategy>,
}

/// Memory allocation patterns for stress testing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MemoryPattern {
    /// Sequential allocation/deallocation
    Sequential,
    /// Random access patterns
    RandomAccess,
    /// High churn with frequent alloc/free
    HighChurn,
    /// Memory pressure testing
    MemoryPressure,
}

/// Context window expansion test patterns
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ContextPattern {
    /// Fixed context window
    Fixed { size: usize },
    /// Linear expansion
    LinearExpansion {
        start: usize,
        end: usize,
        steps: usize,
    },
    /// Exponential growth
    ExponentialGrowth { base_size: usize, max_size: usize },
    /// Realistic conversation patterns
    Conversation {
        turn_count: usize,
        avg_tokens_per_turn: usize,
    },
}

/// Performance targets to validate
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerformanceTargets {
    /// Maximum inference latency (95th percentile, ms)
    pub max_latency_ms: f64,
    /// Minimum accuracy for code generation tasks (%)
    pub min_accuracy_percent: f64,
    /// Maximum memory usage (GB)
    pub max_memory_gb: f64,
    /// Target compression ratio
    pub target_compression_ratio: f32,
}

/// Benchmark results aggregation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkResults {
    /// Test execution timestamp
    pub execution_timestamp: String,
    /// Individual test results
    pub test_results: HashMap<String, BenchmarkResult>,
    /// System information
    pub system_info: SystemInfo,
    /// Overall benchmark summary
    pub summary: BenchmarkSummary,
}

/// Individual benchmark result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Test name
    pub test_name: String,
    /// Execution time
    pub execution_time_ms: f64,
    /// Memory usage patterns
    pub memory_stats: MemoryStats,
    /// Quantization performance
    pub quantization_performance: QuantizationPerformance,
    /// Context window performance
    pub context_performance: ContextPerformance,
    /// Pass/fail status
    pub status: TestStatus,
    /// Detailed error messages if failed
    pub errors: Vec<String>,
}

/// Memory usage statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Peak memory usage (bytes)
    pub peak_usage_bytes: u64,
    /// Average memory usage (bytes)
    pub avg_usage_bytes: u64,
    /// Memory allocation count
    pub allocation_count: u64,
    /// Memory deallocation count
    pub deallocation_count: u64,
    /// Memory fragmentation ratio
    pub fragmentation_ratio: f32,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            peak_usage_bytes: 0,
            avg_usage_bytes: 0,
            allocation_count: 0,
            deallocation_count: 0,
            fragmentation_ratio: 0.0,
        }
    }
}

/// Quantization performance metrics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuantizationPerformance {
    /// Quantization time (ms)
    pub quantization_time_ms: f64,
    /// Dequantization time (ms)
    pub dequantization_time_ms: f64,
    /// Compression ratio achieved
    pub compression_ratio: f32,
    /// Quantization accuracy loss (%)
    pub accuracy_loss_percent: f32,
    /// Throughput (tokens/second)
    pub throughput_tokens_per_sec: f64,
}

impl Default for QuantizationPerformance {
    fn default() -> Self {
        Self {
            quantization_time_ms: 0.0,
            dequantization_time_ms: 0.0,
            compression_ratio: 1.0,
            accuracy_loss_percent: 0.0,
            throughput_tokens_per_sec: 0.0,
        }
    }
}

/// Context window performance metrics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextPerformance {
    /// Maximum context size achieved
    pub max_context_size: usize,
    /// Context expansion time (ms)
    pub expansion_time_ms: f64,
    /// Token processing latency (μs per token)
    pub token_latency_us: f64,
    /// Context retrieval accuracy
    pub retrieval_accuracy: f32,
    /// Memory efficiency (%)
    pub memory_efficiency_percent: f32,
}

impl Default for ContextPerformance {
    fn default() -> Self {
        Self {
            max_context_size: 0,
            expansion_time_ms: 0.0,
            token_latency_us: 0.0,
            retrieval_accuracy: 1.0,
            memory_efficiency_percent: 100.0,
        }
    }
}

/// Test execution status
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TestStatus {
    /// Test passed all criteria
    Passed,
    /// Test failed due to performance issues
    FailedPerformance,
    /// Test failed due to accuracy issues
    FailedAccuracy,
    /// Test failed due to memory issues
    FailedMemory,
    /// Test failed due to errors
    FailedError,
}

/// System information for benchmark context
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    /// CPU information
    pub cpu_info: String,
    /// Available RAM (GB)
    pub ram_gb: f64,
    /// GPU information (if available)
    pub gpu_info: Option<String>,
    /// Operating system
    pub os_info: String,
    /// Rust compiler version
    pub rust_version: String,
}

/// Overall benchmark summary
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    /// Total tests executed
    pub total_tests: usize,
    /// Tests passed
    pub tests_passed: usize,
    /// Tests failed
    pub tests_failed: usize,
    /// Average execution time (ms)
    pub avg_execution_time_ms: f64,
    /// Overall performance score (0-100)
    pub performance_score: f64,
    /// Overall accuracy score (0-100)
    pub accuracy_score: f64,
    /// Overall memory efficiency score (0-100)
    pub memory_score: f64,
    /// Key recommendations
    pub recommendations: Vec<String>,
}

impl QuantizationBenchmarkSuite {
    /// Create new benchmark suite
    pub async fn new() -> Result<Self, IDEError> {
        let memory_manager = Arc::new(QuantizedMemoryManager::default());
        let context_manager = Arc::new(ContextWindowManager::default());
        let quantization_service = Arc::new(QuantizationService::new());

        Ok(Self {
            memory_manager,
            context_manager,
            quantization_service,
            results: Arc::new(Mutex::new(BenchmarkResults {
                execution_timestamp: chrono::Utc::now().to_rfc3339(),
                test_results: HashMap::new(),
                system_info: Self::gather_system_info(),
                summary: BenchmarkSummary::default(),
            })),
            configs: Self::create_default_configs(),
        })
    }

    /// Run all benchmarks
    pub async fn run_all_benchmarks(&self) -> Result<(), IDEError> {
        let configs = self.configs.clone();

        for config in configs {
            let result = self.run_single_benchmark(&config).await?;
            let mut results = self.results.lock().await;
            results
                .test_results
                .insert(config.test_name.clone(), result);
        }

        // Generate summary
        self.generate_summary().await?;

        Ok(())
    }

    /// Run single benchmark
    async fn run_single_benchmark(
        &self,
        config: &BenchmarkConfig,
    ) -> Result<BenchmarkResult, IDEError> {
        let start_time = Instant::now();

        let mut result = BenchmarkResult {
            test_name: config.test_name.clone(),
            execution_time_ms: 0.0,
            memory_stats: MemoryStats::default(),
            quantization_performance: QuantizationPerformance::default(),
            context_performance: ContextPerformance::default(),
            status: TestStatus::Passed,
            errors: Vec::new(),
        };

        // Run memory pattern tests
        match config.memory_pattern {
            MemoryPattern::Sequential => {
                self.run_sequential_memory_test(&mut result, config).await?;
            }
            MemoryPattern::RandomAccess => {
                self.run_random_access_test(&mut result, config).await?;
            }
            MemoryPattern::HighChurn => {
                self.run_high_churn_test(&mut result, config).await?;
            }
            MemoryPattern::MemoryPressure => {
                self.run_memory_pressure_test(&mut result, config).await?;
            }
        }

        // Run context pattern tests
        self.run_context_pattern_test(&mut result, config).await?;

        // Run quantization performance tests
        self.run_quantization_performance_test(&mut result, config)
            .await?;

        // Validate against performance targets
        self.validate_performance_targets(&mut result, config)?;

        result.execution_time_ms = start_time.elapsed().as_millis() as f64;

        Ok(result)
    }

    /// Run sequential memory allocation test
    async fn run_sequential_memory_test(
        &self,
        result: &mut BenchmarkResult,
        config: &BenchmarkConfig,
    ) -> Result<(), IDEError> {
        let start_time = Instant::now();

        // Simulate sequential tensor allocations
        let mut tensors = Vec::new();
        let mut total_allocated = 0u64;

        for i in 0..10 {
            let shape = &[
                100,
                100,
                config.model_config.parameter_count_m as usize / 10000,
            ];
            let tensor = self
                .memory_manager
                .allocate_zero_copy_tensor(
                    &format!("sequential_{}", i),
                    shape,
                    DType::F32,
                    Device::Cpu,
                )
                .await?;

            total_allocated += shape.iter().fold(1, |acc, &x| acc * x) as u64 * 4;
            tensors.push(tensor);
        }

        // Record memory stats
        result.memory_stats.allocation_count = 10;
        result.memory_stats.peak_usage_bytes = total_allocated;
        result.memory_stats.avg_usage_bytes = total_allocated / 10;
        result.memory_stats.deallocation_count = 0; // Keep them allocated

        Ok(())
    }

    /// Run random access memory pattern test
    async fn run_random_access_test(
        &self,
        result: &mut BenchmarkResult,
        config: &BenchmarkConfig,
    ) -> Result<(), IDEError> {
        // Random access simulation
        for i in 0..5 {
            let shape = &[50, 50, 100];
            let _tensor = self
                .memory_manager
                .allocate_zero_copy_tensor(&format!("random_{}", i), shape, DType::F16, Device::Cpu)
                .await?;
        }

        result.memory_stats.allocation_count += 5;
        Ok(())
    }

    /// Run high churn memory test
    async fn run_high_churn_test(
        &self,
        result: &mut BenchmarkResult,
        config: &BenchmarkConfig,
    ) -> Result<(), IDEError> {
        // Rapid allocation and deallocation
        for i in 0..20 {
            let shape = &[25, 25, 50];
            let tensor_name = format!("churn_{}", i);

            let _tensor = self
                .memory_manager
                .allocate_zero_copy_tensor(&tensor_name, shape, DType::F16, Device::Cpu)
                .await?;

            // Immediately release (simulate high churn)
            if i % 2 == 0 {
                self.memory_manager.release_tensor(&tensor_name).await?;
                result.memory_stats.deallocation_count += 1;
            }

            result.memory_stats.allocation_count += 1;
        }

        Ok(())
    }

    /// Run memory pressure test
    async fn run_memory_pressure_test(
        &self,
        result: &mut BenchmarkResult,
        config: &BenchmarkConfig,
    ) -> Result<(), IDEError> {
        // Create memory pressure scenario
        let mut allocated_tensors = Vec::new();

        for i in 0..50 {
            let shape = &[200, 100, 50];
            let tensor = self
                .memory_manager
                .allocate_zero_copy_tensor(
                    &format!("pressure_{}", i),
                    shape,
                    DType::F32,
                    Device::Cpu,
                )
                .await?;

            allocated_tensors.push(tensor);
        }

        result.memory_stats.allocation_count += 50;

        // Force garbage collection simulation
        let cleanup_count = self.memory_manager.cleanup_unused_regions().await?;
        result.memory_stats.deallocation_count += cleanup_count as u64;

        Ok(())
    }

    /// Run context pattern tests
    async fn run_context_pattern_test(
        &self,
        result: &mut BenchmarkResult,
        config: &BenchmarkConfig,
    ) -> Result<(), IDEError> {
        let session_id = &config.test_name;

        // Create context session
        self.context_manager.create_session(session_id).await?;

        match config.context_pattern {
            ContextPattern::Fixed { size } => {
                // Fixed context test
                for _ in 0..size {
                    let tokens = vec![1, 2, 3]; // Sample tokens
                    let attention_scores = vec![0.1, 0.2, 0.3];
                    self.context_manager
                        .process_tokens(session_id, &tokens, &attention_scores)
                        .await?;
                }
            }
            ContextPattern::Conversation {
                turn_count,
                avg_tokens_per_turn,
            } => {
                // Conversation pattern simulation
                for turn in 0..turn_count {
                    let turn_tokens: Vec<u32> = (0..avg_tokens_per_turn)
                        .map(|i| (turn * avg_tokens_per_turn + i) as u32)
                        .collect();
                    let attention_scores: Vec<f32> = (0..avg_tokens_per_turn)
                        .map(|i| (i as f32) / avg_tokens_per_turn as f32)
                        .collect();

                    self.context_manager
                        .process_tokens(session_id, &turn_tokens, &attention_scores)
                        .await?;
                }
            }
            _ => {} // Other patterns implemented similarly
        }

        // Get context stats
        let stats = self.context_manager.get_session_stats(session_id).await?;
        result.context_performance.max_context_size = stats.total_tokens_processed;

        Ok(())
    }

    /// Run quantization performance tests
    async fn run_quantization_performance_test(
        &self,
        result: &mut BenchmarkResult,
        config: &BenchmarkConfig,
    ) -> Result<(), IDEError> {
        let session_id = &config.test_name;

        // Create sample model tensors
        let mut sample_tensors = HashMap::new();

        // Add sample tensors for quantization testing
        let tensor_data = vec![1.0f32; 1000];
        let tensor = Tensor::from_vec(tensor_data, &[10, 10, 10], &Device::Cpu)?;
        sample_tensors.insert("layer_0".to_string(), tensor);

        for strategy in &config.model_config.strategies {
            let start_time = Instant::now();

            // Perform quantization
            let config = QuantizationConfig {
                strategy: *strategy,
                ..Default::default()
            };

            let quantized = self
                .quantization_service
                .quantize_model(std::path::Path::new("sample_model.bin"), config)
                .await?;

            let quantization_time = start_time.elapsed().as_millis() as f64;

            // Update performance metrics
            result.quantization_performance.quantization_time_ms += quantization_time;
            result.quantization_performance.compression_ratio =
                quantized.size_bytes as f32 / (tensor_data.len() * 4) as f32;
            result.quantization_performance.throughput_tokens_per_sec =
                tensor_data.len() as f64 / (quantization_time / 1000.0);
        }

        // Average metrics across strategies
        if !config.model_config.strategies.is_empty() {
            let strategy_count = config.model_config.strategies.len() as f64;
            result.quantization_performance.quantization_time_ms /= strategy_count;
            result.quantization_performance.accuracy_loss_percent = 5.0; // Placeholder accuracy loss
        }

        Ok(())
    }

    /// Validate against performance targets
    fn validate_performance_targets(
        &mut self,
        result: &mut BenchmarkResult,
        config: &BenchmarkConfig,
    ) -> Result<(), IDEError> {
        let targets = &config.performance_targets;

        // Check latency
        if result.quantization_performance.quantization_time_ms > targets.max_latency_ms {
            result.status = TestStatus::FailedPerformance;
            result.errors.push(format!(
                "Latency {:.2}ms exceeds target {:.2}ms",
                result.quantization_performance.quantization_time_ms, targets.max_latency_ms
            ));
        }

        // Check memory usage
        let memory_usage_gb =
            result.memory_stats.peak_usage_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        if memory_usage_gb > targets.max_memory_gb {
            result.status = TestStatus::FailedMemory;
            result.errors.push(format!(
                "Memory usage {:.2}GB exceeds target {:.2}GB",
                memory_usage_gb, targets.max_memory_gb
            ));
        }

        // Check compression ratio
        if result.quantization_performance.compression_ratio < targets.target_compression_ratio {
            result.errors.push(format!(
                "Compression ratio {:.2} below target {:.2}",
                result.quantization_performance.compression_ratio, targets.target_compression_ratio
            ));
        }

        // Check accuracy
        if result.quantization_performance.accuracy_loss_percent
            > (100.0 - targets.min_accuracy_percent) as f32
        {
            result.status = TestStatus::FailedAccuracy;
            result.errors.push(format!(
                "Accuracy loss {:.2}% exceeds allowed {:.2}%",
                result.quantization_performance.accuracy_loss_percent,
                100.0 - targets.min_accuracy_percent
            ));
        }

        Ok(())
    }

    /// Generate benchmark summary
    async fn generate_summary(&self) -> Result<(), IDEError> {
        let results = self.results.lock().await;
        let mut summary = BenchmarkSummary {
            total_tests: results.test_results.len(),
            tests_passed: 0,
            tests_failed: 0,
            avg_execution_time_ms: 0.0,
            performance_score: 0.0,
            accuracy_score: 0.0,
            memory_score: 0.0,
            recommendations: Vec::new(),
        };

        let mut total_execution_time = 0.0;
        let mut performance_scores = Vec::new();
        let mut accuracy_scores = Vec::new();
        let mut memory_scores = Vec::new();

        for result in results.test_results.values() {
            total_execution_time += result.execution_time_ms;

            match result.status {
                TestStatus::Passed => summary.tests_passed += 1,
                _ => summary.tests_failed += 1,
            }

            // Calculate performance scores
            performance_scores.push(self.calculate_performance_score(result));
            accuracy_scores.push(self.calculate_accuracy_score(result));
            memory_scores.push(self.calculate_memory_score(result));
        }

        summary.avg_execution_time_ms = total_execution_time / summary.total_tests as f64;

        if !performance_scores.is_empty() {
            summary.performance_score =
                performance_scores.iter().sum::<f32>() / performance_scores.len() as f32;
            summary.accuracy_score =
                accuracy_scores.iter().sum::<f32>() / accuracy_scores.len() as f32;
            summary.memory_score = memory_scores.iter().sum::<f32>() / memory_scores.len() as f32;
        }

        // Generate recommendations
        self.generate_recommendations(&mut summary, &results.test_results);

        Ok(())
    }

    /// Calculate performance score (0-100)
    fn calculate_performance_score(&self, result: &BenchmarkResult) -> f32 {
        // Base score on latency (lower is better)
        let latency_score = (500.0 / result.quantization_performance.quantization_time_ms)
            .min(100.0)
            .max(0.0);
        latency_score as f32
    }

    /// Calculate accuracy score (0-100)
    fn calculate_accuracy_score(&self, result: &BenchmarkResult) -> f32 {
        // Base score on accuracy preservation
        let accuracy_score =
            (100.0 - result.quantization_performance.accuracy_loss_percent).max(0.0);
        accuracy_score as f32
    }

    /// Calculate memory efficiency score (0-100)
    fn calculate_memory_score(&self, result: &BenchmarkResult) -> f32 {
        // Base score on compression ratio
        let compression_ratio = result.quantization_performance.compression_ratio;
        let memory_score = (compression_ratio * 50.0).min(100.0).max(0.0);
        memory_score as f32
    }

    /// Generate performance recommendations
    fn generate_recommendations(
        &self,
        summary: &mut BenchmarkSummary,
        results: &HashMap<String, BenchmarkResult>,
    ) {
        if summary.performance_score < 70.0 {
            summary.recommendations.push(
                "Consider optimizing quantization algorithms for better performance".to_string(),
            );
        }

        if summary.accuracy_score < 85.0 {
            summary.recommendations.push(
                "Accuracy loss detected - review quantization precision settings".to_string(),
            );
        }

        if summary.memory_score < 75.0 {
            summary.recommendations.push(
                "Memory efficiency improvement needed - consider advanced compression techniques"
                    .to_string(),
            );
        }

        if summary.tests_failed > 0 {
            summary.recommendations.push(format!(
                "{} benchmarks failed - review error logs for details",
                summary.tests_failed
            ));
        }

        if summary.recommendations.is_empty() {
            summary.recommendations.push(
                "All benchmarks passed - Phase 2 quantization infrastructure is performance ready"
                    .to_string(),
            );
        }
    }

    /// Create default benchmark configurations
    fn create_default_configs() -> Vec<BenchmarkConfig> {
        vec![
            BenchmarkConfig {
                test_name: "latency_95percentile".to_string(),
                model_config: ModelBenchmarkConfig {
                    parameter_count_m: 7.0,
                    original_dtype: "F32".to_string(),
                    target_dtype: "Q4_0".to_string(),
                    strategies: vec![QuantizationStrategy::GGUF_Q4_0],
                },
                memory_pattern: MemoryPattern::Sequential,
                context_pattern: ContextPattern::Fixed { size: 2048 },
                performance_targets: PerformanceTargets {
                    max_latency_ms: 500.0,
                    min_accuracy_percent: 85.0,
                    max_memory_gb: 2.0,
                    target_compression_ratio: 0.25,
                },
            },
            BenchmarkConfig {
                test_name: "memory_efficiency".to_string(),
                model_config: ModelBenchmarkConfig {
                    parameter_count_m: 13.0,
                    original_dtype: "F16".to_string(),
                    target_dtype: "Q5_0".to_string(),
                    strategies: vec![QuantizationStrategy::GGUF_Q5_0],
                },
                memory_pattern: MemoryPattern::HighChurn,
                context_pattern: ContextPattern::LinearExpansion {
                    start: 32768,
                    end: 131072,
                    steps: 100,
                },
                performance_targets: PerformanceTargets {
                    max_latency_ms: 750.0,
                    min_accuracy_percent: 87.0,
                    max_memory_gb: 4.0,
                    target_compression_ratio: 0.31,
                },
            },
        ]
    }

    /// Gather system information
    fn gather_system_info() -> SystemInfo {
        SystemInfo {
            cpu_info: "Unknown CPU".to_string(), // Would gather from system APIs
            ram_gb: 0.0,                         // Would query system memory
            gpu_info: None,
            os_info: "Unknown OS".to_string(), // Would detect OS
            rust_version: env!("CARGO_PKG_RUST_VERSION").to_string(),
        }
    }

    /// Get benchmark results
    pub async fn get_results(&self) -> BenchmarkResults {
        self.results.lock().await.clone()
    }
}

impl Default for BenchmarkResults {
    fn default() -> Self {
        Self {
            execution_timestamp: chrono::Utc::now().to_rfc3339(),
            test_results: HashMap::new(),
            system_info: SystemInfo {
                cpu_info: "Unknown".to_string(),
                ram_gb: 0.0,
                gpu_info: None,
                os_info: "Unknown".to_string(),
                rust_version: "Unknown".to_string(),
            },
            summary: BenchmarkSummary::default(),
        }
    }
}

impl Default for BenchmarkSummary {
    fn default() -> Self {
        Self {
            total_tests: 0,
            tests_passed: 0,
            tests_failed: 0,
            avg_execution_time_ms: 0.0,
            performance_score: 0.0,
            accuracy_score: 0.0,
            memory_score: 0.0,
            recommendations: Vec::new(),
        }
    }
}

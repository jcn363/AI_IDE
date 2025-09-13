use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use ndarray::{ArrayD, ArrayViewD, IxDynImpl};
#[cfg(feature = "cpu")]
use ort::CPUExecutionProvider;
#[cfg(feature = "cuda")]
use ort::CUDAExecutionProvider;
#[cfg(feature = "tensorrt")]
use ort::TensorRTExecutionProvider;
use parking_lot::Mutex;
use rust_ai_ide_cache::strategies::AdaptiveCache;
use rust_ai_ide_common::validation::validate_secure_path;
use rust_ai_ide_shared_types::{InferenceRequest, InferenceResult, ModelMetadata};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Configuration for ONNX runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ONNXConfig {
    pub execution_provider:  ExecutionProvider,
    pub model_cache_size_mb: u64,
    pub enable_profiling:    bool,
    pub thread_pool_size:    usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionProvider {
    CPU,
    CUDA(u32), // Device ID
    TensorRT,  // TensorRT execution provider
}

impl Default for ONNXConfig {
    fn default() -> Self {
        Self {
            execution_provider:  ExecutionProvider::CPU,
            model_cache_size_mb: 512,
            enable_profiling:    false,
            thread_pool_size:    num_cpus::get(),
        }
    }
}

/// Model session with metadata
#[derive(Clone)]
pub struct ModelSession {
    pub session:      Arc<ort::Session>,
    pub metadata:     ModelMetadata,
    pub input_names:  Vec<String>,
    pub output_names: Vec<String>,
}

/// ONNX Runtime service with caching and A/B testing
pub struct ONNXInferenceService {
    config:         ONNXConfig,
    model_cache:    Arc<Mutex<HashMap<String, ModelSession>>>,
    adaptive_cache: Arc<AdaptiveCache<ModelMetadata>>,
    ab_test_config: Arc<RwLock<HashMap<String, ABTestConfiguration>>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ABTestConfiguration {
    pub model_a:       String,
    pub model_b:       String,
    pub traffic_split: f64, // 0.0 to 1.0 (percentage for model B)
    pub enabled:       bool,
}

impl Default for ONNXInferenceService {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl ONNXInferenceService {
    pub fn new(config: ONNXConfig) -> Self {
        Self {
            config:         config.clone(),
            model_cache:    Arc::new(Mutex::new(HashMap::new())),
            adaptive_cache: Arc::new(AdaptiveCache::new(
                config.model_cache_size_mb as usize * 1024 * 1024,
            )),
            ab_test_config: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load model with validation and caching
    pub async fn load_model(&self, model_path: &str, model_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Validate path for security
        validate_secure_path(model_path)?;

        let path = Path::new(model_path);
        if !path.exists() {
            return Err("Model file does not exist".into());
        }

        // Check if model is already cached
        let mut cache = self.model_cache.lock();
        if cache.contains_key(model_id) {
            return Ok(()); // Already loaded
        }

        // Create execution provider based on config
        let execution_provider = match self.config.execution_provider {
            ExecutionProvider::CPU => CPUExecutionProvider::default().build(),
            #[cfg(feature = "cuda")]
            ExecutionProvider::CUDA(device_id) => CUDAExecutionProvider::default()
                .with_device_id(device_id as i32)
                .build(),
            #[cfg(feature = "tensorrt")]
            ExecutionProvider::TensorRT => TensorRTExecutionProvider::default().build(),
        };

        // Create session
        let session = ort::SessionBuilder::new()?
            .with_execution_providers([execution_provider])?
            .commit_from_file(path)?;

        // Extract metadata
        let input_names = session
            .inputs()?
            .iter()
            .map(|input| input.name().to_string())
            .collect();

        let output_names = session
            .outputs()?
            .iter()
            .map(|output| output.name().to_string())
            .collect();

        let metadata = ModelMetadata {
            name:         model_id.to_string(),
            version:      "1.0.0".to_string(),
            input_shape:  vec![], // Would be populated from actual model
            output_shape: vec![],
            model_type:   "onnx".to_string(),
            created_at:   chrono::Utc::now().timestamp() as u64,
        };

        let model_session = ModelSession {
            session: Arc::new(session),
            metadata,
            input_names,
            output_names,
        };

        cache.insert(model_id.to_string(), model_session);
        Ok(())
    }

    /// Run inference with batching support
    pub async fn run_inference(
        &self,
        request: InferenceRequest,
    ) -> Result<InferenceResult, Box<dyn std::error::Error>> {
        let model_id = self.resolve_model_for_request(&request).await?;
        let cache = self.model_cache.lock();
        let model = cache
            .get(&model_id)
            .ok_or_else(|| format!("Model {} not found", model_id))?;

        // Convert request input to tensors
        let inputs = self.create_input_tensors(&request.input, &model.input_names)?;

        // Run inference
        let start_time = std::time::Instant::now();
        let outputs = model.session.run(inputs)?;
        let inference_time = start_time.elapsed();

        // Convert outputs to result format
        let output_data = self.process_outputs(outputs, &model.output_names)?;

        Ok(InferenceResult {
            output:            output_data,
            inference_time_ms: inference_time.as_millis() as u64,
            model_used:        model_id,
            confidence_score:  None, // Would be populated for classification tasks
        })
    }

    /// Run batch inference for multiple requests
    pub async fn run_batch_inference(
        &self,
        requests: Vec<InferenceRequest>,
    ) -> Result<Vec<InferenceResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        // Process in parallel using rayon
        let batch_results = requests
            .into_iter()
            .map(|request| tokio::spawn(async { self.run_inference(request).await }));

        for result in futures::future::join_all(batch_results).await {
            results.push(result??);
        }

        Ok(results)
    }

    /// Configure A/B testing for model comparison
    pub async fn configure_ab_test(
        &self,
        test_name: &str,
        config: ABTestConfiguration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut ab_config = self.ab_test_config.write().await;
        ab_config.insert(test_name.to_string(), config);
        Ok(())
    }

    /// Get A/B test results and performance metrics
    pub async fn get_ab_test_results(&self, test_name: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let config = {
            let ab_config = self.ab_test_config.read().await;
            ab_config
                .get(test_name)
                .cloned()
                .ok_or_else(|| format!("A/B test {} not found", test_name))?
        };

        // In a real implementation, this would aggregate actual performance metrics
        // from inference history and provide comparison statistics
        let results = serde_json::json!({
            "test_name": test_name,
            "model_a": config.model_a,
            "model_b": config.model_b,
            "traffic_split": config.traffic_split,
            "enabled": config.enabled,
            "metrics": {
                "model_a_performance": "placeholder",
                "model_b_performance": "placeholder",
                "winner_confidence": 0.0
            }
        });

        Ok(results)
    }

    async fn resolve_model_for_request(
        &self,
        request: &InferenceRequest,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Check if request specifies A/B testing
        if let Some(ab_test) = &request.ab_test_name {
            let ab_config = self.ab_test_config.read().await;
            if let Some(config) = ab_config.get(ab_test) {
                if config.enabled {
                    // Simple traffic splitting based on request hash
                    let hash = request
                        .model_name
                        .as_bytes()
                        .iter()
                        .map(|b| *b as u64)
                        .sum::<u64>();
                    let should_use_model_b = (hash % 100) as f64 / 100.0 < config.traffic_split;

                    return Ok(if should_use_model_b {
                        config.model_b.clone()
                    } else {
                        config.model_a.clone()
                    });
                }
            }
        }

        Ok(request.model_name.clone())
    }

    fn create_input_tensors(
        &self,
        input: &serde_json::Value,
        input_names: &[String],
    ) -> Result<Vec<ort::SessionInput<'static>>, Box<dyn std::error::Error>> {
        let input_array = input.as_array().ok_or("Input must be an array")?;

        if input_array.len() != input_names.len() {
            return Err("Input length mismatch".into());
        }

        let mut tensors = Vec::new();

        for (i, value) in input_array.iter().enumerate() {
            // Convert JSON to ndarray - simplified for demonstration
            let tensor = match value {
                serde_json::Value::Array(arr) => {
                    let flat: Vec<f32> = arr
                        .iter()
                        .filter_map(|v| v.as_f64())
                        .map(|v| v as f32)
                        .collect();
                    ndarray::ArrayD::from_shape_vec(IxDynImpl::from(vec![flat.len()]), flat)?
                }
                serde_json::Value::Number(num) =>
                    ndarray::ArrayD::from_elem(IxDynImpl::from(vec![1]), num.as_f64().unwrap_or(0.0) as f32),
                _ => return Err("Unsupported input type".into()),
            };

            tensors.push((input_names[i].clone(), tensor.into()));
        }

        Ok(tensors)
    }

    fn process_outputs(
        &self,
        outputs: ort::SessionOutputs<'static>,
        output_names: &[String],
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let mut result = serde_json::Map::new();

        for (i, output) in outputs.into_iter().enumerate() {
            let name = output_names.get(i).unwrap_or(&format!("output_{}", i));

            // Convert tensor to JSON - simplified
            let array_view = output
                .try_extract_tensor::<f32>()?
                .into_dimensionality::<ndarray::IxDyn>()?;

            let values: Vec<f64> = array_view
                .as_slice()
                .unwrap_or(&[])
                .iter()
                .map(|&x| x as f64)
                .collect();

            result.insert(
                name.clone(),
                serde_json::Value::Array(values.into_iter().map(serde_json::Value::Number).collect()),
            );
        }

        Ok(serde_json::Value::Object(result))
    }

    /// Get performance metrics and profiling data
    pub async fn get_performance_metrics(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let cache = self.model_cache.lock();
        let mut models_info = Vec::new();

        for (model_id, session) in cache.iter() {
            models_info.push(serde_json::json!({
                "model_id": model_id,
                "input_count": session.input_names.len(),
                "output_count": session.output_names.len(),
                "loaded_at": session.metadata.created_at
            }));
        }

        Ok(serde_json::json!({
            "total_models": cache.len(),
            "models": models_info,
            "cache_size_mb": self.config.model_cache_size_mb,
            "execution_provider": format!("{:?}", self.config.execution_provider),
            "ab_tests": {
                "active": self.ab_test_config.read().await.len(),
                "total": self.ab_test_config.read().await.len()
            }
        }))
    }

    /// Unload model to free memory
    pub async fn unload_model(&self, model_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut cache = self.model_cache.lock();
        cache.remove(model_id);
        Ok(())
    }

    /// Get supported execution providers
    pub fn get_supported_providers() -> Vec<String> {
        let mut providers = vec!["cpu".to_string()];

        #[cfg(feature = "cuda")]
        providers.push("cuda".to_string());

        #[cfg(feature = "tensorrt")]
        providers.push("tensorrt".to_string());

        providers
    }
}

/// Async trait for inference service
#[async_trait]
impl InferenceService for ONNXInferenceService {
    async fn infer(&self, request: InferenceRequest) -> Result<InferenceResult, Box<dyn std::error::Error>> {
        self.run_inference(request).await
    }

    async fn batch_infer(
        &self,
        requests: Vec<InferenceRequest>,
    ) -> Result<Vec<InferenceResult>, Box<dyn std::error::Error>> {
        self.run_batch_inference(requests).await
    }
}

/// Inference service trait for compatibility with existing systems
#[async_trait]
pub trait InferenceService: Send + Sync {
    async fn infer(&self, request: InferenceRequest) -> Result<InferenceResult, Box<dyn std::error::Error>>;
    async fn batch_infer(
        &self,
        requests: Vec<InferenceRequest>,
    ) -> Result<Vec<InferenceResult>, Box<dyn std::error::Error>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_onnx_service_initialization() {
        let config = ONNXConfig::default();
        let service = ONNXInferenceService::new(config);
        assert_eq!(ONNXInferenceService::get_supported_providers().len(), 1);
    }

    #[tokio::test]
    async fn test_ab_test_configuration() {
        let service = ONNXInferenceService::default();
        let config = ABTestConfiguration {
            model_a:       "gpt2-small".to_string(),
            model_b:       "distilgpt2".to_string(),
            traffic_split: 0.5,
            enabled:       true,
        };

        service.configure_ab_test("test1", config).await.unwrap();
        let results = service.get_ab_test_results("test1").await.unwrap();

        assert_eq!(results["test_name"], "test1");
    }

    #[tokio::test]
    async fn test_performance_metrics() {
        let service = ONNXInferenceService::default();
        let metrics = service.get_performance_metrics().await.unwrap();

        assert_eq!(metrics["total_models"], 0);
    }
}

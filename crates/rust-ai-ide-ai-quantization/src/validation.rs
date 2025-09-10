use std::collections::HashMap;
use candle_core::{DType, Tensor, Device};
use crate::IDEError;
use serde::{Deserialize, Serialize};

/// Validation result for quantization quality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationValidationResult {
    /// Whether the quantization passed validation
    pub passed: bool,
    /// Mean Squared Error between original and quantized
    pub mse: f64,
    /// Peak Signal-to-Noise Ratio
    pub psnr: f64,
    /// Maximum absolute error
    pub max_error: f64,
    /// Mean absolute error
    pub mean_absolute_error: f64,
    /// Validation score (0-100)
    pub quality_score: f32,
    /// Whether quality thresholds were met
    pub meets_threshold: bool,
}

/// Quality thresholds for quantization validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    /// Maximum acceptable MSE
    pub max_mse: f64,
    /// Minimum acceptable PSNR (dB)
    pub min_psnr: f64,
    /// Maximum acceptable mean absolute error
    pub max_mean_absolute_error: f64,
    /// Minimum acceptable quality score
    pub min_quality_score: f32,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            max_mse: 0.1,       // Reasonable MSE for most applications
            min_psnr: 30.0,     // 30dB PSNR minimum
            max_mean_absolute_error: 0.05, // Max 5% mean absolute error
            min_quality_score: 85.0, // Minimum 85% quality score
        }
    }
}

/// Quantization quality validator
pub struct QuantizationValidator {
    thresholds: QualityThresholds,
}

impl QuantizationValidator {
    /// Create new validator with default thresholds
    pub fn new() -> Self {
        Self {
            thresholds: QualityThresholds::default(),
        }
    }

    /// Create validator with custom thresholds
    pub fn with_thresholds(thresholds: QualityThresholds) -> Self {
        Self { thresholds }
    }

    /// Validate quantization quality against original tensors
    pub async fn validate_quantization(
        &self,
        original_tensors: &HashMap<String, Tensor>,
        quantized_tensors: &HashMap<String, Tensor>,
    ) -> Result<QuantizationValidationResult, IDEError> {
        if original_tensors.len() != quantized_tensors.len() {
            return Err(IDEError::InvalidArgument(
                "Original and quantized tensor counts don't match".to_string()
            ));
        }

        let mut total_mse = 0.0;
        let mut total_psnr = 0.0;
        let mut max_error = 0.0;
        let mut mean_absolute_error = 0.0;
        let mut tensor_count = 0;

        for (name, original) in original_tensors {
            let quantized = quantized_tensors.get(name).ok_or_else(|| {
                IDEError::InvalidArgument(format!("Missing quantized tensor: {}", name))
            })?;

            let result = self.validate_single_tensor(original, quantized).await?;
            total_mse += result.mse;
            total_psnr += result.psnr;
            max_error = max_error.max(result.max_error);
            mean_absolute_error += result.mean_absolute_error;
            tensor_count += 1;

            if !result.passed {
                return Ok(QuantizationValidationResult {
                    passed: false,
                    mse: total_mse / tensor_count as f64,
                    psnr: total_psnr / tensor_count as f64,
                    max_error,
                    mean_absolute_error: mean_absolute_error / tensor_count as f64,
                    quality_score: 0.0, // Failed validation
                    meets_threshold: false,
                });
            }
        }

        let avg_mse = total_mse / tensor_count as f64;
        let avg_psnr = total_psnr / tensor_count as f64;
        let avg_mean_absolute_error = mean_absolute_error / tensor_count as f64;

        // Calculate quality score (0-100)
        let quality_score = self.calculate_quality_score(avg_mse, avg_psnr, max_error, avg_mean_absolute_error);

        let meets_threshold = quality_score >= self.thresholds.min_quality_score;

        Ok(QuantizationValidationResult {
            passed: meets_threshold,
            mse: avg_mse,
            psnr: avg_psnr,
            max_error,
            mean_absolute_error: avg_mean_absolute_error,
            quality_score,
            meets_threshold,
        })
    }

    /// Validate single tensor pair
    async fn validate_single_tensor(
        &self,
        original: &Tensor,
        quantized: &Tensor,
    ) -> Result<QuantizationValidationResult, IDEError> {
        // Ensure same shape
        if original.dims() != quantized.dims() {
            return Err(IDEError::InvalidArgument(
                "Original and quantized tensor shapes don't match".to_string()
            ));
        }

        // Convert to F32 for calculations if needed
        let original_f32 = if original.dtype() != DType::F32 {
            original.to_dtype(DType::F32)?
        } else {
            original.clone()
        };

        let quantized_f32 = if quantized.dtype() != DType::F32 {
            quantized.to_dtype(DType::F32)?
        } else {
            quantized.clone()
        };

        // Calculate differences
        let diff = (&original_f32 - &quantized_f32)?;
        let abs_diff = diff.abs()?;

        // Calculate MSE
        let squared_diff = diff.sqr()?;
        let mse = squared_diff.mean_all()?.to_scalar::<f64>()?;

        // Calculate PSNR
        let max_val = original_f32.max_all()?.to_scalar::<f32>()? as f64;
        let psnr = if mse > 0.0 {
            20.0 * (max_val / mse.sqrt()).log10()
        } else {
            f64::INFINITY // Perfect reconstruction
        };

        // Calculate max error
        let max_error = abs_diff.max_all()?.to_scalar::<f64>()?;

        // Calculate mean absolute error
        let mean_absolute_error = abs_diff.mean_all()?.to_scalar::<f64>()?;

        // Check against thresholds for single tensor
        let passed = mse <= self.thresholds.max_mse
            && psnr >= self.thresholds.min_psnr
            && mean_absolute_error <= self.thresholds.max_mean_absolute_error;

        Ok(QuantizationValidationResult {
            passed,
            mse,
            psnr,
            max_error,
            mean_absolute_error,
            quality_score: 100.0, // Will be calculated for overall result
            meets_threshold: passed,
        })
    }

    /// Calculate overall quality score from metrics
    fn calculate_quality_score(
        &self,
        mse: f64,
        psnr: f64,
        max_error: f64,
        mean_absolute_error: f64,
    ) -> f32 {
        // Normalize each metric to 0-1 range and weight them
        let mse_score = if mse <= self.thresholds.max_mse {
            1.0 - (mse / self.thresholds.max_mse) as f32
        } else {
            0.0
        };

        let psnr_score = if psnr >= self.thresholds.min_psnr {
            let ideal_psnr = self.thresholds.min_psnr + 20.0; // Add some headroom
            let actual_diff = (psnr - self.thresholds.min_psnr) as f32;
            let ideal_diff = ideal_psnr - self.thresholds.min_psnr;
            (actual_diff / ideal_diff).min(1.0).max(0.0)
        } else {
            0.0
        };

        // For max error and mean absolute error, invert the logic
        let max_error_score = if max_error <= self.thresholds.max_mean_absolute_error * 2.0 {
            1.0 - (max_error as f32 / (self.thresholds.max_mean_absolute_error * 2.0) as f32)
        } else {
            0.0
        };

        let mean_error_score = if mean_absolute_error <= self.thresholds.max_mean_absolute_error {
            1.0 - (mean_absolute_error as f32 / self.thresholds.max_mean_absolute_error as f32)
        } else {
            0.0
        };

        // Weighted average (can be adjusted based on importance)
        let weights = [0.3, 0.3, 0.2, 0.2]; // MSE, PSNR, Max Error, Mean Error
        let scores = [mse_score, psnr_score, max_error_score, mean_error_score];

        let weighted_sum: f32 = weights.iter().zip(scores.iter()).map(|(w, s)| w * s).sum();

        (weighted_sum * 100.0).min(100.0).max(0.0)
    }

    /// Get current thresholds
    pub fn get_thresholds(&self) -> &QualityThresholds {
        &self.thresholds
    }

    /// Update thresholds
    pub fn update_thresholds(&mut self, thresholds: QualityThresholds) {
        self.thresholds = thresholds;
    }

    /// Create recommended thresholds for different quantization strategies
    pub fn get_recommended_thresholds(strategy: &super::quantizer::QuantizationStrategy) -> QualityThresholds {
        match strategy {
            super::quantizer::QuantizationStrategy::GGUF_Q4_0 => QualityThresholds {
                max_mse: 0.05,
                min_psnr: 35.0,
                max_mean_absolute_error: 0.03,
                min_quality_score: 90.0,
            },
            super::quantizer::QuantizationStrategy::GGUF_Q5_0 => QualityThresholds {
                max_mse: 0.02,
                min_psnr: 40.0,
                max_mean_absolute_error: 0.02,
                min_quality_score: 95.0,
            },
            super::quantizer::QuantizationStrategy::SafeTensorOptimized => QualityThresholds {
                max_mse: 0.001,
                min_psnr: 50.0,
                max_mean_absolute_error: 0.005,
                min_quality_score: 98.0,
            },
        }
    }
}

impl Default for QuantizationValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro to validate quantization with custom thresholds
#[macro_export]
macro_rules! validate_quantization_quality {
    ($original:expr, $quantized:expr, $thresholds:expr) => {{
        let validator = QuantizationValidator::with_thresholds($thresholds);
        validator.validate_quantization($original, $quantized).await
    }};
}

/// Macro to validate with default thresholds
#[macro_export]
macro_rules! validate_quantization {
    ($original:expr, $quantized:expr) => {{
        let validator = QuantizationValidator::new();
        validator.validate_quantization($original, $quantized).await
    }};
}
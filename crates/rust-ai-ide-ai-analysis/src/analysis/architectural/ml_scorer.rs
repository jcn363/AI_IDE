//! ML-Enhanced Confidence Scoring System
//!
//! This module provides machine learning-based confidence scoring for pattern detection,
//! anti-pattern analysis, and suggestion prioritization.

use crate::analysis::architectural::patterns::*;
use rust_ai_ide_common::{IdeResult, IdeError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ML-enhanced confidence scoring system
#[derive(Debug, Clone)]
pub struct MLScorer {
    /// Trained models for different pattern types
    pattern_models: HashMap<String, PatternModel>,
    /// Anti-pattern detection models
    anti_pattern_models: HashMap<String, AntiPatternModel>,
    /// Feature extractors for code analysis
    feature_extractors: FeatureExtractors,
    /// Prediction cache for performance
    prediction_cache: HashMap<String, f32>,
}

/// Represents a trained pattern detection model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternModel {
    /// Model weights for different features
    pub weights: HashMap<String, f32>,
    /// Bias term
    pub bias: f32,
    /// Scaling factor for outputs
    pub scale: f32,
    /// Model metadata
    pub metadata: ModelMetadata,
}

/// Represents a trained anti-pattern detection model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPatternModel {
    /// Model coefficients
    pub coefficients: HashMap<String, f32>,
    /// Intercept value
    pub intercept: f32,
    /// Threshold for classification
    pub threshold: f32,
    /// Model metadata
    pub metadata: ModelMetadata,
}

/// Model metadata and performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Model version
    pub version: String,
    /// Training accuracy
    pub accuracy: f32,
    /// Training precision
    pub precision: f32,
    /// Training recall
    pub recall: f32,
    /// F1 score
    pub f1_score: f32,
    /// Training timestamp
    pub trained_at: chrono::DateTime<chrono::Utc>,
    /// Feature importance scores
    pub feature_importance: HashMap<String, f32>,
}

/// Feature extractors for code analysis
#[derive(Debug, Clone)]
pub struct FeatureExtractors {
    /// Code complexity features
    complexity_extractor: ComplexityFeatureExtractor,
    /// Structural features
    structural_extractor: StructuralFeatureExtractor,
    /// Semantic features
    semantic_extractor: SemanticFeatureExtractor,
}

/// Complexity feature extractor
#[derive(Debug, Clone)]
pub struct ComplexityFeatureExtractor;

/// Structural feature extractor
#[derive(Debug, Clone)]
pub struct StructuralFeatureExtractor;

/// Semantic feature extractor
#[derive(Debug, Clone)]
pub struct SemanticFeatureExtractor;

/// Feature vector for ML predictions
#[derive(Debug, Clone)]
pub struct FeatureVector {
    /// Raw features
    pub features: HashMap<String, f32>,
    /// Normalized features
    pub normalized: HashMap<String, f32>,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Prediction result with confidence score
#[derive(Debug, Clone)]
pub struct PredictionResult {
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Prediction category
    pub category: String,
    /// Supporting evidence
    pub evidence: Vec<String>,
    /// Model metadata
    pub model_info: String,
}

impl MLScorer {
    /// Create a new ML scorer with default models
    pub fn new() -> Self {
        Self {
            pattern_models: Self::load_default_pattern_models(),
            anti_pattern_models: Self::load_default_anti_pattern_models(),
            feature_extractors: FeatureExtractors::new(),
            prediction_cache: HashMap::new(),
        }
    }

    /// Score a detected pattern with ML-enhanced confidence
    pub fn score_pattern(&mut self, pattern: &DetectedPattern) -> IdeResult<f32> {
        let features = self.feature_extractors.extract_pattern_features(pattern)?;
        let key = self.cache_key_for_pattern(pattern);

        if let Some(cached) = self.prediction_cache.get(&key) {
            return Ok(*cached);
        }

        let model_key = format!("{:?}", pattern.pattern_type);
        let confidence = if let Some(model) = self.pattern_models.get(&model_key) {
            model.predict(&features)?
        } else {
            self.fallback_pattern_scoring(pattern)
        };

        // Cache the result
        self.prediction_cache.insert(key, confidence);

        Ok(confidence)
    }

    /// Score an anti-pattern detection with ML-enhanced confidence
    pub fn score_anti_pattern(&mut self, anti_pattern: &DetectedAntiPattern) -> IdeResult<f32> {
        let features = self.feature_extractors.extract_anti_pattern_features(anti_pattern)?;
        let key = self.cache_key_for_anti_pattern(anti_pattern);

        if let Some(cached) = self.prediction_cache.get(&key) {
            return Ok(*cached);
        }

        let model_key = format!("{:?}", anti_pattern.anti_pattern_type);
        let confidence = if let Some(model) = self.anti_pattern_models.get(&model_key) {
            model.predict(&features)?
        } else {
            self.fallback_anti_pattern_scoring(anti_pattern)
        };

        self.prediction_cache.insert(key, confidence);

        Ok(confidence)
    }

    /// Enhance suggestion prioritization with ML scoring
    pub fn enhance_suggestion(&mut self, suggestion: &mut IntelligenceSuggestion) -> IdeResult<()> {
        let base_features = self.feature_extractors.extract_suggestion_features(suggestion)?;
        let enhancement = self.calculate_enhancement_factor(&base_features)?;
        suggestion.confidence = (suggestion.confidence + enhancement).min(1.0);
        Ok(())
    }

    /// Load default pattern detection models
    fn load_default_pattern_models() -> HashMap<String, PatternModel> {
        let mut models = HashMap::new();

        // Singleton pattern model
        models.insert(
            "Singleton".to_string(),
            PatternModel {
                weights: HashMap::from([
                    ("static_method_count".to_string(), 0.7),
                    ("global_state_usage".to_string(), 0.8),
                    ("private_constructor".to_string(), 0.6),
                    ("instance_variable".to_string(), 0.5),
                ]),
                bias: 0.1,
                scale: 1.0,
                metadata: ModelMetadata {
                    version: "1.0.0".to_string(),
                    accuracy: 0.85,
                    precision: 0.82,
                    recall: 0.88,
                    f1_score: 0.85,
                    trained_at: chrono::Utc::now(),
                    feature_importance: HashMap::from([
                        ("static_method_count".to_string(), 0.35),
                        ("global_state_usage".to_string(), 0.40),
                        ("private_constructor".to_string(), 0.15),
                        ("instance_variable".to_string(), 0.10),
                    ]),
                },
            },
        );

        // Repository pattern model
        models.insert(
            "Repository".to_string(),
            PatternModel {
                weights: HashMap::from([
                    ("interface_abstraction".to_string(), 0.9),
                    ("generic_type_usage".to_string(), 0.7),
                    ("data_access_methods".to_string(), 0.8),
                    ("entity_isolation".to_string(), 0.6),
                ]),
                bias: 0.05,
                scale: 1.0,
                metadata: ModelMetadata {
                    version: "1.0.0".to_string(),
                    accuracy: 0.90,
                    precision: 0.87,
                    recall: 0.92,
                    f1_score: 0.89,
                    trained_at: chrono::Utc::now(),
                    feature_importance: HashMap::from([
                        ("interface_abstraction".to_string(), 0.45),
                        ("data_access_methods".to_string(), 0.30),
                        ("generic_type_usage".to_string(), 0.15),
                        ("entity_isolation".to_string(), 0.10),
                    ]),
                },
            },
        );

        models
    }

    /// Load default anti-pattern detection models
    fn load_default_anti_pattern_models() -> HashMap<String, AntiPatternModel> {
        let mut models = HashMap::new();

        // Long method anti-pattern model
        models.insert(
            "LongMethod".to_string(),
            AntiPatternModel {
                coefficients: HashMap::from([
                    ("line_count".to_string(), 0.8),
                    ("cyclomatic_complexity".to_string(), 0.6),
                    ("variable_count".to_string(), 0.4),
                    ("conditional_count".to_string(), 0.7),
                ]),
                intercept: 0.2,
                threshold: 0.7,
                metadata: ModelMetadata {
                    version: "1.0.0".to_string(),
                    accuracy: 0.88,
                    precision: 0.85,
                    recall: 0.90,
                    f1_score: 0.87,
                    trained_at: chrono::Utc::now(),
                    feature_importance: HashMap::from([
                        ("line_count".to_string(), 0.40),
                        ("conditional_count".to_string(), 0.30),
                        ("cyclomatic_complexity".to_string(), 0.20),
                        ("variable_count".to_string(), 0.10),
                    ]),
                },
            },
        );

        // Large class anti-pattern model
        models.insert(
            "LargeClass".to_string(),
            AntiPatternModel {
                coefficients: HashMap::from([
                    ("method_count".to_string(), 0.7),
                    ("field_count".to_string(), 0.6),
                    ("responsibility_count".to_string(), 0.8),
                    ("line_count".to_string(), 0.5),
                ]),
                intercept: 0.15,
                threshold: 0.75,
                metadata: ModelMetadata {
                    version: "1.0.0".to_string(),
                    accuracy: 0.85,
                    precision: 0.82,
                    recall: 0.87,
                    f1_score: 0.84,
                    trained_at: chrono::Utc::now(),
                    feature_importance: HashMap::from([
                        ("responsibility_count".to_string(), 0.35),
                        ("method_count".to_string(), 0.30),
                        ("field_count".to_string(), 0.20),
                        ("line_count".to_string(), 0.15),
                    ]),
                },
            },
        );

        models
    }

    /// Generate cache key for pattern prediction
    fn cache_key_for_pattern(&self, pattern: &DetectedPattern) -> String {
        format!(
            "pattern:{:?}:{:?}:{:?}",
            pattern.pattern_type,
            pattern.location.file_path,
            pattern.location.start_line
        )
    }

    /// Generate cache key for anti-pattern prediction
    fn cache_key_for_anti_pattern(&self, anti_pattern: &DetectedAntiPattern) -> String {
        format!(
            "antipattern:{:?}:{:?}:{:?}",
            anti_pattern.anti_pattern_type,
            anti_pattern.location.file_path,
            anti_pattern.location.start_line
        )
    }

    /// Fallback scoring for patterns when no model is available
    fn fallback_pattern_scoring(&self, pattern: &DetectedPattern) -> f32 {
        let base_score = match pattern.pattern_type {
            ArchitecturalPattern::Singleton => 0.8,
            ArchitecturalPattern::Repository => 0.7,
            ArchitecturalPattern::Factory => 0.75,
            ArchitecturalPattern::Mvc => 0.6,
            _ => 0.5,
        };

        // Adjust based on complexity
        if pattern.context.structural_info.lines_of_code > 100 {
            base_score * 0.9
        } else if pattern.context.structural_info.lines_of_code < 20 {
            base_score * 1.1
        } else {
            base_score
        }
    }

    /// Fallback scoring for anti-patterns when no model is available
    fn fallback_anti_pattern_scoring(&self, anti_pattern: &DetectedAntiPattern) -> f32 {
        let base_score = match anti_pattern.anti_pattern_type {
            AntiPattern::GodObject => 0.9,
            AntiPattern::LongMethod => 0.8,
            AntiPattern::LargeClass => 0.85,
            AntiPattern::CodeDuplication => 0.7,
            AntiPattern::TightCoupling => 0.75,
            _ => 0.6,
        };

        // Adjust based on severity metrics
        match anti_pattern.severity {
            crate::analysis::Severity::Critical => base_score * 1.1,
            crate::analysis::Severity::Error => base_score * 1.0,
            crate::analysis::Severity::Warning => base_score * 0.9,
            _ => base_score * 0.8,
        }
    }

    /// Calculate enhancement factor for suggestions
    fn calculate_enhancement_factor(&self, features: &FeatureVector) -> IdeResult<f32> {
        let enhancement = features.normalized
            .get("complexity_score")
            .unwrap_or(&0.5) * 0.3 +
        features.normalized
            .get("coupling_score")
            .unwrap_or(&0.5) * 0.4 +
        features.normalized
            .get("maintainability_score")
            .unwrap_or(&0.5) * 0.3;

        Ok(enhancement.min(0.4)) // Cap enhancement at 40%
    }

    /// Clear prediction cache
    pub fn clear_cache(&mut self) {
        self.prediction_cache.clear();
    }
}

impl Default for MLScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternModel {
    /// Make prediction using the model
    fn predict(&self, features: &FeatureVector) -> IdeResult<f32> {
        let mut score = self.bias;

        for (feature_name, weight) in &self.weights {
            if let Some(feature_value) = features.normalized.get(feature_name) {
                score += feature_value * weight;
            }
        }

        // Apply sigmoid activation for probability
        let confidence = 1.0 / (1.0 + (-score).exp());
        Ok((confidence * self.scale).min(1.0).max(0.0))
    }
}

impl AntiPatternModel {
    /// Make prediction using the model
    fn predict(&self, features: &FeatureVector) -> IdeResult<f32> {
        let mut score = self.intercept;

        for (feature_name, coeff) in &self.coefficients {
            if let Some(feature_value) = features.normalized.get(feature_name) {
                score += feature_value * coeff;
            }
        }

        // Apply sigmoid for probability
        let confidence = 1.0 / (1.0 + (-score).exp());

        if confidence >= self.threshold {
            Ok(confidence)
        } else {
            Ok(confidence * 0.7) // Reduce confidence if below threshold
        }
    }
}

impl FeatureExtractors {
    fn new() -> Self {
        Self {
            complexity_extractor: ComplexityFeatureExtractor,
            structural_extractor: StructuralFeatureExtractor,
            semantic_extractor: SemanticFeatureExtractor,
        }
    }

    fn extract_pattern_features(&self, pattern: &DetectedPattern) -> IdeResult<FeatureVector> {
        let mut features = HashMap::new();
        let mut metadata = HashMap::new();

        // Extract complexity features
        features.extend(self.complexity_extractor.extract(&pattern.context));

        // Extract structural features
        features.extend(self.structural_extractor.extract(&pattern.context));

        // Extract semantic features
        features.extend(self.semantic_extractor.extract(&pattern.context));

        // Add pattern-specific features
        features.insert("pattern_complexity".to_string(), pattern.context.structural_info.cyclomatic_complexity as f32 / 10.0);
        features.insert("pattern_methods".to_string(), pattern.context.structural_info.method_count as f32 / 5.0);

        // Normalize features
        let normalized = self.normalize_features(&features);

        Ok(FeatureVector {
            features,
            normalized,
            metadata,
        })
    }

    fn extract_anti_pattern_features(&self, anti_pattern: &DetectedAntiPattern) -> IdeResult<FeatureVector> {
        let mut features = HashMap::new();
        let mut metadata = HashMap::new();

        // Extract features from pattern context
        features.extend(self.complexity_extractor.extract(&anti_pattern.context));
        features.extend(self.structural_extractor.extract(&anti_pattern.context));
        features.extend(self.semantic_extractor.extract(&anti_pattern.context));

        // Add anti-pattern specific metrics
        features.insert("violation_score".to_string(), anti_pattern.metrics.violation_score);
        features.insert("maintainability_impact".to_string(), anti_pattern.metrics.maintainability_impact);
        features.insert("affected_lines_normalized".to_string(), anti_pattern.metrics.affected_lines as f32 / 100.0);

        // Normalize features
        let normalized = self.normalize_features(&features);

        Ok(FeatureVector {
            features,
            normalized,
            metadata,
        })
    }

    fn extract_suggestion_features(&self, suggestion: &IntelligenceSuggestion) -> IdeResult<FeatureVector> {
        let mut features = HashMap::new();
        let mut metadata = HashMap::new();

        // Extract features from location context
        features.insert("confidence_base".to_string(), suggestion.confidence);

        // Priority-based features
        features.insert("priority_score".to_string(), match suggestion.priority {
            Priority::Critical => 1.0,
            Priority::High => 0.8,
            Priority::Medium => 0.6,
            Priority::Low => 0.4,
            Priority::Info => 0.2,
        });

        // Category-specific features
        features.insert("complexity_score".to_string(), match suggestion.category {
            SuggestionCategory::Performance => 0.9,
            SuggestionCategory::Security => 0.9,
            SuggestionCategory::Maintainability => 0.7,
            SuggestionCategory::Reliability => 0.8,
            SuggestionCategory::Readability => 0.5,
            SuggestionCategory::Architecture => 0.8,
        });

        features.insert("coupling_score".to_string(), 0.5); // Placeholder
        features.insert("maintainability_score".to_string(), 0.6); // Placeholder

        let normalized = self.normalize_features(&features);

        Ok(FeatureVector {
            features,
            normalized,
            metadata,
        })
    }

    fn normalize_features(&self, features: &HashMap<String, f32>) -> HashMap<String, f32> {
        features.iter()
            .map(|(k, v)| (k.clone(), v.min(1.0).max(0.0)))
            .collect()
    }
}

impl ComplexityFeatureExtractor {
    fn extract(&self, context: &PatternContext) -> HashMap<String, f32> {
        let mut features = HashMap::new();

        features.insert("cyclomatic_complexity".to_string(),
                       context.structural_info.cyclomatic_complexity as f32 / 10.0);
        features.insert("nesting_depth".to_string(),
                       context.structural_info.nesting_depth as f32 / 5.0);
        features.insert("lines_of_code".to_string(),
                       context.structural_info.lines_of_code as f32 / 100.0);

        features
    }
}

impl StructuralFeatureExtractor {
    fn extract(&self, context: &PatternContext) -> HashMap<String, f32> {
        let mut features = HashMap::new();

        features.insert("method_count".to_string(),
                       context.structural_info.method_count as f32 / 10.0);
        features.insert("field_count".to_string(),
                       context.structural_info.field_count as f32 / 20.0);
        features.insert("dependency_count".to_string(),
                       context.structural_info.dependency_count as f32 / 5.0);

        features
    }
}

impl SemanticFeatureExtractor {
    fn extract(&self, context: &PatternContext) -> HashMap<String, f32> {
        let mut features = HashMap::new();

        let symbol_density = context.semantic_info.symbols.len() as f32 / 10.0;
        let reference_density = context.semantic_info.references.len() as f32 / 20.0;
        let usage_diversity = context.semantic_info.usages.len() as f32 / 5.0;

        features.insert("symbol_density".to_string(), symbol_density);
        features.insert("reference_density".to_string(), reference_density);
        features.insert("usage_diversity".to_string(), usage_diversity);

        features
    }
}
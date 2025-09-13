//! Model Consensus Engine
//!
//! This module implements intelligent result aggregation and consensus determination
//! across multiple AI models to improve accuracy and provide confidence scores.

use crate::config::{validate_config, OrchestrationConfig};
use crate::types::{
    ConsensusResult, ModelContribution, ModelId, ModelTask, RequestContext, VotingMechanism,
};
use crate::{OrchestrationError, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

/// Result aggregator for combining outputs from multiple models
#[derive(Debug)]
pub struct ResultAggregator {
    result_history: Arc<RwLock<HashMap<String, Vec<String>>>>,
    max_history_size: usize,
}

impl ResultAggregator {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            result_history: Arc::new(RwLock::new(HashMap::new())),
            max_history_size,
        }
    }

    pub async fn aggregate_results(&self, model_outputs: HashMap<ModelId, String>) -> String {
        if model_outputs.is_empty() {
            return String::new();
        }

        if model_outputs.len() == 1 {
            let (_, result) = model_outputs.into_iter().next().unwrap();
            return result;
        }

        // For now, implement simple majority voting on outputs
        // In practice, this could use more sophisticated NLP techniques
        self.majority_vote_aggregate(model_outputs).await
    }

    async fn majority_vote_aggregate(&self, model_outputs: HashMap<ModelId, String>) -> String {
        let mut output_counts: HashMap<String, usize> = HashMap::new();

        for (_, output) in model_outputs {
            let trimmed = output.trim().to_string();
            *output_counts.entry(trimmed).or_insert(0) += 1;
        }

        // Return the most common output
        output_counts
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(output, _)| output)
            .unwrap_or_else(|| String::new())
    }

    pub async fn normalize_output(&self, output: &str) -> String {
        // Basic output normalization
        let normalized = output
            .trim()
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        // Cache result for potential reuse
        let mut history = self.result_history.write().await;
        let key = format!("{:x}", md5::compute(output.as_bytes()));
        let entry = history.entry(key).or_insert_with(Vec::new);

        if !entry.contains(&normalized) {
            entry.push(normalized.clone());
            // Keep history bounded
            if entry.len() > self.max_history_size {
                entry.remove(0);
            }
        }

        normalized
    }
}

/// Consensus calculator using various voting and scoring mechanisms
#[derive(Debug)]
pub struct ConsensusCalculator {
    voting_mechanism: VotingMechanism,
    confidence_function: ConsensusConfidence,
    result_cache: Arc<RwLock<HashMap<String, ConsensusResult>>>,
}

#[derive(Debug, Clone)]
pub enum ConsensusConfidence {
    MajorityThreshold(f64),
    AgreementPercentage,
    StatisticalVariance,
    WeightedScore,
}

impl ConsensusCalculator {
    pub async fn calculate_consensus(
        &self,
        model_outputs: HashMap<ModelId, String>,
        model_confidences: HashMap<ModelId, f64>,
    ) -> Result<ConsensusResult> {
        if model_outputs.is_empty() {
            return Err(OrchestrationError::ConsensusError(
                "No model outputs provided".to_string(),
            ));
        }

        let consensus_result = match self.voting_mechanism {
            VotingMechanism::Majority => {
                self.majority_voting_consensus(model_outputs, model_confidences)
                    .await
            }
            VotingMechanism::Weighted => {
                self.weighted_voting_consensus(model_outputs, model_confidences)
                    .await
            }
            VotingMechanism::ConfidenceBased => {
                self.confidence_based_consensus(model_outputs, model_confidences)
                    .await
            }
        };

        // Cache result
        let mut cache = self.result_cache.write().await;
        if cache.len() > 1000 {
            // Limit cache size
            // Remove oldest entries (this is a simple FIFO eviction)
            let to_remove: Vec<_> = cache.keys().take(cache.len() - 1000).cloned().collect();
            for key in to_remove {
                cache.remove(&key);
            }
        }

        Ok(consensus_result)
    }

    async fn majority_voting_consensus(
        &self,
        model_outputs: HashMap<ModelId, String>,
        model_confidences: HashMap<ModelId, f64>,
    ) -> ConsensusResult {
        let mut vote_counts: HashMap<String, (usize, Vec<ModelId>, f64)> = HashMap::new();

        for (model_id, output) in &model_outputs {
            let confidence = model_confidences.get(model_id).unwrap_or(&0.5);
            let trimmed = output.trim().to_string();

            let (count, voters, total_confidence) =
                vote_counts
                    .entry(trimmed.clone())
                    .or_insert((0, Vec::new(), 0.0));

            *count += 1;
            voters.push(*model_id);
            *total_confidence += confidence;
        }

        // Find the majority result
        let (final_result, (count, voters, total_confidence)) = vote_counts
            .into_iter()
            .max_by_key(|(_, (count, _, _))| *count)
            .unwrap_or_else(|| (String::new(), (0, Vec::new(), 0.0)));

        let agreement_percentage = if model_outputs.len() > 0 {
            count as f64 / model_outputs.len() as f64
        } else {
            0.0
        };

        let contributions: HashMap<ModelId, ModelContribution> = voters
            .into_iter()
            .map(|model_id| {
                let result = model_outputs.get(&model_id).unwrap().clone();
                let confidence = model_confidences.get(&model_id).unwrap_or(&0.5);
                let weight = confidence / total_confidence.max(1.0); // Normalize weights

                (
                    model_id,
                    ModelContribution {
                        model_id,
                        result,
                        confidence: *confidence,
                        weight_in_consensus: weight,
                    },
                )
            })
            .collect();

        let primary_model = voters.into_iter().next().unwrap_or(ModelId::new());

        ConsensusResult {
            final_result,
            confidence_score: agreement_percentage * (total_confidence / count.max(1) as f64),
            model_contributions: contributions,
            disagreement_score: (1.0 - agreement_percentage).clamp(0.0, 1.0),
            primary_model,
        }
    }

    async fn weighted_voting_consensus(
        &self,
        model_outputs: HashMap<ModelId, String>,
        model_confidences: HashMap<ModelId, f64>,
    ) -> ConsensusResult {
        let mut weight_scores: HashMap<String, (f64, Vec<ModelId>, f64)> = HashMap::new();

        for (model_id, output) in &model_outputs {
            let confidence = model_confidences.get(model_id).unwrap_or(&0.5);
            let trimmed = output.trim().to_string();

            let (total_weight, voters, total_confidence) = weight_scores
                .entry(trimmed.clone())
                .or_insert((0.0, Vec::new(), 0.0));

            *total_weight += confidence;
            voters.push(*model_id);
            *total_confidence += confidence;
        }

        // Find the weighted majority result
        let (final_result, (total_weight, voters, total_confidence)) = weight_scores
            .into_iter()
            .max_by(|a, b| {
                a.1 .0
                    .partial_cmp(&b.1 .0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or_else(|| (String::new(), (0.0, Vec::new(), 0.0)));

        let total_possible_weight = model_confidences.values().sum::<f64>();
        let agreement_percentage = if total_possible_weight > 0.0 {
            total_weight / total_possible_weight
        } else {
            0.0
        };

        let contributions: HashMap<ModelId, ModelContribution> = voters
            .into_iter()
            .map(|model_id| {
                let result = model_outputs.get(&model_id).unwrap().clone();
                let confidence = model_confidences.get(&model_id).unwrap_or(&0.5);
                let weight = confidence / total_confidence.max(1.0);

                (
                    model_id,
                    ModelContribution {
                        model_id,
                        result,
                        confidence: *confidence,
                        weight_in_consensus: weight,
                    },
                )
            })
            .collect();

        let primary_model = voters.into_iter().next().unwrap_or(ModelId::new());

        ConsensusResult {
            final_result,
            confidence_score: total_confidence / voters.len().max(1) as f64,
            model_contributions: contributions,
            disagreement_score: (1.0 - agreement_percentage).clamp(0.0, 1.0),
            primary_model,
        }
    }

    async fn confidence_based_consensus(
        &self,
        model_outputs: HashMap<ModelId, String>,
        model_confidences: HashMap<ModelId, f64>,
    ) -> ConsensusResult {
        // Use confidence scores directly to select the highest confidence output
        let (primary_model, best_output, best_confidence) = model_confidences
            .iter()
            .filter_map(|(model_id, confidence)| {
                model_outputs
                    .get(model_id)
                    .map(|output| (model_id, output, confidence))
            })
            .max_by(|a, b| a.2.partial_cmp(b.2).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or_else(|| (&ModelId::new(), &String::new(), &0.0));

        let contributions: HashMap<ModelId, ModelContribution> = model_outputs
            .keys()
            .map(|model_id| {
                let result = model_outputs.get(model_id).unwrap().clone();
                let confidence = model_confidences.get(model_id).unwrap_or(&0.5);

                (
                    *model_id,
                    ModelContribution {
                        model_id: *model_id,
                        result,
                        confidence: *confidence,
                        weight_in_consensus: if model_id == &primary_model { 1.0 } else { 0.0 },
                    },
                )
            })
            .collect();

        ConsensusResult {
            final_result: best_output.clone(),
            confidence_score: *best_confidence,
            model_contributions: contributions,
            disagreement_score: 0.5, // High disagreement when selecting single model
            primary_model: *primary_model,
        }
    }
}

/// Result validator for ensuring output quality and consistency
#[derive(Debug)]
pub struct ResultValidator {
    validation_rules: Vec<ValidationRule>,
    quality_thresholds: QualityThresholds,
}

#[derive(Debug, Clone)]
pub enum ValidationRule {
    MinLength(usize),
    NoEmptyResults,
    ConsistentFormat,
    LanguageDetection,
    CodeSyntaxValidation,
}

#[derive(Debug, Clone)]
pub struct QualityThresholds {
    pub min_confidence: f64,
    pub max_disagreement: f64,
    pub min_result_length: usize,
    pub max_result_length: usize,
}

impl ResultValidator {
    pub fn new() -> Self {
        Self {
            validation_rules: vec![
                ValidationRule::NoEmptyResults,
                ValidationRule::MinLength(10),
            ],
            quality_thresholds: QualityThresholds {
                min_confidence: 0.6,
                max_disagreement: 0.4,
                min_result_length: 1,
                max_result_length: 10000,
            },
        }
    }

    pub fn validate_result(&self, consensus_result: &ConsensusResult) -> Result<()> {
        // Check quality thresholds
        if consensus_result.confidence_score < self.quality_thresholds.min_confidence {
            return Err(OrchestrationError::ConsensusError(format!(
                "Consensus confidence {} below threshold {}",
                consensus_result.confidence_score, self.quality_thresholds.min_confidence
            )));
        }

        if consensus_result.disagreement_score > self.quality_thresholds.max_disagreement {
            return Err(OrchestrationError::ConsensusError(format!(
                "Consensus disagreement {} above threshold {}",
                consensus_result.disagreement_score, self.quality_thresholds.max_disagreement
            )));
        }

        // Apply validation rules
        for rule in &self.validation_rules {
            match rule {
                ValidationRule::NoEmptyResults => {
                    if consensus_result.final_result.trim().is_empty() {
                        return Err(OrchestrationError::ConsensusError(
                            "Empty result after consensus".to_string(),
                        ));
                    }
                }
                ValidationRule::MinLength(min_len) => {
                    if consensus_result.final_result.len() < *min_len {
                        return Err(OrchestrationError::ConsensusError(format!(
                            "Result length {} below minimum {}",
                            consensus_result.final_result.len(),
                            min_len
                        )));
                    }
                }
                ValidationRule::MaxLength(max_len) => {
                    if consensus_result.final_result.len() > *max_len {
                        return Err(OrchestrationError::ConsensusError(format!(
                            "Result length {} above maximum {}",
                            consensus_result.final_result.len(),
                            max_len
                        )));
                    }
                }
                ValidationRule::ConsistentFormat => {
                    // Placeholder for more sophisticated format consistency checks
                    self.check_format_consistency(&consensus_result.final_result)?;
                }
                ValidationRule::LanguageDetection => {
                    // Placeholder for language detection validation
                    self.check_language_consistency(&consensus_result.final_result)?;
                }
                ValidationRule::CodeSyntaxValidation => {
                    // Placeholder for syntax validation
                    self.check_code_syntax(&consensus_result.final_result)?;
                }
            }
        }

        Ok(())
    }

    fn check_format_consistency(&self, result: &str) -> Result<()> {
        // Basic format checking (this could be much more sophisticated)
        if result.contains("<?xml") && !result.contains("</") || !result.contains(">") {
            return Err(OrchestrationError::ConsensusError(
                "Inconsistent XML-like format".to_string(),
            ));
        }
        Ok(())
    }

    fn check_language_consistency(&self, result: &str) -> Result<()> {
        // Placeholder - could use language detection libraries
        // For now, just check for basic syntax patterns
        let has_code_patterns =
            result.contains(';') || result.contains('{') || result.contains('}');
        let has_text_patterns =
            result.contains('.') || result.contains(' ') || result.contains('\n');

        if has_code_patterns && has_text_patterns {
            // Mixed content is okay for some tasks
        }

        Ok(())
    }

    fn check_code_syntax(&self, result: &str) -> Result<()> {
        // Placeholder - would integrate with language-specific parsers
        // For now, check for obvious syntax errors in common formats
        if result.contains("{{") && !result.contains("}}")
            || result.contains("}}") && !result.contains("{{")
        {
            return Err(OrchestrationError::ConsensusError(
                "Mismatched template syntax".to_string(),
            ));
        }
        Ok(())
    }
}

/// Confidence scorer for dynamic confidence calculation
#[derive(Debug)]
pub struct ConfidenceScorer {
    historical_performances: Arc<RwLock<HashMap<ModelId, Vec<f64>>>>,
    decay_factor: f64,
}

impl ConfidenceScorer {
    pub fn new(decay_factor: f64) -> Self {
        Self {
            historical_performances: Arc::new(RwLock::new(HashMap::new())),
            decay_factor,
        }
    }

    pub async fn calculate_model_confidence(&self, model_id: &ModelId) -> f64 {
        let historical = self.historical_performances.read().await;
        if let Some(performances) = historical.get(model_id) {
            if performances.is_empty() {
                return 0.5; // Default neutral confidence
            }

            // Calculate weighted average with exponential decay
            let mut total_weight = 0.0;
            let mut total_score = 0.0;

            for (i, &score) in performances.iter().enumerate() {
                let weight = self
                    .decay_factor
                    .powi(performances.len() as i32 - i as i32 - 1);
                total_weight += weight;
                total_score += score * weight;
            }

            if total_weight > 0.0 {
                total_score / total_weight
            } else {
                0.5
            }
        } else {
            0.5
        }
    }

    pub async fn record_performance(&self, model_id: ModelId, accuracy_score: f64) {
        let mut historical = self.historical_performances.write().await;
        let performances = historical.entry(model_id).or_insert_with(Vec::new);

        performances.push(accuracy_score);

        // Keep last 100 performance records
        if performances.len() > 100 {
            performances.remove(0);
        }
    }
}

/// Disagreement resolver for handling conflicting model outputs
#[derive(Debug)]
pub struct DisagreementResolver {
    resolution_strategy: DisagreementStrategy,
    conflict_history: Arc<RwLock<Vec<ConflictRecord>>>,
}

#[derive(Debug, Clone)]
pub enum DisagreementStrategy {
    FallbackToHighestConfidence,
    WeightedMedianSelection,
    ExpertVoting(String),
}

#[derive(Debug, Clone)]
pub struct ConflictRecord {
    pub timestamp: Instant,
    pub conflicting_outputs: Vec<String>,
    pub resolution_strategy: DisagreementStrategy,
    pub final_result: String,
    pub confidence: f64,
}

impl DisagreementResolver {
    pub fn new(strategy: DisagreementStrategy) -> Self {
        Self {
            resolution_strategy: strategy,
            conflict_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn resolve_disagreement(
        &self,
        conflicting_outputs: HashMap<ModelId, String>,
        model_confidences: HashMap<ModelId, f64>,
    ) -> String {
        let result = match &self.resolution_strategy {
            DisagreementStrategy::FallbackToHighestConfidence => {
                self.resolve_by_highest_confidence(&conflicting_outputs, &model_confidences)
            }
            DisagreementStrategy::WeightedMedianSelection => {
                self.resolve_by_weighted_median(&conflicting_outputs, &model_confidences)
            }
            DisagreementStrategy::ExpertVoting(expert_model) => {
                // If expert model is available, use its output
                conflicting_outputs
                    .iter()
                    .find(|(model_id, _)| model_id.0.to_string().contains(expert_model))
                    .map(|(_, output)| output.clone())
                    .unwrap_or_else(|| {
                        self.resolve_by_highest_confidence(&conflicting_outputs, &model_confidences)
                    })
            }
        };

        // Record the conflict for future analysis
        let mut history = self.conflict_history.write().await;
        history.push(ConflictRecord {
            timestamp: Instant::now(),
            conflicting_outputs: conflicting_outputs.values().cloned().collect(),
            resolution_strategy: self.resolution_strategy.clone(),
            final_result: result.clone(),
            confidence: 0.5, // Placeholder
        });

        // Keep history bounded
        if history.len() > 1000 {
            history.remove(0);
        }

        result
    }

    fn resolve_by_highest_confidence(
        &self,
        conflicting_outputs: &HashMap<ModelId, String>,
        model_confidences: &HashMap<ModelId, f64>,
    ) -> String {
        let (result, _) = model_confidences
            .iter()
            .filter_map(|(model_id, confidence)| {
                conflicting_outputs
                    .get(model_id)
                    .map(|output| (output.clone(), confidence))
            })
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or_else(|| (String::new(), &0.0));

        result
    }

    fn resolve_by_weighted_median(
        &self,
        conflicting_outputs: &HashMap<ModelId, String>,
        model_confidences: &HashMap<ModelId, f64>,
    ) -> String {
        // Sort outputs by confidence and pick the middle one
        let mut outputs: Vec<(String, f64)> = conflicting_outputs
            .iter()
            .filter_map(|(model_id, output)| {
                model_confidences
                    .get(model_id)
                    .map(|conf| (output.clone(), *conf))
            })
            .collect();

        outputs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mid = outputs.len() / 2;
        outputs.get(mid).cloned().unwrap_or((String::new(), 0.0)).0
    }
}

/// Main Model Consensus Engine
#[derive(Debug)]
pub struct ModelConsensusEngine {
    pub aggregator: Arc<ResultAggregator>,
    pub consensus_calculator: Arc<ConsensusCalculator>,
    pub validator: Arc<ResultValidator>,
    pub confidence_scorer: Arc<ConfidenceScorer>,
    pub disagreement_resolver: Arc<DisagreementResolver>,
    config: OrchestrationConfig,
}

impl ModelConsensusEngine {
    pub fn new(config: OrchestrationConfig) -> Result<Self> {
        validate_config(&config)?;
        Ok(Self {
            aggregator: Arc::new(ResultAggregator::new(100)),
            consensus_calculator: Arc::new(ConsensusCalculator {
                voting_mechanism: config.consensus_config.voting_mechanism.clone(),
                confidence_function: ConsensusConfidence::WeightedScore,
                result_cache: Arc::new(RwLock::new(HashMap::new())),
            }),
            validator: Arc::new(ResultValidator::new()),
            confidence_scorer: Arc::new(ConfidenceScorer::new(0.9)),
            disagreement_resolver: Arc::new(DisagreementResolver::new(
                DisagreementStrategy::FallbackToHighestConfidence,
            )),
            config,
        })
    }

    pub async fn process_consensus(
        &self,
        model_outputs: HashMap<ModelId, String>,
        context: &RequestContext,
    ) -> Result<ConsensusResult> {
        // Normalize outputs
        let mut normalized_outputs = HashMap::new();
        for (model_id, output) in &model_outputs {
            let normalized = self.aggregator.normalize_output(output).await;
            normalized_outputs.insert(*model_id, normalized);
        }

        // Get confidence scores for each model
        let mut model_confidences = HashMap::new();
        for model_id in normalized_outputs.keys() {
            let confidence = self
                .confidence_scorer
                .calculate_model_confidence(model_id)
                .await;
            model_confidences.insert(*model_id, confidence);
        }

        // Check if we have multiple outputs to build consensus
        if normalized_outputs.len()
            >= self.config.consensus_config.min_models_for_consensus as usize
        {
            let consensus_result = self
                .consensus_calculator
                .calculate_consensus(normalized_outputs, model_confidences)
                .await?;

            // Validate the consensus result
            self.validator.validate_result(&consensus_result)?;

            Ok(consensus_result)
        } else if !normalized_outputs.is_empty() {
            // Not enough models for consensus, use the best available output
            let fallback_result = self
                .disagreement_resolver
                .resolve_disagreement(normalized_outputs, model_confidences)
                .await;

            // Create consensus result with single contribution
            let model_id = ModelId::new();
            let contributions = HashMap::from([(
                model_id,
                ModelContribution {
                    model_id,
                    result: fallback_result.clone(),
                    confidence: 0.5,
                    weight_in_consensus: 1.0,
                },
            )]);

            Ok(ConsensusResult {
                final_result: fallback_result,
                confidence_score: 0.5,
                model_contributions: contributions,
                disagreement_score: 0.0,
                primary_model: model_id,
            })
        } else {
            Err(OrchestrationError::ConsensusError(
                "No model outputs available".to_string(),
            ))
        }
    }

    pub async fn record_performance(&self, model_id: ModelId, performance_score: f64) {
        self.confidence_scorer
            .record_performance(model_id, performance_score)
            .await;
    }

    pub fn update_voting_mechanism(&self, mechanism: VotingMechanism) {
        // Note: This would require Arc<Mutex<P>> instead of Arc<P> for mutability
        // For now, this is a placeholder that would need architectural changes
        todo!("Implement voting mechanism updates with proper synchronization");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OrchestrationConfigBuilder;

    #[tokio::test]
    async fn test_consensus_majority_voting() {
        let config = OrchestrationConfigBuilder::new()
            .with_min_models_for_consensus(2)
            .build();
        let engine = ModelConsensusEngine::new(config).await.unwrap();

        let model_outputs = HashMap::from([
            (ModelId::new(), "Result A".to_string()),
            (ModelId::new(), "Result A".to_string()),
            (ModelId::new(), "Result B".to_string()),
        ]);

        let context = RequestContext {
            task_type: ModelTask::Completion,
            input_length: 100,
            priority: RequestPriority::Medium,
            expected_complexity: crate::types::Complexity::Medium,
            acceptable_latency: Duration::from_secs(5),
            preferred_hardware: None,
        };

        let consensus = engine
            .process_consensus(model_outputs, &context)
            .await
            .unwrap();
        assert_eq!(consensus.final_result, "Result A");
        assert!(consensus.confidence_score > 0.0);
    }

    #[tokio::test]
    async fn test_empty_inputs() {
        let config = OrchestrationConfigBuilder::new().build();
        let engine = ModelConsensusEngine::new(config).await.unwrap();

        let context = RequestContext {
            task_type: ModelTask::Completion,
            input_length: 100,
            priority: RequestPriority::Medium,
            expected_complexity: crate::types::Complexity::Medium,
            acceptable_latency: Duration::from_secs(5),
            preferred_hardware: None,
        };

        let result = engine.process_consensus(HashMap::new(), &context).await;
        assert!(result.is_err());
    }
}

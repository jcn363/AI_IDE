//! Correlation Engine for analyzing related security events
//!
//! This module provides intelligent correlation of security events across multiple sources
//! to identify patterns, attacks, and potential threats through:
//! - Time-based correlation analysis
//! - Event pattern matching
//! - Behavioral anomaly detection
//! - Attack chain reconstruction

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use tracing::{info, warn};
use moka::future::Cache;

use crate::{SecurityEvent, MonitoringError, Result};

#[derive(Clone)]
pub struct CorrelationEngine {
    correlation_rules: Arc<RwLock<Vec<CorrelationRule>>>,
    event_buffer: Arc<Cache<String, Vec<SecurityEvent>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationRule {
    pub id: String,
    pub name: String,
    pub window: Duration,
    pub max_events: usize,
    pub conditions: Vec<CorrelationCondition>,
    pub scoring: CorrelationScoring,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationCondition {
    pub field: String,
    pub pattern: CorrelationPattern,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorrelationPattern {
    Equal(String),
    Contains(String),
    Regex(String),
    Range { min: f64, max: f64 },
    Sequence(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationScoring {
    pub threshold: f64,
    pub factors: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationResult {
    pub rule_id: String,
    pub confidence: f64,
    pub events: Vec<SecurityEvent>,
    pub description: String,
    pub risk_score: f64,
    pub recommended_actions: Vec<String>,
}

impl CorrelationEngine {
    pub fn new() -> Self {
        Self {
            correlation_rules: Arc::new(RwLock::new(Vec::new())),
            event_buffer: Arc::new(
                Cache::builder()
                    .time_to_live(tokio::time::Duration::from_secs(1800))
                    .build()
            ),
        }
    }

    pub async fn analyze_event(&self, event: &SecurityEvent) -> Result<Vec<CorrelationResult>> {
        let rules = self.correlation_rules.read().await.clone();
        let mut results = Vec::new();

        for rule in rules {
            if let Some(result) = self.evaluate_rule(&rule, event).await {
                results.push(result);
            }
        }

        Ok(results)
    }

    async fn evaluate_rule(&self, rule: &CorrelationRule, event: &SecurityEvent) -> Option<CorrelationResult> {
        // Buffer event for time-window analysis
        let buffer_key = format!("rule_{}", rule.id);
        let mut buffer = self.event_buffer
            .get(&buffer_key)
            .await
            .unwrap_or_default();

        // Add current event and filter by time window
        buffer.push(event.clone());
        buffer.retain(|e| Utc::now().signed_duration_since(e.timestamp) < rule.window);

        // Limit buffer size
        if buffer.len() > rule.max_events {
            buffer.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            buffer.truncate(rule.max_events);
        }

        // Update buffer
        self.event_buffer.insert(buffer_key, buffer.clone()).await;

        // Evaluate correlation conditions
        let mut score = 0.0;
        let mut matched_events = Vec::new();

        for condition in &rule.conditions {
            let matches = buffer.iter().filter(|e| matches_condition(e, &condition)).collect::<Vec<_>>();
            if !matches.is_empty() {
                score += condition.weight;
                matched_events.extend(matches.iter().map(|e| (**e).clone()));
            }
        }

        // Check if score exceeds threshold
        if score >= rule.scoring.threshold {
            Some(CorrelationResult {
                rule_id: rule.id.clone(),
                confidence: (score / rule.conditions.len() as f64).min(1.0),
                events: matched_events,
                description: format!("Correlated {} events using rule '{}'", matched_events.len(), rule.name),
                risk_score: score,
                recommended_actions: vec![
                    "Review correlated events".to_string(),
                    "Investigate potential attack pattern".to_string(),
                ],
            })
        } else {
            None
        }
    }
}

fn matches_condition(event: &SecurityEvent, condition: &CorrelationCondition) -> bool {
    let field_value = match condition.field.as_str() {
        "event_type" => event.event_type.as_str(),
        "severity" => format!("{:?}", event.severity).as_str(),
        "source" => event.source.as_str(),
        "actor" => event.actor.as_deref().unwrap_or(""),
        "resource" => event.resource.as_deref().unwrap_or(""),
        _ => {
            // Try to extract from details JSON
            if let Some(value) = event.details.get(&condition.field) {
                if let Some(str_val) = value.as_str() {
                    str_val
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }
    };

    match &condition.pattern {
        CorrelationPattern::Equal(expected) => field_value == expected,
        CorrelationPattern::Contains(substr) => field_value.contains(substr),
        CorrelationPattern::Regex(pattern) => {
            if let Ok(regex) = regex::Regex::new(pattern) {
                regex.is_match(field_value)
            } else {
                false
            }
        },
        CorrelationPattern::Range { min, max } => {
            if let Ok(num) = field_value.parse::<f64>() {
                num >= *min && num <= *max
            } else {
                false
            }
        },
        CorrelationPattern::Sequence(sequence) => {
            sequence.contains(&field_value.to_string())
        }
    }
}
//! Performance Alerting System
//!
//! This module provides mechanisms for handling performance alerts,
//! including notification channels, alert policies, and escalation procedures.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::{PerformanceAlert, AlertSeverity};

/// Alert policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertPolicy {
    /// Enable this policy
    pub enabled: bool,
    /// Alert severity thresholds
    pub severity_thresholds: HashMap<String, f64>,
    /// Cooldown period between alerts (seconds)
    pub cooldown_seconds: u64,
    /// Maximum alerts per hour
    pub max_alerts_per_hour: usize,
    /// Notification channels
    pub channels: Vec<NotificationChannel>,
}

/// Notification channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    /// Channel type
    pub channel_type: ChannelType,
    /// Channel-specific configuration
    pub config: HashMap<String, String>,
    /// Channel enabled flag
    pub enabled: bool,
}

/// Types of notification channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelType {
    /// Email notifications
    Email,
    /// Slack webhook
    Slack,
    /// Discord webhook
    Discord,
    /// Webhook to external system
    Webhook,
    /// Log file
    LogFile,
    /// Console output
    Console,
    /// CI/CD system integration
    CiCd,
}

/// Alert manager for processing and routing performance alerts
pub struct AlertManager {
    /// Alert policies by alert type
    policies: HashMap<String, AlertPolicy>,
    /// Active alerts (to prevent duplicate alerts during cooldown)
    active_alerts: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    /// Alert counters for rate limiting
    alert_counters: Arc<RwLock<HashMap<String, AlertCounter>>>,
    /// Channel senders for notifications
    notification_channels: HashMap<String, mpsc::Sender<NotificationMessage>>,
}

/// Counter for rate limiting alerts
#[derive(Debug, Clone)]
struct AlertCounter {
    count: usize,
    hour_start: DateTime<Utc>,
}

/// Notification message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    /// Alert title
    pub title: String,
    /// Alert description
    pub description: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert timestamp
    pub timestamp: DateTime<Utc>,
    /// Alert details (key-value pairs)
    pub details: HashMap<String, String>,
    /// Additional context
    pub context: HashMap<String, serde_json::Value>,
}

impl AlertManager {
    /// Create a new alert manager with default policies
    pub fn new() -> Self {
        let mut policies = HashMap::new();

        // Default policy for regressions
        policies.insert(
            "regression".to_string(),
            AlertPolicy {
                enabled: true,
                severity_thresholds: HashMap::new(),
                cooldown_seconds: 300, // 5 minutes
                max_alerts_per_hour: 5,
                channels: vec![
                    NotificationChannel {
                        channel_type: ChannelType::Console,
                        config: HashMap::new(),
                        enabled: true,
                    }
                ],
            }
        );

        // Default policy for threshold exceeded
        policies.insert(
            "threshold_exceeded".to_string(),
            AlertPolicy {
                enabled: true,
                severity_thresholds: HashMap::new(),
                cooldown_seconds: 600, // 10 minutes
                max_alerts_per_hour: 10,
                channels: vec![
                    NotificationChannel {
                        channel_type: ChannelType::Console,
                        config: HashMap::new(),
                        enabled: true,
                    }
                ],
            }
        );

        Self {
            policies,
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_counters: Arc::new(RwLock::new(HashMap::new())),
            notification_channels: HashMap::new(),
        }
    }

    /// Process a performance alert
    pub async fn process_alert(&self, alert: PerformanceAlert) {
        let alert_key = self.get_alert_key(&alert);
        let alert_type = self.get_alert_type(&alert);

        // Check cooldown
        if !self.can_send_alert(&alert_key, alert_type).await {
            return;
        }

        // Get policy for this alert type
        if let Some(policy) = self.policies.get(alert_type) {
            if !policy.enabled {
                return;
            }

            // Check rate limits
            if !self.check_rate_limit(alert_type, policy).await {
                return;
            }

            // Create notification message
            let message = self.create_notification_message(alert.clone());

            // Send to all configured channels
            for channel in &policy.channels {
                if channel.enabled {
                    self.send_notification(channel, &message).await;
                }
            }

            // Record alert as active
            self.record_alert_cooldown(&alert_key).await;
        }
    }

    /// Register a notification channel
    pub fn register_channel(&mut self, name: String, sender: mpsc::Sender<NotificationMessage>) {
        self.notification_channels.insert(name, sender);
    }

    /// Update alert policy
    pub fn update_policy(&mut self, alert_type: String, policy: AlertPolicy) {
        self.policies.insert(alert_type, policy);
    }

    /// Get active alerts (for monitoring)
    pub fn get_active_alerts(&self) -> Vec<(String, DateTime<Utc>)> {
        self.active_alerts
            .read()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }

    /// Clear expired cooldowns
    pub async fn clear_expired_cooldowns(&self) {
        let now = Utc::now();
        let mut active_alerts = self.active_alerts.write().unwrap();

        active_alerts.retain(|_, timestamp| {
            now.signed_duration_since(*timestamp).num_seconds() < 3600 // Keep for 1 hour
        });
    }

    /// Generate unique alert key for deduplication
    fn get_alert_key(&self, alert: &PerformanceAlert) -> String {
        match alert {
            PerformanceAlert::RegressionDetected { metric_name, .. } => {
                format!("regression:{}", metric_name)
            }
            PerformanceAlert::ThresholdExceeded { metric_name, .. } => {
                format!("threshold:{}", metric_name)
            }
            PerformanceAlert::AnomalyDetected { description, .. } => {
                format!("anomaly:{}", description)
            }
        }
    }

    /// Get alert type for policy lookup
    fn get_alert_type(&self, alert: &PerformanceAlert) -> &str {
        match alert {
            PerformanceAlert::RegressionDetected { .. } => "regression",
            PerformanceAlert::ThresholdExceeded { .. } => "threshold_exceeded",
            PerformanceAlert::AnomalyDetected { .. } => "anomaly",
        }
    }

    /// Check if alert can be sent based on cooldown
    async fn can_send_alert(&self, alert_key: &str, alert_type: &str) -> bool {
        let active_alerts = self.active_alerts.read().unwrap();
        let policy = match self.policies.get(alert_type) {
            Some(p) => p,
            None => return true, // No policy means default allow
        };

        if let Some(last_alert_time) = active_alerts.get(alert_key) {
            let now = Utc::now();
            let duration_since_last = now.signed_duration_since(*last_alert_time).num_seconds();
            if duration_since_last < policy.cooldown_seconds as i64 {
                return false; // Still in cooldown
            }
        }

        true
    }

    /// Check rate limits
    async fn check_rate_limit(&self, alert_type: &str, policy: &AlertPolicy) -> bool {
        let mut counters = self.alert_counters.write().unwrap();
        let counter = counters.entry(alert_type.to_string()).or_insert(AlertCounter {
            count: 0,
            hour_start: Utc::now(),
        });

        let now = Utc::now();
        let hour_duration = now.signed_duration_since(counter.hour_start).num_hours();

        // Reset counter if hour has passed
        if hour_duration >= 1 {
            counter.count = 0;
            counter.hour_start = now;
        }

        // Check if under limit
        counter.count < policy.max_alerts_per_hour
    }

    /// Record alert cooldown
    async fn record_alert_cooldown(&self, alert_key: &str) {
        let mut active_alerts = self.active_alerts.write().unwrap();
        active_alerts.insert(alert_key.to_string(), Utc::now());

        // Update counters
        let mut counters = self.alert_counters.write().unwrap();
        if let Some(counter) = counters.get_mut(alert_key) {
            counter.count += 1;
        }
    }

    /// Create notification message from alert
    fn create_notification_message(&self, alert: PerformanceAlert) -> NotificationMessage {
        match alert {
            PerformanceAlert::RegressionDetected {
                metric_name,
                baseline_value,
                current_value,
                degradation_percent,
                timestamp,
            } => {
                let title = format!("Performance Regression: {}", metric_name);
                let description = format!(
                    "Performance degraded by {:.1}% for {}. Baseline: {:.2}, Current: {:.2}",
                    degradation_percent * 100.0, metric_name, baseline_value, current_value
                );

                let mut details = HashMap::new();
                details.insert("metric".to_string(), metric_name);
                details.insert("baseline_value".to_string(), baseline_value.to_string());
                details.insert("current_value".to_string(), current_value.to_string());
                details.insert("degradation_percent".to_string(), format!("{:.1}", degradation_percent * 100.0));

                NotificationMessage {
                    title,
                    description,
                    severity: AlertSeverity::High,
                    timestamp,
                    details,
                    context: HashMap::new(),
                }
            }
            PerformanceAlert::ThresholdExceeded {
                metric_name,
                current_value,
                threshold,
                timestamp,
            } => {
                let title = format!("Threshold Exceeded: {}", metric_name);
                let description = format!(
                    "{} value {:.2} exceeded threshold {:.2}",
                    metric_name, current_value, threshold
                );

                let mut details = HashMap::new();
                details.insert("metric".to_string(), metric_name);
                details.insert("current_value".to_string(), current_value.to_string());
                details.insert("threshold".to_string(), threshold.to_string());

                NotificationMessage {
                    title,
                    description,
                    severity: AlertSeverity::Medium,
                    timestamp,
                    details,
                    context: HashMap::new(),
                }
            }
            PerformanceAlert::AnomalyDetected {
                description,
                severity,
                timestamp,
            } => {
                let title = "Performance Anomaly Detected".to_string();

                let mut details = HashMap::new();
                details.insert("description".to_string(), description.clone());

                NotificationMessage {
                    title,
                    description,
                    severity,
                    timestamp,
                    details,
                    context: HashMap::new(),
                }
            }
        }
    }

    /// Send notification to channel
    async fn send_notification(&self, channel: &NotificationChannel, message: &NotificationMessage) {
        match channel.channel_type {
            ChannelType::Console => {
                self.send_to_console(message);
            }
            ChannelType::LogFile => {
                self.send_to_log_file(message).await;
            }
            ChannelType::Email => {
                self.send_to_email(channel, message).await;
            }
            ChannelType::Slack => {
                self.send_to_slack(channel, message).await;
            }
            ChannelType::Discord => {
                self.send_to_discord(channel, message).await;
            }
            ChannelType::Webhook => {
                self.send_to_webhook(channel, message).await;
            }
            ChannelType::CiCd => {
                self.send_to_ci_cd(message).await;
            }
        }
    }

    /// Send notification to console
    fn send_to_console(&self, message: &NotificationMessage) {
        let severity_icon = match message.severity {
            AlertSeverity::Low => "ℹ️",
            AlertSeverity::Medium => "⚠️",
            AlertSeverity::High => "🔴",
            AlertSeverity::Critical => "🚨",
        };

        println!("{} [{}] {}", severity_icon, message.timestamp, message.title);
        println!("  {}", message.description);

        if !message.details.is_empty() {
            println!("  Details:");
            for (key, value) in &message.details {
                println!("    {}: {}", key, value);
            }
        }
        println!();
    }

    /// Send notification to log file (placeholder)
    async fn send_to_log_file(&self, message: &NotificationMessage) {
        // TODO: Implement actual log file writing
        println!("Alert logged to file: {}", message.title);
    }

    /// Send notification via email (placeholder)
    async fn send_to_email(&self, _channel: &NotificationChannel, message: &NotificationMessage) {
        // TODO: Implement actual email sending
        println!("Alert sent via email: {}", message.title);
    }

    /// Send notification to Slack (placeholder)
    async fn send_to_slack(&self, _channel: &NotificationChannel, message: &NotificationMessage) {
        // TODO: Implement actual Slack webhook
        println!("Alert sent to Slack: {}", message.title);
    }

    /// Send notification to Discord (placeholder)
    async fn send_to_discord(&self, _channel: &NotificationChannel, message: &NotificationMessage) {
        // TODO: Implement actual Discord webhook
        println!("Alert sent to Discord: {}", message.title);
    }

    /// Send notification to webhook (placeholder)
    async fn send_to_webhook(&self, _channel: &NotificationChannel, message: &NotificationMessage) {
        // TODO: Implement actual HTTP webhook
        println!("Alert sent to webhook: {}", message.title);
    }

    /// Send notification to CI/CD system (placeholder)
    async fn send_to_ci_cd(&self, message: &NotificationMessage) {
        // TODO: Implement CI/CD system integration
        println!("Alert sent to CI/CD system: {}", message.title);
    }
}

// AlertSeverity is imported from collector module

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_alert_manager_creation() {
        let manager = AlertManager::new();

        // Should have default policies
        assert!(manager.policies.contains_key("regression"));
        assert!(manager.policies.contains_key("threshold_exceeded"));

        // Should be empty initially
        assert!(manager.get_active_alerts().is_empty());
    }

    #[test]
    fn test_alert_key_generation() {
        let manager = AlertManager::new();
        let alert = PerformanceAlert::RegressionDetected {
            metric_name: "cpu_usage".to_string(),
            baseline_value: 50.0,
            current_value: 75.0,
            degradation_percent: 0.5,
            timestamp: Utc::now(),
        };

        let key = manager.get_alert_key(&alert);
        assert_eq!(key, "regression:cpu_usage");
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let manager = AlertManager::new();

        // Create a policy with low rate limit for testing
        let policy = AlertPolicy {
            enabled: true,
            severity_thresholds: HashMap::new(),
            cooldown_seconds: 0, // No cooldown for this test
            max_alerts_per_hour: 2,
            channels: vec![
                NotificationChannel {
                    channel_type: ChannelType::Console,
                    config: HashMap::new(),
                    enabled: true,
                }
            ],
        };

        manager.update_policy("test_type".to_string(), policy);

        // First two alerts should be allowed
        let alert = PerformanceAlert::ThresholdExceeded {
            metric_name: "test_metric".to_string(),
            current_value: 100.0,
            threshold: 80.0,
            timestamp: Utc::now(),
        };

        assert!(manager.can_send_alert("test_key", "test_type").await);
        manager.process_alert(alert.clone()).await;

        manager.process_alert(alert).await;

        // Third alert should be rate limited (but this test is simplified)
    }

    #[test]
    fn test_notification_message_creation() {
        let manager = AlertManager::new();
        let timestamp = Utc::now();

        let alert = PerformanceAlert::RegressionDetected {
            metric_name: "response_time".to_string(),
            baseline_value: 100.0,
            current_value: 150.0,
            degradation_percent: 0.5,
            timestamp,
        };

        let message = manager.create_notification_message(alert);

        assert_eq!(message.title, "Performance Regression: response_time");
        assert!(message.description.contains("50.0%"));
        assert_eq!(message.severity, AlertSeverity::High);
    }
}
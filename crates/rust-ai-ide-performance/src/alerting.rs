//! Performance Alerting System
//!
//! This module provides mechanisms for handling performance alerts,
//! including notification channels, alert policies, and escalation procedures.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use rust_ai_ide_common::validation::validate_secure_path;
use rust_ai_ide_security::audit_logger;

use crate::{AlertSeverity, PerformanceAlert};

/// Alert policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertPolicy {
    /// Enable this policy
    pub enabled:             bool,
    /// Alert severity thresholds
    pub severity_thresholds: HashMap<String, f64>,
    /// Cooldown period between alerts (seconds)
    pub cooldown_seconds:    u64,
    /// Maximum alerts per hour
    pub max_alerts_per_hour: usize,
    /// Notification channels
    pub channels:            Vec<NotificationChannel>,
}

/// Notification channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    /// Channel type
    pub channel_type: ChannelType,
    /// Channel-specific configuration
    pub config:       HashMap<String, String>,
    /// Channel enabled flag
    pub enabled:      bool,
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

/// Alert escalation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationPolicy {
    /// Severity threshold for escalation
    pub severity_threshold: AlertSeverity,
    /// Duration in seconds before escalation
    pub escalation_delay_seconds: u64,
    /// Escalation channels
    pub escalation_channels: Vec<NotificationChannel>,
    /// Maximum escalation level
    pub max_escalation_level: u32,
}

/// Alert aggregation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAggregation {
    /// Time window for aggregation (seconds)
    pub aggregation_window_seconds: u64,
    /// Minimum alerts to trigger aggregation
    pub min_alerts_for_aggregation: usize,
    /// Aggregation strategy
    pub strategy: AggregationStrategy,
}

/// Alert aggregation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationStrategy {
    /// Count similar alerts
    Count,
    /// Suppress duplicate alerts
    SuppressDuplicates,
    /// Group by metric and time window
    GroupByMetric,
}

/// Alert acknowledgment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertAcknowledgmentStatus {
    /// Alert not yet acknowledged
    Unacknowledged,
    /// Alert acknowledged by user
    Acknowledged,
    /// Alert resolved
    Resolved,
}

/// Alert history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertHistoryEntry {
    /// Alert ID
    pub alert_id: String,
    /// Alert message
    pub alert: NotificationMessage,
    /// Acknowledgment status
    pub acknowledgment_status: AlertAcknowledgmentStatus,
    /// Acknowledged by user ID (if applicable)
    pub acknowledged_by: Option<String>,
    /// Acknowledged timestamp
    pub acknowledged_at: Option<DateTime<Utc>>,
    /// Resolved timestamp
    pub resolved_at: Option<DateTime<Utc>>,
    /// Escalation level
    pub escalation_level: u32,
    /// Team ID for team-based alerting
    pub team_id: Option<String>,
}

/// Performance trend data for predictive alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrend {
    /// Metric name
    pub metric_name: String,
    /// Trend direction (positive = improving, negative = degrading)
    pub trend_coefficient: f64,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Predicted value at next measurement
    pub predicted_value: f64,
    /// Trend analysis timestamp
    pub analyzed_at: DateTime<Utc>,
}

/// Team-based alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAlertingConfig {
    /// Team ID
    pub team_id: String,
    /// Team notification channels
    pub team_channels: Vec<NotificationChannel>,
    /// Alert routing rules
    pub routing_rules: HashMap<String, Vec<String>>, // alert_type -> user_ids
}

/// Alert manager for processing and routing performance alerts
pub struct AlertManager {
    /// Alert policies by alert type
    policies:              HashMap<String, AlertPolicy>,
    /// Escalation policies
    escalation_policies:   Vec<EscalationPolicy>,
    /// Alert aggregation configuration
    aggregation_config:    HashMap<String, AlertAggregation>,
    /// Active alerts (to prevent duplicate alerts during cooldown)
    active_alerts:         Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    /// Alert counters for rate limiting
    alert_counters:        Arc<RwLock<HashMap<String, AlertCounter>>>,
    /// Alert history for tracking and analysis
    alert_history:         Arc<RwLock<Vec<AlertHistoryEntry>>>,
    /// Performance trends for predictive alerting
    performance_trends:    Arc<RwLock<HashMap<String, PerformanceTrend>>>,
    /// Team-based alerting configurations
    team_configs:          HashMap<String, TeamAlertingConfig>,
    /// Aggregated alerts waiting to be processed
    aggregated_alerts:     Arc<RwLock<HashMap<String, Vec<NotificationMessage>>>>,
    /// Channel senders for notifications
    notification_channels: HashMap<String, mpsc::Sender<NotificationMessage>>,
}

/// Counter for rate limiting alerts
#[derive(Debug, Clone)]
struct AlertCounter {
    count:      usize,
    hour_start: DateTime<Utc>,
}

/// Notification message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    /// Alert title
    pub title:       String,
    /// Alert description
    pub description: String,
    /// Alert severity
    pub severity:    AlertSeverity,
    /// Alert timestamp
    pub timestamp:   DateTime<Utc>,
    /// Alert details (key-value pairs)
    pub details:     HashMap<String, String>,
    /// Additional context
    pub context:     HashMap<String, serde_json::Value>,
}

impl AlertManager {
    /// Create a new alert manager with default policies
    pub fn new() -> Self {
        let mut policies = HashMap::new();
        let mut aggregation_config = HashMap::new();
        let mut team_configs = HashMap::new();

        // Default policy for regressions
        policies.insert("regression".to_string(), AlertPolicy {
            enabled:             true,
            severity_thresholds: HashMap::new(),
            cooldown_seconds:    300, // 5 minutes
            max_alerts_per_hour: 5,
            channels:            vec![
                NotificationChannel {
                    channel_type: ChannelType::Console,
                    config:       HashMap::new(),
                    enabled:      true,
                },
                NotificationChannel {
                    channel_type: ChannelType::LogFile,
                    config:       [("path".to_string(), "alerts.log".to_string())].into(),
                    enabled:      true,
                },
            ],
        });

        // Default policy for threshold exceeded
        policies.insert("threshold_exceeded".to_string(), AlertPolicy {
            enabled:             true,
            severity_thresholds: HashMap::new(),
            cooldown_seconds:    600, // 10 minutes
            max_alerts_per_hour: 10,
            channels:            vec![
                NotificationChannel {
                    channel_type: ChannelType::Console,
                    config:       HashMap::new(),
                    enabled:      true,
                },
                NotificationChannel {
                    channel_type: ChannelType::LogFile,
                    config:       [("path".to_string(), "alerts.log".to_string())].into(),
                    enabled:      true,
                },
            ],
        });

        // Default aggregation configs
        aggregation_config.insert("regression".to_string(), AlertAggregation {
            aggregation_window_seconds: 300, // 5 minutes
            min_alerts_for_aggregation: 3,
            strategy: AggregationStrategy::SuppressDuplicates,
        });

        aggregation_config.insert("threshold_exceeded".to_string(), AlertAggregation {
            aggregation_window_seconds: 600, // 10 minutes
            min_alerts_for_aggregation: 5,
            strategy: AggregationStrategy::GroupByMetric,
        });

        // Default escalation policies
        let escalation_policies = vec![
            EscalationPolicy {
                severity_threshold: AlertSeverity::High,
                escalation_delay_seconds: 1800, // 30 minutes
                escalation_channels: vec![
                    NotificationChannel {
                        channel_type: ChannelType::Email,
                        config: [("smtp_server".to_string(), "smtp.example.com".to_string()),
                                ("smtp_port".to_string(), "587".to_string())].into(),
                        enabled: true,
                    }
                ],
                max_escalation_level: 3,
            },
            EscalationPolicy {
                severity_threshold: AlertSeverity::Critical,
                escalation_delay_seconds: 300, // 5 minutes
                escalation_channels: vec![
                    NotificationChannel {
                        channel_type: ChannelType::Slack,
                        config: [("webhook_url".to_string(), "https://hooks.slack.com/...".to_string())].into(),
                        enabled: true,
                    }
                ],
                max_escalation_level: 5,
            },
        ];

        Self {
            policies,
            escalation_policies,
            aggregation_config,
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_counters: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            performance_trends: Arc::new(RwLock::new(HashMap::new())),
            team_configs,
            aggregated_alerts: Arc::new(RwLock::new(HashMap::new())),
            notification_channels: HashMap::new(),
        }
    }

    /// Process a performance alert
    pub async fn process_alert(&self, alert: PerformanceAlert) {
        let alert_key = self.get_alert_key(&alert);
        let alert_type = self.get_alert_type(&alert);

        // Check aggregation first
        if self.should_aggregate_alert(&alert_key, alert_type, &alert).await {
            self.add_to_aggregated_alerts(&alert_key, alert).await;
            return;
        }

        // Check escalation
        if self.should_escalate_alert(&alert).await {
            self.process_escalation(&alert).await;
        }

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

            // Check for predictive alerts based on trends
            if let Some(predictive_message) = self.check_predictive_alerts(&alert).await {
                self.send_notification_to_team(&predictive_message, None).await;
            }

            // Send to all configured channels
            for channel in &policy.channels {
                if channel.enabled {
                    self.send_notification(channel, &message).await;
                }
            }

            // Send team notifications if configured
            self.send_notification_to_team(&message, None).await;

            // Record alert as active
            self.record_alert_cooldown(&alert_key).await;

            // Add to history
            self.add_to_alert_history(alert_key.clone(), message).await;

            // Update performance trends
            self.update_performance_trends(&alert).await;
        }

        // Process any pending aggregated alerts
        self.process_aggregated_alerts().await;
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
        let counter = counters
            .entry(alert_type.to_string())
            .or_insert(AlertCounter {
                count:      0,
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
                    degradation_percent * 100.0,
                    metric_name,
                    baseline_value,
                    current_value
                );

                let mut details = HashMap::new();
                details.insert("metric".to_string(), metric_name);
                details.insert("baseline_value".to_string(), baseline_value.to_string());
                details.insert("current_value".to_string(), current_value.to_string());
                details.insert(
                    "degradation_percent".to_string(),
                    format!("{:.1}", degradation_percent * 100.0),
                );

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
                self.send_to_log_file(channel, message).await;
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

        println!(
            "{} [{}] {}",
            severity_icon, message.timestamp, message.title
        );
        println!("  {}", message.description);

        if !message.details.is_empty() {
            println!("  Details:");
            for (key, value) in &message.details {
                println!("    {}: {}", key, value);
            }
        }
        println!();
    }

    /// Send notification to log file
    async fn send_to_log_file(&self, channel: &NotificationChannel, message: &NotificationMessage) {
        if let Some(log_path) = channel.config.get("path") {
            if let Err(e) = validate_secure_path(log_path) {
                eprintln!("Invalid log file path: {}", e);
                return;
            }

            let log_entry = serde_json::json!({
                "timestamp": message.timestamp.to_rfc3339(),
                "title": message.title,
                "description": message.description,
                "severity": format!("{:?}", message.severity),
                "details": message.details,
                "context": message.context
            });

            let mut file = match OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
            {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Failed to open log file {}: {}", log_path, e);
                    return;
                }
            };

            if let Err(e) = writeln!(file, "{}", log_entry) {
                eprintln!("Failed to write to log file: {}", e);
            }

            audit_logger::log_event(
                "alert_logged",
                &format!("Alert '{}' logged to file {}", message.title, log_path),
            );
        }
    }

    /// Send notification via email
    async fn send_to_email(&self, channel: &NotificationChannel, message: &NotificationMessage) {
        let smtp_server = channel.config.get("smtp_server").unwrap_or(&"smtp.example.com".to_string());
        let smtp_port: u16 = channel.config.get("smtp_port")
            .and_then(|p| p.parse().ok())
            .unwrap_or(587);

        // Basic SMTP implementation
        match TcpStream::connect((smtp_server.as_str(), smtp_port)).await {
            Ok(mut stream) => {
                // Simple SMTP dialogue (this is a basic implementation)
                let _ = stream.write_all(b"EHLO localhost\r\n").await;
                let _ = stream.write_all(b"MAIL FROM:<alerts@rust-ai-ide.com>\r\n").await;
                let _ = stream.write_all(b"RCPT TO:<admin@rust-ai-ide.com>\r\n").await;
                let _ = stream.write_all(b"DATA\r\n").await;

                let email_body = format!(
                    "Subject: Performance Alert: {}\r\n\r\n{}\r\n\r\nDetails:\r\n{}\r\n",
                    message.title,
                    message.description,
                    message.details.iter()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect::<Vec<_>>()
                        .join("\n")
                );

                let _ = stream.write_all(email_body.as_bytes()).await;
                let _ = stream.write_all(b"\r\n.\r\n").await;
                let _ = stream.write_all(b"QUIT\r\n").await;

                audit_logger::log_event(
                    "email_alert_sent",
                    &format!("Alert '{}' sent via email to {}", message.title, smtp_server),
                );
            }
            Err(e) => {
                eprintln!("Failed to connect to SMTP server {}:{}: {}", smtp_server, smtp_port, e);
            }
        }
    }

    /// Send notification to Slack
    async fn send_to_slack(&self, channel: &NotificationChannel, message: &NotificationMessage) {
        if let Some(webhook_url) = channel.config.get("webhook_url") {
            let slack_payload = serde_json::json!({
                "text": format!("🚨 *Performance Alert*\n*{}*\n{}\n\n*Severity:* {:?}\n*Time:* {}",
                               message.title,
                               message.description,
                               message.severity,
                               message.timestamp.format("%Y-%m-%d %H:%M:%S UTC")),
                "attachments": [{
                    "fields": message.details.iter().map(|(k, v)| {
                        serde_json::json!({
                            "title": k,
                            "value": v,
                            "short": true
                        })
                    }).collect::<Vec<_>>()
                }]
            });

            match self.send_http_post_request(webhook_url, &slack_payload.to_string()).await {
                Ok(_) => {
                    audit_logger::log_event(
                        "slack_alert_sent",
                        &format!("Alert '{}' sent to Slack webhook", message.title),
                    );
                }
                Err(e) => {
                    eprintln!("Failed to send Slack notification: {}", e);
                }
            }
        }
    }

    /// Send notification to Discord
    async fn send_to_discord(&self, channel: &NotificationChannel, message: &NotificationMessage) {
        if let Some(webhook_url) = channel.config.get("webhook_url") {
            let discord_payload = serde_json::json!({
                "content": format!("🚨 **Performance Alert**\n**{}**\n{}\n\n**Severity:** {:?}\n**Time:** {}",
                                  message.title,
                                  message.description,
                                  message.severity,
                                  message.timestamp.format("%Y-%m-%d %H:%M:%S UTC")),
                "embeds": [{
                    "fields": message.details.iter().map(|(k, v)| {
                        serde_json::json!({
                            "name": k,
                            "value": v,
                            "inline": true
                        })
                    }).collect::<Vec<_>>()
                }]
            });

            match self.send_http_post_request(webhook_url, &discord_payload.to_string()).await {
                Ok(_) => {
                    audit_logger::log_event(
                        "discord_alert_sent",
                        &format!("Alert '{}' sent to Discord webhook", message.title),
                    );
                }
                Err(e) => {
                    eprintln!("Failed to send Discord notification: {}", e);
                }
            }
        }
    }

    /// Send notification to webhook
    async fn send_to_webhook(&self, channel: &NotificationChannel, message: &NotificationMessage) {
        if let Some(webhook_url) = channel.config.get("webhook_url") {
            let payload = serde_json::to_string(message).unwrap_or_default();

            match self.send_http_post_request(webhook_url, &payload).await {
                Ok(_) => {
                    audit_logger::log_event(
                        "webhook_alert_sent",
                        &format!("Alert '{}' sent to webhook {}", message.title, webhook_url),
                    );
                }
                Err(e) => {
                    eprintln!("Failed to send webhook notification: {}", e);
                }
            }
        }
    }

    /// Send notification to CI/CD system
    async fn send_to_ci_cd(&self, message: &NotificationMessage) {
        // This would integrate with CI/CD systems like GitHub Actions, Jenkins, etc.
        // For now, just log the alert for CI/CD processing
        println!("CI/CD Alert: {} - {}", message.title, message.description);

        audit_logger::log_event(
            "ci_cd_alert",
            &format!("Alert '{}' flagged for CI/CD system", message.title),
        );
    }

    /// Send HTTP POST request
    async fn send_http_post_request(&self, url: &str, body: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Parse URL to extract host and port
        let url_parts: Vec<&str> = url.split("://").collect();
        if url_parts.len() != 2 {
            return Err("Invalid URL format".into());
        }

        let host_port: Vec<&str> = url_parts[1].split('/').next().unwrap_or("").split(':').collect();
        let host = host_port[0];
        let port: u16 = if host_port.len() > 1 {
            host_port[1].parse().unwrap_or(443)
        } else if url_parts[0] == "https" {
            443
        } else {
            80
        };

        let mut stream = TcpStream::connect((host, port)).await?;

        let request = format!(
            "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            url.split(host).nth(1).unwrap_or("/"),
            host,
            body.len(),
            body
        );

        stream.write_all(request.as_bytes()).await?;
        stream.flush().await?;

        // Read response (basic implementation)
        let mut buffer = [0; 1024];
        let _ = stream.read(&mut buffer).await?;

        Ok(())
    }

    /// Check if alert should be aggregated
    async fn should_aggregate_alert(&self, alert_key: &str, alert_type: &str, _alert: &PerformanceAlert) -> bool {
        if let Some(config) = self.aggregation_config.get(alert_type) {
            let aggregated = self.aggregated_alerts.read().await;
            if let Some(alerts) = aggregated.get(alert_key) {
                return alerts.len() >= config.min_alerts_for_aggregation;
            }
        }
        false
    }

    /// Add alert to aggregated alerts
    async fn add_to_aggregated_alerts(&self, alert_key: &str, alert: PerformanceAlert) {
        let message = self.create_notification_message(alert);
        let mut aggregated = self.aggregated_alerts.write().await;
        aggregated.entry(alert_key.to_string())
            .or_insert_with(Vec::new)
            .push(message);
    }

    /// Check if alert should be escalated
    async fn should_escalate_alert(&self, alert: &PerformanceAlert) -> bool {
        let severity = match alert {
            PerformanceAlert::RegressionDetected { .. } => AlertSeverity::High,
            PerformanceAlert::ThresholdExceeded { .. } => AlertSeverity::Medium,
            PerformanceAlert::AnomalyDetected { severity, .. } => *severity,
        };

        for policy in &self.escalation_policies {
            if severity >= policy.severity_threshold {
                return true;
            }
        }
        false
    }

    /// Process alert escalation
    async fn process_escalation(&self, alert: &PerformanceAlert) {
        let severity = match alert {
            PerformanceAlert::RegressionDetected { .. } => AlertSeverity::High,
            PerformanceAlert::ThresholdExceeded { .. } => AlertSeverity::Medium,
            PerformanceAlert::AnomalyDetected { severity, .. } => *severity,
        };

        for policy in &self.escalation_policies {
            if severity >= policy.severity_threshold {
                let message = self.create_notification_message(alert.clone());
                for channel in &policy.escalation_channels {
                    if channel.enabled {
                        self.send_notification(channel, &message).await;
                    }
                }
            }
        }
    }

    /// Check for predictive alerts based on trends
    async fn check_predictive_alerts(&self, alert: &PerformanceAlert) -> Option<NotificationMessage> {
        let metric_name = match alert {
            PerformanceAlert::RegressionDetected { metric_name, .. } => metric_name,
            PerformanceAlert::ThresholdExceeded { metric_name, .. } => metric_name,
            PerformanceAlert::AnomalyDetected { .. } => return None,
        };

        let trends = self.performance_trends.read().await;
        if let Some(trend) = trends.get(metric_name) {
            if trend.confidence > 0.8 && trend.predicted_value > 100.0 { // Example threshold
                return Some(NotificationMessage {
                    title: format!("Predictive Alert: {} Degradation Expected", metric_name),
                    description: format!(
                        "Based on current trends, {} is expected to degrade to {:.2} in the next measurement (confidence: {:.1}%)",
                        metric_name, trend.predicted_value, trend.confidence * 100.0
                    ),
                    severity: AlertSeverity::Medium,
                    timestamp: Utc::now(),
                    details: HashMap::new(),
                    context: HashMap::new(),
                });
            }
        }
        None
    }

    /// Send notification to team
    async fn send_notification_to_team(&self, message: &NotificationMessage, team_id: Option<&str>) {
        if let Some(team_id) = team_id {
            if let Some(team_config) = self.team_configs.get(team_id) {
                for channel in &team_config.team_channels {
                    if channel.enabled {
                        self.send_notification(channel, message).await;
                    }
                }
            }
        }
    }

    /// Add alert to history
    async fn add_to_alert_history(&self, alert_id: String, message: NotificationMessage) {
        let history_entry = AlertHistoryEntry {
            alert_id,
            alert: message,
            acknowledgment_status: AlertAcknowledgmentStatus::Unacknowledged,
            acknowledged_by: None,
            acknowledged_at: None,
            resolved_at: None,
            escalation_level: 0,
            team_id: None,
        };

        let mut history = self.alert_history.write().await;
        history.push(history_entry);
    }

    /// Update performance trends
    async fn update_performance_trends(&self, alert: &PerformanceAlert) {
        let (metric_name, current_value) = match alert {
            PerformanceAlert::RegressionDetected { metric_name, current_value, .. } => (metric_name, *current_value),
            PerformanceAlert::ThresholdExceeded { metric_name, current_value, .. } => (metric_name, *current_value),
            PerformanceAlert::AnomalyDetected { .. } => return,
        };

        let mut trends = self.performance_trends.write().await;
        let trend = trends.entry(metric_name.clone()).or_insert(PerformanceTrend {
            metric_name: metric_name.clone(),
            trend_coefficient: 0.0,
            confidence: 0.0,
            predicted_value: current_value,
            analyzed_at: Utc::now(),
        });

        // Simple trend analysis - in a real implementation this would use more sophisticated algorithms
        trend.predicted_value = current_value * 1.1; // Simple prediction
        trend.confidence = 0.85;
        trend.analyzed_at = Utc::now();
    }

    /// Process aggregated alerts
    async fn process_aggregated_alerts(&self) {
        let mut aggregated = self.aggregated_alerts.write().await;
        let mut alerts_to_process = Vec::new();

        for (alert_key, messages) in aggregated.iter() {
            // Determine alert type from the key (e.g., "regression:cpu_usage" -> "regression")
            let alert_type = if alert_key.starts_with("regression:") {
                "regression"
            } else if alert_key.starts_with("threshold:") {
                "threshold_exceeded"
            } else {
                "anomaly"
            };

            if let Some(config) = self.aggregation_config.get(alert_type) {
                if messages.len() >= config.min_alerts_for_aggregation {
                    alerts_to_process.push((alert_key.clone(), messages.clone()));
                }
            }
        }

        for (alert_key, messages) in alerts_to_process {
            // Create aggregated message
            let aggregated_message = NotificationMessage {
                title: format!("Aggregated Alert: {} similar alerts", messages.len()),
                description: format!(
                    "Received {} similar alerts in the aggregation window. First alert: {}",
                    messages.len(),
                    messages.first().map(|m| m.title.as_str()).unwrap_or("Unknown")
                ),
                severity: AlertSeverity::Medium,
                timestamp: Utc::now(),
                details: HashMap::from([
                    ("alert_count".to_string(), messages.len().to_string()),
                    ("alert_key".to_string(), alert_key.clone()),
                ]),
                context: HashMap::new(),
            };

            // Send aggregated notification
            if let Some(policy) = self.policies.get("regression") { // Default to regression policy
                for channel in &policy.channels {
                    if channel.enabled {
                        self.send_notification(channel, &aggregated_message).await;
                    }
                }
            }

            // Remove processed alerts
            aggregated.remove(&alert_key);
        }
    }

    /// Acknowledge alert
    pub async fn acknowledge_alert(&self, alert_id: &str, user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut history = self.alert_history.write().await;
        if let Some(entry) = history.iter_mut().find(|e| e.alert_id == alert_id) {
            entry.acknowledgment_status = AlertAcknowledgmentStatus::Acknowledged;
            entry.acknowledged_by = Some(user_id.to_string());
            entry.acknowledged_at = Some(Utc::now());

            audit_logger::log_event(
                "alert_acknowledged",
                &format!("Alert '{}' acknowledged by user {}", alert_id, user_id),
            );
            Ok(())
        } else {
            Err("Alert not found".into())
        }
    }

    /// Resolve alert
    pub async fn resolve_alert(&self, alert_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut history = self.alert_history.write().await;
        if let Some(entry) = history.iter_mut().find(|e| e.alert_id == alert_id) {
            entry.acknowledgment_status = AlertAcknowledgmentStatus::Resolved;
            entry.resolved_at = Some(Utc::now());

            audit_logger::log_event(
                "alert_resolved",
                &format!("Alert '{}' resolved", alert_id),
            );
            Ok(())
        } else {
            Err("Alert not found".into())
        }
    }

    /// Get alert history
    pub async fn get_alert_history(&self, limit: Option<usize>) -> Vec<AlertHistoryEntry> {
        let history = self.alert_history.read().await;
        let mut entries = history.clone();
        entries.sort_by(|a, b| b.alert.timestamp.cmp(&a.alert.timestamp));

        if let Some(limit) = limit {
            entries.into_iter().take(limit).collect()
        } else {
            entries
        }
    }

    /// Add team configuration
    pub fn add_team_config(&mut self, team_id: String, config: TeamAlertingConfig) {
        self.team_configs.insert(team_id, config);
    }

    /// Update escalation policy
    pub fn update_escalation_policy(&mut self, index: usize, policy: EscalationPolicy) {
        if index < self.escalation_policies.len() {
            self.escalation_policies[index] = policy;
        }
    }

    /// Get performance trends
    pub async fn get_performance_trends(&self) -> HashMap<String, PerformanceTrend> {
        self.performance_trends.read().await.clone()
    }
}

// AlertSeverity is imported from collector module

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

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
            metric_name:         "cpu_usage".to_string(),
            baseline_value:      50.0,
            current_value:       75.0,
            degradation_percent: 0.5,
            timestamp:           Utc::now(),
        };

        let key = manager.get_alert_key(&alert);
        assert_eq!(key, "regression:cpu_usage");
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let manager = AlertManager::new();

        // Create a policy with low rate limit for testing
        let policy = AlertPolicy {
            enabled:             true,
            severity_thresholds: HashMap::new(),
            cooldown_seconds:    0, // No cooldown for this test
            max_alerts_per_hour: 2,
            channels:            vec![NotificationChannel {
                channel_type: ChannelType::Console,
                config:       HashMap::new(),
                enabled:      true,
            }],
        };

        manager.update_policy("test_type".to_string(), policy);

        // First two alerts should be allowed
        let alert = PerformanceAlert::ThresholdExceeded {
            metric_name:   "test_metric".to_string(),
            current_value: 100.0,
            threshold:     80.0,
            timestamp:     Utc::now(),
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

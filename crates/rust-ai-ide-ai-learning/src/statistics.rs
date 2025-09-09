//! Statistics and analytics for the learning system

use super::database::LearningDatabase;
use super::models::{LearnedPattern, PatternStatistics};
use super::types::AIResult;
use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Comprehensive statistics tracker for learning patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStatistics {
    pub overview: PatternStatistics,
    pub error_type_breakdown: HashMap<String, ErrorTypeStats>,
    pub temporal_distribution: TemporalStats,
    pub confidence_distribution: ConfidenceStats,
    pub effectiveness_metrics: EffectivenessMetrics,
    pub usage_patterns: UsagePatterns,
    pub performance_summary: PerformanceSummary,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTypeStats {
    pub error_code: String,
    pub pattern_count: u32,
    pub avg_confidence: f32,
    pub success_rate: f32,
    pub avg_attempts: f32,
    pub most_common_fixes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalStats {
    pub total_days_tracked: i64,
    pub patterns_per_day: f32,
    pub recent_activity: Vec<DailyActivity>,
    pub growth_rate: f32,
    pub maturity_score: f32, // 0-1 score based on historical data
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyActivity {
    pub date: DateTime<Utc>,
    pub new_patterns: u32,
    pub applications: u32,
    pub successes: u32,
    pub failures: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceStats {
    pub distribution: Vec<ConfidenceBucket>,
    pub average_confidence: f32,
    pub median_confidence: f32,
    pub confidence_trend: f32, // Recent trend (-1 to 1)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceBucket {
    pub range_start: f32,
    pub range_end: f32,
    pub count: u32,
    pub percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectivenessMetrics {
    pub overall_success_rate: f32,
    pub average_confidence_when_applied: f32,
    pub improvement_over_time: f32,
    pub user_satisfaction_estimate: f32, // Would be based on user feedback
    pub automation_potential: f32,       // Percentage of fixes that could be automated
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePatterns {
    pub most_used_patterns: Vec<PatternUsage>,
    pub category_distribution: HashMap<String, u32>,
    pub time_of_day_usage: Vec<HourlyUsage>,
    pub average_resolution_time: Option<f32>, // In seconds, if tracked
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternUsage {
    pub pattern_id: String,
    pub usage_count: u32,
    pub last_used: Option<DateTime<Utc>>,
    pub unique_users: Option<u32>, // If tracking multi-user scenarios
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyUsage {
    pub hour: u32, // 0-23
    pub usage_percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub avg_query_time_ms: f32,
    pub cache_hit_rate: f32,
    pub memory_usage_mb: Option<f32>,
    pub database_size_mb: Option<f32>,
    pub recommendations: Vec<String>,
}

impl LearningStatistics {
    /// Calculate comprehensive statistics from pattern data
    pub async fn calculate(database: &LearningDatabase) -> AIResult<Self> {
        let patterns = database.get_all_patterns().await?;
        let basic_stats = database.get_pattern_statistics().await?;

        let error_type_breakdown = Self::calculate_error_type_breakdown(&patterns);
        let temporal_distribution = Self::calculate_temporal_stats(&patterns)?;
        let confidence_distribution = Self::calculate_confidence_stats(&patterns);
        let effectiveness_metrics = Self::calculate_effectiveness_metrics(&patterns);
        let usage_patterns = Self::calculate_usage_patterns(&patterns);
        let performance_summary = Self::calculate_performance_metrics();

        Ok(LearningStatistics {
            overview: basic_stats,
            error_type_breakdown,
            temporal_distribution,
            confidence_distribution,
            effectiveness_metrics,
            usage_patterns,
            performance_summary,
            last_updated: Utc::now(),
        })
    }

    fn calculate_error_type_breakdown(
        patterns: &[LearnedPattern],
    ) -> HashMap<String, ErrorTypeStats> {
        let mut breakdown = HashMap::new();

        for pattern in patterns {
            let error_code = pattern
                .error_code
                .clone()
                .unwrap_or_else(|| "unknown".to_string());

            let stats = breakdown
                .entry(error_code.clone())
                .or_insert_with(|| ErrorTypeStats {
                    error_code: error_code.clone(),
                    pattern_count: 0,
                    avg_confidence: 0.0,
                    success_rate: 0.0,
                    avg_attempts: 0.0,
                    most_common_fixes: Vec::new(),
                });

            stats.pattern_count += 1;
            // Would need to accumulate and calculate averages
            // Implementation would track rolling averages
        }

        breakdown
    }

    fn calculate_temporal_stats(_patterns: &[LearnedPattern]) -> AIResult<TemporalStats> {
        // Calculate temporal distribution
        Ok(TemporalStats {
            total_days_tracked: 0,
            patterns_per_day: 0.0,
            recent_activity: Vec::new(),
            growth_rate: 0.0,
            maturity_score: 0.0,
        })
    }

    fn calculate_confidence_stats(patterns: &[LearnedPattern]) -> ConfidenceStats {
        let mut distribution = vec![
            ConfidenceBucket {
                range_start: 0.0,
                range_end: 0.2,
                count: 0,
                percentage: 0.0,
            },
            ConfidenceBucket {
                range_start: 0.2,
                range_end: 0.4,
                count: 0,
                percentage: 0.0,
            },
            ConfidenceBucket {
                range_start: 0.4,
                range_end: 0.6,
                count: 0,
                percentage: 0.0,
            },
            ConfidenceBucket {
                range_start: 0.6,
                range_end: 0.8,
                count: 0,
                percentage: 0.0,
            },
            ConfidenceBucket {
                range_start: 0.8,
                range_end: 1.0,
                count: 0,
                percentage: 0.0,
            },
        ];

        let mut confidences: Vec<f32> = patterns.iter().map(|p| p.confidence).collect();
        confidences.sort_by(|a, b| a.partial_cmp(b).unwrap());

        for pattern in patterns {
            let bucket = distribution
                .iter_mut()
                .find(|b| pattern.confidence >= b.range_start && pattern.confidence < b.range_end);
            if let Some(bucket) = bucket {
                bucket.count += 1;
            }
        }

        // Calculate percentages
        let total_patterns = patterns.len() as f32;
        for bucket in &mut distribution {
            bucket.percentage = if total_patterns > 0.0 {
                bucket.count as f32 / total_patterns * 100.0
            } else {
                0.0
            };
        }

        let average_confidence = if patterns.is_empty() {
            0.0
        } else {
            let total: f32 = patterns.iter().map(|p| p.confidence).sum();
            total / patterns.len() as f32
        };

        let median_confidence = if confidences.is_empty() {
            0.0
        } else {
            let mid = confidences.len() / 2;
            confidences[mid]
        };

        ConfidenceStats {
            distribution,
            average_confidence,
            median_confidence,
            confidence_trend: 0.0, // Would require historical data
        }
    }

    fn calculate_effectiveness_metrics(patterns: &[LearnedPattern]) -> EffectivenessMetrics {
        let total_attempts: u32 = patterns.iter().map(|p| p.attempt_count).sum();
        let total_successes: u32 = patterns.iter().map(|p| p.success_count).sum();

        let overall_success_rate = if total_attempts > 0 {
            total_successes as f32 / total_attempts as f32
        } else {
            0.0
        };

        let average_confidence_when_applied = if !patterns.is_empty() {
            let total: f32 = patterns
                .iter()
                .filter(|p| p.attempt_count > 0)
                .map(|p| p.confidence)
                .sum();
            let count: usize = patterns.iter().filter(|p| p.attempt_count > 0).count();
            if count > 0 {
                total / count as f32
            } else {
                0.0
            }
        } else {
            0.0
        };

        EffectivenessMetrics {
            overall_success_rate,
            average_confidence_when_applied,
            improvement_over_time: 0.0, // Would need time series data
            user_satisfaction_estimate: overall_success_rate * 0.8, // Rough estimate
            automation_potential: overall_success_rate * average_confidence_when_applied,
        }
    }

    fn calculate_usage_patterns(patterns: &[LearnedPattern]) -> UsagePatterns {
        let mut most_used_patterns = Vec::new();
        let mut category_distribution = HashMap::new();

        for pattern in patterns {
            for tag in &pattern.tags {
                *category_distribution.entry(tag.clone()).or_insert(0) += 1;
            }

            if pattern.attempt_count > 0 {
                most_used_patterns.push(PatternUsage {
                    pattern_id: pattern.id.clone(),
                    usage_count: pattern.attempt_count,
                    last_used: Some(pattern.updated_at),
                    unique_users: None,
                });
            }
        }

        most_used_patterns.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));

        UsagePatterns {
            most_used_patterns,
            category_distribution,
            time_of_day_usage: Vec::new(), // Would require usage timestamp data
            average_resolution_time: None,
        }
    }

    fn calculate_performance_metrics() -> PerformanceSummary {
        PerformanceSummary {
            avg_query_time_ms: 0.0, // Would need query timing data
            cache_hit_rate: 0.0,    // Would need cache statistics
            memory_usage_mb: None,
            database_size_mb: None,
            recommendations: Self::generate_recommendations(),
        }
    }

    fn generate_recommendations() -> Vec<String> {
        vec![
            "Consider increasing confidence threshold if success rate is low".to_string(),
            "Patterns with low success rates may need manual review".to_string(),
            "High-confidence patterns are good candidates for automation".to_string(),
        ]
    }
}

impl fmt::Display for LearningStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Learning System Statistics\n")?;
        write!(f, "========================\n")?;
        write!(f, "Total Patterns: {}\n", self.overview.total_patterns)?;
        write!(
            f,
            "Successful Patterns: {}\n",
            self.overview.successful_patterns
        )?;
        write!(f, "Recent Patterns: {}\n", self.overview.recent_patterns)?;
        write!(
            f,
            "Success Rate: {:.1}%\n",
            self.overview.success_rate * 100.0
        )?;
        write!(
            f,
            "Average Confidence: {:.2}\n",
            self.confidence_distribution.average_confidence
        )?;
        write!(
            f,
            "Overall Success Rate: {:.1}%\n",
            self.effectiveness_metrics.overall_success_rate * 100.0
        )?;
        Ok(())
    }
}

/// Utility functions for statistical analysis
pub mod analysis {
    use super::*;

    /// Analyze learning trends over time
    pub fn analyze_trends(patterns: &[LearnedPattern]) -> TrendAnalysis {
        let mut monthly_stats = HashMap::new();

        for pattern in patterns {
            let month_key = format!(
                "{}-{:02}",
                pattern.created_at.year(),
                pattern.created_at.month()
            );

            let stats = monthly_stats
                .entry(month_key.clone())
                .or_insert(MonthlyStats {
                    month: month_key.clone(),
                    new_patterns: 0,
                    successes: 0,
                    failures: 0,
                });

            stats.new_patterns += 1;
            stats.successes += pattern.success_count;
            stats.failures += pattern.attempt_count.saturating_sub(pattern.success_count);
        }

        let months: Vec<_> = monthly_stats.values().collect();

        TrendAnalysis {
            monthly_stats: months.into_iter().cloned().collect(),
            growth_trend: calculate_growth_trend(&monthly_stats),
            performance_trend: calculate_performance_trend(&monthly_stats),
        }
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct MonthlyStats {
        pub month: String,
        pub new_patterns: u32,
        pub successes: u32,
        pub failures: u32,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct TrendAnalysis {
        pub monthly_stats: Vec<MonthlyStats>,
        pub growth_trend: f32, // -1 to 1 (decreasing to increasing)
        pub performance_trend: f32,
    }

    fn calculate_growth_trend(_monthly_stats: &HashMap<String, MonthlyStats>) -> f32 {
        // Simple linear regression on pattern counts
        0.0 // Placeholder
    }

    fn calculate_performance_trend(_monthly_stats: &HashMap<String, MonthlyStats>) -> f32 {
        // Calculate trend in success rates
        0.0 // Placeholder
    }

    /// Generate insights from statistics
    pub fn generate_insights(stats: &LearningStatistics) -> Vec<String> {
        let mut insights = Vec::new();

        if stats.effectiveness_metrics.overall_success_rate > 0.8 {
            insights.push("Excellent pattern success rate!".to_string());
        } else if stats.effectiveness_metrics.overall_success_rate < 0.6 {
            insights.push("Consider reviewing failed patterns for improvements".to_string());
        }

        if stats.effectiveness_metrics.automation_potential > 0.7 {
            insights.push("High automation potential detected".to_string());
        }

        if stats.overview.total_patterns < 10 {
            insights.push("Consider building more patterns for better coverage".to_string());
        }

        insights
    }
}

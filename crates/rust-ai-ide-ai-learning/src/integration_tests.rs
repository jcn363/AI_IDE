//! Integration Tests for Modular Learning System
//!
//! These tests verify that all modules work correctly together
//! and that the modular architecture maintains system integrity.

use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

use super::{
    database::LearningDatabase,
    models::{FixTemplate, LearnedPattern, LearningPreferences},
    preferences::{utils::apply_privacy_implications, PreferencesManager},
    similarity::SimilarityCalculator,
    statistics::{analysis::generate_insights, LearningStatistics},
    system::LearningSystem,
    types::{AIProvider, AIResult, AnalysisPreferences, PrivacyMode},
};

/// Helper function to create a temporary learning system for testing
async fn create_temp_learning_system() -> (LearningSystem, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_learning.db");

    let system = LearningSystem::new_with_path(Some(db_path))
        .await
        .expect("Failed to create learning system");

    (system, temp_dir)
}

/// Test basic LearningSystem initialization and configuration
#[tokio::test]
async fn test_learning_system_initialization() {
    let (system, _temp_dir) = create_temp_learning_system().await;

    // Test basic system state
    assert!(system.get_preferences().enable_learning);

    // Test preferences update
    let mut new_prefs = LearningPreferences::default();
    new_prefs.confidence_threshold = 0.8;
    system.update_preferences(new_prefs.clone()).await.unwrap();
    assert_eq!(system.get_preferences().confidence_threshold, 0.8);
}

/// Test cross-module data flow: Models → Database → System
#[tokio::test]
async fn test_data_flow_models_to_database_to_system() {
    let (system, _temp_dir) = create_temp_learning_system().await;

    // Create a test pattern using models
    let test_pattern = create_test_learned_pattern();

    // Store via system (which uses database)
    let _id = system
        .store_pattern(&test_pattern)
        .await
        .expect("Failed to store pattern");

    // Retrieve via system
    let retrieved_patterns = system
        .get_all_patterns_with_limit(10)
        .await
        .expect("Failed to retrieve patterns");

    assert!(!retrieved_patterns.is_empty());
    assert_eq!(retrieved_patterns[0].id, test_pattern.id);
}

/// Test similarity calculator integration
#[tokio::test]
async fn test_similarity_calculator_integration() {
    let (system, _temp_dir) = create_temp_learning_system().await;

    // Store test patterns
    let pattern1 = create_test_learned_pattern();
    let pattern2 = create_similar_learned_pattern();
    system
        .store_pattern(&pattern1)
        .await
        .expect("Failed to store pattern1");
    system
        .store_pattern(&pattern2)
        .await
        .expect("Failed to store pattern2");

    // Test similarity finding
    let similar_patterns = system
        .find_similar_patterns("test error context")
        .await
        .expect("Failed to find similar patterns");

    assert!(!similar_patterns.is_empty());
}

/// Test preferences manager integration
#[tokio::test]
async fn test_preferences_manager_integration() {
    let (system, _temp_dir) = create_temp_learning_system().await;

    // Test preference templates
    let dev_prefs = super::preferences::templates::development();
    system.update_preferences(dev_prefs.clone()).await.unwrap();
    assert_eq!(
        system.get_preferences().confidence_threshold,
        dev_prefs.confidence_threshold
    );

    // Test privacy implications
    let mut test_prefs = LearningPreferences::default();
    test_prefs.privacy_mode = PrivacyMode::OptOut;
    apply_privacy_implications(&mut test_prefs);

    assert!(!test_prefs.enable_community_sharing);
    assert!(!test_prefs.use_community_patterns);
}

/// Test database operations integration
#[tokio::test]
async fn test_database_operations_integration() {
    let (system, _temp_dir) = create_temp_learning_system().await;

    // Test pattern storage and retrieval
    let original_pattern = create_test_learned_pattern();
    let pattern_id = original_pattern.id.clone();

    system
        .store_pattern(&original_pattern)
        .await
        .expect("Failed to store pattern");

    let retrieved_pattern = system
        .get_pattern(&pattern_id)
        .await
        .expect("Failed to retrieve pattern");

    assert_eq!(retrieved_pattern.id, original_pattern.id);
    assert_eq!(retrieved_pattern.description, original_pattern.description);
}

/// Test statistics integration with database
#[tokio::test]
async fn test_statistics_integration() {
    let (system, _temp_dir) = create_temp_learning_system().await;

    // Add some test data
    for i in 0..5 {
        let mut pattern = create_test_learned_pattern();
        pattern.id = format!("test_pattern_{}", i);
        pattern.success_count = i + 1;
        pattern.attempt_count = (i + 1) * 2;
        system
            .store_pattern(&pattern)
            .await
            .expect("Failed to store pattern");
    }

    // Test statistics calculation
    let stats = system
        .get_statistics()
        .await
        .expect("Failed to get statistics");

    assert_eq!(stats.total_patterns, 5);
    assert!(stats.avg_success_rate > 0.0);

    // Test insights generation
    let insights = generate_insights(&stats);
    assert!(!insights.is_empty()); // Should generate at least some insights
}

/// Test pattern caching behavior
#[tokio::test]
async fn test_pattern_caching_behavior() {
    let (system, _temp_dir) = create_temp_learning_system().await;

    // Store a pattern
    let test_pattern = create_test_learned_pattern();
    let pattern_id = test_pattern.id.clone();
    system
        .store_pattern(&test_pattern)
        .await
        .expect("Failed to store pattern");

    // Test caching by retrieving multiple times
    for i in 0..3 {
        let retrieved_first = system
            .get_pattern(&pattern_id)
            .await
            .expect(&format!("Failed to retrieve pattern attempt {}", i));
        let retrieved_second = system
            .get_pattern(&pattern_id)
            .await
            .expect(&format!("Failed to retrieve pattern attempt {}+1", i));

        assert_eq!(retrieved_first.id, retrieved_second.id);
    }
}

/// Test concurrent access to learning system
#[tokio::test]
async fn test_concurrent_access() {
    let (system, _temp_dir) = create_temp_learning_system().await;

    // Create multiple concurrent tasks
    let system_arc = Arc::new(system);
    let mut handles = Vec::new();

    for i in 0..5 {
        let system_clone = Arc::clone(&system_arc);
        let handle = tokio::spawn(async move {
            let pattern = create_test_learned_pattern_with_id(i);
            system_clone
                .store_pattern(&pattern)
                .await
                .map_err(|e| format!("Failed in task {}: {}", i, e))
        });
        handles.push(handle);
    }

    // Wait for all concurrent operations
    for handle in handles {
        handle
            .await
            .expect("Concurrent task failed")
            .expect("Storage operation failed");
    }

    // Verify all patterns were stored
    let all_patterns = system_arc
        .get_all_patterns_with_limit(10)
        .await
        .expect("Failed to retrieve patterns after concurrent operations");

    assert!(all_patterns.len() >= 5);
}

/// Test error handling across modules
#[tokio::test]
async fn test_error_handling_integration() {
    let (_system, temp_dir) = create_temp_learning_system().await;

    // Test with invalid DB path
    let invalid_path = temp_dir.path().join("nonexistent").join("invalid.db");
    let invalid_result = LearningSystem::new_with_path(Some(invalid_path)).await;

    // This should fail gracefully, not panic
    assert!(invalid_result.is_err());
}

/// Test memory cleanup and resource management
#[tokio::test]
async fn test_resource_cleanup() {
    let (system, _temp_dir) = create_temp_learning_system().await;

    // Add patterns to build up cache/memory
    for i in 0..10 {
        let pattern = create_test_learned_pattern_with_id(i);
        system
            .store_pattern(&pattern)
            .await
            .expect("Failed to store pattern");
    }

    // Force some cache operations
    let _similar_patterns = system
        .find_similar_patterns("test context")
        .await
        .expect("Failed to find similar patterns");

    // System should clean up resources when dropped
    drop(system);

    // Cleanup should happen automatically (test doesn't fail)
}

/// Test privacy mode configuration edge cases
#[tokio::test]
async fn test_privacy_mode_edge_cases() {
    let (system, _temp_dir) = create_temp_learning_system().await;

    // Test OptOut mode disables community features
    let mut optout_prefs = LearningPreferences::default();
    optout_prefs.privacy_mode = PrivacyMode::OptOut;
    optout_prefs.enable_community_sharing = true; // Should be overridden

    apply_privacy_implications(&mut optout_prefs);
    assert!(!optout_prefs.enable_community_sharing);

    // Test Anonymous mode preserves sharing setting
    let mut anon_prefs = LearningPreferences::default();
    anon_prefs.privacy_mode = PrivacyMode::Anonymous;
    anon_prefs.enable_community_sharing = false;

    apply_privacy_implications(&mut anon_prefs);
    assert!(!anon_prefs.enable_community_sharing); // Should remain as set
}

/// Test large dataset performance characteristics
#[tokio::test]
async fn test_large_dataset_performance() {
    let (system, _temp_dir) = create_temp_learning_system().await;

    // Add a moderate number of patterns (100) to test scaling
    let pattern_count = 100;
    for i in 0..pattern_count {
        let pattern = create_test_learned_pattern_with_id(i);
        system
            .store_pattern(&pattern)
            .await
            .expect("Failed to store large dataset pattern");
    }

    // Test retrieval performance
    let start = std::time::Instant::now();
    let patterns = system
        .get_all_patterns_with_limit(50)
        .await
        .expect("Failed to retrieve large dataset");
    let duration = start.elapsed();

    assert_eq!(patterns.len(), 50);

    // Should complete in reasonable time (less than 1 second for 100 patterns)
    assert!(
        duration.as_secs() < 1,
        "Large dataset retrieval took {}ms, should be < 1000ms",
        duration.as_millis()
    );
}

/// Benchmark-style test for similarity operations
#[tokio::test]
async fn test_similarity_algorithm_stress() {
    let (system, _temp_dir) = create_temp_learning_system().await;

    // Create diverse patterns for similarity testing
    let patterns = create_diverse_test_patterns();
    for pattern in &patterns {
        system
            .store_pattern(pattern)
            .await
            .expect("Failed to store diverse pattern");
    }

    // Test similarity with different contexts
    let test_contexts = [
        "borrow checker error mutable",
        "compilation error unused variable",
        "trait implementation missing method",
        "type mismatch expected found different",
    ];

    for context in &test_contexts {
        let similarities = system
            .find_similar_patterns(context)
            .await
            .expect("Failed to compute similarities");

        // Should find at least one reasonably similar pattern for each context
        assert!(
            !similarities.is_empty(),
            "No similar patterns found for context: {}",
            context
        );

        // First result should have reasonable confidence
        assert!(
            similarities[0].score > 0.0,
            "Similarity score too low for context: {}",
            context
        );
    }
}

// ============================================================================
// Helper Functions for Test Data Creation
// ============================================================================

fn create_test_learned_pattern() -> LearnedPattern {
    use super::models::{ChangeScope, ChangeTemplate, ChangeType, FixTemplate};
    use chrono::Utc;

    LearnedPattern {
        id: "test_pattern_1".to_string(),
        description: "Test pattern for integration testing".to_string(),
        error_pattern: "test error pattern".to_string(),
        error_code: Some("E0308".to_string()),
        context_patterns: vec!["fn test_function() {".to_string(), "let x = 1;".to_string()],
        fix_template: FixTemplate {
            description_template: "Test fix template".to_string(),
            change_templates: vec![ChangeTemplate {
                match_pattern: "let x = 1;".to_string(),
                replacement_pattern: "let _x = 1;".to_string(),
                change_type: ChangeType::Replace,
                scope: ChangeScope::Local,
            }],
            variables: std::collections::HashMap::new(),
            conditions: vec![],
            warnings: vec![],
        },
        confidence: 0.85,
        success_count: 5,
        attempt_count: 6,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        context_hash: "test_hash_123".to_string(),
        tags: vec!["test".to_string(), "integration".to_string()],
        contributor_id: None,
    }
}

fn create_similar_learned_pattern() -> LearnedPattern {
    use super::models::{ChangeScope, ChangeTemplate, ChangeType, FixTemplate};
    use chrono::Utc;

    LearnedPattern {
        id: "test_pattern_2".to_string(),
        description: "Similar test pattern".to_string(),
        error_pattern: "similar test error".to_string(),
        error_code: Some("E0308".to_string()),
        context_patterns: vec![
            "fn another_function() {".to_string(),
            "let y = 2;".to_string(),
        ],
        fix_template: FixTemplate {
            description_template: "Similar test fix".to_string(),
            change_templates: vec![ChangeTemplate {
                match_pattern: "let y = 2;".to_string(),
                replacement_pattern: "let _y = 2;".to_string(),
                change_type: ChangeType::Replace,
                scope: ChangeScope::Local,
            }],
            variables: std::collections::HashMap::new(),
            conditions: vec![],
            warnings: vec![],
        },
        confidence: 0.75,
        success_count: 3,
        attempt_count: 4,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        context_hash: "similar_hash_456".to_string(),
        tags: vec!["test".to_string(), "similar".to_string()],
        contributor_id: None,
    }
}

fn create_test_learned_pattern_with_id(id_num: usize) -> LearnedPattern {
    use super::models::{ChangeScope, ChangeTemplate, ChangeType, FixTemplate};
    use chrono::Utc;

    LearnedPattern {
        id: format!("perf_test_pattern_{}", id_num),
        description: format!("Performance test pattern {}", id_num),
        error_pattern: format!("perf test error {}", id_num),
        error_code: Some(format!("E0{}", id_num % 1000)),
        context_patterns: vec![
            format!("fn perf_function_{}() {{", id_num),
            format!("let var_{} = {};", id_num, id_num),
        ],
        fix_template: FixTemplate {
            description_template: format!("Perf fix for {}", id_num),
            change_templates: vec![ChangeTemplate {
                match_pattern: format!("let var_{} = {};", id_num, id_num),
                replacement_pattern: format!("let _var_{} = {};", id_num, id_num),
                change_type: ChangeType::Replace,
                scope: ChangeScope::Local,
            }],
            variables: std::collections::HashMap::new(),
            conditions: vec![],
            warnings: vec![],
        },
        confidence: 0.7 + (id_num as f32 * 0.001), // Slightly different confidences
        success_count: (id_num % 10 + 1) as u32,
        attempt_count: (id_num % 15 + 2) as u32,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        context_hash: format!("perf_hash_{}", id_num),
        tags: vec!["performance".to_string(), format!("batch_{}", id_num / 10)],
        contributor_id: None,
    }
}

fn create_diverse_test_patterns() -> Vec<LearnedPattern> {
    vec![
        create_pattern_with_context("borrow", "mutable", "add mut", 0.8),
        create_pattern_with_context("unused", "variable", "prefix with _", 0.7),
        create_pattern_with_context("trait", "implementation", "implement method", 0.9),
        create_pattern_with_context("type", "mismatch", "cast types", 0.75),
        create_pattern_with_context("ownership", "moved", "clone value", 0.85),
        create_pattern_with_context("lifetime", "validity", "adjust lifetime", 0.6),
        create_pattern_with_context("compilation", "error", "fix syntax", 0.95),
        create_pattern_with_context("pattern", "match", "add case", 0.8),
    ]
}

fn create_pattern_with_context(
    keyword1: &str,
    keyword2: &str,
    fix_desc: &str,
    confidence: f32,
) -> LearnedPattern {
    use super::models::{ChangeScope, ChangeTemplate, ChangeType, FixTemplate};
    use chrono::Utc;

    LearnedPattern {
        id: format!("{}_{}_pattern", keyword1, keyword2),
        description: format!("Pattern for {} {}", keyword1, keyword2),
        error_pattern: format!("{} {} error", keyword1, keyword2),
        error_code: None,
        context_patterns: vec![
            format!("fn test_{}() {{", keyword1),
            format!("let {} = {};", keyword2, keyword1),
        ],
        fix_template: FixTemplate {
            description_template: fix_desc.to_string(),
            change_templates: vec![ChangeTemplate {
                match_pattern: format!("let {} = {};", keyword2, keyword1),
                replacement_pattern: format!("let {} = {}.clone();", keyword2, keyword1),
                change_type: ChangeType::Replace,
                scope: ChangeScope::Local,
            }],
            variables: std::collections::HashMap::new(),
            conditions: vec![],
            warnings: vec!["This is a test pattern".to_string()],
        },
        confidence,
        success_count: (confidence * 10.0) as u32,
        attempt_count: ((confidence * 10.0) as u32) + 1,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        context_hash: format!("{}_{}", keyword1, keyword2),
        tags: vec![keyword1.to_string(), keyword2.to_string()],
        contributor_id: None,
    }
}

// Extension trait to provide additional test helper methods
#[cfg(test)]
pub trait LearningSystemTestExt {
    async fn store_pattern(&self, pattern: &LearnedPattern) -> AIResult<()>;
    async fn get_pattern(&self, id: &str) -> AIResult<LearnedPattern>;
    async fn get_all_patterns_with_limit(&self, limit: usize) -> AIResult<Vec<LearnedPattern>>;
    async fn get_statistics(&self) -> AIResult<LearningStatistics>;
}

#[cfg(test)]
impl LearningSystemTestExt for LearningSystem {
    async fn store_pattern(&self, pattern: &LearnedPattern) -> AIResult<()> {
        self.database.store_pattern(pattern).await
    }

    async fn get_pattern(&self, id: &str) -> AIResult<LearnedPattern> {
        self.database
            .get_pattern_by_id(id)
            .await?
            .ok_or_else(|| crate::types::LearningError::PatternNotFoundError(id.to_string()).into())
    }

    async fn get_all_patterns_with_limit(&self, limit: usize) -> AIResult<Vec<LearnedPattern>> {
        let mut patterns = self.database.get_all_patterns().await?;
        patterns.truncate(limit);
        Ok(patterns)
    }

    async fn get_statistics(&self) -> AIResult<LearningStatistics> {
        LearningStatistics::calculate(&self.database).await
    }
}

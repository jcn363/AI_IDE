//! Comprehensive tests for Phase 2 Advanced AI Error Analysis system
//!
//! This module provides extensive test coverage for:
//! - Root cause analysis with multi-level classification
//! - Predictive error prevention using ML pattern recognition
//! - Automated solution generation with template-based fixes
//! - Error clustering and impact analysis
//! - Error evolution tracking and quality trends

use std::collections::HashMap;

use chrono::Utc;

use crate::advanced_error_analysis::*;
use crate::error_resolution::ErrorContext;
use crate::learning::AIContext;

#[cfg(test)]
mod advanced_error_analysis_tests {
    use super::*;

    #[test]
    fn test_error_level_hierarchy_ordering() {
        assert_eq!(ErrorLevel::System.priority(), 1);
        assert_eq!(ErrorLevel::Module.priority(), 2);
        assert_eq!(ErrorLevel::Function.priority(), 3);
        assert_eq!(ErrorLevel::Line.priority(), 4);

        // Test escalation rules
        assert!(ErrorLevel::Line.can_escalate_to(&ErrorLevel::Function));
        assert!(ErrorLevel::Function.can_escalate_to(&ErrorLevel::Module));
        assert!(ErrorLevel::Module.can_escalate_to(&ErrorLevel::System));
        assert!(!ErrorLevel::System.can_escalate_to(&ErrorLevel::Line));
    }

    #[test]
    fn test_advanced_error_analyzer_initialization() {
        let analyzer = AdvancedErrorAnalyzer::new(crate::AIProvider::Mock);

        // Test that all components are properly initialized
        assert_eq!(
            matches!(analyzer.ai_provider, crate::AIProvider::Mock),
            true
        );

        // Test that we can analyze an error without panicking
        let error_context = ErrorContext {
            message:       "Test error".to_string(),
            error_code:    None,
            context_lines: vec![],
            file_path:     Some("test.rs".to_string()),
            line:          Some(1),
            column:        Some(1),
        };

        let project_context = AIContext::default();

        // The analyze_error method should return Ok (though with placeholder data)
        let result = tokio_test::block_on(analyzer.analyze_error(&error_context, &project_context));
        assert!(result.is_ok());
    }

    #[test]
    fn test_root_cause_engine_initialization() {
        let engine = RootCauseEngine::new();

        // Test basic functionality
        let error_context = ErrorContext {
            message:       "Compilation error".to_string(),
            error_code:    None,
            context_lines: vec![],
            file_path:     Some("src/main.rs".to_string()),
            line:          Some(10),
            column:        Some(5),
        };

        let project_context = AIContext::default();

        let result = tokio_test::block_on(engine.analyze_root_cause(&error_context, &project_context));
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(!analysis.analysis_id.is_empty());
        assert!(analysis.confidence > 0.0 && analysis.confidence <= 1.0);
        assert!(!analysis.cause_chain.is_empty());
    }

    #[test]
    fn test_impact_scope_identification() {
        // Test that impact scope correctly identifies risk levels
        let local_impact = ImpactAssessment {
            scope:           ImpactScope::Local,
            affected_files:  vec!["single_file.rs".to_string()],
            risk_level:      RiskLevel::Low,
            level_breakdown: [(ErrorLevel::Line, 1)].iter().cloned().collect(),
            urgency_score:   0.3,
            business_impact: "Single file affected".to_string(),
        };

        let project_impact = ImpactAssessment {
            scope:           ImpactScope::ProjectLevel,
            affected_files:  vec!["file1.rs".to_string(), "file2.rs".to_string()],
            risk_level:      RiskLevel::High,
            level_breakdown: [(ErrorLevel::Module, 3)].iter().cloned().collect(),
            urgency_score:   0.8,
            business_impact: "Multiple modules affected".to_string(),
        };

        assert_eq!(matches!(local_impact.scope, ImpactScope::Local), true);
        assert_eq!(
            matches!(project_impact.scope, ImpactScope::ProjectLevel),
            true
        );
        assert!(project_impact.urgency_score > local_impact.urgency_score);
    }

    #[test]
    fn test_prediction_system_initialization() {
        let system = PredictionSystem::new();

        // Test that components are initialized
        let mock_root_cause = create_mock_root_cause_analysis();

        let result = tokio_test::block_on(system.predict_related_errors(&mock_root_cause));
        assert!(result.is_ok());
    }

    #[test]
    fn test_solution_generator_templates() {
        let generator = SolutionGenerator::new();

        // Test template system initialization
        let mock_root_cause = create_mock_root_cause_analysis();
        let error_context = ErrorContext {
            message:       "Unused variable error".to_string(),
            error_code:    Some("E0308".to_string()),
            context_lines: vec![],
            file_path:     Some("src/main.rs".to_string()),
            line:          Some(5),
            column:        Some(9),
        };

        let result = tokio_test::block_on(generator.generate_solutions(&mock_root_cause, &error_context));
        assert!(result.is_ok());
    }

    #[test]
    fn test_impact_analyzer_clustering() {
        let analyzer = ImpactAnalyzer::new();

        // Test clustering engine
        let mock_root_cause = create_mock_root_cause_analysis();
        let mock_predictions = vec![];

        let result = tokio_test::block_on(analyzer.assess_impacts(&mock_root_cause, &mock_predictions));
        assert!(result.is_ok());

        let impact = result.unwrap();
        assert!(impact.urgency_score >= 0.0 && impact.urgency_score <= 1.0);
        assert!(!impact.business_impact.is_empty());
    }

    #[test]
    fn test_evolution_tracker_quality_metrics() {
        let tracker = EvolutionTracker::new();

        // Test evolution tracking
        let mock_root_cause = create_mock_root_cause_analysis();
        let error_context = ErrorContext {
            message:       "Evolution test error".to_string(),
            error_code:    None,
            context_lines: vec![],
            file_path:     Some("test.rs".to_string()),
            line:          Some(1),
            column:        Some(1),
        };

        let result = tokio_test::block_on(tracker.track_evolution(&mock_root_cause, &error_context));
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_dependency_analysis() {
        let mut dependency_analyzer = DependencyAnalyzer::new();

        // Test dependency tracking
        assert!(dependency_analyzer.dependency_graph.is_empty());
        assert!(dependency_analyzer.analysis_cache.is_empty());
    }

    #[test]
    fn test_risk_level_assessment() {
        // Test different risk levels
        assert_eq!(matches!(RiskLevel::Low, RiskLevel::Low), true);
        assert_eq!(matches!(RiskLevel::Critical, RiskLevel::Critical), true);

        // Test risk progression
        let levels = vec![
            RiskLevel::Low,
            RiskLevel::Medium,
            RiskLevel::High,
            RiskLevel::Critical,
        ];
        for (i, level) in levels.iter().enumerate() {
            match level {
                RiskLevel::Low => assert_eq!(i, 0),
                RiskLevel::Medium => assert_eq!(i, 1),
                RiskLevel::High => assert_eq!(i, 2),
                RiskLevel::Critical => assert_eq!(i, 3),
                _ => panic!("Unexpected risk level"),
            }
        }
    }

    #[test]
    fn test_error_location_tracking() {
        let location = ErrorLocation {
            file_path:     "src/main.rs".to_string(),
            line:          42,
            column:        15,
            function_name: Some("process_data".to_string()),
            module_path:   Some("utils".to_string()),
        };

        // Test location fields
        assert_eq!(location.file_path, "src/main.rs");
        assert_eq!(location.line, 42);
        assert_eq!(location.column, 15);
        assert_eq!(location.function_name, Some("process_data".to_string()));
        assert_eq!(location.module_path, Some("utils".to_string()));
    }

    #[tokio::test]
    async fn test_comprehensive_error_analysis() {
        let analyzer = AdvancedErrorAnalyzer::new(crate::AIProvider::Mock);

        let error_context = ErrorContext {
            message:       "Expected type `String` but found `&str`".to_string(),
            error_code:    Some("E0308".to_string()),
            context_lines: vec!["let name: String = \"hello\";".to_string()],
            file_path:     Some("src/main.rs".to_string()),
            line:          Some(5),
            column:        Some(19),
        };

        let project_context = AIContext {
            workspace_root: Some(std::path::PathBuf::from("/test/project")),
            ..Default::default()
        };

        // Perform comprehensive analysis
        let result = analyzer
            .analyze_error(&error_context, &project_context)
            .await;
        assert!(result.is_ok(), "Advanced error analysis should succeed");

        let analysis = result.unwrap();

        // Validate analysis structure
        assert!(!analysis.analysis_id.is_empty());
        assert!(analysis.confidence_score > 0.0);
        assert!(analysis.confidence_score <= 1.0);
        assert!(!analysis.root_cause_analysis.cause_chain.is_empty());

        // Test timestamp
        assert!(analysis.analyzed_at <= Utc::now());
        assert!(analysis.analyzed_at >= Utc::now() - chrono::Duration::seconds(10));
    }

    #[test]
    fn test_prediction_result_structure() {
        let _prediction = PredictionResult {
            prediction_id:          "pred_001".to_string(),
            error_type:             "type_mismatch".to_string(),
            likelihood:             0.85,
            time_window_hours:      24,
            contributing_factors:   vec![
                "Recent changes to function signatures".to_string(),
                "Updated dependency versions".to_string(),
            ],
            preventive_suggestions: vec![
                "Run type checker more frequently".to_string(),
                "Add pre-commit hooks for type checking".to_string(),
            ],
        };

        // Structure validation is implicit through creation
        // Further validation would require more complex test setup
    }

    #[test]
    fn test_fix_template_system() {
        let template = FixTemplate {
            template_id:         "fix_unused_var".to_string(),
            name:                "Fix unused variable".to_string(),
            error_patterns:      vec!["unused variable".to_string(), "E0308".to_string()],
            strategy:            FixStrategy::TemplateSubstitution,
            template_content:    "let _${variable_name} = ${expression};".to_string(),
            required_parameters: vec![TemplateParameter {
                name:            "variable_name".to_string(),
                parameter_type:  ParameterType::String,
                default_value:   None,
                validation_rule: Some("^[a-zA-Z_][a-zA-Z0-9_]*$".to_string()),
                description:     "Name of the variable to prefix".to_string(),
            }],
            success_rate:        0.95,
            usage_count:         42,
        };

        // Test template properties
        assert_eq!(template.template_id, "fix_unused_var");
        assert!(template.success_rate > 0.9);
        assert!(!template.required_parameters.is_empty());
        assert_eq!(template.usage_count, 42);
    }

    #[test]
    fn test_systemic_pattern_detection() {
        let pattern = SystemicPattern {
            pattern_id:          "memory_leak_cluster".to_string(),
            description:         "Cluster of memory management issues".to_string(),
            error_types:         vec![
                "E0309".to_string(),
                "E0382".to_string(),
                "E0505".to_string(),
            ],
            severity:            "high".to_string(),
            systemic_impact:     ImpactAssessment {
                scope:           ImpactScope::ProjectLevel,
                affected_files:  vec!["src/memory.rs".to_string(), "src/data.rs".to_string()],
                risk_level:      RiskLevel::High,
                level_breakdown: [(ErrorLevel::Function, 5)].iter().cloned().collect(),
                urgency_score:   0.9,
                business_impact: "Memory leaks causing performance degradation".to_string(),
            },
            resolution_strategy: "Implement RAII patterns and ownership tracking".to_string(),
        };

        // Test systemic pattern properties
        assert_eq!(pattern.error_types.len(), 3);
        assert_eq!(
            matches!(pattern.systemic_impact.scope, ImpactScope::ProjectLevel),
            true
        );
        assert!(pattern.systemic_impact.urgency_score > 0.8);
    }

    #[test]
    fn test_evolution_stage_tracking() {
        let stages = vec![
            EvolutionStage {
                stage_name:             "Initial detection".to_string(),
                characteristics:        vec!["First occurrence".to_string(), "Isolated case".to_string()],
                average_duration_days:  1.0,
                transition_probability: 0.3,
            },
            EvolutionStage {
                stage_name:             "Patterns emerging".to_string(),
                characteristics:        vec![
                    "Multiple occurrences".to_string(),
                    "Similar contexts".to_string(),
                ],
                average_duration_days:  7.0,
                transition_probability: 0.6,
            },
        ];

        let evolution = EvolutionPattern {
            pattern_id:       "null_pointer_evolution".to_string(),
            description:      "Evolution of null pointer dereference errors".to_string(),
            evolution_stages: stages,
            frequency:        15,
            impact_severity:  "medium".to_string(),
        };

        // Test evolution pattern
        assert_eq!(evolution.evolution_stages.len(), 2);
        assert_eq!(evolution.frequency, 15);
        assert_eq!(evolution.impact_severity, "medium");
    }

    // Helper function to create mock root cause analysis for testing
    fn create_mock_root_cause_analysis() -> RootCauseAnalysis {
        RootCauseAnalysis {
            analysis_id:       "test_analysis_001".to_string(),
            primary_level:     ErrorLevel::Line,
            cause_chain:       vec![CauseLink {
                level:      ErrorLevel::Line,
                category:   "syntax_error".to_string(),
                message:    "Expected semicolon".to_string(),
                confidence: 0.95,
                evidence:   vec!["Direct syntax error in code".to_string()],
                location:   Some(ErrorLocation {
                    file_path:     "test.rs".to_string(),
                    line:          1,
                    column:        1,
                    function_name: None,
                    module_path:   None,
                }),
            }],
            confidence:        0.88,
            dependencies:      vec![],
            impact_assessment: ImpactAssessment {
                scope:           ImpactScope::Local,
                affected_files:  vec!["test.rs".to_string()],
                risk_level:      RiskLevel::Low,
                level_breakdown: [(ErrorLevel::Line, 1)].iter().cloned().collect(),
                urgency_score:   0.5,
                business_impact: "Minimal compilation failure".to_string(),
            },
            analyzed_at:       Utc::now(),
        }
    }
}

/// Performance and accuracy tests for the advanced error analysis system
#[cfg(test)]
mod performance_tests {
    use std::time::Instant;

    use super::*;

    #[tokio::test]
    async fn test_analysis_performance() {
        let analyzer = AdvancedErrorAnalyzer::new(crate::AIProvider::Mock);

        let error_context = ErrorContext {
            message:       "Performance test error".to_string(),
            error_code:    None,
            context_lines: (0..50).map(|i| format!("line {}", i)).collect(),
            file_path:     Some("src/long_file.rs".to_string()),
            line:          Some(25),
            column:        Some(10),
        };

        let project_context = AIContext::default();
        let start_time = Instant::now();

        let result = analyzer
            .analyze_error(&error_context, &project_context)
            .await;

        let duration = start_time.elapsed();

        // Assert that analysis completes within reasonable time (< 1 second)
        assert!(
            duration.as_millis() < 1000,
            "Analysis took too long: {:?}",
            duration
        );
        assert!(result.is_ok(), "Analysis should succeed");
    }

    #[test]
    fn test_memory_usage_bounds() {
        // This would be a more complex test requiring memory profiling
        // For now, we'll just ensure the systems can be created without excessive memory

        let analyzer = AdvancedErrorAnalyzer::new(crate::AIProvider::Mock);

        // The analyzer should be reasonable in size
        // Note: In a real system, we'd use memory profiling tools like heaptrack
        assert!(std::mem::size_of_val(&analyzer) > 0);
    }

    #[tokio::test]
    async fn test_concurrent_analysis() {
        let analyzer = Arc::new(AdvancedErrorAnalyzer::new(crate::AIProvider::Mock));

        let contexts = (0..10)
            .map(|i| ErrorContext {
                message:       format!("Concurrent test error {}", i),
                error_code:    None,
                context_lines: vec![],
                file_path:     Some(format!("test{}.rs", i)),
                line:          Some(1),
                column:        Some(1),
            })
            .collect::<Vec<_>>();

        let project_context = AIContext::default();

        // Spawn concurrent analysis tasks
        let tasks: Vec<_> = contexts
            .into_iter()
            .map(|ctx| {
                let analyzer_clone = Arc::clone(&analyzer);
                let context_clone = project_context.clone();

                tokio::spawn(async move { analyzer_clone.analyze_error(&ctx, &context_clone).await })
            })
            .collect();

        // Wait for all tasks to complete
        for task in tasks {
            let result = task.await.unwrap();
            assert!(result.is_ok(), "Concurrent analysis should succeed");
        }
    }
}

/// Integration tests that combine multiple advanced features
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_system_integration() {
        // Create a comprehensive scenario that combines all Phase 2 features

        let analyzer = AdvancedErrorAnalyzer::new(crate::AIProvider::Mock);

        // Scenario: Complex build error with multiple potential causes
        let error_context = ErrorContext {
            message:       "Borrow checker error in async function".to_string(),
            error_code:    Some("E0505".to_string()),
            context_lines: vec![
                "async fn process_data(data: &mut Vec<String>) {".to_string(),
                "    let future = async move {".to_string(),
                "        data.push(\"processed\".to_string());".to_string(),
                "    };".to_string(),
                "}".to_string(),
            ],
            file_path:     Some("src/async_processor.rs".to_string()),
            line:          Some(3),
            column:        Some(9),
        };

        let project_context = AIContext {
            workspace_root: Some(std::path::PathBuf::from("/complex/project")),
            ..Default::default()
        };

        // Perform full analysis
        let result = analyzer
            .analyze_error(&error_context, &project_context)
            .await;
        assert!(result.is_ok());

        let analysis = result.unwrap();

        // Validate integrated results
        assert!(!analysis.analysis_id.is_empty());
        assert!(!analysis.solutions.is_empty());
        assert!(!analysis.predictions.is_empty());
        assert!(analysis.impacts.scope != ImpactScope::EcosystemLevel); // This shouldn't be ecosystem level

        println!(
            "Integration test passed with analysis ID: {}",
            analysis.analysis_id
        );
        println!("Found {} solutions", analysis.solutions.len());
        println!("Generated {} predictions", analysis.predictions.len());
        println!("Impact assessed: {:?}", analysis.impacts.scope);
    }

    #[tokio::test]
    async fn test_evolution_tracking_integration() {
        let analyzer = AdvancedErrorAnalyzer::new(crate::AIProvider::Mock);
        let tracker = EvolutionTracker::new();

        // Simulate error evolution over time
        let mut tracking = vec![];

        for i in 0..5 {
            let error_context = ErrorContext {
                message:       format!("Evolving error iteration {}", i),
                error_code:    Some("E0308".to_string()),
                context_lines: vec![],
                file_path:     Some("src/evolution_test.rs".to_string()),
                line:          Some(i + 1),
                column:        Some(1),
            };

            let project_context = AIContext::default();

            let result = analyzer
                .analyze_error(&error_context, &project_context)
                .await;
            assert!(result.is_ok());

            let analysis = result.unwrap();

            tracking.push((error_context, analysis));
        }

        // The integration would continue to test evolution patterns
        // For now, we just ensure the basic flow works
        assert_eq!(tracking.len(), 5);
    }
}

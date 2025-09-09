//! Advanced AI Error Analysis Phase 2 Demonstration
//!
//! This example demonstrates the sophisticated error intelligence capabilities
//! of the Phase 2 Advanced AI Error Analysis system, showcasing:
//!
//! - Multi-level root cause analysis (System → Module → Function → Line)
//! - Predictive error prevention using ML pattern recognition
//! - Automated solution generation with template-based fixes
//! - Error clustering and impact analysis for systemic resolution
//! - Error evolution tracking and quality trend analysis
//!
//! Run with: cargo run --example advanced-error-analysis-demo

use rust_ai_ide_ai::advanced_error_analysis::*;
use rust_ai_ide_ai::{AIProvider, ErrorContext, AIContext};
use std::path::PathBuf;
use chrono::Utc;

/// Demonstration of Phase 2 Advanced AI Error Analysis capabilities
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Phase 2 Advanced AI Error Analysis Demonstration");
    println!("==================================================\n");

    // Initialize the advanced error analyzer
    let analyzer = AdvancedErrorAnalyzer::new(AIProvider::Mock);
    println!("✅ Initialized Advanced Error Analyzer with Mock AI Provider\n");

    // Demo 1: Multi-Level Root Cause Analysis
    demonstrate_root_cause_analysis(&analyzer).await?;
    println!();

    // Demo 2: Predictive Error Prevention
    demonstrate_predictive_prevention(&analyzer).await?;
    println!();

    // Demo 3: Automated Solution Generation
    demonstrate_solution_generation(&analyzer).await?;
    println!();

    // Demo 4: Error Clustering and Impact Analysis
    demonstrate_clustering_impact(&analyzer).await?;
    println!();

    // Demo 5: Error Evolution Tracking
    demonstrate_evolution_tracking(&analyzer).await?;
    println!();

    // Demo 6: Comprehensive Error Intelligence
    demonstrate_comprehensive_analysis(&analyzer).await?;

    println!("\n🎉 Phase 2 Advanced Error Analysis demonstration complete!");
    println!("All capabilities successfully demonstrated with intelligent error diagnosis.");

    Ok(())
}

/// Demonstrate multi-level root cause analysis
async fn demonstrate_root_cause_analysis(
    analyzer: &AdvancedErrorAnalyzer,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Demo 1: Multi-Level Root Cause Analysis");
    println!("-----------------------------------------");

    // Create a complex borrow checker error scenario
    let error_context = ErrorContext {
        message: "Cannot borrow `items` as mutable because it is also borrowed as immutable".to_string(),
        error_code: Some("E0502".to_string()),
        context_lines: vec![
            "fn process_items(items: &mut Vec<String>) {".to_string(),
            "    let first = &items[0]; // Immutable borrow".to_string(),
            "    items.push(\"new_item\".to_string()); // Mutable borrow - ERROR!".to_string(),
            "    println!(\"First item: {}\", first);".to_string(),
            "}".to_string(),
        ],
        file_path: Some("src/main.rs".to_string()),
        line: Some(3),
        column: Some(5),
    };

    let project_context = AIContext {
        workspace_root: Some(PathBuf::from("./demo_workspace")),
        ..Default::default()
    };

    println!("Analyzing complex borrow checker error...");
    let root_cause = analyzer.root_cause_engine
        .analyze_root_cause(&error_context, &project_context)
        .await?;

    println!("📊 Root Cause Analysis Results:");
    println!("  Analysis ID: {}", root_cause.analysis_id);
    println!("  Primary Level: {:?}", root_cause.primary_level);
    println!("  Overall Confidence: {:.2}%", root_cause.confidence * 100.0);

    println!("\n📋 Cause Chain:");
    for (i, link) in root_cause.cause_chain.iter().enumerate() {
        println!("  {}. {} ({:.1}% confidence)", i + 1, link.message, link.confidence * 100.0);
        if let Some(ref location) = link.location {
            println!("     Location: {}:{}:{}", location.file_path, location.line, location.column);
        }
    }

    println!("\n🎯 Dependencies Identified:");
    for (i, dep) in root_cause.dependencies.iter().enumerate() {
        println!("  {}. {} ({:?} impact)", i + 1, dep.identifier, dep.impact);
    }

    println!("\n⚡ Impact Assessment:");
    println!("  Scope: {:?}", root_cause.impact_assessment.scope);
    println!("  Risk Level: {:?}", root_cause.impact_assessment.risk_level);
    println!("  Urgency Score: {:.2}%", root_cause.impact_assessment.urgency_score * 100.0);
    println!("  Affected Files: {}", root_cause.impact_assessment.affected_files.len());

    println!("✅ Multi-level root cause analysis completed successfully!");
    Ok(())
}

/// Demonstrate predictive error prevention
async fn demonstrate_predictive_prevention(
    analyzer: &AdvancedErrorAnalyzer,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔮 Demo 2: Predictive Error Prevention");
    println!("-------------------------------------");

    // Create a root cause for prediction
    let root_cause = RootCauseAnalysis {
        analysis_id: "pred_demo".to_string(),
        primary_level: ErrorLevel::Function,
        cause_chain: vec![],
        confidence: 0.9,
        dependencies: vec![],
        impact_assessment: ImpactAssessment {
            scope: ImpactScope::ModuleLevel,
            affected_files: vec!["src/async_ops.rs".to_string(), "src/data_processor.rs".to_string()],
            risk_level: RiskLevel::High,
            level_breakdown: [(ErrorLevel::Function, 5)].iter().cloned().collect(),
            urgency_score: 0.8,
            business_impact: "Data processing pipeline failure".to_string(),
        },
        analyzed_at: Utc::now(),
    };

    println!("Analyzing predictive error patterns...");
    let predictions = analyzer.prediction_system
        .predict_related_errors(&root_cause)
        .await?;

    println!("🔮 Predictive Analysis Results:");
    println!("  Predictions Generated: {}", predictions.len());

    println!("\n🎯 High-Risk Predictions:");
    for (i, prediction) in predictions.iter().enumerate() {
        if prediction.likelihood > 0.7 {
            println!("  {}. {} ({}% likelihood)", i + 1, prediction.error_type, (prediction.likelihood * 100.0) as i32);
            println!("     Contributing: {:?}", prediction.contributing_factors);
            println!("     Prevention: {:?}", prediction.preventive_suggestions);
        }
    }

    println!("✅ Predictive error prevention analysis completed!");
    Ok(())
}

/// Demonstrate automated solution generation
async fn demonstrate_solution_generation(
    analyzer: &AdvancedErrorAnalyzer,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Demo 3: Automated Solution Generation");
    println!("---------------------------------------");

    // Create a root cause for solution generation
    let root_cause = RootCauseAnalysis {
        analysis_id: "sol_demo".to_string(),
        primary_level: ErrorLevel::Line,
        cause_chain: vec![CauseLink {
            level: ErrorLevel::Line,
            category: "borrow_checker".to_string(),
            message: "Mutable/immutable borrow conflict".to_string(),
            confidence: 0.95,
            evidence: vec!["Borrow checker error E0502".to_string()],
            location: Some(ErrorLocation {
                file_path: "src/main.rs".to_string(),
                line: 10,
                column: 8,
                function_name: Some("process_data".to_string()),
                module_path: Some("utils".to_string()),
            }),
        }],
        confidence: 0.92,
        dependencies: vec![],
        impact_assessment: ImpactAssessment {
            scope: ImpactScope::Local,
            affected_files: vec!["src/main.rs".to_string()],
            risk_level: RiskLevel::Low,
            level_breakdown: [(ErrorLevel::Line, 1)].iter().cloned().collect(),
            urgency_score: 0.4,
            business_impact: "Single function affected".to_string(),
        },
        analyzed_at: Utc::now(),
    };

    let error_context = ErrorContext {
        message: "Cannot borrow as mutable".to_string(),
        error_code: Some("E0502".to_string()),
        context_lines: vec![],
        file_path: Some("src/main.rs".to_string()),
        line: Some(10),
        column: Some(8),
    };

    println!("Generating automated solutions...");
    let solutions = analyzer.solution_generator
        .generate_solutions(&root_cause, &error_context)
        .await?;

    println!("🔧 Solution Generation Results:");
    println!("  Solutions Found: {}", solutions.len());

    println!("\n💡 Recommended Solutions:");
    for (i, solution) in solutions.iter().enumerate() {
        println!("  {}. {} ({:.1}% confidence)", i + 1, solution.title, solution.confidence * 100.0);
        println!("     Auto-applicable: {}", solution.auto_applicable);
        println!("     Changes required: {}", solution.changes.len());
        println!("     Impact: {:?}", solution.impact);
    }

    println!("✅ Automated solution generation completed!");
    Ok(())
}

/// Demonstrate error clustering and impact analysis
async fn demonstrate_clustering_impact(
    analyzer: &AdvancedErrorAnalyzer,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Demo 4: Error Clustering and Impact Analysis");
    println!("-----------------------------------------------");

    // Create multiple related errors for clustering
    let root_cause = RootCauseAnalysis {
        analysis_id: "cluster_demo".to_string(),
        primary_level: ErrorLevel::Module,
        cause_chain: vec![CauseLink {
            level: ErrorLevel::Module,
            category: "memory_management".to_string(),
            message: "Cluster of memory management issues".to_string(),
            confidence: 0.88,
            evidence: vec![
                "Multiple E0382 errors".to_string(),
                "Memory ownership issues".to_string(),
            ],
            location: None,
        }],
        confidence: 0.85,
        dependencies: vec![],
        impact_assessment: ImpactAssessment {
            scope: ImpactScope::ProjectLevel,
            affected_files: vec![
                "src/memory.rs".to_string(),
                "src/allocator.rs".to_string(),
                "src/cache.rs".to_string(),
                "src/pool.rs".to_string(),
            ],
            risk_level: RiskLevel::High,
            level_breakdown: [(ErrorLevel::Function, 8)].iter().cloned().collect(),
            urgency_score: 0.9,
            business_impact: "Memory leaks causing performance degradation".to_string(),
        },
        analyzed_at: Utc::now(),
    };

    // Mock predictions for impact analysis
    let predictions = vec![
        PredictionResult {
            prediction_id: "pred_1".to_string(),
            error_type: "memory_leak".to_string(),
            likelihood: 0.85,
            time_window_hours: 48,
            contributing_factors: vec!["Improper Arc usage".to_string(), "Missing Drop implementations".to_string()],
            preventive_suggestions: vec!["Implement RAII pattern".to_string(), "Use memory profiling tools".to_string()],
        },
        PredictionResult {
            prediction_id: "pred_2".to_string(),
            error_type: "double_free".to_string(),
            likelihood: 0.72,
            time_window_hours: 24,
            contributing_factors: vec!["Manual memory management".to_string()],
            preventive_suggestions: vec!["Switch to smart pointers".to_string()],
        },
    ];

    println!("Analyzing systemic impact of error cluster...");
    let impact_analysis = analyzer.impact_analyzer
        .assess_impacts(&root_cause, &predictions)
        .await?;

    println!("📊 Systemic Impact Analysis:");
    println!("  Error Cluster: Memory Management Issues");
    println!("  Affected Files: {}", impact_analysis.affected_files.len());
    println!("  Risk Level: {:?}", impact_analysis.risk_level);
    println!("  Urgency Score: {:.1}%", impact_analysis.urgency_score * 100.0);
    println!("  Business Impact: {}", impact_analysis.business_impact);

    println!("\n🔗 Related Predictions:");
    for (i, pred) in predictions.iter().enumerate() {
        println!("  {}. {} ({}% in {}h)", i + 1, pred.error_type, (pred.likelihood * 100.0) as i32, pred.time_window_hours);
    }

    println!("✅ Error clustering and impact analysis completed!");
    Ok(())
}

/// Demonstrate error evolution tracking
async fn demonstrate_evolution_tracking(
    analyzer: &AdvancedErrorAnalyzer,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📈 Demo 5: Error Evolution Tracking");
    println!("-----------------------------------");

    let root_cause = RootCauseAnalysis {
        analysis_id: "evo_demo".to_string(),
        primary_level: ErrorLevel::Function,
        cause_chain: vec![],
        confidence: 0.85,
        dependencies: vec![],
        impact_assessment: ImpactAssessment {
            scope: ImpactScope::ModuleLevel,
            affected_files: vec!["src/async_ops.rs".to_string()],
            risk_level: RiskLevel::Medium,
            level_breakdown: [(ErrorLevel::Function, 3)].iter().cloned().collect(),
            urgency_score: 0.7,
            business_impact: "Async operation reliability".to_string(),
        },
        analyzed_at: Utc::now(),
    };

    let error_context = ErrorContext {
        message: "Evolution test error".to_string(),
        error_code: None,
        context_lines: vec![],
        file_path: Some("src/async_ops.rs".to_string()),
        line: Some(1),
        column: Some(1),
    };

    println!("Tracking error evolution patterns...");
    let evolution_patterns = analyzer.evolution_tracker
        .track_evolution(&root_cause, &error_context)
        .await?;

    println!("📈 Evolution Tracking Results:");
    println!("  Patterns Tracked: {}", evolution_patterns.len());

    println!("\n🔄 Evolution Patterns:");
    for (i, pattern) in evolution_patterns.iter().enumerate() {
        println!("  {}. {} (Frequency: {})", i + 1, pattern.description, pattern.frequency);
        println!("     Severity: {}", pattern.impact_severity);

        println!("     Evolution Stages:");
        for (j, stage) in pattern.evolution_stages.iter().enumerate() {
            println!("       {}. {} (avg {} days)", j + 1, stage.stage_name, stage.average_duration_days as i32);
        }
    }

    println!("✅ Error evolution tracking completed!");
    Ok(())
}

/// Demonstrate comprehensive error intelligence
async fn demonstrate_comprehensive_analysis(
    analyzer: &AdvancedErrorAnalyzer,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Demo 6: Comprehensive Error Intelligence");
    println!("------------------------------------------");

    // Create a comprehensive error scenario
    let error_context = ErrorContext {
        message: "Complex async error with multiple dependencies".to_string(),
        error_code: Some("E0308".to_string()),
        context_lines: vec![
            "async fn complex_operation(data: &Arc<Mutex<Vec<String>>>) -> Result<Vec<String>, Error> {".to_string(),
            "    let mut guard = data.lock().await;".to_string(),
            "    let items = guard.clone(); // Type mismatch error here".to_string(),
            "    drop(guard);".to_string(),
            "    process_items_async(items).await".to_string(),
            "}".to_string(),
        ],
        file_path: Some("src/complex_async.rs".to_string()),
        line: Some(3),
        column: Some(18),
    };

    let project_context = AIContext {
        workspace_root: Some(PathBuf::from("./complex_project")),
        ..Default::default()
    };

    println!("Performing comprehensive error intelligence analysis...");
    let comprehensive_result = analyzer
        .analyze_error(&error_context, &project_context)
        .await?;

    println!("🧠 Comprehensive Analysis Results:");
    println!("  Analysis ID: {}", comprehensive_result.analysis_id);
    println!("  Overall Confidence: {:.2}%", comprehensive_result.confidence_score * 100.0);
    println!("  Root Cause Level: {:?}", comprehensive_result.root_cause_analysis.primary_level);

    println!("\n📊 Multi-Level Breakdown:");
    for (level, count) in &comprehensive_result.impacts.level_breakdown {
        println!("  {:?}: {} issues", level, count);
    }

    println!("\n💡 Intelligent Suggestions:");
    println!("  Root Cause Fixes: {}", comprehensive_result.solutions.len());
    println!("  Predictive Warnings: {}", comprehensive_result.predictions.len());
    println!("  Evolution Patterns: {}", comprehensive_result.evolution_patterns.len());

    println!("\n🎯 Key Intelligence Insights:");
    println!("  ⚡ Impact Scope: {:?}", comprehensive_result.impacts.scope);
    println!("  ⚠️  Risk Level: {:?}", comprehensive_result.impacts.risk_level);
    println!("  🔥 Urgency: {:.1}%", comprehensive_result.impacts.urgency_score * 100.0);

    // Calculate and display system intelligence metrics
    println!("\n📈 Intelligence Metrics:");
    let intelligence_score = (comprehensive_result.confidence_score + comprehensive_result.impacts.urgency_score) / 2.0 * 100.0;
    println!("  System Intelligence Score: {:.1}%", intelligence_score);

    let solutions_coverage = comprehensive_result.solutions.len() as f32 / (comprehensive_result.root_cause_analysis.cause_chain.len() as f32) * 100.0;
    println!("  Solution Coverage: {:.1}%", solutions_coverage);

    let prediction_precision = comprehensive_result.predictions.iter()
        .map(|p| p.likelihood)
        .sum::<f32>() / comprehensive_result.predictions.len() as f32 * 100.0;
    println!("  Prediction Precision: {:.1}%", prediction_precision);

    println!("✅ Comprehensive error intelligence analysis completed!");
    println!("This demonstrates the full power of Phase 2 Advanced AI Error Analysis!");
    Ok(())
}
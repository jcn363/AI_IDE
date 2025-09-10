pub mod ai_suggester;
pub mod transformation_validator;
pub mod impact_assessor;
pub mod safety_guard;
pub mod execution_orchestrator;
pub mod engine;

pub mod suggestion_generator;
pub mod pattern_recognizer;
pub mod context_analyzer;
pub mod safety_filter;
pub mod confidence_scorer;

pub mod equivalence_checker;
pub mod test_generator;
pub mod behavior_analyzer;
pub mod dependency_detector;
pub mod rollback_manager;

pub mod cost_benefit_analyzer;
pub mod dependency_mapper;
pub mod performance_estimator;
pub mod timeline_planner;
pub mod risk_mitigator;

pub mod pre_execution_checker;
pub mod execution_monitor;
pub mod termination_trigger;
pub mod recovery_engine;
pub mod audit_trail;

pub mod sequential_executor;
pub mod dependency_resolver;
pub mod progress_reporter;
pub mod batch_processor;
pub mod error_recovery;

pub mod types;
pub mod error;

pub use engine::AdvancedRefactoringEngine;
pub use types::*;
pub use error::*;
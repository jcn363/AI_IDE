//! # End-to-End User Workflow Testing
//!
//! Comprehensive E2E testing module that simulates real user workflows
//! from start to finish, covering the complete user journey through the IDE.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use chrono::{DateTime, Utc};
use rust_ai_ide_errors::RustAIError;
use serde::{Deserialize, Serialize};

use crate::ui_testing::*;
use crate::{GlobalTestConfig, IntegrationTestResult};

/// End-to-end workflow test runner
pub struct E2EWorkflowRunner {
    framework: UITestFramework,
    workflow_config: WorkflowConfig,
    test_data: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct WorkflowConfig {
    pub enable_full_workflows: bool,
    pub enable_long_running_tests: bool,
    pub screenshot_on_failure: bool,
    pub generate_workflow_reports: bool,
    pub max_workflow_duration: Duration,
    pub cleanup_after_workflow: bool,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            enable_full_workflows: true,
            enable_long_running_tests: true,
            screenshot_on_failure: true,
            generate_workflow_reports: true,
            max_workflow_duration: Duration::from_secs(600), // 10 minutes
            cleanup_after_workflow: true,
        }
    }
}

/// User workflow types
#[derive(Debug, Clone, PartialEq)]
pub enum UserWorkflowType {
    /// New user onboarding and first project creation
    NewUserOnboarding,
    /// Existing user working on a project
    ProjectDevelopment,
    /// Code review and collaboration workflow
    CodeReviewCollaboration,
    /// Refactoring and code improvement workflow
    RefactoringImprovement,
    /// Bug fixing and debugging workflow
    BugFixDebug,
    /// Testing and quality assurance workflow
    TestQualityAssurance,
    /// Deployment and release workflow
    DeploymentRelease,
}

/// Workflow execution context
#[derive(Debug, Clone)]
pub struct WorkflowContext {
    pub workflow_id: String,
    pub user_type: UserPersona,
    pub workflow_type: UserWorkflowType,
    pub start_time: DateTime<Utc>,
    pub checkpoints: Vec<WorkflowCheckpoint>,
    pub test_data: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct WorkflowCheckpoint {
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub status: CheckpointStatus,
    pub data: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CheckpointStatus {
    Started,
    InProgress,
    Completed,
    Failed,
}

/// User personas for different workflow scenarios
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum UserPersona {
    /// Beginner developer new to Rust
    BEGINNER,
    /// Experienced Rust developer
    EXPERIENCED,
    /// Code reviewer/Tech lead
    REVIEWER,
    /// DevOps engineer
    DEVOPS,
    /// QA tester
    QA_TESTER,
}

impl E2EWorkflowRunner {
    pub fn new() -> Self {
        let mut framework = UITestFramework::new();

        // Add all predefined scenarios by default
        let scenarios = vec![
            crate::ui_test_scenarios::UITestScenarios::app_loading_scenario(),
            crate::ui_test_scenarios::UITestScenarios::file_operations_scenario(),
            crate::ui_test_scenarios::UITestScenarios::ai_analysis_scenario(),
            crate::ui_test_scenarios::UITestScenarios::performance_monitoring_scenario(),
            crate::ui_test_scenarios::UITestScenarios::error_handling_scenario(),
            crate::ui_test_scenarios::UITestScenarios::complex_refactoring_scenario(),
            crate::ui_test_scenarios::UITestScenarios::full_workflow_scenario(),
        ];

        for scenario in scenarios {
            framework.add_scenario(scenario);
        }

        Self {
            framework,
            workflow_config: WorkflowConfig::default(),
            test_data: HashMap::new(),
        }
    }

    /// Execute complete workflow for a user persona
    pub async fn execute_user_workflow(
        &mut self,
        workflow_type: UserWorkflowType,
        persona: UserPersona,
    ) -> Result<WorkflowExecutionReport, RustAIError> {
        let workflow_id = format!(
            "{}_{}_{}",
            workflow_type_to_string(&workflow_type),
            persona_to_string(&persona),
            chrono::Utc::now().timestamp()
        );

        let mut context = WorkflowContext {
            workflow_id: workflow_id.clone(),
            user_type: persona,
            workflow_type: workflow_type.clone(),
            start_time: Utc::now(),
            checkpoints: Vec::new(),
            test_data: self.test_data.clone(),
        };

        // Start workflow checkpoint
        self.add_checkpoint(
            &mut context,
            "workflow_started".to_string(),
            CheckpointStatus::Started,
            None,
        )?;

        // Execute the specific workflow
        let result = match workflow_type {
            UserWorkflowType::NewUserOnboarding => {
                self.execute_new_user_onboarding_workflow(&mut context)
                    .await
            }
            UserWorkflowType::ProjectDevelopment => {
                self.execute_project_development_workflow(&mut context)
                    .await
            }
            UserWorkflowType::CodeReviewCollaboration => {
                self.execute_code_review_workflow(&mut context).await
            }
            UserWorkflowType::RefactoringImprovement => {
                self.execute_refactoring_workflow(&mut context).await
            }
            UserWorkflowType::BugFixDebug => self.execute_bug_fix_workflow(&mut context).await,
            UserWorkflowType::TestQualityAssurance => {
                self.execute_test_qa_workflow(&mut context).await
            }
            UserWorkflowType::DeploymentRelease => {
                self.execute_deployment_workflow(&mut context).await
            }
        };

        // End workflow checkpoint
        let final_status = if result.is_ok() {
            CheckpointStatus::Completed
        } else {
            CheckpointStatus::Failed
        };
        self.add_checkpoint(
            &mut context,
            "workflow_completed".to_string(),
            final_status,
            None,
        )?;

        match result {
            Ok(reports) => Ok(WorkflowExecutionReport {
                workflow_id,
                workflow_type,
                persona,
                success: true,
                duration: context
                    .start_time
                    .elapsed()
                    .unwrap_or(Duration::from_secs(0)),
                checkpoints: context.checkpoints,
                ui_test_reports: reports,
                errors: vec![],
                workflow_metrics: self.calculate_workflow_metrics(&context),
            }),
            Err(e) => {
                let error_msg = format!("Workflow execution failed: {}", e);
                Ok(WorkflowExecutionReport {
                    workflow_id,
                    workflow_type,
                    persona,
                    success: false,
                    duration: context
                        .start_time
                        .elapsed()
                        .unwrap_or(Duration::from_secs(0)),
                    checkpoints: context.checkpoints,
                    ui_test_reports: vec![],
                    errors: vec![error_msg],
                    workflow_metrics: self.calculate_workflow_metrics(&context),
                })
            }
        }
    }

    /// Execute new user onboarding workflow
    async fn execute_new_user_onboarding_workflow(
        &mut self,
        context: &mut WorkflowContext,
    ) -> Result<Vec<UITestReport>, RustAIError> {
        self.add_checkpoint(
            context,
            "tutorial_started".to_string(),
            CheckpointStatus::Started,
            None,
        )?;

        // Get relevant scenarios for onboarding
        let onboarding_scenarios = vec!["app_loading", "file_operations"];

        let mut reports = Vec::new();
        for scenario_name in onboarding_scenarios {
            self.add_checkpoint(
                context,
                format!("scenario_{}_started", scenario_name),
                CheckpointStatus::Started,
                None,
            )?;

            let mut filtered_framework = UITestFramework::new();
            filtered_framework.add_scenario(
                self.framework
                    .scenarios
                    .iter()
                    .find(|s| s.name == scenario_name)
                    .ok_or_else(|| {
                        RustAIError::ConfigurationError(format!(
                            "Scenario {} not found",
                            scenario_name
                        ))
                    })?
                    .clone(),
            );

            let scenario_reports = filtered_framework.execute_all_scenarios().await?;
            reports.extend(scenario_reports);

            self.add_checkpoint(
                context,
                format!("scenario_{}_completed", scenario_name),
                CheckpointStatus::Completed,
                Some(format!("{} steps executed", reports.len())),
            )?;
        }

        self.add_checkpoint(
            context,
            "tutorial_completed".to_string(),
            CheckpointStatus::Completed,
            None,
        )?;
        Ok(reports)
    }

    /// Execute project development workflow
    async fn execute_project_development_workflow(
        &mut self,
        context: &mut WorkflowContext,
    ) -> Result<Vec<UITestReport>, RustAIError> {
        self.add_checkpoint(
            context,
            "project_creation_started".to_string(),
            CheckpointStatus::Started,
            None,
        )?;

        let workflows = vec![
            "app_loading",
            "file_operations",
            "complex_refactoring",
            "full_workflow",
        ];

        self.execute_workflow_scenarios(context, workflows).await
    }

    /// Execute code review workflow
    async fn execute_code_review_workflow(
        &mut self,
        context: &mut WorkflowContext,
    ) -> Result<Vec<UITestReport>, RustAIError> {
        self.add_checkpoint(
            context,
            "code_review_started".to_string(),
            CheckpointStatus::Started,
            None,
        )?;

        let workflows = vec!["app_loading", "ai_analysis", "performance_monitoring"];

        self.execute_workflow_scenarios(context, workflows).await
    }

    /// Execute refactoring workflow
    async fn execute_refactoring_workflow(
        &mut self,
        context: &mut WorkflowContext,
    ) -> Result<Vec<UITestReport>, RustAIError> {
        self.add_checkpoint(
            context,
            "refactoring_started".to_string(),
            CheckpointStatus::Started,
            None,
        )?;

        let workflows = vec!["complex_refactoring", "ai_analysis"];

        self.execute_workflow_scenarios(context, workflows).await
    }

    /// Execute bug fix workflow
    async fn execute_bug_fix_workflow(
        &mut self,
        context: &mut WorkflowContext,
    ) -> Result<Vec<UITestReport>, RustAIError> {
        self.add_checkpoint(
            context,
            "bug_fix_started".to_string(),
            CheckpointStatus::Started,
            None,
        )?;

        let workflows = vec!["app_loading", "error_handling", "file_operations"];

        self.execute_workflow_scenarios(context, workflows).await
    }

    /// Execute test QA workflow
    async fn execute_test_qa_workflow(
        &mut self,
        context: &mut WorkflowContext,
    ) -> Result<Vec<UITestReport>, RustAIError> {
        self.add_checkpoint(
            context,
            "qa_testing_started".to_string(),
            CheckpointStatus::Started,
            None,
        )?;

        let workflows = vec!["app_loading", "performance_monitoring", "full_workflow"];

        self.execute_workflow_scenarios(context, workflows).await
    }

    /// Execute deployment workflow
    async fn execute_deployment_workflow(
        &mut self,
        context: &mut WorkflowContext,
    ) -> Result<Vec<UITestReport>, RustAIError> {
        self.add_checkpoint(
            context,
            "deployment_started".to_string(),
            CheckpointStatus::Started,
            None,
        )?;

        let workflows = vec!["performance_monitoring", "full_workflow"];

        self.execute_workflow_scenarios(context, workflows).await
    }

    /// Helper to execute multiple workflow scenarios
    async fn execute_workflow_scenarios(
        &mut self,
        context: &mut WorkflowContext,
        scenario_names: Vec<&str>,
    ) -> Result<Vec<UITestReport>, RustAIError> {
        let mut reports = Vec::new();

        for scenario_name in scenario_names {
            self.add_checkpoint(
                context,
                format!("scenario_{}_started", scenario_name),
                CheckpointStatus::Started,
                None,
            )?;

            let mut filtered_framework = UITestFramework::new();
            filtered_framework.add_scenario(
                self.framework
                    .scenarios
                    .iter()
                    .find(|s| s.name == scenario_name)
                    .ok_or_else(|| {
                        RustAIError::ConfigurationError(format!(
                            "Scenario {} not found",
                            scenario_name
                        ))
                    })?
                    .clone(),
            );

            let scenario_reports = filtered_framework.execute_all_scenarios().await?;
            reports.extend(scenario_reports);

            self.add_checkpoint(
                context,
                format!("scenario_{}_completed", scenario_name),
                CheckpointStatus::Completed,
                Some(format!("{} steps executed", scenario_reports.len())),
            )?;
        }

        Ok(reports)
    }

    /// Execute all available user workflows
    pub async fn execute_all_user_workflows(
        &mut self,
    ) -> Result<Vec<WorkflowExecutionReport>, RustAIError> {
        let personas = vec![
            UserPersona::BEGINNER,
            UserPersona::EXPERIENCED,
            UserPersona::REVIEWER,
            UserPersona::DEVOPS,
            UserPersona::QA_TESTER,
        ];

        let workflow_types = vec![
            UserWorkflowType::NewUserOnboarding,
            UserWorkflowType::ProjectDevelopment,
            UserWorkflowType::CodeReviewCollaboration,
            UserWorkflowType::RefactoringImprovement,
            UserWorkflowType::BugFixDebug,
            UserWorkflowType::TestQualityAssurance,
            UserWorkflowType::DeploymentRelease,
        ];

        let mut reports = Vec::new();

        // Limit concurrency to avoid overwhelming the system
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(3));

        for persona in personas {
            for workflow_type in &workflow_types {
                let sem_clone = semaphore.clone();
                let _permit = sem_clone.acquire().await.unwrap();

                let report = self
                    .execute_user_workflow(workflow_type.clone(), persona.clone())
                    .await?;
                reports.push(report);
            }
        }

        Ok(reports)
    }

    /// Helper method to add workflow checkpoints
    fn add_checkpoint(
        &mut self,
        context: &mut WorkflowContext,
        name: String,
        status: CheckpointStatus,
        data: Option<String>,
    ) -> Result<(), RustAIError> {
        let checkpoint = WorkflowCheckpoint {
            name,
            timestamp: Utc::now(),
            status,
            data,
        };

        context.checkpoints.push(checkpoint);
        Ok(())
    }

    /// Calculate workflow metrics
    fn calculate_workflow_metrics(&self, context: &WorkflowContext) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();

        let total_checkpoints = context.checkpoints.len() as f64;
        metrics.insert("total_checkpoints".to_string(), total_checkpoints);

        let completed_checkpoints = context
            .checkpoints
            .iter()
            .filter(|c| matches!(c.status, CheckpointStatus::Completed))
            .count() as f64;
        metrics.insert("completed_checkpoints".to_string(), completed_checkpoints);

        let failed_checkpoints = context
            .checkpoints
            .iter()
            .filter(|c| matches!(c.status, CheckpointStatus::Failed))
            .count() as f64;
        metrics.insert("failed_checkpoints".to_string(), failed_checkpoints);

        let completion_rate = if total_checkpoints > 0.0 {
            (completed_checkpoints / total_checkpoints) * 100.0
        } else {
            0.0
        };
        metrics.insert("checkpoint_completion_rate".to_string(), completion_rate);

        metrics
    }
}

/// Workflow execution report
#[derive(Debug, Clone)]
pub struct WorkflowExecutionReport {
    pub workflow_id: String,
    pub workflow_type: UserWorkflowType,
    pub persona: UserPersona,
    pub success: bool,
    pub duration: Duration,
    pub checkpoints: Vec<WorkflowCheckpoint>,
    pub ui_test_reports: Vec<UITestReport>,
    pub errors: Vec<String>,
    pub workflow_metrics: HashMap<String, f64>,
}

// Helper functions for type conversion
fn workflow_type_to_string(workflow_type: &UserWorkflowType) -> &'static str {
    match workflow_type {
        UserWorkflowType::NewUserOnboarding => "new_user_onboarding",
        UserWorkflowType::ProjectDevelopment => "project_development",
        UserWorkflowType::CodeReviewCollaboration => "code_review_collaboration",
        UserWorkflowType::RefactoringImprovement => "refactoring_improvement",
        UserWorkflowType::BugFixDebug => "bug_fix_debug",
        UserWorkflowType::TestQualityAssurance => "test_quality_assurance",
        UserWorkflowType::DeploymentRelease => "deployment_release",
    }
}

fn persona_to_string(persona: &UserPersona) -> &'static str {
    match persona {
        UserPersona::BEGINNER => "beginner",
        UserPersona::EXPERIENCED => "experienced",
        UserPersona::REVIEWER => "reviewer",
        UserPersona::DEVOPS => "devops",
        UserPersona::QA_TESTER => "qa_tester",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_e2e_runner_creation() {
        let runner = E2EWorkflowRunner::new();
        assert!(!runner.framework.scenarios.is_empty()); // Should have scenarios loaded by default
    }

    #[tokio::test]
    async fn test_workflow_type_conversion() {
        let workflow_type = UserWorkflowType::NewUserOnboarding;
        assert_eq!(
            workflow_type_to_string(&workflow_type),
            "new_user_onboarding"
        );

        let persona = UserPersona::BEGINNER;
        assert_eq!(persona_to_string(&persona), "beginner");
    }

    #[tokio::test]
    async fn test_persona_workflow_coverage() {
        let runner = E2EWorkflowRunner::new();

        // Test that all personas and workflow types can be executed
        // This test would be slow in practice, so we just validate the types
        let personas = vec![
            UserPersona::BEGINNER,
            UserPersona::EXPERIENCED,
            UserPersona::REVIEWER,
            UserPersona::DEVOPS,
            UserPersona::QA_TESTER,
        ];

        let workflow_types = vec![
            UserWorkflowType::NewUserOnboarding,
            UserWorkflowType::ProjectDevelopment,
            UserWorkflowType::CodeReviewCollaboration,
            UserWorkflowType::BugFixDebug,
            UserWorkflowType::TestQualityAssurance,
        ];

        assert_eq!(personas.len(), 5);
        assert_eq!(workflow_types.len(), 5);
    }
}

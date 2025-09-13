use rust_ai_ide_cargo::build::{BuildError, BuildStatus, ErrorLevel};
use rust_ai_ide_cargo::models::BuildMetrics;

#[test]
fn test_build_status_display() {
    // Test Pending
    assert_eq!(BuildStatus::Pending.to_string(), "Pending");

    // Test Building
    assert_eq!(
        BuildStatus::Building {
            progress:       0.5,
            current_target: Some("my_crate".to_string()),
            jobs_running:   1,
            jobs_total:     2,
        }
        .to_string(),
        "Compiling my_crate (50% - 1/2)"
    );

    // Test Success
    assert_eq!(
        BuildStatus::Success {
            duration: 5000.0,
            metrics:  BuildMetrics {
                warning_count: 0,
                ..Default::default()
            },
        }
        .to_string(),
        "Success (5s)"
    );

    // Test Success with warnings and errors
    assert_eq!(
        BuildStatus::Success {
            duration: 5000.0,
            metrics:  BuildMetrics {
                warning_count: 2,
                ..Default::default()
            },
        }
        .to_string(),
        "Success in 5s (2 warnings)"
    );

    // Test Failed
    assert_eq!(
        BuildStatus::Failed {
            error:         "Build failed".to_string(),
            duration:      10000.0,
            error_details: vec![
                BuildError {
                    message: "Warning 1".to_string(),
                    file:    None,
                    line:    None,
                    column:  None,
                    code:    None,
                    level:   ErrorLevel::Warning,
                },
                BuildError {
                    message: "Warning 2".to_string(),
                    file:    None,
                    line:    None,
                    column:  None,
                    code:    None,
                    level:   ErrorLevel::Warning,
                },
                BuildError {
                    message: "Error message".to_string(),
                    file:    None,
                    line:    None,
                    column:  None,
                    code:    None,
                    level:   ErrorLevel::Error,
                },
            ],
        }
        .to_string(),
        "Failed after 10s: Build failed (2 warnings, 1 errors)"
    );

    // Test Cancelled
    assert_eq!(BuildStatus::Cancelled.to_string(), "Cancelled");
}

#[test]
fn test_build_status_checks() {
    // Test Building variant
    let building = BuildStatus::Building {
        progress:       0.5,
        current_target: Some("my_crate".to_string()),
        jobs_running:   1,
        jobs_total:     2,
    };

    // Test Success variant
    let success = BuildStatus::Success {
        duration: 5000.0,
        metrics:  BuildMetrics {
            warning_count: 0,
            ..Default::default()
        },
    };

    // Test Failed variant
    let failed = BuildStatus::Failed {
        error:         "Build failed".to_string(),
        duration:      10000.0,
        error_details: vec![
            BuildError {
                message: "Warning 1".to_string(),
                file:    None,
                line:    None,
                column:  None,
                code:    None,
                level:   ErrorLevel::Warning,
            },
            BuildError {
                message: "Warning 2".to_string(),
                file:    None,
                line:    None,
                column:  None,
                code:    None,
                level:   ErrorLevel::Warning,
            },
            BuildError {
                message: "Error message".to_string(),
                file:    None,
                line:    None,
                column:  None,
                code:    None,
                level:   ErrorLevel::Error,
            },
        ],
    };

    // Test Cancelled variant
    let cancelled = BuildStatus::Cancelled;

    // Verify Building state
    assert!(building.is_building());
    assert!(!building.is_success());
    assert!(!building.is_failed());
    assert!(!building.is_cancelled());
    assert_eq!(building.duration(), None);
    assert_eq!(building.warnings(), 0);
    assert_eq!(building.errors(), 0);
    assert_eq!(building.progress(), 50);

    // Verify Success state
    assert!(!success.is_building());
    assert!(success.is_success());
    assert!(!success.is_failed());
    assert!(!success.is_cancelled());
    assert_eq!(success.duration(), Some(5000.0));
    assert_eq!(success.warnings(), 0);
    assert_eq!(success.errors(), 0);
    assert_eq!(success.progress(), 100);
    assert!(success.is_success());
    assert!(!success.is_failed());
    assert!(!success.is_cancelled());

    // Verify Failed state
    assert!(!failed.is_building());
    assert!(!failed.is_success());
    assert!(failed.is_failed());
    assert!(!failed.is_cancelled());

    // Verify Cancelled state
    assert!(!cancelled.is_building());
    assert!(!cancelled.is_success());
    assert!(!cancelled.is_failed());
    assert!(cancelled.is_cancelled());

    // Test cancelled status
    assert!(cancelled.is_cancelled());
    assert!(!cancelled.is_success());
    assert!(!cancelled.is_failed());
}

//! Tests for the logging functionality in rust-ai-ide-common

use std::sync::Once;

use rust_ai_ide_common::default_logger::{global_logger, init_default_logger, DefaultLogger};
use rust_ai_ide_common::logging::{LogContext, LogLevel};
use rust_ai_ide_common::{Loggable, LoggableError, UnifiedLogger};

static INIT: Once = Once::new();

fn setup_logger() {
    INIT.call_once(|| {
        // Initialize the logger only once for all tests
        let _ = env_logger::builder().is_test(true).try_init();

        // Initialize our default logger
        let context = LogContext::new("test_runner").with_operation("testing");
        init_default_logger(Some(context));
    });
}

#[tokio::test]
async fn test_basic_logging() {
    setup_logger();
    let logger = global_logger();

    // Test basic logging at different levels
    logger.log_info("This is an info message").await;
    logger.log_debug("This is a debug message").await;
    logger.log_warn("This is a warning message").await;

    // Test error logging
    let error = LoggableError::new("Test error occurred").with_context("test_case", "test_basic_logging");
    logger.log_error("An error occurred", Some(&error)).await;

    // Log with custom context
    let context = LogContext::new("test_component")
        .with_operation("test_operation")
        .with_metadata("test_id", "123");

    logger
        .log_with_level(LogLevel::Info, "Test message with context", Some(&context))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_child_logger() {
    setup_logger();
    let parent_logger = global_logger();

    // Create a child logger with additional context
    let child_context = LogContext::new("child_component", "test_module")
        .with_operation("child_operation")
        .with_metadata("parent_id", "test_parent");

    let child_logger = parent_logger.with_context(child_context);

    // Log using the child logger
    child_logger.log_info("Child logger message").await;

    // Verify the child logger has the correct context
    let context = child_logger.context();
    assert_eq!(context.get_metadata("parent_id"), Some("test_parent"));
}

#[tokio::test]
async fn test_loggable_trait() {
    setup_logger();

    // Create a test struct that implements Loggable
    #[derive(Debug)]
    struct TestComponent {
        name:   String,
        logger: Box<dyn Loggable>,
    }

    impl TestComponent {
        fn new(name: &str) -> Self {
            let logger = global_logger().with_context(LogContext::new("TestComponent").with_metadata("name", name));

            Self {
                name: name.to_string(),
                logger,
            }
        }

        async fn do_something(&self) -> Result<(), LoggableError> {
            self.logger
                .log_info(&format!("Doing something in {}", self.name))
                .await;
            Ok(())
        }
    }

    // Test the component
    let component = TestComponent::new("test_component");
    component.do_something().await.unwrap();
}

#[tokio::test]
async fn test_log_macros() {
    setup_logger();

    // Test the log_with_ctx macro
    let context = LogContext::new("test_macros").with_operation("macro_test");

    rust_ai_ide_common::log_with_ctx!(
        LogLevel::Info,
        "Testing log macro with value: {}",
        42,
        context
    )
    .await
    .unwrap();

    // Test error logging with a result
    let result: Result<(), LoggableError> = Err(LoggableError::new("Test error"));

    if let Err(e) = result.log_error("Operation failed").await {
        // This is expected
        assert_eq!(e.to_string(), "Operation failed: Test error");
    }
}

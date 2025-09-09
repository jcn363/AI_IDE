//! Example demonstrating the usage of DefaultLogger

use rust_ai_ide_common::{
    default_logger::{global_logger, init_default_logger, DefaultLogger},
    logging::{LogContext, LogLevel},
    Loggable, LoggableError, UnifiedLogger,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the default logger with a custom context
    let context = LogContext::new("demo_app")
        .with_operation("startup")
        .with_metadata("version", "1.0.0");

    init_default_logger(Some(context));

    // Get a reference to the global logger
    let logger = global_logger();

    // Log messages at different levels
    logger.log_info("Starting demo application").await;

    // Demonstrate structured logging
    logger
        .log_with_level(
            LogLevel::Info,
            "Processing request",
            Some(
                LogContext::new("request_handler")
                    .with_metadata("request_id", "req_123")
                    .with_metadata("user_id", "user_456"),
            ),
        )
        .await?;

    // Simulate an operation with timing
    let start = std::time::Instant::now();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let duration = start.elapsed();

    logger
        .log_info(&format!("Operation completed in {:?}", duration))
        .await;

    // Demonstrate error handling with logging
    if let Err(e) = fallible_operation().await {
        logger.log_error("Operation failed", Some(&e)).await;
    }

    // Create a child logger with additional context
    let child_logger = logger.with_context(
        LogContext::new("child_component")
            .with_operation("data_processing")
            .with_metadata("batch_size", "1000"),
    );

    // Use the child logger
    child_logger.log_info("Processing batch data").await;

    // Log completion
    logger.log_info("Demo application finished").await;

    Ok(())
}

/// Example function that might fail
async fn fallible_operation() -> Result<(), LoggableError> {
    // Simulate an error condition
    if true {
        // In a real app, this would be some condition
        return Err(LoggableError::new("Example error occurred")
            .with_context("operation", "fallible_operation")
            .with_metadata("retry_count", "3"));
    }

    Ok(())
}

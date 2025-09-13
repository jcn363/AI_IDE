//! Example demonstrating the usage of UnifiedLogger and DefaultLogger

use std::collections::HashMap;

use rust_ai_ide_common::default_logger::{global_logger, init_default_logger, DefaultLogger};
use rust_ai_ide_common::logging::{LogContext, LogLevel};
use rust_ai_ide_common::{Loggable, LoggableError, LoggableResult, UnifiedLogger};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the default logger with a custom context
    let context = LogContext::new("example_app")
        .with_operation("startup")
        .with_metadata("version", "1.0.0");

    init_default_logger(Some(context));

    // Get a reference to the global logger
    let logger = global_logger();

    // Log messages at different levels
    logger.log_info("Application starting up").await;
    logger.log_debug("Debug information").await;
    logger.log_warn("This is a warning").await;

    // Example function that returns a Result with error logging
    if let Err(e) = process_data("test_data").await {
        logger.log_error("Failed to process data", Some(&e)).await;
    }

    // Create a child logger with additional context
    let child_logger = logger.with_context(
        LogContext::new("child_component")
            .with_operation("data_processing")
            .with_metadata("batch_id", "123"),
    );

    // Use the child logger
    child_logger.log_info("Processing batch").await;

    // Example of using the LoggableResult extension
    let result: Result<(), std::io::Error> = Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "File not found",
    ));

    // This will log the error automatically
    let _ = result.log_error("File operation failed").await;

    // Log completion
    logger.log_info("Application shutting down").await;

    Ok(())
}

/// Example function that returns a Result with automatic error logging
async fn process_data(data: &str) -> LoggableResult<()> {
    if data.is_empty() {
        return Err(LoggableError::new("Empty data provided"));
    }

    // Simulate some processing
    log_with_ctx!(
        LogLevel::Info,
        "Processing data: {}",
        data,
        LogContext::new("process_data").with_metadata("data_length", data.len())
    )
    .await;

    Ok(())
}

//! Shared common types and utilities for the Rust AI IDE project
//!
//! This crate provides foundational types, error handling, and common traits
//! that are shared across all other crates in the Rust AI IDE project.
//!
//! ## Re-exports
//!
//! For convenience, the most commonly used types and functions are re-exported at the crate root.
//! You can use them like: `use rust_ai_ide_common::{ProgrammingLanguage, Cache, IdeError};`

pub mod caching;
pub mod duplication;
pub mod errors;
pub mod fs; // Consolidated filesystem utilities
pub mod fs_utils;
pub mod path_utils;
pub mod perf_utils;
pub mod rate_limiting;
pub mod traits;
pub mod types;
pub mod utils;
// Consolidated HTTP client utilities
pub mod http;
pub mod test_utils;
pub mod validation;
// Consolidated async utilities
pub mod async_utils;

// # Rust AI IDE Common Crate
//
// This crate provides shared functionality used across the Rust AI IDE project,
// including logging, error handling, caching, and common types.
//
// ## Logging System
//
// The logging system provides a unified interface for logging across the application.
// It supports multiple log levels, structured logging, and context-aware logging.
//
// ### Basic Usage
//
// ```rust
// use rust_ai_ide_common::default_logger::{global_logger, DefaultLogger};
// use rust_ai_ide_common::logging::LogLevel;
// use rust_ai_ide_common::{Loggable, UnifiedLogger};
//
// #[tokio::main]
// async fn main() {
//     // Create a new logger
//     let logger = DefaultLogger::default();
//
//     // Log messages at different levels
//     logger.log_info("Application started").await;
//     logger.log_warn("This is a warning").await;
//
//     // Log with error context
//     if let Err(e) = some_operation() {
//         logger.log_error("Operation failed", Some(&e)).await;
//     }
//
//     // Use the global logger
//     let global = global_logger();
//     global.log_info("Using global logger").await;
// }
//
// fn some_operation() -> Result<(), std::io::Error> {
//     Err(std::io::Error::new(
//         std::io::ErrorKind::Other,
//         "Example error",
//     ))
// }
// ```
//
// ### Features
// - Multiple log levels (trace, debug, info, warn, error)
// - Structured logging with context
// - Async-aware logging
// - Global logger instance
// - Error logging utilities

// Unified logging infrastructure
pub mod default_logger;
pub mod logging;
pub mod unified_logger;

// Unified configuration management system
pub mod config;

// ===== PLUGIN TYPES =====================================================================
// Re-export plugin-related types for plugin architecture
// ===== CACHE AND PERFORMANCE =================================================
// Re-export cache infrastructure and performance utilities
pub use caching::{Cache, CacheEntry, CacheStats, MemoryCache};
// ===== DUPLICATION DETECTION ===============================================
// Re-export duplication detection and prevention utilities
pub use duplication::{
    calculate_similarity, check_potential_duplication, create_duplication_prevention_template,
    create_safe_function_template, detect_duplications, verify_duplication_free, DuplicationKind, DuplicationResult,
    DuplicationStats, SimilarityMatch,
};
// ===== TYPES =================================================================
// Re-export common types
pub use errors::IdeError;
// Macros are exported via #[macro_export] in validation.rs

// ===== ERRORS ==============================================================
// Re-export error handling types and utilities
pub use errors::{
    convert_error,
    option_to_result,
    safe_unwrap as safe_unwrap_result, // Avoid naming conflict with utils::safe_unwrap
    wrap_result,
    IdeResult,
    IntoIdeError,
    Service,
};
// Re-export consolidated filesystem utilities
pub use fs::{
    append_to_file,

    // File management
    copy_file,
    // Directory operations
    create_dir,
    create_dir_all,
    dir_exists,
    ensure_directory,
    // Utility operations
    ensure_parent_dirs,
    ensure_platform_path,
    exists,
    // File operations
    file_exists,
    get_file_size,
    // Metadata and properties
    get_metadata,
    is_absolute_path,
    is_directory,

    is_file,
    list_files_recursive,
    move_file,
    // Path utilities
    normalize_path,
    parent_directory,
    path_to_string,
    read_dir,
    read_dir_filtered,

    read_file_to_bytes,
    read_file_to_string,
    read_file_with_limit,
    relative_path_from,
    remove_dir,
    remove_dir_all,
    remove_file,

    safe_canonicalize,
    safe_path_join,
    string_to_path,
    temporary_path_in,

    update_file_atomically,
    validate_path,
    watch_file_changes,
    write_bytes_to_file,
    write_string_to_file,
};
// ===== BACKWARD COMPATIBILITY ===============================================
// Re-export legacy functions for backward compatibility
// DEPRECATED: Use the consolidated functions above instead
pub use fs_utils as legacy_fs_utils;
// ===== UTILITIES ===========================================================
// Re-export consolidated HTTP client utilities
pub use http::{
    HttpClient, HttpClientBuilder, HttpConfig, HttpError, HttpRequest, HttpResponse, LoggingInterceptor,
    RequestInterceptor, ResponseInterceptor,
};
pub use logging::{
    example_logging_setup,

    // Main interface
    get_logger,
    init_logging,
    init_logging_from_config,
    instrument_async_operation,
    log_ide_error,
    BasicMetricsCollector,

    ConsoleSink,
    ExternalServiceSink,
    FileSink,
    LogContext,
    LogEntry,
    LogFormat,
    LogLevel,
    // Sinks and collectors (Phase 2 enhanced)
    LogSink,
    LogSinkEnum,
    LogSinkType,

    // Configuration types (Phase 3 enhanced)
    LoggingConfiguration,
    // Error handling
    LoggingError,
    LoggingGuard,
    // Logging managers and guards
    LoggingManager,
    OperationTimer,
};
pub use path_utils as legacy_path_utils;
pub use perf_utils::{
    markers, time_async_operation, time_operation, PerformanceCollector, PerformanceMarker, PerformanceMetrics,
    PerformanceSummary, ScopedTimer, TimedOperation, Timer,
};
// ===== RATE LIMITING =======================================================
// Re-export rate limiting utilities
pub use rate_limiting::RateLimiter;
// ===== TEST UTILITIES ======================================================
// Re-export common test utilities under #[cfg(test)] to avoid production overhead
#[cfg(test)]
pub use test_utils::*;
// ===== TRAITS ===============================================================
// Re-export common trait definitions
pub use traits::{CodeGenerator, Configurable, Loggable, Refreshable, Serializable, Validatable};
// ===== PLUGIN TRAITS =====================================================================
// Re-export plugin-related traits
pub use traits::{Pluggable, Plugin};
// Re-export security types
pub use types::security::*;
pub use types::{
    language_to_lsp_identifier, lsp_identifier_to_language, safe_position_conversion, CodeChange, CoverageType,
    GeneratedTest, GeneratedTests, InternalPosition, InternalRange, LanguageConfig, Location, PluginCapability,
    PluginContext, PluginEvent, PluginEventBus, PluginMessage, PluginMessageType, PluginMetadata, PluginMetrics,
    PluginPerformanceMonitor, Position, ProgrammingLanguage, ProjectContext, Range, RefactoringContext,
    RefactoringResult, RefactoringType, TestCoverage, TestFrameworkInfo, TestGenerationConfig, TestGenerationContext,
    TestType,
};
// ===== UNIFIED LOGGING SYSTEM =============================================
// Re-export unified logging infrastructure for enterprise-grade observability
pub use unified_logger::{LoggableError, LoggableResult, UnifiedLogger};
pub use utils::{
    async_operation, process_stream_concurrent, retry_with_backoff, safe_substring, safe_unwrap, with_timeout,
};

// ===== VALIDATION UTILITIES ================================================
// Re-export validation functions and macros for comprehensive input and path validation
pub use crate::validation::{
    sanitize_file_path,
    // Sanitization utilities
    sanitize_string_for_processing,
    validate_dependency_format,
    validate_directory_exists,
    validate_file_exists,
    validate_file_size_content,
    validate_file_size_path,
    validate_file_size_with_operation,
    validate_path_not_excluded,
    validate_string_input,
    validate_string_input_extended,
    SanitizedQuery,
    ValidatedFilePath,
    // New strong types
    ValidatedString,
};

// Note: production and monitoring submodules are defined within logging::production
// Re-export production and monitoring features from submodules
// Temporarily disabled due to module visibility issues - will be restored after compilation fixes
// pub use logging::production::{
// setup_production_logging,
// HealthChecker,
// HealthStatus,
// AlertManager,
// Alert,
// AlertLevel,
// log_alert,
// HealthCheck,
// };
//
// pub use logging::production::monitoring_dashboard::{
// generate_dashboard_snapshot,
// DashboardSnapshot,
// LogStatistics,
// SystemMetrics,
// AlertInfo,
// PerformanceTrends,
// PerformanceSpike,
// };

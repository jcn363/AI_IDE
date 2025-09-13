//! Structured logging and tracing setup
//!
//! Provides OpenTelemetry-compatible tracing with structured logging,
//! span management, and distributed tracing capabilities.

use crate::{errors::Result, ObservabilityConfig};
use std::env;

/// Tracer for managing distributed tracing
pub struct Tracer {
    config: ObservabilityConfig,
}

impl Tracer {
    /// Create a new tracer instance
    pub fn new(config: ObservabilityConfig) -> Self {
        Self { config }
    }

    /// Initialize global tracing subscriber
    pub fn init(&self) -> Result<()> {
        if !self.config.tracing.enabled {
            return Ok(());
        }

        // Set up tracing subscriber with appropriate level
        let level = match self.config.tracing.level.as_str() {
            "TRACE" => tracing::Level::TRACE,
            "DEBUG" => tracing::Level::DEBUG,
            "INFO" => tracing::Level::INFO,
            "WARN" => tracing::Level::WARN,
            "ERROR" => tracing::Level::ERROR,
            _ => tracing::Level::INFO,
        };

        // Build the subscriber
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(level)
            .with_target(false)
            .with_thread_ids(true)
            .with_thread_names(true)
            .compact();

        #[cfg(feature = "tracing")]
        let subscriber = if self.config.tracing.otel_enabled {
            // Set up OpenTelemetry if enabled
            use tracing_opentelemetry::OpenTelemetryLayer;
            use tracing_subscriber::prelude::*;

            let tracer = self.setup_opentelemetry_tracer()?;
            subscriber.with(OpenTelemetryLayer::new(tracer))
        } else {
            subscriber
        };

        tracing::subscriber::set_global_default(subscriber).map_err(|e| {
            crate::errors::ObservabilityError::tracing(format!(
                "Failed to set global subscriber: {}",
                e
            ))
        })?;

        tracing::info!(
            service = %self.config.tracing.service_name,
            version = %self.config.tracing.service_version,
            level = %self.config.tracing.level,
            "Tracing initialized"
        );

        Ok(())
    }

    /// Set up OpenTelemetry tracer
    #[cfg(feature = "tracing")]
    fn setup_opentelemetry_tracer(&self) -> Result<opentelemetry::sdk::trace::Tracer> {
        use opentelemetry::sdk::trace::{self, Tracer};
        use opentelemetry::sdk::Resource;
        use opentelemetry_semantic_conventions as semconv;

        let mut resource = Resource::new(vec![
            semconv::resource::SERVICE_NAME.string(self.config.tracing.service_name.clone()),
            semconv::resource::SERVICE_VERSION.string(self.config.tracing.service_version.clone()),
        ]);

        if let Some(endpoint) = &self.config.tracing.otel_endpoint {
            // Set up OTLP exporter
            let tracer = opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(
                    opentelemetry_otlp::new_exporter()
                        .http()
                        .with_endpoint(endpoint),
                )
                .with_trace_config(trace::config().with_resource(resource))
                .install_batch(opentelemetry::runtime::Tokio)
                .map_err(|e| {
                    crate::errors::ObservabilityError::tracing(format!(
                        "Failed to create OTLP tracer: {}",
                        e
                    ))
                })?;

            Ok(tracer)
        } else {
            // Fallback to no-op tracer if no endpoint configured
            Ok(opentelemetry::sdk::trace::TracerProvider::builder()
                .with_simple_exporter(opentelemetry::sdk::trace::SpanExporter::builder().build())
                .build()
                .tracer("rust-ai-ide"))
        }
    }

    /// Create a new tracing span
    pub fn create_span(&self, name: &str) -> tracing::Span {
        tracing::info_span!("operation", name = name)
    }

    /// Record an event with structured data
    pub fn record_event(&self, event: &str, fields: &[(&str, &str)]) {
        let mut event_builder = tracing::event!(tracing::Level::INFO, event);

        for (key, value) in fields {
            event_builder = event_builder.with(*key, *value);
        }
    }

    /// Flush any pending tracing data
    pub async fn flush(&self) -> Result<()> {
        #[cfg(feature = "tracing")]
        if self.config.tracing.otel_enabled {
            opentelemetry::global::shutdown_tracer_provider();
        }

        Ok(())
    }
}

/// Convenience macros for common tracing patterns
pub mod macros {
    /// Trace a function call with timing
    #[macro_export]
    macro_rules! trace_function {
        ($func:expr) => {
            let span = tracing::info_span!("function", name = $func);
            let _enter = span.enter();
            tracing::debug!("Calling function: {}", $func);
        };
        ($func:expr, $($fields:tt)*) => {
            let span = tracing::info_span!("function", name = $func, $($fields)*);
            let _enter = span.enter();
            tracing::debug!("Calling function: {}", $func);
        };
    }

    /// Trace an async operation
    #[macro_export]
    macro_rules! trace_async {
        ($operation:expr, $future:expr) => {{
            let span = tracing::info_span!("async_operation", name = $operation);
            let _enter = span.enter();
            tracing::debug!("Starting async operation: {}", $operation);

            let result = $future.await;

            tracing::debug!("Completed async operation: {}", $operation);
            result
        }};
    }

    /// Log a security event
    #[macro_export]
    macro_rules! log_security_event {
        ($event:expr) => {
            tracing::warn!(security_event = true, event = $event, "Security event detected");
        };
        ($event:expr, $($fields:tt)*) => {
            tracing::warn!(security_event = true, event = $event, $($fields)*, "Security event detected");
        };
    }

    /// Log performance metrics
    #[macro_export]
    macro_rules! log_performance {
        ($operation:expr, $duration_ms:expr) => {
            tracing::info!(
                performance = true,
                operation = $operation,
                duration_ms = $duration_ms,
                "Performance measurement"
            );
        };
        ($operation:expr, $duration_ms:expr, $($fields:tt)*) => {
            tracing::info!(
                performance = true,
                operation = $operation,
                duration_ms = $duration_ms,
                $($fields)*,
                "Performance measurement"
            );
        };
    }
}

/// Helper functions for tracing setup
pub mod helpers {
    use super::*;

    /// Initialize tracing from environment variables
    pub fn init_from_env() -> Result<Tracer> {
        let config = crate::ObservabilityConfig::from_env();
        let tracer = Tracer::new(config);
        tracer.init()?;
        Ok(tracer)
    }

    /// Create a child span from the current context
    pub fn child_span(name: &str) -> tracing::Span {
        tracing::info_span!("child", name = name)
    }

    /// Add fields to the current span
    pub fn add_span_fields(fields: &[(&str, &str)]) {
        let span = tracing::Span::current();
        for (key, value) in fields {
            span.record(*key, *value);
        }
    }

    /// Check if tracing is enabled
    pub fn is_tracing_enabled() -> bool {
        env::var("RUST_AI_IDE_TRACING_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true)
    }

    /// Get current tracing level
    pub fn current_level() -> String {
        env::var("RUST_AI_IDE_TRACING_LEVEL").unwrap_or_else(|_| "INFO".to_string())
    }
}

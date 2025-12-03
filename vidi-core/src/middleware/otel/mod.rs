//! Opentelemetry request tracking & metrics middleware.

#[cfg(feature = "otel-metrics")]
pub mod metrics;
#[cfg(feature = "otel-tracing")]
pub mod tracing;

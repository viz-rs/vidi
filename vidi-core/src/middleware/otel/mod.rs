//! Opentelemetry request tracking & metrics middleware.

#[cfg(any(feature = "otel-tracing", feature = "otel-metrics"))]
mod attributes;
#[cfg(any(feature = "otel-tracing", feature = "otel-metrics"))]
pub(crate) use attributes::build_attributes;

#[cfg(feature = "otel-metrics")]
pub mod metrics;
#[cfg(feature = "otel-tracing")]
pub mod tracing;

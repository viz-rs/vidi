//! [OpenTelemetry(OTEL) Prometheus Exporter][OTEL].
//!
//! [OTEL]: https://docs.rs/opentelemetry-prometheus

use http_body_util::Full;
use opentelemetry::{global::handle_error, metrics::MetricsError};
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{Encoder, TextEncoder};

use viz_core::{
    async_trait,
    header::{HeaderValue, CONTENT_TYPE},
    Handler, IntoResponse, Request, Response, Result, StatusCode,
};

#[doc(inline)]
pub use opentelemetry_prometheus::ExporterBuilder;

/// The [`PrometheusExporter`] wrapper.
///
/// [`PrometheusExporter`]: opentelemetry_prometheus::PrometheusExporter
#[derive(Clone, Debug)]
pub struct Prometheus {
    exporter: PrometheusExporter,
}

impl Prometheus {
    /// Creates a new Prometheus.
    #[must_use]
    pub fn new(exporter: PrometheusExporter) -> Self {
        Self { exporter }
    }
}

#[async_trait]
impl Handler<Request> for Prometheus {
    type Output = Result<Response>;

    async fn call(&self, _: Request) -> Self::Output {
        let metric_families = self.exporter.registry().gather();
        let encoder = TextEncoder::new();
        let mut body = Vec::new();

        if let Err(err) = encoder.encode(&metric_families, &mut body) {
            let text = err.to_string();
            handle_error(MetricsError::Other(text.clone()));
            Err((StatusCode::INTERNAL_SERVER_ERROR, text).into_error())?;
        }

        let mut res = Response::new(Full::from(body).into());

        res.headers_mut().append(
            CONTENT_TYPE,
            HeaderValue::from_str(encoder.format_type())
                .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_error())?,
        );

        Ok(res)
    }
}

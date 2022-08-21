//! [OpenTelemetry(OTEL) Prometheus Exporter][OTEL]
//!
//! [OTEL]: https://docs.rs/opentelemetry-prometheus

use opentelemetry_prometheus::{Encoder, PrometheusExporter, TextEncoder};

use viz_core::{
    async_trait,
    header::{HeaderValue, CONTENT_TYPE},
    Handler, IntoResponse, Request, Response, Result, StatusCode,
};

pub use opentelemetry_prometheus::ExporterBuilder;

/// Prometheus Exporter
#[derive(Clone, Debug)]
pub struct Exporter {
    inner: PrometheusExporter,
}

impl From<ExporterBuilder> for Exporter {
    fn from(e: ExporterBuilder) -> Self {
        Exporter { inner: e.init() }
    }
}

#[async_trait]
impl Handler<Request> for Exporter {
    type Output = Result<Response>;

    async fn call(&self, _: Request) -> Self::Output {
        let metric_families = self.inner.registry().gather();
        let encoder = TextEncoder::new();
        let mut body = Vec::new();

        if let Err(err) = encoder.encode(&metric_families, &mut body) {
            let text = err.to_string();
            opentelemetry::global::handle_error(opentelemetry::metrics::MetricsError::Other(
                text.clone(),
            ));
            Err((StatusCode::INTERNAL_SERVER_ERROR, text).into_error())?
        }

        let mut res = Response::new(body.into());

        res.headers_mut().append(
            CONTENT_TYPE,
            HeaderValue::from_str(encoder.format_type())
                .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_error())?,
        );

        Ok(res)
    }
}
